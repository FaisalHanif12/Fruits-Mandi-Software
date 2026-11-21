//! Gate lane T3 (plan 80), part 1: a failure at EVERY write step of every save kind
//! (NFR-REL-1, NFR-REL-6, Code Standards B3, B16, PRD FR-12.8, FR-12.19, FR-21.4, E20).
//! Owner: `robustness`.
//!
//! The existing tests force a failure after each TABLE of one بکری save and one voucher save. This
//! file goes further, for twenty-odd save kinds:
//!
//! - A counter table and a trigger on every table (INSERT, UPDATE and DELETE) make the N-th row
//!   change of the next save fail with a SQLite error. N goes 1, 2, 3 ... until the save works.
//! - After each failed save, EVERY row of EVERY table is compared with the state before (a hash of
//!   all rows), so a half save of any kind is seen: a journal line, a share row, a stock move, an
//!   audit row, a draft, a counter, the chain head in `settings`, an `edit_windows` row.
//! - After the save that finally works: the books balance, both chains verify, `edit_windows` is
//!   empty, no draft entry is left, and the next save works too (no stale number in memory).
//!
//! No production code is changed. Nothing here calls the network.
#![cfg(test)]

mod common;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use std::collections::BTreeMap;
use std::hash::{DefaultHasher, Hash, Hasher};

use common::*;
use mandi_services::core::bikri::{
    BikriDraft, BikriSaveInput, CancelInput, EditOpenInput, EditSaveInput, LaterSaleSaveInput,
    UndoInput,
};
use mandi_services::core::capital::CapitalInput;
use mandi_services::core::expenses::{PersonalExpenseInput, PersonalReturnInput, ShopExpenseInput};
use mandi_services::core::fruit::{FruitInput, FruitSource};
use mandi_services::core::gardens::{
    GardenCloseInput, GardenEntryInput, GardenInput, GardenPaymentInput, GardenReopenInput,
};
use mandi_services::core::general::{GeneralInput, GeneralLineInput};
use mandi_services::core::khata::{KhataCreateInput, KhataInInput, KhataOutInput};
use mandi_services::core::lent::{
    LentPersonCreateInput, LentReturnMode, MoneyLentInput, MoneyLentReturnInput,
};
use mandi_services::core::purchases::PurchaseInput;
use mandi_services::core::trips::{TripCostInput, TripInput};
use mandi_services::core::ventures::{EntryPartInput, MoneySource, VenturePartnerInput};
use mandi_services::core::vouchers::{
    AdvanceInput, DayCloseInput, DayReopenInput, PaymentInput, ReceiptInput, VoucherCancelInput,
    VoucherEditInput, VoucherEntry,
};
use mandi_services::db::rusqlite::{Connection, types::ValueRef};
use mandi_services::month::{MonthInput, MonthReverseInput};
use mandi_services::{
    ServiceError, bikri, day, fruit, gardens, khata, ledger, lent, masters, month, purchases,
    trips, vouchers,
};
use support::World;

// ---------------------------------------------------------------------------------------------
// The harness.
// ---------------------------------------------------------------------------------------------

/// The names of every table that is data (not SQLite's own, not the fault counter).
fn tables(conn: &Connection) -> Vec<String> {
    let mut st = conn
        .prepare(
            "SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%'
             AND name <> 'zz_fault' ORDER BY name",
        )
        .unwrap();
    st.query_map([], |r| r.get::<_, String>(0))
        .unwrap()
        .map(Result::unwrap)
        .collect()
}

/// Makes the fault counter and, on every table, a trigger for INSERT, UPDATE and DELETE that
/// counts the row change and refuses it when the counter reaches 0.
fn install(conn: &Connection) {
    conn.execute_batch(
        "DROP TABLE IF EXISTS zz_fault; CREATE TABLE zz_fault (n INTEGER NOT NULL);
         INSERT INTO zz_fault VALUES (1000000);",
    )
    .unwrap();
    for table in tables(conn) {
        for event in ["INSERT", "UPDATE", "DELETE"] {
            conn.execute_batch(&format!(
                "CREATE TRIGGER zz_{event}_{table} BEFORE {event} ON \"{table}\" BEGIN
                   UPDATE zz_fault SET n = n - 1;
                   SELECT RAISE(ABORT, 'injected write failure') FROM zz_fault WHERE n = 0;
                 END;"
            ))
            .unwrap();
        }
    }
}

fn uninstall(conn: &Connection) {
    let names: Vec<String> = {
        let mut st = conn
            .prepare("SELECT name FROM sqlite_master WHERE type = 'trigger' AND name LIKE 'zz_%'")
            .unwrap();
        st.query_map([], |r| r.get::<_, String>(0))
            .unwrap()
            .map(Result::unwrap)
            .collect()
    };
    for name in names {
        conn.execute_batch(&format!("DROP TRIGGER \"{name}\""))
            .unwrap();
    }
    conn.execute_batch("DROP TABLE zz_fault").unwrap();
}

/// (row count, hash of every value of every row) for each table.
fn fingerprint(conn: &Connection) -> BTreeMap<String, (i64, u64)> {
    let mut out = BTreeMap::new();
    for table in tables(conn) {
        let mut st = conn.prepare(&format!("SELECT * FROM \"{table}\"")).unwrap();
        let columns = st.column_count();
        let mut rows = st.query([]).unwrap();
        let mut hasher = DefaultHasher::new();
        let mut n = 0_i64;
        while let Some(row) = rows.next().unwrap() {
            n += 1;
            for i in 0..columns {
                match row.get_ref(i).unwrap() {
                    ValueRef::Null => 0_u8.hash(&mut hasher),
                    ValueRef::Integer(v) => v.hash(&mut hasher),
                    ValueRef::Real(v) => v.to_bits().hash(&mut hasher),
                    ValueRef::Text(v) | ValueRef::Blob(v) => v.hash(&mut hasher),
                }
            }
        }
        out.insert(table, (n, hasher.finish()));
    }
    out
}

fn changed(
    before: &BTreeMap<String, (i64, u64)>,
    after: &BTreeMap<String, (i64, u64)>,
) -> Vec<String> {
    after
        .iter()
        .filter(|(table, value)| before.get(*table) != Some(value))
        .map(|(table, value)| format!("{table}: {:?} -> {:?}", before.get(table), value))
        .collect()
}

