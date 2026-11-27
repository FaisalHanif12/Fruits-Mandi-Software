//! The shipped word lists: names in `data/translit_seed.tsv` and everyday words in
//! `data/translit_everyday.tsv` (Tech Stack 4.4, PRD FR-7.27, NFR-URD-30, plans 05 and 99).
#![cfg(test)]

use std::collections::HashSet;

use mandi_core::text::is_urdu_name;
use mandi_core::translit::{Direction, Hint, Kind, WordList, suggest_for};

const SEED: &str = include_str!("../data/translit_seed.tsv");
const EVERYDAY: &str = include_str!("../data/translit_everyday.tsv");

#[test]
fn the_list_has_the_agreed_columns_and_every_entry_is_pending_review() {
    let first = SEED.lines().next().unwrap_or_default();
    assert_eq!(first, "roman\turdu\tkind\tweight\treview\tsource");
    let list = WordList::embedded();
    assert_eq!(
        EVERYDAY.lines().next().unwrap_or_default(),
        "roman\turdu\tkind\tweight\treview\tsource"
    );
    // The first list lost the 4,730 rows only our rules had proposed (plan 99); they are in
    // `data/translit_engine_places.tsv` for a reviewer.
    let names = list.entries().iter().filter(|e| !e.is_everyday()).count();
    assert!(names >= 8_200, "the name list shrank: {names} entries");
    for entry in list.entries() {
        // Plan 05: every entry waits for the Urdu reviewer (NFR-URD-4) until it is marked
        // reviewed. Everyday words are checked by a second, independent checker (plan 99).
        let allowed: &[&str] = if entry.is_everyday() {
            &["checked", "reviewed"]
        } else {
            &["pending", "reviewed"]
        };
        assert!(
            allowed.contains(&entry.review.as_str()),
            "{} has review {:?}",
            entry.roman,
            entry.review
        );
        assert!((1..=100).contains(&entry.weight), "{}", entry.roman);
    }
    // Nothing has been reviewed by the Urdu reviewer yet.
    assert!(
        list.entries().iter().all(|e| e.review != "reviewed"),
        "an entry was marked reviewed without a recorded reviewer"
    );
}

#[test]
fn the_everyday_dictionary_is_large_and_every_row_is_checked() {
    // NFR-URD-30: about 15,000 common words with about 40,000 spellings, each row checked.
    let list = WordList::embedded();
    let everyday: Vec<_> = list.entries().iter().filter(|e| e.is_everyday()).collect();
    let words: HashSet<&str> = everyday.iter().map(|e| e.urdu.as_str()).collect();
    let spellings: HashSet<&str> = everyday.iter().map(|e| e.roman.as_str()).collect();
    println!(
        "everyday: {} rows, {} Urdu words, {} spellings",
        everyday.len(),
        words.len(),
        spellings.len()
    );
    assert!(words.len() >= 14_500, "{} words", words.len());
    assert!(spellings.len() >= 40_000, "{} spellings", spellings.len());
    for entry in everyday {
        assert_eq!(entry.review, "checked", "{} {}", entry.roman, entry.urdu);
        assert_eq!(entry.kind, Kind::Word, "{} {}", entry.roman, entry.urdu);
    }
}

/// How many entries count as known words, by source (printed with `-- --nocapture`).
fn known_report() -> (usize, usize) {
    let list = WordList::embedded();
    let known = list
        .entries()
        .iter()
        .filter(|e| e.counts_as_known())
        .count();
    let mut by_source: std::collections::BTreeMap<&str, usize> = std::collections::BTreeMap::new();
    for e in list.entries() {
        *by_source.entry(e.source.as_str()).or_default() += 1;
    }
    println!(
        "word list: {} entries, {} known; by source {:?}",
        list.len(),
        known,
        by_source
    );
    (list.len(), known)
}

#[test]
fn no_pair_only_our_rules_proposed_is_shipped() {
    // Plan 99: the proposed rows (ghar غار, aya عیہ) outranked the rules and spoiled typing.
    let (total, known) = known_report();
    assert_eq!(total, known, "every shipped row was written by a person");
    let list = WordList::embedded();
    for entry in list.entries() {
        assert_ne!(entry.source, "engine", "{}", entry.roman);
    }
}

/// PRD FR-7.27 v1.2 and NFR-URD-24 v1.2: at least 15,000 known entries.
#[test]
fn the_list_has_fifteen_thousand_known_entries() {
    let (_, known) = known_report();
    assert!(known >= 15_000, "{known} known entries");
}

