//! Plan 17 T2b on the PRD worked day: partner capital, the partner total (کل رقم) with its dated
//! statement, الگ کھاتہ and the drawings opening (PRD 16.6, 17.3, 17.4, FR-16.7, FR-17.9, FR-17.10;
//! E2E-35, E2E-37, E2E-38, D4-A31). The statement lines of D4-A33 (the E2E-34 run) are in
//! `own_goods`. Every expected number is from the PRD and is never changed to suit the code.
use super::day_money::{event5, events_1_to_4, expense_head};
use super::*;
use mandi_services::core::accounts::{OpeningLineInput, OpeningsInput};
use mandi_services::core::bikri::LotInput;
use mandi_services::core::capital::{CapitalHistoryInput, CapitalInput, CapitalRowKind};
use mandi_services::core::error::ErrorValue;
use mandi_services::core::expenses::{PartnerTakesInput, TakeSource};
use mandi_services::core::khata::{
    KhataCreateInput, KhataInInput, KhataOutInput, KhataRowKind, KhataStatementInput,
};
use mandi_services::core::statement::{PartnersStatementInput, StatementDetail, StatementHeading};
use mandi_services::{ServiceError, khata, partners};

pub fn capital_input(partner: i64, amount: &str, date: &str) -> CapitalInput {
    CapitalInput {
        date: date.into(),
        partner_id: Some(partner),
        amount: amount.into(),
        narration: None,
        confirm_cash_below_zero: false,
    }
}

