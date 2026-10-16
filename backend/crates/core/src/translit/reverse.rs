//! Urdu letters to an English name by the spelling rules, for words that are not in the list
//! (PRD FR-7.24, Tech Stack 4.4).
//!
//! The Urdu word is cut into pieces (one to three letters). Each piece has English letters the
//! model counted for it. Between two pieces a short vowel may have to be put back, because
//! Urdu does not write it: the model counted which vowel goes between which letters.

use std::collections::BTreeMap;

use super::forward;
use super::graphemes::Place;
use super::model::{END, Model, START};

/// Partial spellings kept at each place of the word.
const BEAM: usize = 64;
/// Urdu letters in the longest word the rules try.
pub(crate) const MAX_LETTERS: usize = 40;
/// English spellings checked the other way round.
const RERANK: usize = 40;

/// One partial English spelling.
#[derive(Debug, Clone)]
struct State {
    english: String,
    chance: f64,
    /// The last Urdu letter used, for the gap before the next piece.
    before: char,
}

/// The best English spellings of an Urdu word, lower case, best first, each with its score.
pub(crate) fn candidates(model: &Model, urdu: &str, keep: usize) -> Vec<(String, f64)> {
    let letters = urdu.chars().count();
    if letters == 0 || letters > MAX_LETTERS {
        return Vec::new();
    }
    // byte offset in the Urdu word -> partial spellings that reach it
    let mut reached: BTreeMap<usize, Vec<State>> = BTreeMap::new();
    reached.insert(
        0,
        vec![State {
            english: String::new(),
            chance: 1.0,
            before: START,
        }],
    );
    let total = urdu.len();
    let mut finished: Vec<(String, f64)> = Vec::new();
    while let Some((offset, mut states)) = reached.pop_first() {
        states.sort_by(|a, b| {
            b.chance
                .total_cmp(&a.chance)
                .then_with(|| a.english.cmp(&b.english))
        });
        states.truncate(BEAM);
        let rest = urdu.get(offset..).unwrap_or("");
        if rest.is_empty() {
            // The gap after the last piece, then done.
            for state in &states {
                for (gap, gap_chance) in model.fill_between(state.before, END) {
                    let mut text = state.english.clone();
                    text.push_str(&gap);
                    let score = state.chance
                        * gap_chance
                        * roman_chance(model, &state.english, &gap)
                        * model
                            .roman_letters
                            .lift(text.chars().last().unwrap_or(START), END);
                    finished.push((text, score));
                }
            }
            continue;
        }
        for chunk in &model.chunks {
            let Some(after) = rest.strip_prefix(chunk.as_str()) else {
                continue;
            };
            let end_offset = total.saturating_sub(after.len());
            let place = if offset == 0 {
                Place::Initial
            } else if after.is_empty() {
                Place::Final
            } else {
                Place::Medial
            };
            let first = chunk.chars().next().unwrap_or(END);
            let options = model.english_for(chunk, place);
            for state in &states {
                for (gap, gap_chance) in model.fill_between(state.before, first) {
                    for (english, english_chance) in &options {
                        let mut text = state.english.clone();
                        text.push_str(&gap);
                        text.push_str(english);
                        let mut score = state.chance * gap_chance * english_chance;
                        score *= roman_chance(model, &state.english, &format!("{gap}{english}"));
                        let grown = State {
                            english: text,
                            chance: score,
                            before: chunk.chars().last().unwrap_or(END),
                        };
                        reached.entry(end_offset).or_default().push(grown);
                    }
                }
            }
        }
    }
    // The same spelling can come from different cuts: add their chances.
    let mut merged: BTreeMap<String, f64> = BTreeMap::new();
    for (text, score) in finished {
        *merged.entry(text).or_insert(0.0) += score;
    }
    let mut list: Vec<(String, f64)> = merged.into_iter().collect();
    list.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    list.truncate(RERANK);
    // Check each English spelling the other way round: how likely is it that the forward rules
    // write it as exactly this Urdu word? Spellings that would not give back the word fall.
    let mut checked: Vec<(String, f64)> = list
        .into_iter()
        .map(|(text, score)| {
            let back = forward::channel(model, &text, urdu);
            (text, score.sqrt() * back)
        })
        .filter(|(_, score)| *score > 0.0)
        .collect();
    checked.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    checked.truncate(keep);
    checked
}

/// Chance of the English letters `added` after `so_far`, from the English letter-pair model.
fn roman_chance(model: &Model, so_far: &str, added: &str) -> f64 {
    let mut previous = so_far.chars().last().unwrap_or(START);
    let mut score = 1.0;
    for c in added.chars() {
        score *= model.roman_letters.lift(previous, c);
        previous = c;
    }
    score
}
