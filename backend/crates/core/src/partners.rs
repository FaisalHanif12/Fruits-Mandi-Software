//! Partners and the live split (PRD 17.1 to 17.3, FR-6.5, plan 11): the shapes the partner
//! screens send and read back. Only shapes and constants; the split is in `domain::partners`
//! and the database work is in `services::partners`.
//!
//! A partner is a name, two accounts (capital 0602xxxx and drawings 0603xxxx) and a history of
//! shares. Shares are basis points (1% = 100 bps) and must total exactly 10,000 for every date
//! (PRD FR-17.1). Money travels as paisa numbers; the frontend only formats them.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::accounts::LineError;
use crate::error::{ErrorCode, ErrorValue, FieldError};
use crate::money::{BPS_FULL, Bps, Paisa, parse_bps};

/// The most partners the firm can have (NFR-SCL-5).
pub const MAX_PARTNERS: usize = 10;

/// The control under which the capital account of a partner goes (PRD FR-10.4, 0602xxxx).
pub const CAPITAL_CONTROL: &str = "0602";
/// The control under which the drawings account (ذاتی خرچہ) of a partner goes (0603xxxx).
pub const DRAWINGS_CONTROL: &str = "0603";

/// One partner as the lists show him (PRD 17.1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct PartnerRow {
    /// Row id. A lower id means the partner was created first (PRD C10 tie rule).
    #[ts(type = "number")]
    pub id: i64,
    /// Urdu name.
    pub name_ur: String,
    /// English name.
    pub name_en: String,
    /// His capital account (control 0602).
    #[ts(type = "number")]
    pub capital_account_id: i64,
    /// Its code, for example `06020001`.
    pub capital_account_code: String,
    /// His drawings account (control 0603).
    #[ts(type = "number")]
    pub drawings_account_id: i64,
    /// Its code, for example `06030001`.
    pub drawings_account_code: String,
    /// His share on today's date in bps (4000 = 40%). 0 until shares are set.
    pub share_bps: i32,
    /// False once he is made inactive. Inactive partners keep their history.
    pub is_active: bool,
    /// The version to send back with an edit.
    #[ts(type = "number")]
    pub version: i64,
}

/// One entry of a partner's share history (PRD FR-17.2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct ShareHistoryRow {
    /// The share in bps.
    pub share_bps: i32,
    /// The first day it applies, `YYYY-MM-DD`.
    pub effective_from: String,
}

/// One partner with his share history, oldest first.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct PartnerDetail {
    /// The partner.
    pub row: PartnerRow,
    /// Every share row he has had.
    pub shares: Vec<ShareHistoryRow>,
}

/// Which partner to read.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct PartnersGetInput {
    /// Row id.
    #[ts(type = "number")]
    pub id: i64,
}

/// A new partner (PRD 17.1). He has no share until shares are set.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct PartnerCreateInput {
    /// Urdu name (PRD FR-7.28).
    pub name_ur: String,
    /// English name.
    pub name_en: String,
    /// His opening capital in whole rupees, if any. Posted as the opening credit of his capital
    /// account dated today (PRD FR-10.9). Empty or null for none.
    #[serde(default)]
    pub opening_capital: Option<String>,
    /// Words the user confirmed in the spelling check (PRD FR-7.66).
    #[serde(default)]
    pub confirmed_rules_words: Vec<String>,
}

/// An edit of a partner's names or his active state.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct PartnerUpdateInput {
    /// Row id.
    #[ts(type = "number")]
    pub id: i64,
    /// The `version` read with the row.
    #[ts(type = "number")]
    pub version: i64,
    /// Urdu name.
    pub name_ur: String,
    /// English name.
    pub name_en: String,
    /// False to make him inactive. Needs his share to be 0 from today (`ERR_PARTNER_HAS_SHARE`).
    pub is_active: bool,
    /// Words the user confirmed in the spelling check.
    #[serde(default)]
    pub confirmed_rules_words: Vec<String>,
}

/// One partner's share as typed.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct ShareInput {
    /// The partner.
    #[ts(type = "number")]
    pub partner_id: i64,
    /// Percent text with up to 2 decimals, 0 to 100, for example `40` or `33.33`.
    pub share: String,
}

/// A new set of shares for all active partners (PRD FR-17.1, FR-17.2).
#[derive(Debug, Clone, Deserialize, TS)]
pub struct PartnersSetSharesInput {
    /// The first day the shares apply, `DD-MM-YYYY`. Today or later (`ERR_SHARE_BACKDATED`).
    pub effective_from: String,
    /// One row per active partner, each once, totalling exactly 100%.
    pub shares: Vec<ShareInput>,
}

