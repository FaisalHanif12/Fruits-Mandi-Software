//! What the transliteration engine promises (PRD FR-7.22, FR-7.24, FR-7.26, plan 05).
#![cfg(test)]

use mandi_core::translit::{
    LearnedPair, MAX_INPUT_CHARS, MAX_SUGGESTIONS, WordList, suggest_to_english, suggest_to_urdu,
};

fn list() -> &'static WordList {
    WordList::embedded()
}

fn learned(roman: &str, urdu: &str, use_count: u32) -> LearnedPair {
    LearnedPair {
        roman: roman.to_owned(),
        urdu: urdu.to_owned(),
        use_count,
    }
}

fn urdu(text: &str) -> Vec<String> {
    suggest_to_urdu(text, &[], list())
}

fn english(text: &str) -> Vec<String> {
    suggest_to_english(text, &[], list())
}

#[test]
fn the_plan_examples_convert() {
    // E2E-23: the new buyer.
    assert_eq!(urdu("tariq mehmood")[0], "طارق محمود");
    assert_eq!(english("طارق محمود")[0], "Tariq Mehmood");
    // PRD FR-7.22: a whole phrase, word by word.
    assert_eq!(urdu("arshad multani")[0], "ارشد ملتانی");
    assert_eq!(english("ارشد ملتانی")[0], "Arshad Multani");
    // PRD 4.10: the names of the worked day.
    assert_eq!(english("اکرم ٹول ٹیکس")[0], "Akram Toll Tax");
    assert_eq!(english("رحیم")[0], "Rahim");
    assert!(english("کیلا").contains(&"Banana".to_owned()));
}

#[test]
fn several_spellings_are_offered_with_the_usual_one_first() {
    // NFR section 0.1: "Shahid" can be شاہد or شاھد.
    let got = urdu("shahid");
    assert_eq!(&got[..2], ["شاہد", "شاھد"]);
    assert!(got.len() <= MAX_SUGGESTIONS);
    // A small spelling mistake still finds the name.
    assert_eq!(urdu("tarq")[0], "طارق");
    assert_eq!(urdu("arshd")[0], "ارشد");
}

#[test]
fn words_not_in_the_list_are_spelled_by_the_rules() {
    let got = urdu("zxqvj");
    assert!(!got.is_empty() && got.len() <= MAX_SUGGESTIONS, "{got:?}");
    // A name that is not in the list but whose parts are.
    assert_eq!(urdu("abdulsamad")[0], "عبدالصمد");
    assert_eq!(urdu("noorulhaq")[0], "نور الحق");
    assert_eq!(urdu("noor ul haq")[0], "نور الحق");
    // Words typed together or apart.
    assert!(urdu("ghulamrasool").contains(&"غلام رسول".to_owned()));
    assert!(urdu("kanganpur").iter().any(|s| s.ends_with("پور")));
}

#[test]
fn urdu_to_english_has_capital_letters_and_no_marks_problem() {
    assert_eq!(english("مصطفی")[0], "Mustafa");
    assert_eq!(
        english("مصطفیٰ")[0],
        "Mustafa",
        "khari zabar does not change the word"
    );
    let got = english("الرحمن");
    assert!(got[0].starts_with("Ur "), "{got:?}");
}

#[test]
fn learned_pairs_come_first_and_the_most_used_first() {
    let pairs = [learned("xyz", "زیڈ", 1)];
    let got = suggest_to_urdu("xyz", &pairs, list());
    assert_eq!(got[0], "زیڈ");
    // The rules still give their spellings after it.
    assert!(got.len() > 1);
    // A learned pair beats the word list: Shahid with the second spelling.
    let pairs = [learned("shahid", "شاھد", 5)];
    let got = suggest_to_urdu("shahid", &pairs, list());
    assert_eq!(&got[..2], ["شاھد", "شاہد"]);
    // Two learned pairs for one word: the one chosen more often is first.
    let pairs = [
        learned("zed", "زڈ", 2),
        learned("zed", "زیڈ", 9),
        learned("zed", "زیڈا", 2),
    ];
    let got = suggest_to_urdu("zed", &pairs, list());
    assert_eq!(&got[..3], ["زیڈ", "زڈ", "زیڈا"]);
    // Letters case does not matter, and a learned word works inside a phrase.
    let got = suggest_to_urdu("Arshad XYZ", &[learned("xyz", "زیڈ", 1)], list());
    assert_eq!(got[0], "ارشد زیڈ");
}

#[test]
fn learned_pairs_work_the_other_way_too() {
    let pairs = [learned("zed", "زیڈ", 1)];
    let got = suggest_to_english("زیڈ", &pairs, list());
    assert_eq!(got[0], "Zed");
    let got = suggest_to_english("ارشد زیڈ", &pairs, list());
    assert_eq!(got[0], "Arshad Zed");
}

#[test]
fn numbers_punctuation_and_other_scripts_are_kept_as_typed() {
    assert_eq!(urdu("arshad 2")[0], "ارشد 2");
    assert_eq!(urdu("arshad-multani")[0], "ارشد-ملتانی");
    assert_eq!(urdu("(arshad)")[0], "(ارشد)");
    // Urdu already typed in an English-letters field stays.
    assert_eq!(urdu("ارشد arshad")[0], "ارشد ارشد");
    assert_eq!(english("ارشد 2")[0], "Arshad 2");
    // Nothing to convert gives nothing.
    assert!(urdu("123").is_empty());
    assert!(urdu("").is_empty());
    assert!(english("   ").is_empty());
}

#[test]
fn white_space_is_one_space() {
    assert_eq!(urdu("  arshad \t\n multani ")[0], "ارشد ملتانی");
}

