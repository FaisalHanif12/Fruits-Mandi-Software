//! Amount in English words (PRD FR-19.7, NFR-URD-23, Tech Stack 4.5).
//!
//! Two groupings, by the number grouping setting: billion, million, thousand (international)
//! or crore, lakh, thousand (lakh). Integer `/` and `%` only. Range 0 to 10,00,00,00,000
//! rupees; outside it, an error.
//!
//! Example: 19,316 → "Nineteen thousand three hundred sixteen rupees only".
//!
//! Style for the English reviewer (NFR-URD-21): no "and"; 21 to 99 with a hyphen
//! ("twenty-one"); "One rupee only" for exactly one rupee.

use crate::money::{Grouping, Paisa};
use crate::urdu_words::{WordsError, rupees_in_range};

/// English words for 0 to 19.
const ONES: [&str; 20] = [
    "zero",
    "one",
    "two",
    "three",
    "four",
    "five",
    "six",
    "seven",
    "eight",
    "nine",
    "ten",
    "eleven",
    "twelve",
    "thirteen",
    "fourteen",
    "fifteen",
    "sixteen",
    "seventeen",
    "eighteen",
    "nineteen",
];

/// English words for the tens, 20 to 90 (index = tens digit).
const TENS: [&str; 10] = [
    "", "", "twenty", "thirty", "forty", "fifty", "sixty", "seventy", "eighty", "ninety",
];

/// Place names from the largest down, with their size in rupees.
const INTERNATIONAL: [(u64, &str); 3] = [
    (1_000_000_000, "billion"),
    (1_000_000, "million"),
    (1_000, "thousand"),
];

/// Place names for the lakh grouping.
const LAKH: [(u64, &str); 3] = [
    (10_000_000, "crore"),
    (100_000, "lakh"),
    (1_000, "thousand"),
];

/// A word from a table, or empty text if the index is outside it (never happens).
fn word(table: &[&'static str], n: u64) -> &'static str {
    usize::try_from(n)
        .ok()
        .and_then(|i| table.get(i))
        .copied()
        .unwrap_or_default()
}

/// Words for 1 to 999 (empty for 0).
fn below_thousand(n: u64) -> Vec<String> {
    let mut out = Vec::new();
    let hundreds = n.checked_div(100).unwrap_or_default();
    let rest = n.checked_rem(100).unwrap_or_default();
    if hundreds > 0 {
        out.push(word(&ONES, hundreds).to_owned());
        out.push("hundred".to_owned());
    }
    if rest >= 20 {
        let tens = word(&TENS, rest.checked_div(10).unwrap_or_default());
        match rest.checked_rem(10).unwrap_or_default() {
            0 => out.push(tens.to_owned()),
            one => out.push(format!("{tens}-{}", word(&ONES, one))),
        }
    } else if rest > 0 {
        out.push(word(&ONES, rest).to_owned());
    }
    out
}

/// Words for any count, using the place table. A count above the largest place is itself
/// written in words ("one thousand crore").
fn count_words(n: u64, places: &[(u64, &'static str)]) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = n;
    for &(size, name) in places {
        let count = rest.checked_div(size).unwrap_or_default();
        rest = rest.checked_rem(size).unwrap_or_default();
        if count > 0 {
            let remaining = places
                .iter()
                .skip_while(|&&(s, _)| s >= size)
                .copied()
                .collect::<Vec<_>>();
            out.extend(count_words(count, &remaining));
            out.push(name.to_owned());
        }
    }
    out.extend(below_thousand(rest));
    out
}

/// The amount in English words, ending in "rupees only" ("rupee only" for one rupee),
/// with a capital first letter.
///
/// 1,316,040 → "One million three hundred sixteen thousand forty rupees only" (international)
/// or "Thirteen lakh sixteen thousand forty rupees only" (lakh).
pub fn english_words(amount: Paisa, grouping: Grouping) -> Result<String, WordsError> {
    let rupees = rupees_in_range(amount)?;
    let places: &[(u64, &'static str)] = match grouping {
        Grouping::International => &INTERNATIONAL,
        Grouping::Lakh => &LAKH,
    };
    let mut words = count_words(rupees, places);
    if words.is_empty() {
        words.push("zero".to_owned());
    }
    words.push(
        if rupees == 1 {
            "rupee only"
        } else {
            "rupees only"
        }
        .to_owned(),
    );
    Ok(capitalise_first(&words.join(" ")))
}

/// The text with its first letter in capitals: "nineteen ..." → "Nineteen ...".
fn capitalise_first(text: &str) -> String {
    let mut chars = text.chars();
    let first: String = chars
        .next()
        .into_iter()
        .flat_map(char::to_uppercase)
        .collect();
    first + chars.as_str()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::money::MAX_RUPEES;

    fn en(rupees: i64, grouping: Grouping) -> String {
        english_words(Paisa::from_rupees(rupees).unwrap(), grouping).unwrap()
    }

    #[test]
    fn small_and_special() {
        use Grouping::{International as I, Lakh as L};
        assert_eq!(en(1, I), "One rupee only");
        assert_eq!(en(21, I), "Twenty-one rupees only");
        assert_eq!(en(110, L), "One hundred ten rupees only");
        assert_eq!(en(1_000_001, I), "One million one rupees only");
        assert_eq!(en(10_000_000, L), "One crore rupees only");
        assert_eq!(
            en(123_456_789, L),
            "Twelve crore thirty-four lakh fifty-six thousand seven hundred eighty-nine rupees only"
        );
        assert_eq!(
            en(123_456_789, I),
            "One hundred twenty-three million four hundred fifty-six thousand seven hundred eighty-nine rupees only"
        );
    }

    #[test]
    fn out_of_range() {
        let too_big = Paisa::from_rupees(MAX_RUPEES + 1).unwrap();
        assert_eq!(
            english_words(too_big, Grouping::Lakh),
            Err(WordsError::OutOfRange)
        );
        assert_eq!(word(&ONES, 99), "");
    }
}
