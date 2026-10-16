//! English-to-Urdu suggestions per key press (NFR-SPD-20, D2 attack list item 2). Owner:
//! `robustness`.
//!
//! The call is the real one the field makes on each key: `translit::suggest` with the learned pairs
//! held in memory and the embedded word list. The texts come from the frozen 1,000-name list of
//! NFR-URD-24 (`core/tests/translit_names.tsv`): for every name, the text as it looks after each
//! key press (`a`, `ar`, `ars` ... `arshad`, then `arshad m`, `arshad mu` ...). Names are typed as
//! the first column says; the Urdu column is used for the other direction (FR-7.24).
//!
//! The learned pairs are not empty: 500 pairs are learned first through the real `learn` call, so
//! the lookup is measured with a table the shop has used for a while.
//!
//! Needs a `[[bench]] name = "translit" harness = false` entry in
//! `backend/crates/services/Cargo.toml`.

mod support;

use std::hint::black_box;
use std::time::Duration;

use criterion::{Criterion, SamplingMode, criterion_group, criterion_main};
use mandi_services::core::translit::{Direction, Kind};
use mandi_services::translit::{self, NewPair};
use support::{fail, note, open};

/// The frozen list, read at compile time. Column 1: typed English letters. Column 2: Urdu.
const NAMES: &str = include_str!("../../core/tests/translit_names.tsv");

/// Texts per bench. A bench goes round them.
const MAX_TEXTS: usize = 2_000;

/// Learned pairs put in before measuring.
const LEARNED: usize = 500;

fn rows() -> Vec<(String, String)> {
    NAMES
        .lines()
        .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
        .filter_map(|l| {
            let mut cols = l.split('\t');
            let roman = cols.next()?.trim().to_owned();
            let urdu = cols.next()?.trim().to_owned();
            // Only the first Urdu spelling when the list gives alternatives.
            let urdu = urdu.split('|').next().unwrap_or("").trim().to_owned();
            (!roman.is_empty() && !urdu.is_empty()).then_some((roman, urdu))
        })
        .collect()
}

/// Every text the field holds while one name is typed: one letter more each time.
fn per_key(name: &str) -> Vec<String> {
    let chars: Vec<char> = name.chars().collect();
    (1..=chars.len())
        .map(|n| chars.iter().take(n).collect::<String>())
        .filter(|t| !t.ends_with(' '))
        .collect()
}

/// Up to `MAX_TEXTS` texts, spread evenly over the whole list so every kind of name is in it.
fn spread(all: Vec<String>) -> Vec<String> {
    if all.len() <= MAX_TEXTS {
        return all;
    }
    let step = all.len().div_ceil(MAX_TEXTS);
    all.into_iter().step_by(step).collect()
}

fn translit_speed(c: &mut Criterion) {
    let dir = tempfile::tempdir().unwrap_or_else(|e| fail(&format!("no temp folder: {e}")));
    // A fresh data folder: `open` makes the Owner on an empty folder and signs in.
    let (env, actor) = open(dir.path());
    let ctx = &*env.ctx;

    let list = rows();
    if list.len() < 900 {
        fail(&format!("the name list has only {} rows", list.len()));
    }

    // Learn pairs the way the screen does: one English word and its Urdu spelling.
    let mut learned = 0_usize;
    for (roman, urdu) in list.iter().take(LEARNED * 2) {
        if learned >= LEARNED {
            break;
        }
        let (Some(r), Some(u)) = (
            roman.split_whitespace().next(),
            urdu.split_whitespace().next(),
        ) else {
            continue;
        };
        let pair = NewPair {
            roman: r.to_owned(),
            urdu: u.to_owned(),
            kind: Kind::Name,
        };
        if translit::learn(ctx, &actor, &pair).is_ok() {
            learned += 1;
        }
    }
    note(&format!(
        "translit bench: {learned} learned pairs in the table"
    ));
    if learned < LEARNED / 2 {
        fail("could not learn enough pairs for a fair measure");
    }
    // The first key press of a session builds the trained tables; the app does this at start-up.
    translit::warm_up(ctx);

    let english: Vec<String> = spread(list.iter().flat_map(|(r, _)| per_key(r)).collect());
    let urdu: Vec<String> = spread(list.iter().flat_map(|(_, u)| per_key(u)).collect());
    let full: Vec<String> = list.iter().map(|(r, _)| r.clone()).collect();
    note(&format!(
        "translit bench: {} English key texts, {} Urdu key texts, {} full names",
        english.len(),
        urdu.len(),
        full.len()
    ));

    let mut group = c.benchmark_group("translit");
    group.sampling_mode(SamplingMode::Flat);
    group.sample_size(50);
    group.measurement_time(Duration::from_secs(6));

    let mut run = |id: &str, texts: &[String], direction: Direction| {
        let mut next = 0_usize;
        group.bench_function(id, |b| {
            b.iter(|| {
                let text = texts
                    .get(next % texts.len())
                    .map(String::as_str)
                    .unwrap_or("");
                next = next.wrapping_add(1);
                match translit::suggest(ctx, black_box(text), direction) {
                    Ok(found) => black_box(found.len()),
                    Err(error) => fail(&format!("suggest failed on '{text}': {error}")),
                }
            });
        });
    };
    run("suggest_to_urdu_per_key", &english, Direction::ToUrdu);
    run("suggest_to_urdu_full_name", &full, Direction::ToUrdu);
    run("suggest_to_english_per_key", &urdu, Direction::ToEnglish);
    group.finish();
}

criterion_group!(benches, translit_speed);
criterion_main!(benches);
