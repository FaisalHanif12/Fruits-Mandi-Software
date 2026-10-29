//! The PRD worked day, through the real services with an injected clock (PRD Section 4 and 27,
//! Code Standards 9.2). The expected numbers come from the PRD and are never changed to suit
//! the code (CLAUDE.md "Rules that are never broken").
//!
//! Plan 10 starts the file with the opening position of PRD 4.3 and E2E-19. Plan 13 adds Event 1
//! and Event 2 (PRD 4.4 and 4.5: E2E-1 to E2E-5) and the بکری part of E2E-28. Later plans add the
//! other events of Section 4.4 to 4.8 and E2E-6 to E2E-16b, E2E-20 to E2E-21 and E2E-29 to
//! E2E-33 on top of the same world.
#![cfg(test)]

mod common;
/// The builders of the plan 12 tests: the worked day's parties, items and partners, and the
/// typed draft of a بکری.
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use common::*;
use mandi_services::core::accounts::{
    AccountInput, AccountType, BalanceSide, OpeningLineInput, OpeningsInput,
};
use mandi_services::core::bikri::{BikriDraft, BikriSaved, HeaderInput, PukhtaInput};
use mandi_services::core::error::ErrorCode;
use mandi_services::core::money::Paisa;
use mandi_services::core::vouchers::{
    AdvanceInput, AdvancesReportInput, DayCloseInput, DayClosePreviewInput, LineSource,
    OpenBillsInput, OpenPayablesInput, PaymentInput, ReceiptInput, VoucherEntry, VoucherPreview,
    VoucherPreviewInput, VoucherType, VouchersListInput,
};
use mandi_services::testing::TestEnv;
use mandi_services::{accounts, assist, auth, bikri, day, ledger, vouchers};
use tempfile::TempDir;

/// The accounts of the worked day that exist after the opening position.
pub struct Position {
    pub dir: TempDir,
    pub env: TestEnv,
    pub owner: auth::Actor,
    /// رحیم, seller 0010.
    pub rahim: i64,
    /// روکڑ, 01010001.
    pub cash: i64,
    /// ابتدائی سرمایہ, 06010001.
    pub capital: i64,
    /// ابتدائی فرق, 06019999.
    pub difference: i64,
}

fn rupees(value: i64) -> Paisa {
    Paisa::from_rupees(value).unwrap()
}

fn account_id(env: &TestEnv, code: &str) -> i64 {
    env.ctx
        .db()
        .read(|c| mandi_services::db::accounts::id_by_code(c, code))
        .unwrap()
        .unwrap_or_else(|| panic!("no account {code}"))
}

fn seller_rahim(env: &TestEnv, owner: &auth::Actor) -> i64 {
    let input = AccountInput {
        id: None,
        version: None,
        account_type: AccountType::Seller,
        control_id: None,
        code: String::new(),
        name_ur: "رحیم".into(),
        name_en: "Rahim".into(),
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
        collection_weekday: None,
        is_active: true,
        note: String::new(),
        tag: String::new(),
    };
    accounts::save(&env.ctx, owner, &input).unwrap().id
}

/// PRD 4.3, steps 1 and 2: on 31-08-26 the openings are typed on the account forms; on 01-09-26 a
/// پیشگی voucher gives رحیم 15,000 cash. The clock is left on 01-09-26.
pub fn opening_position() -> Position {
    let (dir, env) = env();
    set_up(&env);
    login_owner(&env);
    let owner = auth::require(&env.ctx).unwrap();
    let rahim = seller_rahim(&env, &owner);
    let cash = account_id(&env, "01010001");
    let capital = account_id(&env, "06010001");
    let difference = account_id(&env, "06019999");

    // Step 1, "today" 31-08-26: روکڑ opening بنام 65,000 and ابتدائی سرمایہ opening جمع 65,000.
    set_today(&env, 2026, 8, 31);
    let line = |id: i64, dr: &str, cr: &str| OpeningLineInput {
        account_id: id,
        version: accounts::get(&env.ctx, id).unwrap().version,
        opening_date: "31-08-2026".into(),
        opening_dr: dr.into(),
        opening_cr: cr.into(),
        opening_is_advance: false,
    };
    accounts::openings_save(
        &env.ctx,
        &owner,
        &OpeningsInput {
            lines: vec![line(cash, "65,000", ""), line(capital, "", "65,000")],
        },
    )
    .unwrap();

    // Step 2, "today" 01-09-26: پیشگی پ-1 to رحیم, 15,000 cash.
    set_today(&env, 2026, 9, 1);
    let saved = vouchers::save_advance(
        &env.ctx,
        &owner,
        &AdvanceInput {
            confirm_cash_below_zero: false,
            date: "01-09-2026".into(),
            seller_account_id: rahim,
            amount: "15,000".into(),
            removed: mandi_services::core::vouchers::RemovedPaymentFields::default(),
            expected_goods: String::new(),
            narration: String::new(),
        },
    )
    .unwrap();
    assert_eq!(
        (saved.voucher_type, saved.number),
        (VoucherType::Advance, 1)
    );
    Position {
        dir,
        env,
        owner,
        rahim,
        cash,
        capital,
        difference,
    }
}

