//! Grows the word list from pairs of labels (docs/research/translit-options.md Section 7,
//! plan 05). A development tool, not part of the app.
//!
//! ```bash
//! cargo run -p mandi-core --release --example translit_import -- \
//!     --seed crates/core/data/translit_seed.tsv \
//!     --pairs pk.tsv --place-words place_words.tsv --target 15500 \
//!     --out new_seed.tsv --holdout translit_holdout.tsv
//! ```
//!
//! `--pairs` files are made by `data/import/prepare_pairs.py` (columns: English label, Urdu
//! label, kind, source, id). The tool:
//!
//! 1. keeps every Wikidata item whose number ends in 0 OUT of the word list, as the hold-out
//!    set of the accuracy test (research 5.2). The rule is fixed in code, so it is repeatable;
//! 2. cuts each label pair into word pairs by place, and only if both labels have the same
//!    number of clean words (letters only);
//! 3. counts how often each pair occurs and gives the most common Urdu spelling of an English
//!    word the weight 100 (variants a lower weight, and never above the old list's);
//! 4. drops pairs the spelling model cannot line up (`WordList::unlined_pairs`);
//! 5. with `--place-words FILE` (word, count), adds the place words that the list does not know
//!    with an Urdu spelling PROPOSED BY OUR OWN RULES. They have the source `engine`: they stay
//!    in the file for the reviewer to confirm in bulk, but do not count as known words, so the
//!    app still marks them "check the spelling" (PRD FR-7.66);
//! 6. with `--target N`, keeps only as many new pairs as the list needs to reach N entries:
//!    pairs from the first `--pairs` file first, then the more often seen ones;
//! 7. writes the old list followed by the new pairs, every new pair `pending` review, with its
//!    source, and the hold-out names that are not whole entries of the new list.
//!
//! Nothing here reads a test file: the frozen list `translit_names.tsv` is never an input.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fmt::Write as _;
use std::io::Write as _;
use std::process::ExitCode;

use mandi_core::text::{is_urdu_letter, normalize_urdu};
use mandi_core::translit::{Entry, Kind, WordList, suggest_to_urdu};

/// Words that are English grammar, not names.
const STOP_WORDS: [&str; 14] = [
    "the", "of", "and", "in", "at", "for", "de", "la", "le", "el", "du", "des", "van", "von",
];
/// English number words and fillers in place names (`Chak No. One Hundred`), not place words.
const PLACE_STOP_WORDS: [&str; 18] = [
    "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten", "hundred",
    "thousand", "number", "the", "and", "ward", "block", "unit",
];
/// The longest English word kept, in letters.
const MAX_WORD_LETTERS: usize = 20;
/// The most Urdu spellings kept for one new English word.
const MAX_VARIANTS: usize = 3;
/// A variant of a word the old list already has must be seen at least this often.
const MIN_COUNT_FOR_OLD_WORD: u32 = 2;
/// The weight a variant of an old word may have at most, so the old top spelling stays first.
const OLD_WORD_VARIANT_WEIGHT: u32 = 50;
/// The most words in a hold-out name.
const MAX_HOLDOUT_WORDS: usize = 4;

type AnyError = Box<dyn std::error::Error>;

fn say(line: &str) {
    // Printing macros are denied in this workspace; this writes the same way.
    writeln!(std::io::stderr(), "{line}").ok();
}

struct Args {
    seed: String,
    pairs: Vec<String>,
    place_words: Option<String>,
    target: Option<usize>,
    out: String,
    holdout: String,
}

fn parse_args() -> Option<Args> {
    let mut args = std::env::args().skip(1);
    let (mut seed, mut pairs, mut out, mut holdout) = (None, Vec::new(), None, None);
    let mut target = None;
    let mut place_words = None;
    while let Some(flag) = args.next() {
        let value = args.next()?;
        match flag.as_str() {
            "--seed" => seed = Some(value),
            "--pairs" => pairs.push(value),
            "--place-words" => place_words = Some(value),
            "--target" => target = Some(value.parse().ok()?),
            "--out" => out = Some(value),
            "--holdout" => holdout = Some(value),
            _ => return None,
        }
    }
    Some(Args {
        seed: seed?,
        pairs,
        place_words,
        target,
        out: out?,
        holdout: holdout?,
    })
}

