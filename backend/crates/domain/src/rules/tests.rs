//! Tests for rule resolution: the worked numbers of PRD 4.2, 4.4 and 11.3, each level winning
//! over every level below it, the newer-rule tie, the dates, and a model-based property test.

use mandi_core::dates::BusinessDate;
use mandi_core::money::{Bps, Paisa, Qty};
use mandi_core::rules::{
    ChargeSide, ChargeType, ChargeValue, ItemGroup, RuleLevel, SkipReason, WorkingValue,
};
use proptest::prelude::*;

use super::*;

const RAHIM: i64 = 10;
const KARIM: i64 = 11;
const AKRAM: i64 = 20;
const BANANA: i64 = 1;
const APPLE: i64 = 2;
const JAPANI: i64 = 3;
const MOOLI: i64 = 4;

fn day(text: &str) -> BusinessDate {
    BusinessDate::parse_iso(text).unwrap()
}

fn rupees(value: i64) -> Paisa {
    Paisa::from_rupees(value).unwrap()
}

fn percent(bps: i32) -> ChargeValue {
    ChargeValue::Percent(Bps::new(bps).unwrap())
}

fn per_unit_value(value: i64) -> ChargeValue {
    ChargeValue::PerUnit(rupees(value))
}

#[allow(clippy::too_many_arguments)]
fn rule(
    id: i64,
    side: ChargeSide,
    charge: ChargeType,
    account: Option<i64>,
    item: Option<i64>,
    group: Option<ItemGroup>,
    value: ChargeValue,
) -> Rule {
    Rule {
        id,
        side,
        charge,
        account_id: account,
        item_id: item,
        item_group: group,
        value,
        from_date: day("2000-01-01"),
        to_date: None,
        is_active: true,
    }
}

/// The seeded defaults of PRD 4.2.
fn defaults() -> Vec<Rule> {
    vec![
        rule(
            1,
            ChargeSide::Seller,
            ChargeType::Commission,
            None,
            None,
            None,
            percent(600),
        ),
        rule(
            2,
            ChargeSide::Buyer,
            ChargeType::Commission,
            None,
            None,
            None,
            percent(600),
        ),
        rule(
            3,
            ChargeSide::Buyer,
            ChargeType::Laga,
            None,
            None,
            Some(ItemGroup::Fruit),
            per_unit_value(8),
        ),
    ]
}

/// The amount of a commission charge for a winner, the per-lot amount due.
fn charged(
    winner: Option<&Winner>,
    base: Paisa,
    qty: Qty,
) -> Result<Charged, mandi_core::money::MoneyError> {
    charge_amount(winner, ChargeType::Commission, base, qty, true)
}

fn group_of(item: i64) -> ItemGroup {
    if item == MOOLI {
        ItemGroup::Vegetable
    } else {
        ItemGroup::Fruit
    }
}

fn ask(set: &RuleSet, side: ChargeSide, charge: ChargeType, party: i64, item: i64) -> Resolution {
    resolve(
        set,
        &Query {
            side,
            charge,
            party_id: Some(party),
            item_id: item,
            item_group: Some(group_of(item)),
            date: day("2026-10-10"),
            line_override: None,
        },
    )
}

/// The amount of a charge on 20 نگ at 600 = 12,000.
fn amount(set: &RuleSet, side: ChargeSide, charge: ChargeType, party: i64, item: i64) -> i64 {
    let found = ask(set, side, charge, party, item);
    charge_amount(
        found.winner.as_ref(),
        charge,
        rupees(12_000),
        Qty::new(20).unwrap(),
        true,
    )
    .unwrap()
    .amount
    .rupees()
}

