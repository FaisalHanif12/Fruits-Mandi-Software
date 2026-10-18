//! The per-lot seller commission moves when its line is removed (PRD FR-11.9, D4-A31, FR-12.20).
//!
//! A rule "500 per lot" on کیلا charges 500 once, on the first line of the lot's first posting.
//! An edit that removes that line moves the charge, with its STORED method and value, to the first
//! line left, so the lot is still charged exactly once.

use mandi_services::bikri;
use mandi_services::core::rules::{ChargeMethod, ChargeSide, ChargeType, RuleInput, RuleSaved};
use mandi_services::rules;

use crate::helpers::*;
use crate::support::*;
use crate::tools;

fn item_rule(
    w: &World,
    side: ChargeSide,
    charge: ChargeType,
    method: ChargeMethod,
    value: &str,
) -> RuleSaved {
    rules::save(
        &w.env.ctx,
        &w.owner,
        &RuleInput {
            id: None,
            version: None,
            side,
            charge_type: charge,
            account_id: None,
            item_id: Some(w.banana),
            item_group: None,
            method,
            value: value.into(),
            from_date: "01-01-2026".into(),
            to_date: String::new(),
            is_active: true,
            note: String::new(),
        },
    )
    .unwrap()
}

fn per_lot_rule(w: &World, value: &str) -> RuleSaved {
    item_rule(
        w,
        ChargeSide::Seller,
        ChargeType::Commission,
        ChargeMethod::PerLot,
        value,
    )
}

/// The seller commission of each line in force, as (rupees, method, stored value in rupees).
fn comms(w: &World) -> Vec<(i64, String, i64)> {
    let conn = crate::common::raw(&w.dir);
    let mut statement = conn
        .prepare(
            "SELECT seller_comm_paisa, seller_comm_method, COALESCE(seller_comm_value_paisa, 0)
             FROM bikri_sales WHERE is_active = 1 ORDER BY lot_id, line_no, id",
        )
        .unwrap();
    statement
        .query_map([], |r| {
            Ok((
                r.get::<_, i64>(0)? / 100,
                r.get::<_, String>(1)?,
                r.get::<_, i64>(2)? / 100,
            ))
        })
        .unwrap()
        .map(Result::unwrap)
        .collect()
}

fn lot_comm(w: &World, bikri_id: i64) -> i64 {
    bikri::get(&w.env.ctx, bikri_id).unwrap().lots[0]
        .seller_comm
        .rupees()
}

/// A lot of 50 کیلا with two lines, 20 at 600 and 30 at 580.
fn two_lines(w: &World) -> mandi_services::core::bikri::BikriSaved {
    save_ok(w, &event1(w))
}

#[test]
fn the_per_lot_commission_sits_on_line_one_and_moves_when_that_line_is_removed() {
    let w = world();
    per_lot_rule(&w, "500");
    let saved = two_lines(&w);
    assert_eq!(
        comms(&w),
        [(500, "per_lot".into(), 500), (0, "per_lot".into(), 500)]
    );
    assert_eq!(lot_comm(&w, saved.bikri_id), 500);
    // Remove line 1: line 2 now carries the 500, once.
    let opened = tools::open(&w, saved.bikri_id, saved.entry_id);
    let mut d = opened.draft.clone();
    d.lots[0].sales.remove(0);
    let edited = tools::edit(&w, &opened, &d);
    assert_eq!(comms(&w), [(500, "per_lot".into(), 500)]);
    assert_eq!(lot_comm(&w, saved.bikri_id), 500);
    assert_eq!(edited.preview.summary.seller_comm.rupees(), 500);
    // The removed line ended as `removed`, not `replaced`.
    assert_eq!(
        scalar(
            &w,
            "SELECT COUNT(*) FROM bikri_sales WHERE ended = 'removed'"
        ),
        1
    );
    assert_eq!(tools::trial_difference(&w), 0);
}

