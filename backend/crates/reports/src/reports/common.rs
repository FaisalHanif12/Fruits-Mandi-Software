//! Small read helpers shared by the reports: today, accounts by id, names of masters, the row
//! source of a journal entry. Read only (PRD FR-18.8: reports write nothing).

use std::collections::HashMap;

use chrono::{DateTime, Duration, Utc};
use mandi_core::accounts::{Balance, BalanceSide};
use mandi_core::dates::BusinessDate;
use mandi_core::money::{MoneyError, Paisa};
use mandi_db::rusqlite::{Connection, params_from_iter};

use super::model::{Cell, Names, PartyRef, RowSource, SourceKind};
use crate::error::ReportsError;

/// Pakistan time is UTC+5 all year (Code Standards G9).
const PAKISTAN_OFFSET_HOURS: i64 = 5;

/// How many ids go into one `IN (...)` list. SQLite allows far more; a page is at most 1,000
/// rows, and an "every row" read for a print goes in chunks.
pub(crate) const IN_CHUNK: usize = 500;

/// Today in Pakistan, from the database clock (the same clock the audit rows use).
///
/// # Errors
/// `ReportsError::Data` if the clock gives a day outside the calendar.
pub(crate) fn today(now_utc: DateTime<Utc>) -> Result<BusinessDate, ReportsError> {
    let local = now_utc + Duration::hours(PAKISTAN_OFFSET_HOURS);
    BusinessDate::parse_iso(&local.format("%Y-%m-%d").to_string())
        .ok_or_else(|| ReportsError::Data("the clock is outside the calendar".to_owned()))
}

/// `?,?,?` for a list of `count` parameters.
pub(crate) fn placeholders(count: usize) -> String {
    vec!["?"; count].join(",")
}

/// A checked paisa from a stored integer. A stored amount is a multiple of 100 (a CHECK of every
/// money column), so a failure here is a bug signal, not a user error.
pub(crate) fn paisa(value: i64) -> Result<Paisa, ReportsError> {
    Paisa::try_from_paisa(value).map_err(ReportsError::Money)
}

/// A balance as a cell.
pub(crate) fn balance_cell(balance: Balance) -> Cell {
    Cell::Balance {
        paisa: balance.amount_paisa,
        side: balance.side,
    }
}

/// A debit or credit amount as a cell: money, or empty when it is 0.
pub(crate) fn amount_or_empty(amount: Paisa) -> Cell {
    if amount == Paisa::ZERO {
        Cell::Empty
    } else {
        Cell::Money { paisa: amount }
    }
}

/// The balance of an account as the debit and the credit cell of a trial balance: the net on its
/// side, nothing on the other (money agent Section 9: the difference is worked out in Rust with
/// checked maths).
///
/// # Errors
/// `ReportsError::Money` if the difference does not fit.
pub(crate) fn dr_cr_cells(dr: Paisa, cr: Paisa) -> Result<(Cell, Cell, Balance), MoneyError> {
    let balance = Balance::from_totals(dr, cr)?;
    let (left, right) = match balance.side {
        BalanceSide::Dr => (
            Cell::Money {
                paisa: balance.amount_paisa,
            },
            Cell::Empty,
        ),
        BalanceSide::Cr => (
            Cell::Empty,
            Cell::Money {
                paisa: balance.amount_paisa,
            },
        ),
        BalanceSide::Zero => (Cell::Empty, Cell::Empty),
    };
    Ok((left, right, balance))
}

/// Accounts by id, with their names, tag and short code (PRD FR-19.13).
///
/// # Errors
/// Any SQLite error.
pub(crate) fn load_parties(
    conn: &Connection,
    ids: &[i64],
) -> Result<HashMap<i64, PartyRef>, ReportsError> {
    let mut found = HashMap::new();
    let mut unique: Vec<i64> = ids.to_vec();
    unique.sort_unstable();
    unique.dedup();
    for chunk in unique.chunks(IN_CHUNK) {
        let sql = format!(
            "SELECT id, code, serial, party_type, name_ur, name_en, tag FROM accounts
             WHERE id IN ({})",
            placeholders(chunk.len())
        );
        let mut statement = conn.prepare(&sql)?;
        let rows = statement.query_map(params_from_iter(chunk.iter()), |r| {
            let party_type: String = r.get(3)?;
            let serial: i64 = r.get(2)?;
            let short_code =
                matches!(party_type.as_str(), "seller" | "buyer").then(|| serial.to_string());
            Ok(PartyRef {
                account_id: r.get(0)?,
                code: r.get(1)?,
                short_code,
                name_ur: r.get(4)?,
                name_en: r.get(5)?,
                tag: r.get(6)?,
                person_key: None,
            })
        })?;
        for row in rows {
            let party = row?;
            found.insert(party.account_id, party);
        }
    }
    Ok(found)
}

/// One account, if it exists.
///
/// # Errors
/// Any SQLite error.
pub(crate) fn load_party(conn: &Connection, id: i64) -> Result<Option<PartyRef>, ReportsError> {
    Ok(load_parties(conn, &[id])?.remove(&id))
}

/// The party of a line: the account and, for a پختہ کھاتہ line, the person key.
pub(crate) fn party_of_line(
    parties: &HashMap<i64, PartyRef>,
    account_id: i64,
    person_key: Option<String>,
) -> Result<PartyRef, ReportsError> {
    let mut party = parties.get(&account_id).cloned().ok_or_else(|| {
        ReportsError::Data(format!("a line is on an unknown account {account_id}"))
    })?;
    party.person_key = person_key;
    Ok(party)
}

/// The two names of a master row (`cities`, `items`, `addas`), if it exists. The table name is
/// one of our own constants, never user input.
///
/// # Errors
/// Any SQLite error.
pub(crate) fn names_of(
    conn: &Connection,
    table: MasterTable,
    id: i64,
) -> Result<Option<Names>, ReportsError> {
    use mandi_db::rusqlite::OptionalExtension;
    let sql = format!(
        "SELECT name_ur, name_en FROM {} WHERE id = ?1",
        table.name()
    );
    Ok(conn
        .query_row(&sql, [id], |r| {
            Ok(Names {
                ur: r.get(0)?,
                en: r.get(1)?,
            })
        })
        .optional()?)
}

/// The master tables a filter can point at.
#[derive(Debug, Clone, Copy)]
pub(crate) enum MasterTable {
    /// `items`.
    Items,
    /// `cities`.
    Cities,
    /// `addas`.
    Addas,
}

impl MasterTable {
    fn name(self) -> &'static str {
        match self {
            MasterTable::Items => "items",
            MasterTable::Cities => "cities",
            MasterTable::Addas => "addas",
        }
    }
}

/// The record a journal entry opens (PRD FR-19.4). An entry whose source has no screen opens
/// itself.
pub(crate) fn source_of(source_type: &str, source_id: i64, entry_id: i64) -> RowSource {
    let (kind, id) = match source_type {
        "bikri" => (SourceKind::Bikri, source_id),
        "voucher" => (SourceKind::Voucher, source_id),
        "account" => (SourceKind::Account, source_id),
        "month_close" => (SourceKind::MonthClose, entry_id),
        "purchase" => (SourceKind::Purchase, source_id),
        "venture" => (SourceKind::Venture, source_id),
        "fruit" => (SourceKind::Fruit, source_id),
        _ => (SourceKind::Entry, entry_id),
    };
    RowSource { kind, id }
}