#[test]
fn the_seeded_defaults_give_the_prd_4_4_numbers() {
    // Plan 09 Section 3, rows 1 and 2: 20 × 600 = 12,000.
    let set = RuleSet::build(defaults());
    let (s, b, l) = (
        (ChargeSide::Seller, ChargeType::Commission),
        (ChargeSide::Buyer, ChargeType::Commission),
        (ChargeSide::Buyer, ChargeType::Laga),
    );
    assert_eq!(amount(&set, s.0, s.1, RAHIM, BANANA), 720);
    assert_eq!(amount(&set, b.0, b.1, AKRAM, BANANA), 720);
    assert_eq!(amount(&set, l.0, l.1, AKRAM, BANANA), 160);
    // Vegetables: commission as before, no لاگا (no rule, so 0).
    assert_eq!(amount(&set, s.0, s.1, RAHIM, MOOLI), 720);
    assert_eq!(amount(&set, b.0, b.1, AKRAM, MOOLI), 720);
    assert_eq!(amount(&set, l.0, l.1, AKRAM, MOOLI), 0);
    let none = ask(&set, l.0, l.1, AKRAM, MOOLI);
    assert!(none.winner.is_none());
    assert_eq!(none.skipped.len(), 6, "all six levels were looked at");
    // The fruit لاگا is decided at level 5 (the group), after four levels were skipped.
    let fruit = ask(&set, l.0, l.1, AKRAM, BANANA);
    assert_eq!(
        fruit.winner.map(|w| (w.level, w.rule_id)),
        Some((RuleLevel::ItemGroup, Some(3)))
    );
    assert_eq!(fruit.skipped.len(), 4);
    assert_eq!(fruit.skipped[0].reason, SkipReason::NoOverride);
    assert!(
        fruit.skipped[1..]
            .iter()
            .all(|s| s.reason == SkipReason::NoRule)
    );
    // No default for the lot charges: they resolve to nothing (typed on the lot).
    for charge in [ChargeType::Labour, ChargeType::Munshiana, ChargeType::Phone] {
        assert!(
            ask(&set, ChargeSide::Seller, charge, RAHIM, BANANA)
                .winner
                .is_none()
        );
    }
}

#[test]
fn a_buyer_rule_beats_the_default() {
    // Plan 09 Section 3, row 3: "اکرم buys at 7%" gives 12,000 × 7 ÷ 100 = 840.
    let mut rules = defaults();
    rules.push(rule(
        4,
        ChargeSide::Buyer,
        ChargeType::Commission,
        Some(AKRAM),
        None,
        None,
        percent(700),
    ));
    let set = RuleSet::build(rules);
    assert_eq!(
        amount(
            &set,
            ChargeSide::Buyer,
            ChargeType::Commission,
            AKRAM,
            BANANA
        ),
        840
    );
    // Another buyer still pays 6%.
    assert_eq!(
        amount(&set, ChargeSide::Buyer, ChargeType::Commission, 21, BANANA),
        720
    );
}

#[test]
fn the_prd_11_3_examples() {
    // PRD 11.3 worked example, each on 20 نگ × 600 = 12,000.
    let mut rules = defaults();
    let seller = (ChargeSide::Seller, ChargeType::Commission);
    rules.push(rule(
        10,
        seller.0,
        seller.1,
        Some(RAHIM),
        Some(BANANA),
        None,
        percent(500),
    ));
    rules.push(rule(
        11,
        seller.0,
        seller.1,
        Some(RAHIM),
        None,
        None,
        percent(400),
    ));
    rules.push(rule(
        12,
        seller.0,
        seller.1,
        None,
        Some(JAPANI),
        None,
        per_unit_value(30),
    ));
    let set = RuleSet::build(rules);
    let by = |party, item| {
        let found = ask(&set, seller.0, seller.1, party, item);
        let charged = charge_amount(
            found.winner.as_ref(),
            seller.1,
            rupees(12_000),
            Qty::new(20).unwrap(),
            true,
        )
        .unwrap();
        (found.winner.map(|w| w.level), charged.amount.rupees())
    };
    // رحیم sells کیلا: party + item 5% = 600.
    assert_eq!(by(RAHIM, BANANA), (Some(RuleLevel::PartyItem), 600));
    // رحیم sells سیب: party 4% = 480.
    assert_eq!(by(RAHIM, APPLE), (Some(RuleLevel::Party), 480));
    // کریم sells جاپانی پھل: item, 30 per نگ × 20 = 600.
    assert_eq!(by(KARIM, JAPANI), (Some(RuleLevel::Item), 600));
    // کریم sells کیلا: default 6% = 720.
    assert_eq!(by(KARIM, BANANA), (Some(RuleLevel::Default), 720));
}

