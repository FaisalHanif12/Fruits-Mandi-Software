//! Account screens, the party card, the ledger and a voucher save on the NFR-SCL-1 volume
//! (NFR-SPD-3, NFR-SPD-9, NFR-SPD-11, NFR-SPD-13, NFR-SPD-21, D2-A10). Owner: `robustness`.
//!
//! 20,000 accounts and about 3.5 million posted journal lines over 10 years. The busiest buyer
//! and the busiest seller have about 30,000 lines each: the worst case for a card and a ledger.
//!
//! - `accounts/list_all_20000`: the account list with no page limit, as a screen that draws
//!   only the visible rows (NFR-SPD-18) still asks for it. NFR-SPD-3, 300 ms.
//! - `accounts/list_page_200` and `accounts/search_text_all`: a page and a text filter.
//! - `coa/tree`: the chart of accounts with every balance (FR-10.1). NFR-SPD-3, 300 ms.
//! - `assist/party_card_*`: the card of a code just typed. NFR-SPD-21, 100 ms.
//! - `ledger/party_ledger_busiest_10y`: NFR-SPD-11, 2 s.
//! - `ledger/trial_balance_10y`: NFR-SPD-13, 2 s.
//! - `vouchers/save_advance`: NFR-SPD-9, 150 ms. This one writes, so it runs on a private copy.

mod support;

use std::hint::black_box;
use std::time::Duration;

use criterion::{Criterion, SamplingMode, criterion_group, criterion_main};
use mandi_services::core::accounts::{AccountFilter, AccountsListInput};
use mandi_services::core::vouchers::{AdvanceInput, VoucherMode};
use mandi_services::{assist, coa, ledger, vouchers};
use support::{Rng, ensure_volume, fail, open};

fn flat<'a>(
    c: &'a mut Criterion,
    name: &str,
    samples: usize,
    seconds: u64,
) -> criterion::BenchmarkGroup<'a, criterion::measurement::WallTime> {
    let mut group = c.benchmark_group(name);
    group.sampling_mode(SamplingMode::Flat);
    group.sample_size(samples);
    group.measurement_time(Duration::from_secs(seconds));
    group
}

fn list_input(text: &str, limit: u32) -> AccountsListInput {
    AccountsListInput {
        filter: AccountFilter {
            account_type: None,
            text: text.to_owned(),
            include_inactive: true,
        },
        offset: 0,
        limit,
    }
}

