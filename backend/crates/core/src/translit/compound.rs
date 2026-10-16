//! Words that are not in the list but are made of parts that are (plan 05 Section 4).
//!
//! Names like `ataullah`, `fazlurrehman`, `abdulsamad` or `kanganpur` are rarely in a word
//! list, but their parts are: a name, a small joining word (`ul`, `ur`, `ud`), a place ending
//! (`pur`, `abad`). Each part is spelled from the list (or by the rules), and the parts are
//! put together the way Urdu writes them. Several ways of joining are offered, because Urdu
//! writes some of them with a space and some without.

use super::forward;
use super::model::Model;
use super::reverse;
use super::wordlist::WordList;

/// The little words that mean "the" before a name: `fazl ur rehman`, `noor ul haq`.
pub(crate) const PARTICLES: [&str; 6] = ["ul", "ur", "ud", "us", "un", "uz"];
/// Forms of "Abdul" that join the next name: `abdur rehman`, `abdus samad`.
pub(crate) const ABDUL: [&str; 4] = ["abdul", "abdur", "abdus", "abdun"];
/// "Abdul" in Urdu, with the joining `ال`.
pub(crate) const ABDUL_URDU: &str = "عبدال";
/// The joining `ال` of a particle.
pub(crate) const AL_URDU: &str = "ال";
/// Place endings, English and Urdu.
const PLACE_ENDINGS: [(&str, &str); 14] = [
    ("pur", "پور"),
    ("abad", "آباد"),
    ("wala", "والا"),
    ("wali", "والی"),
    ("garh", "گڑھ"),
    ("nagar", "نگر"),
    ("ganj", "گنج"),
    ("kot", "کوٹ"),
    ("pura", "پورہ"),
    ("zai", "زئی"),
    ("khel", "خیل"),
    ("khurd", "خورد"),
    ("kalan", "کلاں"),
    ("bagh", "باغ"),
];
/// `ullah` in Urdu and English.
const ALLAH_URDU: &str = "اللہ";
const ALLAH_ROMAN: &str = "ullah";

/// Shortest word the parts are looked for in.
const MIN_LETTERS: usize = 5;
/// Compound spellings kept for one word.
const KEEP: usize = 3;

/// A spelling and its score.
pub(crate) type Scored = (String, f64);

/// The few best spellings of an English part: the list first, else the rules.
fn urdu_parts(model: &Model, list: &WordList, roman: &str, keep: usize) -> Vec<String> {
    let mut found: Vec<String> = list
        .by_roman(roman)
        .iter()
        .map(|e| e.urdu.clone())
        .collect();
    if found.is_empty() {
        found = forward::candidates(model, roman, keep)
            .into_iter()
            .map(|(text, _)| text)
            .collect();
    }
    found.truncate(keep);
    found
}

/// The few best English spellings (lower case) of an Urdu part.
fn roman_parts(model: &Model, list: &WordList, urdu: &str, keep: usize) -> Vec<String> {
    let mut found: Vec<String> = list.by_urdu(urdu).iter().map(|e| e.roman.clone()).collect();
    if found.is_empty() {
        found = reverse::candidates(model, urdu, keep)
            .into_iter()
            .map(|(text, _)| text)
            .collect();
    }
    found.truncate(keep);
    found
}

/// Keeps the best [`KEEP`] distinct spellings.
fn finish(mut found: Vec<Scored>) -> Vec<Scored> {
    found.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    let mut seen: Vec<String> = Vec::new();
    found.retain(|(text, _)| {
        if seen.contains(text) {
            false
        } else {
            seen.push(text.clone());
            true
        }
    });
    found.truncate(KEEP);
    found
}

/// A word split at a character count: the two halves.
fn split_at_letters(word: &str, count: usize) -> Option<(&str, &str)> {
    let byte = word.char_indices().nth(count).map(|(i, _)| i)?;
    Some((word.get(..byte)?, word.get(byte..)?))
}