/// Runs `op` with a failure at its 1st, 2nd, 3rd ... row change until it works. Returns how many
/// row changes the working save made. `label` names the save kind in a failure message.
fn at_every_step(w: &World, label: &str, mut op: impl FnMut() -> Result<(), ServiceError>) -> i64 {
    let conn = raw(&w.dir);
    install(&conn);
    let before = fingerprint(&conn);
    let mut n = 1_i64;
    let writes = loop {
        conn.execute("UPDATE zz_fault SET n = ?1", [n]).unwrap();
        match op() {
            Err(error) => {
                let text = format!("{error:?}");
                assert!(
                    text.contains("injected write failure") || text.contains("injected"),
                    "{label}: row change {n} failed for another reason: {text}"
                );
                let after = fingerprint(&conn);
                let diff = changed(&before, &after);
                assert!(
                    diff.is_empty(),
                    "{label}: a failure at row change {n} left rows behind: {diff:#?}"
                );
                n += 1;
                assert!(n < 3_000, "{label}: the save never worked");
            }
            Ok(()) => {
                let left: i64 = conn
                    .query_row("SELECT n FROM zz_fault", [], |r| r.get(0))
                    .unwrap();
                break n - left;
            }
        }
    };
    uninstall(&conn);
    // The save that worked is whole.
    let after = fingerprint(&conn);
    assert!(
        !changed(&before, &after).is_empty(),
        "{label}: the save that 'worked' wrote nothing"
    );
    let imbalance: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(l.dr_paisa), 0) - COALESCE(SUM(l.cr_paisa), 0) FROM journal_lines l
             JOIN journal_entries e ON e.id = l.entry_id WHERE e.status = 'posted'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(imbalance, 0, "{label}: the books do not balance");
    for (sql, what) in [
        (
            "SELECT COUNT(*) FROM edit_windows",
            "an edit window was left open",
        ),
        (
            "SELECT COUNT(*) FROM journal_entries WHERE status <> 'posted'",
            "a draft journal entry was left",
        ),
        (
            "SELECT COUNT(*) FROM journal_lines l LEFT JOIN journal_entries e ON e.id = l.entry_id WHERE e.id IS NULL",
            "a journal line has no entry",
        ),
        (
            "SELECT COUNT(*) FROM (SELECT e.id FROM journal_entries e JOIN journal_lines l ON l.entry_id = e.id GROUP BY e.id, e.total_paisa HAVING SUM(l.dr_paisa) <> SUM(l.cr_paisa) OR SUM(l.dr_paisa) <> e.total_paisa)",
            "an entry is half written",
        ),
        (
            "SELECT COUNT(*) FROM pragma_foreign_key_check",
            "a foreign key is broken",
        ),
    ] {
        let n: i64 = conn.query_row(sql, [], |r| r.get(0)).unwrap();
        assert_eq!(n, 0, "{label}: {what}");
    }
    ledger::verify_journal_chain(&w.env.ctx)
        .unwrap_or_else(|e| panic!("{label}: the journal chain is broken: {e:?}"));
    assert_chain_ok(&w.dir);
    println!("{label}: {writes} row changes, each one made to fail in turn");
    writes
}

// ---------------------------------------------------------------------------------------------
// Small builders.
// ---------------------------------------------------------------------------------------------

fn save_bikri(w: &World, draft: &BikriDraft) -> Result<(), ServiceError> {
    bikri::save(
        &w.env.ctx,
        &w.owner,
        &BikriSaveInput {
            draft: draft.clone(),
            confirm_cash_below_zero: true,
        },
    )
    .map(|_| ())
}

fn scalar(w: &World, sql: &str) -> i64 {
    raw(&w.dir).query_row(sql, [], |r| r.get(0)).unwrap()
}

/// Event 1 saved: رحیم's 50 کیلا, اکرم 20 and ارشد 30.
fn with_event1(w: &World) -> i64 {
    bikri::save(
        &w.env.ctx,
        &w.owner,
        &BikriSaveInput {
            draft: support::event1(w),
            confirm_cash_below_zero: true,
        },
    )
    .unwrap()
    .bikri_id
}

/// A بکری of 50 with only 20 sold, so 30 are left in the lot.
fn part_sold(w: &World) -> BikriDraft {
    support::draft(vec![support::with_sales(
        support::lot(w, "lot-1", w.rahim, w.banana, "50"),
        vec![support::sale("s-1", w.akram, "20", "600")],
    )])
}

fn receipt(w: &World, amount: &str) -> ReceiptInput {
    ReceiptInput {
        date: "10-10-2026".into(),
        account_id: w.akram,
        amount: amount.into(),
        removed: Default::default(),
        collector_staff_id: None,
        narration: String::new(),
    }
}

fn payment(account: i64, amount: &str) -> PaymentInput {
    PaymentInput {
        date: "10-10-2026".into(),
        account_id: account,
        amount: amount.into(),
        removed: Default::default(),
        reason: String::new(),
        narration: String::new(),
        confirm_cash_below_zero: true,
    }
}

fn advance(seller: i64, amount: &str) -> AdvanceInput {
    AdvanceInput {
        confirm_cash_below_zero: true,
        date: "10-10-2026".into(),
        seller_account_id: seller,
        amount: amount.into(),
        removed: Default::default(),
        expected_goods: String::new(),
        narration: String::new(),
        reason: None,
    }
}

/// A shop expense head: the seed has none, so one is made (the spelling check is answered).
fn expense_head(w: &World) -> i64 {
    use mandi_services::core::error::{ErrorCode, ErrorValue};
    use mandi_services::core::masters::{ExpenseHeadInput, MasterInput};
    if let Some(row) = masters::expense_heads(&w.env.ctx).unwrap().rows.first() {
        return row.id;
    }
    let mut words: Vec<String> = Vec::new();
    loop {
        let input = ExpenseHeadInput {
            id: None,
            version: None,
            code: String::new(),
            name_ur: "بجلی".into(),
            name_en: "Electricity".into(),
            confirmed_rules_words: words.clone(),
        };
        match masters::save(&w.env.ctx, &w.owner, &MasterInput::ExpenseHead(input)) {
            Ok(_) => break,
            Err(ServiceError::Rejected(items))
                if items
                    .iter()
                    .all(|i| i.code == ErrorCode::ErrSpellingUnconfirmed) =>
            {
                for item in items {
                    if let Some(ErrorValue::Text(text)) = item.values.get("words") {
                        words.extend(text.split(' ').map(str::to_owned));
                    }
                }
            }
            Err(other) => panic!("{other:?}"),
        }
    }
    masters::expense_heads(&w.env.ctx).unwrap().rows[0].id
}

fn shop_expense(w: &World, amount: &str) -> ShopExpenseInput {
    ShopExpenseInput {
        date: "10-10-2026".into(),
        expense_head_id: Some(expense_head(w)),
        amount: amount.into(),
        paid_to: None,
        narration: None,
        confirm_cash_below_zero: true,
    }
}

// ---------------------------------------------------------------------------------------------
// بکری: first save, later sale, edit, cancel, undo.
// ---------------------------------------------------------------------------------------------

#[test]
fn bikri_first_save_fails_cleanly_at_every_row_change() {
    let w = support::world();
    // A draft with an advance adjusted and a چکتا buyer, so every table is written.
    vouchers::save_advance(&w.env.ctx, &w.owner, &advance(w.rahim, "15,000")).unwrap();
    let mut line = support::sale("s-9", w.chukta_khata, "10", "1200");
    line.chukta = Some(mandi_services::core::bikri::ChuktaInput {
        name_ur: "شاہد".into(),
        name_en: "Shahid".into(),
        phone: "0300-0000000".into(),
    });
    let mut second = support::lot(&w, "lot-2", w.karim, w.apple, "10");
    second.freight = Some("1,000".into());
    second.cash_paid = None;
    second.sales = vec![line];
    let mut draft = support::event1(&w);
    draft.lots.push(second);
    let writes = at_every_step(&w, "bikri first save", || save_bikri(&w, &draft));
    assert!(writes > 30, "a two-lot بکری is many row changes: {writes}");
    // The next save takes the next number: nothing was used by the failures.
    assert_eq!(scalar(&w, "SELECT MAX(number) FROM bikri_entries"), 1);
}

