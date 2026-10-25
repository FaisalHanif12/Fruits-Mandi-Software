//! Formatting of stored values for print (PRD FR-19.7, NFR-URD-14, NFR-URD-16). Text only; no
//! money is worked out here. Digits, grouping and dates are decided here so a template never
//! has to (CLAUDE.md rule 1).

use chrono::{DateTime, Duration, Utc};
use mandi_core::auth::Language;
use mandi_core::dates::{BusinessDate, format_ddmmyyyy};
use mandi_core::money::{Grouping, Paisa, format_count, format_paisa};

use crate::error::ReportsError;

/// Pakistan time is UTC+5 all year (Code Standards G9).
const PAKISTAN_OFFSET_HOURS: i64 = 5;

/// How numbers are written on one print: the grouping, and whether the digits are Urdu.
#[derive(Debug, Clone, Copy)]
pub(crate) struct NumberStyle {
    grouping: Grouping,
    urdu_digits: bool,
}

impl NumberStyle {
    /// Urdu digits are used only on an Urdu print with the setting on (NFR-URD-14).
    pub(crate) fn new(language: Language, grouping: Grouping, urdu_digits_setting: bool) -> Self {
        Self {
            grouping,
            urdu_digits: urdu_digits_setting && language == Language::Ur,
        }
    }

    /// True when this print uses Urdu digits.
    pub(crate) fn urdu_digits(self) -> bool {
        self.urdu_digits
    }

    /// Whole rupees with grouping commas.
    pub(crate) fn money(self, amount: Paisa) -> Result<String, ReportsError> {
        Ok(self.digits(format_paisa(amount, self.grouping)?))
    }

    /// A whole count, grouped like money.
    pub(crate) fn count(self, value: i64) -> String {
        self.digits(format_count(value, self.grouping))
    }

    /// A whole number with no grouping commas, for a year: `2026`, not `2,026`.
    pub(crate) fn plain(self, value: i64) -> String {
        self.digits(value.to_string())
    }

    /// A day `YYYY-MM-DD` as `DD-MM-YYYY` (NFR-URD-16).
    pub(crate) fn date(self, iso: &str) -> Result<String, ReportsError> {
        let day = BusinessDate::parse_iso(iso)
            .ok_or_else(|| ReportsError::Data(format!("a stored date is not a date: {iso}")))?;
        Ok(self.digits(format_ddmmyyyy(day)))
    }

    /// The time of the print in Pakistan time, `DD-MM-YYYY HH:MM`.
    pub(crate) fn printed_at(self, now_utc: DateTime<Utc>) -> String {
        let local = now_utc + Duration::hours(PAKISTAN_OFFSET_HOURS);
        self.digits(local.format("%d-%m-%Y %H:%M").to_string())
    }

    /// A share in hundredths of a percent as `72.10` (analytics, PRD 18.2). Display only.
    pub(crate) fn hundredths(self, value: i64) -> String {
        let sign = if value < 0 { "-" } else { "" };
        let value = value.unsigned_abs();
        self.digits(format!("{sign}{}.{:02}", value / 100, value % 100))
    }

    /// A rate in basis points as the percent a person writes, with no trailing zeros: 4,000 →
    /// `40`, 3,750 → `37.5`, 5 → `0.05` (as the screen's `formatBps`). Display only; it works on
    /// the digits and never divides.
    pub(crate) fn bps_text(self, bps: i64) -> String {
        let sign = if bps < 0 { "-" } else { "" };
        let bps = bps.unsigned_abs();
        let (whole, fraction) = (bps / 100, bps % 100);
        let text = if fraction == 0 {
            format!("{sign}{whole}")
        } else {
            let fraction = format!("{fraction:02}");
            format!("{sign}{whole}.{}", fraction.trim_end_matches('0'))
        };
        self.digits(text)
    }

    /// A weight in grams as kg with up to 3 decimals and no trailing zeros: 12,345 → `12.345`,
    /// 12,500 → `12.5`, 12,000 → `12`.
    pub(crate) fn kilos(self, grams: i64) -> String {
        self.digits(kilos_text(grams))
    }

    /// Turns the digits 0-9 into ۰-۹ when this print uses Urdu digits. Every other character
    /// stays (the comma, the point, the dash and the colon).
    fn digits(self, text: String) -> String {
        if self.urdu_digits {
            to_urdu_digits(&text)
        } else {
            text
        }
    }
}

/// A weight in grams as kg with up to 3 decimals and no trailing zeros, in English digits:
/// 12,345 → `12.345`, 12,500 → `12.5`, 12,000 → `12`. Text work on the digits, no arithmetic on
/// a float. A report page cell holds it as text; a print may turn the digits into Urdu ones.
pub(crate) fn kilos_text(grams: i64) -> String {
    let sign = if grams < 0 { "-" } else { "" };
    let grams = grams.unsigned_abs();
    let whole = grams / 1_000;
    let fraction = grams % 1_000;
    if fraction == 0 {
        format!("{sign}{whole}")
    } else {
        let fraction = format!("{fraction:03}");
        format!("{sign}{whole}.{}", fraction.trim_end_matches('0'))
    }
}

