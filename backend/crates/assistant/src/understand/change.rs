//! Which entry screen a request to write opens (plan 55 Section 7, lead ruling of 04-11-2026).
//!
//! The assistant never writes (PRD FR-7.72, rule 11 of CLAUDE.md). A request to write is refused in
//! plain words, and the panel offers the entry screen that does it, with the party chosen as a hint.
//! This file picks that screen from the words of the question, with no model call. It picks one
//! only when the question says it plainly; a request that fits two screens, or none, is left to the
//! model path.
//!
//! The amount is never taken from the question: the entry screen opens with the amount empty and
//! the Owner writes it himself (PRD FR-7.53, rule 11).
//!
//! Example: "akram ki 5000 wasooli likh do" -> the wasooli screen, Akram chosen, no amount.
//! "faisal ko 2000 de do" -> the adaigi screen (a payment to a buyer is cash lent, PRD FR-15.11).
//! "5000 jama karo" -> no screen: it could be a receipt, capital or a journal line.

use super::lexicon::{Cue, key};
use super::scan::Scan;
use super::what::screen_of as screen_of_topic;
use super::who::Names;
use super::{PartyHit, PartyRole};

/// Cues of a question that also asks to read something ("akram ka balance batao aur 5000 jama
/// karo"). The refusal would drop the question, so it goes to the model path, which answers the
/// question and carries the route too.
const ASKS_TO_READ: [Cue; 14] = [
    Cue::Balance,
    Cue::Pending,
    Cue::Show,
    Cue::Ledger,
    Cue::Statement,
    Cue::DaySummary,
    Cue::DayBook,
    Cue::TrialBalance,
    Cue::Many,
    Cue::Most,
    Cue::Least,
    Cue::Decision,
    Cue::Forecast,
    Cue::Print,
];

/// Screens that only show things. A request to write never opens them.
const READ_ONLY: [&str; 8] = [
    "dashboard",
    "cash_book",
    "today_sales",
    "unsold_stock",
    "collection_list",
    "dead_amount",
    "reports",
    "analytics",
];

/// The nouns that name an entry screen, and the screen. A request that says exactly one of them has
/// its screen. The role words (partner, garden, trip) are not here: they say who, not what.
const ENTRY_NOUNS: [(Cue, &str); 10] = [
    (Cue::Collection, "wasooli"),
    (Cue::Dues, "wasooli"),
    (Cue::Payable, "adaigi"),
    (Cue::Advance, "peshgi"),
    (Cue::Expense, "kharcha"),
    (Cue::MoneyLent, "udhar_raqam"),
    (Cue::RoleKhata, "alag_khata"),
    (Cue::Capital, "sarmaya"),
    (Cue::FruitTaken, "phal"),
    (Cue::Sale, "bikri"),
];

/// Words that say "take it away" (delete, cancel, remove), in the normalised form of `Scan`.
const REMOVE_WORDS: [&str; 12] = [
    "delete", "remove", "hata", "hatao", "hatado", "mita", "mitao", "mitado", "ہٹا", "ہٹاؤ", "مٹا",
    "مٹاؤ",
];

/// Phrases that say "give": a payment out. Joined with single spaces.
const GIVE_PHRASES: [&str; 6] = [
    " de do ",
    " dedo ",
    " de dein ",
    " dijiye ",
    " دے دو ",
    " دیجیے ",
];

/// True when the question asks to read something besides the instruction to write.
pub(super) fn asks_to_read(scan: &Scan) -> bool {
    scan.has_any(&ASKS_TO_READ)
}

/// The accounts the question names, found for sure, with their roles: the only ones a route can
/// carry. A one-time person (پختہ کھاتہ) is not an account to choose.
fn named(names: &Names) -> Vec<&PartyHit> {
    names
        .parties
        .iter()
        .filter_map(|p| p.outcome.hit())
        .filter(|hit| !hit.pukhta)
        .collect()
}

fn has_role(hits: &[&PartyHit], roles: &[PartyRole]) -> bool {
    hits.iter().any(|h| roles.contains(&h.role))
}

/// True when the question says one of the words, as a whole word.
fn says_any(scan: &Scan, words: &[&str]) -> bool {
    let keys: Vec<String> = words.iter().map(|w| key(w)).collect();
    scan.words.iter().any(|w| keys.iter().any(|k| k == w))
}

/// True when the question says one of the phrases.
fn says_phrase(scan: &Scan, phrases: &[&str]) -> bool {
    let joined = format!(" {} ", scan.words.join(" "));
    phrases.iter().any(|p| joined.contains(&key(p)))
}

/// The entry screen of a request to write, by its name in the answer schema, or `None` when the
/// question does not say which one.
///
/// The nouns of the entry screens come first (wasooli, peshgi, kharcha ...). Then the topic of the
/// guide (a main screen such as the commission rules). When neither names a screen, the party and
/// the verb decide, in a few plain cases only.
pub(super) fn screen_of(
    scan: &Scan,
    names: &Names,
    topic: Option<&'static str>,
) -> Option<&'static str> {
    let mut nouns: Vec<&'static str> = ENTRY_NOUNS
        .iter()
        .filter(|(cue, _)| scan.has_exact(*cue))
        .map(|(_, screen)| *screen)
        .collect();
    nouns.dedup();
    match nouns.as_slice() {
        [screen] => return Some(screen),
        // Two entry screens named: the question does not say which.
        [_, _, ..] => return None,
        [] => {}
    }
    if let Some(screen) = topic.and_then(screen_of_topic) {
        return (!READ_ONLY.contains(&screen)).then_some(screen);
    }
    let hits = named(names);
    // Money told as received ("akram ne 5000 diye"): a receipt, from a buyer or a seller returning
    // an advance (PRD FR-15.12).
    if scan.has(Cue::Received) && has_role(&hits, &[PartyRole::Buyer, PartyRole::Seller]) {
        return Some("wasooli");
    }
    // "akram ke 5000 jama karo": money in from a buyer.
    if says_any(scan, &["jama", "jamaa", "جمع"])
        && hits.len() == 1
        && has_role(&hits, &[PartyRole::Buyer])
    {
        return Some("wasooli");
    }
    // "x ko 2000 de do": a payment out. To a buyer it is cash lent (PRD FR-15.11).
    if says_phrase(scan, &GIVE_PHRASES) && hits.len() == 1 {
        if has_role(&hits, &[PartyRole::Seller, PartyRole::Buyer]) {
            return Some("adaigi");
        }
        if has_role(&hits, &[PartyRole::Staff]) {
            return Some("udhar_raqam");
        }
    }
    // "delete akram": the account form, where an account is made inactive.
    if says_any(scan, &REMOVE_WORDS)
        && hits.len() == 1
        && has_role(
            &hits,
            &[
                PartyRole::Seller,
                PartyRole::Buyer,
                PartyRole::Khata,
                PartyRole::Staff,
            ],
        )
    {
        return Some("account");
    }
    None
}
