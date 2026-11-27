//! Words people type as one: `krdo` is کر دو, `dukanpe` is دکان پہ, `sabziwala` is سبزی والا
//! (plan 99, NFR-URD-30). Each ending has its own Urdu, so it never turns into another word.
//! Only used in free text, for a word the list does not have. Pure and safe on any input.

use std::collections::HashMap;
use std::sync::OnceLock;

use super::wordlist::{Field, WordList};

/// The English spellings of loanwords, the same file the generator reads.
const ENGLISH_STEMS: &str = include_str!("../../data/everyday/english_stems.txt");
/// An English word with an Urdu plural ending: the typed ending, what it replaced at the end
/// of the English word (`emergenc` + `iyan` was `emergency`), and the Urdu ending.
const ENGLISH_PLURALS: [(&str, &str, &str); 12] = [
    ("iyon", "y", "وں"),
    ("iyan", "y", "یں"),
    ("ion", "y", "وں"),
    ("ian", "y", "یں"),
    ("ein", "", "یں"),
    ("ain", "", "یں"),
    ("on", "", "وں"),
    ("en", "", "یں"),
    // An English word ending in `a`: `cameray`, `cameron`.
    ("on", "a", "وں"),
    ("ay", "a", "ے"),
    ("ey", "a", "ے"),
    ("e", "a", "ے"),
];

/// If the word is an English loanword of the list: true when it is feminine in Urdu.
fn english_stem(word: &str) -> Option<bool> {
    static STEMS: OnceLock<HashMap<&'static str, bool>> = OnceLock::new();
    STEMS
        .get_or_init(|| {
            ENGLISH_STEMS
                .lines()
                .filter(|l| !l.starts_with('#'))
                .filter_map(|l| {
                    let mut parts = l.split_whitespace();
                    let stem = parts.next()?;
                    Some((stem, parts.next() == Some("f")))
                })
                .collect()
        })
        .get(word)
        .copied()
}

/// The Urdu plural before a postposition (`وں`), the plural of a word ending in ہ or ا
/// (`ے`), or the feminine plural (`یں`).
fn plural_of(urdu: &str, ending: &str) -> Option<String> {
    let after_he_or_alif = urdu.strip_suffix('ہ').or_else(|| urdu.strip_suffix('ا'));
    match ending {
        "وں" => Some(format!("{}{ending}", after_he_or_alif.unwrap_or(urdu))),
        "ے" => after_he_or_alif.map(|stem| format!("{stem}{ending}")),
        // ایمرجنسی becomes ایمرجنسیاں, کیبل becomes کیبلیں.
        _ if urdu.ends_with('ی') => Some(format!("{urdu}اں")),
        _ => Some(format!("{urdu}{ending}")),
    }
}

/// `doctoron` is ڈاکٹروں. The dictionary keeps no such mixed spellings (checkers, plan 99),
/// so they are made here. A `وں` or `ے` plural suits any such noun. A `یں` plural is only made
/// for a word marked feminine, or when the list has it, so no made-up plural comes out.
pub(crate) fn english_plural(list: &WordList, word: &str) -> Option<String> {
    ENGLISH_PLURALS.iter().find_map(|(typed, back, ending)| {
        let stem = format!("{}{back}", word.strip_suffix(typed)?);
        let feminine = english_stem(&stem)?;
        let head = listed_urdu(list, &stem)?;
        let plural = plural_of(&head, ending)?;
        (*ending != "یں" || feminine || list.contains_known_urdu(&plural)).then_some(plural)
    })
}

/// Little words typed glued to the word before, the longest first, with their Urdu.
const ENDINGS: [(&str, &str); 93] = [
    ("waalon", "والوں"),
    ("walon", "والوں"),
    ("waala", "والا"),
    ("walay", "والے"),
    ("waali", "والی"),
    ("jaiye", "جائیے"),
    ("jayen", "جائیں"),
    ("jaein", "جائیں"),
    ("karna", "کرنا"),
    ("karni", "کرنی"),
    ("karne", "کرنے"),
    ("karta", "کرتا"),
    ("karti", "کرتی"),
    ("karte", "کرتے"),
    ("wala", "والا"),
    ("wale", "والے"),
    ("waly", "والے"),
    ("wali", "والی"),
    ("jaen", "جائیں"),
    ("jaye", "جائے"),
    ("jata", "جاتا"),
    ("jati", "جاتی"),
    ("jate", "جاتے"),
    ("jana", "جانا"),
    ("jaun", "جاؤں"),
    ("gaya", "گیا"),
    ("gayi", "گئی"),
    ("gaye", "گئے"),
    ("raha", "رہا"),
    ("rahi", "رہی"),
    ("rahe", "رہے"),
    ("diya", "دیا"),
    ("diye", "دیے"),
    ("dein", "دیں"),
    ("liya", "لیا"),
    ("liye", "لیے"),
    ("lein", "لیں"),
    ("krna", "کرنا"),
    ("krni", "کرنی"),
    ("krne", "کرنے"),
    ("krta", "کرتا"),
    ("krti", "کرتی"),
    ("krte", "کرتے"),
    ("karo", "کرو"),
    ("hain", "ہیں"),
    ("mein", "میں"),
    ("jae", "جائے"),
    ("jao", "جاؤ"),
    ("gya", "گیا"),
    ("gia", "گیا"),
    ("gyi", "گئی"),
    ("gai", "گئی"),
    ("gye", "گئے"),
    ("gae", "گئے"),
    ("rha", "رہا"),
    ("rhi", "رہی"),
    ("rhe", "رہے"),
    ("dya", "دیا"),
    ("dia", "دیا"),
    ("den", "دیں"),
    ("lia", "لیا"),
    ("kro", "کرو"),
    ("kar", "کر"),
    ("hai", "ہے"),
    ("tha", "تھا"),
    ("thi", "تھی"),
    ("thy", "تھے"),
    ("the", "تھے"),
    ("par", "پر"),
    ("wal", "وال"),
    ("kr", "کر"),
    ("do", "دو"),
    ("di", "دی"),
    ("de", "دے"),
    ("li", "لی"),
    ("lo", "لو"),
    ("le", "لے"),
    ("pe", "پہ"),
    ("pr", "پر"),
    ("se", "سے"),
    ("ka", "کا"),
    ("ki", "کی"),
    ("ko", "کو"),
    ("ke", "کے"),
    ("hn", "ہیں"),
    ("hy", "ہے"),
    ("ga", "گا"),
    ("gi", "گی"),
    ("ge", "گے"),
    ("ja", "جا"),
    ("ya", "یا"),
    ("na", "نا"),
    ("me", "میں"),
];

