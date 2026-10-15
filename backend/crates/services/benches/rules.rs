//! Finding the winning charge rule with 10,000 rules (NFR-SCL-6, D2-A13). Owner: `robustness`.
//!
//! The target is under 1 ms at the 95th percentile. Four shapes of 10,000 rules, because the
//! speed of the lookup depends on how the rules are spread:
//!
//! - `spread`: rules for many parties and items, as a real book would have them (most lookups
//!   find a party and item rule, some fall to a lower level).
//! - `one_party_item`: all 10,000 rules for the same party and item with different dates, the
//!   worst case for the "newest rule that applies" scan.
//! - `one_default`: 10,000 dated versions of the seller commission default.
//! - `miss`: a lookup that matches nothing at the five specific levels and falls to the default.
//!
//! `rules/build_10000` is the time to file 10,000 rules again after any rule change.

use std::hint::black_box;

mod support;

use criterion::{Criterion, criterion_group, criterion_main};
use mandi_services::core::dates::BusinessDate;
use mandi_services::core::money::Bps;
use mandi_services::core::rules::{ChargeSide, ChargeType, ChargeValue, ItemGroup};
use mandi_services::domain::rules::{Query, Rule, RuleSet, resolve};

const RULES: i64 = 10_000;

fn day(offset: i64) -> BusinessDate {
    let date = chrono::NaiveDate::from_ymd_opt(1999, 1, 1).unwrap_or_default()
        + chrono::Duration::days(offset);
    match BusinessDate::parse_iso(&date.format("%Y-%m-%d").to_string()) {
        Some(found) => found,
        None => support::fail("bad date in the rules bench"),
    }
}

fn percent(bps: i32) -> ChargeValue {
    ChargeValue::Percent(Bps::new(bps).unwrap_or(Bps::FULL))
}

fn rule(
    id: i64,
    party: Option<i64>,
    item: Option<i64>,
    from: BusinessDate,
    to: Option<BusinessDate>,
) -> Rule {
    Rule {
        id,
        side: ChargeSide::Seller,
        charge: ChargeType::Commission,
        account_id: party,
        item_id: item,
        item_group: None,
        value: percent(300 + i32::try_from(id % 600).unwrap_or(0)),
        from_date: from,
        to_date: to,
        is_active: true,
    }
}

fn query(party: Option<i64>, item: i64, date: BusinessDate) -> Query {
    Query {
        side: ChargeSide::Seller,
        charge: ChargeType::Commission,
        party_id: party,
        item_id: item,
        item_group: Some(ItemGroup::Fruit),
        date,
        line_override: None,
    }
}

/// Rules for 400 parties and 25 items, by id; plus item rules, group rules and a default.
fn spread() -> Vec<Rule> {
    let start = day(0);
    let mut rules = vec![rule(1, None, None, start, None)];
    for id in 2..=RULES {
        let n = id - 2;
        let level = n % 10;
        let party = 1 + n % 400;
        let item = 1 + (n / 7) % 25;
        let built = match level {
            0..=5 => rule(id, Some(party), Some(item), start, None),
            6..=7 => rule(id, Some(party), None, start, None),
            _ => rule(id, None, Some(item), start, None),
        };
        rules.push(built);
    }
    rules
}

/// 10,000 rules, one day each, for one key.
fn one_key(party: Option<i64>, item: Option<i64>) -> Vec<Rule> {
    let base = day(0);
    (1..=RULES)
        .map(|id| {
            let from = day(id);
            rule(id, party, item, from, Some(from))
        })
        .chain(std::iter::once(rule(RULES + 1, party, item, base, None)))
        .collect()
}

fn rules_speed(c: &mut Criterion) {
    let spread_set = RuleSet::build(spread());
    let party_set = RuleSet::build(one_key(Some(7), Some(3)));
    let default_set = RuleSet::build(one_key(None, None));
    let when = day(5_000);

    let mut group = c.benchmark_group("rules");
    // 400 parties x 25 items: take them in turn.
    let queries: Vec<Query> = (0..2_000_i64)
        .map(|n| query(Some(1 + n % 450), 1 + n % 30, when))
        .collect();
    let mut next = 0_usize;
    group.bench_function("resolve_spread_10000", |b| {
        b.iter(|| {
            next = next.wrapping_add(1);
            let q = queries
                .get(next % queries.len())
                .copied()
                .unwrap_or(query(None, 1, when));
            black_box(resolve(black_box(&spread_set), black_box(&q)))
        });
    });
    // Worst case: every rule is for party 7 and item 3. The newest rule that applies on
    // day 5,000 is found among 10,001.
    let hit = query(Some(7), 3, day(5_000));
    group.bench_function("resolve_one_party_item_10000", |b| {
        b.iter(|| black_box(resolve(black_box(&party_set), black_box(&hit))));
    });
    let default_query = query(Some(11), 4, day(7_777));
    group.bench_function("resolve_one_default_10000", |b| {
        b.iter(|| black_box(resolve(black_box(&default_set), black_box(&default_query))));
    });
    // A lookup for a charge that has no rules at all: all six levels are tried and skipped.
    let miss = Query {
        side: ChargeSide::Buyer,
        charge: ChargeType::Laga,
        ..query(Some(1), 1, when)
    };
    group.bench_function("resolve_miss_10000", |b| {
        b.iter(|| black_box(resolve(black_box(&spread_set), black_box(&miss))));
    });
    group.bench_function("build_10000", |b| {
        b.iter_batched(
            spread,
            |rules| black_box(RuleSet::build(rules).len()),
            criterion::BatchSize::LargeInput,
        );
    });
    group.finish();
}

criterion_group!(benches, rules_speed);
criterion_main!(benches);