/// Urdu spellings of an English word made of known parts (lower case letters only).
pub(crate) fn urdu_compounds(model: &Model, list: &WordList, word: &str) -> Vec<Scored> {
    let letters = word.chars().count();
    if !(MIN_LETTERS..=40).contains(&letters) {
        return Vec::new();
    }
    let mut found: Vec<Scored> = Vec::new();
    // `abdulsamad`
    for prefix in ABDUL {
        if let Some(rest) = word.strip_prefix(prefix).filter(|r| r.chars().count() >= 3) {
            for (rank, tail) in urdu_parts(model, list, rest, 2).iter().enumerate() {
                let drop = 0.02 * f64::from(u32::try_from(rank).unwrap_or(0));
                found.push((format!("{ABDUL_URDU}{tail}"), 0.9 - drop));
                found.push((format!("عبد {AL_URDU}{tail}"), 0.6 - drop));
            }
        }
    }
    // `ataullah`
    if let Some(stem) = word
        .strip_suffix(ALLAH_ROMAN)
        .filter(|s| s.chars().count() >= 2)
    {
        for (rank, part) in urdu_parts(model, list, stem, 2).iter().enumerate() {
            let drop = 0.02 * f64::from(u32::try_from(rank).unwrap_or(0));
            found.push((format!("{part} {ALLAH_URDU}"), 0.85 - drop));
            found.push((format!("{part}{ALLAH_URDU}"), 0.6 - drop));
        }
    }
    // `fazlurrehman`, `noorulhaq`, `shamsuddin`
    for cut in 2..letters.saturating_sub(2) {
        let Some((stem, rest)) = split_at_letters(word, cut) else {
            continue;
        };
        let Some(tail) = PARTICLES.iter().find_map(|p| rest.strip_prefix(p)) else {
            continue;
        };
        if tail.chars().count() < 3 || !list.contains_roman(tail) {
            continue;
        }
        let stems = urdu_parts(model, list, stem, 1);
        let tails = urdu_parts(model, list, tail, 1);
        if let (Some(s), Some(t)) = (stems.first(), tails.first()) {
            let sure = if list.contains_roman(stem) { 0.88 } else { 0.8 };
            found.push((format!("{s} {AL_URDU}{t}"), sure));
        }
    }
    // `kanganpur`
    for (ending, urdu) in PLACE_ENDINGS {
        if let Some(stem) = word.strip_suffix(ending).filter(|s| s.chars().count() >= 3) {
            for (rank, part) in urdu_parts(model, list, stem, 2).iter().enumerate() {
                let drop = 0.02 * f64::from(u32::try_from(rank).unwrap_or(0));
                found.push((format!("{part}{urdu}"), 0.8 - drop));
                found.push((format!("{part} {urdu}"), 0.7 - drop));
            }
        }
    }
    // `ghulamrasool`: two words of the list written together
    for cut in 3..letters.saturating_sub(2) {
        let Some((left, right)) = split_at_letters(word, cut) else {
            continue;
        };
        if list.contains_roman(left) && list.contains_roman(right) {
            let lefts = urdu_parts(model, list, left, 1);
            let rights = urdu_parts(model, list, right, 1);
            if let (Some(l), Some(r)) = (lefts.first(), rights.first()) {
                found.push((format!("{l} {r}"), 0.75));
                found.push((format!("{l}{r}"), 0.65));
            }
        }
    }
    finish(found)
}

/// English spellings (lower case) of an Urdu word made of known parts.
pub(crate) fn roman_compounds(model: &Model, list: &WordList, word: &str) -> Vec<Scored> {
    let letters = word.chars().count();
    if !(MIN_LETTERS.saturating_sub(1)..=40).contains(&letters) {
        return Vec::new();
    }
    let mut found: Vec<Scored> = Vec::new();
    if let Some(rest) = word
        .strip_prefix(ABDUL_URDU)
        .filter(|r| r.chars().count() >= 2)
    {
        for (rank, part) in roman_parts(model, list, rest, 2).iter().enumerate() {
            let drop = 0.02 * f64::from(u32::try_from(rank).unwrap_or(0));
            found.push((format!("abdul {part}"), 0.85 - drop));
            found.push((format!("abdul{part}"), 0.6 - drop));
        }
    }
    if let Some(stem) = word
        .strip_suffix(ALLAH_URDU)
        .filter(|s| s.chars().count() >= 2)
    {
        for (rank, part) in roman_parts(model, list, stem, 2).iter().enumerate() {
            let drop = 0.02 * f64::from(u32::try_from(rank).unwrap_or(0));
            found.push((format!("{part}{ALLAH_ROMAN}"), 0.85 - drop));
            found.push((format!("{part} {ALLAH_ROMAN}"), 0.6 - drop));
        }
    }
    for (ending, urdu) in PLACE_ENDINGS {
        if let Some(stem) = word.strip_suffix(urdu).filter(|s| s.chars().count() >= 2) {
            for (rank, part) in roman_parts(model, list, stem, 2).iter().enumerate() {
                let drop = 0.02 * f64::from(u32::try_from(rank).unwrap_or(0));
                found.push((format!("{part}{ending}"), 0.8 - drop));
                found.push((format!("{part} {ending}"), 0.6 - drop));
            }
        }
    }
    for cut in 2..letters.saturating_sub(1) {
        let Some((left, right)) = split_at_letters(word, cut) else {
            continue;
        };
        if !list.by_urdu(left).is_empty() && !list.by_urdu(right).is_empty() {
            let lefts = roman_parts(model, list, left, 1);
            let rights = roman_parts(model, list, right, 1);
            if let (Some(l), Some(r)) = (lefts.first(), rights.first()) {
                found.push((format!("{l} {r}"), 0.75));
                found.push((format!("{l}{r}"), 0.65));
            }
        }
    }
    finish(found)
}

/// For an Urdu word that starts with the joining `ال` (`الرحمن`): the English joining word
/// that goes with it. It is spoken as the first letter of the name after it if that letter is
/// a "sun" letter (`ur rehman`, `ud din`), else `ul` (`ul haq`).
pub(crate) fn particle_for(tail: &str) -> &'static str {
    match tail.chars().next() {
        Some('ر') => "ur",
        Some('د' | 'ڈ') => "ud",
        Some('ن') => "un",
        Some('س' | 'ث' | 'ص') => "us",
        Some('ش') => "ush",
        Some('ز' | 'ذ' | 'ض' | 'ظ') => "uz",
        Some('ت' | 'ٹ' | 'ط') => "ut",
        _ => "ul",
    }
}
