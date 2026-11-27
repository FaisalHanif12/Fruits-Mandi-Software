//! Other ways of typing a word we know (plan 99, NFR-URD-30).
//!
//! People type Roman Urdu in many ways: `acha` and `accha`, `bachay` and `bache`, `kiya` and
//! `kia`, or two words as one (`aarahe`). When a typed word is not in the list, these keys find
//! the words that are typed almost the same way. The dictionary is only used in free text
//! (name fields keep the name list and the rules).
//!
//! - [`loose_key`] removes the common differences: doubled letters, long vowels, `ay`/`ai`/`ae`,
//!   `q`/`k`, `w`/`v`, `ph`/`f`, a silent final `h` and the like.
//! - [`skeleton_key`] also drops the vowels, for hurried words such as `mjhy`.
//!
//! [`other_spellings`] uses them, and the [`super::glued`] splits for words typed as one,
//! such as `krdo` or `sabziwala`.

use std::collections::HashSet;

use super::glued;
use super::wordlist::WordList;

/// A vowel as typed in Roman Urdu.
fn is_vowel(c: char) -> bool {
    matches!(c, 'a' | 'e' | 'i' | 'o' | 'u')
}

/// Sound spellings that people mix, longest first. Applied once, left to right.
const SAME_SOUND: [(&str, &str); 18] = [
    // چ is doubled as `cch` or `chch`.
    ("chch", "ch"),
    ("cch", "ch"),
    // `chalaunga` ends in اؤں, so its `au` is not `o`.
    ("aun", "aun"),
    ("aey", "e"),
    ("aay", "e"),
    ("aye", "e"),
    ("aae", "e"),
    ("ae", "e"),
    ("ai", "e"),
    ("ei", "e"),
    ("ee", "i"),
    ("ii", "i"),
    ("oo", "u"),
    ("uu", "u"),
    ("au", "o"),
    ("ou", "o"),
    ("ph", "f"),
    ("ck", "k"),
];

/// The word with the same sounds written one way.
fn same_sounds(word: &str) -> String {
    let mut out = String::with_capacity(word.len());
    let mut rest = word;
    'outer: while !rest.is_empty() {
        for (from, to) in SAME_SOUND {
            if let Some(next) = rest.strip_prefix(from) {
                out.push_str(to);
                rest = next;
                continue 'outer;
            }
        }
        let mut chars = rest.chars();
        if let Some(c) = chars.next() {
            out.push(match c {
                'q' => 'k',
                'w' => 'v',
                other => other,
            });
        }
        rest = chars.as_str();
    }
    out
}

/// `y`, `c` and a final `h`, which depend on the letters around them.
fn by_position(word: &str) -> String {
    let chars: Vec<char> = word.chars().collect();
    let mut out = String::with_capacity(word.len());
    for (i, &c) in chars.iter().enumerate() {
        let before = i.checked_sub(1).and_then(|j| chars.get(j)).copied();
        let next = chars.get(i.saturating_add(1)).copied();
        let after_consonant = before.is_some_and(|b| !is_vowel(b));
        let after_vowel = before.is_some_and(is_vowel);
        let next_vowel = next.is_some_and(is_vowel);
        match c {
            // `c` that is not `ch` is the sound of `k`.
            'c' if next != Some('h') => out.push('k'),
            // `kya` is `kia`.
            'y' if after_consonant && next_vowel => out.push('i'),
            // A final `y` after a consonant is ے (`mery`).
            'y' if after_consonant && next.is_none() => out.push('e'),
            // `ay` or `ey` before a consonant or at the end is `e`.
            'y' if matches!(before, Some('a' | 'e')) && !next_vowel => {
                out.pop();
                out.push('e');
            }
            // `biyopari` is `biopari`.
            'y' if before == Some('i') && next_vowel => {}
            // A silent final `h`: `wajah` is `waja`.
            'h' if next.is_none() && after_vowel => {}
            // غ is typed `g` or `gh`.
            'h' if before == Some('g') => {}
            // ڑھ is typed `rh`, `dh` or just `r`, but never starts a word.
            'd' if i > 0 && next == Some('h') => out.push('r'),
            'h' if i > 1 && matches!(before, Some('r' | 'd')) => {}
            other => out.push(other),
        }
    }
    out
}

/// Doubled letters written once (`pakka` is `paka`, `abb` is `ab`).
fn undoubled(word: &str) -> String {
    let mut out = String::with_capacity(word.len());
    for c in word.chars() {
        if !out.ends_with(c) {
            out.push(c);
        }
    }
    out
}

/// The key that spellings of the same word share. Spaces are dropped, so two words typed as one
/// find their phrase.
pub(crate) fn loose_key(roman: &str) -> String {
    let joined: String = roman
        .chars()
        .filter(char::is_ascii_alphabetic)
        .map(|c| c.to_ascii_lowercase())
        .collect();
    undoubled(&by_position(&same_sounds(&joined)))
}