/// Makes a khata, confirming the spellings the engine is unsure of, and returns its account id.
pub fn make_khata(w: &support::World, ur: &str, en: &str) -> i64 {
    let mut words: Vec<String> = Vec::new();
    for _ in 0..5 {
        let input = KhataCreateInput {
            name_ur: ur.into(),
            name_en: en.into(),
            note: None,
            confirmed_rules_words: words.clone(),
        };
        match khata::create(&w.env.ctx, &w.owner, &input) {
            Ok(done) => return done.account.id,
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
    panic!("the spelling question did not end");
}

pub fn khata_out_input(khata: i64, amount: &str, date: &str) -> KhataOutInput {
    KhataOutInput {
        date: date.into(),
        khata_account_id: Some(khata),
        amount: amount.into(),
        reason: "خاندان کا خرچ".into(),
        narration: None,
        confirm_cash_below_zero: false,
    }
}

pub fn khata_in_input(khata: i64, amount: &str, date: &str) -> KhataInInput {
    KhataInInput {
        date: date.into(),
        khata_account_id: Some(khata),
        amount: amount.into(),
        reason: "واپس آئے".into(),
        narration: None,
        confirm_cash_below_zero: false,
    }
}

fn overview_row(
    w: &support::World,
    partner: i64,
) -> mandi_services::core::partners::PartnerOverviewRow {
    partners::overview(&w.env.ctx)
        .unwrap()
        .into_iter()
        .find(|r| r.partner_id == partner)
        .unwrap()
}

fn statement_of(
    w: &support::World,
    partner: i64,
) -> mandi_services::core::statement::PartnerStatement {
    partners::statement(
        &w.env.ctx,
        &PartnersStatementInput {
            group_by_month: false,
            sort: None,
            partner_id: partner,
            from: None,
            to: None,
            offset: 0,
            limit: 0,
        },
    )
    .unwrap()
}

/// Σ credit − Σ debit of the books for the firm's equity: every income, expense and capital
/// account. It does not use the share rows, so it is a separate proof of the partner totals.
fn books_equity(w: &support::World) -> i64 {
    scalar(
        w,
        "SELECT COALESCE(SUM(l.cr_paisa - l.dr_paisa), 0) / 100 FROM journal_lines l
         JOIN accounts a ON a.id = l.account_id
         WHERE substr(a.code, 1, 2) IN ('04', '05', '06')",
    )
}

/// The worked day, Events 1 to 5 (PRD 4.4 to 4.8), and ب's 50,000 on 11-10-26 (E2E-35).
fn events_1_to_5_and_capital() -> support::World {
    let w = worked_day();
    events_1_to_4(&w);
    let electricity = expense_head(&w, "بجلی", "Electricity");
    event5(&w, electricity);
    set_today(&w.env, 2026, 10, 11);
    vouchers::save_capital_in(
        &w.env.ctx,
        &w.owner,
        &capital_input(w.partners[1], "50,000", "11-10-2026"),
    )
    .unwrap();
    w
}

#[test]
fn e2e_35_partner_b_invests_50000_it_is_never_split_and_never_income() {
    let w = worked_day();
    events_1_to_4(&w);
    let electricity = expense_head(&w, "بجلی", "Electricity");
    event5(&w, electricity);
    assert_eq!(super::day_money::cash_now(&w), 58_484);
    let share_rows_before = scalar(&w, "SELECT COUNT(*) FROM partner_share_rows");
    let b = w.partners[1];
    let before = overview_row(&w, b);
    assert_eq!(before.month_profit_paisa.rupees(), 746);

    set_today(&w.env, 2026, 10, 11);
    let saved = vouchers::save_capital_in(
        &w.env.ctx,
        &w.owner,
        &capital_input(b, "50,000", "11-10-2026"),
    )
    .unwrap();
    assert_eq!(
        (saved.voucher_type, saved.number),
        (VoucherType::CapitalIn, 1)
    );

    // Cash 58,484 + 50,000.
    assert_eq!(super::day_money::cash_now(&w), 108_484);
    // His capital account (0602) holds 50,000 on the credit side, and it is on the account the
    // partner screen names.
    let capital_id = partners::get(&w.env.ctx, b).unwrap().row.capital_account_id;
    assert_eq!(balance(&w, capital_id), (BalanceSide::Cr, 50_000));
    // Capital is not income and is never split: no share row was written, and his profit share is
    // still 746.
    assert_eq!(
        scalar(&w, "SELECT COUNT(*) FROM partner_share_rows"),
        share_rows_before
    );
    let after = overview_row(&w, b);
    assert_eq!(after.month_profit_paisa.rupees(), 746);
    assert_eq!(after.month_capital_in_paisa.rupees(), 50_000);
    assert_eq!(after.capital_balance_paisa.rupees(), 50_000);
    // The shares of the other partners did not change either.
    assert_eq!(
        overview_row(&w, w.partners[0]).month_profit_paisa.rupees(),
        996
    );

    // E2E-35: the trial balance is 135,048 on both sides.
    let trial = ledger::trial_balance(&w.env.ctx, None).unwrap();
    assert_eq!(trial.total_dr, rupees(135_048));
    assert_eq!(trial.total_cr, rupees(135_048));

    // The capital history of ب: one row, +50,000, balance 50,000.
    let history = partners::capital_history(
        &w.env.ctx,
        &CapitalHistoryInput {
            group_by_month: false,
            sort: None,
            partner_id: b,
            offset: 0,
            limit: 0,
        },
    )
    .unwrap();
    assert_eq!(history.balance_paisa.rupees(), 50_000);
    assert_eq!(history.rows.len(), 1);
    assert_eq!(
        (
            history.rows[0].kind,
            history.rows[0].number,
            history.rows[0].amount_paisa.rupees(),
            history.rows[0].running_paisa.rupees(),
            history.rows[0].entry_date.as_str()
        ),
        (
            CapitalRowKind::Invested,
            Some(1),
            50_000,
            50_000,
            "2026-10-11"
        )
    );
    assert_eq!(
        (
            history.totals.invested_paisa.rupees(),
            history.totals.taken_back_paisa.rupees()
        ),
        (50_000, 0)
    );
    ledger::verify_journal_chain(&w.env.ctx).unwrap();
}

#[test]
fn e2e_37_partner_totals_the_statement_of_b_and_the_firm_equity() {
    let w = events_1_to_5_and_capital();
    let [a, b, c] = w.partners;

    // کل رقم: الف 2,196 - 1,200 - 2,000 = -1,004; ب 50,000 + 1,646 - 900 = 50,746; ج 1,646 - 900.
    let totals: Vec<i64> = [a, b, c]
        .iter()
        .map(|p| overview_row(&w, *p).total_paisa.rupees())
        .collect();
    assert_eq!(totals, vec![-1_004, 50_746, 746]);
    // Σ 50,488 + ابتدائی سرمایہ 65,000 = 115,488 = the equity of the books.
    assert_eq!(totals.iter().sum::<i64>(), 50_488);
    assert_eq!(totals.iter().sum::<i64>() + 65_000, 115_488);
    assert_eq!(books_equity(&w), 115_488);
    assert_eq!(books_equity(&w), 65_000 + 50_000 + 5_488 - 3_000 - 2_000);

    // The statement of ب, in date order (PRD E2E-37): the numbers and the running کل رقم.
    let statement = statement_of(&w, b);
    let rows: Vec<_> = statement
        .lines
        .iter()
        .map(|l| {
            (
                l.entry_date.as_str(),
                l.heading,
                l.detail,
                l.number,
                l.amount_paisa.map(Paisa::rupees),
                l.running_paisa.rupees(),
            )
        })
        .collect();
    assert_eq!(
        rows,
        vec![
            (
                "2026-10-10",
                StatementHeading::Bikri,
                StatementDetail::IncomeShare,
                Some(1),
                Some(1_184),
                1_184
            ),
            (
                "2026-10-10",
                StatementHeading::Bikri,
                StatementDetail::IncomeShare,
                Some(2),
                Some(462),
                1_646
            ),
            (
                "2026-10-10",
                StatementHeading::Expense,
                StatementDetail::ExpenseShare,
                Some(1),
                Some(-900),
                746
            ),
            (
                "2026-10-11",
                StatementHeading::Capital,
                StatementDetail::CapitalIn,
                Some(1),
                Some(50_000),
                50_746
            ),
        ]
    );
    // The expense line names its head and the capital line its voucher.
    assert_eq!(
        statement.lines[2]
            .subject
            .as_ref()
            .map(|s| s.name_en.as_str()),
        Some("Electricity")
    );
    assert_eq!(
        (
            statement.opening_total_paisa.rupees(),
            statement.closing_total_paisa.rupees(),
            statement.total_count
        ),
        (0, 50_746, 4)
    );
    assert_eq!(
        (
            statement.totals.plus_paisa.rupees(),
            statement.totals.minus_paisa.rupees()
        ),
        (51_646, 900)
    );

    // The last running total of every partner is his کل رقم.
    for (partner, total) in [a, b, c].iter().zip(&totals) {
        let s = statement_of(&w, *partner);
        assert_eq!(s.closing_total_paisa.rupees(), *total);
        assert_eq!(
            s.lines.last().map(|l| l.running_paisa.rupees()),
            Some(*total)
        );
    }

    // الف's statement shows the 2,000 he took with its reason (PRD FR-17.10).
    let sa = statement_of(&w, a);
    let taken = sa
        .lines
        .iter()
        .find(|l| l.detail == StatementDetail::Taken)
        .unwrap();
    assert_eq!(
        (
            taken.heading,
            taken.number,
            taken.reason.as_str(),
            taken.amount_paisa.map(Paisa::rupees)
        ),
        (
            StatementHeading::Drawings,
            Some(1),
            "گھر کا خرچ",
            Some(-2_000)
        )
    );

    // The overview columns: الف took 2,000 and returned nothing, so his خالص is 996 - 2,000 (C20).
    let row_a = overview_row(&w, a);
    assert_eq!(
        (
            row_a.month_taken_paisa.rupees(),
            row_a.month_returned_paisa.rupees(),
            row_a.month_net_paisa.rupees(),
            row_a.month_ventures_paisa.rupees()
        ),
        (2_000, 0, -1_004, 0)
    );
}

#[test]
fn month_close_leaves_the_partner_total_unchanged() {
    // PRD FR-17.6, FR-17.9: month close moves each partner's profit into his capital account and
    // never changes his کل رقم. The real month close is D5; here the same entry is posted by hand.
    use mandi_domain::core::ids::AccountId;
    use mandi_domain::ledger::{LedgerLine, PostingType, Side, SourceType, build_entry};
    use mandi_services::ledger::PostingEvent;

    let w = worked_day();
    events_1_to_4(&w);
    let electricity = expense_head(&w, "بجلی", "Electricity");
    event5(&w, electricity);
    let totals_before: Vec<i64> = w
        .partners
        .iter()
        .map(|p| overview_row(&w, *p).total_paisa.rupees())
        .collect();
    assert_eq!(totals_before, vec![-1_004, 746, 746]);

    let id = |code: &str| AccountId::new(support::id_of(&w.env, code));
    let cap = |p: i64| AccountId::new(partners::get(&w.env.ctx, p).unwrap().row.capital_account_id);
    let money = |r: i64| rupees(r);
    // Each income account Dr its balance, the expense account Cr its balance, and each partner's
    // capital Cr his profit share: 2,484 + 2,484 + 480 + 40 = 5,488 less 3,000 = 2,488.
    let entry = build_entry(vec![
        LedgerLine::new(id("04010001"), Side::Dr, money(2_484)),
        LedgerLine::new(id("04010002"), Side::Dr, money(2_484)),
        LedgerLine::new(id("04010003"), Side::Dr, money(480)),
        LedgerLine::new(id("04010004"), Side::Dr, money(40)),
        LedgerLine::new(id("05010001"), Side::Cr, money(3_000)),
        LedgerLine::new(cap(w.partners[0]), Side::Cr, money(996)),
        LedgerLine::new(cap(w.partners[1]), Side::Cr, money(746)),
        LedgerLine::new(cap(w.partners[2]), Side::Cr, money(746)),
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
                    date: mandi_services::core::dates::BusinessDate::from_ymd(2026, 10, 10)
                        .unwrap(),
                    posting_type: PostingType::MonthClose,
                    source_type: SourceType::MonthClose,
                    source_id: 1,
                    narration: None,
                    reverses: None,
                    entry,
                },
            )
        })
        .unwrap();

    // The capital accounts now hold the profit, but the totals did not move.
    let a_capital = partners::get(&w.env.ctx, w.partners[0])
        .unwrap()
        .row
        .capital_account_id;
    assert_eq!(balance(&w, a_capital), (BalanceSide::Cr, 996));
    let totals_after: Vec<i64> = w
        .partners
        .iter()
        .map(|p| overview_row(&w, *p).total_paisa.rupees())
        .collect();
    assert_eq!(totals_after, totals_before);
    // The capital history shows the month close as a row of its own, not as money put in.
    let history = partners::capital_history(
        &w.env.ctx,
        &CapitalHistoryInput {
            group_by_month: false,
            sort: None,
            partner_id: w.partners[0],
            offset: 0,
            limit: 0,
        },
    )
    .unwrap();
    assert_eq!(history.rows.len(), 1);
    assert_eq!(history.rows[0].kind, CapitalRowKind::MonthClose);
    assert_eq!(history.balance_paisa.rupees(), 996);
    assert_eq!(
        (
            history.totals.invested_paisa.rupees(),
            history.totals.taken_back_paisa.rupees()
        ),
        (0, 0)
    );
    // The statement has a note line for it, with no amount, and the running total stays.
    let statement = statement_of(&w, w.partners[0]);
    let note = statement.lines.last().unwrap();
    assert_eq!(
        (note.detail, note.amount_paisa, note.running_paisa.rupees()),
        (StatementDetail::MonthClose, None, -1_004)
    );
    // The overview does not count the month close as capital put in.
    assert_eq!(
        overview_row(&w, w.partners[0])
            .month_capital_in_paisa
            .rupees(),
        0
    );
}