/// A set of shares to check, nothing saved.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct PartnersCheckSharesInput {
    /// The shares as typed.
    pub shares: Vec<ShareInput>,
}

/// The running total of typed shares, worked out by the backend so the screen never adds
/// percents itself (CLAUDE.md rule 1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct SharesCheck {
    /// The total of the rows that could be read, in bps. 10,000 is exactly 100%.
    pub total_bps: i32,
    /// True when the total is exactly 10,000 and nothing else is wrong.
    pub is_100: bool,
    /// What is wrong, with field paths `shares[i].share` or `shares`. Empty if nothing.
    pub errors: Vec<LineError>,
}

/// One partner on the overview (PRD 17.3). The returned, خالص, capital invested and کل رقم
/// columns come in plan 17.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct PartnerOverviewRow {
    /// The partner.
    #[ts(type = "number")]
    pub partner_id: i64,
    /// Urdu name.
    pub name_ur: String,
    /// English name.
    pub name_en: String,
    /// His share today in bps.
    pub share_bps: i32,
    /// His income share of today (آج کا حصہ).
    #[ts(type = "Paisa")]
    pub today_income_paisa: Paisa,
    /// His income share this calendar month up to today (مہینے کا حصہ).
    #[ts(type = "Paisa")]
    pub month_income_paisa: Paisa,
    /// His income share this calendar year up to today (سال کا حصہ).
    #[ts(type = "Paisa")]
    pub year_income_paisa: Paisa,
    /// His expense share this month, as a positive number (خرچہ حصہ).
    #[ts(type = "Paisa")]
    pub month_expense_paisa: Paisa,
    /// Income minus expense this month (منافع حصہ, PRD C12).
    #[ts(type = "Paisa")]
    pub month_profit_paisa: Paisa,
    /// Money he took this month: the debits of his drawings account (ذاتی خرچہ).
    #[ts(type = "Paisa")]
    pub month_taken_paisa: Paisa,
    /// The credit balance of his capital account (سرمایہ بقایا). Negative if it stands on the
    /// debit side.
    #[ts(type = "Paisa")]
    pub capital_balance_paisa: Paisa,
}

/// The shares as typed, read one by one (PRD FR-17.1). Rows that cannot be read are left out of
/// `shares` and named in `errors`; the total counts only the rows that could be read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadShares {
    /// `(partner id, share)` for each row that could be read, in the order typed.
    pub shares: Vec<(i64, Bps)>,
    /// The total of those rows in bps.
    pub total_bps: i32,
    /// Field errors, path `shares[i].share`, one per unreadable row.
    pub errors: Vec<FieldError>,
}

/// Reads the typed percents with `parse_bps` (up to 2 decimals, 0 to 100). This is the one
/// parser of the save and of the read-only check, so the screen and the save never differ
/// (Code Standards B5).
///
/// Example: `["40", "30", "30"]` give 4000, 3000, 3000 and a total of 10,000.
pub fn read_shares(shares: &[ShareInput]) -> ReadShares {
    let mut read = Vec::with_capacity(shares.len());
    let mut errors = Vec::new();
    let mut total: i64 = 0;
    for (index, row) in shares.iter().enumerate() {
        match parse_bps(&row.share) {
            Ok(bps) => {
                total = total.saturating_add(i64::from(bps.get()));
                read.push((row.partner_id, bps));
            }
            Err(code) => errors.push(FieldError::new(code, &format!("shares[{index}].share"))),
        }
    }
    ReadShares {
        shares: read,
        total_bps: i32::try_from(total).unwrap_or(i32::MAX),
        errors,
    }
}

/// Checks that the active partners are each named once, and nobody else (PRD FR-17.1).
/// `ERR_PARTNER_NOT_LISTED` on `shares` if one is missing, repeated or not an active partner.
pub fn check_listed(shares: &[ShareInput], active_ids: &[i64]) -> Option<FieldError> {
    let mut named: Vec<i64> = shares.iter().map(|s| s.partner_id).collect();
    named.sort_unstable();
    let mut wanted: Vec<i64> = active_ids.to_vec();
    wanted.sort_unstable();
    (named != wanted).then(|| FieldError::new(ErrorCode::ErrPartnerNotListed, "shares"))
}

