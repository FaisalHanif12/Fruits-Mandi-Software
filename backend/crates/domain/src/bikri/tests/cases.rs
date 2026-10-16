//! The cases the money agent required for plan 12 (design review, section c).

use mandi_core::bikri::{
    ChargeInput, ChargeSource, LineChargeInput, LineChargeMethod, LotChargeMethod, PaymentType,
    PukhtaInput, RateHint, RateHintSource,
};
use mandi_core::error::{ErrorCode, ErrorValue};
use mandi_core::money::Paisa;
use mandi_core::rules::ChargeValue;

use super::*;
use crate::bikri::checks::far_from_average;
use crate::cash;

fn number(list: &[mandi_core::error::FieldError], code: ErrorCode, key: &str) -> Option<i64> {
    list.iter()
        .find(|e| e.code == code)
        .and_then(|e| match e.values.get(key) {
            Some(ErrorValue::Number(n)) => Some(*n),
            _ => None,
        })
}

// ---- Per-lot commission (PRD FR-11.9, money agent point 2) ----

fn per_lot_rule() -> Vec<Rule> {
    let mut rules = default_rules();
    rules.push(Rule {
        account_id: Some(RAHIM),
        ..rule(
            10,
            ChargeSide::Seller,
            ChargeType::Commission,
            ChargeValue::PerLot(p(500)),
        )
    });
    rules
}

fn run_per_lot(draft: &mandi_core::bikri::BikriDraft) -> PostingCalc {
    run_with(draft, &lookups(), per_lot_rule(), switches())
}

#[test]
fn a_per_lot_commission_is_charged_once_on_the_first_line() {
    let calc = run_per_lot(&event1());
    assert_no_errors(&calc);
    let first = &line(&calc, 0, 0).seller_comm;
    let second = &line(&calc, 0, 1).seller_comm;
    assert_eq!((r(first.amount), r(second.amount)), (500, 0));
    assert_eq!(first.working.message_id, "working-per-lot");
    assert_eq!(second.working.message_id, "working-per-lot-already-charged");
    let t = single_lot_totals(&calc);
    assert_eq!(r(t.seller_comm), 500);
    assert_eq!(r(t.deductions), 8_820);
    assert_eq!(r(t.net), 20_580);
    assert_eq!(r(t.seller_credit), 25_580);
    assert_eq!(r(t.shared_income), 500 + 1_764 + 400 + 20);
    // The sale part still balances at 31,564, and the whole entry is 36,564 (the same as E2E-3).
    assert_eq!(entry_of(&calc).total(), p(36_564));
    assert_every_working_matches(&calc);
}

#[test]
fn a_per_lot_commission_moves_to_the_first_line_that_can_carry_it() {
    // A blank first row.
    let calc = run_per_lot(&draft(vec![with(
        lot(RAHIM, BANANA, "50"),
        vec![
            blank("50"),
            sale(AKRAM, "20", "600"),
            sale(ARSHAD, "30", "580"),
        ],
    )]));
    assert_no_errors(&calc);
    assert_eq!(r(line(&calc, 0, 1).seller_comm.amount), 500);
    assert_eq!(r(line(&calc, 0, 2).seller_comm.amount), 0);

    // A first row with no rate yet is not calculable, so the second carries the charge.
    let calc = run_per_lot(&draft(vec![with(
        lot(RAHIM, BANANA, "50"),
        vec![sale(AKRAM, "20", ""), sale(ARSHAD, "30", "580")],
    )]));
    assert!(has(
        &calc.errors,
        ErrorCode::ErrRequired,
        "lots[0].sales[0].seller_rate"
    ));
    assert!(calc.lots[0].lines[0].calc.is_none());
    assert_eq!(r(line(&calc, 0, 1).seller_comm.amount), 500);

    // A 4% typed on line 1 (480) replaces the rule there, so line 2 carries the 500: the lot
    // is still charged exactly once.
    let mut first = sale(AKRAM, "20", "600");
    first.seller_comm = Some(LineChargeInput {
        method: LineChargeMethod::Percent,
        value: "4".into(),
    });
    let calc = run_per_lot(&draft(vec![with(
        lot(RAHIM, BANANA, "50"),
        vec![first, sale(ARSHAD, "30", "580")],
    )]));
    assert_no_errors(&calc);
    assert_eq!(r(line(&calc, 0, 0).seller_comm.amount), 480);
    assert_eq!(r(line(&calc, 0, 1).seller_comm.amount), 500);
    assert_eq!(r(single_lot_totals(&calc).seller_comm), 980);
}

#[test]
fn a_per_lot_commission_is_charged_on_each_lot_of_the_seller() {
    let calc = run_per_lot(&draft(vec![
        with(lot(RAHIM, BANANA, "50"), vec![sale(AKRAM, "20", "600")]),
        with(lot(RAHIM, APPLE, "10"), vec![sale(ARSHAD, "10", "1,200")]),
    ]));
    assert_eq!(r(line(&calc, 1, 0).seller_comm.amount), 500);
    assert_eq!(r(calc.lots[0].totals.as_ref().unwrap().seller_comm), 500);
    assert_eq!(r(calc.summary.seller_comm), 1_000);
}

