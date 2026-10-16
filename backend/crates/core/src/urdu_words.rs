//! Amount in Urdu words (PRD FR-19.7, NFR-URD-15, Tech Stack 4.5).
//!
//! Pakistani grouping: ارب (10^9), کروڑ (10^7), لاکھ (10^5), ہزار, سو. Integer `/` and `%`
//! only. Range 0 to 10,00,00,00,000 rupees (دس ارب); outside it, an error, never wrong words.
//!
//! Example: 1,316,040 → "تیرہ لاکھ سولہ ہزار چالیس روپے".
//!
//! The 0 to 99 word list and the singular "ایک روپیہ" need the Urdu reviewer's sign-off
//! (NFR-URD-4) like every other Urdu string.

use thiserror::Error;

use crate::error::ErrorCode;
use crate::money::{MAX_RUPEES, Paisa};

/// The amount cannot be written in words.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum WordsError {
    /// Below 0 or above 10,00,00,00,000 rupees.
    #[error("amount out of range for words")]
    OutOfRange,
}

impl WordsError {
    /// The error code for the screen. Words are only made from stored amounts, so an
    /// amount out of range is a bug.
    pub fn code(self) -> ErrorCode {
        ErrorCode::ErrInternal
    }
}

/// Urdu words for 0 to 99.
const UNITS: [&str; 100] = [
    "صفر",
    "ایک",
    "دو",
    "تین",
    "چار",
    "پانچ",
    "چھ",
    "سات",
    "آٹھ",
    "نو",
    "دس",
    "گیارہ",
    "بارہ",
    "تیرہ",
    "چودہ",
    "پندرہ",
    "سولہ",
    "سترہ",
    "اٹھارہ",
    "انیس",
    "بیس",
    "اکیس",
    "بائیس",
    "تیئس",
    "چوبیس",
    "پچیس",
    "چھبیس",
    "ستائیس",
    "اٹھائیس",
    "انتیس",
    "تیس",
    "اکتیس",
    "بتیس",
    "تینتیس",
    "چونتیس",
    "پینتیس",
    "چھتیس",
    "سینتیس",
    "اڑتیس",
    "انتالیس",
    "چالیس",
    "اکتالیس",
    "بیالیس",
    "تینتالیس",
    "چوالیس",
    "پینتالیس",
    "چھیالیس",
    "سینتالیس",
    "اڑتالیس",
    "انچاس",
    "پچاس",
    "اکاون",
    "باون",
    "ترپن",
    "چون",
    "پچپن",
    "چھپن",
    "ستاون",
    "اٹھاون",
    "انسٹھ",
    "ساٹھ",
    "اکسٹھ",
    "باسٹھ",
    "ترسٹھ",
    "چونسٹھ",
    "پینسٹھ",
    "چھیاسٹھ",
    "سڑسٹھ",
    "اڑسٹھ",
    "انہتر",
    "ستر",
    "اکہتر",
    "بہتر",
    "تہتر",
    "چوہتر",
    "پچھتر",
    "چھہتر",
    "ستتر",
    "اٹھہتر",
    "اناسی",
    "اسی",
    "اکاسی",
    "بیاسی",
    "تراسی",
    "چوراسی",
    "پچاسی",
    "چھیاسی",
    "ستاسی",
    "اٹھاسی",
    "نواسی",
    "نوے",
    "اکانوے",
    "بانوے",
    "ترانوے",
    "چورانوے",
    "پچانوے",
    "چھیانوے",
    "ستانوے",
    "اٹھانوے",
    "ننانوے",
];

/// Place names from the largest down, with their size in rupees.
const PLACES: [(u64, &str); 5] = [
    (1_000_000_000, "ارب"),
    (10_000_000, "کروڑ"),
    (100_000, "لاکھ"),
    (1_000, "ہزار"),
    (100, "سو"),
];

/// The Urdu word for 0 to 99.
fn unit(n: u64) -> &'static str {
    usize::try_from(n)
        .ok()
        .and_then(|i| UNITS.get(i))
        .copied()
        .unwrap_or_default()
}

/// Whole rupees of an amount, if it is in the words range.
pub(crate) fn rupees_in_range(amount: Paisa) -> Result<u64, WordsError> {
    match u64::try_from(amount.rupees()) {
        Ok(r) if r <= MAX_RUPEES.unsigned_abs() => Ok(r),
        _ => Err(WordsError::OutOfRange),
    }
}

/// The amount in Urdu words, ending in روپے (روپیہ for exactly one rupee).
///
/// 19,316 → "انیس ہزار تین سو سولہ روپے"; 1,00,000 → "ایک لاکھ روپے"; 0 → "صفر روپے".
pub fn urdu_words(amount: Paisa) -> Result<String, WordsError> {
    let rupees = rupees_in_range(amount)?;
    if rupees == 1 {
        return Ok("ایک روپیہ".to_owned());
    }
    let mut words: Vec<&str> = Vec::new();
    let mut rest = rupees;
    for (size, name) in PLACES {
        let count = rest.checked_div(size).unwrap_or_default();
        rest = rest.checked_rem(size).unwrap_or_default();
        if count > 0 {
            words.push(unit(count));
            words.push(name);
        }
    }
    if rest > 0 || words.is_empty() {
        words.push(unit(rest));
    }
    words.push("روپے");
    Ok(words.join(" "))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn words(rupees: i64) -> String {
        urdu_words(Paisa::from_rupees(rupees).unwrap()).unwrap()
    }

    #[test]
    fn small_and_special() {
        assert_eq!(words(1), "ایک روپیہ");
        assert_eq!(words(2), "دو روپے");
        assert_eq!(words(1_001), "ایک ہزار ایک روپے");
        assert_eq!(words(200_500), "دو لاکھ پانچ سو روپے");
        assert_eq!(words(10_000_000), "ایک کروڑ روپے");
        assert_eq!(words(1_000_000_000), "ایک ارب روپے");
    }

    #[test]
    fn out_of_range() {
        let too_big = Paisa::from_rupees(MAX_RUPEES + 1).unwrap();
        assert_eq!(urdu_words(too_big), Err(WordsError::OutOfRange));
        let negative = Paisa::from_rupees(-1).unwrap();
        assert_eq!(urdu_words(negative), Err(WordsError::OutOfRange));
        assert_eq!(WordsError::OutOfRange.code(), ErrorCode::ErrInternal);
        assert_eq!(unit(100), "");
    }
}
