//! What the model's `find` may match on: the code, the short code, the names and the tag, never a
//! phone or a father or firm name (PRD FR-7.83, Code Standards Q7; plan 80, T5-10).
//!
//! The pickers' search (PRD FR-7.40) also matches the phone, the father or firm name and the city, and
//! that is right for the Owner typing in a picker. For the model it would be an oracle: no tool result
//! holds a phone, but a model told by a name in the books to probe ("find 0345", "find 03455" ...) could
//! learn one digit by digit from which account comes back. So every hit of the model's `find` is kept
//! only when the typed text also matches its code, its name in either script or its tag.
//!
//! The match on a name is generous the way the search is: a word that starts with the typed word, one
//! that has the same consonants ("ikram" and "akram"), or one a letter or two away ("arshd"). Every
//! typed word must match.
//!
//! Example: "0345" finds nothing of an account whose phone starts 0345 but whose code is 0020 and whose
//! name is Sabir; "sabir", "sabr" and "0020" find him.

use mandi_services::core::text;

/// The typed word or name as the search keys write it: lower case, one Urdu letter for each look-alike.
fn key(s: &str) -> String {
    text::search_ur(&text::search_en(s))
}

fn words(s: &str) -> Vec<String> {
    key(s)
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(str::to_owned)
        .collect()
}

/// The edit distance of two words, a swap of two neighbours counted as one (optimal string alignment).
fn distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let width = b.len().saturating_add(1);
    let mut rows = vec![0usize; a.len().saturating_add(1).saturating_mul(width)];
    let at = |i: usize, j: usize| i.saturating_mul(width).saturating_add(j);
    for i in 0..=a.len() {
        if let Some(cell) = rows.get_mut(at(i, 0)) {
            *cell = i;
        }
    }
    for j in 0..=b.len() {
        if let Some(cell) = rows.get_mut(at(0, j)) {
            *cell = j;
        }
    }
    for i in 1..=a.len() {
        for j in 1..=b.len() {
            let same = a.get(i.saturating_sub(1)) == b.get(j.saturating_sub(1));
            let get = |x: usize, y: usize| rows.get(at(x, y)).copied().unwrap_or(usize::MAX);
            let mut best =
                get(i.saturating_sub(1), j.saturating_sub(1)).saturating_add(usize::from(!same));
            best = best.min(get(i.saturating_sub(1), j).saturating_add(1));
            best = best.min(get(i, j.saturating_sub(1)).saturating_add(1));
            if i > 1
                && j > 1
                && a.get(i.saturating_sub(1)) == b.get(j.saturating_sub(2))
                && a.get(i.saturating_sub(2)) == b.get(j.saturating_sub(1))
            {
                best = best.min(get(i.saturating_sub(2), j.saturating_sub(2)).saturating_add(1));
            }
            if let Some(cell) = rows.get_mut(at(i, j)) {
                *cell = best;
            }
        }
    }
    rows.get(at(a.len(), b.len()))
        .copied()
        .unwrap_or(usize::MAX)
}

/// True when the typed word is a word of a name, or the start of one, or sounds like one.
fn word_matches(typed: &str, name_words: &[String]) -> bool {
    let letters = typed.chars().count();
    let allowed = if letters <= 5 { 1 } else { 2 };
    let sound = text::search_phonetic(typed);
    name_words.iter().any(|w| {
        w.starts_with(typed)
            || (letters >= 3 && w.contains(typed))
            || (sound.chars().count() >= 2 && text::search_phonetic(w).starts_with(&sound))
            || distance(typed, w) <= allowed
    })
}

/// True when the typed text matches the code, the short code, a name or the tag of a record, and not
/// only its phone, father or firm name or city. `code` is the full code (`0020`: the type number 002
/// and the running number 0). Blank text matches everything.
#[must_use]
pub fn matches(typed: &str, code: &str, name_ur: &str, name_en: &str, tag: Option<&str>) -> bool {
    let typed_words = words(typed);
    if typed_words.is_empty() {
        return true;
    }
    let mut name_words = words(name_ur);
    name_words.extend(words(name_en));
    if let Some(tag) = tag {
        name_words.extend(words(tag));
    }
    // The short code: the running number after the 3-digit type number.
    let short = code.get(3..).unwrap_or("");
    typed_words.iter().all(|word| {
        let digits = word.chars().all(|c| c.is_ascii_digit());
        if digits && (code == word || (!short.is_empty() && short == word)) {
            return true;
        }
        if digits && word.len() > 1 && code.starts_with(word.as_str()) {
            return true;
        }
        word_matches(word, &name_words)
    })
}

#[cfg(test)]
mod tests {
    use super::matches;

    #[test]
    fn a_phone_a_father_and_a_city_are_not_matched() {
        // Sabir, code 0020, whose phone is 0345-5551234 and whose father is Haji Raazdar.
        for probe in [
            "0345",
            "03455551234",
            "034555",
            "Haji Raazdar",
            "Raazdar",
            "حاجی رازدار",
        ] {
            assert!(
                !matches(probe, "0020", "صابر", "Sabir", None),
                "{probe} matched on private text"
            );
        }
    }

    #[test]
    fn the_code_the_short_code_the_name_and_the_tag_are_matched() {
        assert!(matches("0020", "0020", "صابر", "Sabir", None));
        assert!(matches("0", "0020", "صابر", "Sabir", None));
        assert!(matches("sabir", "0020", "صابر", "Sabir", None));
        assert!(matches("SAB", "0020", "صابر", "Sabir", None));
        assert!(matches("صابر", "0020", "صابر", "Sabir", None));
        // A tag tells two of the same name apart: "ashiq k" is Ashiq Zafar K.
        assert!(matches(
            "ashiq k",
            "00211",
            "عاشق ظفر",
            "Ashiq Zafar",
            Some("K")
        ));
        assert!(!matches(
            "ashiq z9",
            "00211",
            "عاشق ظفر",
            "Ashiq Zafar",
            Some("K")
        ));
    }

    #[test]
    fn a_close_or_a_vowel_free_spelling_still_finds_the_name() {
        assert!(matches(
            "ikram",
            "0020",
            "اکرم ٹول ٹیکس",
            "Akram Toll Tax",
            None
        ));
        assert!(matches(
            "arshd",
            "0021",
            "ارشد ملتانی",
            "Arshad Multani",
            None
        ));
        assert!(matches(
            "akrm",
            "0020",
            "اکرم ٹول ٹیکس",
            "Akram Toll Tax",
            None
        ));
        assert!(!matches(
            "zubair",
            "0020",
            "اکرم ٹول ٹیکس",
            "Akram Toll Tax",
            None
        ));
    }

    #[test]
    fn blank_text_matches_every_record() {
        assert!(matches("  ", "0020", "صابر", "Sabir", None));
    }
}