#[test]
fn a_value_typed_on_the_line_wins_and_is_manual() {
    // Plan 09 Section 3: line override 4% on 12,000 = 480, level 1, manual.
    let set = RuleSet::build(defaults());
    let query = Query {
        side: ChargeSide::Buyer,
        charge: ChargeType::Commission,
        party_id: Some(AKRAM),
        item_id: BANANA,
        item_group: Some(ItemGroup::Fruit),
        date: day("2026-10-10"),
        line_override: Some(LineOverride {
            value: percent(400),
        }),
    };
    let found = resolve(&set, &query);
    let winner = found.winner.unwrap();
    assert_eq!(
        (winner.level, winner.rule_id, winner.manual),
        (RuleLevel::LineOverride, None, true)
    );
    assert!(found.skipped.is_empty());
    let charged = charged(Some(&winner), rupees(12_000), Qty::new(20).unwrap()).unwrap();
    assert_eq!(charged.amount, rupees(480));
    assert_eq!(charged.working.message_id, "working-manual-percent");
    assert_eq!(charged.working.rule_id, None);
    // The override is for commission and لاگا only (PRD FR-11.10): a typed value on a lot
    // charge is ignored by the lookup.
    let mut labour = query;
    labour.side = ChargeSide::Seller;
    labour.charge = ChargeType::Labour;
    labour.line_override = Some(LineOverride {
        value: per_unit_value(5),
    });
    assert!(resolve(&set, &labour).winner.is_none());
}

#[test]
fn each_level_wins_over_every_level_below_it() {
    // Plan 09 Section 3: all six levels have a rule that matches; remove the winner one by one.
    let side = ChargeSide::Seller;
    let charge = ChargeType::Commission;
    let mut rules = vec![
        rule(1, side, charge, None, None, None, percent(100)), // level 6
        rule(
            2,
            side,
            charge,
            None,
            None,
            Some(ItemGroup::Fruit),
            percent(200),
        ), // 5
        rule(3, side, charge, None, Some(BANANA), None, percent(300)), // 4
        rule(4, side, charge, Some(RAHIM), None, None, percent(400)), // 3
        rule(
            5,
            side,
            charge,
            Some(RAHIM),
            Some(BANANA),
            None,
            percent(500),
        ), // 2
    ];
    let query = Query {
        side,
        charge,
        party_id: Some(RAHIM),
        item_id: BANANA,
        item_group: Some(ItemGroup::Fruit),
        date: day("2026-10-10"),
        line_override: Some(LineOverride {
            value: percent(600),
        }),
    };
    let mut want = [
        (RuleLevel::LineOverride, None),
        (RuleLevel::PartyItem, Some(5)),
        (RuleLevel::Party, Some(4)),
        (RuleLevel::Item, Some(3)),
        (RuleLevel::ItemGroup, Some(2)),
        (RuleLevel::Default, Some(1)),
    ]
    .into_iter();
    // Level 1 first.
    let got = resolve(&RuleSet::build(rules.clone()), &query)
        .winner
        .unwrap();
    assert_eq!((got.level, got.rule_id), want.next().unwrap());
    let mut query = query;
    query.line_override = None;
    // Then each rule level, taking the winner away each time.
    for (level, id) in want {
        let got = resolve(&RuleSet::build(rules.clone()), &query)
            .winner
            .unwrap();
        assert_eq!((got.level, got.rule_id), (level, id));
        // Every level above the winner is skipped, in order.
        let skipped: Vec<RuleLevel> = resolve(&RuleSet::build(rules.clone()), &query)
            .skipped
            .iter()
            .map(|s| s.level)
            .collect();
        let expected: Vec<RuleLevel> = RuleLevel::ALL
            .iter()
            .copied()
            .take_while(|l| *l != level)
            .collect();
        assert_eq!(skipped, expected);
        rules.retain(|r| Some(r.id) != id);
    }
    assert!(resolve(&RuleSet::build(rules), &query).winner.is_none());
}

