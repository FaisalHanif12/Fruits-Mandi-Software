//! The next code of a type (plan 32 R2): the largest running number among the codes of that type
//! plus one, so a code is never reused. The type number and the running number are in
//! `mandi_core::codes`; this only reads the table.

use mandi_core::codes::{CodeType, build_code, next_running};
use rusqlite::Connection;

/// The tables that hold codes of this scheme. A fixed list, never built from input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodeTable {
    /// Sellers, buyers and the two پختہ کھاتہ accounts.
    Accounts,
    /// قسم مال.
    Items,
    /// پیکنگ.
    Packings,
    /// عملہ.
    Staff,
}

impl CodeTable {
    const fn name(self) -> &'static str {
        match self {
            CodeTable::Accounts => "accounts",
            CodeTable::Items => "items",
            CodeTable::Packings => "packings",
            CodeTable::Staff => "staff",
        }
    }
}

/// The largest running number among the codes of this type in the table, or `None` if there is no
/// such code. Only codes of exactly this scheme count: the type number, then digits with no leading
/// zero (a lone `0` is allowed), at most 10 digits in all.
///
/// # Errors
/// Any SQLite error.
pub fn largest_running(
    conn: &Connection,
    table: CodeTable,
    code_type: CodeType,
) -> rusqlite::Result<Option<u64>> {
    let number = code_type.number();
    // `table.name()` and `number` are fixed text, never input.
    let sql = format!(
        "SELECT MAX(CAST(substr(code, 4) AS INTEGER)) FROM {table}
         WHERE code GLOB '{number}[0-9]*' AND code NOT GLOB '*[^0-9]*' AND length(code) <= 10
           AND (substr(code, 4) = '0' OR substr(code, 4, 1) <> '0')",
        table = table.name()
    );
    let largest: Option<i64> = conn.query_row(&sql, [], |r| r.get(0))?;
    Ok(largest.and_then(|n| u64::try_from(n).ok()))
}

/// The next free code of a type: `{type number}{largest running + 1}`, or the first code (running
/// number 0) when there is none. `None` when the list is full (the code would pass 10 digits).
///
/// # Errors
/// Any SQLite error.
pub fn next_code(
    conn: &Connection,
    table: CodeTable,
    code_type: CodeType,
) -> rusqlite::Result<Option<(u64, String)>> {
    let largest = largest_running(conn, table, code_type)?;
    Ok(next_running(largest).and_then(|n| build_code(code_type, n).map(|code| (n, code))))
}

/// SQL for `ORDER BY` that puts accounts "by code" (plan 32 R7): a code of the scheme (`0010`,
/// `00110`, `0021`) by type number, then by length, then by text, which is running-number order
/// (`0019` before `00110`); every other code after them, as text. `column` is a column name such
/// as `a.code`, never input.
///
/// ```
/// assert!(mandi_db::codes::order_by_code("a.code").starts_with("CASE WHEN a.code GLOB"));
/// ```
pub fn order_by_code(column: &str) -> String {
    let scheme = format!("{column} GLOB '00[1-6][0-9]*'");
    format!(
        "CASE WHEN {scheme} THEN 0 ELSE 1 END, CASE WHEN {scheme} THEN substr({column}, 1, 3) ELSE '' END, \
         CASE WHEN {scheme} THEN length({column}) ELSE 0 END, {column}"
    )
}