/// The family khata of E2E-38, made and used on the opening position.
struct Family {
    khata: i64,
}

fn family_khata(w: &support::World) -> Family {
    Family {
        khata: make_khata(w, "خاندانی کھاتہ", "Family Khata"),
    }
}

fn khata_balance(w: &support::World, khata_id: i64) -> i64 {
    let (side, amount) = balance(w, khata_id);
    match side {
        BalanceSide::Cr => amount,
        BalanceSide::Dr => -amount,
        BalanceSide::Zero => 0,
    }
}

#[test]
fn e2e_38_the_family_khata_pays_out_gets_back_and_sells_its_goods() {
    let w = worked_day();
    let f = family_khata(&w);
    assert!(
        raw(&w.dir)
            .query_row(
                "SELECT c.code FROM accounts a JOIN account_controls c ON c.id = a.control_id
                 WHERE a.id = ?1",
                [f.khata],
                |r| r.get::<_, String>(0)
            )
            .unwrap()
            == "0305"
    );

    // 10-10-26 pays out 10,000 cash: -10,000, cash 40,000.
    let out = vouchers::save_khata_out(
        &w.env.ctx,
        &w.owner,
        &khata_out_input(f.khata, "10,000", "10-10-2026"),
    )
    .unwrap();
    assert_eq!((out.voucher_type, out.number), (VoucherType::KhataOut, 1));
    assert_eq!(khata_balance(&w, f.khata), -10_000);
    assert_eq!(super::day_money::cash_now(&w), 40_000);

    // 11-10-26 gets 4,000 back: -6,000, cash 44,000.
    set_today(&w.env, 2026, 10, 11);
    let back = vouchers::save_khata_in(
        &w.env.ctx,
        &w.owner,
        &khata_in_input(f.khata, "4,000", "11-10-2026"),
    )
    .unwrap();
    assert_eq!((back.voucher_type, back.number), (VoucherType::KhataIn, 1));
    assert_eq!(khata_balance(&w, f.khata), -6_000);
    assert_eq!(super::day_money::cash_now(&w), 44_000);
    // No share row so far: money paid out for a khata is not an expense.
    assert_eq!(scalar(&w, "SELECT COUNT(*) FROM partner_share_rows"), 0);

    // The same day a lot with seller خاندانی کھاتہ sells 10 پیٹی کیلا to ارشد at 900 on credit. The
    // lot has no charges, and the seller-side defaults are 0 for a khata (PRD FR-16.7).
    let mut draft = support::draft(vec![support::with_sales(
        LotInput {
            adjust_advance: false,
            ..support::bare_lot(&w, "khata-1", f.khata, w.banana, "10")
        },
        vec![support::sale("s-1", w.arshad, "10", "900")],
    )]);
    draft.header.entry_date = "11-10-2026".into();
    let preview = support::preview(&w, &draft);
    assert!(preview.errors.is_empty(), "{:?}", preview.errors);
    let saved = save_ok(&w, &draft, false);

    // Value 9,000; buyer comm 540; لاگا 10 × 8 = 80; ارشد pays 9,620; the entry is 9,620 =
    // 9,000 (the khata) + 540 + 80.
    assert_eq!(
        entry_lines(&w, saved.entry_id),
        lines(&[
            ("0021", 9_620, 0),
            ("03050001", 0, 9_000),
            ("04010002", 0, 540),
            ("04010003", 0, 80),
        ])
    );
    // The khata shows +3,000 now.
    assert_eq!(khata_balance(&w, f.khata), 3_000);
    // Shared income 620 = 540 + 80, split 248 / 186 / 186; nothing of the 9,000 goes to anyone.
    assert_eq!(shares_of(&w, saved.entry_id), vec![248, 186, 186]);
    assert_eq!(
        scalar(
            &w,
            "SELECT COUNT(*) FROM partner_share_rows WHERE kind = 'expense'"
        ),
        0
    );
    let totals: Vec<i64> = w
        .partners
        .iter()
        .map(|p| overview_row(&w, *p).total_paisa.rupees())
        .collect();
    assert_eq!(
        totals,
        vec![248, 186, 186],
        "the khata is in nobody's total"
    );
    // The partners' gardens, trips and fruit accounts are empty.
    assert_eq!(
        scalar(
            &w,
            "SELECT COUNT(*) FROM journal_lines l JOIN accounts a ON a.id = l.account_id
             WHERE a.code LIKE '0604%'"
        ),
        0
    );
    let trial = ledger::trial_balance(&w.env.ctx, None).unwrap();
    assert_eq!(trial.total_dr, trial.total_cr);

    // The statement of the khata: three rows with the running balance, one page.
    let statement = khata::statement(
        &w.env.ctx,
        &KhataStatementInput {
            group_by_month: false,
            sort: None,
            khata_account_id: f.khata,
            from: None,
            to: None,
            offset: 0,
            limit: 0,
        },
    )
    .unwrap();
    let rows: Vec<_> = statement
        .rows
        .iter()
        .map(|r| {
            (
                r.kind,
                r.number,
                r.entry_date.as_str(),
                r.amount_paisa.rupees(),
                r.running_paisa.rupees(),
            )
        })
        .collect();
    assert_eq!(
        rows,
        vec![
            (
                KhataRowKind::PaidOut,
                Some(1),
                "2026-10-10",
                -10_000,
                -10_000
            ),
            (KhataRowKind::Received, Some(1), "2026-10-11", 4_000, -6_000),
            (
                KhataRowKind::Sale,
                Some(saved.number),
                "2026-10-11",
                9_000,
                3_000
            ),
        ]
    );
    assert_eq!(
        (
            statement.opening_paisa.rupees(),
            statement.closing_paisa.rupees(),
            statement.khata.balance_paisa.rupees(),
            statement.totals.paid_out_paisa.rupees(),
            statement.totals.received_paisa.rupees()
        ),
        (0, 3_000, 3_000, 10_000, 13_000)
    );
    assert_eq!(rows[0].2, "2026-10-10");
    ledger::verify_journal_chain(&w.env.ctx).unwrap();
    assert_chain_ok(&w.dir);
}

