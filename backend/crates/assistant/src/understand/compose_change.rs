//! The answer to a request to write ("akram ki 5000 wasooli likh do"), with no model call (plan 55
//! Section 7, lead ruling of 04-11-2026).
//!
//! The assistant never writes (PRD FR-7.72). The outcome is `change_refused`: the panel says the
//! plain line that the assistant only tells, and offers the entry screen with the party chosen.
//! The party is a hint: the screen opens with the amount empty, and the Owner writes it himself
//! (PRD FR-7.53, rule 11 of CLAUDE.md). Nothing from the question except the screen and the party
//! reaches the route, so an amount cannot be pre-filled by mistake.
//!
//! Which record the party is depends on the screen (the `EntryRoute` contract): a partner record
//! for kharcha, sarmaya and phal, an account record for the others.
//!
//! Example: "akram ki 5000 wasooli likh do" -> `ChangeRefused`, the wasooli screen, Akram. "wasooli
//! likh do" -> the same screen with no party. "5000 jama karo" -> `None`: no screen is plain.

use super::compose_common::{Composed, entry_screen, record_of};
use super::{Intent, PartyHit, PartyRole, Understanding};
use crate::answer::{EntryRoute, EntryScreen};
use crate::ask::AskOutcome;
use crate::source::AnswerRecord;

/// The local answer to a request to write, or `None` when the understanding is not sure, the
/// question is not a request to write, or it does not say which screen to open (the model path
/// decides then).
pub fn compose_change(u: &Understanding) -> Option<Composed> {
    if !u.sure || u.intent.value != Intent::ChangeRequest {
        return None;
    }
    let screen = entry_screen(u.screen?)?;
    Some(Composed {
        outcome: AskOutcome::ChangeRefused,
        answer: None,
        asked_role: None,
        found_as: None,
        entry_route: Some(EntryRoute {
            screen,
            party: party_for(u, screen),
        }),
    })
}

/// True for the screens that open empty and take a party (the entry screens). A main screen opens
/// as it is.
fn takes_party(screen: EntryScreen) -> bool {
    matches!(
        screen,
        EntryScreen::Bikri
            | EntryScreen::Wasooli
            | EntryScreen::Adaigi
            | EntryScreen::Peshgi
            | EntryScreen::Kharcha
            | EntryScreen::UdharRaqam
            | EntryScreen::AlagKhata
            | EntryScreen::Sarmaya
            | EntryScreen::Phal
            | EntryScreen::General
            | EntryScreen::Account
    )
}

/// True when a record of this role is a value the screen's party picker can take.
fn fits(screen: EntryScreen, role: PartyRole) -> bool {
    match screen {
        EntryScreen::Kharcha | EntryScreen::Sarmaya | EntryScreen::Phal => {
            role == PartyRole::Partner
        }
        EntryScreen::UdharRaqam => role == PartyRole::Staff,
        EntryScreen::AlagKhata => role == PartyRole::Khata,
        EntryScreen::Wasooli | EntryScreen::Adaigi | EntryScreen::Peshgi => {
            matches!(role, PartyRole::Seller | PartyRole::Buyer)
        }
        _ => matches!(
            role,
            PartyRole::Seller | PartyRole::Buyer | PartyRole::Staff | PartyRole::Khata
        ),
    }
}

/// The one party the question names that the screen can take, as a whole record. `None` when none
/// fits, when two fit (the Owner chooses), or when the name is a one-time person.
fn party_for(u: &Understanding, screen: EntryScreen) -> Option<AnswerRecord> {
    if !takes_party(screen) {
        return None;
    }
    let mut fitting: Vec<&PartyHit> = Vec::new();
    for hit in u.parties.iter().filter_map(|p| p.outcome.hit()) {
        if hit.pukhta || !fits(screen, hit.role) {
            continue;
        }
        if !fitting.iter().any(|f| f.id == hit.id && f.role == hit.role) {
            fitting.push(hit);
        }
    }
    match fitting.as_slice() {
        [only] => Some(record_of(only)),
        _ => None,
    }
}