#[test]
fn a_later_posting_charges_no_lot_charge_and_no_per_lot_commission() {
    // PRD FR-12.17: freight, store rent, the lot charges and the per-lot commission belong to the
    // first posting. Only the sale itself, its commissions and لاگا are posted again.
    let mut event = lot(RAHIM, BANANA, "50");
    event.store_rent = Some("700".into());
    event.sales = vec![sale(AKRAM, "20", "600"), sale(ARSHAD, "30", "580")];
    let prepared = mandi_core::bikri::prepare_draft(&draft(vec![event]), today(), None);
    let calc = calc_posting(
        PostingKind::Later,
        &prepared,
        &lookups(),
        &RuleSet::build(per_lot_rule()),
        switches(),
    )
    .unwrap();
    assert_no_errors(&calc);
    assert_eq!(r(line(&calc, 0, 0).seller_comm.amount), 0);
    let t = single_lot_totals(&calc);
    assert_eq!(
        (
            r(t.labour),
            r(t.munshiana),
            r(t.phone),
            r(t.freight),
            r(t.store_rent)
        ),
        (0, 0, 0, 0, 0),
        "no lot charge in a later sale"
    );
    // Gross 29,400, nothing taken off but نقد as typed; the buyers' commissions and لاگا stay.
    assert_eq!(r(t.seller_credit), 29_400);
    assert_eq!(r(t.net), 24_400);
    assert_eq!(r(t.shared_income), 1_764 + 400);
    // The entry still balances: 31,564 of buyer bills = 29,400 + 1,764 + 400.
    assert_eq!(entry_of(&calc).total(), p(31_564 + 5_000));
}

#[test]
fn a_per_lot_commission_is_not_lost_when_the_first_line_is_refused() {
    // Money agent F2. Line 1 is worth exactly the most a line can hold, so its buyer total is
    // over the limit once the commission is added and the line is refused. The 500 then goes to
    // the next line, and the lot is still charged once.
    let mut huge = lot(RAHIM, BANANA, "200");
    huge.sales = vec![
        sale(AKRAM, "100", "10,000,000,000"),
        sale(ARSHAD, "20", "600"),
    ];
    let calc = run_per_lot(&draft(vec![huge]));
    assert!(has(
        &calc.errors,
        ErrorCode::ErrAmountTooLarge,
        "lots[0].sales[0].seller_rate"
    ));
    assert!(calc.lots[0].lines[0].calc.is_none());
    assert_eq!(r(line(&calc, 0, 1).seller_comm.amount), 500);
    assert_eq!(
        r(single_lot_totals(&calc).seller_comm),
        500,
        "charged exactly once"
    );
}

// ---- Money agent cases 5 to 10 ----

#[test]
fn e28_a_negative_net_is_allowed_with_a_warning() {
    // 1 received, 1 x 600, freight 3,000 (firm), نقد 5,000, no labour, munshiana or phone.
    let calc = run(&draft(vec![with(
        bare_lot_with_freight_and_cash(RAHIM, BANANA, "1"),
        vec![sale(AKRAM, "1", "600")],
    )]));
    assert_no_errors(&calc);
    let l = line(&calc, 0, 0);
    assert_eq!(
        (
            r(l.seller_comm.amount),
            r(l.buyer_comm.amount),
            r(l.laga.amount),
            r(l.buyer_total)
        ),
        (36, 36, 8, 644)
    );
    let t = single_lot_totals(&calc);
    assert_eq!(r(t.deductions), 8_036);
    assert_eq!(r(t.net), -7_436);
    assert_eq!(r(t.seller_credit), -2_436);
    assert_eq!(r(t.shared_income), 80);
    assert!(calc.lots[0].net_is_negative);
    assert_eq!(
        number(&calc.warnings, ErrorCode::ErrLotNetNegative, "net_paisa"),
        Some(-743_600)
    );
    // Sale part 3,080 = 3,080 (a debit on the seller), whole entry 8,080.
    assert_eq!(entry_of(&calc).total(), p(8_080));
    assert_every_working_matches(&calc);
}

fn bare_lot_with_freight_and_cash(
    seller: i64,
    item: i64,
    qty: &str,
) -> mandi_core::bikri::LotInput {
    mandi_core::bikri::LotInput {
        labour: None,
        munshiana: None,
        phone: None,
        ..lot(seller, item, qty)
    }
}

#[test]
fn a_seller_credit_of_exactly_zero_posts_no_seller_line() {
    // 1 x 1,000, freight 940: commission 60, credit 1,000 - 60 - 940 = 0.
    let mut one = bare_lot(RAHIM, BANANA, "1");
    one.freight = Some("940".into());
    let calc = run(&draft(vec![with(one, vec![sale(AKRAM, "1", "1,000")])]));
    assert_no_errors(&calc);
    let t = single_lot_totals(&calc);
    assert_eq!(r(t.seller_credit), 0);
    assert_eq!(r(t.net), 0);
    assert!(calc.warnings.is_empty(), "net 0 is not negative");
    let entry = entry_of(&calc);
    assert_eq!(entry.total(), p(1_068));
    assert!(
        entry
            .lines()
            .iter()
            .all(|l| l.account_id != AccountId::new(RAHIM))
    );
}

#[test]
fn a_different_buyer_rate_makes_a_rate_difference() {
    let one_line = |buyer_rate: &str| {
        let mut first = sale(AKRAM, "20", "600");
        first.rate_differs = true;
        first.buyer_rate = Some(buyer_rate.into());
        run(&draft(vec![with(
            bare_lot(RAHIM, BANANA, "20"),
            vec![first],
        )]))
    };
    // Buyer rate 590: value 11,800, commission 708, total 12,668, difference -200.
    let calc = one_line("590");
    assert_no_errors(&calc);
    let l = line(&calc, 0, 0);
    assert_eq!(
        (
            r(l.buyer_value),
            r(l.buyer_comm.amount),
            r(l.buyer_total),
            r(l.rate_diff)
        ),
        (11_800, 708, 12_668, -200)
    );
    assert_eq!(r(single_lot_totals(&calc).shared_income), 1_388);
    assert_eq!(entry_of(&calc).total(), p(12_868));
    assert_every_working_matches(&calc);
    // Buyer rate 610: total 13,092, difference +200.
    let calc = one_line("610");
    let l = line(&calc, 0, 0);
    assert_eq!((r(l.buyer_total), r(l.rate_diff)), (13_092, 200));
    assert_eq!(entry_of(&calc).total(), p(13_092));
}

