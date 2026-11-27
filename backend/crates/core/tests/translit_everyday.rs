//! The everyday words, typed the way people type them (NFR-URD-30, plan 99 U1c).
//!
//! The held-out lists were written by someone who never saw the dictionary, so they measure
//! the dictionary and the engine, not their memory of the lists. Never change an item to make
//! the engine pass, and never copy its items into the dictionary.
//!
//! One line: `roman <TAB> urdu`, accepted spellings split by `|`. Each item is typed in a
//! free-text field word by word, as the app does it: each word is converted with the text
//! before it as the context.
//!
//! - Items of one to three words: the first suggestion is right for at least 95%, and the right
//!   spelling is among the three suggestions of its words for at least 99%.
//! - Sentences (four words or more): every word right with its first suggestion for at least
//!   90%.
//!
//! Three lists are measured:
//! - `translit_everyday_test3.tsv`, the final list. It was written after the engine and the
//!   dictionary were finished, and was first run once, blind (lead, plan 99): that run is the
//!   official score. Only its figures are ever printed, never an item.
//! - `translit_everyday_test2.tsv`, the second list. Its first blind run came before the last
//!   rounds of general rules and topic words; no item of it was ever printed.
//! - `translit_everyday_test.tsv`, the first list. The engine and the dictionary were improved
//!   from the misses of its first half (every second line), so only its second half, whose
//!   misses are never shown, is a fair figure.
//!
//! No build may fall below the figures of 27-11-2026 on any of the three (Umar, plan 99). The
//! 95 / 99 / 90 targets above are for go-live, like those of NFR-URD-24: they are printed
//! with the current figures on every run, and checked by an ignored test until they are met.
//!
//! The figures print with `cargo test -p mandi-core --test translit_everyday -- --nocapture`
//! and are written to `translit_everyday_final.txt`, `translit_everyday_fresh.txt` and
//! `translit_everyday_accuracy.txt` in the build folder.
#![cfg(test)]

use std::collections::{BTreeMap, HashSet};
use std::fmt::Write as _;

use mandi_core::text::normalize_urdu;
use mandi_core::translit::{Direction, Hint, Kind, MAX_SUGGESTIONS, WordList, suggest_for};

const LIST: &str = include_str!("translit_everyday_test.tsv");
const FRESH: &str = include_str!("translit_everyday_test2.tsv");
const FINAL: &str = include_str!("translit_everyday_test3.tsv");
const GLOSSARY: &str = include_str!("../data/everyday/src/00_glossary.lex");
/// The NFR-URD-30 targets, for go-live.
const GATE_TOP1: f64 = 95.0;
const GATE_TOP3: f64 = 99.0;
const GATE_SENTENCES: f64 = 90.0;

/// What a list must still get right: at least as many as on 27-11-2026 (Umar, plan 99). Counts,
/// so rounding never decides. The lists are never changed, so their sizes are checked too.
struct Floor {
    items: usize,
    top1: usize,
    top3: usize,
    sentences: usize,
    sentences_right: usize,
}

/// The official final score: 94.99% first, 97.31% in the top 3, 94.98% of sentences.
const FINAL_FLOOR: Floor = Floor {
    items: 1_896,
    top1: 1_801,
    top3: 1_845,
    sentences: 1_195,
    sentences_right: 1_135,
};
/// 88.29%, 95.10% and 93.88%.
const SECOND_FLOOR: Floor = Floor {
    items: 2_408,
    top1: 2_126,
    top3: 2_290,
    sentences: 849,
    sentences_right: 797,
};
/// The blind half of the first list: 98.35%, 99.33% and 98.06%.
const FIRST_BLIND_FLOOR: Floor = Floor {
    items: 2_540,
    top1: 2_498,
    top3: 2_523,
    sentences: 930,
    sentences_right: 912,
};
/// Floors for spellings the list has never seen (see the test), a little under what they
/// measured on 27-11-2026.
const UNSEEN_TOP1: f64 = 80.0;
const UNSEEN_TOP3: f64 = 90.0;
/// Every how many spellings one is hidden.
const HIDE_ONE_IN: usize = 20;
const MIN_ITEMS: usize = 3_000;
const MIN_SENTENCES: usize = 500;
/// An item of this many words or more is a sentence.
const SENTENCE_WORDS: usize = 4;
/// The most misses written in the report.
const MISSES_SHOWN: usize = 2_000;