/// The error of a total that is not exactly 100% (`ERR_SHARES_NOT_100`, value `total_bps`).
pub fn not_100(total_bps: i32) -> FieldError {
    FieldError::new(ErrorCode::ErrSharesNot100, "shares")
        .with("total_bps", ErrorValue::Number(i64::from(total_bps)))
}

/// The running total of typed shares for the screen: the same reading and the same total as the
/// save. `active_ids` are the active partners; pass `None` to skip the check that every active
/// partner is listed (when the caller does not have them).
///
/// Example: shares 40, 30, 20 give `total_bps` 9,000, `is_100` false and `ERR_SHARES_NOT_100`.
pub fn check_shares(shares: &[ShareInput], active_ids: Option<&[i64]>) -> SharesCheck {
    let read = read_shares(shares);
    let mut errors = read.errors;
    if let Some(ids) = active_ids {
        errors.extend(check_listed(shares, ids));
    }
    if read.total_bps != BPS_FULL {
        errors.push(not_100(read.total_bps));
    }
    SharesCheck {
        total_bps: read.total_bps,
        is_100: errors.is_empty(),
        errors: errors
            .into_iter()
            .map(|e| LineError {
                code: e.code,
                field: e.field,
                values: e.values,
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(rows: &[(i64, &str)]) -> Vec<ShareInput> {
        rows.iter()
            .map(|&(partner_id, share)| ShareInput {
                partner_id,
                share: share.to_owned(),
            })
            .collect()
    }

    #[test]
    fn forty_thirty_thirty_is_100() {
        let check = check_shares(&input(&[(1, "40"), (2, "30"), (3, "30")]), Some(&[1, 2, 3]));
        assert_eq!(check.total_bps, 10_000);
        assert!(check.is_100);
        assert!(check.errors.is_empty());
    }

    #[test]
    fn two_decimals_are_read_exactly() {
        let read = read_shares(&input(&[(1, "33.33"), (2, "33.33"), (3, "33.34")]));
        assert_eq!(read.total_bps, 10_000);
        assert_eq!(
            read.shares.iter().map(|(_, b)| b.get()).collect::<Vec<_>>(),
            vec![3333, 3333, 3334]
        );
    }

    #[test]
    fn nine_thousand_is_not_100() {
        let check = check_shares(&input(&[(1, "40"), (2, "30"), (3, "20")]), None);
        assert_eq!(check.total_bps, 9_000);
        assert!(!check.is_100);
        assert_eq!(check.errors.len(), 1);
        assert_eq!(check.errors[0].code, ErrorCode::ErrSharesNot100);
        assert_eq!(
            check.errors[0].values.get("total_bps"),
            Some(&ErrorValue::Number(9_000))
        );
    }

    #[test]
    fn a_bad_row_is_named_by_its_place() {
        let check = check_shares(&input(&[(1, "40"), (2, "abc"), (3, "2.555")]), None);
        let fields: Vec<(&str, ErrorCode)> = check
            .errors
            .iter()
            .map(|e| (e.field.as_str(), e.code))
            .collect();
        assert!(fields.contains(&("shares[1].share", ErrorCode::ErrNotANumber)));
        assert!(fields.contains(&("shares[2].share", ErrorCode::ErrTooManyDecimals)));
        assert!(!check.is_100);
        // The rows that could be read still count.
        assert_eq!(check.total_bps, 4_000);
    }

    #[test]
    fn over_100_percent_is_refused() {
        let check = check_shares(&input(&[(1, "101")]), None);
        assert_eq!(check.errors[0].code, ErrorCode::ErrPercentOver100);
    }

    #[test]
    fn every_active_partner_is_named_once() {
        let rows = input(&[(1, "50"), (2, "50")]);
        assert!(check_listed(&rows, &[1, 2]).is_none());
        assert!(check_listed(&rows, &[1, 2, 3]).is_some(), "3 is missing");
        assert!(check_listed(&rows, &[1]).is_some(), "2 is not active");
        let twice = input(&[(1, "50"), (1, "50")]);
        assert!(check_listed(&twice, &[1]).is_some(), "1 is named twice");
        let check = check_shares(&twice, Some(&[1]));
        assert!(
            check
                .errors
                .iter()
                .any(|e| e.code == ErrorCode::ErrPartnerNotListed && e.field == "shares")
        );
    }

    #[test]
    fn an_empty_list_is_not_100() {
        let check = check_shares(&[], None);
        assert_eq!(check.total_bps, 0);
        assert!(!check.is_100);
    }
}
