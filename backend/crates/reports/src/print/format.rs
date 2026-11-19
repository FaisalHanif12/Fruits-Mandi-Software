//! Formatting of stored values for print (PRD FR-19.7, NFR-URD-14, NFR-URD-16). Text only; no
//! money is worked out here. Digits, grouping and dates are decided here so a template never
//! has to (CLAUDE.md rule 1).

use chrono::{DateTime, Duration, Utc};
use mandi_core::dates::{BusinessDate, format_ddmmyyyy};
use mandi_core::money::{Grouping, Paisa, format_count, format_paisa};

use crate::error::ReportsError;

/// Pakistan time is UTC+5 all year (Code Standards G9).
const PAKISTAN_OFFSET_HOURS: i64 = 5;

/// How numbers are written on one print: the grouping. The digits are always 0 to 9, in Urdu and
/// in English (PRD FR-7.32, NFR-URD-14, plan 63). There is no switch for Urdu digits.
#[derive(Debug, Clone, Copy)]
pub(crate) struct NumberStyle {
    grouping: Grouping,
}

impl NumberStyle {
    pub(crate) fn new(grouping: Grouping) -> Self {
        Self { grouping }
    }

    /// Whole rupees with grouping commas.
    pub(crate) fn money(self, amount: Paisa) -> Result<String, ReportsError> {
        Ok(format_paisa(amount, self.grouping)?)
    }

    /// Whole rupees with no grouping commas (`1200`), for a line written the way the client's own
    /// print writes it: "(سیب 8 . 2600)" (PRD 19.2 R5, FR-19.12).
    pub(crate) fn plain_money(self, amount: Paisa) -> String {
        amount.rupees().to_string()
    }

    /// A weight in grams as kilos with up to 3 decimals and no trailing zeros (`12.345`).
    pub(crate) fn weight(self, grams: i64) -> Result<String, ReportsError> {
        let text = mandi_core::masters::format_kg(grams)
            .ok_or_else(|| ReportsError::Data("a weight is not more than 0".to_owned()))?;
        Ok(text)
    }

    /// A whole count, grouped like money.
    pub(crate) fn count(self, value: i64) -> String {
        format_count(value, self.grouping)
    }

    /// A whole number with no grouping commas, for a year: `2026`, not `2,026`.
    pub(crate) fn plain(self, value: i64) -> String {
        value.to_string()
    }

    /// A day `YYYY-MM-DD` as `DD-MM-YYYY` (NFR-URD-16).
    pub(crate) fn date(self, iso: &str) -> Result<String, ReportsError> {
        let day = BusinessDate::parse_iso(iso)
            .ok_or_else(|| ReportsError::Data(format!("a stored date is not a date: {iso}")))?;
        Ok(format_ddmmyyyy(day))
    }

    /// The time of the print in Pakistan time, `DD-MM-YYYY HH:MM`.
    pub(crate) fn printed_at(self, now_utc: DateTime<Utc>) -> String {
        let local = now_utc + Duration::hours(PAKISTAN_OFFSET_HOURS);
        local.format("%d-%m-%Y %H:%M").to_string()
    }

    /// A share in hundredths of a percent as `72.10` (analytics, PRD 18.2). Display only.
    pub(crate) fn hundredths(self, value: i64) -> String {
        let sign = if value < 0 { "-" } else { "" };
        let value = value.unsigned_abs();
        format!("{sign}{}.{:02}", value / 100, value % 100)
    }

    /// A rate in basis points as the percent a person writes, with no trailing zeros: 4,000 →
    /// `40`, 3,750 → `37.5`, 5 → `0.05` (as the screen's `formatBps`). Display only; it works on
    /// the digits and never divides.
    pub(crate) fn bps_text(self, bps: i64) -> String {
        let sign = if bps < 0 { "-" } else { "" };
        let bps = bps.unsigned_abs();
        let (whole, fraction) = (bps / 100, bps % 100);
        if fraction == 0 {
            format!("{sign}{whole}")
        } else {
            let fraction = format!("{fraction:02}");
            format!("{sign}{whole}.{}", fraction.trim_end_matches('0'))
        }
    }

