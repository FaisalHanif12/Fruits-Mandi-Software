//! The worked numbers of PRD 4.4, 4.5 and E2E-16, through `calc_posting` from the typed text.

use mandi_core::bikri::{
    ChargeSource, LineChargeInput, LineChargeMethod, PaymentType, PukhtaInput, StoredMethod,
};
use mandi_core::error::ErrorCode;
use mandi_core::money::Paisa;
use mandi_core::rules::RuleLevel;

use super::*;
use crate::cash;

#[test]
fn event_1_lines_lot_summary_and_entry() {
    // PRD 4.4: Akram 20 x 600, Arshad 30 x 580 on Rahim's 50 banana.
    let calc = run(&event1());
    assert_no_errors(&calc);
    let a = line(&calc, 0, 0);
    assert_eq!(
        (
            r(a.seller_amount),
            r(a.seller_comm.amount),
            r(a.buyer_value),
            r(a.buyer_comm.amount),
            r(a.laga.amount),
            r(a.buyer_total),
            r(a.rate_diff)
        ),
        (12_000, 720, 12_000, 720, 160, 12_880, 0)
    );
    let b = line(&calc, 0, 1);
    assert_eq!(
        (
            r(b.seller_amount),
            r(b.seller_comm.amount),
            r(b.buyer_comm.amount),
            r(b.laga.amount),
            r(b.buyer_total)
        ),
        (17_400, 1_044, 1_044, 240, 18_684)
    );
    // The charges say where they came from: the 6% defaults and the fruit لاگا.
    assert_eq!(a.seller_comm.source, ChargeSource::Default);
    assert_eq!(a.seller_comm.method, StoredMethod::Percent);
    assert_eq!(a.seller_comm.value_bps, Some(600));
    assert_eq!(a.laga.method, StoredMethod::PerUnit);
    assert_eq!(a.laga.value_paisa, Some(p(8)));
    assert_eq!(a.payment_type, PaymentType::Credit);
    // E2E-26: the working of line 1 seller commission is 12,000 x 6 / 100 = 720.
    assert_eq!(a.seller_comm.working.message_id, "working-percent");

    let t = single_lot_totals(&calc);
    assert_eq!(r(t.gross), 29_400);
    assert_eq!(r(t.freight), 3_000);
    assert_eq!(r(t.seller_comm), 1_764);
    assert_eq!(r(t.labour), 250);
    assert_eq!(r(t.munshiana), 50);
    assert_eq!(r(t.cash_paid), 5_000);
    assert_eq!(r(t.phone), 20);
    assert_eq!(r(t.deductions), 10_084);
    assert_eq!(r(t.net), 19_316);
    assert_eq!(r(t.seller_credit), 24_316);
    assert_eq!(r(t.buyer_comm), 1_764);
    assert_eq!(r(t.laga), 400);
    assert_eq!(r(t.buyer_credit), 31_564);
    assert_eq!(r(t.buyer_cash), 0);
    assert_eq!(r(t.shared_income), 3_948);
    // The whole بکری has one lot, so its summary is the lot's.
    assert_eq!(r(calc.summary.deductions), 10_084);
    assert_eq!(r(calc.summary.net), 19_316);
    assert_eq!(r(calc.summary.shared_income), 3_948);

    // The lot, 50 received and 50 sold.
    assert_eq!(calc.lots[0].sold_qty, 50);
    assert_eq!(calc.lots[0].balance_qty, Some(0));
    assert!(!calc.lots[0].net_is_negative);
    assert!(calc.warnings.is_empty(), "{:?}", calc.warnings);

    // The journal: sale part 31,564, whole entry 36,564; cash falls by 5,000.
    let entry = entry_of(&calc);
    assert_eq!(entry.total(), p(36_564));
    assert_eq!(
        cash::net_change(&entry, system_accounts().cash).unwrap(),
        p(-5_000)
    );
    // The formula the screen used to have agrees with the entry (money agent point 6).
    assert_eq!(t.buyer_cash.checked_sub(t.cash_paid).unwrap(), p(-5_000));
    assert_every_working_matches(&calc);
}

#[test]
fn event_1_lot_charges_say_where_they_came_from() {
    let calc = run(&event1());
    let lot = lot_of(&calc, 0);
    let labour = lot.labour.as_ref().unwrap();
    // Typed per نگ: 50 received x 5 = 250.
    assert_eq!((r(labour.amount), labour.value_paisa), (250, p(5)));
    assert_eq!(labour.source, ChargeSource::Typed);
    assert_eq!(labour.working.message_id, "working-manual-per-unit");
    assert!(
        labour.working.args.iter().any(|a| a.name == "qty_received"),
        "a lot charge counts the نگ received"
    );
    assert_eq!(r(lot.munshiana.as_ref().unwrap().amount), 50);
    assert_eq!(r(lot.phone.as_ref().unwrap().amount), 20);
}

#[test]
fn event_2_one_time_buyer_pays_cash() {
    // PRD 4.5: Karim's 10 apples; Shahid (no account) buys all at 1,200 and pays cash.
    let mut karim = lot(KARIM, APPLE, "10");
    karim.freight = Some("1,000".into());
    karim.cash_paid = None;
    let mut shahid = sale(PUKHTA_BUYER, "10", "1,200");
    shahid.pukhta = Some(PukhtaInput {
        name_ur: "شاہد".into(),
        name_en: "Shahid".into(),
        phone: "0300-0000000".into(),
    });
    let calc = run(&draft(vec![with(karim, vec![shahid])]));
    assert_no_errors(&calc);
    let l = line(&calc, 0, 0);
    assert_eq!(
        (
            r(l.seller_amount),
            r(l.seller_comm.amount),
            r(l.buyer_comm.amount),
            r(l.laga.amount),
            r(l.buyer_total)
        ),
        (12_000, 720, 720, 80, 12_800)
    );
    assert_eq!(
        l.payment_type,
        PaymentType::Cash,
        "a one-time buyer pays cash"
    );
    let t = single_lot_totals(&calc);
    assert_eq!(r(t.deductions), 1_840);
    assert_eq!(r(t.net), 10_160);
    assert_eq!(r(t.seller_credit), 10_160);
    assert_eq!(r(t.shared_income), 1_540);
    assert_eq!((r(t.buyer_cash), r(t.buyer_credit)), (12_800, 0));
    let entry = entry_of(&calc);
    assert_eq!(entry.total(), p(25_600));
    assert_eq!(
        cash::net_change(&entry, system_accounts().cash).unwrap(),
        p(12_800)
    );
    assert_every_working_matches(&calc);
}