#[test]
fn two_rules_at_one_level_the_newer_wins() {
    // PRD FR-11.5: overlapping rules at the same level, the one made last wins.
    let side = ChargeSide::Seller;
    let charge = ChargeType::Commission;
    let older = rule(7, side, charge, Some(RAHIM), None, None, percent(400));
    let newer = rule(9, side, charge, Some(RAHIM), None, None, percent(450));
    let set = RuleSet::build(vec![older.clone(), newer.clone()]);
    assert_eq!(amount(&set, side, charge, RAHIM, BANANA), 540); // 4.5%
    // The order they are given in does not matter.
    let set = RuleSet::build(vec![newer.clone(), older.clone()]);
    assert_eq!(amount(&set, side, charge, RAHIM, BANANA), 540);
    // If the newer one has not started yet, the older still decides.
    let mut later = newer;
    later.from_date = day("2026-11-01");
    let set = RuleSet::build(vec![older, later]);
    assert_eq!(amount(&set, side, charge, RAHIM, BANANA), 480);
}

#[test]
fn a_rule_applies_from_its_first_day_to_its_last_day_both_included() {
    let side = ChargeSide::Seller;
    let charge = ChargeType::Commission;
    let mut windowed = rule(5, side, charge, Some(RAHIM), None, None, percent(400));
    windowed.from_date = day("2026-10-01");
    windowed.to_date = Some(day("2026-10-31"));
    let mut rules = defaults();
    rules.push(windowed);
    let set = RuleSet::build(rules);
    let on = |d: &str| {
        let q = Query {
            side,
            charge,
            party_id: Some(RAHIM),
            item_id: BANANA,
            item_group: Some(ItemGroup::Fruit),
            date: day(d),
            line_override: None,
        };
        resolve(&set, &q).winner.map(|w| w.level)
    };
    assert_eq!(on("2026-09-30"), Some(RuleLevel::Default));
    assert_eq!(on("2026-10-01"), Some(RuleLevel::Party));
    assert_eq!(on("2026-10-31"), Some(RuleLevel::Party));
    assert_eq!(on("2026-11-01"), Some(RuleLevel::Default));
}

#[test]
fn an_inactive_rule_and_a_rule_of_another_side_never_decide() {
    let mut off = rule(
        5,
        ChargeSide::Seller,
        ChargeType::Commission,
        Some(RAHIM),
        None,
        None,
        percent(100),
    );
    off.is_active = false;
    // The buyer-side rule names the same account but is for the other side.
    let other_side = rule(
        6,
        ChargeSide::Buyer,
        ChargeType::Commission,
        Some(RAHIM),
        None,
        None,
        percent(900),
    );
    let mut rules = defaults();
    rules.extend([off, other_side]);
    let set = RuleSet::build(rules);
    assert_eq!(
        amount(
            &set,
            ChargeSide::Seller,
            ChargeType::Commission,
            RAHIM,
            BANANA
        ),
        720
    );
    // A rule with an item group and a party has no level and is never filed.
    let odd = rule(
        8,
        ChargeSide::Seller,
        ChargeType::Commission,
        Some(RAHIM),
        None,
        Some(ItemGroup::Fruit),
        percent(100),
    );
    assert_eq!(odd.level(), None);
    assert_eq!(RuleSet::build(vec![odd]).len(), 0);
}

#[test]
fn the_party_of_a_buyer_side_charge_is_the_buyer() {
    // Rahim is the seller and Akram the buyer: the caller passes the party of the side.
    let mut rules = defaults();
    rules.push(rule(
        4,
        ChargeSide::Buyer,
        ChargeType::Commission,
        Some(AKRAM),
        None,
        None,
        percent(700),
    ));
    let set = RuleSet::build(rules);
    // With the seller's id as the party the buyer rule is not found.
    assert_eq!(
        amount(
            &set,
            ChargeSide::Buyer,
            ChargeType::Commission,
            RAHIM,
            BANANA
        ),
        720
    );
    assert_eq!(
        amount(
            &set,
            ChargeSide::Buyer,
            ChargeType::Commission,
            AKRAM,
            BANANA
        ),
        840
    );
}

