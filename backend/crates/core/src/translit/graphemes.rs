//! The letter groups of the spelling rules (Tech Stack 4.4, plan 05 Section 4).
//!
//! English letters are cut into groups by longest match (`kh`, `sh`, `aa`, `oo` ...). Each
//! group lists the Urdu spellings it can have. A short vowel that Urdu does not write is the
//! empty spelling. Which spelling is likely is NOT written here: it is counted from the word
//! list (see `model`), so the rules follow what the reviewed words really do.

/// A consonant or a vowel letter group.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Class {
    Consonant,
    Vowel,
}

/// One group of English letters and its possible Urdu spellings.
#[derive(Debug)]
pub(crate) struct Grapheme {
    /// The English letters, lower case.
    pub roman: &'static str,
    /// Every Urdu spelling this group can have. `""` is a short vowel that is not written.
    pub options: &'static [&'static str],
    /// Vowel or consonant.
    pub class: Class,
}

const fn g(roman: &'static str, options: &'static [&'static str], class: Class) -> Grapheme {
    Grapheme {
        roman,
        options,
        class,
    }
}

use Class::{Consonant as C, Vowel as V};

/// Every group, longest first inside each starting letter does not matter: the match is made
/// by length (see [`longest_match`]).
pub(crate) static GRAPHEMES: &[Grapheme] = &[
    // Vowels
    g("aa", &["ا", "آ", "عا", "ع", "اع"], V),
    g(
        "ai",
        &["ی", "ے", "ای", "ائی", "عی", "یا", "ئی", "ئے", "ائ", "ائے"],
        V,
    ),
    g("au", &["و", "او", "عو", "ؤ", "اؤ"], V),
    g("ae", &["ع", "ائ", "ی"], V),
    g("ee", &["ی", "ای", "ئی", "عی", "یع"], V),
    g("ei", &["ی", "ے"], V),
    g("ie", &["ی", "ے"], V),
    g("oo", &["و", "او", "ؤ", "عو", "وں"], V),
    g("ou", &["و", "او", "عو"], V),
    g(
        "a",
        &["", "ا", "ع", "آ", "ہ", "ے", "ی", "عا", "اء", "اع"],
        V,
    ),
    g("e", &["", "ی", "ے", "ا", "ع", "ای", "ہ", "اع", "عی"], V),
    g("i", &["", "ی", "ا", "ع", "ای", "ے", "یع", "اع", "ئی"], V),
    g("o", &["", "و", "او", "ا", "ع", "ؤ", "ہ", "وا"], V),
    g("u", &["", "و", "ا", "ع", "او", "ؤ", "وا"], V),
    // Consonant groups
    g("kh", &["خ", "کھ"], C),
    g("gh", &["غ", "گھ", "گ"], C),
    g("sh", &["ش", "سھ", "سح", "سہ", "صح"], C),
    g("ch", &["چ", "چھ", "ش", "ک"], C),
    g("ck", &["ک"], C),
    g("th", &["تھ", "ٹھ", "ث", "ت", "ذ", "ٹ", "تہ", "طہ", "ٹہ"], C),
    g("ph", &["ف", "پھ"], C),
    g("dh", &["دھ", "ڈھ", "ذ", "د"], C),
    g("bh", &["بھ"], C),
    g("jh", &["جھ"], C),
    g("zh", &["ژ", "ذ", "ز", "ظہ", "زہ", "ضہ", "ذہ"], C),
    g("rh", &["ڑھ", "رھ", "ر", "ڑ"], C),
    // Single consonants
    g("b", &["ب"], C),
    g("c", &["ک", "س", "چ", "ق"], C),
    g("d", &["د", "ڈ", "ض", "ذ"], C),
    g("f", &["ف"], C),
    g("g", &["گ", "غ", "ج"], C),
    g("h", &["ہ", "ح", "", "ھ"], C),
    g("j", &["ج", "ژ"], C),
    g("k", &["ک", "ق", "کھ"], C),
    g("l", &["ل"], C),
    g("m", &["م", "ن"], C),
    g("n", &["ن", "ں", "نگ"], C),
    g("p", &["پ"], C),
    g("q", &["ق", "ک"], C),
    g("r", &["ر", "ڑ"], C),
    g("s", &["س", "ص", "ث", "ش"], C),
    g("t", &["ت", "ٹ", "ط", "ۃ"], C),
    g("v", &["و", "ف"], C),
    g("w", &["و", "", "ؤ"], C),
    g("x", &["کس", "ز", "ایکس"], C),
    g("y", &["ی", "ے", "ئ", ""], C),
    g("z", &["ز", "ض", "ظ", "ذ", "ژ"], C),
];

/// The longest group that starts `rest`, with the rest of the text after it.
///
/// ```
/// // "shahid": the group is "sh", the rest "ahid".
/// ```
pub(crate) fn longest_match(rest: &str) -> Option<(usize, &'static Grapheme, &str)> {
    GRAPHEMES
        .iter()
        .enumerate()
        .filter_map(|(index, grapheme)| {
            rest.strip_prefix(grapheme.roman)
                .map(|after| (index, grapheme, after))
        })
        .max_by_key(|(_, grapheme, _)| grapheme.roman.len())
}

/// Where a group stands in the word. A one-group word counts as `Initial`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) enum Place {
    Initial,
    Medial,
    Final,
}

