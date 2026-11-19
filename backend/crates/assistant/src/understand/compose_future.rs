//! The answer to a month that is too far ahead, with no model call (D8 Feature 10, H9).
//!
//! "december 2027 ka commission kitna hai" in October 2026 asks for a month the books do not hold yet,
//! and the estimator goes at most 12 months ahead (PRD FR-7.86), so there is no figure to give. The
//! answer is the plain sentence that says so, `assistant-outside-future`, with no figure, no table and
//! no button, so a made-up number is not even possible. A month inside the 12 months is the model's:
//! it sets up the estimate.
//!
//! Example: today 10-10-2026, "december 2027 ka commission kitna hai": `OutsideBooks`, the sentence,
//! no figure. "december 2026 ka commission kitna hoga": not this composer (an estimate).

use mandi_services::core::auth::Language;

use super::compose_common::{Composed, bare};
use super::{Intent, Understanding};
use crate::answer::AnswerPart;
use crate::ask::AskOutcome;
use crate::results::Words;

/// The Fluent sentence of a month that is out of reach.
pub const FUTURE_MESSAGE: &str = "assistant-outside-future";

/// The local answer for a month beyond the estimate, or `None` when the question is not that.
pub fn compose_future(
    u: &Understanding,
    language: Language,
    words: &Words,
    read_at: &str,
) -> Option<Composed> {
    if !u.sure || u.intent.value != Intent::Forecast || !u.beyond_estimate {
        return None;
    }
    let mut answer = bare(read_at);
    answer.parts.push(AnswerPart::Text {
        text: words.text(language, FUTURE_MESSAGE),
    });
    Some(Composed {
        outcome: AskOutcome::OutsideBooks,
        answer: Some(answer),
        asked_role: None,
        found_as: None,
        entry_route: None,
    })
}
