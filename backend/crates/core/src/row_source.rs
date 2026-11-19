//! The record a row of a list opens (PRD FR-19.4, FR-7.71): a report row, a روکڑ بک row.
//!
//! The reports and the cash book share these two types, so a row of either opens its entry the
//! same way. They live in `core` because the cash book (a `core` type) carries one.

use serde::Serialize;
use ts_rs::TS;

/// What a row opens (PRD FR-19.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename = "ReportSourceKind")]
pub enum SourceKind {
    /// A saved بکری.
    Bikri,
    /// A saved voucher.
    Voucher,
    /// An account.
    Account,
    /// A month close entry.
    MonthClose,
    /// A trip purchase.
    Purchase,
    /// A garden arrival (مال آیا, PRD FR-12.40): the id is the arrival's.
    Arrival,
    /// A trip or a garden. In a report row the screen knows which one from the report.
    Venture,
    /// A trip: the id is the trip's. Used by the روکڑ بک for the entries of a trip (a trip cost, a
    /// sale in another shop).
    Trip,
    /// A garden: the id is the garden's. Used by the روکڑ بک for the entries of a garden (an
    /// installment or running cost, a sale in another shop, the closing).
    Garden,
    /// Fruit taken.
    Fruit,
    /// A journal entry that belongs to no screen of its own.
    Entry,
}

/// The record a row opens.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[ts(rename = "ReportRowSource")]
pub struct RowSource {
    /// What kind of record.
    pub kind: SourceKind,
    /// Its id.
    #[ts(type = "number")]
    pub id: i64,
}

impl RowSource {
    /// The record a journal entry opens, from its stored `source_type` and `source_id`. An entry
    /// whose source has no screen opens itself (`entry_id`).
    ///
    /// `venture` is the trip or garden a `venture` entry belongs to, already worked out by the
    /// caller (the stored `source_id` of such an entry is the id of one of its entries, not of the
    /// trip or garden). Without it a `venture` entry opens itself.
    ///
    /// Example: a voucher entry with `source_id` 7 opens `Voucher` 7; a trip cost whose trip is 3
    /// opens `Trip` 3.
    #[must_use]
    pub fn of_entry(
        source_type: &str,
        source_id: i64,
        entry_id: i64,
        venture: Option<RowSource>,
    ) -> RowSource {
        let (kind, id) = match source_type {
            "bikri" => (SourceKind::Bikri, source_id),
            "voucher" => (SourceKind::Voucher, source_id),
            "account" => (SourceKind::Account, source_id),
            "month_close" => (SourceKind::MonthClose, entry_id),
            "purchase" => (SourceKind::Purchase, source_id),
            "venture" => match venture {
                Some(found) => (found.kind, found.id),
                None => (SourceKind::Entry, entry_id),
            },
            "fruit" => (SourceKind::Fruit, source_id),
            _ => (SourceKind::Entry, entry_id),
        };
        RowSource { kind, id }
    }
}

impl RowSource {
    /// The trip or garden a `venture` entry belongs to, from the `ventures.kind` (`trip` or
    /// `garden`) and the id of the venture. `None` if either is missing or the kind is unknown.
    #[must_use]
    pub fn of_venture(kind: Option<&str>, venture_id: Option<i64>) -> Option<RowSource> {
        let kind = match kind? {
            "trip" => SourceKind::Trip,
            "garden" => SourceKind::Garden,
            _ => return None,
        };
        Some(RowSource {
            kind,
            id: venture_id?,
        })
    }

    /// [`RowSource::of_entry`] from the columns a query reads: the stored source and, for a
    /// `venture` entry, the kind and id of its trip or garden (see `mandi_db::row_source_sql`).
    #[must_use]
    pub fn of_stored(
        source_type: &str,
        source_id: i64,
        entry_id: i64,
        venture_kind: Option<&str>,
        venture_id: Option<i64>,
    ) -> RowSource {
        Self::of_entry(
            source_type,
            source_id,
            entry_id,
            Self::of_venture(venture_kind, venture_id),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_source_type_opens_its_record() {
        let open = |t: &str| RowSource::of_entry(t, 7, 90, None);
        assert_eq!(
            open("bikri"),
            RowSource {
                kind: SourceKind::Bikri,
                id: 7
            }
        );
        assert_eq!(
            open("voucher"),
            RowSource {
                kind: SourceKind::Voucher,
                id: 7
            }
        );
        assert_eq!(
            open("account"),
            RowSource {
                kind: SourceKind::Account,
                id: 7
            }
        );
        assert_eq!(
            open("purchase"),
            RowSource {
                kind: SourceKind::Purchase,
                id: 7
            }
        );
        assert_eq!(
            open("fruit"),
            RowSource {
                kind: SourceKind::Fruit,
                id: 7
            }
        );
    }

    #[test]
    fn entries_without_a_screen_open_themselves() {
        let open = |t: &str| RowSource::of_entry(t, 7, 90, None);
        assert_eq!(
            open("month_close"),
            RowSource {
                kind: SourceKind::MonthClose,
                id: 90
            }
        );
        assert_eq!(
            open("something_new"),
            RowSource {
                kind: SourceKind::Entry,
                id: 90
            }
        );
        // A venture entry with no trip or garden found opens itself.
        assert_eq!(
            open("venture"),
            RowSource {
                kind: SourceKind::Entry,
                id: 90
            }
        );
    }

    #[test]
    fn the_columns_of_a_query_give_the_trip_or_garden() {
        let stored = |kind, id| RowSource::of_stored("venture", 7, 90, kind, id);
        assert_eq!(
            stored(Some("trip"), Some(3)),
            RowSource {
                kind: SourceKind::Trip,
                id: 3
            }
        );
        assert_eq!(
            stored(Some("garden"), Some(4)),
            RowSource {
                kind: SourceKind::Garden,
                id: 4
            }
        );
        // Nothing found, or a kind we do not know: the entry opens itself.
        for (kind, id) in [(None, None), (Some("trip"), None), (Some("farm"), Some(1))] {
            assert_eq!(
                stored(kind, id),
                RowSource {
                    kind: SourceKind::Entry,
                    id: 90
                }
            );
        }
        // Another source ignores the venture columns.
        assert_eq!(
            RowSource::of_stored("voucher", 7, 90, Some("trip"), Some(3)),
            RowSource {
                kind: SourceKind::Voucher,
                id: 7
            }
        );
    }

    #[test]
    fn a_venture_entry_opens_its_trip_or_garden() {
        let trip = RowSource {
            kind: SourceKind::Trip,
            id: 3,
        };
        assert_eq!(RowSource::of_entry("venture", 7, 90, Some(trip)), trip);
        let garden = RowSource {
            kind: SourceKind::Garden,
            id: 4,
        };
        assert_eq!(RowSource::of_entry("venture", 7, 90, Some(garden)), garden);
    }
}