#[test]
fn amounts_follow_the_money_rules_and_the_working_is_the_amount() {
    let qty = Qty::new(20).unwrap();
    let winner = |value| Winner {
        level: RuleLevel::Default,
        rule_id: Some(1),
        value,
        manual: false,
    };
    // Half up (PRD C2): 5,625 at 6% = 337.5, so 338; 5,624 gives 337; 5,630 gives 338.
    for (base, want) in [
        (5_625, 338),
        (5_624, 337),
        (5_630, 338),
        (0, 0),
        (12_000, 720),
    ] {
        let c = charged(Some(&winner(percent(600))), rupees(base), qty).unwrap();
        assert_eq!(c.amount, rupees(want), "6% of {base}");
        assert_eq!(c.working.message_id, "working-percent");
    }
    // 2.5% of 1,000 = 25; 0% = 0; 100% = the whole base.
    assert_eq!(
        charged(Some(&winner(percent(250))), rupees(1_000), qty)
            .unwrap()
            .amount,
        rupees(25)
    );
    assert_eq!(
        charged(Some(&winner(percent(0))), rupees(1_000), qty)
            .unwrap()
            .amount,
        rupees(0)
    );
    assert_eq!(
        charged(Some(&winner(percent(10_000))), rupees(1_000), qty)
            .unwrap()
            .amount,
        rupees(1_000)
    );
    // Per نگ: 10 × 20 = 200 (PRD C3); لاگا 20 × 8 = 160.
    let c = charged(
        Some(&winner(per_unit_value(20))),
        rupees(0),
        Qty::new(10).unwrap(),
    )
    .unwrap();
    assert_eq!(
        (c.amount, c.working.message_id.as_str()),
        (rupees(200), "working-per-unit")
    );
    // Per lot: the value (PRD C4).
    let c = charged(
        Some(&winner(ChargeValue::PerLot(rupees(50)))),
        rupees(0),
        qty,
    )
    .unwrap();
    assert_eq!(
        (c.amount, c.working.message_id.as_str()),
        (rupees(50), "working-per-lot")
    );
    // No winner: 0 and a working that says so.
    let c = charged(None, rupees(12_000), qty).unwrap();
    assert_eq!(
        (c.amount, c.working.message_id.as_str(), c.working.rule_id),
        (Paisa::ZERO, "working-none", None)
    );
    // The result inside every working is the amount.
    for value in [
        percent(600),
        per_unit_value(8),
        ChargeValue::PerLot(rupees(50)),
    ] {
        let c = charged(Some(&winner(value)), rupees(12_000), qty).unwrap();
        let result = c.working.args.iter().find(|a| a.name == "result").unwrap();
        assert_eq!(result.value, WorkingValue::Money { paisa: c.amount });
        assert!(c.working.args.iter().any(|a| a.name == "level"));
        assert_eq!(c.working.rule_id, Some(1));
    }
    // The working of the 6% example carries its inputs: base 12,000, 6%.
    let c = charged(Some(&winner(percent(600))), rupees(12_000), qty).unwrap();
    assert_eq!(
        c.working.args[0].value,
        WorkingValue::Money {
            paisa: rupees(12_000)
        }
    );
    assert_eq!(c.working.args[1].value, WorkingValue::Percent { bps: 600 });
}

#[test]
fn a_huge_per_unit_charge_overflows_with_an_error_not_a_wrong_number() {
    let big = Winner {
        level: RuleLevel::Item,
        rule_id: Some(1),
        value: ChargeValue::PerUnit(Paisa::from_rupees(10_000_000_000).unwrap()),
        manual: false,
    };
    // 1,000,000 نگ × 10,00,00,00,000 rupees = 1e16 rupees = 1e18 paisa: it still fits an i64.
    let c = charged(Some(&big), rupees(0), Qty::new(1_000_000).unwrap()).unwrap();
    assert_eq!(c.amount.as_paisa(), 1_000_000_i64 * 1_000_000_000_000);
    // A quantity from a sum of lots that is bigger still gives an error, never a wrap.
    assert!(charged(Some(&big), rupees(0), Qty::new(100_000_000).unwrap()).is_err());
}