#[test]
fn commissions_are_summed_from_the_lines_never_rounded_again() {
    // Two lines of 9 x 625 = 5,625: 6% is 337.5, which rounds to 338 on each line. The lot
    // commission is 676, not 6% of 11,250 = 675.
    let calc = run(&draft(vec![with(
        bare_lot(RAHIM, BANANA, "18"),
        vec![sale(AKRAM, "9", "625"), sale(ARSHAD, "9", "625")],
    )]));
    assert_no_errors(&calc);
    assert_eq!(r(line(&calc, 0, 0).seller_comm.amount), 338);
    assert_eq!(r(line(&calc, 0, 1).seller_comm.amount), 338);
    assert_eq!(r(single_lot_totals(&calc).seller_comm), 676);
    assert_eq!(r(calc.summary.buyer_comm), 676);
}

#[test]
fn a_weight_item_values_the_weight_and_charges_per_nag_on_the_count() {
    // E23: 2 nag, 12.345 kg at 100 per kg = 1,234.5, rounds to 1,235.
    let mut grapes = bare_lot(RAHIM, GRAPES, "2");
    grapes.weight_received = Some("12.345".into());
    let mut row = sale(AKRAM, "2", "100");
    row.weight = Some("12.345".into());
    let calc = run(&draft(vec![with(grapes, vec![row])]));
    assert_no_errors(&calc);
    let l = line(&calc, 0, 0);
    assert_eq!(r(l.seller_amount), 1_235);
    assert_eq!((r(l.seller_comm.amount), r(l.buyer_comm.amount)), (74, 74));
    assert_eq!(r(l.laga.amount), 16, "لاگا counts the 2 nag, not the kg");
    assert_eq!(r(l.buyer_total), 1_325);
    assert_eq!(l.weight_grams, Some(12_345));
    assert_eq!(l.workings.seller_amount.message_id, "working-weight-value");
    assert_every_working_matches(&calc);
}

#[test]
fn a_weight_item_needs_a_weight_and_a_value_above_zero() {
    let mut grapes = bare_lot(RAHIM, GRAPES, "2");
    grapes.weight_received = Some("12.345".into());
    let calc = run(&draft(vec![with(
        grapes.clone(),
        vec![sale(AKRAM, "2", "100")],
    )]));
    assert!(has(
        &calc.errors,
        ErrorCode::ErrWeightRequired,
        "lots[0].sales[0].weight"
    ));
    // 0.001 kg at 1 rupee per kg is worth less than a rupee: refused (PRD E24).
    let mut tiny = sale(AKRAM, "2", "1");
    tiny.weight = Some("0.001".into());
    let calc = run(&draft(vec![with(grapes, vec![tiny])]));
    assert!(has(
        &calc.errors,
        ErrorCode::ErrMustBeMoreThanZero,
        "lots[0].sales[0].weight"
    ));
    assert!(calc.lots[0].lines[0].calc.is_none());
    // A lot of a weight item needs its weight received.
    let calc = run(&draft(vec![with(
        bare_lot(RAHIM, GRAPES, "2"),
        vec![sale(AKRAM, "2", "100")],
    )]));
    assert!(has(
        &calc.errors,
        ErrorCode::ErrWeightRequired,
        "lots[0].weight_received"
    ));
}

#[test]
fn a_weight_typed_for_an_item_sold_by_nag_is_ignored() {
    let mut row = sale(AKRAM, "20", "600");
    row.weight = Some("5".into());
    let calc = run(&draft(vec![with(lot(RAHIM, BANANA, "50"), vec![row])]));
    assert_no_errors(&calc);
    let l = line(&calc, 0, 0);
    assert_eq!((r(l.seller_amount), l.weight_grams), (12_000, None));
}

#[test]
fn e6_freight_paid_by_the_seller_is_shown_but_not_deducted() {
    let mut event = lot(RAHIM, BANANA, "50");
    event.freight_paid_by = mandi_core::bikri::FreightPaidBy::Seller;
    let calc = run(&draft(vec![with(
        event,
        vec![sale(AKRAM, "20", "600"), sale(ARSHAD, "30", "580")],
    )]));
    assert_no_errors(&calc);
    let t = single_lot_totals(&calc);
    assert_eq!((r(t.freight), r(t.deductions)), (0, 7_084));
    assert_eq!((r(t.net), r(t.seller_credit)), (22_316, 27_316));
    assert_eq!(
        r(calc.lots[0].freight),
        3_000,
        "the typed freight is still shown"
    );
    assert!(
        entry_of(&calc)
            .lines()
            .iter()
            .all(|l| l.account_id != system_accounts().freight)
    );
}

#[test]
fn cash_change_of_event_2_with_cash_paid_is_plus_7_800() {
    let mut karim = lot(KARIM, APPLE, "10");
    karim.freight = Some("1,000".into());
    let mut shahid = sale(PUKHTA_BUYER, "10", "1,200");
    shahid.pukhta = Some(PukhtaInput {
        name_ur: "شاہد".into(),
        name_en: "Shahid".into(),
        phone: String::new(),
    });
    let calc = run(&draft(vec![with(karim, vec![shahid])]));
    assert_no_errors(&calc);
    let change = cash::net_change(&entry_of(&calc), system_accounts().cash).unwrap();
    assert_eq!(change, p(7_800));
    assert!(!cash::check(p(0), change).unwrap().needs_confirm);
}

// ---- Rate hint and remaining quantity (PRD FR-7.52, FR-7.53, E2E-26) ----

