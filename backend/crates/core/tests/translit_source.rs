//! Where a suggestion came from, and the words of a saved name that rest on the spelling rules
//! alone (Tech Stack 4.4 v1.2, PRD FR-7.66, E49, plan 05 Section 4).
#![cfg(test)]

use mandi_core::translit::{
    Direction, LearnedPair, Script, Source, WordList, english_spelling_source, rules_only_words,
    suggest, urdu_spelling_source,
};
use proptest::prelude::*;

fn list() -> &'static WordList {
    WordList::embedded()
}

fn learned(roman: &str, urdu: &str) -> LearnedPair {
    LearnedPair {
        roman: roman.to_owned(),
        urdu: urdu.to_owned(),
        use_count: 3,
    }
}

#[test]
fn listed_words_say_word_list_and_the_plan_example_has_no_rules_word() {
    // E2E-23: tariq mehmood -> طارق محمود, both words are in the word list.
    let got = suggest("tariq mehmood", &[], list(), Direction::ToUrdu);
    assert_eq!(got[0].text, "طارق محمود");
    assert_eq!(got[0].source, Source::WordList);
    // Every word (a phrase of the list is one word) is from the word list, and together they
    // make the text.
    assert!(got[0].words.iter().all(|w| w.source == Source::WordList));
    let joined: Vec<&str> = got[0].words.iter().map(|w| w.text.as_str()).collect();
    assert_eq!(joined.join(" "), got[0].text);
    // And back to English.
    let back = suggest("طارق محمود", &[], list(), Direction::ToEnglish);
    assert_eq!(back[0].text, "Tariq Mehmood");
    assert_eq!(back[0].source, Source::WordList);
}

#[test]
fn a_word_only_the_rules_know_is_marked_rules_and_marks_the_whole_name() {
    let got = suggest("arshad qzxwv", &[], list(), Direction::ToUrdu);
    assert!(!got.is_empty());
    let first = &got[0];
    assert!(first.text.starts_with("ارشد "), "{}", first.text);
    assert_eq!(first.words.len(), 2);
    assert_eq!(first.words[0].source, Source::WordList);
    assert_eq!(first.words[1].source, Source::Rules);
    assert_eq!(first.source, Source::Rules, "as weak as its weakest word");
}

#[test]
fn a_phrase_of_the_list_counts_as_one_word() {
    // `arshad multani` is a listed phrase (one unit), so it has one word with the source of
    // the whole phrase.
    let got = suggest("arshad multani", &[], list(), Direction::ToUrdu);
    assert_eq!(got[0].text, "ارشد ملتانی");
    assert_eq!(got[0].source, Source::WordList);
    assert!(got[0].words.iter().all(|w| w.source == Source::WordList));
}

#[test]
fn a_learned_pair_is_marked_learned_and_the_mix_is_as_strong_as_the_weakest() {
    let pairs = [learned("qzxwv", "قزکس")];
    let got = suggest("qzxwv", &pairs, list(), Direction::ToUrdu);
    assert_eq!(
        (got[0].text.as_str(), got[0].source),
        ("قزکس", Source::Learned)
    );
    let mixed = suggest("arshad qzxwv", &pairs, list(), Direction::ToUrdu);
    assert_eq!(mixed[0].words[0].source, Source::WordList);
    assert_eq!(mixed[0].words[1].source, Source::Learned);
    assert_eq!(mixed[0].source, Source::WordList);
    // The other direction.
    let back = suggest("قزکس", &pairs, list(), Direction::ToEnglish);
    assert_eq!(back[0].text, "Qzxwv");
    assert_eq!(back[0].source, Source::Learned);
}

#[test]
fn numbers_and_punctuation_are_not_words() {
    let got = suggest("ali 12 (arshad)", &[], list(), Direction::ToUrdu);
    assert_eq!(got[0].words.len(), 2, "{:?}", got[0]);
}

#[test]
fn every_suggestion_has_a_source_for_each_converted_word() {
    for text in [
        "ali",
        "arshad multani",
        "shahid",
        "banana",
        "zzzqq www",
        "a b c",
    ] {
        for direction in [Direction::ToUrdu, Direction::ToEnglish] {
            for s in suggest(text, &[], list(), direction) {
                assert!(!s.words.is_empty(), "{text} {direction:?} {s:?}");
                let weakest = s.words.iter().map(|w| w.source).min().unwrap();
                assert_eq!(s.source, weakest);
            }
        }
    }
}

#[test]
fn spelling_sources_follow_the_text_not_the_path() {
    let pairs = [learned("qzxwv", "قزکس")];
    assert_eq!(urdu_spelling_source("علی", &[], list()), Source::WordList);
    assert_eq!(
        urdu_spelling_source("قزکس", &pairs, list()),
        Source::Learned
    );
    assert_eq!(urdu_spelling_source("قزکس", &[], list()), Source::Rules);
    // Marks (zabar) do not make it another word.
    assert_eq!(urdu_spelling_source("عَلی", &[], list()), Source::WordList);
    assert_eq!(
        english_spelling_source("Ali", &[], list()),
        Source::WordList
    );
    assert_eq!(
        english_spelling_source("QZXWV", &pairs, list()),
        Source::Learned
    );
    assert_eq!(english_spelling_source("qzxwv", &[], list()), Source::Rules);
}