#[test]
fn ten_thousand_rules_resolve_in_microseconds() {
    // NFR-SCL-6: under 1 ms with 10,000 rules (a generous check here; `pnpm perf` holds the
    // real number).
    let mut rules = defaults();
    for n in 0..10_000_i64 {
        let kind = n % 4;
        rules.push(rule(
            100 + n,
            if n % 2 == 0 {
                ChargeSide::Seller
            } else {
                ChargeSide::Buyer
            },
            ChargeType::Commission,
            if kind < 2 {
                Some(1_000 + n % 500)
            } else {
                None
            },
            if kind == 0 || kind == 2 {
                Some(2_000 + n % 300)
            } else {
                None
            },
            None,
            percent(i32::try_from(n % 1_000).unwrap()),
        ));
    }
    let set = RuleSet::build(rules);
    assert_eq!(set.len(), 10_003);
    let query = Query {
        side: ChargeSide::Seller,
        charge: ChargeType::Commission,
        party_id: Some(1_100),
        item_id: 2_100,
        item_group: Some(ItemGroup::Fruit),
        date: day("2026-10-10"),
        line_override: None,
    };
    let started = std::time::Instant::now();
    let rounds = 2_000_u32;
    for _ in 0..rounds {
        std::hint::black_box(resolve(&set, std::hint::black_box(&query)));
    }
    let per_lookup = started.elapsed() / rounds;
    assert!(
        per_lookup < std::time::Duration::from_millis(1),
        "{per_lookup:?}"
    );
}

// ---- Property: the lookup agrees with a brute-force model ----

#[derive(Debug, Clone)]
struct Spec {
    side: bool,
    party: Option<i64>,
    item: Option<i64>,
    group: Option<bool>,
    from: u32,
    to: Option<u32>,
    active: bool,
    bps: i32,
}

fn arb_spec() -> impl Strategy<Value = Spec> {
    (
        any::<bool>(),
        proptest::option::of(1_i64..4),
        proptest::option::of(1_i64..4),
        proptest::option::of(any::<bool>()),
        1_u32..20,
        proptest::option::of(1_u32..28),
        proptest::bool::weighted(0.85),
        0_i32..=10_000,
    )
        .prop_map(|(side, party, item, group, from, to, active, bps)| Spec {
            side,
            party,
            item,
            group,
            from,
            to,
            active,
            bps,
        })
}

