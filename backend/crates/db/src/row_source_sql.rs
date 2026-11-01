//! The two columns a query adds so a journal entry of a trip or garden opens the trip or garden
//! (PRD FR-19.4).
//!
//! The `source_id` of a `venture` journal entry is the id of one entry INSIDE a trip or garden
//! (`venture_entries.id`), not the id of the trip or garden. These two expressions find the
//! venture and its kind. They read the journal entry as `e`, so a query that names its journal
//! entries `e` adds them to its column list and passes the result to
//! `mandi_core::row_source::RowSource::of_stored`.
//!
//! Example: a trip cost whose entry is `venture_entries.id` 12 of trip 3 gives `3` and `trip`.

/// The id of the trip or garden of a `venture` entry `e`; NULL for any other entry.
pub const VENTURE_ID: &str = "(SELECT vx.venture_id FROM venture_entries vx
     WHERE e.source_type = 'venture' AND vx.id = e.source_id)";

/// `trip` or `garden`, the kind of the venture of [`VENTURE_ID`]; NULL for any other entry.
pub const VENTURE_KIND: &str =
    "(SELECT vk.kind FROM venture_entries vy JOIN ventures vk ON vk.id = vy.venture_id
     WHERE e.source_type = 'venture' AND vy.id = e.source_id)";
