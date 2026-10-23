//! Expense heads and the shop expenses entered under them, in SQL (PRD 16.2, FR-7.62, plan 17).
//! No rules here.

use rusqlite::{Connection, OptionalExtension, params};

/// An expense head as an entry needs it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeadFacts {
    /// `expense_heads.id`.
    pub id: i64,
    /// Its code.
    pub code: String,
    /// Urdu name.
    pub name_ur: String,
    /// English name.
    pub name_en: String,
    /// Its account under control 0501.
    pub account_id: i64,
    /// False when hidden from pickers.
    pub is_active: bool,
}

const HEAD_SELECT: &str = "SELECT h.id, h.code, h.name_ur, h.name_en, h.account_id, h.is_active
     FROM expense_heads h";

fn head_from(r: &rusqlite::Row<'_>) -> rusqlite::Result<HeadFacts> {
    Ok(HeadFacts {
        id: r.get(0)?,
        code: r.get(1)?,
        name_ur: r.get(2)?,
        name_en: r.get(3)?,
        account_id: r.get(4)?,
        is_active: r.get(5)?,
    })
}

/// One expense head by id.
///
/// # Errors
/// Any SQLite error.
pub fn head(conn: &Connection, id: i64) -> rusqlite::Result<Option<HeadFacts>> {
    conn.query_row(&format!("{HEAD_SELECT} WHERE h.id = ?1"), [id], head_from)
        .optional()
}

/// The head named تنخواہ (Salary or Salaries), the default of a salary deduction (PRD FR-16.4).
///
/// # Errors
/// Any SQLite error.
pub fn salary_head(conn: &Connection) -> rusqlite::Result<Option<HeadFacts>> {
    conn.query_row(
        &format!(
            "{HEAD_SELECT} WHERE h.is_active = 1
               AND (h.name_ur = 'تنخواہ' OR lower(h.name_en) IN ('salary', 'salaries'))
             ORDER BY h.id LIMIT 1"
        ),
        [],
        head_from,
    )
    .optional()
}

/// An active head with how often it was used.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeadUse {
    /// The head.
    pub head: HeadFacts,
    /// How many saved shop expenses use it.
    pub use_count: i64,
    /// The day of the latest one, `YYYY-MM-DD`.
    pub last_used: Option<String>,
}

/// The active heads, most used first, then by code (PRD FR-7.62). A cancelled expense does not
/// count.
///
/// # Errors
/// Any SQLite error.
pub fn heads_by_use(conn: &Connection) -> rusqlite::Result<Vec<HeadUse>> {
    let mut statement = conn.prepare(
        "SELECT h.id, h.code, h.name_ur, h.name_en, h.account_id, h.is_active,
                COUNT(v.id), MAX(v.entry_date)
         FROM expense_heads h
         LEFT JOIN vouchers v ON v.expense_head_id = h.id AND v.type = 'shop_expense'
                              AND v.status = 'saved'
         WHERE h.is_active = 1
         GROUP BY h.id
         ORDER BY COUNT(v.id) DESC, h.code, h.id",
    )?;
    let rows = statement.query_map([], |r| {
        Ok(HeadUse {
            head: head_from(r)?,
            use_count: r.get(6)?,
            last_used: r.get(7)?,
        })
    })?;
    rows.collect()
}

/// A saved shop expense of a month.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonthExpense {
    /// The head.
    pub head_id: i64,
    /// The amount, paisa.
    pub amount_paisa: i64,
    /// "Paid to". Empty if none.
    pub paid_to: String,
    /// The note. Empty if none.
    pub narration: String,
    /// The day, `YYYY-MM-DD`.
    pub entry_date: String,
}

/// The saved shop expenses dated from `from` up to but not including `before`, by day and number.
///
/// # Errors
/// Any SQLite error.
pub fn shop_expenses_between(
    conn: &Connection,
    from: &str,
    before: &str,
) -> rusqlite::Result<Vec<MonthExpense>> {
    let mut statement = conn.prepare(
        "SELECT v.expense_head_id, v.amount_paisa, COALESCE(v.reason, ''), COALESCE(v.narration, ''),
                v.entry_date
         FROM vouchers v
         WHERE v.type = 'shop_expense' AND v.status = 'saved' AND v.expense_head_id IS NOT NULL
           AND v.entry_date >= ?1 AND v.entry_date < ?2
         ORDER BY v.entry_date, v.number, v.id",
    )?;
    let rows = statement.query_map(params![from, before], |r| {
        Ok(MonthExpense {
            head_id: r.get(0)?,
            amount_paisa: r.get(1)?,
            paid_to: r.get(2)?,
            narration: r.get(3)?,
            entry_date: r.get(4)?,
        })
    })?;
    rows.collect()
}