/// One group cut from a word, with what the model needs to know about its surroundings.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Piece {
    /// Position of the group in [`GRAPHEMES`].
    pub index: usize,
    pub grapheme: &'static Grapheme,
    pub place: Place,
    /// True if the group before is a vowel (or there is none).
    pub after_vowel: bool,
    /// True if this single consonant letter repeats the one before (`mm`, `ll`, `ss`).
    pub doubled: bool,
}

/// Cuts an English word (lower case letters only) into groups. `None` if a character is not
/// an English letter.
pub(crate) fn pieces(word: &str) -> Option<Vec<Piece>> {
    let mut rest = word;
    let mut found: Vec<(usize, &'static Grapheme)> = Vec::new();
    while !rest.is_empty() {
        let (index, grapheme, after) = longest_match(rest)?;
        found.push((index, grapheme));
        rest = after;
    }
    let count = found.len();
    let mut result = Vec::with_capacity(count);
    let mut previous: Option<&'static Grapheme> = None;
    for (position, (index, grapheme)) in found.into_iter().enumerate() {
        let place = if position == 0 {
            Place::Initial
        } else if position.saturating_add(1) == count {
            Place::Final
        } else {
            Place::Medial
        };
        let after_vowel = previous.is_none_or(|p| p.class == Class::Vowel);
        let doubled = grapheme.class == Class::Consonant
            && grapheme.roman.len() == 1
            && previous.is_some_and(|p| p.roman == grapheme.roman);
        result.push(Piece {
            index,
            grapheme,
            place,
            after_vowel,
            doubled,
        });
        previous = Some(grapheme);
    }
    Some(result)
}

/// The number that stands for "dropped" when the second letter of a doubled consonant is not
/// written. The groups do not list it, so it has no place in their `options`.
pub(crate) const DROPPED: usize = usize::MAX;

/// The Urdu spellings a piece may have in its place, each with its number in the group's
/// `options` (or [`DROPPED`]). A word cannot start with an unwritten vowel, and the second
/// letter of a doubled consonant may be dropped.
pub(crate) fn allowed_options(piece: &Piece) -> Vec<(usize, &'static str)> {
    let mut options: Vec<(usize, &'static str)> =
        piece.grapheme.options.iter().copied().enumerate().collect();
    if piece.place == Place::Initial && piece.grapheme.class == Class::Vowel {
        options.retain(|(_, option)| !option.is_empty());
    }
    if piece.doubled && !options.iter().any(|(_, option)| option.is_empty()) {
        options.insert(0, (DROPPED, ""));
    }
    options
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn groups_are_cut_by_longest_match() {
        let cut = |word: &str| -> Vec<&'static str> {
            pieces(word)
                .unwrap()
                .iter()
                .map(|p| p.grapheme.roman)
                .collect()
        };
        assert_eq!(cut("shahid"), ["sh", "a", "h", "i", "d"]);
        assert_eq!(cut("khan"), ["kh", "a", "n"]);
        assert_eq!(cut("mehmood"), ["m", "e", "h", "m", "oo", "d"]);
        assert_eq!(cut("aamir"), ["aa", "m", "i", "r"]);
        assert_eq!(cut("hussain"), ["h", "u", "s", "s", "ai", "n"]);
    }

    #[test]
    fn places_and_doubles() {
        let list = pieces("hussain").unwrap();
        assert_eq!(list[0].place, Place::Initial);
        assert_eq!(list[6 - 1].place, Place::Final);
        assert!(list[3].doubled && !list[2].doubled);
        assert!(allowed_options(&list[3]).iter().any(|(_, o)| o.is_empty()));
        // An initial vowel cannot be unwritten.
        let first = pieces("ali").unwrap();
        assert!(!allowed_options(&first[0]).iter().any(|(_, o)| o.is_empty()));
    }

    #[test]
    fn a_non_letter_is_refused() {
        assert!(pieces("ali1").is_none());
        assert!(pieces("é").is_none());
    }
}
