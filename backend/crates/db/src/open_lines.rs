//! The live lines of an account, oldest first, with what their voucher says (PRD FR-16.2, FR-16.5,
//! plan 17). The services work the open amounts of money taken and money lent out of them at read
//! time with `mandi_domain::matching::match_bills`; nothing is stored (money agent 8.2b).
//!
//! A line is live when its entry is posted, is not a reversal and has not been reversed, so a
//! cancelled or edited voucher leaves no trace here. The order is the one matching uses: an
//! opening first, then the day, the order key and the entry (migration 0005), then the line.

use std::collections::HashMap;

use rusqlite::{Connection, OptionalExtension, params, params_from_iter};

/// A live line of an account with its voucher, if it has one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiveLine {
    /// The journal line.
    pub line_id: i64,
    /// Its account.
    pub account_id: i64,
    /// Its entry.
    pub entry_id: i64,
    /// The day, `YYYY-MM-DD`.
    pub entry_date: String,
    /// True for an opening balance.
    pub is_opening: bool,
    /// The debit, paisa (0 on a credit line).
    pub dr_paisa: i64,
    /// The credit, paisa (0 on a debit line).
    pub cr_paisa: i64,
    /// The voucher the entry belongs to, if any.
    pub voucher_id: Option<i64>,
    /// Its number.
    pub number: Option<i64>,
    /// Its type, stored form.
    pub voucher_type: Option<String>,
    /// Its reason. Empty if none.
    pub reason: String,
    /// Who took the cash (a partner's take). Empty if none.
    pub taken_by: String,
    /// Its return-by day, `YYYY-MM-DD`.
    pub return_by: Option<String>,
    /// True for a salary deduction.
    pub salary: bool,
    /// The posting type of its entry, stored form (`opening`, `bikri_first`, `general` ...).
    pub posting_type: String,
    /// The number of the بکری the entry belongs to, if it is a بکری posting.
    pub bikri_number: Option<i64>,
    /// The note of the entry. Empty if none.
    pub narration: String,
}

/// The live lines of these accounts, by account and then oldest first.
///
/// # Errors
/// Any SQLite error.
pub fn live_lines(conn: &Connection, account_ids: &[i64]) -> rusqlite::Result<Vec<LiveLine>> {
    if account_ids.is_empty() {
        return Ok(Vec::new());
    }
    let marks = vec!["?"; account_ids.len()].join(", ");
    let mut statement = conn.prepare(&format!(
        "SELECT l.id, l.account_id, l.entry_id, e.entry_date, e.posting_type = 'opening',
                l.dr_paisa, l.cr_paisa, v.id, v.number, v.type, COALESCE(v.reason, ''),
                COALESCE(v.taken_by, ''), v.return_by_date, COALESCE(v.mode = 'salary', 0),
                e.posting_type, b.number, COALESCE(e.narration, '')
         FROM journal_lines l
         JOIN journal_entries e ON e.id = l.entry_id
         LEFT JOIN vouchers v ON e.source_type = 'voucher' AND v.id = e.source_id
         LEFT JOIN bikri_entries b ON e.source_type = 'bikri' AND b.id = e.source_id
         WHERE l.account_id IN ({marks}) AND e.status = 'posted' AND e.reverses_entry_id IS NULL
           AND NOT EXISTS (SELECT 1 FROM journal_entries r
                           WHERE r.reverses_entry_id = e.id AND r.status = 'posted')
         ORDER BY l.account_id, (e.posting_type <> 'opening'), e.entry_date,
                  COALESCE(e.order_key, e.id), e.id, l.line_no"
    ))?;
    let rows = statement.query_map(params_from_iter(account_ids.iter()), |r| {
        Ok(LiveLine {
            line_id: r.get(0)?,
            account_id: r.get(1)?,
            entry_id: r.get(2)?,
            entry_date: r.get(3)?,
            is_opening: r.get(4)?,
            dr_paisa: r.get(5)?,
            cr_paisa: r.get(6)?,
            voucher_id: r.get(7)?,
            number: r.get(8)?,
            voucher_type: r.get(9)?,
            reason: r.get(10)?,
            taken_by: r.get(11)?,
            return_by: r.get(12)?,
            salary: r.get(13)?,
            posting_type: r.get(14)?,
            bikri_number: r.get(15)?,
            narration: r.get(16)?,
        })
    })?;
    rows.collect()
}

