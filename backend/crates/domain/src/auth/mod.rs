//! Sign-in rules as pure functions (PRD Section 8, FR-2.4, NFR-SEC-1, NFR-SEC-12, plan 04).
//!
//! Nothing here reads the clock, the database or a random source: the services pass those in.

pub mod lockout;
pub mod recovery;
pub mod validate;
