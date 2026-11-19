//! The edit window of one trip or garden entry (migration 12, plan 44, PRD FR-7.70).
//!
//! The triggers of `purchases`, `venture_entries`, `venture_entry_parts`, `purchase_cost_parts`,
//! `garden_arrivals` and `ventures` (the rent) refuse a change of what was bought, paid or received.
//! The two parts tables are soft state (migration 14): an edit retires the old rows
//! (`is_active = 0`) and writes new ones, and no row is ever deleted.
//! The edit of ONE entry opens a window for it here, changes the entry, and closes the window again,
//! all inside the one transaction of the edit. Only the edit services call these functions (a test
//! checks that the table is empty after every save and after a failed save).
//!
//! Example: the edit of purchase 7 opens `Purchase` 7 and the entry of the purchase's payment
//! `VentureEntry` 12; it writes; it closes both before the transaction ends.

use rusqlite::{Connection, params};

/// Whose id a window is for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Window {
    /// `purchases.id`.
    Purchase,
    /// `venture_entries.id`: its row, its parts and the cost parts spread from it.
    VentureEntry,
    /// `garden_arrivals.id`.
    GardenArrival,
    /// `ventures.id`: the rent of a garden.
    Venture,
}

impl Window {
    /// The text the table stores.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Window::Purchase => "purchase",
            Window::VentureEntry => "venture_entry",
            Window::GardenArrival => "garden_arrival",
            Window::Venture => "venture",
        }
    }
}

/// Opens the window of one entry. Called by the edit service of that entry only.
///
/// # Errors
/// Any SQLite error (a window is open for the entry already).
pub fn open(
    conn: &Connection,
    window: Window,
    id: i64,
    user_id: i64,
    now: &str,
) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO edit_windows (created_at, created_by, updated_at, updated_by, entity, entity_id)
         VALUES (?1, ?2, ?1, ?2, ?3, ?4)",
        params![now, user_id, window.as_str(), id],
    )?;
    Ok(())
}

/// Closes the window of one entry. Called before the transaction ends.
///
/// # Errors
/// Any SQLite error.
pub fn close(conn: &Connection, window: Window, id: i64) -> rusqlite::Result<()> {
    conn.execute(
        "DELETE FROM edit_windows WHERE entity = ?1 AND entity_id = ?2",
        params![window.as_str(), id],
    )?;
    Ok(())
}

/// How many windows are open. It is 0 between two saves.
///
/// # Errors
/// Any SQLite error.
pub fn open_count(conn: &Connection) -> rusqlite::Result<i64> {
    conn.query_row("SELECT COUNT(*) FROM edit_windows", [], |r| r.get(0))
}