#[test]
fn a_rule_changed_later_does_not_change_the_charge_that_moves() {
    let w = world();
    let rule = per_lot_rule(&w, "500");
    let saved = two_lines(&w);
    // The rule is now 600 for new lines.
    rules::save(
        &w.env.ctx,
        &w.owner,
        &RuleInput {
            id: Some(rule.row.id),
            version: Some(rule.row.version),
            side: ChargeSide::Seller,
            charge_type: ChargeType::Commission,
            account_id: None,
            item_id: Some(w.banana),
            item_group: None,
            method: ChargeMethod::PerLot,
            value: "600".into(),
            from_date: "01-01-2026".into(),
            to_date: String::new(),
            is_active: true,
            note: String::new(),
        },
    )
    .unwrap();
    let opened = tools::open(&w, saved.bikri_id, saved.entry_id);
    let mut d = opened.draft.clone();
    d.lots[0].sales.remove(0);
    tools::edit(&w, &opened, &d);
    // The charge that moved is the stored 500, not the new 600 (FR-11.9, FR-12.20).
    assert_eq!(comms(&w), [(500, "per_lot".into(), 500)]);
    assert_eq!(lot_comm(&w, saved.bikri_id), 500);
}

#[test]
fn editing_only_line_one_looks_its_rules_up_again_and_line_two_stays() {
    let w = world();
    let rule = per_lot_rule(&w, "500");
    let saved = two_lines(&w);
    // Rule still 500: line 1 changed → looked up again → 500; line 2 unchanged → 0.
    let opened = tools::open(&w, saved.bikri_id, saved.entry_id);
    let mut d = opened.draft.clone();
    d.lots[0].sales[0].seller_rate = "610".into();
    let first = tools::edit(&w, &opened, &d);
    assert_eq!(
        comms(&w),
        [(500, "per_lot".into(), 500), (0, "per_lot".into(), 500)]
    );
    // The rule is switched off, so the default 6% applies to a changed line. Line 1 changes again:
    // 20 × 620 = 12,400 × 6% = 744. Line 2 stays 0 (unchanged lines keep their stored values).
    rules::save(
        &w.env.ctx,
        &w.owner,
        &RuleInput {
            id: Some(rule.row.id),
            version: Some(rule.row.version),
            side: ChargeSide::Seller,
            charge_type: ChargeType::Commission,
            account_id: None,
            item_id: Some(w.banana),
            item_group: None,
            method: ChargeMethod::PerLot,
            value: "500".into(),
            from_date: "01-01-2026".into(),
            to_date: String::new(),
            is_active: false,
            note: String::new(),
        },
    )
    .unwrap();
    let again = tools::open(&w, saved.bikri_id, first.entry_id);
    let mut d = again.draft.clone();
    d.lots[0].sales[0].seller_rate = "620".into();
    tools::edit(&w, &again, &d);
    assert_eq!(
        comms(&w),
        [(744, "percent".into(), 0), (0, "per_lot".into(), 500)]
    );
    assert_eq!(lot_comm(&w, saved.bikri_id), 744);
    // The lot now holds a per-lot charge on line 2 that is not the first line: it never charges
    // twice, because its amount is stored as 0.
    assert_eq!(tools::trial_difference(&w), 0);
}

#[test]
fn a_typed_commission_on_the_remaining_line_is_not_given_the_moved_charge() {
    // Line 2 has a typed seller commission: the moved charge would be a second one, so it goes
    // nowhere and the lot keeps only what the user typed (PRD FR-11.9: the first line left that
    // has no typed seller commission).
    let w = world();
    per_lot_rule(&w, "500");
    let saved = two_lines(&w);
    let opened = tools::open(&w, saved.bikri_id, saved.entry_id);
    let mut d = opened.draft.clone();
    d.lots[0].sales.remove(0);
    d.lots[0].sales[0].seller_comm =
        typed(mandi_services::core::bikri::LineChargeMethod::Percent, "4");
    let edited = tools::edit(&w, &opened, &d);
    // 30 × 580 = 17,400 × 4% = 696; no 500 on top.
    assert_eq!(edited.preview.summary.seller_comm.rupees(), 696);
    let left = comms(&w);
    assert_eq!(left.len(), 1);
    assert_eq!(left[0].0, 696);
}

