//! Attacks on a new account's opening balance on any day (plan 53 T1, PRD FR-10.9, FR-10.11, E79,
//! Umar 03-11-2026): a NEW account with an opening saves even on a closed day, the opening posts on
//! its own date, an audit row says "opening on a closed day", and only a date after today is refused.
//! A CHANGED opening and the openings grid keep `ERR_DATE_CLOSED`. Owner: `robustness`.
//!
//! Backend asked for attacks on: an opening dated before the first transaction, in a month that is
//! closed, a seller's opening advance, and a 0103 (money lent) or 0603 (drawings) account. In every
//! case:
//!
//! - the books balance in the raw tables and both hash chains verify;
//! - a refusal writes nothing (a fingerprint of every table before and after);
//! - the partners' totals (کل رقم) do not move, because an opening touches no income or expense.
#![cfg(test)]

mod common;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use common::*;
use mandi_services::core::accounts::{
    AccountInput, AccountType, BalanceSide, OpeningLineInput, OpeningsInput,
};
use mandi_services::core::error::ErrorCode;
use mandi_services::core::expenses::ShopExpenseInput;
use mandi_services::core::vouchers::DayCloseInput;
use mandi_services::db::rusqlite::types::Value;
use mandi_services::month::{self, MonthInput};
use mandi_services::{ServiceError, accounts, day, ledger, masters, partners, vouchers};
use support::World;
use tempfile::TempDir;

const OPENING_REASON: &str = "opening on a closed day";

fn account_input(kind: AccountType, ur: &str, en: &str) -> AccountInput {
    AccountInput {
        id: None,
        version: None,
        account_type: kind,
        control_id: None,
        code: String::new(),
        name_ur: ur.into(),
        name_en: en.into(),
        confirmed_rules_words: Vec::new(),
        father_or_firm_ur: String::new(),
        father_or_firm_en: String::new(),
        city_id: None,
        adda_id: None,
        phone1: String::new(),
        phone2: String::new(),
        address: String::new(),
        linked_account_id: None,
        opening_date: String::new(),
        opening_dr: String::new(),
        opening_cr: String::new(),
        opening_is_advance: false,
        credit_allowed: true,
        credit_limit: String::new(),
        default_marka_id: None,
        collector_staff_id: None,
        is_active: true,
        note: String::new(),
        tag: String::new(),
    }
}

/// A new account with an opening debit on a date, as the form sends it.
fn with_opening(kind: AccountType, name: (&str, &str), date: &str, dr: &str) -> AccountInput {
    let mut form = account_input(kind, name.0, name.1);
    form.opening_date = date.into();
    form.opening_dr = dr.into();
    form
}

fn code_of(error: ServiceError) -> ErrorCode {
    match error {
        ServiceError::Rejected(items) => items[0].code,
        other => panic!("not a rejection: {other:?}"),
    }
}

/// Every table, row by row, hashed. Equal before and after means nothing was written.
fn fingerprint(dir: &TempDir) -> u64 {
    let conn = raw(dir);
    let tables: Vec<String> = {
        let mut statement = conn
            .prepare(
                "SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%' ORDER BY name",
            )
            .unwrap();
        statement
            .query_map([], |r| r.get(0))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap()
    };
    let mut hasher = DefaultHasher::new();
    for table in tables {
        table.hash(&mut hasher);
        let mut statement = conn
            .prepare(&format!("SELECT * FROM \"{table}\" ORDER BY rowid"))
            .unwrap();
        let columns = statement.column_count();
        let mut rows = statement.query([]).unwrap();
        while let Some(row) = rows.next().unwrap() {
            for i in 0..columns {
                let value: Value = row.get(i).unwrap();
                format!("{value:?}").hash(&mut hasher);
            }
        }
    }
    hasher.finish()
}

fn one(dir: &TempDir, sql: &str) -> i64 {
    raw(dir).query_row(sql, [], |r| r.get(0)).unwrap()
}

