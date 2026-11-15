//! The link of a khata or of a person who is lent money, and the type of that person (plan 76 points
//! 5 and 10, PRD FR-16.4, FR-16.6 v1.41). Plain reads and writes of `accounts.link_kind`,
//! `accounts.link_partner_id` and `accounts.person_type`; the rules are in `mandi-services`.

use std::collections::HashMap;

use rusqlite::{Connection, OptionalExtension, params, params_from_iter};

/// The link of one account.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredLink {
    /// `none`, `shop` or `partner`.
    pub kind: String,
    /// The partner of a partner link.
    pub partner_id: Option<i64>,
    /// The newest journal entry when the link was set (0 if it never was).
    pub since_entry_id: i64,
}

/// An account of an entry that has a link.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkedAccount {
    /// The account.
    pub account_id: i64,
    /// Its control code: `0305` for a khata, `0103` for a person who is lent money.
    pub control_code: String,
    /// `shop` or `partner`.
    pub kind: String,
    /// The partner of a partner link.
    pub partner_id: Option<i64>,
    /// The newest journal entry when the link was set.
    pub since_entry_id: i64,
}

/// The link of an account, or `None` when there is no such account.
///
/// # Errors
/// Any SQLite error.
pub fn link_of(conn: &Connection, account_id: i64) -> rusqlite::Result<Option<StoredLink>> {
    conn.query_row(
        "SELECT link_kind, link_partner_id, link_since_entry_id FROM accounts WHERE id = ?1",
        [account_id],
        |r| {
            Ok(StoredLink {
                kind: r.get(0)?,
                partner_id: r.get(1)?,
                since_entry_id: r.get(2)?,
            })
        },
    )
    .optional()
}

/// Writes the link of an account.
///
/// # Errors
/// Any SQLite error (the trigger refuses a partner link with no partner).
pub fn set_link(
    conn: &Connection,
    account_id: i64,
    kind: &str,
    partner_id: Option<i64>,
    user_id: i64,
    now: &str,
) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE accounts SET link_kind = ?2, link_partner_id = ?3, updated_at = ?5, updated_by = ?4,
             version = version + 1,
             link_since_entry_id = (SELECT COALESCE(MAX(id), 0) FROM journal_entries)
         WHERE id = ?1",
        params![account_id, kind, partner_id, user_id, now],
    )?;
    Ok(())
}

/// Writes the type of a person who is lent money.
///
/// # Errors
/// Any SQLite error.
pub fn set_person_type(
    conn: &Connection,
    account_id: i64,
    person_type: &str,
) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE accounts SET person_type = ?2 WHERE id = ?1",
        params![account_id, person_type],
    )?;
    Ok(())
}

/// The accounts among `ids` that have a link and that a link means something for: a khata (control
/// 0305) with a shop or partner link, and a person who is lent money (control 0103) with a partner
/// link (a shop-linked loan posts as before and only shows a note in the statements).
///
/// # Errors
/// Any SQLite error.
pub fn linked_among(conn: &Connection, ids: &[i64]) -> rusqlite::Result<Vec<LinkedAccount>> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    let marks = vec!["?"; ids.len()].join(",");
    let sql = format!(
        "SELECT a.id, k.code, a.link_kind, a.link_partner_id, a.link_since_entry_id
         FROM accounts a JOIN account_controls k ON k.id = a.control_id
         WHERE a.id IN ({marks}) AND a.link_kind <> 'none'
           AND (k.code = '0305' OR (k.code = '0103' AND a.link_kind = 'partner'))"
    );
    let mut statement = conn.prepare(&sql)?;
    let rows = statement.query_map(params_from_iter(ids.iter()), |r| {
        Ok(LinkedAccount {
            account_id: r.get(0)?,
            control_code: r.get(1)?,
            kind: r.get(2)?,
            partner_id: r.get(3)?,
            since_entry_id: r.get(4)?,
        })
    })?;
    rows.collect()
}

/// True when an account linked to this partner has a balance that is not 0 (a partner link only: a
/// shop link splits by the shares, which the share check of the partner save already guards).
///
/// # Errors
/// Any SQLite error.
pub fn partner_has_linked_balance(conn: &Connection, partner_id: i64) -> rusqlite::Result<bool> {
    conn.query_row(
        "SELECT EXISTS(
             SELECT 1 FROM accounts a
             WHERE a.link_kind = 'partner' AND a.link_partner_id = ?1
               AND (SELECT COALESCE(SUM(l.dr_paisa), 0) FROM journal_lines l
                    WHERE l.account_id = a.id)
                   <> (SELECT COALESCE(SUM(l.cr_paisa), 0) FROM journal_lines l
                       WHERE l.account_id = a.id))",
        [partner_id],
        |r| r.get(0),
    )
}

/// The 0605 account of each partner, by partner id.
///
/// # Errors
/// Any SQLite error.
pub fn partner_accounts(conn: &Connection) -> rusqlite::Result<HashMap<i64, i64>> {
    let mut statement = conn.prepare(
        "SELECT id, linked_account_id FROM partners WHERE linked_account_id IS NOT NULL",
    )?;
    let rows = statement
        .query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?)))?
        .collect::<rusqlite::Result<HashMap<_, _>>>()?;
    Ok(rows)
}

/// The contra account (03070001).
///
/// # Errors
/// Any SQLite error.
pub fn offset_account(conn: &Connection) -> rusqlite::Result<Option<i64>> {
    conn.query_row("SELECT id FROM accounts WHERE code = '03070001'", [], |r| {
        r.get(0)
    })
    .optional()
}
