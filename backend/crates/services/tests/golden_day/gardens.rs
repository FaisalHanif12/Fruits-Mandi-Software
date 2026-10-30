//! Plan 17c, gardens (PRD 12.11, FR-12.34 to FR-12.38, C23, C26): the worked checks E2E-40 and
//! E2E-40b and the refusals. The expected numbers are the PRD's (`worked_day.py`) and the money
//! agent's; they are never changed to suit the code.
//!
//! The garden شاہ پور باغ has rent 50,000. الف has 60% and pays from the shop's cash, ب has 40% and
//! pays from his own pocket, ج is not in it. Installments 30,000 (10-10) and 20,000 (11-10), labour
//! 5,000 (11-10): the parts are 18,000 / 12,000, 12,000 / 8,000 and 3,000 / 2,000, so الف paid
//! 33,000 from the shop and ب 22,000 himself.

use mandi_services::core::gardens::{
    GardenCloseInput, GardenDetail, GardenEntryCancelInput, GardenEntryInput, GardenGetInput,
    GardenHistoryKind, GardenInput, GardenOtherSaleInput, GardenPaymentInput, GardenReopenInput,
    GardenRow, GardenSetPartnersInput,
};
use mandi_services::core::partners::PartnerVentureCard;
use mandi_services::core::statement::{PartnerStatement, PartnersStatementInput, StatementDetail};
use mandi_services::core::ventures::{EntryPartInput, MoneySource, VentureKind, VentureStatus};
use mandi_services::{ServiceError, gardens, partners};

use super::own_goods::{
    assert_equity_is_the_partner_totals, cash_balance, credit_balance, partner_total, pocket, shop,
    totals_of_three, trial,
};
use super::*;

/// The garden of the worked check: الف 60% from the shop, ب 40% from his pocket, ج not in it.
fn garden_input(w: &support::World, paid_now: Option<&str>) -> GardenInput {
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
        rent: "50,000".into(),
        season_end: None,
        note: None,
        partners: vec![shop(a, "60"), pocket(b, "40")],
        paid_now: paid_now.map(str::to_owned),
        paid_now_reason: paid_now.map(|_| "first installment".to_owned()),
        parts: Vec::new(),
        confirm_cash_below_zero: false,
    }
}

fn create(w: &support::World, paid_now: Option<&str>) -> GardenRow {
    gardens::create(&w.env.ctx, &w.owner, &garden_input(w, paid_now))
        .unwrap()
        .row
}

fn payment(garden: i64, date: &str, amount: &str, reason: &str) -> GardenPaymentInput {
    GardenPaymentInput {
        garden_id: Some(garden),
        date: date.into(),
        amount: amount.into(),
        reason: reason.into(),
        parts: Vec::new(),
        confirm_cash_below_zero: false,
    }
}

fn installment(w: &support::World, garden: i64, date: &str, amount: &str) {
    gardens::entry_save(
        &w.env.ctx,
        &w.owner,
        &GardenEntryInput::Installment(payment(garden, date, amount, "installment")),
    )
    .unwrap();
}

fn cost(w: &support::World, garden: i64, date: &str, amount: &str) {
    gardens::entry_save(
        &w.env.ctx,
        &w.owner,
        &GardenEntryInput::Cost(payment(garden, date, amount, "labour")),
    )
    .unwrap();
}

fn other_sale(garden: i64, date: &str, qty: &str, amount: &str) -> GardenEntryInput {
    GardenEntryInput::OtherSale(GardenOtherSaleInput {
        arrival_id: None,
        garden_id: Some(garden),
        date: date.into(),
        shop: Some("Haji Traders".into()),
        qty: qty.into(),
        amount: amount.into(),
        note: None,
    })
}

fn detail(w: &support::World, garden: i64) -> GardenDetail {
    gardens::get(
        &w.env.ctx,
        &GardenGetInput {
            id: garden,
            history_offset: 0,
            history_limit: 0,
        },
    )
    .unwrap()
}

fn close_input(w: &support::World, garden: i64, date: &str) -> GardenCloseInput {
    GardenCloseInput {
        id: garden,
        version: detail(w, garden).row.version,
        date: date.into(),
    }
}

/// The garden's three payments of the worked check: 30,000 (10-10, with the garden), 20,000 and the
/// labour 5,000 (11-10).
fn pay_the_garden(w: &support::World) -> GardenRow {
    let garden = create(w, Some("30,000"));
    set_today(&w.env, 2026, 10, 11);
    installment(w, garden.id, "11-10-2026", "20,000");
    cost(w, garden.id, "11-10-2026", "5,000");
    garden
}

#[test]
fn e2e_40_the_parts_of_the_payments_and_what_each_partner_paid() {
    let w = worked_day();
    let garden = pay_the_garden(&w);
    // Parts 18,000 / 12,000, 12,000 / 8,000 and 3,000 / 2,000 (C23).
    let d = detail(&w, garden.id);
    let parts: Vec<(i64, i64)> = d
        .history
        .iter()
        .filter(|h| !h.parts.is_empty())
        .map(|h| {
            (
                h.parts[0].amount_paisa.rupees(),
                h.parts[1].amount_paisa.rupees(),
            )
        })
        .collect();
    assert_eq!(parts, [(18_000, 12_000), (12_000, 8_000), (3_000, 2_000)]);
    // الف paid 33,000 from the shop's cash, ب 22,000 from his pocket.
    assert_eq!(cash_balance(&w), 17_000);
    let [a, b] = [&d.partner_results[0], &d.partner_results[1]];
    assert_eq!(
        (
            a.paid_shop_paisa.rupees(),
            a.paid_pocket_paisa.rupees(),
            b.paid_shop_paisa.rupees(),
            b.paid_pocket_paisa.rupees()
        ),
        (33_000, 0, 0, 22_000)
    );
    // The rent still due: 50,000 less the two installments; the labour is not rent.
    assert_eq!(d.row.rent_due_paisa.rupees(), 0);
    assert_eq!(d.row.paid_paisa.rupees(), 55_000);
    // Running balance before any sale: 0 - 55,000, and -30,000 after the first installment.
    assert_eq!(d.row.balance_paisa.rupees(), -55_000);
    let balances: Vec<i64> = d
        .history
        .iter()
        .map(|h| h.running_balance_paisa.rupees())
        .collect();
    assert_eq!(balances, [-30_000, -50_000, -55_000]);
    // الف's total came down by his shop parts (E2E-40: -33,000), ب's did not move.
    assert_eq!(partner_total(&w, 0), -33_000);
    assert_eq!(partner_total(&w, 1), 0);
    assert_eq!(
        trial(&w),
        65_000,
        "only 33,000 moved, between cash and 0604"
    );
}