/// The books in the raw tables: debits equal credits, no draft is left, both chains verify.
fn assert_books(w: &World, note: &str) {
    let dr = one(
        &w.dir,
        "SELECT COALESCE(SUM(l.dr_paisa), 0) FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id WHERE e.status = 'posted'",
    );
    let cr = one(
        &w.dir,
        "SELECT COALESCE(SUM(l.cr_paisa), 0) FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id WHERE e.status = 'posted'",
    );
    assert_eq!(dr, cr, "debits and credits differ after {note}");
    assert_eq!(
        one(
            &w.dir,
            "SELECT COUNT(*) FROM journal_entries WHERE status <> 'posted'"
        ),
        0,
        "a draft is left after {note}"
    );
    ledger::verify_journal_chain(&w.env.ctx).unwrap();
    assert_chain_ok(&w.dir);
}

fn totals(w: &World) -> Vec<i64> {
    partners::overview(&w.env.ctx)
        .unwrap()
        .into_iter()
        .map(|r| r.total_paisa.rupees())
        .collect()
}

fn close_through(w: &World, date: &str) {
    day::close(&w.env.ctx, &w.owner, &DayCloseInput { up_to: date.into() }).unwrap();
}

/// How many openings with this audit reason are written.
fn audit_reasons(w: &World) -> i64 {
    raw(&w.dir)
        .query_row(
            "SELECT COUNT(*) FROM audit_log WHERE reason = ?1",
            [OPENING_REASON],
            |r| r.get(0),
        )
        .unwrap()
}

/// The entries of the opening posting type dated this day (`YYYY-MM-DD`).
fn openings_on(w: &World, iso: &str) -> i64 {
    raw(&w.dir)
        .query_row(
            "SELECT COUNT(*) FROM journal_entries WHERE entry_date = ?1 AND posting_type = 'opening'",
            [iso],
            |r| r.get(0),
        )
        .unwrap()
}

// ---- before the first transaction -------------------------------------------------------------

#[test]
fn an_opening_dated_long_before_the_first_transaction_saves_and_the_books_balance() {
    let w = support::world();
    for (date, iso) in [
        ("31-12-2025", "2025-12-31"),
        ("01-01-2020", "2020-01-01"),
        ("29-02-2024", "2024-02-29"),
        ("01-01-2000", "2000-01-01"),
    ] {
        let before = totals(&w);
        let saved = accounts::save(
            &w.env.ctx,
            &w.owner,
            &with_opening(
                AccountType::Buyer,
                ("ناصر", &format!("Nasir {iso}")),
                date,
                "100",
            ),
        )
        .unwrap_or_else(|e| panic!("{date}: {e:?}"));
        assert_eq!(saved.opening_dr_paisa.rupees(), 100, "{date}");
        assert_eq!(
            openings_on(&w, iso),
            1,
            "{date}: the entry is dated that day"
        );
        assert_eq!(
            totals(&w),
            before,
            "{date}: an opening moves no partner's total"
        );
        assert_books(&w, date);
    }
    // The ledger of a new account starts with its opening, whatever the date.
    let akram = ledger::account_balance(&w.env.ctx, w.akram).unwrap();
    assert_eq!(akram.amount_paisa.rupees(), 0);
}

#[test]
fn an_impossible_or_future_opening_date_is_refused_and_writes_nothing() {
    let w = support::world();
    close_through(&w, "05-10-2026");
    for date in [
        // After today (10-10-2026).
        "11-10-2026",
        "01-01-2027",
        "31-12-9999",
        // Not dates at all.
        "31-02-2026",
        "00-00-0000",
        "ten",
        "10-13-2026",
        "01-01-0000",
    ] {
        let before = fingerprint(&w.dir);
        let result = accounts::save(
            &w.env.ctx,
            &w.owner,
            &with_opening(AccountType::Buyer, ("ناصر", "Nasir"), date, "100"),
        );
        let error = result.expect_err(date);
        let code = code_of(error);
        assert!(
            matches!(code, ErrorCode::ErrDateInFuture | ErrorCode::ErrDateInvalid),
            "{date}: {code:?}"
        );
        assert_eq!(
            fingerprint(&w.dir),
            before,
            "{date}: a refusal wrote something"
        );
    }
    assert_books(&w, "the refused dates");
}

// ---- a closed day, and a month that is closed ------------------------------------------------

