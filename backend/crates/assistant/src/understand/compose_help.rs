//! The answer to a how-to and to "open this screen" with no model call (plan 55 Section 7, PRD
//! FR-7.89). The words are the guide's own (`prompt/guide.json`): what the screen is for, its steps,
//! its keys and its worked example. A report that is asked for by name or party is a button.
//!
//! Example: "wasooli kaise likhun" gives the guide's topic `receipt` word for word, in the app
//! language; "faisal ka khata kholo" gives the button "R13 with Faisal".

use mandi_services::core::auth::Language;

use super::compose_common::{
    Composed, bare, guide_parts, guide_words, route_of, sources_of, steps_of,
};
use super::{Intent, Understanding};
use crate::answer::{AnswerSource, ScreenRoute};

/// The guide's answer to a how-to. `None` when the understanding holds no topic of the guide.
pub fn compose_help(u: &Understanding, language: Language, read_at: &str) -> Option<Composed> {
    if !u.sure || u.intent.value != Intent::HowTo {
        return None;
    }
    let topic = u.topic.as_ref()?.value;
    let words = guide_words(topic, language)?;
    let mut answer = bare(read_at);
    answer.parts = guide_parts(words);
    Some(Composed::answered(answer))
}

/// The answer to "open this": a report with its filters as a button and the steps to find it by
/// hand, or the screen of the guide opened as it is. `None` when the understanding holds neither.
pub fn compose_navigate(u: &Understanding, language: Language, read_at: &str) -> Option<Composed> {
    if !u.sure || u.intent.value != Intent::Navigate {
        return None;
    }
    let mut answer = bare(read_at);
    if let Some(plan) = &u.report {
        let sources = sources_of(plan);
        if sources.is_empty() {
            return None;
        }
        answer.sources = sources;
        answer.how = steps_of(plan, false);
        // The guide's words on reports lead; they say what the screen is for.
        if let Some(words) = guide_words("reports", language) {
            answer.parts = guide_parts(words).into_iter().take(1).collect();
        }
        return Some(Composed::answered(answer));
    }
    let topic = u.topic.as_ref()?.value;
    let words = guide_words(topic, language)?;
    answer.parts = guide_parts(words);
    // The audit log is a tab of the settings: a link-only button opens it (plan 55 §11.1).
    if topic == "audit" {
        answer.sources = vec![AnswerSource::Screen {
            result_id: String::new(),
            route: ScreenRoute::Audit,
        }];
    }
    let mut composed = Composed::answered(answer);
    composed.entry_route = u.screen.and_then(route_of);
    Some(composed)
}