fn date_of(n: u32) -> BusinessDate {
    BusinessDate::from_ymd(2026, 10, n.clamp(1, 28)).unwrap()
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10_000))]

    /// For any set of rules and any lookup, the winner is the matching rule with the lowest level
    /// number and, among those, the one that starts latest, then the highest id; and the skipped levels are exactly the levels
    /// before it.
    #[test]
    fn the_lookup_agrees_with_a_brute_force_model(
        specs in proptest::collection::vec(arb_spec(), 0..40),
        buyer_side in any::<bool>(),
        party in 1_i64..4,
        item in 1_i64..4,
        group in any::<bool>(),
        asked in 1_u32..28,
    ) {
        let side = if buyer_side { ChargeSide::Buyer } else { ChargeSide::Seller };
        let group_value = if group { ItemGroup::Fruit } else { ItemGroup::Vegetable };
        let rules: Vec<Rule> = specs
            .iter()
            .enumerate()
            .map(|(i, s)| Rule {
                id: i64::try_from(i).unwrap() + 1,
                side: if s.side { ChargeSide::Buyer } else { ChargeSide::Seller },
                charge: ChargeType::Commission,
                account_id: s.party,
                item_id: s.item,
                item_group: s.group.map(|g| if g { ItemGroup::Fruit } else { ItemGroup::Vegetable }),
                value: percent(s.bps),
                from_date: date_of(s.from),
                to_date: s.to.map(date_of),
                is_active: s.active,
            })
            .collect();
        let date = date_of(asked);
        let query = Query {
            side,
            charge: ChargeType::Commission,
            party_id: Some(party),
            item_id: item,
            item_group: Some(group_value),
            date,
            line_override: None,
        };
        let found = resolve(&RuleSet::build(rules.clone()), &query);
        // The model: look at every rule.
        let best = rules
            .iter()
            .filter(|r| r.side == side && r.applies_on(date))
            .filter_map(|r| {
                let level = r.level()?;
                let matches = match level {
                    RuleLevel::PartyItem => r.account_id == Some(party) && r.item_id == Some(item),
                    RuleLevel::Party => r.account_id == Some(party),
                    RuleLevel::Item => r.item_id == Some(item),
                    RuleLevel::ItemGroup => r.item_group == Some(group_value),
                    RuleLevel::Default => true,
                    RuleLevel::LineOverride => false,
                };
                matches.then_some((level.number(), std::cmp::Reverse((r.from_date, r.id))))
            })
            .min();
        match (found.winner, best) {
            (None, None) => prop_assert_eq!(found.skipped.len(), 6),
            (Some(w), Some((number, std::cmp::Reverse((_, id))))) => {
                prop_assert_eq!(w.level.number(), number);
                prop_assert_eq!(w.rule_id, Some(id));
                prop_assert_eq!(found.skipped.len(), usize::from(number) - 1);
            }
            (got, want) => prop_assert!(false, "{got:?} against {want:?}"),
        }
    }

    /// The working's result is always the amount, and a percent charge never exceeds its base
    /// (up to 100%) and is never negative.
    #[test]
    fn the_working_result_is_the_amount(
        base in 0_i64..=10_000_000_000,
        bps in 0_i32..=10_000,
        qty in 1_i64..=1_000_000,
        unit in 1_i64..=100_000,
    ) {
        let winner = |value| Winner { level: RuleLevel::Default, rule_id: Some(1), value, manual: false };
        let qty = Qty::new(qty).unwrap();
        for value in [percent(bps), per_unit_value(unit), ChargeValue::PerLot(rupees(unit))] {
            let c = charged(Some(&winner(value)), rupees(base), qty).unwrap();
            let result = c.working.args.iter().find(|a| a.name == "result").unwrap();
            prop_assert_eq!(result.value, WorkingValue::Money { paisa: c.amount });
            prop_assert!(!c.amount.is_negative());
            prop_assert_eq!(c.amount.as_paisa() % 100, 0);
        }
        let pct = charged(Some(&winner(percent(bps))), rupees(base), qty).unwrap();
        prop_assert!(pct.amount <= rupees(base));
    }
}

#[test]
fn a_planned_change_is_not_cancelled_by_a_later_entry_that_starts_earlier() {
    // "Newer" is the rule that starts later; on the same start day, the one made last. So
    // "7% from 01-11" (entered first) still applies in November after "5% from 01-10" is
    // entered (money agent review, FR-11.5).
    let side = ChargeSide::Seller;
    let charge = ChargeType::Commission;
    let mut planned = rule(10, side, charge, Some(RAHIM), None, None, percent(700));
    planned.from_date = day("2026-11-01");
    let mut earlier = rule(11, side, charge, Some(RAHIM), None, None, percent(500));
    earlier.from_date = day("2026-10-01");
    let set = RuleSet::build(vec![planned, earlier]);
    let on = |d: &str| {
        resolve(
            &set,
            &Query {
                side,
                charge,
                party_id: Some(RAHIM),
                item_id: BANANA,
                item_group: Some(ItemGroup::Fruit),
                date: day(d),
                line_override: None,
            },
        )
        .winner
        .and_then(|w| w.rule_id)
    };
    assert_eq!(on("2026-10-10"), Some(11));
    assert_eq!(on("2026-11-05"), Some(10));
}