struct Item {
    roman: String,
    urdu: Vec<String>,
}

fn parse(list: &str) -> Vec<Item> {
    list.lines()
        .filter(|l| !l.trim().is_empty() && !l.starts_with('#') && !l.starts_with("roman\t"))
        .map(|line| {
            let mut columns = line.split('\t');
            let roman = columns.next().unwrap_or_default().trim().to_owned();
            let urdu = columns
                .next()
                .unwrap_or_default()
                .split('|')
                .map(key)
                .filter(|u| !u.is_empty())
                .collect();
            Item { roman, urdu }
        })
        .collect()
}

/// The form two spellings are compared in: marks and Arabic look-alikes do not count.
fn key(urdu: &str) -> String {
    normalize_urdu(urdu)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// The suggestions of one typed word after `before`, as a free-text field gives them.
fn suggestions(word: &str, before: &str) -> Vec<String> {
    if !word.chars().any(|c| c.is_ascii_alphabetic()) {
        return vec![word.to_owned()];
    }
    let hint = Hint {
        kind: Some(Kind::Word),
        before: Some(before),
    };
    let found: Vec<String> = suggest_for(word, &[], WordList::embedded(), Direction::ToUrdu, &hint)
        .into_iter()
        .map(|s| s.text)
        .collect();
    if found.is_empty() {
        vec![word.to_owned()]
    } else {
        found
    }
}

/// The text the app would show if the user always takes the first suggestion.
fn typed(roman: &str) -> String {
    let mut out = String::new();
    for word in roman.split_whitespace() {
        let first = suggestions(word, &out).swap_remove(0);
        if !out.is_empty() {
            out.push(' ');
        }
        out.push_str(&first);
    }
    out
}

/// Every text the user can make by picking among the suggestions of each word.
fn reachable(roman: &str) -> Vec<String> {
    let mut texts = vec![String::new()];
    for word in roman.split_whitespace() {
        let mut grown = Vec::new();
        for text in &texts {
            for pick in suggestions(word, text).into_iter().take(MAX_SUGGESTIONS) {
                grown.push(if text.is_empty() {
                    pick
                } else {
                    format!("{text} {pick}")
                });
            }
        }
        texts = grown;
    }
    texts
}

#[derive(Default)]
struct Tally {
    items: usize,
    top1: usize,
    top3: usize,
    sentences: usize,
    sentences_right: usize,
    misses: Vec<String>,
}

fn pct(part: usize, whole: usize) -> f64 {
    if whole == 0 {
        0.0
    } else {
        100.0 * part as f64 / whole as f64
    }
}

fn measure<'a>(items: impl Iterator<Item = &'a Item>, show_misses: bool) -> Tally {
    let mut tally = Tally::default();
    for item in items {
        let first = typed(&item.roman);
        let right = item.urdu.contains(&key(&first));
        if item.roman.split_whitespace().count() >= SENTENCE_WORDS {
            tally.sentences += 1;
            tally.sentences_right += usize::from(right);
            if !right && show_misses {
                tally.misses.push(format!(
                    "sentence\t{}\twant {}\tgot {first}",
                    item.roman,
                    item.urdu.join(" | ")
                ));
            }
            continue;
        }
        tally.items += 1;
        tally.top1 += usize::from(right);
        let in_three = right
            || reachable(&item.roman)
                .iter()
                .any(|t| item.urdu.contains(&key(t)));
        tally.top3 += usize::from(in_three);
        if !right && show_misses {
            let kind = if in_three { "second" } else { "missing" };
            tally.misses.push(format!(
                "{kind}\t{}\twant {}\tgot {first}",
                item.roman,
                item.urdu.join(" | ")
            ));
        }
    }
    tally
}