#[test]
fn a_new_row_starts_at_the_remaining_quantity_with_the_rate_of_the_line_above() {
    let calc = run(&draft(vec![with(
        lot(RAHIM, BANANA, "50"),
        vec![sale(AKRAM, "20", "600"), blank("30")],
    )]));
    let rows = &calc.lots[0].lines;
    assert_eq!(rows[0].remaining_qty, 50);
    assert_eq!(rows[1].remaining_qty, 30);
    let hint = rows[1].rate_hint.as_ref().unwrap();
    assert_eq!(
        (r(hint.rate_paisa), hint.source),
        (600, RateHintSource::ThisBikri)
    );
    // The first row has no earlier line to take a hint from.
    assert!(rows[0].rate_hint.is_none());
}

#[test]
fn a_saved_rate_is_the_hint_until_an_earlier_line_has_one() {
    let mut lookups = lookups();
    lookups.saved_hints.insert(
        (BANANA, BOX),
        RateHint {
            rate_paisa: p(590),
            source: RateHintSource::LastDay,
            date: "2026-10-08".into(),
        },
    );
    let calc = run_with(
        &draft(vec![with(
            lot(RAHIM, BANANA, "50"),
            vec![blank("50"), sale(AKRAM, "20", "600"), blank("30")],
        )]),
        &lookups,
        default_rules(),
        switches(),
    );
    let rows = &calc.lots[0].lines;
    assert_eq!(
        rows[0]
            .rate_hint
            .as_ref()
            .map(|h| (r(h.rate_paisa), h.source)),
        Some((590, RateHintSource::LastDay))
    );
    assert_eq!(
        rows[2]
            .rate_hint
            .as_ref()
            .map(|h| (r(h.rate_paisa), h.source)),
        Some((600, RateHintSource::ThisBikri))
    );
}

#[test]
fn quantities_above_the_lot_are_refused_at_the_row_that_crosses() {
    // 50 received; lines 20, 30, 5: the third is over, with nothing available.
    let calc = run(&draft(vec![with(
        lot(RAHIM, BANANA, "50"),
        vec![
            sale(AKRAM, "20", "600"),
            sale(ARSHAD, "30", "580"),
            sale(AKRAM, "5", "600"),
        ],
    )]));
    assert_eq!(calc.lots[0].lines[2].remaining_qty, 0);
    assert_eq!(
        number(&calc.errors, ErrorCode::ErrQtyExceedsLot, "available"),
        Some(0)
    );
    assert!(has(
        &calc.errors,
        ErrorCode::ErrQtyExceedsLot,
        "lots[0].sales[2].qty"
    ));
    assert_eq!(calc.lots[0].balance_qty, Some(0), "never below 0");
    // Lines 20 and 40: the second has 30 available.
    let calc = run(&draft(vec![with(
        lot(RAHIM, BANANA, "50"),
        vec![sale(AKRAM, "20", "600"), sale(ARSHAD, "40", "580")],
    )]));
    assert_eq!(
        number(&calc.errors, ErrorCode::ErrQtyExceedsLot, "available"),
        Some(30)
    );
}

#[test]
fn a_blank_row_with_a_quantity_is_ignored() {
    let calc = run(&draft(vec![with(
        lot(RAHIM, BANANA, "50"),
        vec![
            sale(AKRAM, "20", "600"),
            blank("99"),
            sale(ARSHAD, "30", "580"),
        ],
    )]));
    assert_no_errors(&calc);
    let rows = &calc.lots[0].lines;
    assert!(rows[1].is_blank && rows[1].calc.is_none());
    assert_eq!((rows[1].remaining_qty, rows[2].remaining_qty), (30, 30));
    assert_eq!(calc.lots[0].sold_qty, 50);
}

#[test]
fn a_half_typed_row_counts_in_the_remaining_quantity_and_the_warning() {
    // Row 2 has a qty but no rate yet: it is not worked out, but its qty is already taken.
    let calc = run(&draft(vec![with(
        lot(RAHIM, BANANA, "50"),
        vec![sale(AKRAM, "20", "600"), sale(ARSHAD, "10", "")],
    )]));
    assert_eq!(calc.lots[0].sold_qty, 30);
    assert_eq!(calc.lots[0].balance_qty, Some(20));
    assert_eq!(
        number(&calc.warnings, ErrorCode::ErrQtyLeftUnsold, "left"),
        Some(20)
    );
}

// ---- Lot charges (money agent point 3, case 15) ----

#[test]
fn a_rule_suggests_a_lot_charge_and_says_so() {
    let mut rules = default_rules();
    rules.push(Rule {
        account_id: Some(RAHIM),
        ..rule(
            11,
            ChargeSide::Seller,
            ChargeType::Labour,
            ChargeValue::PerUnit(p(5)),
        )
    });
    let calc = run_with(
        &draft(vec![with(
            bare_lot(RAHIM, BANANA, "50"),
            vec![sale(AKRAM, "20", "600"), sale(ARSHAD, "30", "580")],
        )]),
        &lookups(),
        rules,
        switches(),
    );
    assert_no_errors(&calc);
    let labour = calc.lots[0].labour.as_ref().unwrap();
    assert_eq!((r(labour.amount), labour.value_paisa), (250, p(5)));
    assert_eq!(labour.source, ChargeSource::Rule);
    assert_eq!(labour.rule_id, Some(11));
    assert_eq!(labour.method, LotChargeMethod::PerUnit);
    // Nothing typed and no rule: 0, and it says so.
    let none = calc.lots[0].munshiana.as_ref().unwrap();
    assert_eq!((r(none.amount), none.source), (0, ChargeSource::None));
}

