//! Picker search speed on the NFR-SCL-1 volume (NFR-SPD-6, D2-A6). Owner: `robustness`.
//!
//! 20,000 accounts and 2,000 masters. Every call is the real service call a picker makes:
//! `search::search_accounts` (the text, the signed-in user's recent list read from the
//! database, the in-memory index) and `search::search` for the masters. The target is 50 ms at
//! the 95th percentile, "including small spelling mistakes and both names" (NFR-SPD-6).
//!
//! Each bench cycles through 64 different typed texts, so one sample covers many names and the
//! p95 is not the p95 of one lucky query. The texts are made from real account names: the exact
//! start of a name, the same with one spelling mistake, with two, and text that matches nothing.
//!
//! Needs, in `backend/crates/services/Cargo.toml` (owned by `backend`):
//! `[dev-dependencies] criterion.workspace = true` and one `[[bench]] harness = false` entry for
//! each of `search`, `accounts` and `rules`.

mod support;

use std::hint::black_box;
use std::time::Duration;

use criterion::{Criterion, SamplingMode, criterion_group, criterion_main};
use mandi_services::auth::Actor;
use mandi_services::core::accounts::{AccountScope, SearchAccountsInput};
use mandi_services::core::search::SearchScope;
use mandi_services::{Context, search};
use support::{Rng, account_names, ensure_volume, fail, open};

/// Queries per bench. The bench goes round them.
const QUERIES: usize = 64;

/// One spelling mistake at a random place in a word: a letter left out, doubled, swapped with
/// its neighbour or replaced.
fn mistake(word: &str, rng: &mut Rng) -> String {
    let mut chars: Vec<char> = word.chars().collect();
    if chars.len() < 3 {
        return word.to_owned();
    }
    let at = 1 + rng.index(chars.len() - 2);
    match rng.below(4) {
        0 => {
            chars.remove(at);
        }
        1 => {
            let c = chars.get(at).copied().unwrap_or('a');
            chars.insert(at, c);
        }
        2 => {
            chars.swap(at, at - 1);
        }
        _ => {
            if let Some(slot) = chars.get_mut(at) {
                *slot = if word.is_ascii() { 'x' } else { 'ک' };
            }
        }
    }
    chars.into_iter().collect()
}

/// The first two words of a name, with `edits` mistakes spread over them.
fn query(name: &str, edits: u32, rng: &mut Rng) -> String {
    let mut words: Vec<String> = name.split_whitespace().take(2).map(str::to_owned).collect();
    for n in 0..edits {
        let which = usize::try_from(n).unwrap_or(0) % words.len().max(1);
        if let Some(word) = words.get_mut(which) {
            *word = mistake(word, rng);
        }
    }
    words.join(" ")
}

fn texts(english: bool, edits: u32, seed: u64) -> Vec<String> {
    let mut rng = Rng(seed);
    (0..QUERIES)
        .map(|_| {
            let i = u32::try_from(rng.below(20_000)).unwrap_or(0);
            let (ur, en) = account_names(i);
            query(if english { &en } else { &ur }, edits, &mut rng)
        })
        .collect()
}

fn no_match() -> Vec<String> {
    let mut rng = Rng(77);
    (0..QUERIES)
        .map(|_| format!("zq{:x} wjv{:x}", rng.below(4096), rng.below(4096)))
        .collect()
}

fn prefixes(len: usize) -> Vec<String> {
    let mut rng = Rng(99);
    (0..QUERIES)
        .map(|_| {
            let i = u32::try_from(rng.below(20_000)).unwrap_or(0);
            account_names(i).1.chars().take(len).collect()
        })
        .collect()
}

fn account_search(ctx: &Context, actor: &Actor, scope: AccountScope, text: &str) -> usize {
    let input = SearchAccountsInput {
        scope,
        control_id: None,
        text: text.to_owned(),
        limit: 20,
    };
    match search::search_accounts(ctx, actor, &input) {
        Ok(hits) => hits.len(),
        Err(error) => fail(&format!("search failed on '{text}': {error}")),
    }
}

