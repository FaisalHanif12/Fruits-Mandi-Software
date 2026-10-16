//! The PRD worked day, through the real services with an injected clock (PRD Section 4 and 27,
//! Code Standards 9.2). The expected numbers come from the PRD and are never changed to suit
//! the code (CLAUDE.md "Rules that are never broken").
//!
//! Plan 10 starts the file with the opening position of PRD 4.3 and E2E-19. Later plans add the
//! events of Section 4.4 to 4.8 and E2E-1 to E2E-16b, E2E-20 to E2E-21 and E2E-28 to E2E-33 on
//! top of `opening_position`.
#![cfg(test)]

mod common;

use common::*;
use mandi_services::core::accounts::{
    AccountInput, AccountType, BalanceSide, OpeningLineInput, OpeningsInput,
};
use mandi_services::core::money::Paisa;
use mandi_services::core::vouchers::{
    AdvanceInput, DayCloseInput, DayClosePreviewInput, VoucherMode, VoucherType, VouchersListInput,
};
use mandi_services::testing::TestEnv;
use mandi_services::{accounts, assist, auth, day, ledger, vouchers};
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