#[test]
fn e2e_40b_only_the_other_shops_10000_so_a_loss_of_45000() {
    let w = worked_day();
    let garden = pay_the_garden(&w);
    set_today(&w.env, 2026, 10, 12);
    let sold = gardens::entry_save(
        &w.env.ctx,
        &w.owner,
        &other_sale(garden.id, "12-10-2026", "20", "10,000"),
    )
    .unwrap();
    assert!(sold.journal_entry_id.is_some());
    assert_eq!(
        cash_balance(&w),
        27_000,
        "17,000 + the 10,000 from the other shop"
    );
    let before = detail(&w, garden.id);
    // Money in 10,000, paid 55,000: the running balance is -45,000, and the account holds 10,000.
    assert_eq!(before.row.money_in_paisa.rupees(), 10_000);
    assert_eq!(before.row.other_shop_paisa.rupees(), 10_000);
    assert_eq!(before.row.balance_paisa.rupees(), -45_000);
    assert_eq!(before.row.account_balance_paisa.rupees(), 10_000);

    set_today(&w.env, 2026, 10, 13);
    let preview =
        gardens::close_preview(&w.env.ctx, &close_input(&w, garden.id, "13-10-2026")).unwrap();
    assert!(preview.errors.is_empty(), "{:?}", preview.errors);
    let numbers = preview.numbers.unwrap();
    assert_eq!(
        (
            numbers.balance_paisa.rupees(),
            numbers.total_cost_paisa.rupees(),
            numbers.money_in_paisa.rupees(),
            numbers.result_paisa.rupees()
        ),
        (10_000, 55_000, 10_000, -45_000)
    );
    let shown: Vec<i64> = numbers
        .credits
        .iter()
        .map(|c| c.amount_paisa.rupees())
        .collect();
    assert_eq!(shown, [6_000, 4_000]);
    // The results of the partners (derived): الف 6,000 - 33,000, ب 4,000 - 22,000; they add up to the
    // result -45,000 exactly.
    let nets: Vec<i64> = numbers
        .partner_results
        .iter()
        .map(|r| r.net_paisa.rupees())
        .collect();
    assert_eq!(nets, [-27_000, -18_000]);
    assert_eq!(nets.iter().sum::<i64>(), -45_000);

    let closed = gardens::close(
        &w.env.ctx,
        &w.owner,
        &close_input(&w, garden.id, "13-10-2026"),
    )
    .unwrap();
    assert_eq!(closed.row.status, VentureStatus::Closed);
    assert_eq!(
        closed.row.account_balance_paisa.rupees(),
        0,
        "the garden's account is 0"
    );
    // The closing is not money in: the running balance still reads the result.
    assert_eq!(closed.row.balance_paisa.rupees(), -45_000);
    assert_eq!(closed.row.money_in_paisa.rupees(), 10_000);
    // E2E-40b: الف -27,000, ب +4,000, ج 0; cash 27,000.
    assert_eq!(totals_of_three(&w), [-27_000, 4_000, 0]);
    assert_eq!(cash_balance(&w), 27_000);
    assert_equity_is_the_partner_totals(&w, 42_000);
    // Debits: cash 27,000 + رحیم's advance 15,000 + الف's 0604 27,000 (33,000 paid less 6,000 back) =
    // 69,000. Credits: the opening capital 65,000 + ب's 0604 4,000 = 69,000.
    assert_eq!(trial(&w), 69_000);
    // The results of the partners stay what the preview said.
    let d = detail(&w, garden.id);
    let nets: Vec<i64> = d
        .partner_results
        .iter()
        .map(|r| r.net_paisa.rupees())
        .collect();
    assert_eq!(nets, [-27_000, -18_000]);
    let last = d.history.last().unwrap();
    assert_eq!(last.kind, GardenHistoryKind::Close);
    assert_eq!(last.running_balance_paisa.rupees(), -45_000);
}

/// E2E-40's sale in our shop: 100 پیٹی کیلا of the garden sold to ارشد at 600 on credit, no lot
/// charges, dated 12-10-26.
fn sell_a_garden_lot(w: &support::World, garden: &GardenRow) -> BikriSaved {
    let mut lot = support::with_sales(
        support::bare_lot(w, "garden-1", garden.account.id, w.banana, "100"),
        vec![support::sale("s-1", w.arshad, "100", "600")],
    );
    lot.adjust_advance = false;
    let mut draft = support::draft(vec![lot]);
    draft.header.entry_date = "12-10-2026".into();
    save_ok(w, &draft, false)
}

