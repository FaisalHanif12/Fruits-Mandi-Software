//! A party's commission or sale over a period, asked in everyday words (plan 82, PRD FR-7.90 v1.42,
//! FR-9.3, FR-7.77).
//!
//! "mani ne abhi tak shop ko kitni commission banwa kr di ?" and "mani gahaq se hum ne kitni commission
//! hasil ki abhi tak ?" are one question: how much commission the shop took from Mani so far. The app
//! already writes that answer by itself (the commission page's three figures for a buyer, two for a
//! seller, the sale of either). Two things kept these wordings from it: the small verbs and pronouns
//! around the figure ("banwa", "kr", "hasil", "get", "taraf") were words nothing explains, and some of
//! them stood where a name stands, so they were read as parties that do not exist.
//!
//! The words below carry NO figure and NO filter in such a question: the shop alone earns commission,
//! so "gave", "got", "made" and "obtained" all mean the same commission, charged to the party or on his
//! goods. They are closed classes by use (a light verb, a pronoun, a postposition), never the words of
//! one run. They count only here: a party found by name, a commission or a sale asked (nothing else),
//! one period, no item, no record, and no other word the app does not know. In any other question the
//! same words stay unknown, as before (an unknown word could be a filter: "chai" of "chai ka kharcha").
//!
//! What this module does NOT do: guess a period (a record's figure with no period goes to the model, as
//! `figure.rs` says), guess a party, or add a figure. It only stops these words from being read as
//! doubts and as names. The party, the period and the figure are read as for any question.
//!
//! Example: in "mani ne abhi tak shop ko kitni commission banwa kr di", "banwa" and "kr" are fillers,
//! so the question is the one of "mani ki abhi tak ki commission".

use std::collections::HashSet;
use std::sync::OnceLock;

use super::lexicon::{Cue, key};
use super::scan::Scan;
use super::who::Names;
use super::{
    Confidence, Doubt, FigureKind, Intent, Outcome, Part, PartyRef, PeriodRef, Understanding, reads,
};
use crate::limits::MAX_OPTIONS;

/// Light verbs of making, giving, getting and doing, with their spellings, and the pronouns and
/// postpositions of "from him", "for us", "on his side". Roman Urdu, Urdu letters and English.
#[rustfmt::skip]
const FILLER: &[&str] = &[
    // made, got made ("commission banwa di", "bana kar di")
    "banwa", "banwai", "banwayi", "banwaya", "banwana", "banwate", "bnwa", "bnwai", "bnwaya",
    "bana", "bani", "bane", "banaya", "banai", "bnaya", "bnai",
    // done ("kar di", "kr di", "karwa di")
    "kar", "kr", "karke", "krke", "kra", "karwa", "karwai", "karwayi", "krwa", "krwai",
    // got, obtained ("hasil ki", "hasil hui")
    "hasil", "hasl", "hassil", "hasal",
    // gave, took ("di", "diya", "li", "liya")
    "di", "diya", "dia", "diye", "dee", "de", "li", "liya", "lia", "liye", "lee", "le",
    // "on his side", "from"
    "taraf", "tarf", "traf", "sy", "sey",
    // we, us, ours: the shop
    "hum", "hume", "humen", "humein", "hamen", "hamein", "hamain", "humain", "humne", "hamne",
    // English
    "did", "do", "does", "done", "we", "us", "our", "get", "gets", "got", "getting", "earn", "earned",
    "earns", "earning", "make", "made", "makes", "receive", "received", "receives", "receiving",
    "have", "has", "had", "give", "gave", "given", "gives",
    // Urdu letters
    "بنوا", "بنوائی", "بنوایا", "بنوانا", "بنا", "بنی", "بنایا", "بنائی", "کر", "کرکے", "کروا", "کروائی",
    "حاصل", "دی", "دیا", "دے", "لی", "لیا", "لے", "ہم", "ہمیں", "ہمنے", "ہمارا", "ہماری", "ہمارے", "طرف",
];

/// The fillers as the app keys its words.
fn fillers() -> &'static HashSet<String> {
    static SET: OnceLock<HashSet<String>> = OnceLock::new();
    SET.get_or_init(|| FILLER.iter().map(|w| key(w)).collect())
}

/// True for a filler word (see the module documentation).
pub(super) fn is_filler(word: &str) -> bool {
    fillers().contains(&key(word))
}

/// A party that does not exist and whose words are all fillers: a verb that stood where a name stands.
fn is_junk(party: &PartyRef) -> bool {
    matches!(party.outcome, Outcome::NotFound)
        && party.text.split_whitespace().all(is_filler)
        && !party.text.trim().is_empty()
}

