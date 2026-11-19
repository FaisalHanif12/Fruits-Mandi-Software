//! The answer to a question about something the app does not keep, with no model call (plan 55
//! Section 4 gate 5): weather, a tax, a bank balance, the rate in another market, the dollar. It
//! says plainly what the app does not keep, with the nearest thing it does keep as a button, and it
//! never guesses.
//!
//! The sentence is a Fluent message (`assistant-outside-weather`, `-tax`, `-bank`, `-market-rate`,
//! `-prices`, `-other`), read through `Words`, so no sentence is written here (Tech Stack 4.2). The
//! nearest thing: tax and a market rate -> the sales register R2; a bank balance -> the cash book
//! R12; anything else the app does not keep -> the dashboard.
//!
//! Example: "bank balance kitna hai": outside the books, the text "This app does not keep a bank
//! balance ...", and the button "R12 cash book".

use mandi_services::core::auth::Language;

use super::compose_common::{Composed, bare, source_of};
use super::plan::{PlanFilters, ReportPlan};
use super::{Intent, OutsideKind, Understanding};
use crate::answer::{AnswerPart, AnswerSource};
use crate::ask::AskOutcome;
use crate::results::Words;

/// The Fluent message of what the app does not keep.
pub fn outside_message(kind: OutsideKind) -> &'static str {
    match kind {
        OutsideKind::Weather => "assistant-outside-weather",
        OutsideKind::Tax => "assistant-outside-tax",
        OutsideKind::Bank => "assistant-outside-bank",
        OutsideKind::MarketRate => "assistant-outside-market-rate",
        OutsideKind::Prices => "assistant-outside-prices",
        OutsideKind::Other => "assistant-outside-other",
    }
}

/// The nearest thing the app does keep, as a button. `None` where nothing is near (the weather,
/// the dollar).
fn nearest(kind: OutsideKind) -> Option<AnswerSource> {
    let report = |id| {
        source_of(&ReportPlan {
            id,
            filters: PlanFilters::default(),
            also: None,
            assumed_period: false,
        })
    };
    match kind {
        OutsideKind::Weather | OutsideKind::Prices => None,
        OutsideKind::Tax | OutsideKind::MarketRate => report("R2"),
        OutsideKind::Bank => report("R12"),
        OutsideKind::Other => Some(AnswerSource::Tiles {
            result_id: String::new(),
        }),
    }
}

/// The answer to a question outside the books. `None` when the understanding is not that.
pub fn compose_outside(
    u: &Understanding,
    language: Language,
    words: &Words,
    read_at: &str,
) -> Option<Composed> {
    if !u.sure || u.intent.value != Intent::NotInApp {
        return None;
    }
    let kind = u.outside?;
    let mut answer = bare(read_at);
    answer.parts.push(AnswerPart::Text {
        text: words.text(language, outside_message(kind)),
    });
    answer.sources.extend(nearest(kind));
    Some(Composed {
        outcome: AskOutcome::OutsideBooks,
        answer: Some(answer),
        asked_role: None,
        found_as: None,
        entry_route: None,
    })
}