#[test]
fn e2e_40_the_whole_check_sale_other_shop_closing_and_the_partner_totals() {
    let w = worked_day();
    let garden = pay_the_garden(&w);
    set_today(&w.env, 2026, 10, 12);
    assert_eq!(cash_balance(&w), 17_000);

    // The sale in our shop: 100 × 600 = 60,000; seller comm 3,600; buyer comm 3,600; لاگا 800;
    // ارشد pays 64,400; the garden's account جمع 56,400; commission 8,000 split by the firm's shares.
    let saved = sell_a_garden_lot(&w, &garden);
    let t = saved.preview.lots[0].totals.as_ref().unwrap();
    assert_eq!(
        (
            support::r(t.gross),
            support::r(t.seller_comm),
            support::r(t.buyer_comm),
            support::r(t.laga),
            support::r(t.buyer_total),
            support::r(t.seller_credit),
            support::r(t.shared_income)
        ),
        (60_000, 3_600, 3_600, 800, 64_400, 56_400, 8_000)
    );
    assert_eq!(balance(&w, w.arshad), (BalanceSide::Dr, 64_400));
    assert_eq!(
        balance(&w, garden.account.id),
        (BalanceSide::Cr, 56_400),
        "the seller credit goes to the garden's account"
    );
    assert_eq!(cash_balance(&w), 17_000, "a garden lot gives no نقد");
    let shares: Vec<i64> = (0..3)
        .map(|i| super::own_goods::partner_total(&w, i))
        .collect();
    assert_eq!(shares, [3_200 - 33_000, 2_400, 2_400]);
    assert_eq!(
        scalar(
            &w,
            &format!(
                "SELECT venture_id FROM analytics_facts WHERE bikri_id = {} LIMIT 1",
                saved.bikri_id
            )
        ),
        garden.id,
        "the fact carries the garden for the analytics filters"
    );

    // 20 پیٹی sold in another shop: the money comes into روکڑ and the garden's account.
    gardens::entry_save(
        &w.env.ctx,
        &w.owner,
        &other_sale(garden.id, "12-10-2026", "20", "10,000"),
    )
    .unwrap();
    assert_eq!(cash_balance(&w), 27_000);
    let d = detail(&w, garden.id);
    assert_eq!(d.row.money_in_paisa.rupees(), 66_400);
    assert_eq!(d.row.paid_paisa.rupees(), 55_000);
    assert_eq!(d.row.balance_paisa.rupees(), 11_400, "66,400 - 55,000");
    assert_eq!(d.row.account_balance_paisa.rupees(), 66_400);
    // The history runs by date with the balance after each row.
    let rows: Vec<(GardenHistoryKind, i64)> = d
        .history
        .iter()
        .map(|h| (h.kind, h.running_balance_paisa.rupees()))
        .collect();
    assert_eq!(
        rows,
        [
            (GardenHistoryKind::Installment, -30_000),
            (GardenHistoryKind::Installment, -50_000),
            (GardenHistoryKind::Cost, -55_000),
            (GardenHistoryKind::Sale, 1_400),
            (GardenHistoryKind::OtherSale, 11_400),
        ]
    );
    assert_eq!(d.history[3].bikri_number, Some(saved.number));

    // The closing: 66,400 to 39,840 / 26,560; the garden's account is 0.
    set_today(&w.env, 2026, 10, 13);
    let closed = gardens::close(
        &w.env.ctx,
        &w.owner,
        &close_input(&w, garden.id, "13-10-2026"),
    )
    .unwrap();
    let shown: Vec<i64> = closed
        .preview
        .numbers
        .as_ref()
        .unwrap()
        .credits
        .iter()
        .map(|c| c.amount_paisa.rupees())
        .collect();
    assert_eq!(shown, [39_840, 26_560]);
    assert_eq!(balance(&w, garden.account.id), (BalanceSide::Zero, 0));
    assert_eq!(
        closed.row.balance_paisa.rupees(),
        11_400,
        "the result stays"
    );
    // الف: -33,000 + 3,200 + 39,840 = 10,040; ب: 2,400 + 26,560 = 28,960; ج: 2,400.
    assert_eq!(totals_of_three(&w), [10_040, 28_960, 2_400]);
    // Their parts of the result: 39,840 - 33,000 = 6,840 and 26,560 - 22,000 = 4,560, which add up to
    // the result 11,400.
    let nets: Vec<i64> = detail(&w, garden.id)
        .partner_results
        .iter()
        .map(|r| r.net_paisa.rupees())
        .collect();
    assert_eq!(nets, [6_840, 4_560]);
    assert_eq!(nets.iter().sum::<i64>(), 11_400);
    // Σ 41,400 + the opening capital 65,000 = 106,400 = the equity in the books = the trial balance.
    assert_equity_is_the_partner_totals(&w, 106_400);
    assert_eq!(trial(&w), 106_400);
    assert_eq!(cash_balance(&w), 27_000);
}