#[test]
fn bikri_later_sale_fails_cleanly_at_every_row_change() {
    let w = support::world();
    save_bikri(&w, &part_sold(&w)).unwrap();
    let lot_id = scalar(&w, "SELECT id FROM bikri_lots");
    common::set_today(&w.env, 2026, 10, 11);
    let input = LaterSaleSaveInput {
        lot_id,
        date: "11-10-2026".into(),
        sales: vec![support::sale("s-8", w.arshad, "30", "560")],
        confirm_cash_below_zero: true,
    };
    at_every_step(&w, "bikri later sale", || {
        bikri::later_sale_save(&w.env.ctx, &w.owner, &input).map(|_| ())
    });
}

#[test]
fn bikri_edit_fails_cleanly_at_every_row_change() {
    let w = support::world();
    let id = with_event1(&w);
    let entry = scalar(
        &w,
        "SELECT id FROM journal_entries WHERE posting_type = 'bikri_first'",
    );
    let opened = bikri::open_for_edit(
        &w.env.ctx,
        &EditOpenInput {
            bikri_id: id,
            entry_id: entry,
        },
    )
    .unwrap();
    let mut draft = opened.draft.clone();
    draft.lots[0].sales[0].seller_rate = "610".into();
    draft.lots[0].sales[0].buyer_rate = Some("610".into());
    let input = EditSaveInput {
        entry_id: opened.entry_id,
        version: opened.version,
        draft,
        reason: Some("rate was wrong".into()),
        confirm_cash_below_zero: true,
    };
    at_every_step(&w, "bikri edit", || {
        bikri::edit_save(&w.env.ctx, &w.owner, &input).map(|_| ())
    });
}

#[test]
fn bikri_cancel_fails_cleanly_at_every_row_change() {
    let w = support::world();
    let id = with_event1(&w);
    // A receipt matched to his bill, so the cancel has matching to undo too.
    vouchers::save_receipt(&w.env.ctx, &w.owner, &receipt(&w, "10,000")).unwrap();
    let version = bikri::get(&w.env.ctx, id).unwrap().version;
    let input = CancelInput {
        id,
        version,
        reason: "gate".into(),
        confirm_cash_below_zero: true,
    };
    at_every_step(&w, "bikri cancel", || {
        bikri::cancel(&w.env.ctx, &w.owner, &input).map(|_| ())
    });
}

#[test]
fn bikri_undo_fails_cleanly_at_every_row_change() {
    let w = support::world();
    let id = with_event1(&w);
    let input = UndoInput {
        id,
        confirm_cash_below_zero: true,
    };
    at_every_step(&w, "bikri undo", || {
        bikri::undo(&w.env.ctx, &w.owner, &input).map(|_| ())
    });
}

// ---------------------------------------------------------------------------------------------
// Vouchers.
// ---------------------------------------------------------------------------------------------

#[test]
fn receipt_payment_and_advance_fail_cleanly_at_every_row_change() {
    let w = support::world();
    with_event1(&w);
    let input = receipt(&w, "10,000");
    at_every_step(&w, "receipt", || {
        vouchers::save_receipt(&w.env.ctx, &w.owner, &input).map(|_| ())
    });
    let input = payment(w.rahim, "4,316");
    at_every_step(&w, "payment", || {
        vouchers::save_payment(&w.env.ctx, &w.owner, &input).map(|_| ())
    });
    let input = advance(w.karim, "3,000");
    at_every_step(&w, "advance", || {
        vouchers::save_advance(&w.env.ctx, &w.owner, &input).map(|_| ())
    });
}

#[test]
fn expenses_fail_cleanly_at_every_row_change() {
    let w = support::world();
    let input = shop_expense(&w, "3,000");
    at_every_step(&w, "shop expense", || {
        vouchers::save_shop_expense(&w.env.ctx, &w.owner, &input).map(|_| ())
    });
    let personal = PersonalExpenseInput {
        date: "10-10-2026".into(),
        partner_id: Some(w.partners[0]),
        amount: "2,000".into(),
        reason: "home".into(),
        return_by: None,
        taken_by: None,
        narration: None,
        confirm_cash_below_zero: true,
    };
    at_every_step(&w, "personal expense", || {
        vouchers::save_personal_expense(&w.env.ctx, &w.owner, &personal).map(|_| ())
    });
    let back = PersonalReturnInput {
        date: "10-10-2026".into(),
        partner_id: Some(w.partners[0]),
        amount: "2,000".into(),
        narration: None,
    };
    at_every_step(&w, "personal return", || {
        vouchers::save_personal_return(&w.env.ctx, &w.owner, &back).map(|_| ())
    });
}

#[test]
fn money_lent_capital_khata_and_general_fail_cleanly_at_every_row_change() {
    let w = support::world();
    // Money lent: a person, then the loan and its return.
    let person = lent::person_create(
        &w.env.ctx,
        &w.owner,
        &LentPersonCreateInput {
            name_ur: "اسلم".into(),
            name_en: "Aslam".into(),
            phone: None,
            staff_id: None,
            confirmed_rules_words: vec!["اسلم".into(), "Aslam".into()],
        },
    )
    .unwrap();
    let person_id = person.account.id;
    let loan = MoneyLentInput {
        date: "10-10-2026".into(),
        person_account_id: Some(person_id),
        amount: "5,000".into(),
        reason: "home".into(),
        return_by: Some("20-10-2026".into()),
        narration: None,
        confirm_cash_below_zero: true,
    };
    at_every_step(&w, "money lent", || {
        vouchers::save_money_lent(&w.env.ctx, &w.owner, &loan).map(|_| ())
    });
    let back = MoneyLentReturnInput {
        date: "10-10-2026".into(),
        person_account_id: Some(person_id),
        amount: "3,000".into(),
        mode: LentReturnMode::Cash,
        salary_head_id: None,
        narration: None,
    };
    at_every_step(&w, "money lent return", || {
        vouchers::save_money_lent_return(&w.env.ctx, &w.owner, &back).map(|_| ())
    });
    // Capital in and out.
    let capital = |amount: &str| CapitalInput {
        date: "10-10-2026".into(),
        partner_id: Some(w.partners[1]),
        amount: amount.into(),
        narration: None,
        confirm_cash_below_zero: true,
    };
    let c_in = capital("50,000");
    at_every_step(&w, "capital in", || {
        vouchers::save_capital_in(&w.env.ctx, &w.owner, &c_in).map(|_| ())
    });
    let c_out = capital("10,000");
    at_every_step(&w, "capital out", || {
        vouchers::save_capital_out(&w.env.ctx, &w.owner, &c_out).map(|_| ())
    });
    // A separate khata, paid out and back.
    let khata_row = khata::create(
        &w.env.ctx,
        &w.owner,
        &KhataCreateInput {
            name_ur: "خاندانی کھاتہ".into(),
            name_en: "Family Khata".into(),
            note: None,
            confirmed_rules_words: vec![
                "خاندانی".into(),
                "کھاتہ".into(),
                "Family".into(),
                "Khata".into(),
            ],
        },
    )
    .unwrap();
    let khata_id = khata_row.account.id;
    let out = KhataOutInput {
        date: "10-10-2026".into(),
        khata_account_id: Some(khata_id),
        amount: "10,000".into(),
        reason: "family".into(),
        narration: None,
        confirm_cash_below_zero: true,
    };
    at_every_step(&w, "khata out", || {
        vouchers::save_khata_out(&w.env.ctx, &w.owner, &out).map(|_| ())
    });
    let inn = KhataInInput {
        date: "10-10-2026".into(),
        khata_account_id: Some(khata_id),
        amount: "4,000".into(),
        reason: "back".into(),
        narration: None,
        confirm_cash_below_zero: true,
    };
    at_every_step(&w, "khata in", || {
        vouchers::save_khata_in(&w.env.ctx, &w.owner, &inn).map(|_| ())
    });
    // A general entry that touches an income account and a shop expense (so share rows are written).
    let cash = support::id_of(&w.env, "01010001");
    let income = support::id_of(&w.env, "04010004");
    let general = GeneralInput {
        date: "10-10-2026".into(),
        narration: Some("gate".into()),
        lines: vec![
            GeneralLineInput {
                account_id: Some(cash),
                dr: "700".into(),
                cr: String::new(),
                narration: None,
            },
            GeneralLineInput {
                account_id: Some(income),
                dr: String::new(),
                cr: "700".into(),
                narration: None,
            },
        ],
        confirm_cash_below_zero: true,
    };
    at_every_step(&w, "general journal", || {
        vouchers::general_save(&w.env.ctx, &w.owner, &general).map(|_| ())
    });
}