#[test]
fn a_rule_on_the_khata_sets_its_seller_commission_and_the_default_does_not() {
    // PRD FR-16.7, FR-11.3: every seller-side default is 0 for a khata's goods; a rule on the
    // khata itself still applies. The buyer side is charged as usual.
    use mandi_services::core::rules::{ChargeMethod, ChargeSide, ChargeType, RuleInput};
    let w = worked_day();
    let f = family_khata(&w);
    let draft = |w: &support::World, key: &str| {
        let mut d = support::draft(vec![support::with_sales(
            LotInput {
                adjust_advance: false,
                ..support::bare_lot(w, key, f.khata, w.banana, "10")
            },
            vec![support::sale("s-1", w.arshad, "10", "900")],
        )]);
        d.header.entry_date = "10-10-2026".into();
        d
    };
    let first = support::preview(&w, &draft(&w, "k-1"));
    let comm = |p: &mandi_services::core::bikri::BikriPreview| {
        support::calc(&p.lots[0], 0).seller_comm.amount.rupees()
    };
    assert_eq!(comm(&first), 0, "the default 6% is not used for a khata");
    assert_eq!(
        support::calc(&first.lots[0], 0).buyer_comm.amount.rupees(),
        540
    );

    mandi_services::rules::save(
        &w.env.ctx,
        &w.owner,
        &RuleInput {
            id: None,
            version: None,
            side: ChargeSide::Seller,
            charge_type: ChargeType::Commission,
            account_id: Some(f.khata),
            item_id: None,
            item_group: None,
            method: ChargeMethod::Percent,
            value: "5".into(),
            from_date: "01-10-2026".into(),
            to_date: String::new(),
            is_active: true,
            note: String::new(),
        },
    )
    .unwrap();
    let second = support::preview(&w, &draft(&w, "k-2"));
    assert_eq!(comm(&second), 450, "9,000 x 5% on the rule of the khata");
}

