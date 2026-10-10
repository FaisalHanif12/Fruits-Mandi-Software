//! Search key speed budgets (plan 02 T6). Owner: `robustness`.
//!
//! The keys are built in two places (Tech Stack 4.7, Code Standards B26, B27):
//!
//! - For every account when the in-memory search index is built, at start-up and after a
//!   restore. 20,000 accounts (NFR-SCL-1). The index build gets 10% of the 2 s cold start
//!   (NFR-SPD-1): 200 ms for 20,000 accounts, 10 µs per account for all three keys.
//! - For the typed text on every key press in a picker. NFR-SPD-6 gives the whole search
//!   50 ms; building the query keys must take under 1% of it (0.5 ms), which the 10 µs
//!   per-account budget already meets with room to spare.
//!
//! Budgets: `normalize_urdu` 5 µs and all three keys 10 µs for a typical name; all three
//! keys 30 µs for a 100-character name (the longest allowed, Code Standards 4.2);
//! 200 ms for 20,000 accounts. Each id has one entry in `scripts/perf/thresholds.json`.

use std::hint::black_box;

use criterion::{Criterion, SamplingMode, criterion_group, criterion_main};
use mandi_core::text::{normalize_urdu, search_en, search_phonetic, search_ur};

/// Accounts in the NFR-SCL-1 volume.
const ACCOUNTS: usize = 20_000;

/// A typical name with Arabic letter variants, a diacritic and extra spaces to clean.
const NAME_UR: &str = "محمد  اكرم\u{064E} ٹول ٹيكس";
const NAME_EN: &str = "Muhammad  Akram Toll Tax";

/// All three keys of one account.
fn keys(name_ur: &str, name_en: &str) -> (String, String, String) {
    (
        search_ur(name_ur),
        search_en(name_en),
        search_phonetic(name_en),
    )
}

/// 20,000 different names, built from a few Urdu and English parts.
fn accounts() -> Vec<(String, String)> {
    let ur = [
        "ارشد",
        "اكرم",
        "شاہد",
        "ناصر",
        "رحيم",
        "كريم",
        "طارق",
        "محمود",
    ];
    let en = [
        "Arshad", "Akram", "Shahid", "Nasir", "Rahim", "Karim", "Tariq", "Mehmood",
    ];
    let ur_last = ["ملتانی", "ٹول ٹيكس", "اینڈ سنز", "بٹ"];
    let en_last = ["Multani", "Toll Tax", "and Sons", "Butt"];
    (0..ACCOUNTS)
        .map(|i| {
            let a = i % ur.len();
            let b = (i / ur.len()) % ur_last.len();
            let first_ur = ur.get(a).copied().unwrap_or_default();
            let last_ur = ur_last.get(b).copied().unwrap_or_default();
            let first_en = en.get(a).copied().unwrap_or_default();
            let last_en = en_last.get(b).copied().unwrap_or_default();
            (
                format!("{first_ur} {last_ur} {i}"),
                format!("{first_en} {last_en} {i}"),
            )
        })
        .collect()
}

fn text(c: &mut Criterion) {
    let long_ur: String = "اكرم\u{064E} ".repeat(16);
    let long_en: String = "Akram Toll ".repeat(8) + "Tax";

    let mut group = c.benchmark_group("text");
    group.bench_function("normalize_urdu_name", |b| {
        b.iter(|| normalize_urdu(black_box(NAME_UR)));
    });
    group.bench_function("search_keys_name", |b| {
        b.iter(|| keys(black_box(NAME_UR), black_box(NAME_EN)));
    });
    group.bench_function("search_keys_name_100", |b| {
        b.iter(|| keys(black_box(&long_ur), black_box(&long_en)));
    });
    group.finish();

    let all = accounts();
    let mut group = c.benchmark_group("index");
    group.sampling_mode(SamplingMode::Flat);
    group.sample_size(20);
    group.bench_function("keys_20000_accounts", |b| {
        b.iter(|| {
            black_box(&all)
                .iter()
                .map(|(ur, en)| keys(ur, en))
                .collect::<Vec<_>>()
        });
    });
    group.finish();
}

criterion_group!(benches, text);
criterion_main!(benches);