#[test]
fn typed_lot_charges_per_lot_and_manual() {
    let mut l = lot(RAHIM, BANANA, "50");
    l.labour = Some(ChargeInput {
        method: LotChargeMethod::PerLot,
        value: "300".into(),
    });
    l.munshiana = manual("275");
    let calc = run(&draft(vec![with(l, vec![sale(AKRAM, "50", "600")])]));
    assert_eq!(r(calc.lots[0].labour.as_ref().unwrap().amount), 300);
    assert_eq!(
        calc.lots[0].labour.as_ref().unwrap().working.message_id,
        "working-manual-per-lot"
    );
    assert_eq!(r(calc.lots[0].munshiana.as_ref().unwrap().amount), 275);
    assert_every_working_matches(&calc);
}

// ---- Payment type, credit limit (points 5 and 8) ----

#[test]
fn credit_is_refused_for_the_cash_buyer_and_a_cash_only_buyer() {
    let credit = |buyer| {
        let mut row = sale(buyer, "20", "600");
        row.payment_type = Some(PaymentType::Credit);
        run(&draft(vec![with(lot(RAHIM, BANANA, "50"), vec![row])]))
    };
    for buyer in [NAQDI, CASH_ONLY_BUYER] {
        let calc = credit(buyer);
        assert!(
            has(
                &calc.errors,
                ErrorCode::ErrCreditNotAllowed,
                "lots[0].sales[0].payment_type"
            ),
            "{buyer}"
        );
        assert!(calc.lots[0].lines[0].calc.is_none());
    }
    // Their default is cash.
    for buyer in [NAQDI, CASH_ONLY_BUYER] {
        let calc = run(&draft(vec![with(
            lot(RAHIM, BANANA, "50"),
            vec![sale(buyer, "20", "600")],
        )]));
        assert_no_errors(&calc);
        assert_eq!(line(&calc, 0, 0).payment_type, PaymentType::Cash);
    }
    // A regular buyer defaults to credit and may pay cash if the line says so.
    let calc = run(&event1());
    assert_eq!(line(&calc, 0, 0).payment_type, PaymentType::Credit);
}

#[test]
fn a_one_time_credit_buyer_needs_a_phone_and_a_name() {
    let mut row = sale(PUKHTA_BUYER, "20", "600");
    row.payment_type = Some(PaymentType::Credit);
    let calc = run(&draft(vec![with(lot(RAHIM, BANANA, "50"), vec![row])]));
    for field in [
        "lots[0].sales[0].pukhta.name_ur",
        "lots[0].sales[0].pukhta.name_en",
    ] {
        assert!(
            has(&calc.errors, ErrorCode::ErrPukhtaNameRequired, field),
            "{field}"
        );
    }
    assert!(has(
        &calc.errors,
        ErrorCode::ErrPukhtaPhoneRequired,
        "lots[0].sales[0].pukhta.phone"
    ));
    assert!(calc.lots[0].lines[0].calc.is_none());
}

fn with_limit(limit: i64, balance: i64) -> Lookups {
    let mut l = lookups();
    if let Some(a) = l.accounts.get_mut(&AKRAM) {
        a.credit_limit = Some(p(limit));
        a.balance = p(balance);
    }
    l
}

fn akram_12_880(lookups: &Lookups, blocks: bool) -> PostingCalc {
    let mut sw = switches();
    sw.credit_limit_blocks = blocks;
    run_with(
        &draft(vec![with(
            lot(RAHIM, BANANA, "50"),
            vec![sale(AKRAM, "20", "600")],
        )]),
        lookups,
        default_rules(),
        sw,
    )
}

#[test]
fn the_credit_limit_warns_only_above_the_limit() {
    // Exactly at the limit: fine.
    let calc = akram_12_880(&with_limit(12_880, 0), false);
    assert!(
        !calc
            .warnings
            .iter()
            .any(|w| w.code == ErrorCode::ErrCreditLimitExceeded)
    );
    // One rupee under: a warning, with the numbers.
    let calc = akram_12_880(&with_limit(12_879, 0), false);
    let field = "lots[0].sales[0].buyer_account_id";
    assert!(has(
        &calc.warnings,
        ErrorCode::ErrCreditLimitExceeded,
        field
    ));
    assert_eq!(
        number(
            &calc.warnings,
            ErrorCode::ErrCreditLimitExceeded,
            "limit_paisa"
        ),
        Some(1_287_900)
    );
    assert_eq!(
        number(
            &calc.warnings,
            ErrorCode::ErrCreditLimitExceeded,
            "would_owe_paisa"
        ),
        Some(1_288_000)
    );
    assert!(calc.errors.is_empty());
    // The same, when the Owner chose to block: an error.
    let calc = akram_12_880(&with_limit(12_879, 0), true);
    assert!(has(&calc.errors, ErrorCode::ErrCreditLimitExceeded, field));
    // A credit balance of 1,000 counts against what he would owe: 11,880 < 12,000.
    let calc = akram_12_880(&with_limit(12_000, -1_000), false);
    assert!(
        !calc
            .warnings
            .iter()
            .any(|w| w.code == ErrorCode::ErrCreditLimitExceeded)
    );
}

#[test]
fn the_credit_limit_adds_up_across_lots_and_ignores_cash_rows() {
    let lookups = with_limit(20_000, 0);
    let mut cash_row = sale(AKRAM, "10", "1,000");
    cash_row.payment_type = Some(PaymentType::Cash);
    // Lot 1: 12,880 on credit. Lot 2: a cash row (not counted) and 10 x 1,000 on credit.
    let calc = run_with(
        &draft(vec![
            with(lot(RAHIM, BANANA, "50"), vec![sale(AKRAM, "20", "600")]),
            with(
                lot(KARIM, APPLE, "30"),
                vec![cash_row, sale(AKRAM, "10", "1,000")],
            ),
        ]),
        &lookups,
        default_rules(),
        switches(),
    );
    let warned: Vec<&str> = calc
        .warnings
        .iter()
        .filter(|w| w.code == ErrorCode::ErrCreditLimitExceeded)
        .map(|w| w.field.as_str())
        .collect();
    // 12,880 + 10,680 = 23,560 is above 20,000 only at the last row.
    assert_eq!(warned, vec!["lots[1].sales[1].buyer_account_id"]);
}

