//! Status, settings, the key and the connection test (PRD FR-7.81, FR-7.85, NFR-SEC-15).
//!
//! The OpenRouter key lives only in Windows Credential Manager. No type here ever holds it in a
//! reply: the status shows the last 4 characters at most, and the key reaches the backend once,
//! in [`AssistantKeySaveInput`].
//!
//! Example: after the key `…ab12` is saved and the assistant is switched on,
//! `assistant_status` gives `enabled` true, `key_saved` true, `key_last4` "ab12",
//! `model` "openai/gpt-6-luna", `count_today` 9, `limit` 20.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::ask::AssistantFailure;

/// The model the app starts with (PRD FR-7.85).
pub const DEFAULT_MODEL: &str = "openai/gpt-6-luna";

/// What the panel and Settings need to know (plan 48 Section 4).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct AssistantStatus {
    /// True when the assistant is switched on. It starts off, and can be turned on only once a
    /// key is saved (PRD FR-7.85).
    pub enabled: bool,
    /// True when a key is saved in Credential Manager.
    pub key_saved: bool,
    /// The last 4 characters of the saved key, or null when none is saved. Never more.
    pub key_last4: Option<String>,
    /// The model name, `openai/gpt-6-luna` by default.
    pub model: String,
    /// The model name the app starts with, so Settings can offer to go back to it.
    pub default_model: String,
    /// Counted questions today, Pakistan date (PRD FR-7.81).
    #[ts(type = "number")]
    pub count_today: i64,
    /// The daily limit, 20.
    #[ts(type = "number")]
    pub limit: i64,
}

/// Turns the assistant on or off and sets the model (`assistant_settings_set`). A change is
/// written to the audit log (PRD FR-7.85).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, TS)]
pub struct AssistantSettingsSetInput {
    /// On or off. On with no saved key is `ERR_ASSISTANT_KEY_MISSING` on `enabled`.
    pub enabled: bool,
    /// The OpenRouter model name, for example `openai/gpt-6-luna`. 1 to 100 characters, with no
    /// spaces: empty is `ERR_REQUIRED`, any other bad form is `ERR_FORMAT_INVALID`, on `model`.
    pub model: String,
}

/// Saves or replaces the key (`assistant_key_save`). The Owner's password is needed to save,
/// replace and remove the key, and the change is in the audit log without the key (PRD FR-7.85).
#[derive(Clone, PartialEq, Eq, Deserialize, TS)]
pub struct AssistantKeySaveInput {
    /// The key as pasted. Trimmed. Empty is `ERR_REQUIRED`; spaces or control characters, or more
    /// than 200 characters, is `ERR_FORMAT_INVALID`, on `key`.
    pub key: String,
    /// The Owner's password. `ERR_WRONG_PASSWORD` on `password`; `ERR_LOCKED_WAIT` after too
    /// many wrong tries (PRD FR-8.2).
    pub password: String,
}

// A key must never reach a log by accident (NFR-SEC-15, NFR-OBS-2): Debug hides both fields.
impl std::fmt::Debug for AssistantKeySaveInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AssistantKeySaveInput")
            .field("key", &"[hidden]")
            .field("password", &"[hidden]")
            .finish()
    }
}

/// Removes the key (`assistant_key_remove`). Removing it also switches the assistant off.
#[derive(Clone, PartialEq, Eq, Deserialize, TS)]
pub struct AssistantKeyRemoveInput {
    /// The Owner's password. `ERR_WRONG_PASSWORD` on `password`.
    pub password: String,
}

impl std::fmt::Debug for AssistantKeyRemoveInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AssistantKeyRemoveInput")
            .field("password", &"[hidden]")
            .finish()
    }
}

/// The result of the connection test (`assistant_test`, PRD FR-7.85): one fixed test question to
/// the model. It counts as a question and writes a log row.
///
/// Example: a model that Azure does not serve gives `worked` false, `failure` kind `no_endpoint`
/// and `http_status` 404; the panel says no zero-retention endpoint fits this model.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct AssistantTestResult {
    /// True when the model answered.
    pub worked: bool,
    /// How long it took, in milliseconds.
    #[ts(type = "number")]
    pub millis: i64,
    /// The model that was tried.
    pub model: String,
    /// Why it did not work. Null when it did.
    pub failure: Option<AssistantFailure>,
    /// Today's counted questions after the test.
    #[ts(type = "number")]
    pub count_today: i64,
    /// The daily limit, 20.
    #[ts(type = "number")]
    pub limit: i64,
}
