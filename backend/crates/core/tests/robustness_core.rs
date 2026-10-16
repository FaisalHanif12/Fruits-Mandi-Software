//! Robustness tests for `mandi-core` (plan 02 T6, D1 attack list item 1). Owner: `robustness`.
//!
//! Every test checks one rule with a clear pass or fail:
//! - extreme values (0, 1 rupee, the maximum 10,00,00,00,000, the `i64` limits) give the
//!   exact result or an error, never a wrapped number and never a panic (Code Standards M7);
//! - hostile text (Code Standards 4.2, PRD Section 23) gives the right error code;
//! - words, search keys and dates hold at every boundary and on hostile Unicode.
//!
//! The tests live in `tests/robustness/`, one file per area. Run them alone with
//! `cargo test -p mandi-core --test robustness_core`.
#![cfg(test)]

mod robustness;