#[test]
fn a_per_lot_charge_is_due_once_and_a_lot_charge_names_the_qty_received() {
    // PRD FR-11.9 and FR-12.17: the per-lot amount is charged on the first line of the first
    // posting only; every other line gets 0 with its own working.
    let qty = Qty::new(50).unwrap();
    let per_lot = Winner {
        level: RuleLevel::Default,
        rule_id: Some(5),
        value: ChargeValue::PerLot(rupees(50)),
        manual: false,
    };
    let due = charge_amount(Some(&per_lot), ChargeType::Munshiana, rupees(0), qty, true).unwrap();
    assert_eq!(
        (due.amount, due.working.message_id.as_str()),
        (rupees(50), "working-per-lot")
    );
    let not_due =
        charge_amount(Some(&per_lot), ChargeType::Munshiana, rupees(0), qty, false).unwrap();
    assert_eq!(not_due.amount, Paisa::ZERO);
    assert_eq!(
        not_due.working.message_id,
        "working-per-lot-already-charged"
    );
    assert_eq!(not_due.working.rule_id, Some(5));
    // A per-نگ lot charge: 5 per نگ on the 50 received is 250 (PRD 4.4).
    let labour = Winner {
        value: per_unit_value(5),
        ..per_lot
    };
    let c = charge_amount(Some(&labour), ChargeType::Labour, rupees(0), qty, true).unwrap();
    assert_eq!(c.amount, rupees(250));
    assert!(c.working.args.iter().any(|a| a.name == "qty_received"));
    // A line charge names the line qty.
    let laga = charge_amount(Some(&labour), ChargeType::Laga, rupees(0), qty, true).unwrap();
    assert!(laga.working.args.iter().any(|a| a.name == "qty"));
    // A per-lot value of 0 is allowed.
    let zero = Winner {
        value: ChargeValue::PerLot(Paisa::ZERO),
        ..per_lot
    };
    assert_eq!(
        charge_amount(Some(&zero), ChargeType::Phone, rupees(0), qty, true)
            .unwrap()
            .amount,
        Paisa::ZERO
    );
}

#[test]
fn a_row_with_a_method_the_charge_does_not_have_is_never_used() {
    // PRD 11.2: seller labour has no percent method. A row like that (made outside the app)
    // is left out of the set.
    let bad = rule(
        5,
        ChargeSide::Seller,
        ChargeType::Labour,
        None,
        None,
        None,
        percent(600),
    );
    assert!(!bad.method_is_allowed());
    assert_eq!(RuleSet::build(vec![bad]).len(), 0);
    let fine = rule(
        6,
        ChargeSide::Seller,
        ChargeType::Labour,
        None,
        None,
        None,
        per_unit_value(5),
    );
    assert_eq!(RuleSet::build(vec![fine]).len(), 1);
}

#[test]
fn a_typed_value_follows_prd_11_2_and_a_negative_value_is_never_used() {
    // A typed per-lot commission on the buyer side is not a thing (PRD 11.2): it would be
    // charged on every line. The lookup skips it, like a rule row with that method.
    let set = RuleSet::build(defaults());
    let query = Query {
        side: ChargeSide::Buyer,
        charge: ChargeType::Commission,
        party_id: Some(AKRAM),
        item_id: BANANA,
        item_group: Some(ItemGroup::Fruit),
        date: day("2026-10-10"),
        line_override: Some(LineOverride {
            value: ChargeValue::PerLot(rupees(500)),
        }),
    };
    let found = resolve(&set, &query);
    assert_eq!(found.winner.map(|w| w.level), Some(RuleLevel::Default));
    assert_eq!(found.skipped[0].level, RuleLevel::LineOverride);
    // A typed per-نگ value on the buyer side is allowed: 20 × 3 = 60 and the message names it.
    let per_unit_override = Query {
        line_override: Some(LineOverride {
            value: per_unit_value(3),
        }),
        ..query
    };
    let winner = resolve(&set, &per_unit_override).winner.unwrap();
    let c = charge_amount(
        Some(&winner),
        ChargeType::Commission,
        rupees(12_000),
        Qty::new(20).unwrap(),
        true,
    )
    .unwrap();
    assert_eq!(
        (c.amount, c.working.message_id.as_str()),
        (rupees(60), "working-manual-per-unit")
    );
    // A negative amount (made outside the app) is left out of the set and refused as a typed
    // value.
    let negative = Paisa::try_from_paisa(-100).unwrap();
    let bad = rule(
        9,
        ChargeSide::Seller,
        ChargeType::Labour,
        None,
        None,
        None,
        ChargeValue::PerUnit(negative),
    );
    assert_eq!(RuleSet::build(vec![bad]).len(), 0);
    let negative_override = Query {
        side: ChargeSide::Buyer,
        charge: ChargeType::Laga,
        line_override: Some(LineOverride {
            value: ChargeValue::PerUnit(negative),
        }),
        ..query
    };
    assert_eq!(
        resolve(&set, &negative_override).winner.map(|w| w.level),
        Some(RuleLevel::ItemGroup)
    );
}