// ---- Rate check (PRD FR-7.55, E8; money agent points 9, cases 1 to 3) ----

fn sales(qty: i64, rupees: i64) -> ItemSales {
    ItemSales {
        qty,
        amount: p(rupees),
    }
}

#[test]
fn the_rate_check_boundaries_with_an_average_of_1_000() {
    // Q = 10, A = 10,000, 30%: 1,300 and 700 are fine, 1,301 and 699 are far.
    let far = |rate: i64| far_from_average(p(rate), sales(10, 10_000), 30).unwrap();
    assert_eq!(far(1_300), None);
    assert_eq!(far(1_301), Some(p(1_000)));
    assert_eq!(far(700), None);
    assert_eq!(far(699), Some(p(1_000)));
}

#[test]
fn the_rate_check_does_not_round_the_average_first() {
    // Q = 3, A = 1,000: the average is 333.33; the shown average is 333.
    let far = |rate: i64| far_from_average(p(rate), sales(3, 1_000), 30).unwrap();
    assert_eq!(far(433), None, "433 x 3 = 1,299 against 1,300");
    assert_eq!(far(434), Some(p(333)));
    assert_eq!(far(234), None, "234 x 3 = 702 against 700");
    assert_eq!(far(233), Some(p(333)));
}

#[test]
fn the_rate_check_has_nothing_to_compare_without_sales_or_with_a_wide_percent() {
    assert_eq!(
        far_from_average(p(5_000), sales(0, 1_000), 30).unwrap(),
        None
    );
    assert_eq!(far_from_average(p(5_000), sales(10, 0), 30).unwrap(), None);
    assert_eq!(
        far_from_average(p(5_000), sales(10, -100), 30).unwrap(),
        None
    );
    // 100% or more: nothing is "below", but a high rate is still far.
    assert_eq!(
        far_from_average(p(1), sales(10, 10_000), 100).unwrap(),
        None
    );
    assert!(
        far_from_average(p(2_001), sales(10, 10_000), 100)
            .unwrap()
            .is_some()
    );
}

fn with_sales_of_banana() -> Lookups {
    let mut l = lookups();
    l.item_sales.insert(BANANA, sales(10, 10_000));
    l
}

#[test]
fn the_rate_check_warns_on_the_line_and_never_blocks() {
    let calc = run_with(
        &draft(vec![with(
            lot(RAHIM, BANANA, "50"),
            vec![sale(AKRAM, "20", "1,300"), sale(ARSHAD, "30", "1,301")],
        )]),
        &with_sales_of_banana(),
        default_rules(),
        switches(),
    );
    assert_no_errors(&calc);
    let warned: Vec<&str> = calc
        .warnings
        .iter()
        .filter(|w| w.code == ErrorCode::ErrRateFarFromAverage)
        .map(|w| w.field.as_str())
        .collect();
    assert_eq!(warned, vec!["lots[0].sales[1].seller_rate"]);
    assert_eq!(
        number(
            &calc.warnings,
            ErrorCode::ErrRateFarFromAverage,
            "average_paisa"
        ),
        Some(100_000)
    );
    assert_eq!(
        number(&calc.warnings, ErrorCode::ErrRateFarFromAverage, "percent"),
        Some(30)
    );
    // The warning is on the row too, and the row has its numbers.
    assert_eq!(calc.lots[0].lines[1].warnings.len(), 1);
    assert!(calc.lots[0].lines[1].calc.is_some());
}

#[test]
fn the_rate_check_also_looks_at_a_different_buyer_rate() {
    let mut row = sale(AKRAM, "20", "1,000");
    row.rate_differs = true;
    row.buyer_rate = Some("1,400".into());
    let calc = run_with(
        &draft(vec![with(lot(RAHIM, BANANA, "50"), vec![row])]),
        &with_sales_of_banana(),
        default_rules(),
        switches(),
    );
    let warned: Vec<&str> = calc
        .warnings
        .iter()
        .filter(|w| w.code == ErrorCode::ErrRateFarFromAverage)
        .map(|w| w.field.as_str())
        .collect();
    assert_eq!(warned, vec!["lots[0].sales[0].buyer_rate"]);
}

#[test]
fn the_rate_check_is_off_when_switched_off_and_skips_weight_items() {
    let mut off = switches();
    off.rate_check_percent = None;
    let calc = run_with(
        &draft(vec![with(
            lot(RAHIM, BANANA, "50"),
            vec![sale(AKRAM, "20", "5,000")],
        )]),
        &with_sales_of_banana(),
        default_rules(),
        off,
    );
    assert!(
        calc.warnings
            .iter()
            .all(|w| w.code != ErrorCode::ErrRateFarFromAverage)
    );
    // A weight item: its rate is per kg, so the average per nag does not apply.
    let mut l = lookups();
    l.item_sales.insert(GRAPES, sales(10, 10_000));
    let mut grapes = bare_lot(RAHIM, GRAPES, "2");
    grapes.weight_received = Some("12.345".into());
    let mut row = sale(AKRAM, "2", "5,000");
    row.weight = Some("12.345".into());
    let calc = run_with(
        &draft(vec![with(grapes, vec![row])]),
        &l,
        default_rules(),
        switches(),
    );
    assert!(
        calc.warnings
            .iter()
            .all(|w| w.code != ErrorCode::ErrRateFarFromAverage)
    );
}

// ---- Limits, required fields, one-time seller ----