fn statement_of(w: &support::World, partner: i64) -> PartnerStatement {
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

fn cards_of(w: &support::World, partner: i64) -> Vec<PartnerVentureCard> {
    partners::overview(&w.env.ctx)
        .unwrap()
        .into_iter()
        .find(|r| r.partner_id == partner)
        .unwrap()
        .open_ventures
}

/// E2E-40 up to the day before the closing: payments, the sale in our shop, the other shop.
fn e2e_40_before_the_closing(w: &support::World) -> GardenRow {
    let garden = pay_the_garden(w);
    set_today(&w.env, 2026, 10, 12);
    sell_a_garden_lot(w, &garden);
    gardens::entry_save(
        &w.env.ctx,
        &w.owner,
        &other_sale(garden.id, "12-10-2026", "20", "10,000"),
    )
    .unwrap();
    garden
}

#[test]
fn e2e_40_the_cards_under_the_partners_while_the_garden_is_open() {
    let w = worked_day();
    let garden = e2e_40_before_the_closing(&w);
    let [a, b, c] = w.partners;
    // الف: 60%, paid 33,000 from the shop; ب: 40%, paid 22,000 from his pocket; ج is not in it.
    let card_a = cards_of(&w, a);
    assert_eq!(card_a.len(), 1);
    let card = &card_a[0];
    assert_eq!(
        (card.kind, card.venture_id, card.number),
        (VentureKind::Garden, garden.id, 1)
    );
    assert_eq!(
        (
            card.share_bps,
            card.paid_shop_paisa.rupees(),
            card.paid_pocket_paisa.rupees(),
            card.money_back_paisa.rupees(),
            card.balance_paisa.rupees()
        ),
        (6_000, 33_000, 0, 0, 11_400),
        "a garden pays its partners only when it is closed"
    );
    let card_b = &cards_of(&w, b)[0];
    assert_eq!(
        (
            card_b.share_bps,
            card_b.paid_shop_paisa.rupees(),
            card_b.paid_pocket_paisa.rupees()
        ),
        (4_000, 0, 22_000)
    );
    assert!(cards_of(&w, c).is_empty());
    // Once the garden is closed it is no longer open: the cards go.
    set_today(&w.env, 2026, 10, 13);
    gardens::close(
        &w.env.ctx,
        &w.owner,
        &close_input(&w, garden.id, "13-10-2026"),
    )
    .unwrap();
    assert!(cards_of(&w, a).is_empty() && cards_of(&w, b).is_empty());
}

#[test]
fn e2e_40_the_statement_of_alif_runs_18000_12000_3000_then_3200_then_39840_to_10040() {
    let w = worked_day();
    let garden = e2e_40_before_the_closing(&w);
    set_today(&w.env, 2026, 10, 13);
    gardens::close(
        &w.env.ctx,
        &w.owner,
        &close_input(&w, garden.id, "13-10-2026"),
    )
    .unwrap();
    let alif = statement_of(&w, w.partners[0]);
    let amounts: Vec<i64> = alif
        .lines
        .iter()
        .filter_map(|l| l.amount_paisa.map(|a| a.rupees()))
        .collect();
    assert_eq!(amounts, [-18_000, -12_000, -3_000, 3_200, 39_840]);
    assert_eq!(alif.closing_total_paisa.rupees(), 10_040);
    let running: Vec<i64> = alif
        .lines
        .iter()
        .map(|l| l.running_paisa.rupees())
        .collect();
    assert_eq!(running, [-18_000, -30_000, -33_000, -29_800, 10_040]);
    // The closing line carries the numbers of FR-17.10, all from stored rows.
    let closing = alif
        .lines
        .iter()
        .find(|l| l.detail == StatementDetail::GardenClosed)
        .unwrap();
    let value = |name: &str| match closing.values.get(name) {
        Some(mandi_services::core::error::ErrorValue::Number(n)) => *n / 100,
        other => panic!("{name}: {other:?}"),
    };
    assert_eq!(
        (
            value("cost_paisa"),
            value("sales_paisa"),
            value("result_paisa"),
            value("part_paisa"),
            value("back_paisa")
        ),
        (55_000, 66_400, 11_400, 6_840, 39_840)
    );
    assert_eq!(
        closing.values.get("share_bps"),
        Some(&mandi_services::core::error::ErrorValue::Number(6_000))
    );
    // ب: his closing line shows his part 26,560 - 22,000 = 4,560.
    let bob = statement_of(&w, w.partners[1]);
    let closing = bob
        .lines
        .iter()
        .find(|l| l.detail == StatementDetail::GardenClosed)
        .unwrap();
    assert_eq!(closing.amount_paisa.unwrap().rupees(), 26_560);
}

// ---- Refusals ----

fn error_codes(error: ServiceError) -> Vec<(ErrorCode, Option<String>)> {
    match error {
        ServiceError::Rejected(items) => items.into_iter().map(|i| (i.code, i.field)).collect(),
        other => panic!("not a refusal of the user's entry: {other:?}"),
    }
}

fn code_at(error: ServiceError, code: ErrorCode, field: &str) {
    let found = error_codes(error);
    assert!(
        found
            .iter()
            .any(|(c, f)| *c == code && f.as_deref() == Some(field)),
        "{code:?} at {field} not in {found:?}"
    );
}

#[test]
fn the_shares_must_total_100_and_are_locked_after_the_first_entry() {
    let w = worked_day();
    let [a, b, _] = w.partners;
    let mut bad = garden_input(&w, None);
    bad.partners = vec![shop(a, "60"), pocket(b, "30")];
    code_at(
        gardens::create(&w.env.ctx, &w.owner, &bad).unwrap_err(),
        ErrorCode::ErrSharesNot100,
        "partners",
    );
    // Nothing was made: no garden, no account.
    assert_eq!(
        scalar(&w, "SELECT COUNT(*) FROM ventures WHERE kind = 'garden'"),
        0
    );
    assert_eq!(
        scalar(&w, "SELECT COUNT(*) FROM accounts WHERE code LIKE '0306%'"),
        0
    );
    // A garden with no entry takes new shares; after its first entry they are fixed (E66).
    let garden = create(&w, None);
    let [_, _, c] = w.partners;
    let changed = gardens::set_partners(
        &w.env.ctx,
        &w.owner,
        &GardenSetPartnersInput {
            id: garden.id,
            version: garden.version,
            partners: vec![shop(a, "50"), shop(b, "30"), shop(c, "20")],
        },
    )
    .unwrap();
    assert_eq!(changed.partners.len(), 3);
    installment(&w, garden.id, "10-10-2026", "10,000");
    let now = detail(&w, garden.id).row;
    assert!(now.shares_locked);
    code_at(
        gardens::set_partners(
            &w.env.ctx,
            &w.owner,
            &GardenSetPartnersInput {
                id: garden.id,
                version: now.version,
                partners: vec![shop(a, "60"), pocket(b, "40")],
            },
        )
        .unwrap_err(),
        ErrorCode::ErrVentureSharesLocked,
        "partners",
    );
}

#[test]
fn a_closed_garden_takes_no_entry_no_cancel_no_lot_and_no_cancel_of_its_bikri() {
    let w = worked_day();
    let garden = e2e_40_before_the_closing(&w);
    let sold_bikri = scalar(&w, "SELECT MAX(id) FROM bikri_entries");
    let other_sale_id = detail(&w, garden.id)
        .history
        .iter()
        .find(|h| h.kind == GardenHistoryKind::OtherSale)
        .and_then(|h| h.entry_id)
        .unwrap();
    set_today(&w.env, 2026, 10, 13);
    gardens::close(
        &w.env.ctx,
        &w.owner,
        &close_input(&w, garden.id, "13-10-2026"),
    )
    .unwrap();

    // E65: an installment, a cost, a sale in another shop and the closing again are refused.
    for entry in [
        GardenEntryInput::Installment(payment(garden.id, "13-10-2026", "100", "x")),
        GardenEntryInput::Cost(payment(garden.id, "13-10-2026", "100", "x")),
        other_sale(garden.id, "13-10-2026", "1", "100"),
    ] {
        code_at(
            gardens::entry_save(&w.env.ctx, &w.owner, &entry).unwrap_err(),
            ErrorCode::ErrGardenClosed,
            "garden_id",
        );
    }
    code_at(
        gardens::close(
            &w.env.ctx,
            &w.owner,
            &close_input(&w, garden.id, "13-10-2026"),
        )
        .unwrap_err(),
        ErrorCode::ErrGardenClosed,
        "id",
    );
    // The cancel of one of its entries is refused until the closing is undone.
    let version = detail(&w, garden.id)
        .history
        .iter()
        .find(|h| h.entry_id == Some(other_sale_id))
        .and_then(|h| h.version)
        .unwrap();
    code_at(
        gardens::entry_cancel(
            &w.env.ctx,
            &w.owner,
            &GardenEntryCancelInput {
                id: other_sale_id,
                version,
                reason: "x".into(),
                confirm_cash_below_zero: false,
            },
        )
        .unwrap_err(),
        ErrorCode::ErrGardenClosed,
        "id",
    );
    // A new lot of the garden is refused, and so is the cancel of the بکری that sold its goods.
    let mut lot = support::with_sales(
        support::bare_lot(&w, "g-2", garden.account.id, w.banana, "10"),
        vec![support::sale("s-1", w.akram, "10", "600")],
    );
    lot.adjust_advance = false;
    let mut draft = support::draft(vec![lot]);
    draft.header.entry_date = "13-10-2026".into();
    let refused = mandi_services::bikri::preview(&w.env.ctx, &draft).unwrap();
    assert!(
        refused
            .errors
            .iter()
            .any(|e| e.code == ErrorCode::ErrGardenClosed && e.field == "lots[0].seller_account_id"),
        "{:?}",
        refused.errors
    );
    let detail_of_bikri = mandi_services::bikri::get(&w.env.ctx, sold_bikri).unwrap();
    code_at(
        mandi_services::bikri::cancel(
            &w.env.ctx,
            &w.owner,
            &mandi_services::core::bikri::CancelInput {
                id: sold_bikri,
                version: detail_of_bikri.version,
                reason: "x".into(),
                confirm_cash_below_zero: false,
            },
        )
        .unwrap_err(),
        ErrorCode::ErrGardenClosed,
        "id",
    );
    // The closed garden's account is offered in no picker (it is hidden by its venture status).
    let hits = mandi_services::search::search_accounts(
        &w.env.ctx,
        &w.owner,
        &mandi_services::core::accounts::SearchAccountsInput {
            include_dead: None,
            scope: mandi_services::core::accounts::AccountScope::LotSeller,
            control_id: None,
            text: "shahpur".into(),
            limit: 10,
            offset: 0,
        },
    )
    .unwrap();
    assert!(hits.is_empty(), "{hits:?}");

    // Undo the closing: the garden takes entries again, its account holds its money again, and the
    // picker offers it.
    let current = detail(&w, garden.id).row;
    let reopened = gardens::reopen(
        &w.env.ctx,
        &w.owner,
        &GardenReopenInput {
            id: garden.id,
            version: current.version,
            reason: "wrong day".into(),
        },
    )
    .unwrap();
    assert_eq!(reopened.row.status, VentureStatus::Open);
    assert_eq!(reopened.row.account_balance_paisa.rupees(), 66_400);
    assert_eq!(
        totals_of_three(&w),
        [-29_800, 2_400, 2_400],
        "the closing is reversed"
    );
    let hits = mandi_services::search::search_accounts(
        &w.env.ctx,
        &w.owner,
        &mandi_services::core::accounts::SearchAccountsInput {
            include_dead: None,
            scope: mandi_services::core::accounts::AccountScope::LotSeller,
            control_id: None,
            text: "shahpur".into(),
            limit: 10,
            offset: 0,
        },
    )
    .unwrap();
    assert_eq!(hits.len(), 1);
    installment(&w, garden.id, "13-10-2026", "100");
    // Closing again gives a new closing with a new number, and works out its balance again.
    let again = gardens::close(
        &w.env.ctx,
        &w.owner,
        &close_input(&w, garden.id, "13-10-2026"),
    )
    .unwrap();
    assert_eq!(again.row.status, VentureStatus::Closed);
    assert_eq!(again.row.account_balance_paisa.rupees(), 0);
    // An open garden has no closing to undo.
    let open = create(&w, None);
    code_at(
        gardens::reopen(
            &w.env.ctx,
            &w.owner,
            &GardenReopenInput {
                id: open.id,
                version: open.version,
                reason: "x".into(),
            },
        )
        .unwrap_err(),
        ErrorCode::ErrGardenNotClosed,
        "id",
    );
}

#[test]
fn a_closed_day_a_day_before_the_garden_or_before_its_last_entry_is_refused() {
    let w = worked_day();
    let garden = e2e_40_before_the_closing(&w);
    // An entry dated before the garden (10-10-26 is its day; the books are closed up to 09-10).
    let early = GardenEntryInput::Installment(payment(garden.id, "09-10-2026", "100", "x"));
    let found = error_codes(gardens::entry_save(&w.env.ctx, &w.owner, &early).unwrap_err());
    assert!(
        found
            .iter()
            .any(|(c, f)| *c == ErrorCode::ErrDateClosed && f.as_deref() == Some("date")),
        "{found:?}"
    );
    // The closing cannot be dated before its latest entry (12-10-26).
    set_today(&w.env, 2026, 10, 13);
    code_at(
        gardens::close(
            &w.env.ctx,
            &w.owner,
            &close_input(&w, garden.id, "11-10-2026"),
        )
        .unwrap_err(),
        ErrorCode::ErrCloseBeforeLastEntry,
        "date",
    );
    let preview =
        gardens::close_preview(&w.env.ctx, &close_input(&w, garden.id, "11-10-2026")).unwrap();
    assert_eq!(preview.earliest_date.as_deref(), Some("2026-10-12"));
    assert!(preview.numbers.is_none());
}

#[test]
fn a_payment_from_pockets_only_posts_nothing_and_a_shop_part_asks_when_cash_would_go_below_zero() {
    let w = worked_day();
    let [a, b, _] = w.partners;
    let garden = create(&w, None);
    // E63: every part from a pocket posts no journal entry; cash does not move.
    let all_pocket: Vec<EntryPartInput> = [a, b]
        .iter()
        .map(|id| EntryPartInput {
            partner_id: *id,
            source: MoneySource::Pocket,
        })
        .collect();
    let mut pocket_only = payment(garden.id, "10-10-2026", "10,000", "spray");
    pocket_only.parts = all_pocket;
    let saved =
        gardens::entry_save(&w.env.ctx, &w.owner, &GardenEntryInput::Cost(pocket_only)).unwrap();
    assert_eq!(saved.journal_entry_id, None);
    assert_eq!(cash_balance(&w), 50_000);
    assert_eq!(detail(&w, garden.id).row.paid_pocket_paisa.rupees(), 10_000);
    // FR-6.7: 60% of 90,000 is 54,000 from the shop's cash of 50,000: it asks.
    let big = payment(garden.id, "10-10-2026", "90,000", "installment");
    let preview =
        gardens::entry_preview(&w.env.ctx, &GardenEntryInput::Installment(big.clone())).unwrap();
    assert!(preview.cash.needs_confirm);
    assert_eq!(preview.cash.cash_after_paisa.rupees(), -4_000);
    let refused = gardens::entry_save(
        &w.env.ctx,
        &w.owner,
        &GardenEntryInput::Installment(big.clone()),
    )
    .unwrap_err();
    assert_eq!(refused.code(), ErrorCode::ErrCashBelowZeroNotConfirmed);
    let mut confirmed = big;
    confirmed.confirm_cash_below_zero = true;
    gardens::entry_save(
        &w.env.ctx,
        &w.owner,
        &GardenEntryInput::Installment(confirmed),
    )
    .unwrap();
    // The helper gives the size of the balance; the side is credit, so the cash is below zero.
    assert_eq!(cash_balance(&w), 4_000);
    assert_eq!(
        mandi_services::ledger::account_balance(&w.env.ctx, support::id_of(&w.env, "01010001"))
            .unwrap()
            .side,
        BalanceSide::Cr
    );
}

#[test]
fn a_garden_with_no_sales_closes_at_zero_and_the_whole_cost_is_the_loss() {
    // E64: the account is 0, nothing is credited, and each shop-cash payer keeps the deduction.
    let w = worked_day();
    let garden = pay_the_garden(&w);
    set_today(&w.env, 2026, 10, 13);
    let closed = gardens::close(
        &w.env.ctx,
        &w.owner,
        &close_input(&w, garden.id, "13-10-2026"),
    )
    .unwrap();
    assert_eq!(
        closed.journal_entry_id, None,
        "nothing to move, nothing posted"
    );
    assert!(!closed.preview.numbers.unwrap().posts_entry);
    assert_eq!(closed.row.status, VentureStatus::Closed);
    assert_eq!(totals_of_three(&w), [-33_000, 0, 0]);
    let nets: Vec<i64> = detail(&w, garden.id)
        .partner_results
        .iter()
        .map(|r| r.net_paisa.rupees())
        .collect();
    assert_eq!(
        nets,
        [-33_000, -22_000],
        "the whole cost of 55,000 is the loss"
    );
}

#[test]
fn a_garden_account_takes_no_opening_no_jaanral_line_and_keeps_its_partners_tied_while_open() {
    use mandi_services::core::general::{GeneralInput, GeneralLineInput};
    let w = worked_day();
    let garden = create(&w, None);
    let [a, b, c] = w.partners;
    // A63: a partner of an open garden is still tied to it (the update check reads this same
    // fact); ج is not in it, and the tie goes when the garden closes.
    let tied = |id: i64| {
        scalar(
            &w,
            &format!(
                "SELECT EXISTS (SELECT 1 FROM venture_partners vp JOIN ventures v ON v.id = vp.venture_id \
                 WHERE vp.partner_id = {id} AND v.status = 'open')"
            ),
        )
    };
    assert_eq!((tied(a), tied(b), tied(c)), (1, 1, 0));
    set_today(&w.env, 2026, 10, 13);
    gardens::close(
        &w.env.ctx,
        &w.owner,
        &close_input(&w, garden.id, "13-10-2026"),
    )
    .unwrap();
    assert_eq!((tied(a), tied(b)), (0, 0));
    // FR-10.9: no opening on a garden's account.
    let account = mandi_services::accounts::get(&w.env.ctx, garden.account.id).unwrap();
    assert!(!account.can_take_opening);
    // A جنرل line on a garden's account is refused: it moves only by its own entries and closing.
    let line = |account: i64, dr: &str, cr: &str| GeneralLineInput {
        account_id: Some(account),
        person_key: None,
        dr: dr.into(),
        cr: cr.into(),
        narration: None,
    };
    let refused = vouchers::general_save(
        &w.env.ctx,
        &w.owner,
        &GeneralInput {
            date: "13-10-2026".into(),
            narration: None,
            lines: vec![
                line(support::id_of(&w.env, "01010001"), "100", ""),
                line(garden.account.id, "", "100"),
            ],
            confirm_cash_below_zero: false,
        },
    )
    .unwrap_err();
    code_at(refused, ErrorCode::ErrFormatInvalid, "lines[1].account_id");
}

#[test]
fn an_edit_of_a_bikri_of_a_closed_garden_is_refused_even_with_the_seller_changed() {
    // PRD FR-12.38: the posting is not edited until the closing is undone. Changing the lot's seller
    // would take the 56,400 back from the garden's account after it went to the partners.
    use mandi_services::core::bikri::{EditOpenInput, EditPreviewInput, EditSaveInput};
    let w = worked_day();
    let garden = e2e_40_before_the_closing(&w);
    let sold = scalar(&w, "SELECT MAX(id) FROM bikri_entries");
    let entry = scalar(
        &w,
        "SELECT id FROM journal_entries WHERE posting_type = 'bikri_first' ORDER BY id DESC LIMIT 1",
    );
    let open = |bikri_id: i64, entry_id: i64| {
        bikri::open_for_edit(&w.env.ctx, &EditOpenInput { bikri_id, entry_id })
    };
    // While the garden is open the edit opens.
    let opened = open(sold, entry).unwrap();
    set_today(&w.env, 2026, 10, 13);
    gardens::close(
        &w.env.ctx,
        &w.owner,
        &close_input(&w, garden.id, "13-10-2026"),
    )
    .unwrap();
    let before = totals_of_three(&w);

    let mut changed = opened.draft.clone();
    changed.lots[0].seller_account_id = Some(w.rahim);
    let error = open(sold, entry).unwrap_err();
    code_at(error, ErrorCode::ErrGardenClosed, "entry_id");
    let error = bikri::edit_preview(
        &w.env.ctx,
        &EditPreviewInput {
            entry_id: entry,
            draft: changed.clone(),
        },
    )
    .unwrap_err();
    code_at(error, ErrorCode::ErrGardenClosed, "entry_id");
    let error = bikri::edit_save(
        &w.env.ctx,
        &w.owner,
        &EditSaveInput {
            entry_id: entry,
            version: opened.version,
            draft: changed,
            reason: Some("wrong seller".into()),
            confirm_cash_below_zero: true,
        },
    )
    .unwrap_err();
    code_at(error, ErrorCode::ErrGardenClosed, "entry_id");
    // Nothing moved: the garden's account is still 0 and the partners keep what they were paid.
    assert_eq!(credit_balance(&w, garden.account.id), 0);
    assert_eq!(totals_of_three(&w), before);
}

fn value_of(line: &mandi_services::core::statement::StatementLine, name: &str) -> i64 {
    match line.values.get(name) {
        Some(mandi_services::core::error::ErrorValue::Number(n)) => *n,
        other => panic!("{name}: {other:?}"),
    }
}

#[test]
fn the_shares_are_also_locked_once_a_lot_sale_has_credited_the_garden() {
    // Umar (25-10-2026): E66 counts a lot sold into the garden as its first entry too.
    let w = worked_day();
    let garden = create(&w, None);
    let [a, b, c] = w.partners;
    assert!(!detail(&w, garden.id).row.shares_locked);
    set_today(&w.env, 2026, 10, 12);
    sell_a_garden_lot(&w, &garden);
    let now = detail(&w, garden.id).row;
    assert!(now.shares_locked, "a lot sale locks the shares");
    // The garden has no entry of its own, yet the shares are refused.
    assert_eq!(scalar(&w, "SELECT COUNT(*) FROM venture_entries"), 0);
    let error = gardens::set_partners(
        &w.env.ctx,
        &w.owner,
        &GardenSetPartnersInput {
            id: garden.id,
            version: now.version,
            partners: vec![shop(a, "50"), shop(b, "30"), shop(c, "20")],
        },
    )
    .unwrap_err();
    code_at(error, ErrorCode::ErrVentureSharesLocked, "partners");
}

#[test]
fn a_garden_lot_gets_no_cash_and_no_advance_deduction_whatever_the_screen_sends() {
    // PRD FR-12.36: the backend forces both off, like an own lot (FR-12.26).
    let w = worked_day();
    let garden = create(&w, None);
    set_today(&w.env, 2026, 10, 12);
    let lot_with = |cash: Option<&str>| {
        let mut lot = support::with_sales(
            support::bare_lot(&w, "garden-1", garden.account.id, w.banana, "10"),
            vec![support::sale("s-1", w.arshad, "10", "600")],
        );
        lot.adjust_advance = true;
        lot.cash_paid = cash.map(str::to_owned);
        let mut draft = support::draft(vec![lot]);
        draft.header.entry_date = "12-10-2026".into();
        draft
    };
    // A typed نقد is refused at its field.
    let refused = bikri::preview(&w.env.ctx, &lot_with(Some("500"))).unwrap();
    assert!(
        refused
            .errors
            .iter()
            .any(|e| e.code == ErrorCode::ErrFormatInvalid && e.field == "lots[0].cash_paid"),
        "{:?}",
        refused.errors
    );
    // With the advance tick on, the lot is saved with the tick off and no cash given.
    let cash_before = cash_balance(&w);
    save_ok(&w, &lot_with(None), false);
    assert_eq!(
        scalar(
            &w,
            "SELECT adjust_advance FROM bikri_lots WHERE seller_account_id = (SELECT account_id FROM ventures WHERE kind = 'garden')"
        ),
        0
    );
    assert_eq!(
        scalar(
            &w,
            "SELECT cash_paid_paisa FROM bikri_lots WHERE seller_account_id = (SELECT account_id FROM ventures WHERE kind = 'garden')"
        ),
        0
    );
    assert_eq!(cash_balance(&w), cash_before);
}

#[test]
fn a_closing_that_posts_nothing_still_gives_each_partner_a_note_line() {
    // PRD FR-17.10, E64: no sale at all, so the balance is 0 and nothing is posted. Each partner of
    // the garden still gets a closing line with no amount: the cost, the sales, his % and his part.
    use mandi_services::core::statement::StatementDetail;
    let w = worked_day();
    let garden = pay_the_garden(&w);
    set_today(&w.env, 2026, 10, 13);
    gardens::close(
        &w.env.ctx,
        &w.owner,
        &close_input(&w, garden.id, "13-10-2026"),
    )
    .unwrap();
    let [a, b, c] = w.partners;

    let alif = statement_of(&w, a);
    let notes: Vec<_> = alif
        .lines
        .iter()
        .filter(|l| l.detail == StatementDetail::GardenClosed)
        .collect();
    assert_eq!(notes.len(), 1);
    let note = notes[0];
    assert_eq!(note.amount_paisa, None, "a note has no amount");
    assert_eq!(note.running_paisa.rupees(), -33_000, "and leaves the total");
    assert_eq!(alif.closing_total_paisa.rupees(), -33_000);
    assert_eq!(
        (
            value_of(note, "cost_paisa") / 100,
            value_of(note, "sales_paisa") / 100,
            value_of(note, "result_paisa") / 100,
            value_of(note, "share_bps"),
            value_of(note, "part_paisa") / 100,
            value_of(note, "back_paisa") / 100
        ),
        (55_000, 0, -55_000, 6_000, -33_000, 0)
    );
    // ب paid from his pocket: nothing posted for him at all, the note is his only line.
    let bob = statement_of(&w, b);
    assert_eq!(bob.lines.len(), 1);
    assert_eq!(bob.lines[0].detail, StatementDetail::GardenClosed);
    assert_eq!(bob.lines[0].amount_paisa, None);
    assert_eq!(bob.closing_total_paisa.rupees(), 0);
    assert_eq!(value_of(&bob.lines[0], "part_paisa") / 100, -22_000);
    // ج is not in the garden: no line.
    assert!(statement_of(&w, c).lines.is_empty());
    // Undo the closing: the notes go (the closing is not in force any more).
    gardens::reopen(
        &w.env.ctx,
        &w.owner,
        &GardenReopenInput {
            id: garden.id,
            version: detail(&w, garden.id).row.version,
            reason: "wrong".into(),
        },
    )
    .unwrap();
    assert!(statement_of(&w, b).lines.is_empty());
}

#[test]
fn a_partner_whose_part_comes_to_zero_gets_a_note_and_the_other_gets_the_line() {
    // 1 rupee in, split 60 / 40: the leftover goes to the larger share, so ب's part is 0.
    use mandi_services::core::statement::StatementDetail;
    let w = worked_day();
    let garden = create(&w, None);
    gardens::entry_save(
        &w.env.ctx,
        &w.owner,
        &other_sale(garden.id, "10-10-2026", "1", "1"),
    )
    .unwrap();
    gardens::close(
        &w.env.ctx,
        &w.owner,
        &close_input(&w, garden.id, "10-10-2026"),
    )
    .unwrap();
    let [a, b, _] = w.partners;
    let alif = statement_of(&w, a);
    let closing = alif
        .lines
        .iter()
        .find(|l| l.detail == StatementDetail::GardenClosed)
        .unwrap();
    assert_eq!(closing.amount_paisa.unwrap().rupees(), 1, "his 1 rupee");
    assert_eq!(alif.closing_total_paisa.rupees(), 1);
    let bob = statement_of(&w, b);
    assert_eq!(bob.lines.len(), 1);
    assert_eq!(bob.lines[0].detail, StatementDetail::GardenClosed);
    assert_eq!(bob.lines[0].amount_paisa, None);
    assert_eq!(value_of(&bob.lines[0], "back_paisa"), 0);
    assert_eq!(value_of(&bob.lines[0], "share_bps"), 4_000);
    assert_eq!(bob.closing_total_paisa.rupees(), 0);
}

#[test]
fn fruit_taken_from_a_garden_lot_goes_to_the_garden_and_a_closed_garden_refuses_it() {
    // PRD FR-16.11, FR-12.38: the fruit value is credited to the garden's account (money in), the
    // partner who took it pays with his 0604 account, and once the garden is closed the entry can
    // no longer be saved, edited or cancelled until the closing is undone.
    use mandi_services::core::fruit::{FruitCancelInput, FruitInput, FruitLotsInput, FruitSource};
    let w = worked_day();
    let garden = create(&w, Some("30,000"));
    set_today(&w.env, 2026, 10, 12);
    // 100 received, 90 sold at 600 to ارشد: 10 left for fruit.
    let mut lot = support::with_sales(
        support::bare_lot(&w, "garden-1", garden.account.id, w.banana, "100"),
        vec![support::sale("s-1", w.arshad, "90", "600")],
    );
    lot.adjust_advance = false;
    let mut draft = support::draft(vec![lot]);
    draft.header.entry_date = "12-10-2026".into();
    save_ok(&w, &draft, false);
    let lot_id = scalar(
        &w,
        &format!(
            "SELECT id FROM bikri_lots WHERE seller_account_id = {}",
            garden.account.id
        ),
    );
    let fruit_input = |qty: &str| FruitInput {
        for_guests: false,
        partner_id: Some(w.partners[2]),
        date: "12-10-2026".into(),
        source: FruitSource::Shop,
        lot_id: Some(lot_id),
        agent_account_id: None,
        item_id: None,
        packing_id: None,
        qty: qty.into(),
        weight: None,
        rate: "600".into(),
        note: None,
        editing_id: None,
    };
    let money_in_before = detail(&w, garden.id).row.money_in_paisa.rupees();
    let total_before = partner_total(&w, 2);
    let taken = mandi_services::fruit::save(&w.env.ctx, &w.owner, &fruit_input("1")).unwrap();
    // The garden's money in goes up by the 600; ج paid it.
    assert_eq!(
        detail(&w, garden.id).row.money_in_paisa.rupees(),
        money_in_before + 600
    );
    assert_eq!(partner_total(&w, 2), total_before - 600);
    // Fruit is not a closing split: closing 12-10 gives every rupee of the account to the partners.
    set_today(&w.env, 2026, 10, 13);
    gardens::close(
        &w.env.ctx,
        &w.owner,
        &close_input(&w, garden.id, "13-10-2026"),
    )
    .unwrap();
    assert_eq!(credit_balance(&w, garden.account.id), 0);
    let sum: i64 = totals_of_three(&w).iter().sum();
    assert_eq!(
        sum + 65_000,
        super::own_goods::equity(&w),
        "the totals add up to the equity"
    );
    // A closed garden: its lot is not offered, and the entry cannot be saved, edited or cancelled.
    assert_eq!(
        mandi_services::fruit::lots(&w.env.ctx, &FruitLotsInput::default())
            .unwrap()
            .total_count,
        0
    );
    let refused = mandi_services::fruit::save(&w.env.ctx, &w.owner, &fruit_input("1")).unwrap_err();
    code_at(refused, ErrorCode::ErrGardenClosed, "lot_id");
    let row = mandi_services::fruit::get(
        &w.env.ctx,
        &mandi_services::core::fruit::FruitGetInput { id: taken.id },
    )
    .unwrap();
    assert!(!row.can_change);
    let cancel = FruitCancelInput {
        id: taken.id,
        version: row.version,
        reason: "x".into(),
    };
    code_at(
        mandi_services::fruit::cancel(&w.env.ctx, &w.owner, &cancel).unwrap_err(),
        ErrorCode::ErrGardenClosed,
        "id",
    );
    // Undo the closing: the entry can be cancelled again, and the garden account is empty of it.
    gardens::reopen(
        &w.env.ctx,
        &w.owner,
        &GardenReopenInput {
            id: garden.id,
            version: detail(&w, garden.id).row.version,
            reason: "wrong".into(),
        },
    )
    .unwrap();
    mandi_services::fruit::cancel(&w.env.ctx, &w.owner, &cancel).unwrap();
    assert_eq!(
        detail(&w, garden.id).row.money_in_paisa.rupees(),
        money_in_before
    );
}