#[test]
fn the_held_out_lists_are_large_enough() {
    for list in [LIST, FRESH, FINAL] {
        let items = parse(list);
        assert!(items.len() >= MIN_ITEMS, "{} items", items.len());
        let sentences = items
            .iter()
            .filter(|i| i.roman.split_whitespace().count() >= SENTENCE_WORDS)
            .count();
        assert!(sentences >= MIN_SENTENCES, "{sentences} sentences");
        let empty = items.iter().filter(|i| i.urdu.is_empty()).count();
        assert_eq!(empty, 0, "items with no Urdu");
    }
}

#[test]
fn no_test_sentence_was_copied_into_the_dictionary() {
    let words = WordList::embedded();
    for list in [LIST, FRESH, FINAL] {
        // Only a count is printed, so the fresh list stays blind.
        let copied = parse(list)
            .iter()
            .filter(|i| i.roman.split_whitespace().count() >= SENTENCE_WORDS)
            .filter(|i| words.contains_roman(&i.roman.to_lowercase()))
            .count();
        assert_eq!(
            copied, 0,
            "test sentences that are entries of the word list"
        );
    }
}

/// One line of the report: the three figures of NFR-URD-30, with the counts behind them.
fn figures(label: &str, t: &Tally) -> String {
    format!(
        "{label}: items of 1 to 3 words: {}  first {:.2}% ({})  top 3 {:.2}% ({})  sentences: {}  fully right {:.2}% ({})",
        t.items,
        pct(t.top1, t.items),
        t.top1,
        pct(t.top3, t.items),
        t.top3,
        t.sentences,
        pct(t.sentences_right, t.sentences),
        t.sentences_right,
    )
}

/// The go-live targets beside the current figures, printed on every run (as NFR-URD-24 does).
fn targets(t: &Tally) -> String {
    format!(
        "NFR-URD-30 go-live targets: first >= {GATE_TOP1}%, top 3 >= {GATE_TOP3}%, sentences >= {GATE_SENTENCES}%. Now: {:.2}% / {:.2}% / {:.2}%.",
        pct(t.top1, t.items),
        pct(t.top3, t.items),
        pct(t.sentences_right, t.sentences),
    )
}

fn write_report(file: &str, report: &str) {
    println!("{report}");
    let folder = std::path::Path::new(env!("CARGO_TARGET_TMPDIR"));
    let _ = std::fs::write(folder.join(file), report);
}

fn assert_floor(t: &Tally, floor: &Floor) {
    assert_eq!(t.items, floor.items, "the list changed: items");
    assert_eq!(t.sentences, floor.sentences, "the list changed: sentences");
    assert!(
        t.top1 >= floor.top1,
        "fell below the floor: {} right first, was {}",
        t.top1,
        floor.top1
    );
    assert!(
        t.top3 >= floor.top3,
        "fell below the floor: {} in the top 3, was {}",
        t.top3,
        floor.top3
    );
    assert!(
        t.sentences_right >= floor.sentences_right,
        "fell below the floor: {} sentences right, was {}",
        t.sentences_right,
        floor.sentences_right
    );
}

/// The final list, measured blind: `measure` collects no misses, so no item is ever printed.
fn final_tally() -> Tally {
    measure(parse(FINAL).iter(), false)
}

#[test]
fn the_final_list_does_not_fall() {
    let tally = final_tally();
    write_report(
        "translit_everyday_final.txt",
        &format!(
            "{}\n{}",
            figures("final list (blind)", &tally),
            targets(&tally)
        ),
    );
    assert_floor(&tally, &FINAL_FLOOR);
}

