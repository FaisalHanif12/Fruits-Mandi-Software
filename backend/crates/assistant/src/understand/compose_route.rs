//! One door for the local answers: what the app writes by itself when it understood a question
//! completely (plan 55 Section 2, step "write: no model call").
//!
//! The caller asks [`compose`] after `understand`. `Some` is the finished answer and no model is
//! called; `None` goes on to the read step or the model. Help and how-to, print, open a screen or a
//! report, no such party or the wrong role, and not in the app are written here; a figure or a list
//! is not. A request to write is refused with the entry screen that does it (`change_refused`).
//!
//! Example: "wasooli kaise likhun" -> the guide's topic `receipt`; "zubair ne kya becha" -> not
//! found, asked as a seller; "bank balance kitna hai" -> outside the books, the cash book as the
//! nearest thing.

use mandi_services::core::auth::Language;

use super::compose_change::compose_change;
use super::compose_help::{compose_help, compose_navigate};
use super::compose_missing::compose_missing;
use super::compose_outside::compose_outside;
use super::compose_print::compose_print;
use super::{Composed, Intent, Understanding};
use crate::results::Words;

/// The local answer of a question, or `None` when a model or a read is needed.
pub fn compose(
    u: &Understanding,
    language: Language,
    words: &Words,
    read_at: &str,
) -> Option<Composed> {
    if !u.sure {
        return None;
    }
    // A name the books do not have comes first: there is nobody to print or to read.
    if let Some(missing) = compose_missing(u) {
        return Some(missing);
    }
    match u.intent.value {
        Intent::HowTo => compose_help(u, language, read_at),
        Intent::Print => compose_print(u, language, read_at),
        Intent::Navigate => compose_navigate(u, language, read_at),
        Intent::NotInApp => compose_outside(u, language, words, read_at),
        Intent::ChangeRequest => compose_change(u),
        _ => None,
    }
}