fn bench_texts(
    group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>,
    id: &str,
    ctx: &Context,
    actor: &Actor,
    scope: AccountScope,
    texts: &[String],
) {
    let mut next = 0_usize;
    group.bench_function(id, |b| {
        b.iter(|| {
            let text = texts
                .get(next % texts.len())
                .map(String::as_str)
                .unwrap_or("");
            next = next.wrapping_add(1);
            black_box(account_search(ctx, actor, scope, black_box(text)))
        });
    });
}

fn search_speed(c: &mut Criterion) {
    let volume = ensure_volume();
    let (env, actor) = open(&volume.dir);
    let ctx = &*env.ctx;
    // Build the index once, as the app does at start-up. Its cost is its own bench below.
    search::warm_up(ctx);

    let mut group = c.benchmark_group("search");
    group.sampling_mode(SamplingMode::Flat);
    group.sample_size(40);
    group.measurement_time(Duration::from_secs(4));

    let any = AccountScope::Any;
    bench_texts(
        &mut group,
        "accounts_en_exact",
        ctx,
        &actor,
        any,
        &texts(true, 0, 1),
    );
    bench_texts(
        &mut group,
        "accounts_en_typo1",
        ctx,
        &actor,
        any,
        &texts(true, 1, 2),
    );
    bench_texts(
        &mut group,
        "accounts_en_typo2",
        ctx,
        &actor,
        any,
        &texts(true, 2, 3),
    );
    bench_texts(
        &mut group,
        "accounts_ur_exact",
        ctx,
        &actor,
        any,
        &texts(false, 0, 4),
    );
    bench_texts(
        &mut group,
        "accounts_ur_typo1",
        ctx,
        &actor,
        any,
        &texts(false, 1, 5),
    );
    bench_texts(
        &mut group,
        "accounts_ur_typo2",
        ctx,
        &actor,
        any,
        &texts(false, 2, 6),
    );
    bench_texts(
        &mut group,
        "accounts_no_match",
        ctx,
        &actor,
        any,
        &no_match(),
    );
    bench_texts(
        &mut group,
        "accounts_prefix_1",
        ctx,
        &actor,
        any,
        &prefixes(1),
    );
    bench_texts(
        &mut group,
        "accounts_prefix_3",
        ctx,
        &actor,
        any,
        &prefixes(3),
    );
    bench_texts(
        &mut group,
        "buyers_en_typo1",
        ctx,
        &actor,
        AccountScope::Buyer,
        &texts(true, 1, 7),
    );

    // The masters: items (500) are the biggest list people type into.
    let item_texts: Vec<String> = [
        "Banan",
        "Appel",
        "Grapes No",
        "Mango",
        "Kinow",
        "Potatoe",
        "Tomatto",
    ]
    .iter()
    .cycle()
    .take(QUERIES)
    .map(|s| (*s).to_owned())
    .collect();
    let mut next = 0_usize;
    group.bench_function("masters_item_typo", |b| {
        b.iter(|| {
            let text = item_texts
                .get(next % QUERIES)
                .map(String::as_str)
                .unwrap_or("");
            next = next.wrapping_add(1);
            match search::search(ctx, &actor, SearchScope::Item, black_box(text), 20) {
                Ok(hits) => black_box(hits.len()),
                Err(error) => fail(&format!("item search failed: {error}")),
            }
        });
    });
    group.finish();

    // The index is built at start-up and after a restore (Tech Stack 4.7). README Section 6.2
    // gives it 10% of the 2 s cold start: 200 ms for 20,000 accounts.
    let mut group = c.benchmark_group("index");
    group.sampling_mode(SamplingMode::Flat);
    group.sample_size(20);
    group.measurement_time(Duration::from_secs(4));
    group.bench_function("rebuild_22000", |b| {
        b.iter(|| match search::rebuild(black_box(ctx)) {
            Ok(()) => black_box(()),
            Err(error) => fail(&format!("rebuild failed: {error}")),
        });
    });
    group.finish();
}

criterion_group!(benches, search_speed);
criterion_main!(benches);