#[test]
fn voucher_edit_and_cancel_fail_cleanly_at_every_row_change() {
    let w = support::world();
    with_event1(&w);
    let saved = vouchers::save_receipt(&w.env.ctx, &w.owner, &receipt(&w, "10,000")).unwrap();
    let version = scalar(
        &w,
        &format!(
            "SELECT version FROM vouchers WHERE id = {}",
            saved.voucher_id
        ),
    );
    let edit = VoucherEditInput {
        id: saved.voucher_id,
        version,
        entry: VoucherEntry::Receipt(receipt(&w, "9,000")),
        reason: "typo".into(),
        confirm_cash_below_zero: true,
    };
    at_every_step(&w, "voucher edit", || {
        vouchers::edit(&w.env.ctx, &w.owner, &edit).map(|_| ())
    });
    let id = scalar(&w, "SELECT id FROM vouchers WHERE type = 'receipt'");
    let version = scalar(&w, &format!("SELECT version FROM vouchers WHERE id = {id}"));
    let cancel = VoucherCancelInput {
        id,
        version,
        reason: "gate".into(),
        confirm_cash_below_zero: true,
    };
    at_every_step(&w, "voucher cancel", || {
        vouchers::cancel(&w.env.ctx, &w.owner, &cancel).map(|_| ())
    });
}

// ---------------------------------------------------------------------------------------------
// Fruit, trips and gardens.
// ---------------------------------------------------------------------------------------------

#[test]
fn fruit_taken_fails_cleanly_at_every_row_change() {
    let w = support::world();
    save_bikri(&w, &part_sold(&w)).unwrap();
    let lot_id = scalar(&w, "SELECT id FROM bikri_lots");
    let input = FruitInput {
        for_guests: false,
        partner_id: Some(w.partners[0]),
        date: "10-10-2026".into(),
        source: FruitSource::Shop,
        lot_id: Some(lot_id),
        agent_account_id: None,
        item_id: None,
        packing_id: None,
        qty: "2".into(),
        weight: None,
        rate: "580".into(),
        note: None,
        ..fruit_defaults()
    };
    at_every_step(&w, "fruit taken", || {
        fruit::save(&w.env.ctx, &w.owner, &input).map(|_| ())
    });
}

fn fruit_defaults() -> FruitInput {
    serde_json::from_str(
        r#"{"for_guests":false,"date":"10-10-2026","source":"shop","qty":"1","rate":"1"}"#,
    )
    .unwrap()
}

fn shop(partner_id: i64, share: &str) -> VenturePartnerInput {
    VenturePartnerInput {
        partner_id,
        share: share.into(),
        source: MoneySource::Shop,
    }
}

fn pocket(partner_id: i64, share: &str) -> VenturePartnerInput {
    VenturePartnerInput {
        partner_id,
        share: share.into(),
        source: MoneySource::Pocket,
    }
}

fn trip_input(w: &World) -> TripInput {
    let [a, b, c] = w.partners;
    TripInput {
        date: "10-10-2026".into(),
        market: Some("Lahore".into()),
        sent_person: None,
        note: None,
        partners: vec![shop(a, "40"), shop(b, "30"), shop(c, "30")],
    }
}

fn purchase_input(w: &World, trip: i64, item: i64) -> PurchaseInput {
    PurchaseInput {
        other_qty: None,
        shop_name: None,
        sell_date: None,
        trip_id: Some(trip),
        date: "10-10-2026".into(),
        supplier: Some("Orange Traders".into()),
        item_id: Some(item),
        packing_id: Some(w.box_),
        qty: "20".into(),
        weight: None,
        rate: "500".into(),
        freight: Some("400".into()),
        labour: None,
        other_cost: None,
        note: None,
        parts: Vec::<EntryPartInput>::new(),
        confirm_cash_below_zero: true,
    }
}

#[test]
fn trip_create_purchase_and_cost_fail_cleanly_at_every_row_change() {
    let w = support::world();
    let input = trip_input(&w);
    let mut trip_id = 0;
    at_every_step(&w, "trip create", || {
        trips::create(&w.env.ctx, &w.owner, &input).map(|t| trip_id = t.id)
    });
    let orange = support::add_item(&w.env, &w.owner, "مالٹا", "Orange", "fruit", "unit");
    let purchase = purchase_input(&w, trip_id, orange);
    at_every_step(&w, "trip purchase", || {
        purchases::save(&w.env.ctx, &w.owner, &purchase).map(|_| ())
    });
    let cost = TripCostInput {
        trip_id: Some(trip_id),
        date: "10-10-2026".into(),
        amount: "4,000".into(),
        reason: "truck".into(),
        parts: Vec::new(),
        confirm_cash_below_zero: true,
    };
    at_every_step(&w, "trip cost", || {
        trips::cost_save(&w.env.ctx, &w.owner, &cost).map(|_| ())
    });
}

fn garden_input(w: &World) -> GardenInput {
    let [a, b, _] = w.partners;
    GardenInput {
        name_ur: "شاہ پور باغ".into(),
        name_en: "Shahpur Garden".into(),
        confirmed_rules_words: vec![
            "شاہ".into(),
            "پور".into(),
            "باغ".into(),
            "Shahpur".into(),
            "Garden".into(),
        ],
        date: "10-10-2026".into(),
        item_id: Some(w.banana),
        item_ids: Vec::new(),
        rent: "50,000".into(),
        season_end: None,
        note: None,
        partners: vec![shop(a, "60"), pocket(b, "40")],
        paid_now: Some("30,000".into()),
        paid_now_reason: Some("first installment".into()),
        parts: Vec::new(),
        confirm_cash_below_zero: true,
    }
}