#[test]
#[ignore = "NFR-URD-30 go-live targets (95 / 99 / 90) are not met on the final list yet; see translit_everyday_final.txt"]
fn nfr_urd_30_go_live_targets_hold_on_the_final_list() {
    let tally = final_tally();
    assert!(
        pct(tally.top1, tally.items) >= GATE_TOP1,
        "first suggestion {:.2}%",
        pct(tally.top1, tally.items)
    );
    assert!(
        pct(tally.top3, tally.items) >= GATE_TOP3,
        "top 3 {:.2}%",
        pct(tally.top3, tally.items)
    );
    assert!(
        pct(tally.sentences_right, tally.sentences) >= GATE_SENTENCES,
        "sentences {:.2}%",
        pct(tally.sentences_right, tally.sentences)
    );
}

#[test]
fn the_second_list_does_not_fall() {
    // Blind: `measure` collects no misses, so no item of this list is ever printed.
    let items = parse(FRESH);
    let tally = measure(items.iter(), false);
    write_report(
        "translit_everyday_fresh.txt",
        &format!(
            "{}\n{}",
            figures("second list (blind)", &tally),
            targets(&tally)
        ),
    );
    assert_floor(&tally, &SECOND_FLOOR);
}

#[test]
fn the_blind_half_of_the_first_list_does_not_fall() {
    let items = parse(LIST);
    let first_half = measure(items.iter().step_by(2), true);
    let blind_half = measure(items.iter().skip(1).step_by(2), false);
    let whole = Tally {
        items: first_half.items + blind_half.items,
        top1: first_half.top1 + blind_half.top1,
        top3: first_half.top3 + blind_half.top3,
        sentences: first_half.sentences + blind_half.sentences,
        sentences_right: first_half.sentences_right + blind_half.sentences_right,
        misses: Vec::new(),
    };
    let mut report = String::new();
    for line in [
        figures("whole list", &whole),
        figures("first half (misses shown)", &first_half),
        figures("blind half (misses never shown)", &blind_half),
        targets(&blind_half),
    ] {
        let _ = writeln!(report, "{line}");
    }
    for miss in first_half.misses.iter().take(MISSES_SHOWN) {
        let _ = writeln!(report, "{miss}");
    }
    write_report("translit_everyday_accuracy.txt", &report);
    assert_floor(&blind_half, &FIRST_BLIND_FLOOR);
}

#[test]
fn a_spelling_the_list_never_saw_still_finds_its_word() {
    let all = WordList::embedded().entries().to_vec();
    let mut spellings: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for e in all
        .iter()
        .filter(|e| e.is_everyday() && !e.roman.contains(' ') && e.weight >= 60)
    {
        spellings.entry(&e.urdu).or_default().push(&e.roman);
    }
    let mut keys: Vec<(&str, &str)> = spellings
        .iter()
        .filter(|(_, r)| r.len() >= 2)
        .flat_map(|(u, r)| r.iter().map(move |r| (*r, *u)))
        .collect();
    keys.sort_unstable();
    let hidden: Vec<(&str, &str)> = keys.into_iter().step_by(HIDE_ONE_IN).collect();
    let gone: HashSet<&str> = hidden.iter().map(|(r, _)| *r).collect();
    let rest = all
        .iter()
        .filter(|e| !gone.contains(e.roman.as_str()))
        .cloned()
        .collect();
    let list = WordList::from_entries(rest);
    let hint = Hint {
        kind: Some(Kind::Word),
        before: None,
    };
    let (mut first, mut three) = (0, 0);
    for (roman, urdu) in &hidden {
        let got = suggest_for(roman, &[], &list, Direction::ToUrdu, &hint);
        first += usize::from(got.first().is_some_and(|s| s.text == *urdu));
        three += usize::from(got.iter().any(|s| s.text == *urdu));
    }
    println!(
        "unseen spellings: {}  first {:.2}%  top 3 {:.2}%",
        hidden.len(),
        pct(first, hidden.len()),
        pct(three, hidden.len())
    );
    assert!(pct(first, hidden.len()) >= UNSEEN_TOP1);
    assert!(pct(three, hidden.len()) >= UNSEEN_TOP3);
}