#[test]
fn e2e_19_the_opening_position() {
    // PRD 4.3 and E2E-19: opening set-up only. Trial balance 65,000 = 65,000; ابتدائی فرق 0;
    // advances open 15,000; cash 50,000; days up to 09-10-26 closed, 10-10-26 open.
    let p = opening_position();

    // Step 3: "today" is 10-10-26. Close the days up to and including 09-10-26, in order.
    set_today(&p.env, 2026, 10, 10);
    let preview = day::close_preview(
        &p.env.ctx,
        &DayClosePreviewInput {
            up_to: "09-10-2026".into(),
        },
    )
    .unwrap();
    assert_eq!(
        (
            preview.from_date.as_str(),
            preview.up_to.as_str(),
            preview.days
        ),
        ("2026-08-31", "2026-10-09", 40)
    );
    assert_eq!(
        (preview.trial_dr_paisa, preview.trial_cr_paisa),
        (rupees(65_000), rupees(65_000))
    );
    assert_eq!(preview.trial_difference.side, BalanceSide::Zero);
    assert_eq!(
        (preview.cash_in_hand.side, preview.cash_in_hand.amount_paisa),
        (BalanceSide::Dr, rupees(50_000))
    );
    // Two opening entries and the advance.
    assert_eq!(preview.entries, 3);
    let status = day::close(
        &p.env.ctx,
        &p.owner,
        &DayCloseInput {
            up_to: "09-10-2026".into(),
        },
    )
    .unwrap();
    assert_eq!(status.closed_up_to.as_deref(), Some("2026-10-09"));
    assert!(status.today_open);
    assert_eq!(status.today, "2026-10-10");

    // E2E-19: trial balance 65,000 = 65,000.
    let trial = ledger::trial_balance(&p.env.ctx, None).unwrap();
    assert_eq!(
        (trial.total_dr, trial.total_cr),
        (rupees(65_000), rupees(65_000))
    );
    let row = |code: &str| {
        trial
            .rows
            .iter()
            .find(|r| r.code == code)
            .map(|r| r.balance)
    };
    assert_eq!(
        row("01010001").map(|b| (b.side, b.amount_paisa)),
        Some((BalanceSide::Dr, rupees(50_000))),
        "cash 50,000"
    );
    assert_eq!(
        row("0010").map(|b| (b.side, b.amount_paisa)),
        Some((BalanceSide::Dr, rupees(15_000))),
        "رحیم بنام 15,000"
    );
    assert_eq!(
        row("06010001").map(|b| (b.side, b.amount_paisa)),
        Some((BalanceSide::Cr, rupees(65_000))),
        "ابتدائی سرمایہ جمع 65,000"
    );
    // ابتدائی فرق is 0: its two sides cancel.
    assert_eq!(
        ledger::account_balance(&p.env.ctx, p.difference)
            .unwrap()
            .side,
        BalanceSide::Zero
    );
    assert_eq!(
        (
            ledger::account_balance(&p.env.ctx, p.cash)
                .unwrap()
                .amount_paisa,
            ledger::account_balance(&p.env.ctx, p.capital)
                .unwrap()
                .amount_paisa
        ),
        (rupees(50_000), rupees(65_000))
    );

    // Advances open 15,000: the card of رحیم and the پیشگی screen agree.
    let info = assist::advances(&p.env.ctx, p.rahim).unwrap();
    assert_eq!(info.open_paisa, rupees(15_000));
    assert_eq!(info.open.len(), 1);
    assert_eq!(
        (
            info.open[0].source.as_str(),
            info.open[0].voucher_number,
            info.open[0].entry_date.as_str()
        ),
        ("voucher", Some(1), "2026-09-01")
    );
    assert_eq!(
        (
            info.open[0].given_paisa,
            info.open[0].adjusted_paisa,
            info.open[0].open_paisa
        ),
        (rupees(15_000), Paisa::ZERO, rupees(15_000))
    );
    let last = info.last.unwrap();
    assert_eq!(
        (
            last.entry_date.as_str(),
            last.amount_paisa,
            last.voucher_number
        ),
        ("2026-09-01", rupees(15_000), 1)
    );
    assert_eq!(
        assist::party_card(&p.env.ctx, p.rahim)
            .unwrap()
            .advance_open,
        Some(rupees(15_000))
    );

    // The voucher list has پ-1.
    let list = vouchers::list(
        &p.env.ctx,
        &VouchersListInput {
            sort: None,
            group_by_month: false,
            search: None,
            show_finished: true,
            account_id: None,
            person_key: None,
            voucher_type: Some(VoucherType::Advance),
            from: String::new(),
            to: String::new(),
            offset: 0,
            limit: 0,
        },
    )
    .unwrap()
    .rows;
    assert_eq!(list.len(), 1);
    assert_eq!(
        (
            list[0].number,
            list[0].amount_paisa,
            list[0].status.as_str()
        ),
        (1, rupees(15_000), "saved")
    );

    // 09-10-26 is closed and 10-10-26 is open (PRD FR-6.6).
    let refused = vouchers::save_advance(
        &p.env.ctx,
        &p.owner,
        &AdvanceInput {
            confirm_cash_below_zero: false,
            date: "09-10-2026".into(),
            seller_account_id: p.rahim,
            amount: "100".into(),
            removed: mandi_services::core::vouchers::RemovedPaymentFields::default(),
            expected_goods: String::new(),
            narration: String::new(),
        },
    )
    .unwrap_err();
    assert_eq!(
        refused.code(),
        mandi_services::core::error::ErrorCode::ErrDateClosed
    );

    // The journal chain and the audit chain are intact, and no partner share was written (an
    // advance touches no income or expense account, PRD FR-6.5).
    ledger::verify_journal_chain(&p.env.ctx).unwrap();
    assert_chain_ok(&p.dir);
    let shares: i64 = raw(&p.dir)
        .query_row("SELECT COUNT(*) FROM partner_share_rows", [], |r| r.get(0))
        .unwrap();
    assert_eq!(shares, 0);
}

// ---------------------------------------------------------------------------------------------
// Plan 13: Event 1 and Event 2 saved (PRD 4.4, 4.5; E2E-1 to E2E-5, E2E-28 بکری part).
// ---------------------------------------------------------------------------------------------

/// The D2 setup of E2E-19 (opening position, days up to 09-10-26 closed, today 10-10-26) with the
/// parties, items and partners 40 / 30 / 30 of the worked day on top.
fn worked_day() -> support::World {
    let p = opening_position();
    set_today(&p.env, 2026, 10, 10);
    day::close(
        &p.env.ctx,
        &p.owner,
        &DayCloseInput {
            up_to: "09-10-2026".into(),
        },
    )
    .unwrap();
    support::finish_world(p.dir, p.env, p.owner, p.rahim, true)
}

/// A line of a posted entry as (code, debit, credit, person key), in rupees, by line number.
fn entry_lines(w: &support::World, entry_id: i64) -> Vec<(String, i64, i64, Option<String>)> {
    let conn = raw(&w.dir);
    let mut statement = conn
        .prepare(
            "SELECT a.code, l.dr_paisa, l.cr_paisa, l.pukhta_person_key
             FROM journal_lines l JOIN accounts a ON a.id = l.account_id
             WHERE l.entry_id = ?1 ORDER BY l.line_no",
        )
        .unwrap();
    statement
        .query_map([entry_id], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, i64>(1)? / 100,
                r.get::<_, i64>(2)? / 100,
                r.get::<_, Option<String>>(3)?,
            ))
        })
        .unwrap()
        .map(Result::unwrap)
        .collect()
}

fn lines(expected: &[(&str, i64, i64)]) -> Vec<(String, i64, i64, Option<String>)> {
    expected
        .iter()
        .map(|&(code, dr, cr)| (code.to_owned(), dr, cr, None))
        .collect()
}

fn scalar(w: &support::World, sql: &str) -> i64 {
    raw(&w.dir).query_row(sql, [], |r| r.get(0)).unwrap()
}

fn save_ok(w: &support::World, draft: &BikriDraft, confirm: bool) -> BikriSaved {
    bikri::save(
        &w.env.ctx,
        &w.owner,
        &mandi_services::core::bikri::BikriSaveInput {
            draft: draft.clone(),
            confirm_cash_below_zero: confirm,
        },
    )
    .unwrap()
}

/// Rupees of a partner's share rows of one entry, in partner order.
fn shares_of(w: &support::World, entry_id: i64) -> Vec<i64> {
    let conn = raw(&w.dir);
    let mut statement = conn
        .prepare(
            "SELECT amount_paisa FROM partner_share_rows WHERE journal_entry_id = ?1 ORDER BY partner_id",
        )
        .unwrap();
    statement
        .query_map([entry_id], |r| Ok(r.get::<_, i64>(0)? / 100))
        .unwrap()
        .map(Result::unwrap)
        .collect()
}

fn balance(w: &support::World, id: i64) -> (BalanceSide, i64) {
    let b = ledger::account_balance(&w.env.ctx, id).unwrap();
    (b.side, b.amount_paisa.rupees())
}