/// True when the figures asked are a commission, a sale, or both (nothing else).
fn flow_figures(kinds: &[FigureKind]) -> bool {
    !kinds.is_empty()
        && kinds
            .iter()
            .all(|k| matches!(k, FigureKind::Commission | FigureKind::Sale))
}

/// True when the question is a party's commission or sale over one period whose only unexplained words
/// are fillers, so the fillers are no doubt and the fillers read as names are no parties.
///
/// `unexplained_but_fillers` is the unexplained-words check made with the fillers explained.
/// `intent` and `kinds` are the intent and the figures read from the words.
///
/// Example: "mani gahaq se hum ne kitni commission hasil ki abhi tak" (Mani a buyer, "hasil" read as a
/// name nobody has) is true; "mani ka chai ka commission" (a word nothing explains) is false.
pub(super) fn settles(
    scan: &Scan,
    intent: &Part<Intent>,
    kinds: &[FigureKind],
    names: &Names,
    periods: &[PeriodRef],
    unexplained_but_fillers: bool,
) -> bool {
    if intent.value != Intent::Figure
        || intent.confidence < Confidence::Medium
        || !flow_figures(kinds)
        || periods.len() != 1
        || !names.items.is_empty()
        || !names.records.is_empty()
        || unexplained_but_fillers
    {
        return false;
    }
    // Exactly one party that is in the books (or several of that name, which the Owner is asked
    // about); every other name is a verb standing where a name stands.
    let real = names
        .parties
        .iter()
        .filter(|p| !matches!(p.outcome, Outcome::NotFound))
        .count();
    // Up to three parties, each found for sure ("haroon aur ans ki commission"): the reads are made for each
    // of them (the app writes no sum, so such a question is the model's, with the reads done).
    let all_found = names
        .parties
        .iter()
        .filter(|p| is_real(p))
        .all(|p| matches!(p.outcome, Outcome::Found(_)));
    if !(1..=3).contains(&real)
        || (real > 1 && !all_found)
        || names.parties.iter().any(|p| !is_real(p) && !is_junk(p))
    {
        return false;
    }
    // "mani aur hasil ki commission": a word beside an "and" may be a second name. The app does not
    // drop it.
    let ands: Vec<usize> = scan
        .hits
        .iter()
        .filter(|h| h.cue == Cue::And)
        .map(|h| h.start)
        .collect();
    names.parties.iter().filter(|p| is_junk(p)).all(|p| {
        let (start, end) = p.words;
        !ands
            .iter()
            .any(|a| Some(*a) == start.checked_sub(1) || *a == end)
    })
}

fn is_real(party: &PartyRef) -> bool {
    !matches!(party.outcome, Outcome::NotFound)
}

/// Takes the verbs that stood where a name stands out of the parties.
pub(super) fn drop_junk(names: &mut Names) {
    names.parties.retain(|p| !is_junk(p));
}

/// The party of a commission or sale question that matches several records, when the app can ask
/// "کون سا؟" itself and then answer with no model: 2 to 5 records of that name, nothing else in doubt,
/// and the question would be a planned figure if the Owner chose any one of them (plan 82, FR-7.77).
///
/// Only a commission or a sale: a balance or any other figure keeps the model's way of asking, which the
/// golden questions pin (E2E-52).
///
/// Example: "akram se is mahine kitna commission mila" with Akram Toll Tax and Akram K both buyers:
/// the party text "akram".
pub fn which_one_party(u: &Understanding) -> Option<String> {
    if u.sure || u.intent.value != Intent::Figure || !flow_figures(&u.figures.value) {
        return None;
    }
    let [party] = u.parties.as_slice() else {
        return None;
    };
    let Outcome::Several(hits) = &party.outcome else {
        return None;
    };
    if !(2..=MAX_OPTIONS).contains(&hits.len()) {
        return None;
    }
    if u.doubts
        .iter()
        .any(|d| !matches!(d, Doubt::SeveralMatches | Doubt::CloseSpelling))
    {
        return None;
    }
    // The same gate as the answer after the click: with the first record chosen, is this a figure the
    // app writes? If not, the model asks.
    let mut chosen = u.clone();
    chosen.parties.first_mut()?.outcome = Outcome::Found(hits.first()?.clone());
    chosen.doubts.clear();
    chosen.sure = true;
    reads::plan(&chosen)
        .is_some_and(|p| !p.lines.is_empty())
        .then(|| party.text.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_verbs_of_the_logged_questions_are_fillers_and_a_noun_is_not() {
        for word in ["banwa", "kr", "hasil", "get", "taraf", "sy", "بنوا", "کر"] {
            assert!(is_filler(word), "{word}");
        }
        // A noun nothing explains is a possible filter, never a filler.
        for word in ["chai", "kela", "bijli", "alawa", "mani"] {
            assert!(!is_filler(word), "{word}");
        }
    }
}
