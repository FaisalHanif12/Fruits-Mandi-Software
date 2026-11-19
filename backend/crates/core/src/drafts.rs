//! Unsaved work of a screen (PRD FR-12.10, NFR-REL-2): what the screen sends every 5 seconds
//! and what it reads back after a crash. The payload is the screen's own JSON; the backend only
//! checks that it is valid JSON and small, and keeps it for the signed-in user.
//!
//! Example: the بکری screen sends `drafts_save { screen: "bikri", payload_json: "{...}" }` every
//! `DRAFT_SAVE_INTERVAL_MS` while the user types. After a power cut it asks `drafts_get` and
//! offers "پچھلی بکری مکمل کریں؟".

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// The biggest payload a draft may have, in bytes (1 MB).
pub const MAX_DRAFT_BYTES: usize = 1_048_576;

/// The longest screen name.
pub const MAX_SCREEN_CHARS: usize = 40;

/// Input of `drafts_save`.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct DraftSaveInput {
    /// The screen the draft belongs to, `a-z 0-9 _ . -` up to 40 characters. The بکری screen is
    /// `bikri`.
    pub screen: String,
    /// The draft as JSON text. Must be valid JSON of at most 1 MB, else `ERR_DRAFT_INVALID`.
    pub payload_json: String,
}

/// What `drafts_save` gives back.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct DraftSaved {
    /// When it was kept, UTC.
    pub saved_at: String,
}

/// Input of `drafts_get`.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct DraftGetInput {
    /// The screen.
    pub screen: String,
}

/// Input of `drafts_discard`.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct DraftDiscardInput {
    /// The screen.
    pub screen: String,
}

/// A kept draft.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct DraftRow {
    /// The draft as it was sent.
    pub payload_json: String,
    /// When it was kept, UTC.
    pub saved_at: String,
}