/// Event 2 of PRD 4.5: کریم's 10 سیب, sold to the one-time buyer شاہد for cash.
fn event2(w: &support::World) -> BikriDraft {
    let mut line = support::sale("s-1", w.pukhta_buyer, "10", "1200");
    line.pukhta = Some(PukhtaInput {
        name_ur: "شاہد".into(),
        name_en: "Shahid".into(),
        phone: "0300-0000000".into(),
    });
    let mut lot = support::lot(w, "lot-2", w.karim, w.apple, "10");
    lot.freight = Some("1,000".into());
    lot.cash_paid = None;
    lot.sales = vec![line];
    BikriDraft {
        header: HeaderInput {
            truck_no: Some("LHR-0077".into()),
            ..support::header()
        },
        lots: vec![lot],
    }
}

#[test]
fn e2e_1_to_4_event_1_is_saved_with_its_journal_split_and_matching() {
    let w = worked_day();
    let draft = support::event1(&w);
    let before = bikri::preview(&w.env.ctx, &draft).unwrap();
    let saved = save_ok(&w, &draft, false);
    // Preview and save are one calculation (Code Standards B5).
    assert_eq!(saved.preview, before);
    assert_eq!((saved.bikri_id, saved.number), (1, 1));

    // E2E-1: خام 29,400; کل اخراجات 10,084; صافی 19,316, as stored on the lot.
    let detail = bikri::get(&w.env.ctx, saved.bikri_id).unwrap();
    let lot = &detail.lots[0];
    assert_eq!(
        (
            lot.gross.rupees(),
            lot.seller_comm.rupees(),
            lot.deductions.rupees(),
            lot.net.rupees()
        ),
        (29_400, 1_764, 10_084, 19_316)
    );
    assert_eq!(
        (lot.cash_paid.rupees(), lot.qty_received, lot.qty_sold),
        (5_000, 50, 50)
    );
    // E2E-2: اکرم 12,000 + 720 + 160 = 12,880; ارشد 17,400 + 1,044 + 240 = 18,684.
    let totals: Vec<i64> = lot.sales.iter().map(|s| s.buyer_total.rupees()).collect();
    assert_eq!(totals, [12_880, 18_684]);
    assert_eq!(
        lot.sales.iter().map(|s| s.line_no).collect::<Vec<_>>(),
        [1, 2]
    );

    // E2E-3: the journal, line by line. Sale part 31,564 = 31,564; whole entry 36,564.
    let journal = entry_lines(&w, saved.entry_id);
    assert_eq!(
        journal,
        lines(&[
            ("0020", 12_880, 0),
            ("0021", 18_684, 0),
            ("0010", 0, 24_316),
            ("04010001", 0, 1_764),
            ("04010002", 0, 1_764),
            ("04010003", 0, 400),
            ("04010004", 0, 20),
            ("02010001", 0, 250),
            ("02010002", 0, 3_000),
            ("02010004", 0, 50),
            ("0010", 5_000, 0),
            ("01010001", 0, 5_000),
        ])
    );
    let sale_part = &journal[..10];
    assert_eq!(sale_part.iter().map(|l| l.1).sum::<i64>(), 31_564);
    assert_eq!(sale_part.iter().map(|l| l.2).sum::<i64>(), 31_564);
    assert_eq!(journal.iter().map(|l| l.1).sum::<i64>(), 36_564);
    assert_eq!(journal.iter().map(|l| l.2).sum::<i64>(), 36_564);

    // E2E-4: shared income 3,948; split 1,580 / 1,184 / 1,184 (written by the plan 11 hook).
    assert_eq!(saved.preview.summary.shared_income.rupees(), 3_948);
    assert_eq!(shares_of(&w, saved.entry_id), [1_580, 1_184, 1_184]);

    // Cash 50,000 - 5,000 = 45,000.
    assert_eq!(
        balance(&w, support::id_of(&w.env, "01010001")),
        (BalanceSide::Dr, 45_000)
    );
    // Advance matching (PRD FR-15.8): 15,000 of رحیم's advance adjusted, fully; payable
    // 24,316 - 5,000 - 15,000 = 4,316 = his ledger (15,000 + 5,000 Dr, 24,316 Cr).
    assert_eq!(
        scalar(
            &w,
            "SELECT COALESCE(SUM(amount_paisa), 0) FROM advance_allocations WHERE is_active = 1"
        ),
        1_500_000
    );
    assert_eq!(
        assist::advances(&w.env.ctx, w.rahim).unwrap().open_paisa,
        Paisa::ZERO
    );
    assert_eq!(balance(&w, w.rahim), (BalanceSide::Cr, 4_316));
    // Buyers owe their bills: nothing is matched on a credit line.
    assert_eq!(balance(&w, w.akram), (BalanceSide::Dr, 12_880));
    assert_eq!(balance(&w, w.arshad), (BalanceSide::Dr, 18_684));
    assert_eq!(scalar(&w, "SELECT COUNT(*) FROM bill_allocations"), 0);
    // Stock: 50 received, 20 and 30 sold, nothing left; two facts; one audit row.
    assert_eq!(
        scalar(&w, "SELECT COALESCE(SUM(qty), 0) FROM stock_moves"),
        0
    );
    assert_eq!(scalar(&w, "SELECT COUNT(*) FROM stock_moves"), 3);
    assert_eq!(scalar(&w, "SELECT COUNT(*) FROM analytics_facts"), 2);
    assert_eq!(
        scalar(
            &w,
            "SELECT COUNT(*) FROM audit_log WHERE entity = 'bikri' AND action = 'create'"
        ),
        1
    );
    // The books balance and both chains are intact.
    let trial = ledger::trial_balance(&w.env.ctx, None).unwrap();
    assert_eq!(trial.total_dr, trial.total_cr);
    ledger::verify_journal_chain(&w.env.ctx).unwrap();
    assert_chain_ok(&w.dir);
}