/// The ids of the accounts under a control, by id (active or not).
///
/// # Errors
/// Any SQLite error.
pub fn accounts_under_control(conn: &Connection, control_code: &str) -> rusqlite::Result<Vec<i64>> {
    let mut statement = conn.prepare(
        "SELECT a.id FROM accounts a JOIN account_controls c ON c.id = a.control_id
         WHERE c.code = ?1 ORDER BY a.id",
    )?;
    let rows = statement.query_map([control_code], |r| r.get(0))?;
    rows.collect()
}

/// A person who can be lent money, as the lists show him.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersonFacts {
    /// His account.
    pub account_id: i64,
    /// Its code.
    pub code: String,
    /// Urdu name.
    pub name_ur: String,
    /// English name.
    pub name_en: String,
    /// His first phone. Empty if none.
    pub phone: String,
    /// The staff member he is, if any.
    pub staff_id: Option<i64>,
    /// The staff member's names.
    pub staff_names: Option<(String, String)>,
}

/// The accounts under control 0103 with their phone and staff record. `ids` limits the answer to
/// these accounts; an empty list means every account of the control.
///
/// # Errors
/// Any SQLite error.
pub fn persons(conn: &Connection, ids: &[i64]) -> rusqlite::Result<HashMap<i64, PersonFacts>> {
    let filter = if ids.is_empty() {
        String::new()
    } else {
        format!("AND a.id IN ({})", vec!["?"; ids.len()].join(", "))
    };
    let mut statement = conn.prepare(&format!(
        "SELECT a.id, a.code, a.name_ur, a.name_en, COALESCE(a.phone1, ''), s.id, s.name_ur, s.name_en
         FROM accounts a JOIN account_controls c ON c.id = a.control_id
         LEFT JOIN staff s ON s.account_id = a.id
         WHERE c.code = '0103' {filter}"
    ))?;
    let rows = statement.query_map(params_from_iter(ids.iter()), |r| {
        let staff_id: Option<i64> = r.get(5)?;
        let staff_ur: Option<String> = r.get(6)?;
        let staff_en: Option<String> = r.get(7)?;
        Ok(PersonFacts {
            account_id: r.get(0)?,
            code: r.get(1)?,
            name_ur: r.get(2)?,
            name_en: r.get(3)?,
            phone: r.get(4)?,
            staff_id,
            staff_names: staff_ur.zip(staff_en),
        })
    })?;
    rows.map(|row| row.map(|p| (p.account_id, p))).collect()
}

/// What `lent_person_create` needs to know about a staff member.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaffFacts {
    /// Urdu name.
    pub name_ur: String,
    /// English name.
    pub name_en: String,
    /// His ادھار رقم account, if he has one.
    pub account_id: Option<i64>,
}

/// A staff member by id.
///
/// # Errors
/// Any SQLite error.
pub fn staff(conn: &Connection, id: i64) -> rusqlite::Result<Option<StaffFacts>> {
    conn.query_row(
        "SELECT name_ur, name_en, account_id FROM staff WHERE id = ?1",
        [id],
        |r| {
            Ok(StaffFacts {
                name_ur: r.get(0)?,
                name_en: r.get(1)?,
                account_id: r.get(2)?,
            })
        },
    )
    .optional()
}

/// Links a staff member to his ادھار رقم account (PRD FR-16.4).
///
/// # Errors
/// Any SQLite error.
pub fn link_staff_account(
    conn: &Connection,
    staff_id: i64,
    account_id: i64,
    user_id: i64,
    now: &str,
) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE staff SET account_id = ?2, updated_at = ?3, updated_by = ?4, version = version + 1
         WHERE id = ?1",
        params![staff_id, account_id, now, user_id],
    )?;
    Ok(())
}
