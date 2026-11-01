//! The event `assistant://step`: what the app is doing while a question runs (PRD FR-7.73,
//! NFR-AI-3).
//!
//! The panel shows its first line "سوال سمجھ رہا ہوں…" itself, within 100 ms of sending. Every
//! later line comes from this event, as a Fluent message ID and its arguments, so the step lines
//! are reviewed words in both languages and no text is written in Rust.
//!
//! Example: while the worked question runs the events are, in order,
//! `assistant-step-finding` with `text` "ikram"; `assistant-step-reading-card` with `name`
//! اکرم ٹول ٹیکس; `assistant-step-reading-report` with `report` R13 and `name`; and
//! `assistant-step-checking`.

use mandi_reports::reports::MessageArg;
use serde::Serialize;
use ts_rs::TS;

/// The name of the Tauri event.
pub const STEP_EVENT: &str = "assistant://step";

/// One step line. Arguments are the report's own message arguments (text, number, money, names).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct AssistantStep {
    /// Fluent message ID, for example `assistant-step-reading-ledger`.
    pub id: String,
    /// Its arguments, by name.
    pub args: Vec<MessageArg>,
}