#[test]
fn d4_a31_a_drawings_opening_debit_counts_as_money_taken() {
    // PRD FR-10.9, C20, C22: an opening debit of 2,000 on الف's drawings account, nothing else
    // posted for him, gives خالص -2,000 and کل رقم -2,000.
    let w = worked_day();
    let a = w.partners[0];
    let drawings = partners::get(&w.env.ctx, a)
        .unwrap()
        .row
        .drawings_account_id;
    let version = mandi_services::accounts::get(&w.env.ctx, drawings)
        .unwrap()
        .version;
    mandi_services::accounts::openings_save(
        &w.env.ctx,
        &w.owner,
        &OpeningsInput {
            lines: vec![OpeningLineInput {
                account_id: drawings,
                version,
                opening_date: "10-10-2026".into(),
                opening_dr: "2,000".into(),
                opening_cr: String::new(),
                opening_is_advance: false,
            }],
        },
    )
    .unwrap();
    let row = overview_row(&w, a);
    assert_eq!(
        (
            row.month_taken_paisa.rupees(),
            row.month_returned_paisa.rupees(),
            row.month_profit_paisa.rupees(),
            row.month_net_paisa.rupees(),
            row.total_paisa.rupees()
        ),
        (2_000, 0, 0, -2_000, -2_000)
    );
    // The take list shows it as an opening, open 2,000.
    let takes = partners::takes(
        &w.env.ctx,
        &PartnerTakesInput {
            group_by_month: false,
            sort: None,
            partner_id: a,
            offset: 0,
            limit: 0,
        },
    )
    .unwrap();
    assert_eq!(takes.rows.len(), 1);
    assert_eq!(
        (
            takes.rows[0].source,
            takes.rows[0].voucher_id,
            takes.rows[0].open_paisa.rupees()
        ),
        (TakeSource::Opening, None, 2_000)
    );
    // His statement has the line, dated its opening date.
    let statement = statement_of(&w, a);
    assert_eq!(statement.lines.len(), 1);
    let line = &statement.lines[0];
    assert_eq!(
        (
            line.heading,
            line.detail,
            line.entry_date.as_str(),
            line.amount_paisa.map(Paisa::rupees),
            line.running_paisa.rupees()
        ),
        (
            StatementHeading::Drawings,
            StatementDetail::DrawingsOpening,
            "2026-10-10",
            Some(-2_000),
            -2_000
        )
    );
    // The other partners are not affected.
    assert_eq!(overview_row(&w, w.partners[1]).total_paisa.rupees(), 0);
}