/// One line of a pairs file.
struct Row {
    english: String,
    urdu: String,
    kind: Kind,
    source: String,
    id: u64,
}

fn read_rows(path: &str) -> Result<Vec<Row>, AnyError> {
    let text = std::fs::read_to_string(path)?;
    let mut rows = Vec::new();
    for line in text.lines() {
        let mut c = line.split('\t');
        let (Some(english), Some(urdu), Some(kind), Some(source)) =
            (c.next(), c.next(), c.next(), c.next())
        else {
            continue;
        };
        let Some(kind) = Kind::parse(kind) else {
            continue;
        };
        rows.push(Row {
            english: english.to_owned(),
            urdu: urdu.to_owned(),
            kind,
            source: source.to_owned(),
            id: c.next().and_then(|v| v.parse().ok()).unwrap_or(0),
        });
    }
    Ok(rows)
}

/// The words of an English label: letters only. `None` if any word is not clean.
fn english_words(label: &str) -> Option<Vec<String>> {
    let mut words = Vec::new();
    for raw in label.split_whitespace() {
        let word: String = raw.chars().filter(|c| *c != '\'' && *c != '’').collect();
        let word = word.trim_matches('.');
        if word.chars().count() < 2
            || word.chars().count() > MAX_WORD_LETTERS
            || !word.chars().all(|c| c.is_ascii_alphabetic())
        {
            return None;
        }
        words.push(word.to_lowercase());
    }
    (!words.is_empty()).then_some(words)
}

/// The words of an Urdu label, normalised: Urdu letters only. `None` if any word is not clean.
fn urdu_words(label: &str) -> Option<Vec<String>> {
    let normalised = normalize_urdu(label);
    let words: Vec<String> = normalised.split(' ').map(str::to_owned).collect();
    let clean = words
        .iter()
        .all(|w| w.chars().count() >= 2 && w.chars().all(is_urdu_letter));
    (clean && !words.is_empty()).then_some(words)
}

/// Both labels cut into words, if they have the same number.
fn aligned(english: &str, urdu: &str) -> Option<(Vec<String>, Vec<String>)> {
    let (e, u) = (english_words(english)?, urdu_words(urdu)?);
    (e.len() == u.len()).then_some((e, u))
}

#[derive(Default)]
struct Tally {
    /// The first `--pairs` file this pair was seen in (lower is more trusted).
    origin: usize,
    count: u32,
    kinds: BTreeMap<&'static str, u32>,
    sources: BTreeSet<String>,
}

fn kind_name(kind: Kind) -> &'static str {
    kind.as_str()
}

fn main() -> ExitCode {
    let Some(args) = parse_args() else {
        say(
            "usage: translit_import --seed FILE --pairs FILE [--pairs FILE ...] --out FILE --holdout FILE",
        );
        return ExitCode::from(2);
    };
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            say(&format!("translit_import: {error}"));
            ExitCode::FAILURE
        }
    }
}