#[test]
fn it_never_gives_more_than_three_or_the_same_twice() {
    for text in [
        "a",
        "ali",
        "muhammad",
        "bhai pheru",
        "kot radha kishan",
        "zzzzzz",
        "x y z",
        "q",
    ] {
        for got in [urdu(text), english(text)] {
            assert!(got.len() <= MAX_SUGGESTIONS, "{text}: {got:?}");
            let mut unique = got.clone();
            unique.sort();
            unique.dedup();
            assert_eq!(unique.len(), got.len(), "{text}: {got:?}");
            assert!(got.iter().all(|s| !s.is_empty()));
        }
    }
}

#[test]
fn the_same_input_always_gives_the_same_suggestions() {
    for text in [
        "arshad",
        "rajoa",
        "fazlurrehman",
        "chak lala musa",
        "ghazanfar ali",
    ] {
        assert_eq!(urdu(text), urdu(text));
    }
    // Two lists built from the same text give the same answers (no hash-order dependence).
    let a = WordList::parse(include_str!("../data/translit_seed.tsv")).unwrap();
    let b = WordList::parse(include_str!("../data/translit_seed.tsv")).unwrap();
    for text in ["rajoa", "zeeshan haider", "ahtesham", "mehrabpur"] {
        assert_eq!(
            suggest_to_urdu(text, &[], &a),
            suggest_to_urdu(text, &[], &b),
            "{text}"
        );
        assert_eq!(
            suggest_to_english(&suggest_to_urdu(text, &[], &a)[0], &[], &a),
            suggest_to_english(&suggest_to_urdu(text, &[], &b)[0], &[], &b),
            "{text}"
        );
    }
}

#[test]
fn only_the_first_part_of_a_long_text_is_read() {
    let long = "arshad ".repeat(200);
    let got = urdu(&long);
    assert!(!got.is_empty());
    assert!(
        got[0].chars().count() <= MAX_INPUT_CHARS * 2,
        "kept to a sane size"
    );
    // Words beyond the limit are kept as typed, never lost or crashing.
    let many = (0..40).map(|_| "ali").collect::<Vec<_>>().join(" ");
    assert!(!urdu(&many).is_empty());
}

#[test]
fn a_small_list_works_and_an_empty_list_still_gives_rule_spellings() {
    let empty = WordList::parse("roman\turdu\tkind\tweight\treview").unwrap();
    assert!(empty.is_empty());
    let got = suggest_to_urdu("ali", &[], &empty);
    assert!(got.len() <= MAX_SUGGESTIONS);
    let got = suggest_to_english("علی", &[], &empty);
    assert!(got.len() <= MAX_SUGGESTIONS);
    let one =
        WordList::parse("roman\turdu\tkind\tweight\treview\nali\tعلی\tname\t100\tpending").unwrap();
    assert_eq!(suggest_to_urdu("ali", &[], &one)[0], "علی");
    assert_eq!(suggest_to_english("علی", &[], &one)[0], "Ali");
}

#[test]
fn a_word_whose_letters_are_all_unwritten_gives_no_empty_suggestion() {
    // Found by the property test: a lone `h` has a rule spelling of no letters at all.
    for text in ["h", "hh", "a", "ee", "y"] {
        assert!(urdu(text).iter().all(|s| !s.is_empty()), "{text}");
        assert!(english(text).iter().all(|s| !s.is_empty()), "{text}");
    }
}

#[test]
fn urdu_words_made_of_known_parts_are_put_together() {
    // A joining `ال` before a name that is not in the list: the joining word follows the sound.
    assert!(
        english("الفلاح")[0].starts_with("Ul "),
        "{:?}",
        english("الفلاح")
    );
    assert!(
        english("الصبور")[0].starts_with("Us "),
        "{:?}",
        english("الصبور")
    );
    assert!(
        english("الدلاور")[0].starts_with("Ud "),
        "{:?}",
        english("الدلاور")
    );
    assert!(
        english("النصیر")[0].starts_with("Un "),
        "{:?}",
        english("النصیر")
    );
    // Abdul-names written as one word.
    assert!(
        english("عبدالجلیل")[0].starts_with("Abdul"),
        "{:?}",
        english("عبدالجلیل")
    );
    // `ullah` at the end.
    assert!(
        english("عزیزاللہ")[0].to_lowercase().contains("ullah"),
        "{:?}",
        english("عزیزاللہ")
    );
    // A place ending and two known words written together.
    assert!(
        english("جلالپور").contains(&"Jalalpur".to_owned()),
        "{:?}",
        english("جلالپور")
    );
    assert!(
        english("غلامرسول")[0].starts_with("Ghulam "),
        "{:?}",
        english("غلامرسول")
    );
}

#[test]
fn a_joining_word_and_abdul_join_the_name_after_them() {
    assert!(
        urdu("abdul falah")[0].starts_with("عبدال"),
        "{:?}",
        urdu("abdul falah")
    );
    assert!(
        urdu("naseem ul falah")[0].starts_with("نسیم ال"),
        "{:?}",
        urdu("naseem ul falah")
    );
    // Alone, or first in the text, they are ordinary words.
    assert!(!urdu("ul").is_empty());
    assert_eq!(urdu("abdul")[0], "عبد");
}

#[test]
fn kinds_and_the_empty_list() {
    use mandi_core::translit::Kind;
    for kind in [Kind::Name, Kind::Place, Kind::Item, Kind::Word] {
        assert_eq!(Kind::parse(kind.as_str()), Some(kind));
    }
    assert_eq!(Kind::parse("colour"), None);
    let empty = WordList::empty();
    assert!(empty.is_empty() && empty.entries().is_empty());
    assert_eq!(empty.len(), 0);
    assert!(empty.unlined_pairs().is_empty());
    assert_eq!(empty.training_report(), (0, 0));
}
