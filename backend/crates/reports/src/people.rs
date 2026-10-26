//! Global search for one-time people (PRD FR-5.4, FR-7.40, FR-13.5): `Ctrl+K` finds a person of
//! the پختہ کھاتہ accounts by either name or his phone, and gives the accounts his lines are on,
//! so the screen can open his ledger (R13) or his register (R9) with the account and his
//! person key.
//!
//! The accounts, the بکری and the vouchers of the global search are found by
//! `search_global` (plan 06, 08, 13, 15); this is the missing kind. Both sides are searched at
//! once and a person who is both a one-time buyer and a one-time seller is one hit with two
//! accounts. The match is the one of the pickers: letter variants unified, close English
//! spellings by the phonetic key, a phone by its digits. Read only (Code Standards B29).
//!
//! Example (PRD 4.5): "shahid" or "شاہد" or "0300" finds شاہد (phone 03000000000) once, with the
//! one-time buyer account 0030.

use std::collections::BTreeMap;

use mandi_core::bikri::PukhtaSide;
use mandi_core::error::{ErrorCode, FieldError};
use mandi_core::text;
use mandi_db::Db;
use mandi_db::bikri::saved::pukhta_people;
use mandi_db::rusqlite::{Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::ReportsError;

/// The most hits one search may ask for.
pub const MAX_PEOPLE: u32 = 50;
/// The hits when the screen sends 0.
pub const DEFAULT_PEOPLE: u32 = 10;
/// The longest text accepted.
const MAX_TEXT_CHARS: usize = 100;

/// The code of the one-time buyer account and of the one-time seller account (PRD FR-13.1).
const BUYER_CODE: &str = mandi_core::codes::PUKHTA_BUYER_CODE;
const SELLER_CODE: &str = mandi_core::codes::PUKHTA_SELLER_CODE;

/// Input of `find_people`.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct FindPeopleInput {
    /// What was typed. Raw, never converted. Empty lists the newest people.
    pub text: String,
    /// The most hits. 0 means 10; at most 50.
    #[ts(type = "number")]
    pub limit: u32,
}

/// One account a person has lines on.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[ts(rename = "PukhtaPersonAccount")]
pub struct PersonAccount {
    /// Buyer or seller.
    pub side: PukhtaSide,
    /// The پختہ کھاتہ account, to send as `party.account_id` with his `person_key`.
    #[ts(type = "number")]
    pub account_id: i64,
}

/// One person found.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[ts(rename = "PukhtaPersonHit")]
pub struct PersonHit {
    /// Urdu name, as last typed.
    pub name_ur: String,
    /// English name, as last typed.
    pub name_en: String,
    /// Phone as 11 digits. Empty if none.
    pub phone: String,
    /// The day of his last line, `YYYY-MM-DD`.
    pub last_date: String,
    /// The key that ties his lines together (PRD FR-13.5).
    pub person_key: String,
    /// The accounts he has lines on: one, or two when he is both.
    pub accounts: Vec<PersonAccount>,
}

fn digits(text: &str) -> String {
    text.chars().filter(char::is_ascii_digit).collect()
}

/// True if the typed text is part of the person's names or phone. Empty text matches everybody.
fn matches(typed: &str, name_ur: &str, name_en: &str, phone: &str) -> bool {
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
    let typed_digits = digits(typed);
    !typed_digits.is_empty() && phone.contains(&typed_digits)
}

fn account_of(conn: &Connection, code: &str) -> Result<Option<i64>, ReportsError> {
    Ok(conn
        .query_row("SELECT id FROM accounts WHERE code = ?1", [code], |r| {
            r.get(0)
        })
        .optional()?)
}

/// The one-time people that match the typed text, newest first.
///
/// # Errors
/// `ERR_OUT_OF_RANGE` on `text` (longer than 100 letters or with a control character) and on
/// `limit` (above 50); a database error.
pub fn find_people(db: &Db, input: &FindPeopleInput) -> Result<Vec<PersonHit>, ReportsError> {
    let mut problems = Vec::new();
    if input.text.chars().count() > MAX_TEXT_CHARS || input.text.chars().any(char::is_control) {
        problems.push(FieldError::new(ErrorCode::ErrOutOfRange, "text"));
    }
    if input.limit > MAX_PEOPLE {
        problems.push(FieldError::new(ErrorCode::ErrOutOfRange, "limit"));
    }
    if !problems.is_empty() {
        return Err(ReportsError::Rejected(problems));
    }
    let limit = usize::try_from(if input.limit == 0 {
        DEFAULT_PEOPLE
    } else {
        input.limit
    })
    .unwrap_or(usize::MAX);
    db.read(|conn| {
        let buyer = account_of(conn, BUYER_CODE)?;
        let seller = account_of(conn, SELLER_CODE)?;
        // person key → (hit, newest first order: the date then the key)
        let mut found: BTreeMap<String, PersonHit> = BTreeMap::new();
        for (side, account) in [(PukhtaSide::Buyer, buyer), (PukhtaSide::Seller, seller)] {
            let Some(account_id) = account else { continue };
            for person in pukhta_people(conn, side)? {
                if !matches(&input.text, &person.name_ur, &person.name_en, &person.phone) {
                    continue;
                }
                let hit = found
                    .entry(person.person_key.clone())
                    .or_insert_with(|| PersonHit {
                        name_ur: person.name_ur.clone(),
                        name_en: person.name_en.clone(),
                        phone: person.phone.clone(),
                        last_date: person.last_date.clone(),
                        person_key: person.person_key.clone(),
                        accounts: Vec::new(),
                    });
                // The names are those of his newest line on either side.
                if person.last_date > hit.last_date {
                    hit.name_ur.clone_from(&person.name_ur);
                    hit.name_en.clone_from(&person.name_en);
                    hit.phone.clone_from(&person.phone);
                    hit.last_date.clone_from(&person.last_date);
                }
                hit.accounts.push(PersonAccount { side, account_id });
            }
        }
        let mut hits: Vec<PersonHit> = found.into_values().collect();
        hits.sort_by(|a, b| {
            (b.last_date.as_str(), a.person_key.as_str())
                .cmp(&(a.last_date.as_str(), b.person_key.as_str()))
        });
        hits.truncate(limit);
        Ok::<_, ReportsError>(hits)
    })
}

#[cfg(test)]
mod tests {
    use super::matches;

    #[test]
    fn a_person_is_found_by_either_name_a_close_spelling_or_the_phone() {
        assert!(matches("shahid", "شاہد", "Shahid", "03000000000"));
        assert!(matches("شاہد", "شاہد", "Shahid", "03000000000"));
        assert!(matches("shahd", "شاہد", "Shahid", "03000000000"));
        assert!(matches("0300", "شاہد", "Shahid", "03000000000"));
        assert!(matches("03-00", "شاہد", "Shahid", "03000000000"));
        assert!(matches("", "شاہد", "Shahid", ""));
        assert!(!matches("nasir", "شاہد", "Shahid", "03000000000"));
        assert!(!matches("0311", "شاہد", "Shahid", "03000000000"));
    }
}