/// The first letter and the consonants after it. Every later `h` is dropped too, so `kidr`
/// and `kidhar`, or `bht` and `bohat`, meet.
pub(crate) fn skeleton_key(roman: &str) -> String {
    let joined: String = roman
        .chars()
        .filter(char::is_ascii_alphabetic)
        .map(|c| c.to_ascii_lowercase())
        .collect();
    let sounds: Vec<char> = same_sounds(&joined).chars().collect();
    let kept: String = sounds
        .iter()
        .enumerate()
        .map(|(i, &c)| match c {
            'c' if sounds.get(i.saturating_add(1)) != Some(&'h') => 'k',
            other => other,
        })
        .enumerate()
        .filter(|&(i, c)| i == 0 || !(is_vowel(c) || matches!(c, 'y' | 'h')))
        .map(|(_, c)| c)
        .collect();
    undoubled(&kept)
}

/// True for a word typed with few vowels, such as `mjhy` or `kidr`: at most one in every
/// three letters.
pub(crate) fn is_hurried(roman: &str) -> bool {
    let letters = roman.chars().count();
    let vowels = roman.chars().filter(|c| is_vowel(*c)).count();
    vowels <= letters / 3
}

/// The edit distance between two spellings: letters added, removed or changed.
pub(crate) fn distance(a: &str, b: &str) -> usize {
    let b: Vec<char> = b.chars().collect();
    let mut row: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.chars().enumerate() {
        let mut diagonal = i;
        let mut left = i.saturating_add(1);
        if let Some(first) = row.first_mut() {
            *first = left;
        }
        for (cb, cell) in b.iter().zip(row.iter_mut().skip(1)) {
            let up = *cell;
            let change = diagonal.saturating_add(usize::from(ca != *cb));
            let best = change.min(up.saturating_add(1)).min(left.saturating_add(1));
            diagonal = up;
            *cell = best;
            left = best;
        }
    }
    row.last().copied().unwrap_or(0)
}

/// A hurried spelling is only looked up by its skeleton from this many letters.
const MIN_SKELETON_LETTERS: usize = 3;
/// A word found by its skeleton counts a little less than a split with a known ending.
const SKELETON_SHARE: f64 = 0.9;

/// Urdu words the list knows that are typed almost like `word`, the best first, each with a
/// score from 0 to 1. `listed` is true when the list has the typed spelling itself; then only
/// close spellings are offered. Used only in free text. Pure and safe on any input.
pub(crate) fn other_spellings(list: &WordList, word: &str, listed: bool) -> Vec<(String, f64)> {
    if listed {
        return scored(list.typed_like(word, false));
    }
    // A short word we know with a common ending (`ho` + `gyi`) is surer than any close
    // spelling.
    if let Some(split) = glued::sure_split(list, word) {
        return vec![(split, 1.0)];
    }
    // `doctoron`: an English word with an Urdu plural ending.
    if let Some(plural) = glued::english_plural(list, word) {
        return vec![(plural, 1.0)];
    }
    let close = list.typed_like(word, false);
    if !close.is_empty() {
        return scored(close);
    }
    let mut made: Vec<(String, f64)> = glued::split(list, word)
        .into_iter()
        .map(|text| (text, 1.0))
        .collect();
    if is_hurried(word) && word.chars().count() >= MIN_SKELETON_LETTERS {
        made.extend(
            scored(list.typed_like(word, true))
                .into_iter()
                .map(|(text, score)| (text, SKELETON_SHARE * score)),
        );
    }
    if made.is_empty() {
        made.extend(
            glued::split_close_stem(list, word)
                .into_iter()
                .map(|text| (text, 1.0)),
        );
    }
    let mut seen = HashSet::new();
    made.retain(|(text, _)| seen.insert(text.clone()));
    made
}

/// The scores of found words, the first at 1.
fn scored(found: Vec<(&str, f64)>) -> Vec<(String, f64)> {
    let best = found
        .first()
        .map_or(1.0, |(_, s)| *s)
        .max(f64::MIN_POSITIVE);
    found
        .into_iter()
        .map(|(urdu, score)| (urdu.to_owned(), score / best))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spellings_of_one_word_share_a_loose_key() {
        for (a, b) in [
            ("acha", "accha"),
            ("kya", "kia"),
            ("bachay", "bache"),
            ("badalwaye ga", "badalwaega"),
            ("wajah", "waja"),
            ("qeemat", "kimat"),
            ("phal", "fal"),
            ("biyopari", "biopari"),
            ("aa rahe", "aarahe"),
            ("badho", "barho"),
            ("barho", "baro"),
            ("ghalti", "galti"),
        ] {
            assert_eq!(loose_key(a), loose_key(b), "{a} and {b}");
        }
        assert_ne!(loose_key("dil"), loose_key("dal"));
    }

    #[test]
    fn a_hurried_word_keeps_its_consonants() {
        assert!(is_hurried("mjhy"));
        assert!(is_hurried("kr"));
        assert!(!is_hurried("mujhe"));
        assert_eq!(skeleton_key("mujhe"), skeleton_key("mjhy"));
    }

    #[test]
    fn distance_counts_edits() {
        assert_eq!(distance("", "abc"), 3);
        assert_eq!(distance("kia", "kia"), 0);
        assert_eq!(distance("kya", "kia"), 1);
        assert_eq!(distance("accha", "acha"), 1);
    }
}