/// The expense head "Electricity", made here (the worked world has none), with the spelling words
/// confirmed as the screen does.
fn electricity(w: &World) -> i64 {
    use mandi_services::core::error::ErrorValue;
    use mandi_services::core::masters::{ExpenseHeadInput, MasterInput};
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
    masters::expense_heads(&w.env.ctx)
        .unwrap()
        .rows
        .iter()
        .find(|r| r.head.name_en == "Electricity")
        .unwrap()
        .id
}

/// October with 3,000 of electricity on 10-10, over (today 02-11-2026), its days closed, and the
/// month closed (the profit moved into the partners' capital).
fn month_closed_world() -> World {
    let w = support::world();
    vouchers::save_shop_expense(
        &w.env.ctx,
        &w.owner,
        &ShopExpenseInput {
            date: "10-10-2026".into(),
            expense_head_id: Some(electricity(&w)),
            amount: "3,000".into(),
            paid_to: None,
            narration: None,
            confirm_cash_below_zero: true,
        },
    )
    .unwrap();
    set_today(&w.env, 2026, 11, 2);
    close_through(&w, "31-10-2026");
    month::close(
        &w.env.ctx,
        &w.owner,
        &MonthInput {
            month: "2026-10".into(),
        },
    )
    .unwrap();
    w
}

#[test]
fn an_opening_in_a_month_that_is_closed_saves_and_changes_no_partner_and_no_month_close() {
    let w = month_closed_world();
    let totals_before = totals(&w);
    let close_entry = one(
        &w.dir,
        "SELECT journal_entry_id FROM month_closes WHERE month = '2026-10' AND is_active = 1",
    );
    let close_total = one(
        &w.dir,
        &format!("SELECT total_paisa FROM journal_entries WHERE id = {close_entry}"),
    );
    let audit_was = audit_reasons(&w);
    let saved = accounts::save(
        &w.env.ctx,
        &w.owner,
        &with_opening(AccountType::Buyer, ("ناصر", "Nasir"), "15-10-2026", "7,770"),
    )
    .expect("a new account's opening in a closed month saves");
    assert_eq!(saved.opening_dr_paisa.rupees(), 7_770);
    assert_eq!(openings_on(&w, "2026-10-15"), 1);
    assert_eq!(audit_reasons(&w), audit_was + 1, "an audit row says so");
    assert_eq!(totals(&w), totals_before, "no partner's total moved");
    // The month close entry is as it was, and still the active one.
    assert_eq!(
        one(
            &w.dir,
            "SELECT journal_entry_id FROM month_closes WHERE month = '2026-10' AND is_active = 1"
        ),
        close_entry
    );
    assert_eq!(
        one(
            &w.dir,
            &format!("SELECT total_paisa FROM journal_entries WHERE id = {close_entry}")
        ),
        close_total
    );
    assert_books(&w, "an opening in a closed month");
    // The month close can still be reversed afterwards, and the books still balance.
    month::reverse(
        &w.env.ctx,
        &w.owner,
        &month::MonthReverseInput {
            month: "2026-10".into(),
            reason: "check".into(),
        },
    )
    .expect("the month close is reversed");
    assert_books(&w, "the month close reversed after an opening in it");
    assert_eq!(
        openings_on(&w, "2026-10-15"),
        1,
        "the opening is still there"
    );
}

#[test]
fn the_openings_grid_and_a_changed_opening_keep_refusing_a_closed_day() {
    let w = month_closed_world();
    // A new account made on the open day 02-11 has an opening on that day (open).
    let nasir = accounts::save(
        &w.env.ctx,
        &w.owner,
        &with_opening(AccountType::Buyer, ("ناصر", "Nasir"), "02-11-2026", "500"),
    )
    .unwrap();
    let before = fingerprint(&w.dir);
    // Changing his opening to a closed day: refused.
    let mut changed = account_input(AccountType::Buyer, "ناصر", "Nasir");
    changed.id = Some(nasir.id);
    changed.version = Some(nasir.version);
    changed.opening_date = "15-10-2026".into();
    changed.opening_dr = "900".into();
    assert_eq!(
        code_of(accounts::save(&w.env.ctx, &w.owner, &changed).unwrap_err()),
        ErrorCode::ErrDateClosed
    );
    // The grid, for his account and for a system one.
    for (account, version) in [
        (w.akram, accounts::get(&w.env.ctx, w.akram).unwrap().version),
        (nasir.id, nasir.version),
    ] {
        let grid = accounts::openings_save(
            &w.env.ctx,
            &w.owner,
            &OpeningsInput {
                lines: vec![OpeningLineInput {
                    account_id: account,
                    version,
                    opening_date: "15-10-2026".into(),
                    opening_dr: "900".into(),
                    opening_cr: String::new(),
                    opening_is_advance: false,
                }],
            },
        );
        assert_eq!(
            code_of(grid.unwrap_err()),
            ErrorCode::ErrDateClosed,
            "grid line for account {account}"
        );
    }
    assert_eq!(fingerprint(&w.dir), before, "the refusals wrote nothing");
    assert_books(&w, "the refused changes");
}