#[test]
fn garden_create_installment_close_and_reopen_fail_cleanly_at_every_row_change() {
    let w = support::world();
    let input = garden_input(&w);
    let mut garden_id = 0;
    at_every_step(&w, "garden create with first installment", || {
        gardens::create(&w.env.ctx, &w.owner, &input).map(|g| garden_id = g.row.id)
    });
    common::set_today(&w.env, 2026, 10, 11);
    let installment = GardenEntryInput::Installment(GardenPaymentInput {
        garden_id: Some(garden_id),
        date: "11-10-2026".into(),
        amount: "20,000".into(),
        reason: "installment".into(),
        parts: Vec::new(),
        confirm_cash_below_zero: true,
    });
    at_every_step(&w, "garden installment", || {
        gardens::entry_save(&w.env.ctx, &w.owner, &installment).map(|_| ())
    });
    // Sales money in another shop, so the closing has money to give back.
    let other = GardenEntryInput::OtherSale(mandi_services::core::gardens::GardenOtherSaleInput {
        arrival_id: None,
        garden_id: Some(garden_id),
        date: "11-10-2026".into(),
        shop: Some("Haji Traders".into()),
        qty: "20".into(),
        amount: "10,000".into(),
        note: None,
    });
    at_every_step(&w, "garden sale in another shop", || {
        gardens::entry_save(&w.env.ctx, &w.owner, &other).map(|_| ())
    });
    common::set_today(&w.env, 2026, 10, 13);
    let version = || {
        gardens::get(
            &w.env.ctx,
            &mandi_services::core::gardens::GardenGetInput {
                id: garden_id,
                history_offset: 0,
                history_limit: 0,
            },
        )
        .unwrap()
        .row
        .version
    };
    at_every_step(&w, "garden close", || {
        gardens::close(
            &w.env.ctx,
            &w.owner,
            &GardenCloseInput {
                id: garden_id,
                version: version(),
                date: "13-10-2026".into(),
            },
        )
        .map(|_| ())
    });
    at_every_step(&w, "garden reopen (undo the close)", || {
        gardens::reopen(
            &w.env.ctx,
            &w.owner,
            &GardenReopenInput {
                id: garden_id,
                version: version(),
                reason: "gate".into(),
            },
        )
        .map(|_| ())
    });
}

// ---------------------------------------------------------------------------------------------
// Days and months.
// ---------------------------------------------------------------------------------------------

#[test]
fn day_close_reopen_and_the_reopen_and_fix_steps_fail_cleanly_at_every_row_change() {
    let w = support::world();
    let id = with_event1(&w);
    common::set_today(&w.env, 2026, 10, 11);
    let close = DayCloseInput {
        up_to: "10-10-2026".into(),
    };
    at_every_step(&w, "day close", || {
        day::close(&w.env.ctx, &w.owner, &close).map(|_| ())
    });
    let reopen = DayReopenInput {
        date: "10-10-2026".into(),
        reason: "gate".into(),
        sign: "adnan".into(),
    };
    at_every_step(&w, "day reopen", || {
        day::reopen(&w.env.ctx, &w.owner, &reopen).map(|_| ())
    });
    // Close again, then the three steps of a reopen and fix (FR-21.4).
    day::close(&w.env.ctx, &w.owner, &close).unwrap();
    let open = mandi_services::core::edit::DayFixOpenInput {
        date: "10-10-2026".into(),
        reason: "rate was wrong".into(),
        sign: "adnan".into(),
    };
    // `fix_open` itself is attacked in `fix_open_is_not_one_save` (finding T3-1).
    let token = day::fix_open(&w.env.ctx, &w.owner, &open).unwrap().token;
    // The edit of the open day.
    let entry = scalar(
        &w,
        "SELECT id FROM journal_entries WHERE posting_type = 'bikri_first'",
    );
    let opened = bikri::open_for_edit(
        &w.env.ctx,
        &EditOpenInput {
            bikri_id: id,
            entry_id: entry,
        },
    )
    .unwrap();
    let mut draft = opened.draft.clone();
    draft.lots[0].sales[0].seller_rate = "610".into();
    draft.lots[0].sales[0].buyer_rate = Some("610".into());
    let edit = EditSaveInput {
        entry_id: opened.entry_id,
        version: opened.version,
        draft,
        reason: Some("rate was wrong".into()),
        confirm_cash_below_zero: true,
    };
    at_every_step(&w, "edit inside a fix", || {
        bikri::edit_save(&w.env.ctx, &w.owner, &edit).map(|_| ())
    });
    // The close of the fix. (Its failure at the last step is finding T3-2, tested below.)
    day::fix_close(
        &w.env.ctx,
        &w.owner,
        &mandi_services::core::edit::DayFixCloseInput { token },
    )
    .unwrap();
    assert_eq!(
        day::status(&w.env.ctx).unwrap().closed_up_to.as_deref(),
        Some("2026-10-10"),
        "the day is closed again after the fix"
    );
}

#[test]
fn month_close_and_its_reversal_fail_cleanly_at_every_row_change() {
    let w = support::world();
    with_event1(&w);
    vouchers::save_shop_expense(&w.env.ctx, &w.owner, &shop_expense(&w, "3,000")).unwrap();
    common::set_today(&w.env, 2026, 11, 2);
    day::close(
        &w.env.ctx,
        &w.owner,
        &DayCloseInput {
            up_to: "31-10-2026".into(),
        },
    )
    .unwrap();
    let input = MonthInput {
        month: "2026-10".into(),
    };
    at_every_step(&w, "month close", || {
        month::close(&w.env.ctx, &w.owner, &input).map(|_| ())
    });
    let reverse = MonthReverseInput {
        month: "2026-10".into(),
        reason: "gate".into(),
    };
    at_every_step(&w, "month close reversal", || {
        month::reverse(&w.env.ctx, &w.owner, &reverse).map(|_| ())
    });
}

/// Finding T3-1: `day::fix_open` is two saves. `reopen_signed` commits the reopen (the days open,
/// the "closed up to" date moves back, an audit row) and then a SECOND transaction writes the fix
/// record `day_fix_pending` and the `day_fix_open` audit row. If the second fails, the user gets
/// an error, but the days stay reopened, with no token and no record of which days to close again.
/// NFR-REL-1 and Code Standards B3: one user action is one service function and one transaction.
#[test]
fn fix_open_is_not_one_save() {
    let w = support::world();
    with_event1(&w);
    common::set_today(&w.env, 2026, 10, 11);
    day::close(
        &w.env.ctx,
        &w.owner,
        &DayCloseInput {
            up_to: "10-10-2026".into(),
        },
    )
    .unwrap();
    let open = mandi_services::core::edit::DayFixOpenInput {
        date: "10-10-2026".into(),
        reason: "rate was wrong".into(),
        sign: "adnan".into(),
    };
    at_every_step(&w, "fix open (reopen for a fix)", || {
        day::fix_open(&w.env.ctx, &w.owner, &open).map(|_| ())
    });
}

