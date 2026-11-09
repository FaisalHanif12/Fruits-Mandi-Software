//! The چکتا buyers (plan 64, PRD Section 13, FR-7.29, FR-13.1 to FR-13.9): walk-in buyers who pay the
//! whole bill on the spot and leave. The list `chukta_buyers` keeps each one by name, so he is found
//! again and his name is spelt the same way next time.
//!
//! A چکتا buyer is not an account. A sale line to him is on the system account چکتا کھاتہ (`0030`) with
//! his names, phone and person key on the line (`bikri::ChuktaInput`). He has no ledger, no balance and
//! no due. This file holds the shapes of the two commands and the pure check of a new buyer.
//!
//! | Command | Input | Answer |
//! |---|---|---|
//! | `chukta_search` | [`ChuktaSearchInput`] | `Vec<ChuktaBuyer>` (newest first, at most 10) |
//! | `chukta_add` | [`ChuktaAddInput`] | [`ChuktaBuyer`] |
//!
//! Field paths in `chukta_add` errors: `name_ur`, `name_en`, `phone`.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::{ErrorCode, FieldError};
use crate::{text, validate};

/// The most buyers `chukta_search` gives (PRD FR-7.29).
pub const SEARCH_LIMIT: usize = 10;

/// One چکتا buyer of the list, as the picker shows him.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct ChuktaBuyer {
    /// Row id in `chukta_buyers`.
    #[ts(type = "number")]
    pub id: i64,
    /// The id of the account چکتا کھاتہ (`0030`), which a sale line to him takes as its buyer
    /// account (`SaleInput.buyer_account_id`). The same for every buyer of the list.
    #[ts(type = "number")]
    pub account_id: i64,
    /// Urdu name, as last typed.
    pub name_ur: String,
    /// English name, as last typed.
    pub name_en: String,
    /// Phone as 11 digits, or null if he gave none.
    pub phone: Option<String>,
    /// The day of his last sale line, `YYYY-MM-DD`, or null if he has none yet.
    pub last_date: Option<String>,
}

/// What was typed in the picker of a بکری line (PRD FR-7.29). Matches either name or the phone, in
/// either script. Empty text lists the newest buyers.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct ChuktaSearchInput {
    /// The text, raw.
    pub text: String,
}

/// A new چکتا buyer, added on the spot from the picker (PRD FR-7.69, FR-13.3).
#[derive(Debug, Clone, Deserialize, TS)]
pub struct ChuktaAddInput {
    /// Urdu name.
    pub name_ur: String,
    /// English name.
    pub name_en: String,
    /// Phone, `03XXXXXXXXX` or `03XX-XXXXXXX`. Null or empty if he gave none.
    #[serde(default)]
    pub phone: Option<String>,
}

/// A چکتا buyer whose names and phone passed the checks, ready to save.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CleanChukta {
    /// Urdu name, cleaned (NFC, no invisible marks, single spaces).
    pub name_ur: String,
    /// English name, cleaned.
    pub name_en: String,
    /// Phone as 11 digits, or `None`.
    pub phone: Option<String>,
    /// The person key: the normalised Urdu name and the phone digits (PRD FR-13.5). One person is
    /// one key.
    pub key: String,
}

/// Checks a new چکتا buyer: both names, each in its own script (PRD FR-7.28), and a phone that is
/// valid when typed. An empty name is `ERR_CHUKTA_NAME_REQUIRED`. All errors come together.
///
/// Example: `شاہد`, `Shahid` and `0300-0000000` give the key `شاہد|03000000000` and the phone
/// `03000000000`.
///
/// # Errors
/// `ERR_CHUKTA_NAME_REQUIRED`, `ERR_NAME_UR_SCRIPT`, `ERR_NAME_EN_SCRIPT` or `ERR_PHONE_INVALID`, on
/// `name_ur`, `name_en` or `phone`.
pub fn clean(input: &ChuktaAddInput) -> Result<CleanChukta, Vec<FieldError>> {
    let mut errors = Vec::new();
    let name_ur = name_part(&mut errors, "name_ur", &input.name_ur, validate::name_ur);
    let name_en = name_part(&mut errors, "name_en", &input.name_en, validate::name_en);
    let phone = match validate::phone_optional("phone", input.phone.as_deref().unwrap_or("")) {
        Ok(phone) => phone,
        Err(e) => {
            errors.push(e);
            None
        }
    };
    match (name_ur, name_en) {
        (Some(name_ur), Some(name_en)) if errors.is_empty() => {
            let key = text::person_key(&name_ur, phone.as_deref().unwrap_or(""));
            Ok(CleanChukta {
                name_ur,
                name_en,
                phone,
                key,
            })
        }
        _ => Err(errors),
    }
}

