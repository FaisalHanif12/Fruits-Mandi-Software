//! The quality of a sale line (درجہ, PRD FR-12.42, plan 58): one of three fixed words, or none.
//!
//! The goods of one lot can differ in quality, and each quality sells at its own rate. The munshi
//! sells them as separate lines of the lot and may mark each line with its quality. The quality
//! never changes a figure: a line is still quantity times rate, with its own commission and لاگا
//! (C1 to C5). It is stored on the line, shown on the bills and the ledgers, and nothing more.
//!
//! Example: `"second"` is `SaleGrade::Second`, shown as "دوم" in Urdu and "Second" in English.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// The quality of a sale line: اول, دوم or سوم (First, Second, Third). A line with no quality has
/// no grade at all (`None` where it is carried); there is no fourth value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum SaleGrade {
    /// اول (First).
    First,
    /// دوم (Second).
    Second,
    /// سوم (Third).
    Third,
}

impl SaleGrade {
    /// The stored form (`bikri_sales.grade`).
    pub const fn as_str(self) -> &'static str {
        match self {
            SaleGrade::First => "first",
            SaleGrade::Second => "second",
            SaleGrade::Third => "third",
        }
    }

    /// Reads the stored or typed form. Nothing but the three words is a grade.
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "first" => Some(SaleGrade::First),
            "second" => Some(SaleGrade::Second),
            "third" => Some(SaleGrade::Third),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SaleGrade;

    #[test]
    fn the_three_words_round_trip_and_nothing_else_is_a_grade() {
        for grade in [SaleGrade::First, SaleGrade::Second, SaleGrade::Third] {
            assert_eq!(SaleGrade::parse(grade.as_str()), Some(grade));
        }
        for text in ["", "First", "fourth", " first", "اول"] {
            assert_eq!(SaleGrade::parse(text), None, "{text:?}");
        }
    }
}