fn account_speed(c: &mut Criterion) {
    let volume = ensure_volume();
    support::describe(&volume);
    let (env, _actor) = open(&volume.dir);
    let ctx = &*env.ctx;

    let mut group = flat(c, "accounts", 20, 6);
    group.bench_function("list_all_20000", |b| {
        b.iter(
            || match mandi_services::accounts::list(ctx, &list_input("", 0)) {
                Ok(list) => black_box(list.rows.len()),
                Err(error) => fail(&format!("list failed: {error}")),
            },
        );
    });
    group.bench_function("list_page_200", |b| {
        b.iter(
            || match mandi_services::accounts::list(ctx, &list_input("", 200)) {
                Ok(list) => black_box(list.rows.len()),
                Err(error) => fail(&format!("list failed: {error}")),
            },
        );
    });
    group.bench_function("search_text_all", |b| {
        b.iter(
            || match mandi_services::accounts::list(ctx, &list_input("ارشد", 0)) {
                Ok(list) => black_box(list.rows.len()),
                Err(error) => fail(&format!("list failed: {error}")),
            },
        );
    });
    group.finish();

    let mut group = flat(c, "coa", 20, 6);
    group.bench_function("tree", |b| {
        b.iter(|| match coa::tree(black_box(ctx)) {
            Ok(tree) => black_box(tree.heads.len()),
            Err(error) => fail(&format!("tree failed: {error}")),
        });
    });
    group.finish();

    // The cards: the busiest buyer, the busiest seller (with advances to add up), an ordinary
    // buyer, and 40 accounts picked at random, taken in turn.
    let mut rng = Rng(5);
    let mut ids: Vec<i64> = vec![volume.busy_buyer, volume.busy_seller, volume.median_buyer];
    let random: Vec<i64> = (0..40)
        .map(|_| {
            let serial = 1 + rng.below(8_000);
            let code = if rng.below(2) == 0 {
                format!("0304{serial:04}")
            } else {
                format!("03011{serial:04}")
            };
            env.ctx
                .db()
                .read(|conn| mandi_services::db::accounts::id_by_code(conn, &code))
                .ok()
                .flatten()
                .unwrap_or(volume.median_buyer)
        })
        .collect();
    ids.extend(random);
    let cash = env
        .ctx
        .db()
        .read(|conn| mandi_services::db::accounts::id_by_code(conn, "01010001"))
        .ok()
        .flatten()
        .unwrap_or(volume.median_buyer);
    let mut group = flat(c, "assist", 50, 4);
    for (id_name, pick) in [
        ("party_card_busiest_buyer", Some(volume.busy_buyer)),
        ("party_card_busiest_seller", Some(volume.busy_seller)),
        ("party_card_ordinary_buyer", Some(volume.median_buyer)),
        ("party_card_cash_account", Some(cash)),
        ("party_card_mixed", None),
    ] {
        let mut next = 0_usize;
        group.bench_function(id_name, |b| {
            b.iter(|| {
                let id = pick.unwrap_or_else(|| {
                    next = next.wrapping_add(1);
                    ids.get(next % ids.len())
                        .copied()
                        .unwrap_or(volume.median_buyer)
                });
                match assist::party_card(black_box(ctx), id) {
                    Ok(card) => black_box(card.id),
                    Err(error) => fail(&format!("party card failed: {error}")),
                }
            });
        });
    }
    group.finish();

    let mut group = flat(c, "ledger", 20, 8);
    group.bench_function("party_ledger_busiest_10y", |b| {
        b.iter(|| {
            match ledger::ledger(
                ctx,
                volume.busy_buyer,
                None,
                Some("2016-10-11"),
                Some("2026-10-10"),
            ) {
                Ok(book) => black_box(book.lines.len()),
                Err(error) => fail(&format!("ledger failed: {error}")),
            }
        });
    });
    group.bench_function("party_ledger_ordinary_10y", |b| {
        b.iter(|| {
            match ledger::ledger(
                ctx,
                volume.median_buyer,
                None,
                Some("2016-10-11"),
                Some("2026-10-10"),
            ) {
                Ok(book) => black_box(book.lines.len()),
                Err(error) => fail(&format!("ledger failed: {error}")),
            }
        });
    });
    // The cash book of one day (PRD R12): the opening is the sum of everything before.
    group.bench_function("cash_book_one_day", |b| {
        b.iter(
            || match ledger::ledger(ctx, cash, None, Some("2026-09-15"), Some("2026-09-15")) {
                Ok(book) => black_box(book.lines.len()),
                Err(error) => fail(&format!("cash book failed: {error}")),
            },
        );
    });
    group.bench_function("trial_balance_10y", |b| {
        b.iter(|| match ledger::trial_balance(black_box(ctx), None) {
            Ok(trial) => black_box(trial.rows.len()),
            Err(error) => fail(&format!("trial balance failed: {error}")),
        });
    });
    group.finish();
}

fn voucher_speed(c: &mut Criterion) {
    let volume = ensure_volume();
    let copy = support::copy_of_volume(&volume);
    let (env, actor) = open(copy.path());
    let seller = volume.busy_seller;
    let input = AdvanceInput {
        confirm_cash_below_zero: false,
        date: "10-10-2026".into(),
        seller_account_id: seller,
        amount: "1,000".into(),
        mode: VoucherMode::Cash,
        bank_account_id: None,
        cheque_no: String::new(),
        cheque_date: String::new(),
        expected_goods: String::new(),
        narration: String::new(),
    };
    let mut group = flat(c, "vouchers", 50, 6);
    group.bench_function("save_advance", |b| {
        b.iter(
            || match vouchers::save_advance(&env.ctx, &actor, black_box(&input)) {
                Ok(saved) => black_box(saved.number),
                Err(error) => fail(&format!("save failed: {error}")),
            },
        );
    });
    // An ordinary seller (about 100 lines in 10 years).
    let ordinary = env
        .ctx
        .db()
        .read(|conn| mandi_services::db::accounts::id_by_code(conn, "030114500"))
        .ok()
        .flatten()
        .unwrap_or(seller);
    let ordinary_input = AdvanceInput {
        confirm_cash_below_zero: false,
        seller_account_id: ordinary,
        ..input.clone()
    };
    group.bench_function("save_advance_ordinary_seller", |b| {
        b.iter(
            || match vouchers::save_advance(&env.ctx, &actor, black_box(&ordinary_input)) {
                Ok(saved) => black_box(saved.number),
                Err(error) => fail(&format!("save failed: {error}")),
            },
        );
    });
    group.finish();
}

criterion_group!(benches, account_speed, voucher_speed);
criterion_main!(benches);
