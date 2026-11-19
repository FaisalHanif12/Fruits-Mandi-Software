//! Gate lane T5 (plan 80): attacks on "ask the app" (PRD 7.7, FR-7.72 to FR-7.90, E82 to E98;
//! NFR-AI-1 to NFR-AI-9; Code Standards Q1 to Q11; `docs/deliverables/D6-system-prompt.md`).
//!
//! The model is a recorded transcript (built in these files); nothing calls the network and the
//! key is a placeholder that only looks like one. This file adds attacks that
//! `robustness_assistant.rs` does not have; it does not repeat them.
//!
//! | Module | What it attacks |
//! |---|---|
//! | `privacy` | typed free text in every free-text column of the books, read through every report, view, tile and find (FR-7.83, Q7, E2E-54, finding R6-1) |
//! | `inject` | names and free text that read like orders or like references (E92) |
//! | `typed` | figures hidden in an answer: every Unicode digit, quantity words, look-alike letters, zero-width letters (FR-7.76) |
//! | `tools` | wrong tool calls: columns, limits, periods, months, call and token limits (Q1, FR-7.79, NFR-AI-5) |
//! | `locked` | every partner source while the partner section is locked (FR-7.82, NFR-SEC-14) |
//! | `limit` | the daily limit of 20 and what does not count (FR-7.81) |
//! | `offline` | no network, an error from OpenRouter, a timeout (FR-7.81, NFR-AI-7) |
//!
//! A test that shows a bug is `#[ignore = "gate finding T5-n"]` and is named in the report; the
//! rest pass on the code as it is.
#![cfg(test)]

#[path = "../../services/tests/common/mod.rs"]
mod common;
#[path = "support/harness.rs"]
mod harness;
#[path = "robustness_year/seed.rs"]
mod seed;
#[path = "support/worked.rs"]
mod worked;
#[allow(dead_code)]
#[path = "../../services/tests/bikri_preview/support.rs"]
mod world;

#[path = "robustness_gate_t5/inject.rs"]
mod inject;
#[path = "robustness_gate_t5/kit.rs"]
mod kit;
#[path = "robustness_gate_t5/known.rs"]
mod known;
#[path = "robustness_gate_t5/limit.rs"]
mod limit;
#[path = "robustness_gate_t5/locked.rs"]
mod locked;
#[path = "robustness_gate_t5/offline.rs"]
mod offline;
#[path = "robustness_gate_t5/privacy.rs"]
mod privacy;
#[path = "robustness_gate_t5/tools.rs"]
mod tools;
#[path = "robustness_gate_t5/typed.rs"]
mod typed;