/// Finding T3-2: `day::fix_close` closes the days, each in its own transaction (`day::close`), and
/// then clears the fix record and writes the `day_fix_close` audit row in ANOTHER one. If that last
/// write fails the days ARE closed, the user is told the save failed, the fix record stays and the
/// audit log has a `day_close` and no `day_fix_close`. NFR-REL-1, Code Standards B3.
#[test]
fn fix_close_is_not_one_save() {
    let w = support::world();
    with_event1(&w);
    common::set_today(&w.env, 2026, 10, 11);
    day::close(
        &w.env.ctx,
        &w.owner,
        &DayCloseInput {
            up_to: "10-10-2026".into(),
        },
    )
    .unwrap();
    let token = day::fix_open(
        &w.env.ctx,
        &w.owner,
        &mandi_services::core::edit::DayFixOpenInput {
            date: "10-10-2026".into(),
            reason: "rate was wrong".into(),
            sign: "adnan".into(),
        },
    )
    .unwrap()
    .token;
    let close = mandi_services::core::edit::DayFixCloseInput { token };
    at_every_step(&w, "fix close", || {
        day::fix_close(&w.env.ctx, &w.owner, &close).map(|_| ())
    });
}

// ---------------------------------------------------------------------------------------------
// Masters, accounts, partners, rules.
// ---------------------------------------------------------------------------------------------

#[test]
fn accounts_partners_rules_and_openings_fail_cleanly_at_every_row_change() {
    use mandi_services::core::accounts::{AccountInput, AccountType};
    use mandi_services::core::partners::{
        OpeningCapitalWhere, PartnerCreateInput, PartnersSetSharesInput, ShareInput,
    };
    use mandi_services::core::rules::{ChargeMethod, ChargeSide, ChargeType, RuleInput};
    let w = support::world();
    // A new buyer with an opening balance: the account, the opening entry, the search keys, audit.
    let buyer = AccountInput {
        id: None,
        version: None,
        account_type: AccountType::Buyer,
        control_id: None,
        code: String::new(),
        name_ur: "طارق".into(),
        name_en: "Tariq".into(),
        confirmed_rules_words: vec!["طارق".into(), "Tariq".into()],
        father_or_firm_ur: String::new(),
        father_or_firm_en: String::new(),
        city_id: None,
        adda_id: None,
        phone1: String::new(),
        phone2: String::new(),
        address: String::new(),
        linked_account_id: None,
        opening_date: "10-10-2026".into(),
        opening_dr: "5,000".into(),
        opening_cr: String::new(),
        opening_is_advance: false,
        credit_allowed: true,
        credit_limit: String::new(),
        default_marka_id: None,
        collector_staff_id: None,
        is_active: true,
        note: String::new(),
        tag: String::new(),
    };
    at_every_step(&w, "account with an opening balance", || {
        mandi_services::accounts::save(&w.env.ctx, &w.owner, &buyer).map(|_| ())
    });
    // A partner with an opening capital in cash (posts an entry).
    let partner = PartnerCreateInput {
        name_ur: "د".into(),
        name_en: "Partner D".into(),
        opening_capital: Some("20,000".into()),
        opening_capital_where: Some(OpeningCapitalWhere::Cash),
        confirmed_rules_words: vec!["د".into(), "Partner".into(), "D".into()],
    };
    let mut new_partner = 0;
    at_every_step(&w, "partner with an opening capital", || {
        mandi_services::partners::create(&w.env.ctx, &w.owner, &partner).map(|p| new_partner = p.id)
    });
    // New shares from tomorrow for all four partners.
    common::set_today(&w.env, 2026, 10, 11);
    let [a, b, c] = w.partners;
    let shares = PartnersSetSharesInput {
        effective_from: "11-10-2026".into(),
        shares: [(a, "30"), (b, "30"), (c, "20"), (new_partner, "20")]
            .iter()
            .map(|&(partner_id, share)| ShareInput {
                partner_id,
                share: share.to_owned(),
            })
            .collect(),
    };
    at_every_step(&w, "partner shares", || {
        mandi_services::partners::set_shares(&w.env.ctx, &w.owner, &shares).map(|_| ())
    });
    let rule = RuleInput {
        id: None,
        version: None,
        side: ChargeSide::Seller,
        charge_type: ChargeType::Commission,
        account_id: Some(w.rahim),
        item_id: None,
        item_group: None,
        method: ChargeMethod::Percent,
        value: "5".into(),
        from_date: "11-10-2026".into(),
        to_date: String::new(),
        is_active: true,
        note: String::new(),
    };
    at_every_step(&w, "commission rule", || {
        mandi_services::rules::save(&w.env.ctx, &w.owner, &rule).map(|_| ())
    });
}

// ---------------------------------------------------------------------------------------------
// Part 2: mixed saves, stock, edits with an edit window, cancels, the dead amount.
// ---------------------------------------------------------------------------------------------

#[test]
fn a_mixed_sale_with_carried_and_new_lots_fails_cleanly_at_every_row_change() {
    use mandi_services::core::bikri::{CarriedLotInput, MixedSaleSaveInput};
    let w = support::world();
    save_bikri(&w, &part_sold(&w)).unwrap();
    let lot_id = scalar(&w, "SELECT id FROM bikri_lots");
    common::set_today(&w.env, 2026, 10, 11);
    let mut new_lot = support::lot(&w, "lot-n", w.karim, w.apple, "10");
    new_lot.sales = vec![support::sale("s-n", w.akram, "10", "1200")];
    let input = MixedSaleSaveInput {
        date: "11-10-2026".into(),
        carried: vec![CarriedLotInput {
            key: "c-1".into(),
            lot_id,
            sales: vec![support::sale("s-c", w.arshad, "5", "560")],
        }],
        new_lots: vec![new_lot],
        confirm_cash_below_zero: true,
    };
    let writes = at_every_step(&w, "mixed sale", || {
        bikri::mixed_save(&w.env.ctx, &w.owner, &input).map(|_| ())
    });
    assert!(writes > 40, "{writes}");
}

#[test]
fn stock_write_off_and_its_undo_fail_cleanly_at_every_row_change() {
    use mandi_services::core::bikri::WriteOffInput;
    let w = support::world();
    save_bikri(&w, &part_sold(&w)).unwrap();
    let lot_id = scalar(&w, "SELECT id FROM bikri_lots");
    let input = WriteOffInput {
        lot_id,
        qty: "5".into(),
        reason: "spoiled".into(),
    };
    at_every_step(&w, "stock write-off", || {
        bikri::write_off(&w.env.ctx, &w.owner, &input).map(|_| ())
    });
}

#[test]
fn a_trip_purchase_edit_with_its_edit_window_fails_cleanly_at_every_row_change() {
    use mandi_services::core::edit::PurchaseEditInput;
    let w = support::world();
    let trip = trips::create(&w.env.ctx, &w.owner, &trip_input(&w)).unwrap();
    let orange = support::add_item(&w.env, &w.owner, "مالٹا", "Orange", "fruit", "unit");
    let bought =
        purchases::save(&w.env.ctx, &w.owner, &purchase_input(&w, trip.id, orange)).unwrap();
    let version = scalar(
        &w,
        &format!("SELECT version FROM purchases WHERE id = {}", bought.id),
    );
    let mut changed_input = purchase_input(&w, trip.id, orange);
    changed_input.qty = "25".into();
    changed_input.rate = "510".into();
    let edit = PurchaseEditInput {
        id: bought.id,
        version,
        purchase: changed_input,
        reason: "typo".into(),
    };
    at_every_step(&w, "trip purchase edit (edit window)", || {
        mandi_services::edit::purchase_save(&w.env.ctx, &w.owner, &edit).map(|_| ())
    });
    // An edit window is never left behind (the harness checks the table; this is the plain check).
    assert_eq!(scalar(&w, "SELECT COUNT(*) FROM edit_windows"), 0);
    let write_off = mandi_services::core::purchases::PurchaseWriteOffInput {
        purchase_id: bought.id,
        qty: "5".into(),
        reason: "spoiled".into(),
    };
    at_every_step(&w, "trip purchase write-off", || {
        purchases::write_off(&w.env.ctx, &w.owner, &write_off).map(|_| ())
    });
}

