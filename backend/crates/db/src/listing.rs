//! What the SQL lists share for search and month subtotals (plan 38 B2, PRD FR-19.11, Code
//! Standards B31, NFR-SPD-24).
//!
//! - [`like_prefix`] and [`like_contains`] turn the text a user typed into a `LIKE` pattern in which
//!   `%`, `_` and `\` are plain characters. Write the condition with `ESCAPE '\\'`.
//! - [`month_sums`] gives the count and the sums of each month of a filter in ONE grouped query, so
//!   a month heading can show its subtotal over the whole filter on any page. Only `SUM()` of
//!   INTEGER columns is used (CLAUDE.md, SQLite sums).
//!
//! Example: `month_sums(conn, "vouchers v", " WHERE v.type = ?", "v.entry_date",
//! &["v.amount_paisa"], &binds)` gives `[("2026-10", 4, [120000]), ("2026-11", 2, [50000])]`.

use rusqlite::{Connection, params_from_iter, types::Value};

/// One month of a filter: its count and the sum of each expression asked for, in the same order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonthSums {
    /// The month, `YYYY-MM`.
    pub month: String,
    /// How many rows the filter has in this month.
    pub count: i64,
    /// The sums, one for each expression given to [`month_sums`].
    pub sums: Vec<i64>,
}

/// Escapes `%`, `_` and `\`, so typed text is taken as plain characters in a `LIKE`.
fn escaped(text: &str) -> String {
    let mut out = String::with_capacity(text.len().saturating_add(2));
    for c in text.chars() {
        if matches!(c, '%' | '_' | '\\') {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

/// A pattern for "starts with `text`".
pub fn like_prefix(text: &str) -> String {
    format!("{}%", escaped(text))
}

/// A pattern for "contains `text`".
pub fn like_contains(text: &str) -> String {
    format!("%{}%", escaped(text))
}

/// Counts the rows of `from_sql` that match `where_sql` for each month of `month_column`, and sums
/// each of `sums` (SQL expressions of INTEGER columns written by the list's own code, never by
/// the user), oldest month first. `where_sql` is the text of the list's own `WHERE` (empty, or
/// starting with ` WHERE `) and `binds` its values. One grouped query.
///
/// # Errors
/// Any SQLite error.
pub fn month_sums(
    conn: &Connection,
    from_sql: &str,
    where_sql: &str,
    month_column: &str,
    sums: &[&str],
    binds: &[Value],
) -> rusqlite::Result<Vec<MonthSums>> {
    let mut select = format!("substr({month_column}, 1, 7), COUNT(*)");
    for expression in sums {
        select.push_str(&format!(", COALESCE(SUM({expression}), 0)"));
    }
    let mut statement = conn.prepare(&format!(
        "SELECT {select} FROM {from_sql}{where_sql} GROUP BY 1 ORDER BY 1"
    ))?;
    let rows = statement.query_map(params_from_iter(binds.iter()), |r| {
        let mut figures = Vec::with_capacity(sums.len());
        for index in 0..sums.len() {
            figures.push(r.get::<_, i64>(index.saturating_add(2))?);
        }
        Ok(MonthSums {
            month: r.get(0)?,
            count: r.get(1)?,
            sums: figures,
        })
    })?;
    rows.collect()
}