fn name_part(
    errors: &mut Vec<FieldError>,
    field: &str,
    typed: &str,
    check: fn(&str, &str) -> Result<String, FieldError>,
) -> Option<String> {
    if typed.trim().is_empty() {
        errors.push(FieldError::new(ErrorCode::ErrChuktaNameRequired, field));
        return None;
    }
    match check(field, typed) {
        Ok(name) => Some(name),
        Err(e) => {
            errors.push(e);
            None
        }
    }
}

/// True if the typed text is part of the buyer's names or phone (PRD FR-7.40 style: both scripts,
/// letter variants unified, close English spellings by the phonetic key). Empty text matches
/// everybody.
///
/// Example: `shahid` and `شاہ` both find شاہد / Shahid; `0300` finds a phone that has it.
pub fn matches(typed: &str, name_ur: &str, name_en: &str, phone: Option<&str>) -> bool {
    let typed = typed.trim();
    if typed.is_empty() {
        return true;
    }
    let ur = text::search_ur(typed);
    if !ur.is_empty() && text::search_ur(name_ur).contains(&ur) {
        return true;
    }
    let en = text::search_en(typed);
    if !en.is_empty() && text::search_en(name_en).contains(&en) {
        return true;
    }
    let phonetic = text::search_phonetic(typed);
    if !phonetic.is_empty() && text::search_phonetic(name_en).contains(&phonetic) {
        return true;
    }
    let digits: String = typed.chars().filter(char::is_ascii_digit).collect();
    !digits.is_empty() && phone.is_some_and(|p| p.contains(&digits))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(ur: &str, en: &str, phone: Option<&str>) -> ChuktaAddInput {
        ChuktaAddInput {
            name_ur: ur.into(),
            name_en: en.into(),
            phone: phone.map(str::to_owned),
        }
    }

    #[test]
    fn shahid_of_prd_4_5_is_cleaned_with_his_key() {
        let clean = clean(&input("شاہد", "Shahid", Some("0300-0000000"))).unwrap();
        assert_eq!(clean.phone.as_deref(), Some("03000000000"));
        assert_eq!(clean.key, "شاہد|03000000000");
        assert_eq!(
            (clean.name_ur.as_str(), clean.name_en.as_str()),
            ("شاہد", "Shahid")
        );
    }

    #[test]
    fn a_phone_is_optional_and_makes_another_key() {
        let with = clean(&input("ناصر", "Nasir", Some("03001111111"))).unwrap();
        let without = clean(&input("ناصر", "Nasir", None)).unwrap();
        let empty = clean(&input("ناصر", "Nasir", Some("  "))).unwrap();
        assert_eq!(without.phone, None);
        assert_eq!(without.key, empty.key);
        assert_ne!(with.key, without.key);
    }

    #[test]
    fn every_error_comes_together_on_its_field() {
        let errors = clean(&input("", "Shahid", Some("123"))).unwrap_err();
        let fields: Vec<(&str, ErrorCode)> =
            errors.iter().map(|e| (e.field.as_str(), e.code)).collect();
        assert_eq!(
            fields,
            vec![
                ("name_ur", ErrorCode::ErrChuktaNameRequired),
                ("phone", ErrorCode::ErrPhoneInvalid)
            ]
        );
        let wrong_script = clean(&input("Shahid", "شاہد", None)).unwrap_err();
        assert_eq!(wrong_script.len(), 2);
    }

    #[test]
    fn search_finds_by_either_script_and_by_phone() {
        assert!(matches("shahid", "شاہد", "Shahid", None));
        assert!(matches("شاہ", "شاہد", "Shahid", None));
        assert!(matches("0300", "شاہد", "Shahid", Some("03000000000")));
        assert!(matches("", "شاہد", "Shahid", None));
        assert!(!matches("nasir", "شاہد", "Shahid", Some("03000000000")));
    }
}