#[test]
fn e2e_5_event_2_with_the_one_time_buyer_is_saved_and_paid() {
    let w = worked_day();
    save_ok(&w, &support::event1(&w), false);
    let draft = event2(&w);
    let saved = save_ok(&w, &draft, false);
    assert_eq!(saved.number, 2);

    // Buyer pays 12,800; seller صافی 10,160.
    let detail = bikri::get(&w.env.ctx, saved.bikri_id).unwrap();
    let lot = &detail.lots[0];
    assert_eq!(lot.net.rupees(), 10_160);
    assert_eq!(lot.deductions.rupees(), 1_840);
    assert_eq!(lot.sales[0].buyer_total.rupees(), 12_800);
    support::assert_totals_equal(&detail.totals, &saved.preview.summary, "Event 2");
    assert_eq!(
        (
            detail.totals.buyer_cash.rupees(),
            detail.totals.buyer_credit.rupees()
        ),
        (12_800, 0)
    );
    // The one-time person is kept on the line with his key (PRD FR-13.5).
    let person = lot.sales[0].pukhta.as_ref().unwrap();
    assert_eq!(
        (
            person.name_ur.as_str(),
            person.phone.as_str(),
            person.person_key.as_str()
        ),
        ("شاہد", "03000000000", "شاہد|03000000000")
    );

    // The journal: sale part 12,800 = 12,800; cash part 12,800; whole entry 25,600.
    let journal = entry_lines(&w, saved.entry_id);
    let key = Some("شاہد|03000000000".to_owned());
    let expected = vec![
        ("0030".to_owned(), 12_800, 0, key.clone()),
        ("0011".to_owned(), 0, 10_160, None),
        ("04010001".to_owned(), 0, 720, None),
        ("04010002".to_owned(), 0, 720, None),
        ("04010003".to_owned(), 0, 80, None),
        ("04010004".to_owned(), 0, 20, None),
        ("02010001".to_owned(), 0, 50, None),
        ("02010002".to_owned(), 0, 1_000, None),
        ("02010004".to_owned(), 0, 50, None),
        ("01010001".to_owned(), 12_800, 0, None),
        ("0030".to_owned(), 0, 12_800, key),
    ];
    assert_eq!(journal, expected);
    assert_eq!(journal[..9].iter().map(|l| l.1).sum::<i64>(), 12_800);
    assert_eq!(journal[..9].iter().map(|l| l.2).sum::<i64>(), 12_800);
    assert_eq!(journal.iter().map(|l| l.1).sum::<i64>(), 25_600);
    assert_eq!(journal.iter().map(|l| l.2).sum::<i64>(), 25_600);

    // Shared income 720 + 720 + 80 + 20 = 1,540; split 616 / 462 / 462.
    assert_eq!(saved.preview.summary.shared_income.rupees(), 1_540);
    assert_eq!(shares_of(&w, saved.entry_id), [616, 462, 462]);
    // Cash 45,000 + 12,800 = 57,800.
    assert_eq!(
        balance(&w, support::id_of(&w.env, "01010001")),
        (BalanceSide::Dr, 57_800)
    );
    // The cash sale is settled at once: pinned to its own cash, so شاہد owes nothing (E2E-5).
    assert_eq!(
        scalar(
            &w,
            "SELECT COUNT(*) FROM bill_allocations WHERE amount_paisa = 1280000"
        ),
        1
    );
    let register = bikri::register(&w.env.ctx, &Default::default()).unwrap();
    assert_eq!(register.rows.len(), 1);
    let row = &register.rows[0];
    assert_eq!(
        (
            row.name_en.as_str(),
            row.phone.as_str(),
            row.amount.rupees(),
            row.commission.rupees(),
            row.paid,
            row.open.rupees(),
            row.bikri_number
        ),
        ("Shahid", "03000000000", 12_800, 720, true, 0, 2)
    );
    // Both postings together: the books balance.
    let trial = ledger::trial_balance(&w.env.ctx, None).unwrap();
    assert_eq!(trial.total_dr, trial.total_cr);
}

#[test]
fn e2e_28_bikri_part_with_the_advance_tick_off() {
    let w = worked_day();
    let mut draft = support::event1(&w);
    draft.lots[0].adjust_advance = false;
    let saved = save_ok(&w, &draft, false);
    // Credit 24,316; nothing adjusted (no row); advance open 15,000; ledger 4,316 جمع.
    assert_eq!(scalar(&w, "SELECT COUNT(*) FROM advance_allocations"), 0);
    assert_eq!(
        assist::advances(&w.env.ctx, w.rahim).unwrap().open_paisa,
        rupees(15_000)
    );
    assert_eq!(balance(&w, w.rahim), (BalanceSide::Cr, 4_316));
    let journal = entry_lines(&w, saved.entry_id);
    assert_eq!(journal[2], ("0010".to_owned(), 0, 24_316, None));
    // The tick is kept on the lot.
    let detail = bikri::get(&w.env.ctx, saved.bikri_id).unwrap();
    assert!(!detail.lots[0].adjust_advance);
}

