//! The answer to a print request with no model call (plan 54 Section 6, plan 55 Section 7).
//!
//! It gives both things Umar asked for: the BUTTON that opens the right report with the party and the
//! period already set (`sources[0]`, and `sources[1]` where the plan has a second report), and the
//! STEPS with the real values filled in (`how`), so he can do it by hand next time. The button and
//! the steps are made from the same plan, so they always agree. The words are the guide's.
//!
//! Example: "faisal ki pichle mahine ki bikri ka print" with Faisal a buyer (code 0022): button R5
//! (Faisal, 01-09-2026 to 30-09-2026), second button R13; steps: open R5, choose the buyer Faisal
//! (0022), set the dates, press F9.

use mandi_services::core::auth::Language;

use super::compose_common::{Composed, bare, guide_words, sources_of, steps_of};
use super::{Intent, Understanding};
use crate::answer::AnswerPart;

/// The guide topic whose summary leads a print answer.
const PRINT_TOPIC: &str = "printing";

/// The answer to a print request. `None` when the understanding holds no report, or a report the
/// app does not have.
pub fn compose_print(u: &Understanding, language: Language, read_at: &str) -> Option<Composed> {
    if !u.sure || u.intent.value != Intent::Print {
        return None;
    }
    let plan = u.report.as_ref()?;
    let sources = sources_of(plan);
    if sources.is_empty() {
        return None;
    }
    let mut answer = bare(read_at);
    answer.sources = sources;
    answer.how = steps_of(plan, true);
    if let Some(words) = guide_words(PRINT_TOPIC, language) {
        answer.parts.push(AnswerPart::Text {
            text: words.summary.clone(),
        });
    }
    Some(Composed::answered(answer))
}