#[test]
fn an_amount_above_what_a_line_can_hold_is_refused_at_the_rate() {
    let calc = run(&draft(vec![with(
        lot(RAHIM, BANANA, "1,000,000"),
        vec![sale(AKRAM, "1,000,000", "10,000,000,000")],
    )]));
    assert!(has(
        &calc.errors,
        ErrorCode::ErrAmountTooLarge,
        "lots[0].sales[0].seller_rate"
    ));
    assert!(calc.lots[0].lines[0].calc.is_none());
    assert_eq!(
        number(&calc.errors, ErrorCode::ErrAmountTooLarge, "max"),
        Some(1_000_000_000_000)
    );
}

#[test]
fn a_draft_needs_a_lot_and_one_sale_line_in_the_whole() {
    // PRD FR-12.8 check 1, read literally: one lot and one sale line for the whole بکری.
    let calc = run(&draft(vec![]));
    assert!(has(&calc.errors, ErrorCode::ErrNoLots, "lots"));
    assert!(!has(&calc.errors, ErrorCode::ErrNoSaleLines, "lots"));
    let calc = run(&draft(vec![with(lot(RAHIM, BANANA, "50"), vec![])]));
    assert!(has(&calc.errors, ErrorCode::ErrNoSaleLines, "lots"));
    let calc = run(&draft(vec![with(
        lot(RAHIM, BANANA, "50"),
        vec![blank("50")],
    )]));
    assert!(has(&calc.errors, ErrorCode::ErrNoSaleLines, "lots"));
}

#[test]
fn a_lot_with_no_sale_row_waits_in_stock_when_another_lot_sold() {
    // PRD E3: a lot that is not fully sold only warns; here nothing of the second lot is sold.
    // Its charges are posted, its net is shown and warns, and it needs no row of its own.
    let calc = run(&draft(vec![
        with(lot(RAHIM, BANANA, "50"), vec![sale(AKRAM, "50", "600")]),
        with(lot(KARIM, APPLE, "10"), vec![]),
    ]));
    assert_no_errors(&calc);
    let second = &calc.lots[1];
    assert_eq!((second.sold_qty, second.balance_qty), (0, Some(10)));
    assert!(
        calc.warnings
            .iter()
            .any(|w| w.code == ErrorCode::ErrQtyLeftUnsold && w.field == "lots[1].qty_received")
    );
    let t = second.totals.as_ref().expect("the lot's charges are shown");
    // Freight 3,000 + labour 10 x 5 + munshiana 50 + phone 20 + نقد 5,000, and nothing sold.
    assert_eq!(
        (r(t.gross), r(t.deductions)),
        (0, 3_000 + 50 + 50 + 20 + 5_000)
    );
    assert!(second.net_is_negative);
    assert!(
        calc.warnings
            .iter()
            .any(|w| w.code == ErrorCode::ErrLotNetNegative
                && w.field == "lots[1].seller_account_id")
    );
    // Nothing was sold, so the credit is minus the charges (3,120): a debit on the seller, not
    // an advance. Its phone charge is income all the same.
    assert_eq!(
        (r(t.seller_credit), r(t.net), r(t.shared_income)),
        (-3_120, -8_120, 20)
    );
    // The first lot: 30,000 sold, commissions 1,800 each, لاگا 400, buyer bill 32,200.
    let first = calc.lots[0].totals.as_ref().unwrap();
    assert_eq!(
        (r(first.buyer_total), r(first.seller_credit)),
        (32_200, 24_880)
    );
    assert_eq!(r(calc.summary.shared_income), 1_800 + 1_800 + 400 + 40);
    assert_eq!(r(calc.summary.net), 19_880 - 8_120);
    // The entry balances. Sale part: Dr 32,200 + 3,120 = 35,320 = Cr 24,880 + 1,800 + 1,800 +
    // 400 + 40 + 300 + 6,000 + 100. Cash part: 5,000 + 5,000 = 10,000. Whole entry 45,320.
    assert_eq!(entry_of(&calc).total(), p(45_320));
    assert_every_working_matches(&calc);
}

#[test]
fn the_screen_is_told_which_lots_take_a_weight() {
    let calc = run(&draft(vec![
        with(lot(RAHIM, BANANA, "50"), vec![sale(AKRAM, "20", "600")]),
        with(bare_lot(KARIM, GRAPES, "2"), vec![]),
    ]));
    assert!(!calc.lots[0].sold_by_weight);
    assert!(calc.lots[1].sold_by_weight);
    // No item picked yet: not by weight.
    let mut none = lot(RAHIM, BANANA, "50");
    none.item_id = None;
    assert!(!run(&draft(vec![none])).lots[0].sold_by_weight);
}

#[test]
fn accounts_must_be_the_right_kind_and_active() {
    let calc = run(&draft(vec![with(
        lot(OWN_GOODS, BANANA, "50"),
        vec![sale(RAHIM, "20", "600")],
    )]));
    assert!(has(
        &calc.errors,
        ErrorCode::ErrNotASeller,
        "lots[0].seller_account_id"
    ));
    assert!(has(
        &calc.errors,
        ErrorCode::ErrNotABuyer,
        "lots[0].sales[0].buyer_account_id"
    ));
    let mut missing_seller = lot(RAHIM, BANANA, "50");
    missing_seller.seller_account_id = None;
    let mut no_buyer = sale(AKRAM, "20", "600");
    no_buyer.buyer_account_id = None;
    let calc = run(&draft(vec![with(missing_seller, vec![no_buyer])]));
    assert!(has(
        &calc.errors,
        ErrorCode::ErrSellerRequired,
        "lots[0].seller_account_id"
    ));
    assert!(has(
        &calc.errors,
        ErrorCode::ErrBuyerRequired,
        "lots[0].sales[0].buyer_account_id"
    ));
    let mut inactive = lookups();
    if let Some(a) = inactive.accounts.get_mut(&AKRAM) {
        a.is_active = false;
    }
    let calc = run_with(&event1(), &inactive, default_rules(), switches());
    assert!(has(
        &calc.errors,
        ErrorCode::ErrAccountInactive,
        "lots[0].sales[0].buyer_account_id"
    ));
    let calc = run(&draft(vec![with(
        lot(99, BANANA, "50"),
        vec![sale(AKRAM, "20", "600")],
    )]));
    assert!(has(
        &calc.errors,
        ErrorCode::ErrNotFound,
        "lots[0].seller_account_id"
    ));
}

