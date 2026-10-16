//! English letters to Urdu by the spelling rules, for words that are not in the list
//! (Tech Stack 4.4 step 3).
//!
//! The word is cut into letter groups. Every group has several Urdu spellings, each with the
//! chance the model counted. A beam search keeps the best partial spellings, multiplying the
//! chance of each choice by the chance that the Urdu letters follow each other in that order.

use std::collections::BTreeMap;

use super::graphemes::pieces;
use super::model::{END, Model, START};

/// Partial spellings kept after each group.
const BEAM: usize = 96;
/// Letter groups in the longest word the rules try. Longer words get no suggestion.
pub(crate) const MAX_PIECES: usize = 40;

/// The best Urdu spellings of an English word (lower case letters only), best first, each with
/// its score. The score is the chance of the spelling; only the order matters.
pub(crate) fn candidates(model: &Model, word: &str, keep: usize) -> Vec<(String, f64)> {
    let Some(list) = pieces(word) else {
        return Vec::new();
    };
    if list.is_empty() || list.len() > MAX_PIECES {
        return Vec::new();
    }
    // text so far -> (chance, last letter)
    let mut beam: BTreeMap<String, (f64, char)> = BTreeMap::new();
    beam.insert(String::new(), (1.0, START));
    for piece in &list {
        let options = model.options(piece);
        let mut next: BTreeMap<String, (f64, char)> = BTreeMap::new();
        for (text, (chance, last)) in &beam {
            for (option, option_chance) in &options {
                let mut score = chance * option_chance;
                let mut previous = *last;
                for c in option.chars() {
                    score *= model.urdu_letters.lift(previous, c);
                    previous = c;
                }
                let mut grown = text.clone();
                grown.push_str(option);
                next.entry(grown)
                    .and_modify(|entry| entry.0 += score)
                    .or_insert((score, previous));
            }
        }
        beam = prune(next);
    }
    let mut done: Vec<(String, f64)> = beam
        .into_iter()
        .map(|(text, (chance, last))| (text, chance * model.urdu_letters.lift(last, END)))
        .collect();
    done.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    done.truncate(keep);
    done
}

/// Keeps the [`BEAM`] best partial spellings.
fn prune(map: BTreeMap<String, (f64, char)>) -> BTreeMap<String, (f64, char)> {
    if map.len() <= BEAM {
        return map;
    }
    let mut list: Vec<(String, (f64, char))> = map.into_iter().collect();
    list.sort_by(|a, b| b.1.0.total_cmp(&a.1.0).then_with(|| a.0.cmp(&b.0)));
    list.truncate(BEAM);
    list.into_iter().collect()
}

/// The chance that the rules write the English word `roman` as exactly `urdu`, adding up every
/// way the letter groups can give that Urdu text. 0 if they cannot.
pub(crate) fn channel(model: &Model, roman: &str, urdu: &str) -> f64 {
    let Some(list) = pieces(roman) else {
        return 0.0;
    };
    if list.is_empty() || list.len() > MAX_PIECES {
        return 0.0;
    }
    // byte offset in the Urdu text -> chance of reaching it
    let mut states: BTreeMap<usize, f64> = BTreeMap::new();
    states.insert(0, 1.0);
    for piece in &list {
        let options = model.options(piece);
        let mut next: BTreeMap<usize, f64> = BTreeMap::new();
        for (offset, chance) in &states {
            let rest = urdu.get(*offset..).unwrap_or("");
            for (option, option_chance) in &options {
                if let Some(after) = rest.strip_prefix(option) {
                    let reached = urdu.len().saturating_sub(after.len());
                    *next.entry(reached).or_insert(0.0) += chance * option_chance;
                }
            }
        }
        states = next;
    }
    states.get(&urdu.len()).copied().unwrap_or(0.0)
}
