//! The list of چکتا buyers (plan 64, PRD FR-7.29, FR-13.2, migration 0020): `chukta_buyers`.
//!
//! One row for each person, found by his key (the normalised Urdu name and the phone digits, built
//! by `mandi_core::text::person_key`). A row is never deleted. A person is added on the spot from
//! the buyer picker of a بکری line, or by a sale line that names him. The sale lines themselves
//! carry his names, phone and key (`bikri_sales.chukta_*`); this table is only the list the picker
//! offers, so a name is spelt the same way next time.

use mandi_core::chukta::{ChuktaBuyer, CleanChukta};
use mandi_core::text;
use rusqlite::{Connection, OptionalExtension, Row, params};

/// Every active buyer of the list with the day of his last sale line, newest first (by that day,
/// then by when he was added). A buyer with no line yet comes after those who have one. The
/// service keeps the ones that match the typed text and the first ten.
///
/// # Errors
/// Any SQLite error.
pub fn all_newest_first(conn: &Connection) -> rusqlite::Result<Vec<ChuktaBuyer>> {
    let mut statement = conn.prepare(
        "SELECT b.id, b.name_ur, b.name_en, b.phone,
                (SELECT MAX(s.entry_date) FROM bikri_sales s
                 WHERE s.chukta_person_key = b.person_key) AS last_date,
                a.id
         FROM chukta_buyers b JOIN accounts a ON a.code = ?1
         WHERE b.is_active = 1
         ORDER BY last_date IS NULL, last_date DESC, b.id DESC",
    )?;
    let rows = statement.query_map([mandi_core::codes::CHUKTA_KHATA_CODE], from_row)?;
    rows.collect()
}

/// One buyer by id, active or not, with the day of his last sale line.
///
/// # Errors
/// Any SQLite error.
pub fn get(conn: &Connection, id: i64) -> rusqlite::Result<Option<ChuktaBuyer>> {
    conn.query_row(
        "SELECT b.id, b.name_ur, b.name_en, b.phone,
                (SELECT MAX(s.entry_date) FROM bikri_sales s
                 WHERE s.chukta_person_key = b.person_key),
                a.id
         FROM chukta_buyers b JOIN accounts a ON a.code = ?2
         WHERE b.id = ?1",
        params![id, mandi_core::codes::CHUKTA_KHATA_CODE],
        from_row,
    )
    .optional()
}

/// One buyer by his person key, active or not, with the day of his last sale line.
///
/// # Errors
/// Any SQLite error.
pub fn by_key(conn: &Connection, person_key: &str) -> rusqlite::Result<Option<ChuktaBuyer>> {
    conn.query_row(
        "SELECT b.id, b.name_ur, b.name_en, b.phone,
                (SELECT MAX(s.entry_date) FROM bikri_sales s
                 WHERE s.chukta_person_key = b.person_key),
                a.id
         FROM chukta_buyers b JOIN accounts a ON a.code = ?2
         WHERE b.person_key = ?1",
        params![person_key, mandi_core::codes::CHUKTA_KHATA_CODE],
        from_row,
    )
    .optional()
}

fn from_row(r: &Row<'_>) -> rusqlite::Result<ChuktaBuyer> {
    Ok(ChuktaBuyer {
        id: r.get(0)?,
        account_id: r.get(5)?,
        name_ur: r.get(1)?,
        name_en: r.get(2)?,
        phone: r.get(3)?,
        last_date: r.get(4)?,
    })
}

/// Adds a buyer to the list unless his key is there already, and returns the row id and whether it
/// was added. The names of an existing row are not changed: the list is the master spelling, and
/// the search keys are built only by `core::text` (Code Standards B26).
///
/// # Errors
/// Any SQLite error.
pub fn ensure(
    conn: &Connection,
    person: &CleanChukta,
    user_id: i64,
    now: &str,
) -> rusqlite::Result<(i64, bool)> {
    let added = conn.execute(
        "INSERT INTO chukta_buyers (created_at, created_by, updated_at, updated_by, person_key,
             name_ur, name_en, search_ur, search_en, search_phonetic, phone)
         VALUES (?1, ?2, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
         ON CONFLICT(person_key) DO NOTHING",
        params![
            now,
            user_id,
            person.key,
            person.name_ur,
            person.name_en,
            text::search_ur(&person.name_ur),
            text::search_en(&person.name_en),
            text::search_phonetic(&person.name_en),
            person.phone,
        ],
    )?;
    let id: i64 = conn.query_row(
        "SELECT id FROM chukta_buyers WHERE person_key = ?1",
        [&person.key],
        |r| r.get(0),
    )?;
    Ok((id, added == 1))
}

/// A چکتا buyer as the in-memory search index of the buyer field needs him (plan 64, PRD FR-7.29).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChuktaSource {
    /// Row id in `chukta_buyers`.
    pub id: i64,
    /// The id of the account چکتا کھاتہ (`0030`): what a sale line takes as its buyer account.
    pub account_id: i64,
    /// The id of the buyer control (`0304`), so his hit has the control of a buyer.
    pub control_id: i64,
    /// Urdu name.
    pub name_ur: String,
    /// English name.
    pub name_en: String,
    /// Phone as 11 digits, if he gave one.
    pub phone: Option<String>,
    /// The three search keys, as stored.
    pub search_ur: String,
    /// See `search_ur`.
    pub search_en: String,
    /// See `search_ur`.
    pub search_phonetic: String,
    /// False when hidden from the picker.
    pub is_active: bool,
}

/// The buyers of the list for the search index: all of them, or one by id. Empty when the account
/// `0030` does not exist (it always does after migration 20).
///
/// # Errors
/// Any SQLite error.
pub fn search_sources(
    conn: &Connection,
    only_id: Option<i64>,
) -> rusqlite::Result<Vec<ChuktaSource>> {
    let mut statement = conn.prepare(
        "SELECT b.id, a.id, a.control_id, b.name_ur, b.name_en, b.phone,
                b.search_ur, b.search_en, b.search_phonetic, b.is_active
         FROM chukta_buyers b JOIN accounts a ON a.code = ?2
         WHERE (?1 IS NULL OR b.id = ?1)
         ORDER BY b.id",
    )?;
    let rows = statement.query_map(
        params![only_id, mandi_core::codes::CHUKTA_KHATA_CODE],
        |r| {
            Ok(ChuktaSource {
                id: r.get(0)?,
                account_id: r.get(1)?,
                control_id: r.get(2)?,
                name_ur: r.get(3)?,
                name_en: r.get(4)?,
                phone: r.get(5)?,
                search_ur: r.get(6)?,
                search_en: r.get(7)?,
                search_phonetic: r.get(8)?,
                is_active: r.get(9)?,
            })
        },
    )?;
    rows.collect()
}