    /// A weight in grams as kg with up to 3 decimals and no trailing zeros: 12,345 → `12.345`,
    /// 12,500 → `12.5`, 12,000 → `12`.
    pub(crate) fn kilos(self, grams: i64) -> String {
        kilos_text(grams)
    }
}

/// A weight in grams as kg with up to 3 decimals and no trailing zeros, in English digits:
/// 12,345 → `12.345`, 12,500 → `12.5`, 12,000 → `12`. Text work on the digits, no arithmetic on
/// a float. A report page cell holds it as text, and a print shows it as it is.
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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn style() -> NumberStyle {
        NumberStyle::new(Grouping::International)
    }

    fn rupees(n: i64) -> Paisa {
        Paisa::from_rupees(n).unwrap()
    }

    #[test]
    fn a_rate_in_basis_points_reads_as_the_percent_a_person_writes() {
        let en = style();
        assert_eq!(en.bps_text(4_000), "40");
        assert_eq!(en.bps_text(3_750), "37.5");
        assert_eq!(en.bps_text(5), "0.05");
        assert_eq!(en.bps_text(250), "2.5");
        assert_eq!(en.bps_text(0), "0");
        assert_eq!(en.bps_text(10_000), "100");
        assert_eq!(en.bps_text(-600), "-6");
    }

    #[test]
    fn every_number_is_written_with_the_digits_0_to_9() {
        // Plan 63: there is no Urdu digit setting, so an Urdu print and an English print are the
        // same. Every number has only 0 to 9, the grouping comma, the point and the dash.
        let s = style();
        assert_eq!(s.money(rupees(19_316)).unwrap(), "19,316");
        assert_eq!(s.money(rupees(10_000)).unwrap(), "10,000");
        assert_eq!(s.count(20), "20");
        assert_eq!(s.plain(2026), "2026");
        assert_eq!(s.date("2026-10-10").unwrap(), "10-10-2026");
        assert_eq!(s.hundredths(7_210), "72.10");
        assert_eq!(s.kilos(12_345), "12.345");
        assert_eq!(s.bps_text(3_750), "37.5");
        let texts = [
            s.money(rupees(1_316_040)).unwrap(),
            s.date("2026-10-10").unwrap(),
            s.weight(12_345).unwrap(),
        ];
        for text in texts {
            assert!(
                text.chars()
                    .all(|c| c.is_ascii_digit() || ",.-".contains(c))
            );
        }
    }

    #[test]
    fn lakh_grouping_follows_the_setting() {
        let s = NumberStyle::new(Grouping::Lakh);
        assert_eq!(s.money(rupees(1_316_040)).unwrap(), "13,16,040");
        assert_eq!(s.count(1_316_040), "13,16,040");
    }

    #[test]
    fn a_bad_stored_date_is_an_error_not_a_guess() {
        assert!(style().date("10-10-2026").is_err());
    }

    #[test]
    fn print_time_is_pakistan_time() {
        // 09:00 UTC on 10-10-2026 is 14:00 in Pakistan.
        let now = Utc.with_ymd_and_hms(2026, 10, 10, 9, 0, 0).unwrap();
        assert_eq!(style().printed_at(now), "10-10-2026 14:00");
        // 21:30 UTC is 02:30 the next day.
        let late = Utc.with_ymd_and_hms(2026, 10, 10, 21, 30, 0).unwrap();
        assert_eq!(style().printed_at(late), "11-10-2026 02:30");
    }

    #[test]
    fn kilos_have_up_to_three_decimals() {
        let s = style();
        assert_eq!(s.kilos(12_345), "12.345");
        assert_eq!(s.kilos(12_500), "12.5");
        assert_eq!(s.kilos(12_050), "12.05");
        assert_eq!(s.kilos(12_000), "12");
        assert_eq!(s.kilos(5), "0.005");
        // A weight is always above 0, but a bad value is still shown with its sign.
        assert_eq!(s.kilos(-1), "-0.001");
        assert_eq!(s.kilos(i64::MIN), "-9223372036854775.808");
    }
}
