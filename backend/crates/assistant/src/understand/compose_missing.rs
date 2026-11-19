//! The answer to a name the books do not have, or have only in another role, with no model call
//! (plan 52 Section 8, plan 55 Section 7). The panel owns the words (`assistant-not-found-role`,
//! `assistant-found-as-other`); the composer gives it what to say them about: the kind of record that
//! was asked for, and, when the name exists in another role, that record.
//!
//! Example: "zubair ne kya becha", no Zubair: `NotFound`, `asked_role` seller. "faisal ne kya
//! becha", Faisal a buyer: `NotFound`, `asked_role` seller, `found_as` Faisal as a buyer.

use super::compose_common::{Composed, found_as_of};
use super::{Intent, Outcome, Understanding};
use crate::ask::{AskOutcome, AskedRole};

/// The answer for a question about a party or an item that is not in the books, or only in another
/// role. `None` when every name was found as asked, or when the understanding is not sure.
pub fn compose_missing(u: &Understanding) -> Option<Composed> {
    if !u.sure
        || !matches!(
            u.intent.value,
            Intent::Figure | Intent::List | Intent::Print | Intent::Navigate
        )
    {
        return None;
    }
    let missing = |o: &Outcome<_>| matches!(o, Outcome::NotFound | Outcome::FoundAs(_));
    let party = u.parties.iter().find(|p| missing(&p.outcome));
    // Only when EVERY name of the question is missing: "akram aur zubair ka bakaya" has Akram, and his
    // figure would be lost with the refusal. That question is the model's.
    if party.is_some() && u.parties.iter().any(|p| !missing(&p.outcome)) {
        return None;
    }
    if let Some(party) = party {
        let asked = party.asked_role.map_or(AskedRole::Any, |r| r.asked_role());
        let found_as = match &party.outcome {
            Outcome::FoundAs(hit) => Some(found_as_of(hit)),
            _ => None,
        };
        return Some(Composed {
            outcome: AskOutcome::NotFound,
            answer: None,
            asked_role: Some(asked),
            found_as,
            entry_route: None,
        });
    }
    // An item the books do not hold ("kinnu ka commission"): no item by that name. Not when another
    // name of the question was found: its figure would be lost.
    let item_missing = u
        .items
        .iter()
        .any(|i| matches!(i.outcome, Outcome::NotFound));
    let something_found = !u.parties.is_empty()
        || u.items
            .iter()
            .any(|i| !matches!(i.outcome, Outcome::NotFound));
    (item_missing && !something_found).then_some(Composed {
        outcome: AskOutcome::NotFound,
        answer: None,
        asked_role: Some(AskedRole::Item),
        found_as: None,
        entry_route: None,
    })
}