#[test]
fn every_entry_names_a_source_that_is_credited() {
    // Only sources whose licence the research report cleared (docs/research/translit-options.md
    // Section 2), and each one credited in data/SOURCES.md.
    const SOURCES_FILE: &str = include_str!("../data/SOURCES.md");
    let list = WordList::embedded();
    let mut used: HashSet<&str> = HashSet::new();
    for entry in list.entries() {
        for part in entry.source.split(',') {
            assert!(
                matches!(part, "team" | "wikidata" | "geonames" | "everyday"),
                "{} has source {:?}",
                entry.roman,
                entry.source
            );
            used.insert(part);
        }
    }
    for source in &used {
        assert!(
            SOURCES_FILE.contains(&format!("`{source}`")),
            "{source} is not credited in SOURCES.md"
        );
    }
    // CC BY 4.0 asks for the attribution wherever GeoNames data is shipped.
    if used.contains("geonames") {
        assert!(SOURCES_FILE.contains("data from GeoNames"));
        assert!(SOURCES_FILE.contains("CC BY 4.0"));
    }
    // No source that the research report ruled out.
    for banned in ["dakshina", "openstreetmap", "osm", "aksharantar"] {
        assert!(
            !used.contains(banned),
            "{banned} is not allowed in the list"
        );
    }
}

#[test]
fn spellings_are_clean() {
    let list = WordList::embedded();
    // A pair may be in both lists (کرایہ is a shop word and an everyday word), once in each.
    let mut pairs: HashSet<(bool, &str, &str)> = HashSet::new();
    for entry in list.entries() {
        assert!(
            !entry.roman.is_empty()
                && entry
                    .roman
                    .split(' ')
                    .all(|w| !w.is_empty() && w.chars().all(|c| c.is_ascii_lowercase())),
            "bad English key {:?}",
            entry.roman
        );
        assert!(
            is_urdu_name(&entry.urdu) && entry.urdu == entry.urdu.trim(),
            "bad Urdu {:?} for {}",
            entry.urdu,
            entry.roman
        );
        assert!(
            !entry.urdu.contains("  "),
            "double space in {:?}",
            entry.urdu
        );
        assert!(
            entry.roman.split(' ').count() <= 6,
            "a key of more than 6 words: {}",
            entry.roman
        );
        assert!(
            pairs.insert((
                entry.is_everyday(),
                entry.roman.as_str(),
                entry.urdu.as_str()
            )),
            "{} listed twice with {}",
            entry.roman,
            entry.urdu
        );
    }
}

#[test]
fn the_list_covers_the_kinds_the_plan_names() {
    let list = WordList::embedded();
    let count =
        |kind: mandi_core::translit::Kind| list.entries().iter().filter(|e| e.kind == kind).count();
    use mandi_core::translit::Kind;
    assert!(count(Kind::Name) >= 2_000, "names {}", count(Kind::Name));
    assert!(count(Kind::Place) >= 450, "places {}", count(Kind::Place));
    assert!(count(Kind::Item) >= 380, "items {}", count(Kind::Item));
    assert!(count(Kind::Word) >= 500, "words {}", count(Kind::Word));
    // Mandi words of the glossary (PRD section 3) are there.
    for word in [
        "bikri",
        "rokar",
        "wasooli",
        "peshgi",
        "munshiana",
        "laga",
        "adda",
        "nag",
    ] {
        assert!(list.contains_roman(word), "{word} is missing");
    }
}

#[test]
fn most_pairs_line_up_with_the_spelling_rules() {
    // The model learns from the pairs it can line up letter group by letter group. The rest
    // (English item words such as banana -> کیلا) still work as plain lookups.
    let (lined, skipped) = WordList::embedded().training_report();
    let total = lined + skipped;
    assert!(
        lined * 100 >= total * 80,
        "only {lined} of {total} pairs line up"
    );
}

#[test]
fn a_known_word_comes_back_the_way_it_was_listed() {
    // In a name field, every name list spelling is among the suggestions.
    let list = WordList::embedded();
    for entry in list
        .entries()
        .iter()
        .filter(|e| !e.is_everyday() && !e.roman.contains(' '))
    {
        let got = mandi_core::translit::suggest_to_urdu(&entry.roman, &[], list);
        assert!(
            got.iter().any(|g| g == &entry.urdu),
            "{} -> {:?} does not give {}",
            entry.roman,
            got,
            entry.urdu
        );
    }
}

#[test]
fn a_strong_everyday_spelling_comes_back_in_free_text() {
    // In free text, a usual everyday spelling is among the three. Every third one is typed,
    // to keep the debug test quick.
    let list = WordList::embedded();
    let free_text = Hint {
        kind: Some(Kind::Word),
        before: None,
    };
    let mut missed = Vec::new();
    for entry in list
        .entries()
        .iter()
        .filter(|e| e.is_everyday() && e.weight >= 85 && !e.roman.contains(' '))
        .step_by(3)
    {
        let got = suggest_for(&entry.roman, &[], list, Direction::ToUrdu, &free_text);
        if !got.iter().any(|g| g.text == entry.urdu) {
            missed.push(format!("{} -> {}", entry.roman, entry.urdu));
        }
    }
    assert!(missed.is_empty(), "{} missed: {:?}", missed.len(), missed);
}