/// Short words that take an ending: verb stems people join with their helper (`ho` + `gya`).
const SHORT_STEMS: [&str; 10] = ["a", "aa", "ho", "ja", "de", "le", "kr", "so", "ro", "pi"];
/// The verb helpers that follow a short stem. Postpositions are left out, so a real word such
/// as `deka` (دیکھا) is not cut into دے کا.
const HELPERS: [&str; 41] = [
    "gaya", "gayi", "gaye", "gya", "gia", "gyi", "gai", "gye", "gae", "raha", "rahi", "rahe",
    "rha", "rhi", "rhe", "jao", "jaen", "jayen", "jaein", "jaye", "jae", "jaiye", "jata", "jati",
    "jate", "jaun", "diya", "dya", "dia", "diye", "dein", "den", "do", "liya", "lia", "liye",
    "lein", "lo", "karo", "kro", "kr",
];
/// `kr` starts no other word, so any ending after it is sure.
const ALWAYS_SPLIT_STEM: &str = "kr";
/// Any other word needs this many letters before an ending is split from it.
const MIN_STEM_LETTERS: usize = 3;
/// A close (not exact) stem needs this many letters.
const MIN_CLOSE_STEM_LETTERS: usize = 4;

/// Every way to split a known ending from the word, the longest ending first.
fn cuts(word: &str) -> impl Iterator<Item = (&str, &'static str)> {
    ENDINGS
        .iter()
        .filter_map(move |(ending, urdu)| word.strip_suffix(ending).map(|stem| (stem, *urdu)))
}

/// The first spelling the list has for exactly this word in free text.
fn listed_urdu(list: &WordList, word: &str) -> Option<String> {
    list.ranked_by_roman(word, Field::Text)
        .first()
        .map(|(e, _)| e.urdu.clone())
}

/// A short verb stem with its helper: `ho` + `gya` is ہو گیا, `a` + `jao` is آ جاؤ.
pub(crate) fn sure_split(list: &WordList, word: &str) -> Option<String> {
    ENDINGS
        .iter()
        .filter_map(|(ending, urdu)| word.strip_suffix(ending).map(|stem| (stem, *ending, *urdu)))
        .filter(|(stem, ending, _)| {
            *stem == ALWAYS_SPLIT_STEM || (SHORT_STEMS.contains(stem) && HELPERS.contains(ending))
        })
        .find_map(|(stem, _, tail)| listed_urdu(list, stem).map(|head| format!("{head} {tail}")))
}

/// A listed word of three letters or more with an ending: `dukan` + `pe`.
pub(crate) fn split(list: &WordList, word: &str) -> Option<String> {
    cuts(word)
        .filter(|(stem, _)| stem.chars().count() >= MIN_STEM_LETTERS)
        .find_map(|(stem, tail)| listed_urdu(list, stem).map(|head| format!("{head} {tail}")))
}

/// A word typed a little differently with an ending: `sabzii` + `wala`.
pub(crate) fn split_close_stem(list: &WordList, word: &str) -> Option<String> {
    cuts(word)
        .filter(|(stem, _)| stem.chars().count() >= MIN_CLOSE_STEM_LETTERS)
        .find_map(|(stem, tail)| {
            list.typed_like(stem, false)
                .first()
                .map(|(head, _)| format!("{head} {tail}"))
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn words_typed_as_one_are_split_with_their_own_urdu() {
        let list = WordList::embedded();
        assert_eq!(sure_split(list, "hogyi"), Some("ہو گئی".to_owned()));
        assert_eq!(sure_split(list, "ajao"), Some("آ جاؤ".to_owned()));
        assert_eq!(split(list, "dukanpe"), Some("دکان پہ".to_owned()));
        assert_eq!(sure_split(list, ""), None);
    }

    #[test]
    fn an_english_word_takes_an_urdu_plural() {
        let list = WordList::embedded();
        assert_eq!(english_plural(list, "doctoron"), Some("ڈاکٹروں".to_owned()));
        assert_eq!(
            english_plural(list, "emergenciyon"),
            Some("ایمرجنسیوں".to_owned())
        );
        // No made-up feminine plural: سروسیں is not a word of the list.
        assert_eq!(english_plural(list, "servicein"), None);
        assert_eq!(english_plural(list, "karon"), None);
        assert_eq!(english_plural(list, "cameray"), Some("کیمرے".to_owned()));
        assert_eq!(english_plural(list, "cameron"), Some("کیمروں".to_owned()));
        assert_eq!(english_plural(list, "wicketein"), Some("وکٹیں".to_owned()));
    }

    #[test]
    fn endings_come_longest_first() {
        assert!(ENDINGS.windows(2).all(|w| match w {
            [(a, _), (b, _)] => a.len() >= b.len(),
            _ => true,
        }));
    }
}