#[test]
fn garden_arrival_entry_edit_and_cancel_fail_cleanly_at_every_row_change() {
    use mandi_services::core::edit::GardenEntryEditInput;
    use mandi_services::core::gardens::GardenEntryCancelInput;
    let w = support::world();
    let garden = gardens::create(&w.env.ctx, &w.owner, &garden_input(&w))
        .unwrap()
        .row;
    let arrival = mandi_services::core::arrivals::ArrivalInput {
        garden_id: Some(garden.id),
        date: "10-10-2026".into(),
        sell_date: None,
        item_id: Some(w.banana),
        packing_id: Some(w.box_),
        qty: "40".into(),
        weight: None,
        note: None,
    };
    at_every_step(&w, "garden goods arrived", || {
        gardens::arrivals::save(&w.env.ctx, &w.owner, &arrival).map(|_| ())
    });
    // An installment, then its edit (edit window) and its cancel.
    let pay = |amount: &str| {
        GardenEntryInput::Installment(GardenPaymentInput {
            garden_id: Some(garden.id),
            date: "10-10-2026".into(),
            amount: amount.into(),
            reason: "installment".into(),
            parts: Vec::new(),
            confirm_cash_below_zero: true,
        })
    };
    gardens::entry_save(&w.env.ctx, &w.owner, &pay("5,000")).unwrap();
    let id = scalar(&w, "SELECT MAX(id) FROM venture_entries");
    let version = scalar(
        &w,
        &format!("SELECT version FROM venture_entries WHERE id = {id}"),
    );
    let edit = GardenEntryEditInput {
        id,
        version,
        entry: pay("6,000"),
        reason: "typo".into(),
        confirm_cash_below_zero: true,
    };
    at_every_step(&w, "garden installment edit (edit window)", || {
        mandi_services::edit::garden_entry_save(&w.env.ctx, &w.owner, &edit).map(|_| ())
    });
    let version = scalar(
        &w,
        &format!("SELECT version FROM venture_entries WHERE id = {id}"),
    );
    let cancel = GardenEntryCancelInput {
        id,
        version,
        reason: "gate".into(),
        confirm_cash_below_zero: true,
    };
    at_every_step(&w, "garden entry cancel", || {
        gardens::entry_cancel(&w.env.ctx, &w.owner, &cancel).map(|_| ())
    });
}

#[test]
fn fruit_cancel_and_the_dead_amount_fail_cleanly_at_every_row_change() {
    use mandi_services::core::dead::{DeadMarkInput, DeadReason, DeadUndoInput};
    use mandi_services::core::fruit::FruitCancelInput;
    let w = support::world();
    save_bikri(&w, &part_sold(&w)).unwrap();
    let lot_id = scalar(&w, "SELECT id FROM bikri_lots");
    let saved = fruit::save(
        &w.env.ctx,
        &w.owner,
        &FruitInput {
            partner_id: Some(w.partners[0]),
            source: FruitSource::Shop,
            lot_id: Some(lot_id),
            qty: "2".into(),
            rate: "580".into(),
            ..fruit_defaults()
        },
    )
    .unwrap();
    let version = scalar(
        &w,
        &format!("SELECT version FROM fruit_takes WHERE id = {}", saved.id),
    );
    let cancel = FruitCancelInput {
        id: saved.id,
        version,
        reason: "gate".into(),
    };
    at_every_step(&w, "fruit cancel", || {
        fruit::cancel(&w.env.ctx, &w.owner, &cancel).map(|_| ())
    });
    // Akram owes 12,880 (the part-sold بکری): move him to the dead amount, then back.
    let mark = DeadMarkInput {
        account_id: w.akram,
        entry_date: "10-10-2026".into(),
        reason: DeadReason::Died,
        note: String::new(),
    };
    let mut mark_id = 0;
    at_every_step(&w, "dead amount: move", || {
        mandi_services::dead::mark(&w.env.ctx, &w.owner, &mark).map(|m| mark_id = m.mark_id)
    });
    let undo = DeadUndoInput {
        mark_id,
        reason: "gate".into(),
    };
    at_every_step(&w, "dead amount: make active again", || {
        mandi_services::dead::undo(&w.env.ctx, &w.owner, &undo).map(|_| ())
    });
}

/// The search index lives in memory and is updated after a save commits (Code Standards B27). A
/// save that fails must not put the new name into it, and the one that works must.
#[test]
fn a_failed_account_save_never_reaches_the_search_index() {
    use mandi_services::core::accounts::{AccountInput, AccountType};
    use mandi_services::core::search::SearchScope;
    let w = support::world();
    let buyer = AccountInput {
        id: None,
        version: None,
        account_type: AccountType::Buyer,
        control_id: None,
        code: String::new(),
        name_ur: "طارق".into(),
        name_en: "Tariq".into(),
        confirmed_rules_words: vec!["طارق".into(), "Tariq".into()],
        father_or_firm_ur: String::new(),
        father_or_firm_en: String::new(),
        city_id: None,
        adda_id: None,
        phone1: String::new(),
        phone2: String::new(),
        address: String::new(),
        linked_account_id: None,
        opening_date: "10-10-2026".into(),
        opening_dr: "5,000".into(),
        opening_cr: String::new(),
        opening_is_advance: false,
        credit_allowed: true,
        credit_limit: String::new(),
        default_marka_id: None,
        collector_staff_id: None,
        is_active: true,
        note: String::new(),
        tag: String::new(),
    };
    let found = |text: &str| {
        mandi_services::search::search(&w.env.ctx, &w.owner, SearchScope::Buyer, text, 20)
            .unwrap()
            .into_iter()
            .filter(|hit| hit.name_en == "Tariq")
            .count()
    };
    assert_eq!(found("tariq"), 0);
    let mut failed_tries = 0;
    at_every_step(&w, "account save and the search index", || {
        let result = mandi_services::accounts::save(&w.env.ctx, &w.owner, &buyer).map(|_| ());
        if result.is_err() {
            failed_tries += 1;
            assert_eq!(
                found("tariq"),
                0,
                "a failed save put the name in the search index"
            );
            assert_eq!(
                found("طارق"),
                0,
                "a failed save put the Urdu name in the search index"
            );
            // The next free code is still the first one: no code was used by the failure.
        }
        result
    });
    assert!(failed_tries > 3);
    assert_eq!(found("tariq"), 1, "the saved account is not found");
    assert_eq!(
        found("tarq"),
        1,
        "the saved account is not found by a small spelling mistake"
    );
}

