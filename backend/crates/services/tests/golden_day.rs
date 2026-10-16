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
    AdvanceInput, DayCloseInput, DayClosePreviewInput, VoucherMode, VoucherType, VouchersListInput,
};
use mandi_services::testing::TestEnv;
use mandi_services::{accounts, assist, auth, bikri, day, ledger, vouchers};
use tempfile::TempDir;

/// The accounts of the worked day that exist after the opening position.
pub struct Position {
    pub dir: TempDir,
    pub env: TestEnv,
    pub owner: auth::Actor,
    /// رحیم, seller 030110001.
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
            date: "01-09-2026".into(),
            seller_account_id: rahim,
            amount: "15,000".into(),
            mode: VoucherMode::Cash,
            bank_account_id: None,
            cheque_no: String::new(),
            cheque_date: String::new(),
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
        row("030110001").map(|b| (b.side, b.amount_paisa)),
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
            voucher_type: Some(VoucherType::Advance),
            from: String::new(),
            to: String::new(),
        },
    )
    .unwrap();
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
            date: "09-10-2026".into(),
            seller_account_id: p.rahim,
            amount: "100".into(),
            mode: VoucherMode::Cash,
            bank_account_id: None,
            cheque_no: String::new(),
            cheque_date: String::new(),
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
            ("03040001", 12_880, 0),
            ("03040002", 18_684, 0),
            ("030110001", 0, 24_316),
            ("04010001", 0, 1_764),
            ("04010002", 0, 1_764),
            ("04010003", 0, 400),
            ("04010004", 0, 20),
            ("02010001", 0, 250),
            ("02010002", 0, 3_000),
            ("02010004", 0, 50),
            ("030110001", 5_000, 0),
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
        ("03049999".to_owned(), 12_800, 0, key.clone()),
        ("030110002".to_owned(), 0, 10_160, None),
        ("04010001".to_owned(), 0, 720, None),
        ("04010002".to_owned(), 0, 720, None),
        ("04010003".to_owned(), 0, 80, None),
        ("04010004".to_owned(), 0, 20, None),
        ("02010001".to_owned(), 0, 50, None),
        ("02010002".to_owned(), 0, 1_000, None),
        ("02010004".to_owned(), 0, 50, None),
        ("01010001".to_owned(), 12_800, 0, None),
        ("03049999".to_owned(), 0, 12_800, key),
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
    assert_eq!(register.len(), 1);
    let row = &register[0];
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
    assert_eq!(journal[2], ("030110001".to_owned(), 0, 24_316, None));
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