fn run(args: &Args) -> Result<(), AnyError> {
    let old_text = std::fs::read_to_string(&args.seed)?;
    let old = WordList::parse(&old_text)?;
    let old_pairs: HashSet<(String, String)> = old
        .entries()
        .iter()
        .map(|e| (e.roman.clone(), e.urdu.clone()))
        .collect();
    let old_romans: HashSet<String> = old.entries().iter().map(|e| e.roman.clone()).collect();

    // 1. Read, hold out, cut into word pairs and count.
    let mut tallies: BTreeMap<(String, String), Tally> = BTreeMap::new();
    let mut holdout: BTreeMap<String, (String, String)> = BTreeMap::new();
    let (mut labels, mut kept_labels, mut held) = (0_usize, 0_usize, 0_usize);
    for (origin, path) in args.pairs.iter().enumerate() {
        for row in read_rows(path)? {
            labels = labels.saturating_add(1);
            let Some((english, urdu)) = aligned(&row.english, &row.urdu) else {
                continue;
            };
            if row.source == "wikidata" && row.id % 10 == 0 {
                held = held.saturating_add(1);
                if english.len() <= MAX_HOLDOUT_WORDS && row.kind == Kind::Name {
                    holdout.insert(english.join(" "), (urdu.join(" "), row.english.clone()));
                }
                continue;
            }
            kept_labels = kept_labels.saturating_add(1);
            for (e, u) in english.into_iter().zip(urdu) {
                if STOP_WORDS.contains(&e.as_str()) {
                    continue;
                }
                let tally = tallies.entry((e, u)).or_insert_with(|| Tally {
                    origin,
                    ..Tally::default()
                });
                tally.count = tally.count.saturating_add(1);
                let slot = tally.kinds.entry(kind_name(row.kind)).or_insert(0);
                *slot = slot.saturating_add(1);
                tally.sources.insert(row.source.clone());
            }
        }
    }
    say(&format!(
        "labels read {labels}, kept {kept_labels}, held out {held}, distinct word pairs {}",
        tallies.len()
    ));

    // 2. Weights: the commonest spelling of an English word first.
    let mut by_roman: BTreeMap<&str, Vec<(&str, &Tally)>> = BTreeMap::new();
    for ((roman, urdu), tally) in &tallies {
        by_roman
            .entry(roman.as_str())
            .or_default()
            .push((urdu.as_str(), tally));
    }
    let mut ranked: Vec<(usize, u32, Entry)> = Vec::new();
    let mut candidates: Vec<Entry>;
    for (roman, mut variants) in by_roman {
        variants.sort_by(|a, b| b.1.count.cmp(&a.1.count).then_with(|| a.0.cmp(b.0)));
        let top = variants.first().map_or(1, |v| v.1.count.max(1));
        let known = old_romans.contains(roman);
        for (rank, (urdu, tally)) in variants.iter().enumerate() {
            if rank >= MAX_VARIANTS
                || old_pairs.contains(&(roman.to_owned(), (*urdu).to_owned()))
                || (known && tally.count < MIN_COUNT_FOR_OLD_WORD)
            {
                continue;
            }
            let scaled = (100_u32.saturating_mul(tally.count) / top).clamp(1, 100);
            let weight = if known {
                scaled.min(OLD_WORD_VARIANT_WEIGHT)
            } else if rank == 0 {
                100
            } else {
                scaled.min(60)
            };
            // The kind seen most often; a name wins a tie.
            let kind = tally
                .kinds
                .iter()
                .max_by(|a, b| a.1.cmp(b.1).then_with(|| b.0.cmp(a.0)))
                .and_then(|(k, _)| Kind::parse(k))
                .unwrap_or(Kind::Name);
            ranked.push((
                tally.origin,
                tally.count,
                Entry {
                    roman: roman.to_owned(),
                    urdu: (*urdu).to_owned(),
                    kind,
                    weight,
                    review: "pending".to_owned(),
                    source: tally.sources.iter().cloned().collect::<Vec<_>>().join(","),
                },
            ));
        }
    }
    say(&format!(
        "candidates before the alignment check: {}",
        ranked.len()
    ));

    // 3. Drop what the spelling model cannot line up.
    let mut all: Vec<Entry> = old.entries().to_vec();
    all.extend(ranked.iter().map(|(_, _, e)| e.clone()));
    let combined = WordList::from_entries(all);
    let unlined: HashSet<(String, String)> = combined.unlined_pairs().iter().cloned().collect();
    let before = ranked.len();
    ranked.retain(|(_, _, e)| !unlined.contains(&(e.roman.clone(), e.urdu.clone())));
    say(&format!(
        "dropped {} that do not line up with the spelling rules; kept {}",
        before.saturating_sub(ranked.len()),
        ranked.len()
    ));

    // 3b. Only as many new pairs as the target needs: trusted files first, then often seen.
    if let Some(target) = args.target {
        let room = target.saturating_sub(old.len());
        ranked.sort_by(|a, b| {
            a.0.cmp(&b.0)
                .then_with(|| b.1.cmp(&a.1))
                .then_with(|| b.2.weight.cmp(&a.2.weight))
                .then_with(|| a.2.roman.cmp(&b.2.roman))
                .then_with(|| a.2.urdu.cmp(&b.2.urdu))
        });
        ranked.truncate(room);
        say(&format!("target {target}: kept {} new pairs", ranked.len()));
    }

    // 3c. The English suggestion for an Urdu word is the heaviest English spelling of it. The
    // old list's spelling must stay first, and among the new ones the most often seen.
    let mut best_count: HashMap<String, u32> = HashMap::new();
    for (_, count, e) in &ranked {
        let slot = best_count.entry(e.urdu.clone()).or_insert(0);
        *slot = (*slot).max(*count);
    }
    for (_, count, e) in &mut ranked {
        if !old.by_urdu(&e.urdu).is_empty() {
            e.weight = e.weight.min(OLD_WORD_VARIANT_WEIGHT);
        } else {
            let top = best_count.get(&e.urdu).copied().unwrap_or(1).max(1);
            let scaled = (100_u32.saturating_mul(*count) / top).clamp(1, 100);
            e.weight = e.weight.min(scaled.max(1));
        }
    }
    candidates = ranked.into_iter().map(|(_, _, e)| e).collect();

    // 3d. Place words with an Urdu spelling proposed by the rules: kept amber until reviewed.
    let mut proposed = 0_usize;
    if let Some(path) = &args.place_words {
        let mut base: Vec<Entry> = old.entries().to_vec();
        base.extend(candidates.iter().cloned());
        let known: HashSet<String> = base.iter().map(|e| e.roman.clone()).collect();
        let base = WordList::from_entries(base);
        let room = args.target.map_or(usize::MAX, |t| {
            t.saturating_sub(old.len().saturating_add(candidates.len()))
        });
        for line in std::fs::read_to_string(path)?.lines() {
            if proposed >= room {
                break;
            }
            let Some(word) = line.split('\t').next().map(str::trim) else {
                continue;
            };
            if word.is_empty() || known.contains(word) || PLACE_STOP_WORDS.contains(&word) {
                continue;
            }
            let Some(urdu) = suggest_to_urdu(word, &[], &base).into_iter().next() else {
                continue;
            };
            if urdu_words(&urdu).is_none_or(|w| w.len() != 1) {
                continue;
            }
            candidates.push(Entry {
                roman: word.to_owned(),
                urdu,
                kind: Kind::Place,
                weight: 100,
                review: "pending".to_owned(),
                source: "engine".to_owned(),
            });
            proposed = proposed.saturating_add(1);
        }
        say(&format!(
            "place words with a proposed Urdu spelling (source engine): {proposed}"
        ));
    }

    // 4. Write the new list: the old lines first, then the new pairs.
    let mut text = String::from("roman\turdu\tkind\tweight\treview\tsource\n");
    for e in old.entries() {
        let _ = writeln!(
            text,
            "{}\t{}\t{}\t{}\t{}\t{}",
            e.roman,
            e.urdu,
            e.kind.as_str(),
            e.weight,
            e.review,
            e.source
        );
    }
    candidates.sort_by(|a, b| {
        a.roman
            .cmp(&b.roman)
            .then_with(|| b.weight.cmp(&a.weight))
            .then_with(|| a.urdu.cmp(&b.urdu))
    });
    for e in &candidates {
        let _ = writeln!(
            text,
            "{}\t{}\t{}\t{}\t{}\t{}",
            e.roman,
            e.urdu,
            e.kind.as_str(),
            e.weight,
            e.review,
            e.source
        );
    }
    std::fs::write(&args.out, &text)?;
    let total = old.len().saturating_add(candidates.len());
    say(&format!(
        "word list: {} old + {} new = {total} entries",
        old.len(),
        candidates.len()
    ));

    // 5. The hold-out: names that are not whole entries of the new list.
    let new_list = WordList::parse(&text)?;
    let mut lines = String::from(
        "# Hold-out test names (research 5.2, T2): Wikidata items whose number ends in 0, never in the word list.\n\
         # Columns: typed English letters, correct Urdu, accepted English name. Source: Wikidata (CC0).\n",
    );
    let mut written = 0_usize;
    let mut seen_urdu: HashMap<String, ()> = HashMap::new();
    for (roman, (urdu, english)) in &holdout {
        let whole_entry =
            !new_list.by_roman(roman).is_empty() || !new_list.by_urdu(urdu).is_empty();
        if whole_entry || seen_urdu.insert(urdu.clone(), ()).is_some() {
            continue;
        }
        let _ = writeln!(lines, "{roman}\t{urdu}\t{english}");
        written = written.saturating_add(1);
    }
    std::fs::write(&args.holdout, lines)?;
    say(&format!(
        "hold-out names written: {written} of {}",
        holdout.len()
    ));
    Ok(())
}