/// Switches a per-lot rule off, so a changed line gets the default 6% again.
fn switch_rule_off(w: &World, rule: &RuleSaved, value: &str) {
    rules::save(
        &w.env.ctx,
        &w.owner,
        &RuleInput {
            id: Some(rule.row.id),
            version: Some(rule.row.version),
            side: ChargeSide::Seller,
            charge_type: ChargeType::Commission,
            account_id: None,
            item_id: Some(w.banana),
            item_group: None,
            method: ChargeMethod::PerLot,
            value: value.into(),
            from_date: "01-01-2026".into(),
            to_date: String::new(),
            is_active: false,
            note: String::new(),
        },
    )
    .unwrap();
}

#[test]
fn an_unchanged_line_with_its_own_percent_commission_does_not_take_the_moved_charge() {
    // Money agent review of plan 16, finding 1. The rule "500 per lot" is on; line 1 (اکرم)
    // carries 500 and line 2 (ارشد) is saved with the per-lot method and 0. The rule is switched
    // off and ارشد's rate is edited, so his changed line looks the rules up again: 17,700 × 6% =
    // 1,062 (percent). Then اکرم's line is removed. ارشد's unchanged line has a commission of its
    // own, so it must keep 1,062 and not be overwritten by the 500 (which goes with the removed
    // line).
    let w = world();
    let rule = per_lot_rule(&w, "500");
    let saved = two_lines(&w);
    switch_rule_off(&w, &rule, "500");
    let opened = tools::open(&w, saved.bikri_id, saved.entry_id);
    let mut d = opened.draft.clone();
    d.lots[0].sales[1].seller_rate = "590".into();
    let first = tools::edit(&w, &opened, &d);
    assert_eq!(
        comms(&w),
        [(500, "per_lot".into(), 500), (1_062, "percent".into(), 0)]
    );
    assert_eq!(lot_comm(&w, saved.bikri_id), 1_562);
    let again = tools::open(&w, saved.bikri_id, first.entry_id);
    let mut d = again.draft.clone();
    d.lots[0].sales.remove(0);
    let edited = tools::edit(&w, &again, &d);
    assert_eq!(comms(&w), [(1_062, "percent".into(), 0)]);
    assert_eq!(lot_comm(&w, saved.bikri_id), 1_062);
    assert_eq!(edited.preview.summary.seller_comm.rupees(), 1_062);
    assert_eq!(tools::trial_difference(&w), 0);
}

#[test]
fn the_moved_charge_is_spent_once_when_an_earlier_changed_line_already_carries_the_rule() {
    // Three lines: اکرم carries 500, ارشد and a third line carry 0. The rule is still on. اکرم's
    // line is removed and ارشد's line is changed, so it looks the rule up again and carries 500
    // (it is the first line the per-lot rule reaches). The third line is unchanged and saved with
    // the per-lot method: it must NOT take the moved charge as well.
    let w = world();
    per_lot_rule(&w, "500");
    let mut draft = event1(&w);
    draft.lots[0].sales = vec![
        sale("s-1", w.akram, "20", "600"),
        sale("s-2", w.arshad, "20", "580"),
        sale("s-3", w.akram, "10", "590"),
    ];
    let saved = save_ok(&w, &draft);
    assert_eq!(
        comms(&w),
        [
            (500, "per_lot".into(), 500),
            (0, "per_lot".into(), 500),
            (0, "per_lot".into(), 500)
        ]
    );
    let opened = tools::open(&w, saved.bikri_id, saved.entry_id);
    let mut d = opened.draft.clone();
    d.lots[0].sales.remove(0);
    d.lots[0].sales[0].seller_rate = "581".into();
    tools::edit(&w, &opened, &d);
    assert_eq!(lot_comm(&w, saved.bikri_id), 500);
    let carried: Vec<i64> = comms(&w).iter().map(|c| c.0).collect();
    assert_eq!(carried, [500, 0]);
    assert_eq!(tools::trial_difference(&w), 0);
}
