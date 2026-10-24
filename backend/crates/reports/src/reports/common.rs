//! Small read helpers shared by the reports: today, accounts by id, names of masters, the row
//! source of a journal entry. Read only (PRD FR-18.8: reports write nothing).

use std::borrow::Cow;
use std::collections::HashMap;

use chrono::{DateTime, Duration, Utc};
use mandi_core::accounts::{Balance, BalanceSide};
use mandi_core::dates::BusinessDate;
use mandi_core::money::{MoneyError, Paisa};
use mandi_db::rusqlite::types::Value;
use mandi_db::rusqlite::{Connection, params_from_iter};

use super::model::{Cell, Column, ColumnKind, Names, PartyRef, RowSource, SourceKind};
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
    /// `partners`.
    Partners,
}

impl MasterTable {
    fn name(self) -> &'static str {
        match self {
            MasterTable::Items => "items",
            MasterTable::Cities => "cities",
            MasterTable::Addas => "addas",
            MasterTable::Partners => "partners",
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

/// The live-entry condition on `journal_entries e` (a posted entry that is not a reversal and is
/// not reversed). The same predicate as `mandi_db::matching`, kept here because that constant is
/// private to the db crate. A reversed entry and its reversal cancel each other, so neither is a
/// real posting.
pub(crate) const LIVE: &str = "e.status = 'posted' AND e.reverses_entry_id IS NULL
    AND NOT EXISTS (SELECT 1 FROM journal_entries r WHERE r.reverses_entry_id = e.id AND r.status = 'posted')";

/// A column of a report.
pub(crate) fn col(key: &str, title: &str, kind: ColumnKind) -> Column {
    Column {
        key: key.to_owned(),
        title_id: title.to_owned(),
        kind,
    }
}

/// A count of rows as the `i64` of the contract.
pub(crate) fn count_of(len: usize) -> i64 {
    i64::try_from(len).unwrap_or(i64::MAX)
}

/// A name cell from two stored names, or an empty cell when the record has none.
pub(crate) fn names_cell(ur: Option<String>, en: Option<String>) -> Cell {
    let ur = ur.unwrap_or_default();
    let en = en.unwrap_or_default();
    if ur.trim().is_empty() && en.trim().is_empty() {
        Cell::Empty
    } else {
        Cell::Names {
            names: Names { ur, en },
        }
    }
}

/// A text cell, or an empty cell for a blank text.
pub(crate) fn text_cell(text: Option<String>) -> Cell {
    match text.map(|t| t.trim().to_owned()).filter(|t| !t.is_empty()) {
        Some(text) => Cell::Text { text },
        None => Cell::Empty,
    }
}

/// A date cell from a stored `YYYY-MM-DD`, or an empty cell.
pub(crate) fn date_cell(iso: Option<String>) -> Cell {
    iso.map_or(Cell::Empty, |iso| Cell::Date { iso })
}

/// A one-time person inside a پختہ کھاتہ account shows with his own names, the account's code,
/// no short code (the serial 9999 is reserved) and his person key (PRD FR-13.5, FR-19.13). A
/// line that names nobody keeps the account's names.
pub(crate) fn pukhta_party(
    mut party: PartyRef,
    person_key: Option<String>,
    name_ur: Option<&str>,
    name_en: Option<&str>,
) -> PartyRef {
    if person_key.is_none() {
        return party;
    }
    party.person_key = person_key;
    party.short_code = None;
    let ur = name_ur.map(str::trim).filter(|t| !t.is_empty());
    let en = name_en.map(str::trim).filter(|t| !t.is_empty());
    if ur.is_some() || en.is_some() {
        party.name_ur = ur.or(en).unwrap_or_default().to_owned();
        party.name_en = en.or(ur).unwrap_or_default().to_owned();
    }
    party
}

/// Adds money with a check. A sum that does not fit is `ERR_OVERFLOW`, never a wrap-around.
pub(crate) fn add(total: &mut Paisa, amount: Paisa) -> Result<(), MoneyError> {
    *total = total.checked_add(amount)?;
    Ok(())
}

/// A WHERE clause made of fixed fragments and bound values, with a fragment only for the filters
/// that are set. A fragment such as `(?1 IS NULL OR col >= ?1)` hides the column from the query
/// planner, so a date range would scan every row; leaving the fragment out lets the planner use
/// the index of the column (Code Standards B18). Every fragment is a constant of this crate; a
/// value from the screen is only ever bound as a parameter (Code Standards B14).
#[derive(Debug, Default, Clone)]
pub struct Conds {
    parts: Vec<Cow<'static, str>>,
    args: Vec<Value>,
}

impl Conds {
    /// An empty clause.
    pub fn new() -> Self {
        Self::default()
    }

    /// A fragment with no value, for example `s.is_active = 1`.
    pub fn always(&mut self, fragment: &'static str) -> &mut Self {
        self.parts.push(Cow::Borrowed(fragment));
        self
    }

    /// `column NOT IN (?, ?, ...)` for a list of ids; nothing for an empty list. The column is
    /// a constant of this crate and the ids are bound values (Code Standards B14).
    pub fn not_in(&mut self, column: &'static str, ids: &[i64]) -> &mut Self {
        if !ids.is_empty() {
            self.parts.push(Cow::Owned(format!(
                "{column} NOT IN ({})",
                placeholders(ids.len())
            )));
            self.args.extend(ids.iter().map(|id| Value::Integer(*id)));
        }
        self
    }

    /// `column IN (?, ?, ...)` for a list of ids; nothing for an empty list (no narrowing). The
    /// column is a constant of this crate and the ids are bound values (Code Standards B14).
    pub fn in_list(&mut self, column: &'static str, ids: &[i64]) -> &mut Self {
        if !ids.is_empty() {
            self.parts.push(Cow::Owned(format!(
                "{column} IN ({})",
                placeholders(ids.len())
            )));
            self.args.extend(ids.iter().map(|id| Value::Integer(*id)));
        }
        self
    }

    /// A fragment with one `?`, only when the filter is set.
    pub fn id(&mut self, fragment: &'static str, value: Option<i64>) -> &mut Self {
        if let Some(value) = value {
            self.parts.push(Cow::Borrowed(fragment));
            self.args.push(Value::Integer(value));
        }
        self
    }

    /// A fragment with one `?`, only when the text is set.
    pub fn text(&mut self, fragment: &'static str, value: Option<&str>) -> &mut Self {
        if let Some(value) = value {
            self.parts.push(Cow::Borrowed(fragment));
            self.args.push(Value::Text(value.to_owned()));
        }
        self
    }

    /// The fragments joined with `AND`; `1` when there are none.
    pub fn sql(&self) -> String {
        if self.parts.is_empty() {
            "1".to_owned()
        } else {
            self.parts.join(" AND ")
        }
    }

    /// The values to bind, in order.
    pub fn args(&self) -> &[Value] {
        &self.args
    }

    /// The values followed by a limit and an offset, for a page query that ends with
    /// `LIMIT ? OFFSET ?`.
    pub fn page_args(&self, limit: i64, offset: i64) -> Vec<Value> {
        let mut all = self.args.clone();
        all.push(Value::Integer(limit));
        all.push(Value::Integer(offset));
        all
    }
}

/// Runs `action` inside one read transaction, so every statement of a report sees the same saved
/// books: the rows and the totals of a page, or the five sheets of the pack, can never come
/// from two different moments (Code Standards B20, money agent Section 9). Inside a
/// transaction already, it just runs the action.
///
/// # Errors
/// What `action` returns; a database error when the transaction cannot start.
pub(crate) fn in_snapshot<T>(
    conn: &Connection,
    action: impl FnOnce(&Connection) -> Result<T, ReportsError>,
) -> Result<T, ReportsError> {
    if !conn.is_autocommit() {
        return action(conn);
    }
    let tx = conn.unchecked_transaction()?;
    let result = action(&tx);
    // Nothing was written, so ending the transaction is a rollback.
    drop(tx);
    result
}

#[cfg(test)]
mod snapshot_tests {
    use super::*;

    #[test]
    fn a_save_during_a_read_is_not_seen_until_the_next_read() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("snapshot.db");
        let reader = Connection::open(&path).unwrap();
        reader
            .execute_batch(
                "PRAGMA journal_mode = WAL; CREATE TABLE t (n INTEGER); INSERT INTO t VALUES (1);",
            )
            .unwrap();
        let writer = Connection::open(&path).unwrap();
        let count = |c: &Connection| -> i64 {
            c.query_row("SELECT COUNT(*) FROM t", [], |r| r.get(0))
                .unwrap()
        };
        let (first, second) = in_snapshot(&reader, |conn| {
            let first = count(conn);
            writer.execute("INSERT INTO t VALUES (2)", []).unwrap();
            // A nested call joins the same snapshot.
            let second = in_snapshot(conn, |inner| Ok(count(inner)))?;
            Ok((first, second))
        })
        .unwrap();
        assert_eq!(
            (first, second),
            (1, 1),
            "both reads see the books as they were"
        );
        assert_eq!(count(&reader), 2, "the next read sees the save");
    }
}