#[test]
fn event_1_with_only_45_sold_e2e_16() {
    // E2E-16: Akram 20 x 600, Arshad 25 x 580. Labour stays 250 on the 50 received.
    let calc = run(&draft(vec![with(
        lot(RAHIM, BANANA, "50"),
        vec![sale(AKRAM, "20", "600"), sale(ARSHAD, "25", "580")],
    )]));
    assert_no_errors(&calc);
    let t = single_lot_totals(&calc);
    assert_eq!(r(t.gross), 26_500);
    assert_eq!(r(t.seller_comm), 1_590);
    assert_eq!(r(t.labour), 250);
    assert_eq!(r(t.deductions), 9_910);
    assert_eq!(r(t.net), 16_590);
    assert_eq!(r(t.buyer_comm), 1_590);
    assert_eq!(r(t.laga), 360);
    assert_eq!(r(t.shared_income), 3_560);
    assert_eq!(calc.lots[0].sold_qty, 45);
    assert_eq!(calc.lots[0].balance_qty, Some(5));
    // "5 نگ باقی ہیں": a warning, not an error.
    let left = calc
        .warnings
        .iter()
        .find(|w| w.code == ErrorCode::ErrQtyLeftUnsold)
        .unwrap();
    assert_eq!(left.field, "lots[0].qty_received");
    assert_every_working_matches(&calc);
}

#[test]
fn a_percent_typed_on_the_line_is_saved_as_manual_e2e_28_note() {
    // FR-11.10: 4% typed for the seller on line 1: 12,000 x 4 / 100 = 480, method manual.
    let mut first = sale(AKRAM, "20", "600");
    first.seller_comm = Some(LineChargeInput {
        method: LineChargeMethod::Percent,
        value: "4".into(),
    });
    let calc = run(&draft(vec![with(
        lot(RAHIM, BANANA, "50"),
        vec![first, sale(ARSHAD, "30", "580")],
    )]));
    assert_no_errors(&calc);
    let c = &line(&calc, 0, 0).seller_comm;
    assert_eq!(r(c.amount), 480);
    assert_eq!(c.method, StoredMethod::Manual);
    assert_eq!(c.value_bps, Some(400));
    assert_eq!(c.source, ChargeSource::Typed);
    assert_eq!(c.rule_id, None);
    assert_eq!(c.rule_label.unwrap().level, RuleLevel::LineOverride);
    assert_eq!(c.working.message_id, "working-manual-percent");
    // The other line still follows the rule; the lot commission is the sum of the stored lines.
    assert_eq!(r(line(&calc, 0, 1).seller_comm.amount), 1_044);
    assert_eq!(r(single_lot_totals(&calc).seller_comm), 1_524);
    assert_every_working_matches(&calc);
}

#[test]
fn a_typed_commission_is_refused_when_the_owner_switched_typing_off() {
    let mut first = sale(AKRAM, "20", "600");
    first.seller_comm = Some(LineChargeInput {
        method: LineChargeMethod::Percent,
        value: "4".into(),
    });
    first.laga = Some(LineChargeInput {
        method: LineChargeMethod::PerUnit,
        value: "3".into(),
    });
    let mut off = switches();
    off.allow_line_override = false;
    let calc = run_with(
        &draft(vec![with(lot(RAHIM, BANANA, "50"), vec![first])]),
        &lookups(),
        default_rules(),
        off,
    );
    assert!(has(
        &calc.errors,
        ErrorCode::ErrOverrideNotAllowed,
        "lots[0].sales[0].seller_comm"
    ));
    assert!(has(
        &calc.errors,
        ErrorCode::ErrOverrideNotAllowed,
        "lots[0].sales[0].laga"
    ));
    // The rule decides the number meanwhile.
    let l = line(&calc, 0, 0);
    assert_eq!((r(l.seller_comm.amount), r(l.laga.amount)), (720, 160));
}

#[test]
fn lot_charges_typed_are_allowed_even_when_line_typing_is_off() {
    // Money agent point 3: only commission and لاگا on a line are limited by the switch.
    let mut off = switches();
    off.allow_line_override = false;
    let calc = run_with(&event1(), &lookups(), default_rules(), off);
    assert_no_errors(&calc);
    assert_eq!(r(single_lot_totals(&calc).labour), 250);
}

#[test]
fn money_is_always_a_whole_rupee() {
    let calc = run(&event1());
    for lot in &calc.lots {
        for line in lot.lines.iter().filter_map(|l| l.calc.as_ref()) {
            for amount in [
                line.seller_amount,
                line.buyer_value,
                line.seller_comm.amount,
                line.buyer_comm.amount,
                line.laga.amount,
                line.buyer_total,
                line.rate_diff,
            ] {
                assert_eq!(amount.as_paisa() % 100, 0);
                assert!(Paisa::try_from_paisa(amount.as_paisa()).is_ok());
            }
        }
    }
}