#[test]
fn the_cash_confirmation_and_a_refused_save_write_nothing() {
    // PRD FR-6.7: cash 20,000 and نقد 25,000 leaves -5,000.
    let w = support::world_with(20_000, true);
    let mut draft = support::event1(&w);
    draft.lots[0].cash_paid = Some("25,000".into());
    let before = scalar(&w, "SELECT COUNT(*) FROM journal_entries");
    let error = bikri::save(
        &w.env.ctx,
        &w.owner,
        &mandi_services::core::bikri::BikriSaveInput {
            draft: draft.clone(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap_err();
    assert_eq!(
        code_and_value(&error, "cash_after_paisa"),
        (ErrorCode::ErrCashBelowZeroNotConfirmed, Some(-500_000))
    );
    assert_eq!(scalar(&w, "SELECT COUNT(*) FROM journal_entries"), before);
    assert_eq!(scalar(&w, "SELECT COUNT(*) FROM bikri_entries"), 0);
    // With the confirmation it saves, and cash is -5,000 + ... (20,000 - 25,000).
    let saved = save_ok(&w, &draft, true);
    assert_eq!(saved.number, 1);
}

// ---------------------------------------------------------------------------------------------
// Plan 15: Event 3 and Event 4 (PRD 4.6, 4.7; E2E-6, E2E-7), E2E-21, E2E-28 (payment part),
// E2E-31 and E2E-32.
// ---------------------------------------------------------------------------------------------

fn receipt_input(account: i64, amount: &str) -> ReceiptInput {
    ReceiptInput {
        date: "10-10-2026".into(),
        account_id: account,
        person_key: None,
        amount: amount.into(),
        removed: mandi_services::core::vouchers::RemovedPaymentFields::default(),
        collector_staff_id: None,
        narration: String::new(),
    }
}

fn payment_input(account: i64, amount: &str, reason: &str) -> PaymentInput {
    PaymentInput {
        date: "10-10-2026".into(),
        account_id: account,
        person_key: None,
        amount: amount.into(),
        removed: mandi_services::core::vouchers::RemovedPaymentFields::default(),
        reason: reason.into(),
        narration: String::new(),
        confirm_cash_below_zero: false,
    }
}

fn preview_of(w: &support::World, entry: VoucherEntry) -> VoucherPreview {
    vouchers::preview(
        &w.env.ctx,
        &VoucherPreviewInput {
            entry,
            replacing_voucher_id: None,
        },
    )
    .unwrap()
}

fn open_bills_of(w: &support::World, account: i64) -> mandi_services::core::vouchers::OpenBills {
    vouchers::open_bills(
        &w.env.ctx,
        &OpenBillsInput {
            account_id: account,
            person_key: None,
        },
    )
    .unwrap()
}

fn payables_of(w: &support::World, seller: i64) -> mandi_services::core::vouchers::OpenPayables {
    vouchers::open_payables(
        &w.env.ctx,
        &OpenPayablesInput {
            seller_id: seller,
            person_key: None,
            include_settled: true,
        },
    )
    .unwrap()
}

#[test]
fn e2e_6_event_3_akram_pays_10000_and_owes_2880() {
    let w = worked_day();
    save_ok(&w, &support::event1(&w), false);
    save_ok(&w, &event2(&w), false);

    // His card before: one open bill, بکری 1 of 10-10-26, 12,880, 0 days (PRD FR-7.58).
    let bills = open_bills_of(&w, w.akram);
    assert_eq!(bills.bills.len(), 1);
    let bill = &bills.bills[0];
    assert_eq!(
        (
            bill.source,
            bill.number,
            bill.entry_date.as_str(),
            bill.amount_paisa.rupees(),
            bill.open_paisa.rupees(),
            bill.days
        ),
        (LineSource::Bikri, Some(1), "2026-10-10", 12_880, 12_880, 0)
    );
    assert_eq!(bills.open_paisa.rupees(), 12_880, "پورا بقایا");

    // The preview: 10,000 settles 10,000 of that bill and leaves 2,880.
    let input = receipt_input(w.akram, "10,000");
    let preview = preview_of(&w, VoucherEntry::Receipt(input.clone()));
    assert!(preview.errors.is_empty() && preview.warnings.is_empty());
    assert_eq!(preview.bill_effects.len(), 1);
    assert_eq!(
        (
            preview.bill_effects[0].bill.line_id == bill.line_id,
            preview.bill_effects[0].settled_paisa.rupees(),
            preview.bill_effects[0].still_open_paisa.rupees()
        ),
        (true, 10_000, 2_880)
    );
    assert_eq!(preview.extra_credit_paisa, Paisa::ZERO);
    assert_eq!(
        (preview.new_balance.side, preview.new_balance.amount_paisa),
        (BalanceSide::Dr, rupees(2_880))
    );
    // A receipt cannot lower cash, so its preview does not read the balance and never asks (PRD
    // FR-6.7). The books hold the numbers: cash 45,000 + 12,800 = 57,800 now.
    assert!(!preview.cash.needs_confirm && !preview.cash.lowers_cash);
    let cash = support::id_of(&w.env, "01010001");
    assert_eq!(balance(&w, cash), (BalanceSide::Dr, 57_800));

    // The save is the preview: the same new balance and the same matching row.
    let saved = vouchers::save_receipt(&w.env.ctx, &w.owner, &input).unwrap();
    assert_eq!(
        (saved.voucher_type, saved.number),
        (VoucherType::Receipt, 1)
    );
    assert_eq!(saved.new_balance, preview.new_balance);
    assert_eq!(balance(&w, w.akram), (BalanceSide::Dr, 2_880));
    assert_eq!(balance(&w, cash), (BalanceSide::Dr, 67_800));
    assert_eq!(
        scalar(
            &w,
            "SELECT COALESCE(SUM(amount_paisa), 0) FROM bill_allocations WHERE is_active = 1 AND account_id = (SELECT id FROM accounts WHERE code = '0020')"
        ),
        1_000_000
    );
    let journal = entry_lines(&w, saved.entry_id);
    assert_eq!(
        journal,
        lines(&[("01010001", 10_000, 0), ("0020", 0, 10_000)])
    );

    // E2E-6: open 2,880 بنام; the oldest open bill is بکری 1 of 10-10-26, 0 days.
    let bills = open_bills_of(&w, w.akram);
    assert_eq!(bills.bills.len(), 1);
    assert_eq!(
        (
            bills.bills[0].number,
            bills.bills[0].open_paisa.rupees(),
            bills.bills[0].days,
            bills.oldest_days
        ),
        (Some(1), 2_880, 0, Some(0))
    );
    assert_eq!(bills.unmatched_credit_paisa, Paisa::ZERO);
    assert_eq!(
        (bills.balance.side, bills.balance.amount_paisa),
        (BalanceSide::Dr, rupees(2_880))
    );
    // Cash 57,800 + 10,000, no partner share rows (a receipt touches no income, PRD FR-6.5).
    assert_eq!(
        balance(&w, support::id_of(&w.env, "01010001")),
        (BalanceSide::Dr, 67_800)
    );
    assert_eq!(
        scalar(
            &w,
            &format!(
                "SELECT COUNT(*) FROM partner_share_rows WHERE journal_entry_id = {}",
                saved.entry_id
            )
        ),
        0
    );
    let trial = ledger::trial_balance(&w.env.ctx, None).unwrap();
    assert_eq!(trial.total_dr, trial.total_cr);
    ledger::verify_journal_chain(&w.env.ctx).unwrap();
    assert_chain_ok(&w.dir);
}

#[test]
fn e2e_7_event_4_rahim_is_paid_4316() {
    let w = worked_day();
    save_ok(&w, &support::event1(&w), false);
    save_ok(&w, &event2(&w), false);
    vouchers::save_receipt(&w.env.ctx, &w.owner, &receipt_input(w.akram, "10,000")).unwrap();

    // What is payable to رحیم: credit 24,316, advance adjusted 15,000, paid 5,000 (نقد), open
    // 4,316 (PRD FR-7.59, E2E-7).
    let payables = payables_of(&w, w.rahim);
    assert_eq!(payables.postings.len(), 1);
    let posting = &payables.postings[0];
    assert_eq!(
        (
            posting.source,
            posting.number,
            posting.credit_paisa.rupees(),
            posting.advance_adjusted_paisa.rupees(),
            posting.paid_paisa.rupees(),
            posting.still_open_paisa.rupees()
        ),
        (LineSource::Bikri, Some(1), 24_316, 15_000, 5_000, 4_316)
    );
    assert_eq!(
        (
            payables.open_paisa.rupees(),
            payables.advances_open_paisa.rupees(),
            payables.owes_firm_paisa.rupees()
        ),
        (4_316, 0, 0),
        "پورا بقایا is 4,316"
    );
    assert_eq!(
        (payables.balance.side, payables.balance.amount_paisa),
        (BalanceSide::Cr, rupees(4_316))
    );

    // The preview: payable 4,316, no warning, the new balance is 0 (a seller with something
    // payable never gets the پیشگی note).
    let input = payment_input(w.rahim, "4,316", "");
    let preview = preview_of(&w, VoucherEntry::Payment(input.clone()));
    assert!(preview.errors.is_empty() && preview.warnings.is_empty());
    assert_eq!(preview.payable_open_paisa, Some(rupees(4_316)));
    assert!(!preview.payable_is_zero_for_seller);
    assert_eq!(preview.new_balance.side, BalanceSide::Zero);
    assert_eq!(preview.cash.cash_after_paisa, rupees(67_800 - 4_316));

    let saved = vouchers::save_payment(&w.env.ctx, &w.owner, &input).unwrap();
    assert_eq!(
        (saved.voucher_type, saved.number),
        (VoucherType::Payment, 1)
    );
    assert_eq!(saved.new_balance, preview.new_balance);
    assert_eq!(balance(&w, w.rahim).0, BalanceSide::Zero);
    assert_eq!(
        entry_lines(&w, saved.entry_id),
        lines(&[("0010", 4_316, 0), ("01010001", 0, 4_316)])
    );

    // After it: nothing payable, nothing owed, and the posting is paid in full.
    let payables = payables_of(&w, w.rahim);
    assert_eq!(payables.open_paisa, Paisa::ZERO);
    assert_eq!(
        (
            payables.postings[0].paid_paisa.rupees(),
            payables.postings[0].still_open_paisa.rupees()
        ),
        (9_316, 0)
    );
    let shown = vouchers::open_payables(
        &w.env.ctx,
        &OpenPayablesInput {
            seller_id: w.rahim,
            person_key: None,
            include_settled: false,
        },
    )
    .unwrap();
    assert!(shown.postings.is_empty());

    // The پیشگی report: given 15,000, returned 0, adjusted 15,000, open 0 (E2E-7).
    let report = vouchers::advances_report(
        &w.env.ctx,
        &AdvancesReportInput {
            seller_id: Some(w.rahim),
            as_of: String::new(),
            offset: 0,
            limit: 0,
        },
    )
    .unwrap();
    assert_eq!(report.rows.len(), 1);
    let row = &report.rows[0];
    assert_eq!(
        (
            row.given_paisa.rupees(),
            row.returned_paisa.rupees(),
            row.adjusted_paisa.rupees(),
            row.open_paisa.rupees(),
            row.last_date.as_deref()
        ),
        (15_000, 0, 15_000, 0, Some("2026-09-01"))
    );
    // Cash: 57,800 + 10,000 - 4,316 = 63,484.
    assert_eq!(
        balance(&w, support::id_of(&w.env, "01010001")),
        (BalanceSide::Dr, 63_484)
    );
    let trial = ledger::trial_balance(&w.env.ctx, None).unwrap();
    assert_eq!(trial.total_dr, trial.total_cr);
}

/// Event 5 of PRD 4.8 as plain journal postings (the خرچہ screen comes in plan 17): electricity
/// 3,000 as دکان خرچہ, split between the partners by the hook of plan 11, and الف's 2,000 as
/// ذاتی خرچہ. Cash goes down by 5,000.
fn event5_direct(w: &support::World) {
    use mandi_services::core::ids::AccountId;
    use mandi_services::core::masters::{ExpenseHeadInput, MasterInput};
    use mandi_services::domain::ledger::{LedgerLine, PostingType, Side, SourceType, build_entry};
    use mandi_services::ledger::PostingEvent;
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
        match mandi_services::masters::save(&w.env.ctx, &w.owner, &MasterInput::ExpenseHead(input))
        {
            Ok(_) => break,
            Err(mandi_services::ServiceError::Rejected(items))
                if items
                    .iter()
                    .all(|i| i.code == ErrorCode::ErrSpellingUnconfirmed) =>
            {
                for item in items {
                    if let Some(mandi_services::core::error::ErrorValue::Text(text)) =
                        item.values.get("words")
                    {
                        words.extend(text.split(' ').map(str::to_owned));
                    }
                }
            }
            Err(other) => panic!("{other:?}"),
        }
    }
    let expense = support::id_of(&w.env, "05010001");
    let drawings = support::id_of(&w.env, "06030001");
    let cash = support::id_of(&w.env, "01010001");
    let date = mandi_services::core::dates::BusinessDate::from_ymd(2026, 10, 10).unwrap();
    let post = |posting_type: PostingType, debit: i64, amount: i64, source_id: i64| {
        let entry = build_entry(vec![
            LedgerLine::new(AccountId::new(debit), Side::Dr, rupees(amount)),
            LedgerLine::new(AccountId::new(cash), Side::Cr, rupees(amount)),
        ])
        .unwrap();
        w.env
            .ctx
            .db()
            .write(|tx| {
                ledger::post(
                    &w.env.ctx,
                    tx,
                    w.owner.user_id,
                    &PostingEvent {
                        date,
                        posting_type,
                        source_type: SourceType::Voucher,
                        source_id,
                        narration: None,
                        reverses: None,
                        entry,
                    },
                )
            })
            .unwrap();
    };
    post(PostingType::ShopExpense, expense, 3_000, 9_001);
    post(PostingType::PersonalExpense, drawings, 2_000, 9_002);
}

#[test]
fn fr_15_5_a_receipt_above_the_bill_stays_as_credit_for_the_next_bill() {
    let w = worked_day();
    save_ok(&w, &support::event1(&w), false);
    // اکرم pays 15,000 against 12,880: 12,880 settled, 2,120 stays as his credit.
    let input = receipt_input(w.akram, "15,000");
    let preview = preview_of(&w, VoucherEntry::Receipt(input.clone()));
    assert_eq!(preview.bill_effects.len(), 1);
    assert_eq!(preview.bill_effects[0].settled_paisa.rupees(), 12_880);
    assert_eq!(preview.extra_credit_paisa.rupees(), 2_120);
    assert_eq!(
        (preview.new_balance.side, preview.new_balance.amount_paisa),
        (BalanceSide::Cr, rupees(2_120))
    );
    vouchers::save_receipt(&w.env.ctx, &w.owner, &input).unwrap();
    let bills = open_bills_of(&w, w.akram);
    assert!(bills.bills.is_empty());
    assert_eq!(bills.unmatched_credit_paisa.rupees(), 2_120);
    assert_eq!(bills.balance.side, BalanceSide::Cr);

    // His next bill (بکری 3, 5 × 600 with no commission or لاگا, 3,000) is settled by the
    // waiting credit at once: 2,120 of it, so 880 is open.
    let mut line = support::sale("s-9", w.akram, "5", "600");
    line.buyer_comm = support::typed(mandi_services::core::bikri::LineChargeMethod::Percent, "0");
    line.seller_comm = support::typed(mandi_services::core::bikri::LineChargeMethod::Percent, "0");
    line.laga = support::typed(mandi_services::core::bikri::LineChargeMethod::PerUnit, "0");
    line.payment_type = Some(mandi_services::core::bikri::PaymentType::Credit);
    let mut lot = support::bare_lot(&w, "lot-9", w.karim, w.apple, "5");
    lot.sales = vec![line];
    let draft = support::draft(vec![lot]);
    save_ok(&w, &draft, false);
    let bills = open_bills_of(&w, w.akram);
    assert_eq!(bills.bills.len(), 1);
    assert_eq!(
        (
            bills.bills[0].amount_paisa.rupees(),
            bills.bills[0].open_paisa.rupees()
        ),
        (3_000, 880)
    );
    assert_eq!(bills.unmatched_credit_paisa, Paisa::ZERO);
    assert_eq!(
        (bills.balance.side, bills.balance.amount_paisa),
        (BalanceSide::Dr, rupees(880))
    );
}

#[test]
fn e2e_21_two_one_time_buyers_are_matched_per_person() {
    use mandi_services::core::bikri::{LineChargeMethod, PaymentType};
    let w = worked_day();
    // شاہد buys 1 × 1,000 and ناصر 2 × 1,000, both on credit, with no commission or لاگا.
    let line = |key: &str, qty: &str, name_ur: &str, name_en: &str, phone: &str| {
        let mut l = support::sale(key, w.pukhta_buyer, qty, "1000");
        l.pukhta = Some(PukhtaInput {
            name_ur: name_ur.into(),
            name_en: name_en.into(),
            phone: phone.into(),
        });
        l.payment_type = Some(PaymentType::Credit);
        l.buyer_comm = support::typed(LineChargeMethod::Percent, "0");
        l.seller_comm = support::typed(LineChargeMethod::Percent, "0");
        l.laga = support::typed(LineChargeMethod::PerUnit, "0");
        l
    };
    let mut lot = support::bare_lot(&w, "lot-1", w.rahim, w.banana, "3");
    lot.sales = vec![
        line("s-1", "1", "شاہد", "Shahid", "0300-0000000"),
        line("s-2", "2", "ناصر", "Nasir", "0300-1111111"),
    ];
    save_ok(&w, &support::draft(vec![lot]), false);
    let shahid = mandi_services::core::text::person_key("شاہد", "0300-0000000");
    let nasir = mandi_services::core::text::person_key("ناصر", "0300-1111111");
    let person_balance = |key: &str| {
        let b = ledger::person_balance(&w.env.ctx, w.pukhta_buyer, key).unwrap();
        (b.side, b.amount_paisa.rupees())
    };
    assert_eq!(person_balance(&shahid), (BalanceSide::Dr, 1_000));
    assert_eq!(person_balance(&nasir), (BalanceSide::Dr, 2_000));

    // A receipt needs the person (PRD FR-13.5, FR-13.7) and he must be on the books.
    let mut input = receipt_input(w.pukhta_buyer, "1,000");
    let refused = vouchers::save_receipt(&w.env.ctx, &w.owner, &input).unwrap_err();
    assert_eq!(refused.code(), ErrorCode::ErrPersonRequired);
    input.person_key = Some("نامعلوم|03000000000".into());
    let refused = vouchers::save_receipt(&w.env.ctx, &w.owner, &input).unwrap_err();
    assert_eq!(refused.code(), ErrorCode::ErrNotFound);

    // The bills of one person are his alone.
    let bills = vouchers::open_bills(
        &w.env.ctx,
        &OpenBillsInput {
            account_id: w.pukhta_buyer,
            person_key: Some(shahid.clone()),
        },
    )
    .unwrap();
    assert_eq!(bills.open_paisa.rupees(), 1_000);
    assert_eq!(bills.bills.len(), 1);

    // E2E-21: the receipt of 1,000 for شاہد: شاہد 0, ناصر 2,000 still due.
    input.person_key = Some(shahid.clone());
    let preview = preview_of(&w, VoucherEntry::Receipt(input.clone()));
    assert_eq!(preview.bill_effects.len(), 1);
    assert_eq!(preview.bill_effects[0].settled_paisa.rupees(), 1_000);
    assert_eq!(preview.new_balance.side, BalanceSide::Zero);
    let saved = vouchers::save_receipt(&w.env.ctx, &w.owner, &input).unwrap();
    assert_eq!(saved.new_balance.side, BalanceSide::Zero);
    assert_eq!(person_balance(&shahid).0, BalanceSide::Zero);
    assert_eq!(person_balance(&nasir), (BalanceSide::Dr, 2_000));
    let nasir_bills = vouchers::open_bills(
        &w.env.ctx,
        &OpenBillsInput {
            account_id: w.pukhta_buyer,
            person_key: Some(nasir),
        },
    )
    .unwrap();
    assert_eq!(
        (nasir_bills.open_paisa.rupees(), nasir_bills.bills.len()),
        (2_000, 1),
        "شاہد's payment never settles ناصر's bill"
    );
    // The voucher keeps the person key, and the list shows him.
    let row = vouchers::get(&w.env.ctx, saved.voucher_id).unwrap();
    assert_eq!(
        row.person.as_ref().map(|p| p.person_key.as_str()),
        Some(shahid.as_str())
    );
    assert_eq!(
        row.person.as_ref().map(|p| p.name_en.as_str()),
        Some("Shahid")
    );
}

#[test]
fn e2e_28_payment_part_the_advance_stays_open_after_the_payment() {
    let w = worked_day();
    let mut draft = support::event1(&w);
    draft.lots[0].adjust_advance = false;
    save_ok(&w, &draft, false);

    // Payable 24,316 - 5,000 = 19,316; advance open 15,000; ledger 4,316 جمع.
    let p = payables_of(&w, w.rahim);
    assert_eq!(
        (
            p.postings[0].advance_adjusted_paisa.rupees(),
            p.postings[0].still_open_paisa.rupees(),
            p.open_paisa.rupees(),
            p.advances_open_paisa.rupees(),
            p.owes_firm_paisa.rupees()
        ),
        (0, 19_316, 19_316, 15_000, 0)
    );
    assert_eq!(
        (p.balance.side, p.balance.amount_paisa),
        (BalanceSide::Cr, rupees(4_316))
    );

    // After paying 19,316: payable 0, ledger 15,000 بنام, advance open 15,000, and he is not in
    // "بیوپاری کے ذمے".
    let saved = vouchers::save_payment(&w.env.ctx, &w.owner, &payment_input(w.rahim, "19,316", ""))
        .unwrap();
    assert_eq!(
        (saved.new_balance.side, saved.new_balance.amount_paisa),
        (BalanceSide::Dr, rupees(15_000))
    );
    let p = payables_of(&w, w.rahim);
    assert_eq!(
        (
            p.open_paisa.rupees(),
            p.advances_open_paisa.rupees(),
            p.owes_firm_paisa.rupees()
        ),
        (0, 15_000, 0)
    );
    assert_eq!(
        assist::advances(&w.env.ctx, w.rahim).unwrap().open_paisa,
        rupees(15_000)
    );
    // A payment never adjusts an advance.
    assert_eq!(scalar(&w, "SELECT COUNT(*) FROM advance_allocations"), 0);
    let trial = ledger::trial_balance(&w.env.ctx, None).unwrap();
    assert_eq!(trial.total_dr, trial.total_cr);

    // Paying more than is payable warns in the preview and still saves; the extra is what he
    // owes the firm, and it is not an advance (PRD FR-15.7).
    let preview = preview_of(
        &w,
        VoucherEntry::Payment(payment_input(w.rahim, "1,000", "")),
    );
    assert!(preview.payable_is_zero_for_seller, "PRD FR-7.68");
    assert_eq!(preview.warnings.len(), 1);
    assert_eq!(preview.warnings[0].code, ErrorCode::ErrPaymentAbovePayable);
    assert_eq!(preview.warnings[0].field, "amount");
}

#[test]
fn e2e_31_a_seller_returns_10000_of_his_advance() {
    let w = worked_day();
    // Before: 15,000 of advance is open. The preview of a return of 10,000.
    assert_eq!(
        assist::advances(&w.env.ctx, w.rahim).unwrap().open_paisa,
        rupees(15_000)
    );
    let input = receipt_input(w.rahim, "10,000");
    let preview = preview_of(&w, VoucherEntry::Receipt(input.clone()));
    assert!(preview.errors.is_empty());
    assert_eq!(preview.return_limit_paisa, Some(rupees(15_000)));
    assert_eq!(preview.advance_effects.len(), 1);
    assert_eq!(
        (
            preview.advance_effects[0].advance.voucher_number,
            preview.advance_effects[0].adjusted_paisa.rupees(),
            preview.advance_effects[0].still_open_paisa.rupees()
        ),
        (Some(1), 10_000, 5_000)
    );
    assert_eq!(preview.owes_settled_paisa, Paisa::ZERO);

    // More than he can return is refused with the open amount (PRD FR-15.12).
    let refused = vouchers::save_receipt(&w.env.ctx, &w.owner, &receipt_input(w.rahim, "15,001"))
        .unwrap_err();
    assert_eq!(
        code_and_value(&refused, "open_paisa"),
        (ErrorCode::ErrReturnExceedsOpen, Some(1_500_000))
    );
    assert_eq!(
        scalar(&w, "SELECT COUNT(*) FROM vouchers WHERE type = 'receipt'"),
        0
    );

    // After the return: advance open 5,000; cash 60,000.
    let saved = vouchers::save_receipt(&w.env.ctx, &w.owner, &input).unwrap();
    assert_eq!(
        (saved.new_balance.side, saved.new_balance.amount_paisa),
        (BalanceSide::Dr, rupees(5_000))
    );
    assert_eq!(
        assist::advances(&w.env.ctx, w.rahim).unwrap().open_paisa,
        rupees(5_000)
    );
    assert_eq!(
        balance(&w, support::id_of(&w.env, "01010001")),
        (BalanceSide::Dr, 60_000)
    );

    // Then Event 1: the last 5,000 is adjusted; the report says given 15,000, returned 10,000,
    // adjusted 5,000, open 0; payable 24,316 - 5,000 - 5,000 = 14,316; ledger 14,316 جمع.
    save_ok(&w, &support::event1(&w), false);
    let report = vouchers::advances_report(
        &w.env.ctx,
        &AdvancesReportInput {
            seller_id: None,
            as_of: String::new(),
            offset: 0,
            limit: 0,
        },
    )
    .unwrap();
    assert_eq!(report.rows.len(), 1);
    let row = &report.rows[0];
    assert_eq!(
        (
            row.given_paisa.rupees(),
            row.returned_paisa.rupees(),
            row.adjusted_paisa.rupees(),
            row.open_paisa.rupees()
        ),
        (15_000, 10_000, 5_000, 0)
    );
    assert_eq!(
        (
            report.given_paisa.rupees(),
            report.returned_paisa.rupees(),
            report.adjusted_paisa.rupees(),
            report.open_paisa.rupees()
        ),
        (15_000, 10_000, 5_000, 0)
    );
    let p = payables_of(&w, w.rahim);
    assert_eq!(
        (
            p.open_paisa.rupees(),
            p.advances_open_paisa.rupees(),
            p.owes_firm_paisa.rupees()
        ),
        (14_316, 0, 0)
    );
    assert_eq!(balance(&w, w.rahim), (BalanceSide::Cr, 14_316));
    // The report as of a day before the return shows the advance as it was.
    let earlier = vouchers::advances_report(
        &w.env.ctx,
        &AdvancesReportInput {
            seller_id: Some(w.rahim),
            as_of: "09-10-2026".into(),
            offset: 0,
            limit: 0,
        },
    )
    .unwrap();
    assert_eq!(
        (
            earlier.rows[0].given_paisa.rupees(),
            earlier.rows[0].returned_paisa.rupees(),
            earlier.rows[0].adjusted_paisa.rupees(),
            earlier.rows[0].open_paisa.rupees()
        ),
        (15_000, 0, 0, 15_000)
    );
    let trial = ledger::trial_balance(&w.env.ctx, None).unwrap();
    assert_eq!(trial.total_dr, trial.total_cr);
}

#[test]
fn e2e_32_cash_lent_to_a_buyer_is_one_of_his_bills() {
    let w = worked_day();
    // Events 1 to 4, then Event 5 as plain postings: cash is 58,484 (E2E-8).
    save_ok(&w, &support::event1(&w), false);
    save_ok(&w, &event2(&w), false);
    vouchers::save_receipt(&w.env.ctx, &w.owner, &receipt_input(w.akram, "10,000")).unwrap();
    vouchers::save_payment(&w.env.ctx, &w.owner, &payment_input(w.rahim, "4,316", "")).unwrap();
    event5_direct(&w);
    let cash = support::id_of(&w.env, "01010001");
    assert_eq!(balance(&w, cash), (BalanceSide::Dr, 58_484));

    // A payment to a buyer needs a reason (PRD FR-15.11), checked at its field.
    let refused =
        vouchers::save_payment(&w.env.ctx, &w.owner, &payment_input(w.akram, "3,000", ""))
            .unwrap_err();
    assert_eq!(
        (refused.code(), first_field(&refused)),
        (
            ErrorCode::ErrPaymentToBuyerNeedsReason,
            Some("reason".to_owned())
        )
    );
    let preview = preview_of(
        &w,
        VoucherEntry::Payment(payment_input(w.akram, "3,000", "")),
    );
    assert_eq!(preview.errors.len(), 1);
    assert_eq!(
        preview.errors[0].code,
        ErrorCode::ErrPaymentToBuyerNeedsReason
    );

    // ادائیگی of 3,000 cash to اکرم, reason نقد ادھار.
    let input = payment_input(w.akram, "3,000", "نقد ادھار");
    let saved = vouchers::save_payment(&w.env.ctx, &w.owner, &input).unwrap();
    assert_eq!(
        (saved.new_balance.side, saved.new_balance.amount_paisa),
        (BalanceSide::Dr, rupees(5_880))
    );
    // The 3,000 is a bill dated 10-10-26 after his بکری 1 bill: 2,880 + 3,000 = 5,880.
    let bills = open_bills_of(&w, w.akram);
    assert_eq!(bills.bills.len(), 2);
    assert_eq!(
        (
            bills.bills[0].source,
            bills.bills[0].number,
            bills.bills[0].open_paisa.rupees()
        ),
        (LineSource::Bikri, Some(1), 2_880)
    );
    assert_eq!(
        (
            bills.bills[1].source,
            bills.bills[1].number,
            bills.bills[1].entry_date.as_str(),
            bills.bills[1].open_paisa.rupees()
        ),
        (LineSource::CashLent, Some(2), "2026-10-10", 3_000)
    );
    // The loan is payment ا-2 (the payment to رحیم was ا-1).
    assert_eq!(bills.open_paisa.rupees(), 5_880);
    // Cash 55,484; Σ buyer debit balances 21,564 + 3,000 = 24,564.
    assert_eq!(balance(&w, cash), (BalanceSide::Dr, 55_484));
    let dues = balance(&w, w.akram).1 + balance(&w, w.arshad).1;
    assert_eq!(dues, 2_880 + 3_000 + 18_684);
    assert_eq!(dues, 24_564);
    // Collected like any bill: a receipt of 2,880 settles the older bill first.
    let preview = preview_of(&w, VoucherEntry::Receipt(receipt_input(w.akram, "2,880")));
    assert_eq!(preview.bill_effects.len(), 1);
    assert_eq!(preview.bill_effects[0].bill.number, Some(1));
    assert_eq!(preview.bill_effects[0].bill.source, LineSource::Bikri);
}

fn first_field(error: &mandi_services::ServiceError) -> Option<String> {
    match error {
        mandi_services::ServiceError::Rejected(items) => {
            items.first().and_then(|i| i.field.clone())
        }
        _ => None,
    }
}

// Plan 16: edit, cancel, later sale (E2E-13, E2E-13b, E2E-14, E2E-15, E2E-16, E2E-16b).
#[path = "golden_day/edit.rs"]
mod edit;

// Plan 16b: the goods of a market trip sold by an own lot (E2E-34, E2E-34b, E2E-41).
#[path = "golden_day/own_goods.rs"]
mod own_goods;

// Plan 17: the cash book, the one خرچہ screen and ادھار رقم on the worked day (E2E-8, E2E-9,
// E2E-10, E2E-12, E2E-20, E2E-29, E2E-30, E2E-30b, E2E-39 preview).
#[path = "golden_day/day_money.rs"]
mod day_money;

// Plan 17: partner capital, the partner total and its statement, الگ کھاتہ and the drawings
// opening (E2E-35, E2E-37, E2E-38, D4-A31).
#[path = "golden_day/partner_money.rs"]
mod partner_money;

// Plan 17: the full day close summary (FR-21.1, PRD 4.9).
#[path = "golden_day/day_close.rs"]
mod day_close;

// Plan 17c: gardens (E2E-40, E2E-40b) and their refusals.
#[path = "golden_day/gardens.rs"]
mod gardens;

// Plan 17d: fruit taken (E2E-42, C27) and its refusals.
#[path = "golden_day/fruit.rs"]
mod fruit_taken;
