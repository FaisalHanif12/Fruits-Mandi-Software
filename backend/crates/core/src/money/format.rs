//! Money as display text in whole rupees (money agent 3.2, PRD Section 21 number grouping).
//!
//! The same rules as `formatPaisa` in `frontend/src/shared/format/money.ts`, used by the
//! backend for prints, Excel and the backup pack. Integer `/` and `%` only.

use serde::{Deserialize, Serialize};

use super::{MoneyError, Paisa};

/// Number grouping setting (PRD Section 21).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "lowercase")]
pub enum Grouping {
    /// `1,316,040` (the default).
    #[default]
    International,
    /// `13,16,040`.
    Lakh,
}

/// Whole rupees with grouping commas. Negative values get a leading `-` (technical views
/// only; ledgers show the side instead).
///
/// `Paisa(131604000)` → `"1,316,040"` (international) or `"13,16,040"` (lakh).
///
/// The result type stays for the contract: a `Paisa` is always a whole rupee, so this
/// never fails today.
pub fn format_paisa(amount: Paisa, grouping: Grouping) -> Result<String, MoneyError> {
    let rupees = amount.rupees();
    let digits = rupees.unsigned_abs().to_string();
    let grouped = group_digits(&digits, grouping);
    Ok(if rupees < 0 {
        format!("-{grouped}")
    } else {
        grouped
    })
}

/// Puts commas into a string of ASCII digits: the last group has 3 digits, the earlier
/// groups 3 (international) or 2 (lakh).
fn group_digits(digits: &str, grouping: Grouping) -> String {
    let (head, tail) = digits.split_at(digits.len().saturating_sub(3));
    let size = match grouping {
        Grouping::International => 3,
        Grouping::Lakh => 2,
    };
    let mut groups: Vec<&str> = Vec::new();
    let mut rest = head;
    while !rest.is_empty() {
        let (left, right) = rest.split_at(rest.len().saturating_sub(size));
        groups.push(right);
        rest = left;
    }
    groups.reverse();
    groups.push(tail);
    groups.join(",")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fmt(rupees: i64, grouping: Grouping) -> String {
        format_paisa(Paisa::from_rupees(rupees).unwrap(), grouping).unwrap()
    }

    #[test]
    fn grouping_cases() {
        use Grouping::{International as I, Lakh as L};
        assert_eq!(fmt(0, I), "0");
        assert_eq!(fmt(7, I), "7");
        assert_eq!(fmt(999, L), "999");
        assert_eq!(fmt(1_000, I), "1,000");
        assert_eq!(fmt(1_000, L), "1,000");
        assert_eq!(fmt(12_000, I), "12,000");
        assert_eq!(fmt(100_000, I), "100,000");
        assert_eq!(fmt(100_000, L), "1,00,000");
        assert_eq!(fmt(1_316_040, I), "1,316,040");
        assert_eq!(fmt(1_316_040, L), "13,16,040");
        assert_eq!(fmt(10_000_000_000, I), "10,000,000,000");
        assert_eq!(fmt(10_000_000_000, L), "10,00,00,00,000");
        assert_eq!(fmt(-1_316_040, I), "-1,316,040");
        assert_eq!(fmt(-5, L), "-5");
    }

    #[test]
    fn extreme_values_do_not_fail() {
        // The largest and smallest whole-rupee paisa values.
        let max = Paisa::try_from_paisa(i64::MAX - i64::MAX % 100).unwrap();
        let min = Paisa::try_from_paisa(i64::MIN - i64::MIN % 100).unwrap();
        assert_eq!(
            format_paisa(max, Grouping::International).unwrap(),
            "92,233,720,368,547,758"
        );
        assert_eq!(
            format_paisa(min, Grouping::Lakh).unwrap(),
            "-92,23,37,20,36,85,47,758"
        );
    }

    #[test]
    fn grouping_serialises_lowercase() {
        assert_eq!(serde_json::to_string(&Grouping::Lakh).unwrap(), "\"lakh\"");
        assert_eq!(Grouping::default(), Grouping::International);
    }
}
