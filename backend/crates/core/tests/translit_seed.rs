//! The shipped word list `data/translit_seed.tsv` (Tech Stack 4.4, PRD FR-7.27, plan 05).
#![cfg(test)]

use std::collections::HashSet;

use mandi_core::text::is_urdu_name;
use mandi_core::translit::WordList;

const SEED: &str = include_str!("../data/translit_seed.tsv");

#[test]
fn the_list_has_the_agreed_columns_and_every_entry_is_pending_review() {
    let first = SEED.lines().next().unwrap_or_default();
    assert_eq!(first, "roman\turdu\tkind\tweight\treview\tsource");
    let list = WordList::embedded();
    // The file keeps what it had when the Indian people sets were taken out (lead decision,
    // 12-10-2026). PRD FR-7.27 v1.2 and NFR-URD-24 v1.2 ask for 15,000 KNOWN entries: see the
    // ignored test below for that gate. The list is not padded.
    assert!(
        list.len() >= 12_000,
        "the file shrank: {} entries",
        list.len()
    );
    for entry in list.entries() {
        // Plan 05: every entry waits for the Urdu reviewer (NFR-URD-4) until it is marked reviewed.
        assert!(
            entry.review == "pending" || entry.review == "reviewed",
            "{} has review {:?}",
            entry.roman,
            entry.review
        );
        assert!((1..=100).contains(&entry.weight), "{}", entry.roman);
    }
    // The first release ships with nothing reviewed yet.
    assert!(
        list.entries().iter().all(|e| e.review == "pending"),
        "an entry was marked reviewed without a recorded reviewer"
    );
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
fn the_known_entries_are_reported_and_proposed_ones_stay_out_of_them() {
    let (total, known) = known_report();
    assert!(
        known < total,
        "rows proposed by the rules exist and are not known"
    );
    let list = WordList::embedded();
    for entry in list.entries() {
        if entry.source == "engine" {
            // A pair whose Urdu only our rules proposed: kept for the reviewer, never known
            // until marked reviewed, and always a place word (research report 4.6).
            assert_eq!(entry.review, "pending", "{}", entry.roman);
            assert_eq!(
                entry.kind,
                mandi_core::translit::Kind::Place,
                "{}",
                entry.roman
            );
            assert!(!entry.counts_as_known(), "{}", entry.roman);
        } else {
            assert!(entry.counts_as_known(), "{}", entry.roman);
        }
    }
}

/// PRD FR-7.27 v1.2 and NFR-URD-24 v1.2: at least 15,000 known entries. Not met yet: the Pakistani
/// Wikidata and GeoNames sets give about 8,000 known entries. The rest has to come from the
/// reviewer confirming the proposed rows and from the shop's own 1,572 account names (Phase 2
/// import). The list is not padded to pass this test.
#[test]
#[ignore = "NFR-URD-24: fewer than 15,000 known entries; see SOURCES.md and the HANDOFF"]
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
                matches!(part, "team" | "wikidata" | "geonames" | "engine"),
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
    let mut pairs: HashSet<(&str, &str)> = HashSet::new();
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
            pairs.insert((entry.roman.as_str(), entry.urdu.as_str())),
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
    let list = WordList::embedded();
    for entry in list.entries().iter().filter(|e| !e.roman.contains(' ')) {
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