/// The checkers asked for proof (plan 99): plurals on English spellings are no longer list
/// rows, and the engine still gives each of these first.
#[test]
fn english_words_with_urdu_plurals_still_come_first() {
    for (roman, urdu) in [
        ("filmein", "فلمیں"),
        ("filmon", "فلموں"),
        ("schoolon", "سکولوں"),
        ("bankon", "بینکوں"),
        ("taxon", "ٹیکسوں"),
        ("orderon", "آرڈروں"),
        ("checkon", "چیکوں"),
        ("registeron", "رجسٹروں"),
        ("feesein", "فیسیں"),
        ("marketon", "مارکیٹوں"),
        ("computeron", "کمپیوٹروں"),
        ("plaze", "پلازے"),
        ("plazay", "پلازے"),
        ("cameray", "کیمرے"),
        ("doctoron", "ڈاکٹروں"),
        ("wicketein", "وکٹیں"),
    ] {
        assert_eq!(typed(roman), urdu, "{roman}");
    }
}

/// Words that other words must never push down (checkers, plan 99): `mandi` is the shop's
/// own منڈی, not مندی.
#[test]
fn the_shop_words_are_never_pushed_down() {
    for (roman, urdu) in [("mandi", "منڈی"), ("bakri", "بکری"), ("bikri", "بکری")] {
        assert_eq!(typed(roman), urdu, "{roman}");
    }
}

/// How people type in a hurry on WhatsApp (plan 99, the lead's examples).
#[test]
fn hurried_whatsapp_spellings_come_first() {
    for (roman, urdu) in [
        ("krdo", "کر دو"),
        ("hogya", "ہو گیا"),
        ("hogyi", "ہو گئی"),
        ("agya", "آ گیا"),
        ("arha", "آ رہا"),
        ("jarha", "جا رہا"),
        ("bht", "بہت"),
        ("mjy", "مجھے"),
        ("kidr", "کدھر"),
        ("yhn", "یہاں"),
        ("dukanpe", "دکان پہ"),
        ("g", "جی"),
    ] {
        assert_eq!(typed(roman), urdu, "{roman}");
    }
}

#[test]
fn umars_examples_are_typed_right() {
    // Plan 99: his three examples of 27-11-2026, typed word by word in a free-text field.
    assert_eq!(typed("gher ly kr gaye"), "گھر لے کر گئے");
    assert_eq!(typed("karaya"), "کرایہ");
    assert_eq!(typed("kiraya"), "کرایہ");
    assert_eq!(typed("hai"), "ہے");
}

#[test]
fn the_word_before_chooses_between_two_readings() {
    assert_eq!(typed("us ne kaha ke"), "اس نے کہا کہ");
    assert_eq!(typed("ap ke ghar"), "آپ کے گھر");
    // `laga` alone is the app word لاگا; after a noun it is the verb.
    assert_eq!(typed("laga"), "لاگا");
    assert_eq!(typed("kharcha laga"), "خرچہ لگا");
}

/// The app's business words (PRD Section 3): the first spelling of each, typed in free text,
/// gives exactly that word.
#[test]
fn the_app_words_come_first_for_their_spelling() {
    let mut wrong = Vec::new();
    for line in GLOSSARY.lines() {
        let line = line.split('#').next().unwrap_or_default().trim();
        let Some((urdu, romans)) = line.split_once('=') else {
            continue;
        };
        let Some(first) = romans.split_whitespace().next() else {
            continue;
        };
        let got = typed(&first.replace('_', " "));
        if key(&got) != key(urdu) {
            wrong.push(format!("{first}: want {} got {got}", urdu.trim()));
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

#[test]
fn name_fields_keep_names_first_and_free_text_keeps_everyday_words_first() {
    let list = WordList::embedded();
    let name = Hint {
        kind: Some(Kind::Name),
        before: None,
    };
    let got = suggest_for("khan", &[], list, Direction::ToUrdu, &name);
    assert_eq!(got[0].text, "خان");
    let got = suggest_for("sara", &[], list, Direction::ToUrdu, &name);
    assert_eq!(got[0].text, "سارہ");
    assert_eq!(typed("sara"), "سارا");
}