fn pukhta_seller_lot(cash: Option<&str>, phone: &str) -> mandi_core::bikri::LotInput {
    let mut l = lot(PUKHTA_SELLER, APPLE, "10");
    l.freight = Some("1,000".into());
    l.cash_paid = cash.map(str::to_owned);
    l.pukhta = Some(PukhtaInput {
        name_ur: "ناصر".into(),
        name_en: "Nasir".into(),
        phone: phone.into(),
    });
    l
}

#[test]
fn a_one_time_seller_is_paid_his_whole_net_on_the_spot() {
    // PRD FR-13.10: gross 12,000, deductions before نقد 1,840: نقد starts at 10,160.
    let calc = run(&draft(vec![with(
        pukhta_seller_lot(None, "0300-0000000"),
        vec![sale(AKRAM, "10", "1,200")],
    )]));
    assert_no_errors(&calc);
    let t = single_lot_totals(&calc);
    assert_eq!((r(t.cash_paid), r(t.net)), (10_160, 0));
    assert_eq!(t.workings.cash_paid.message_id, "working-cash-pukhta");
    assert!(calc.warnings.is_empty());
    // He is paid what the screen shows; a typed نقد is kept as typed.
    let calc = run(&draft(vec![with(
        pukhta_seller_lot(Some("4,000"), "0300-0000000"),
        vec![sale(AKRAM, "10", "1,200")],
    )]));
    let t = single_lot_totals(&calc);
    assert_eq!((r(t.cash_paid), r(t.net)), (4_000, 6_160));
    // His phone is required.
    let calc = run(&draft(vec![with(
        pukhta_seller_lot(None, ""),
        vec![sale(AKRAM, "10", "1,200")],
    )]));
    assert!(has(
        &calc.errors,
        ErrorCode::ErrPukhtaPhoneRequired,
        "lots[0].pukhta.phone"
    ));
}

#[test]
fn the_person_key_is_on_every_journal_line_of_a_one_time_person() {
    // PRD FR-13.5, money agent point 1: the journal of the preview and of the save carries the
    // key of the one-time buyer on his debit and his cash credit, and the key of a one-time
    // seller on his credit and his نقد debit. An ordinary party has no key.
    let mut shahid = sale(PUKHTA_BUYER, "10", "1,000");
    shahid.pukhta = Some(PukhtaInput {
        name_ur: "شاہد".into(),
        name_en: "Shahid".into(),
        phone: "0300-0000000".into(),
    });
    let calc = run(&draft(vec![
        with(lot(RAHIM, BANANA, "10"), vec![shahid]),
        with(
            pukhta_seller_lot(None, "0300-1111111"),
            vec![sale(AKRAM, "10", "1,200")],
        ),
    ]));
    assert_no_errors(&calc);
    let first = &calc.journal.lots[0];
    assert_eq!(first.person_key, None, "an ordinary seller has no key");
    assert_eq!(
        first.sales[0].person_key.as_deref(),
        Some("شاہد|03000000000")
    );
    let second = &calc.journal.lots[1];
    assert_eq!(second.person_key.as_deref(), Some("ناصر|03001111111"));
    assert_eq!(
        second.sales[0].person_key, None,
        "an ordinary buyer has no key"
    );
    // In the built entry: shahid's debit and cash credit, nasir's credit and نقد debit.
    let entry = entry_of(&calc);
    let keyed = |account: i64, side: crate::ledger::Side| -> Vec<Option<String>> {
        entry
            .lines()
            .iter()
            .filter(|l| l.account_id == AccountId::new(account) && l.side == side)
            .map(|l| l.person_key.clone())
            .collect()
    };
    use crate::ledger::Side::{Cr, Dr};
    assert_eq!(
        keyed(PUKHTA_BUYER, Dr),
        [Some("شاہد|03000000000".to_owned())]
    );
    assert_eq!(
        keyed(PUKHTA_BUYER, Cr),
        [Some("شاہد|03000000000".to_owned())]
    );
    assert_eq!(
        keyed(PUKHTA_SELLER, Cr),
        [Some("ناصر|03001111111".to_owned())]
    );
    assert_eq!(
        keyed(PUKHTA_SELLER, Dr),
        [Some("ناصر|03001111111".to_owned())]
    );
}

#[test]
fn the_same_draft_gives_the_same_answer() {
    let draft = event1();
    assert_eq!(run(&draft), run(&draft));
}

#[test]
fn an_incomplete_lot_works_out_no_rows() {
    // No qty received yet: the rows wait, and the error says why.
    let mut incomplete = lot(RAHIM, BANANA, "");
    incomplete.sales = vec![sale(AKRAM, "20", "600")];
    let calc = run(&draft(vec![incomplete]));
    assert!(has(
        &calc.errors,
        ErrorCode::ErrRequired,
        "lots[0].qty_received"
    ));
    assert!(calc.lots[0].lines[0].calc.is_none());
    assert!(calc.lots[0].totals.is_none());
    assert!(calc.journal.lots.is_empty());
    assert_eq!(calc.summary.gross, Paisa::ZERO);
}
