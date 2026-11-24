//! The flags of saved-entry rows: what the Edit and Cancel buttons may do, and the edit stamp
//! (PRD FR-7.70, FR-7.71, FR-21.4; plan 44 Section 8.3 and 9.11).
//!
//! The builder of a row says what the entry itself allows (`EntryFlags::open()`, a lock for moved
//! goods, a closed garden ...). [`Marker::mark`] then adds what only the day and the history know:
//! - a cancelled entry is locked (`cancelled`);
//! - an entry of a closed day still works: `needs_reopen` is true, and the button reads "reopen
//!   and fix" (FR-21.4);
//! - an entry in a month that was month-closed is locked (`month_closed`, FR-21.2);
//! - the stamp of its last edit, from the audit log.
//!
//! It never turns a forbidden button on.
//!
//! Example (PRD E2E-45): 10-10-26 is closed, بکری 1 was never edited. Its row has `can_edit` true,
//! `can_cancel` true, `needs_reopen` true and no stamp. After the fix, the stamp has the time and
//! `count` 1.

use std::collections::HashMap;

use mandi_db::edit_stamps;
use mandi_db::rusqlite::Connection;
use mandi_domain::core::edit::{EditEntity, EditStamp, EntryFlags, LockedReason};

use crate::error::ServiceError;

/// What the day and the history say about a group of rows.
pub(crate) struct Marker {
    /// The "closed up to" day, `YYYY-MM-DD`.
    closed_up_to: Option<String>,
    /// The latest month (`YYYY-MM`) with a month close in force.
    last_closed_month: Option<String>,
    /// The last edit of each row that was edited, by id.
    stamps: HashMap<i64, EditStamp>,
}

impl Marker {
    /// Reads the "closed up to" day, the month closes and the stamps of `ids` (the ids of
    /// `entity`, as the audit log names them), through `conn`.
    ///
    /// # Errors
    /// A database error; `ServiceError::Data` for an audit time that is not a time.
    pub(crate) fn load(
        conn: &Connection,
        entity: EditEntity,
        ids: &[i64],
    ) -> Result<Self, ServiceError> {
        let closed_up_to = mandi_db::month::closed_up_to(conn)?;
        let last_closed_month: Option<String> = conn.query_row(
            "SELECT MAX(month) FROM month_closes WHERE is_active = 1",
            [],
            |r| r.get(0),
        )?;
        let mut stamps = HashMap::new();
        if !ids.is_empty() {
            for (id, last) in edit_stamps::last_edits(conn, entity.audit_entity(), ids)? {
                let stamp = EditStamp::from_utc(
                    &last.at,
                    last.by.unwrap_or_default(),
                    last.count,
                    last.reason,
                )
                .ok_or_else(|| {
                    ServiceError::Data(format!("an audit time is not a time: {}", last.at))
                })?;
                stamps.insert(id, stamp);
            }
        }
        Ok(Self {
            closed_up_to,
            last_closed_month,
            stamps,
        })
    }

    /// Adds the day rules and the stamp to the flags of the row `id` dated `date_iso`
    /// (`YYYY-MM-DD`). `saved` is false for a cancelled entry.
    pub(crate) fn mark(&self, id: i64, date_iso: &str, saved: bool, flags: &mut EntryFlags) {
        if !saved {
            *flags = EntryFlags::locked(LockedReason::Cancelled);
        } else if (flags.can_edit || flags.can_cancel) && self.is_closed(date_iso) {
            // A button that works on a closed day works through a reopen; in a month-closed month
            // nothing works (PRD FR-21.2). An entry whose buttons are both off keeps its own reason.
            if self.is_month_closed(date_iso) {
                *flags = EntryFlags::locked(LockedReason::MonthClosed);
            } else {
                flags.needs_reopen = true;
            }
        }
        flags.edited = self.stamps.get(&id).cloned();
    }

    /// True if the day is on or before the "closed up to" day (dates are `YYYY-MM-DD`, so text
    /// order is date order).
    fn is_closed(&self, date_iso: &str) -> bool {
        self.closed_up_to
            .as_deref()
            .is_some_and(|closed| date_iso <= closed)
    }

    /// True if a month close in force covers the month of the day or a later one (PRD FR-21.2:
    /// such a day cannot be reopened until the month close is reversed).
    fn is_month_closed(&self, date_iso: &str) -> bool {
        let month = date_iso.get(..7).unwrap_or(date_iso);
        self.last_closed_month
            .as_deref()
            .is_some_and(|last| month <= last)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn marker(closed: Option<&str>, month: Option<&str>) -> Marker {
        Marker {
            closed_up_to: closed.map(str::to_owned),
            last_closed_month: month.map(str::to_owned),
            stamps: HashMap::new(),
        }
    }

    #[test]
    fn an_open_day_changes_nothing_and_a_closed_day_asks_for_a_reopen() {
        let m = marker(Some("2026-10-10"), None);
        let mut open = EntryFlags::open();
        m.mark(1, "2026-10-11", true, &mut open);
        assert_eq!(open, EntryFlags::open());
        let mut closed = EntryFlags::open();
        m.mark(1, "2026-10-10", true, &mut closed);
        assert!(closed.can_edit && closed.can_cancel && closed.needs_reopen);
        assert_eq!(closed.locked_reason, None);
    }

    #[test]
    fn a_month_closed_month_and_a_cancelled_entry_are_locked() {
        let m = marker(Some("2026-10-31"), Some("2026-10"));
        let mut in_month = EntryFlags::open();
        m.mark(1, "2026-10-10", true, &mut in_month);
        assert_eq!(in_month, EntryFlags::locked(LockedReason::MonthClosed));
        let mut cancelled = EntryFlags::open();
        m.mark(1, "2026-11-02", false, &mut cancelled);
        assert_eq!(cancelled, EntryFlags::locked(LockedReason::Cancelled));
    }

    #[test]
    fn a_lock_of_the_entry_wins_and_a_button_stays_off() {
        let m = marker(Some("2026-10-10"), None);
        let mut moved = EntryFlags::locked(LockedReason::GoodsMoved);
        m.mark(1, "2026-10-10", true, &mut moved);
        assert_eq!(moved, EntryFlags::locked(LockedReason::GoodsMoved));
        // A stub that cannot be edited yet but can be cancelled keeps its edit button off.
        let mut stub = EntryFlags::plain(false, true);
        m.mark(1, "2026-10-09", true, &mut stub);
        assert!(!stub.can_edit && stub.can_cancel && stub.needs_reopen);
    }
}