#[test]
fn rules_only_words_finds_the_words_the_list_and_the_learned_pairs_do_not_have() {
    // English name: Khichi is not in the list; Ali is.
    assert_eq!(
        rules_only_words("Ali Qzxwv", Script::English, &[], list()),
        ["Qzxwv"]
    );
    // The same word, learned: nothing left.
    let pairs = [learned("qzxwv", "قزکس")];
    assert!(rules_only_words("Ali Qzxwv", Script::English, &pairs, list()).is_empty());
    // Urdu name.
    assert_eq!(
        rules_only_words("علی قزکس", Script::Urdu, &[], list()),
        ["قزکس"]
    );
    assert!(rules_only_words("علی قزکس", Script::Urdu, &pairs, list()).is_empty());
    // A repeated word is reported once, in the order of the name.
    assert_eq!(
        rules_only_words("Qzxwv Ali Qzxwv Wvbnm", Script::English, &[], list()),
        ["Qzxwv", "Wvbnm"]
    );
    // A listed phrase covers its words even if one of them alone is not listed.
    assert!(rules_only_words("Arshad Multani", Script::English, &[], list()).is_empty());
    assert!(rules_only_words("ارشد ملتانی", Script::Urdu, &[], list()).is_empty());
}

#[test]
fn initials_digits_punctuation_and_the_other_script_are_not_checked() {
    assert!(rules_only_words("M. Ali", Script::English, &[], list()).is_empty());
    assert!(
        rules_only_words("Ali & Sons 2", Script::English, &[], list()).is_empty() || {
            // "Sons" may not be a listed word; the point is that "&" and "2" never appear.
            rules_only_words("Ali & Sons 2", Script::English, &[], list())
                .iter()
                .all(|w| w.chars().all(|c| c.is_ascii_alphabetic()))
        }
    );
    assert!(rules_only_words("", Script::English, &[], list()).is_empty());
    assert!(rules_only_words("   ", Script::Urdu, &[], list()).is_empty());
    // Urdu letters in an English name are not English words: they are the script check's job.
    assert!(rules_only_words("علی", Script::English, &[], list()).is_empty());
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(400))]

    /// Any text: no panic, every reported word comes from the text, none twice.
    #[test]
    fn rules_only_words_never_panics(text in "\\PC{0,120}") {
        for script in [Script::Urdu, Script::English] {
            let found = rules_only_words(&text, script, &[], list());
            let mut unique = found.clone();
            unique.sort();
            unique.dedup();
            prop_assert_eq!(unique.len(), found.len());
            for word in &found {
                prop_assert!(text.contains(word.as_str()));
            }
        }
    }

    /// Every suggestion of any text has a source for each of its words, and the whole is as
    /// weak as its weakest word.
    #[test]
    fn suggestions_always_carry_sources(text in "[a-z \u{0627}-\u{06D3}]{0,40}") {
        for direction in [Direction::ToUrdu, Direction::ToEnglish] {
            for s in suggest(&text, &[], list(), direction) {
                let weakest = s.words.iter().map(|w| w.source).min();
                prop_assert_eq!(Some(s.source), weakest);
            }
        }
    }
}

#[test]
fn a_pair_proposed_by_our_own_rules_is_not_known_until_it_is_reviewed() {
    // Lead decision 12-10-2026: a row whose Urdu only the rules proposed (source `engine`) stays
    // in the file for the reviewer, but the screen still marks it and the save still asks.
    let text = |review: &str| {
        format!(
            "roman\turdu\tkind\tweight\treview\tsource\nali\tعلی\tname\t100\tpending\tteam\nkaliari\tکالیاری\tplace\t100\t{review}\tengine"
        )
    };
    let pending = WordList::parse(&text("pending")).unwrap();
    assert_eq!(urdu_spelling_source("علی", &[], &pending), Source::WordList);
    assert_eq!(
        urdu_spelling_source("کالیاری", &[], &pending),
        Source::Rules
    );
    assert_eq!(
        english_spelling_source("Kaliari", &[], &pending),
        Source::Rules
    );
    assert_eq!(
        rules_only_words("Ali Kaliari", Script::English, &[], &pending),
        ["Kaliari"]
    );
    // The suggestion still offers it, marked.
    let got = suggest("kaliari", &[], &pending, Direction::ToUrdu);
    assert_eq!(
        (got[0].text.as_str(), got[0].source),
        ("کالیاری", Source::Rules)
    );
    // A reviewer marks it reviewed: now it is known.
    let reviewed = WordList::parse(&text("reviewed")).unwrap();
    assert_eq!(
        urdu_spelling_source("کالیاری", &[], &reviewed),
        Source::WordList
    );
    assert!(rules_only_words("Ali Kaliari", Script::English, &[], &reviewed).is_empty());
    // A learned pair (the user confirmed it) is known as well.
    let learned_pair = [learned("kaliari", "کالیاری")];
    assert_eq!(
        urdu_spelling_source("کالیاری", &learned_pair, &pending),
        Source::Learned
    );
}