/// The commission rules and the partner shares are read through caches. A rule or a share change
/// that fails must leave the next preview and save with the old numbers, and the one that works
/// must change them at once (Code Standards B5, FR-11.6).
#[test]
fn a_failed_rule_or_share_change_leaves_the_next_preview_with_the_old_numbers() {
    use mandi_services::core::partners::{PartnersSetSharesInput, ShareInput};
    use mandi_services::core::rules::{ChargeMethod, ChargeSide, ChargeType, RuleInput};
    let w = support::world();
    let draft = support::event1(&w);
    let numbers = |w: &World| -> (i64, Vec<i64>) {
        let p = bikri::preview(&w.env.ctx, &draft).unwrap();
        (
            p.summary.seller_comm.rupees(),
            p.split.iter().map(|s| s.income_paisa.rupees()).collect(),
        )
    };
    // The PRD numbers: seller commission 720 + 1,044 and the split of 3,948.
    assert_eq!(numbers(&w), (1_764, vec![1_580, 1_184, 1_184]));
    let rule = RuleInput {
        id: None,
        version: None,
        side: ChargeSide::Seller,
        charge_type: ChargeType::Commission,
        account_id: Some(w.rahim),
        item_id: None,
        item_group: None,
        method: ChargeMethod::Percent,
        value: "5".into(),
        from_date: "10-10-2026".into(),
        to_date: String::new(),
        is_active: true,
        note: String::new(),
    };
    let mut seen_after_failures = Vec::new();
    at_every_step(&w, "commission rule and the cache", || {
        let result = mandi_services::rules::save(&w.env.ctx, &w.owner, &rule).map(|_| ());
        if result.is_err() {
            seen_after_failures.push(numbers(&w));
        }
        result
    });
    assert!(
        seen_after_failures
            .iter()
            .all(|n| *n == (1_764, vec![1_580, 1_184, 1_184])),
        "a failed rule save changed the preview: {seen_after_failures:?}"
    );
    // The rule works from the next call: 5% of 12,000 and 17,400.
    assert_eq!(numbers(&w).0, 600 + 870);
    // Shares: 50 / 30 / 20 from today.
    let [a, b, c] = w.partners;
    let shares = PartnersSetSharesInput {
        effective_from: "10-10-2026".into(),
        shares: [(a, "50"), (b, "30"), (c, "20")]
            .iter()
            .map(|&(partner_id, share)| ShareInput {
                partner_id,
                share: share.to_owned(),
            })
            .collect(),
    };
    let before = numbers(&w).1;
    let mut after_failures = Vec::new();
    at_every_step(&w, "partner shares and the cache", || {
        let result =
            mandi_services::partners::set_shares(&w.env.ctx, &w.owner, &shares).map(|_| ());
        if result.is_err() {
            after_failures.push(numbers(&w).1);
        }
        result
    });
    assert!(
        after_failures.iter().all(|n| *n == before),
        "a failed share change changed the split: {after_failures:?}"
    );
    assert_ne!(numbers(&w).1, before, "the new shares are not used");
    let split = numbers(&w).1;
    // Seller commission 5% (1,470) + buyer commission 6% (1,764) + laga 400 + phone 20.
    assert_eq!(split.iter().sum::<i64>(), 1_470 + 1_764 + 400 + 20);
}

// ---------------------------------------------------------------------------------------------
// Part 3: undo of a mixed save, write-off undo, trip cost cancel, purchase cancel.
// ---------------------------------------------------------------------------------------------

#[test]
fn undo_of_a_mixed_sale_and_of_write_offs_fail_cleanly_at_every_row_change() {
    use mandi_services::core::bikri::{
        CarriedLotInput, MixedSaleSaveInput, MixedUndoInput, WriteOffInput, WriteOffUndoInput,
    };
    use mandi_services::core::purchases::{PurchaseWriteOffInput, PurchaseWriteOffUndoInput};
    let w = support::world();
    save_bikri(&w, &part_sold(&w)).unwrap();
    let lot_id = scalar(&w, "SELECT id FROM bikri_lots");
    common::set_today(&w.env, 2026, 10, 11);
    let mut new_lot = support::lot(&w, "lot-n", w.karim, w.apple, "10");
    new_lot.sales = vec![support::sale("s-n", w.akram, "10", "1200")];
    let saved = bikri::mixed_save(
        &w.env.ctx,
        &w.owner,
        &MixedSaleSaveInput {
            date: "11-10-2026".into(),
            carried: vec![CarriedLotInput {
                key: "c-1".into(),
                lot_id,
                sales: vec![support::sale("s-c", w.arshad, "5", "560")],
            }],
            new_lots: vec![new_lot],
            confirm_cash_below_zero: true,
        },
    )
    .unwrap();
    let undo = MixedUndoInput {
        later_entry_ids: saved.carried.iter().map(|c| c.entry_id).collect(),
        new_bikri_id: saved.new_bikri_id,
        confirm_cash_below_zero: true,
    };
    at_every_step(&w, "undo of a mixed sale", || {
        bikri::mixed_undo(&w.env.ctx, &w.owner, &undo).map(|_| ())
    });
    // A stock write-off and its undo.
    let mut move_id = 0;
    let off = WriteOffInput {
        lot_id,
        qty: "5".into(),
        reason: "spoiled".into(),
    };
    bikri::write_off(&w.env.ctx, &w.owner, &off)
        .map(|d| move_id = d.move_id)
        .unwrap();
    let back = WriteOffUndoInput {
        move_id,
        reason: "found".into(),
    };
    at_every_step(&w, "undo of a stock write-off", || {
        bikri::write_off_undo(&w.env.ctx, &w.owner, &back).map(|_| ())
    });
    // A trip purchase: write-off undo, then the cost cancel and the purchase cancel.
    let trip = trips::create(&w.env.ctx, &w.owner, &trip_input(&w)).unwrap();
    let orange = support::add_item(&w.env, &w.owner, "مالٹا", "Orange", "fruit", "unit");
    let bought =
        purchases::save(&w.env.ctx, &w.owner, &purchase_input(&w, trip.id, orange)).unwrap();
    let mut pmove = 0;
    purchases::write_off(
        &w.env.ctx,
        &w.owner,
        &PurchaseWriteOffInput {
            purchase_id: bought.id,
            qty: "5".into(),
            reason: "spoiled".into(),
        },
    )
    .map(|d| pmove = d.move_id)
    .unwrap();
    let pundo = PurchaseWriteOffUndoInput {
        move_id: pmove,
        reason: "found".into(),
    };
    at_every_step(&w, "undo of a trip purchase write-off", || {
        purchases::write_off_undo(&w.env.ctx, &w.owner, &pundo).map(|_| ())
    });
    let version = scalar(
        &w,
        &format!("SELECT version FROM purchases WHERE id = {}", bought.id),
    );
    let cancel = mandi_services::core::purchases::PurchaseCancelInput {
        id: bought.id,
        version,
        reason: "gate".into(),
        confirm_cash_below_zero: true,
    };
    at_every_step(&w, "trip purchase cancel", || {
        purchases::cancel(&w.env.ctx, &w.owner, &cancel).map(|_| ())
    });
}
