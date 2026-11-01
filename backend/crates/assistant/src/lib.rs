//! `mandi-assistant`: ask the app (PRD Section 7.7, FR-7.72 to FR-7.88, plan 48).
//!
//! The model is the brain; this crate is its arms and its auditor. The model reaches the books
//! only through 8 read-only tools, writes its answer with references instead of figures, and the
//! app checks every figure before it reaches the screen. The only thing this crate writes is its
//! own question log (FR-7.84).
//!
//! Depends on `mandi-services` and `mandi-reports` (Code Standards B1: app -> assistant ->
//! services + reports). No arithmetic on money or quantities is done here (Code Standards Q5):
//! every sum, percent, average and estimate comes from `services::figures` and
//! `services::forecast`. Clippy `arithmetic_side_effects` is denied to keep it so.
//!
//! This is the contract step (plan 48 T1): the types the screens send and read, the limits, and
//! the step event. The logic arrives in T2 and T3.
//!
//! - [`ask`]: `assistant_ask`, `assistant_quick`, `assistant_pick`, `assistant_rerun` and the
//!   reply every one of them gives (`AskReply`).
//! - [`answer`]: the answer itself: parts, tables, charts, estimate, sources.
//! - [`settings`]: status, on or off and the model, the key, the connection test.
//! - [`log`]: the question log, page by page.
//! - [`step`]: the event that shows what the app is doing while a question runs.
//! - [`limits`]: the named limits of Code Standards Q9.
#![deny(clippy::arithmetic_side_effects)]
#![cfg_attr(test, allow(clippy::arithmetic_side_effects))]

pub mod answer;
pub mod ask;
pub mod limits;
pub mod log;
pub mod settings;
pub mod step;

/// The name of this crate. Used as context in log lines and errors (Code Standards B12).
pub const CRATE: &str = env!("CARGO_PKG_NAME");

#[cfg(test)]
mod tests {
    use super::CRATE;

    #[test]
    fn crate_name_matches_the_plan() {
        assert_eq!(CRATE, "mandi-assistant");
    }
}