/// `0123456789` → `۰۱۲۳۴۵۶۷۸۹` (U+06F0 to U+06F9, the digits of the Urdu font).
pub(crate) fn to_urdu_digits(text: &str) -> String {
    text.chars()
        .map(|c| {
            c.to_digit(10)
                .and_then(|digit| char::from_u32(0x06F0_u32.saturating_add(digit)))
                .unwrap_or(c)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn style(language: Language, urdu: bool) -> NumberStyle {
        NumberStyle::new(language, Grouping::International, urdu)
    }

    fn rupees(n: i64) -> Paisa {
        Paisa::from_rupees(n).unwrap()
    }

    #[test]
    fn a_rate_in_basis_points_reads_as_the_percent_a_person_writes() {
        let en = style(Language::En, false);
        assert_eq!(en.bps_text(4_000), "40");
        assert_eq!(en.bps_text(3_750), "37.5");
        assert_eq!(en.bps_text(5), "0.05");
        assert_eq!(en.bps_text(250), "2.5");
        assert_eq!(en.bps_text(0), "0");
        assert_eq!(en.bps_text(10_000), "100");
        assert_eq!(en.bps_text(-600), "-6");
        // Urdu digits follow the print setting, as for every other number.
        assert_eq!(style(Language::Ur, true).bps_text(3_750), "۳۷.۵");
    }

    #[test]
    fn english_digits_by_default() {
        let s = style(Language::Ur, false);
        assert_eq!(s.money(rupees(19_316)).unwrap(), "19,316");
        assert_eq!(s.count(20), "20");
        assert_eq!(s.date("2026-10-10").unwrap(), "10-10-2026");
        assert!(!s.urdu_digits());
    }

    #[test]
    fn urdu_digits_only_on_an_urdu_print_with_the_setting_on() {
        let s = style(Language::Ur, true);
        assert!(s.urdu_digits());
        assert_eq!(s.money(rupees(19_316)).unwrap(), "۱۹,۳۱۶");
        assert_eq!(s.date("2026-10-10").unwrap(), "۱۰-۱۰-۲۰۲۶");
        // The setting does nothing on an English print.
        let en = style(Language::En, true);
        assert!(!en.urdu_digits());
        assert_eq!(en.money(rupees(19_316)).unwrap(), "19,316");
    }

    #[test]
    fn lakh_grouping_follows_the_setting() {
        let s = NumberStyle::new(Language::En, Grouping::Lakh, false);
        assert_eq!(s.money(rupees(1_316_040)).unwrap(), "13,16,040");
        assert_eq!(s.count(1_316_040), "13,16,040");
    }

    #[test]
    fn a_bad_stored_date_is_an_error_not_a_guess() {
        assert!(style(Language::En, false).date("10-10-2026").is_err());
    }

    #[test]
    fn print_time_is_pakistan_time() {
        // 09:00 UTC on 10-10-2026 is 14:00 in Pakistan.
        let now = Utc.with_ymd_and_hms(2026, 10, 10, 9, 0, 0).unwrap();
        assert_eq!(
            style(Language::En, false).printed_at(now),
            "10-10-2026 14:00"
        );
        // 21:30 UTC is 02:30 the next day.
        let late = Utc.with_ymd_and_hms(2026, 10, 10, 21, 30, 0).unwrap();
        assert_eq!(
            style(Language::En, false).printed_at(late),
            "11-10-2026 02:30"
        );
    }

    #[test]
    fn kilos_have_up_to_three_decimals() {
        let s = style(Language::En, false);
        assert_eq!(s.kilos(12_345), "12.345");
        assert_eq!(s.kilos(12_500), "12.5");
        assert_eq!(s.kilos(12_050), "12.05");
        assert_eq!(s.kilos(12_000), "12");
        assert_eq!(s.kilos(5), "0.005");
        // A weight is always above 0, but a bad value is still shown with its sign.
        assert_eq!(s.kilos(-1), "-0.001");
        assert_eq!(s.kilos(i64::MIN), "-9223372036854775.808");
        assert_eq!(style(Language::Ur, true).kilos(12_345), "۱۲.۳۴۵");
    }

    #[test]
    fn urdu_digit_map_covers_all_ten() {
        assert_eq!(to_urdu_digits("0123456789"), "۰۱۲۳۴۵۶۷۸۹");
        assert_eq!(to_urdu_digits("ab-1,2:3."), "ab-۱,۲:۳.");
    }
}