#[test]
fn a_closed_days_close_row_stays_as_it_was_and_the_opening_is_audited_and_balanced() {
    // Plan 53 §9 (lead, 04-11-2026): the close row is a record of the day as it was when it was
    // closed, and it is not rewritten when a new account's opening is dated on that day. The books
    // move (the trial balance of that day and the days after it now holds the opening); the record
    // of what was true at the close does not. What must hold:
    //  (a) the audit row "opening on a closed day" names the account, the amount and the date;
    //  (b) the trial balance for that day balances;
    //  (c) both hash chains verify;
    //  (d) the close row is unchanged.
    let w = month_closed_world();
    let day = "2026-10-15";
    let close_row = |w: &World| -> (i64, String) {
        raw(&w.dir)
            .query_row(
                "SELECT id, summary_json FROM day_closes WHERE entry_date = ?1 AND action = 'close' ORDER BY id DESC LIMIT 1",
                [day],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap()
    };
    let closed_rows = |w: &World| -> i64 {
        raw(&w.dir)
            .query_row("SELECT COUNT(*) FROM day_closes", [], |r| r.get(0))
            .unwrap()
    };
    let (row_id, row_json) = close_row(&w);
    let rows_before = closed_rows(&w);
    let saved = accounts::save(
        &w.env.ctx,
        &w.owner,
        &with_opening(AccountType::Buyer, ("ناصر", "Nasir"), "15-10-2026", "1,000"),
    )
    .unwrap();
    // (a) The audit row names the account, the amount and the date.
    let (entity_id, new_json): (i64, String) = raw(&w.dir)
        .query_row(
            "SELECT entity_id, new_json FROM audit_log WHERE reason = ?1 ORDER BY id DESC LIMIT 1",
            [OPENING_REASON],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .expect("an audit row with the reason 'opening on a closed day'");
    assert_eq!(entity_id, saved.id, "the audit row names the account");
    let new: serde_json::Value = serde_json::from_str(&new_json).unwrap();
    assert_eq!(new["date"], day, "the audit row names the date: {new}");
    assert_eq!(
        new["dr_paisa"], 100_000,
        "the audit row names the amount: {new}"
    );
    assert_eq!(new["cr_paisa"], 0, "{new}");
    // (b) The trial balance for that day balances, and so do the days after it.
    for as_of in [day, "2026-10-31", "2026-11-02"] {
        let trial = ledger::trial_balance(&w.env.ctx, Some(as_of)).unwrap();
        assert_eq!(
            trial.total_dr.as_paisa(),
            trial.total_cr.as_paisa(),
            "the trial balance as of {as_of} does not balance"
        );
    }
    // (c) Both hash chains verify.
    ledger::verify_journal_chain(&w.env.ctx).unwrap();
    assert_chain_ok(&w.dir);
    assert_books(&w, "an opening on a closed day");
    // (d) The close row is the one written at the close, word for word, and no row was added.
    let (row_id_now, row_json_now) = close_row(&w);
    assert_eq!(row_id_now, row_id, "no new close row");
    assert_eq!(
        row_json_now, row_json,
        "the close row of {day} was rewritten"
    );
    assert_eq!(closed_rows(&w), rows_before);
}

// ---- a seller's opening advance ---------------------------------------------------------------

#[test]
fn a_sellers_opening_advance_on_a_closed_day_is_an_advance_and_a_buyers_is_not() {
    let w = support::world();
    close_through(&w, "05-10-2026");
    let was = audit_reasons(&w);
    let mut seller = with_opening(
        AccountType::Seller,
        ("سلیم", "Saleem"),
        "01-10-2026",
        "5,000",
    );
    seller.opening_is_advance = true;
    let saved = accounts::save(&w.env.ctx, &w.owner, &seller).expect("a seller's advance saves");
    assert_eq!(audit_reasons(&w), was + 1);
    let advance_lines = one(
        &w.dir,
        &format!(
            "SELECT COUNT(*) FROM journal_lines WHERE account_id = {} AND is_advance = 1 AND dr_paisa = 500000",
            saved.id
        ),
    );
    assert_eq!(advance_lines, 1, "one advance line of 5,000 on the seller");
    let balance = ledger::account_balance(&w.env.ctx, saved.id).unwrap();
    assert_eq!(
        (balance.amount_paisa.rupees(), balance.side),
        (5_000, BalanceSide::Dr)
    );
    assert_books(&w, "a seller's opening advance");
    // A buyer cannot have an advance: refused, or the line is not marked as one. Never a quiet advance.
    let before = fingerprint(&w.dir);
    let mut buyer = with_opening(AccountType::Buyer, ("ناصر", "Nasir"), "01-10-2026", "5,000");
    buyer.opening_is_advance = true;
    match accounts::save(&w.env.ctx, &w.owner, &buyer) {
        Err(error) => {
            code_of(error);
            assert_eq!(fingerprint(&w.dir), before, "the refusal wrote something");
        }
        Ok(row) => {
            let marked = one(
                &w.dir,
                &format!(
                    "SELECT COUNT(*) FROM journal_lines WHERE account_id = {} AND is_advance = 1",
                    row.id
                ),
            );
            assert_eq!(marked, 0, "a buyer's opening was marked as an advance");
        }
    }
    // An advance with a credit is no advance: refused.
    let before = fingerprint(&w.dir);
    let mut credit = account_input(AccountType::Seller, "جمیل", "Jameel");
    credit.opening_date = "01-10-2026".into();
    credit.opening_cr = "5,000".into();
    credit.opening_is_advance = true;
    match accounts::save(&w.env.ctx, &w.owner, &credit) {
        Err(error) => {
            code_of(error);
            assert_eq!(fingerprint(&w.dir), before);
        }
        Ok(row) => {
            let marked = one(
                &w.dir,
                &format!(
                    "SELECT COUNT(*) FROM journal_lines WHERE account_id = {} AND is_advance = 1",
                    row.id
                ),
            );
            assert_eq!(marked, 0, "a credit opening was marked as an advance");
        }
    }
    assert_books(&w, "the advance attacks");
}

// ---- 0103 (money lent) and 0603 (drawings) ---------------------------------------------------

fn id_by_sql(w: &World, sql: &str) -> i64 {
    one(&w.dir, sql)
}

#[test]
fn a_person_lent_money_gets_his_opening_on_a_closed_day_and_the_drawings_account_only_a_debit() {
    let w = support::world();
    close_through(&w, "05-10-2026");
    // 0103: a new money-lent person with an opening on a closed day.
    let lent = accounts::save(
        &w.env.ctx,
        &w.owner,
        &with_opening(
            AccountType::MoneyLent,
            ("اسلم", "Aslam"),
            "01-10-2026",
            "2,000",
        ),
    )
    .expect("a new 0103 account with an opening on a closed day");
    assert!(lent.code.starts_with("0103"), "{}", lent.code);
    assert_eq!(
        ledger::account_balance(&w.env.ctx, lent.id)
            .unwrap()
            .amount_paisa
            .rupees(),
        2_000
    );
    assert_books(&w, "an opening for a person lent money");

    // 0603: the drawings account of partner A, made with the partner. Its opening can be a debit
    // (money he took before the app) on an open day; on a closed day it is a changed opening.
    let drawings = id_by_sql(
        &w,
        &format!(
            "SELECT drawings_account_id FROM partners WHERE id = {}",
            w.partners[0]
        ),
    );
    let version = accounts::get(&w.env.ctx, drawings).unwrap().version;
    let line = |date: &str, dr: &str, cr: &str| OpeningLineInput {
        account_id: drawings,
        version,
        opening_date: date.into(),
        opening_dr: dr.into(),
        opening_cr: cr.into(),
        opening_is_advance: false,
    };
    let before = fingerprint(&w.dir);
    let closed = accounts::openings_save(
        &w.env.ctx,
        &w.owner,
        &OpeningsInput {
            lines: vec![line("01-10-2026", "2,000", "")],
        },
    );
    assert_eq!(code_of(closed.unwrap_err()), ErrorCode::ErrDateClosed);
    assert_eq!(fingerprint(&w.dir), before, "the refusal wrote something");
    let credit = accounts::openings_save(
        &w.env.ctx,
        &w.owner,
        &OpeningsInput {
            lines: vec![line("10-10-2026", "", "2,000")],
        },
    );
    assert!(
        credit.is_err(),
        "a drawings account takes a debit, not a credit"
    );
    assert_eq!(fingerprint(&w.dir), before, "the refusal wrote something");
    accounts::openings_save(
        &w.env.ctx,
        &w.owner,
        &OpeningsInput {
            lines: vec![line("10-10-2026", "2,000", "")],
        },
    )
    .expect("a debit on an open day");
    assert_books(&w, "an opening on a drawings account");
    // The money he took is in his total: -2,000.
    let a_total = totals(&w)[0];
    assert!(
        a_total < 0,
        "a drawings opening comes off his total: {a_total}"
    );
}

#[test]
fn the_accounts_that_take_no_opening_refuse_one_on_a_closed_day_too() {
    // Income and expense accounts, ابتدائی فرق, نقدی منڈی, the bank, a partner's garden/trip/fruit
    // account: no opening, whatever the day. Nothing is written by the refusal.
    let w = support::world();
    close_through(&w, "05-10-2026");
    let codes = [
        "04010001", "06019999", "03040000", "01020001", "0030", "0031",
    ];
    for code in codes {
        let id = support::id_of(&w.env, code);
        let version = accounts::get(&w.env.ctx, id).unwrap().version;
        for date in ["01-10-2026", "10-10-2026"] {
            let before = fingerprint(&w.dir);
            let result = accounts::openings_save(
                &w.env.ctx,
                &w.owner,
                &OpeningsInput {
                    lines: vec![OpeningLineInput {
                        account_id: id,
                        version,
                        opening_date: date.into(),
                        opening_dr: "100".into(),
                        opening_cr: String::new(),
                        opening_is_advance: false,
                    }],
                },
            );
            assert!(result.is_err(), "{code} {date}: an opening was accepted");
            assert_eq!(
                fingerprint(&w.dir),
                before,
                "{code} {date}: a refusal wrote something"
            );
        }
    }
    let venture = id_by_sql(
        &w,
        &format!(
            "SELECT venture_account_id FROM partners WHERE id = {}",
            w.partners[0]
        ),
    );
    let version = accounts::get(&w.env.ctx, venture).unwrap().version;
    let before = fingerprint(&w.dir);
    let result = accounts::openings_save(
        &w.env.ctx,
        &w.owner,
        &OpeningsInput {
            lines: vec![OpeningLineInput {
                account_id: venture,
                version,
                opening_date: "10-10-2026".into(),
                opening_dr: "100".into(),
                opening_cr: String::new(),
                opening_is_advance: false,
            }],
        },
    );
    assert!(
        result.is_err(),
        "a partner's garden, trip and fruit account took an opening"
    );
    assert_eq!(fingerprint(&w.dir), before);
    assert_books(&w, "the refused system openings");
}

#[test]
fn a_repeated_new_account_with_the_same_name_and_opening_is_asked_and_not_doubled() {
    // Two clicks of Save on a new account with an opening on a closed day must not post the opening
    // twice: the second is a duplicate warning or a refusal, and the books carry one opening.
    let w = support::world();
    close_through(&w, "05-10-2026");
    let form = with_opening(AccountType::Buyer, ("ناصر", "Nasir"), "01-10-2026", "100");
    accounts::save(&w.env.ctx, &w.owner, &form).unwrap();
    let again = accounts::save(&w.env.ctx, &w.owner, &form);
    let openings = openings_on(&w, "2026-10-01");
    match again {
        Ok(_) => assert_eq!(
            openings, 2,
            "a second account is a second account with its own opening"
        ),
        Err(_) => assert_eq!(openings, 1, "a refused duplicate left one opening"),
    }
    assert_books(&w, "a double save");
}
