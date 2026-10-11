//! Putting the spellings of the words together into whole texts (plan 05 Section 4): the best
//! three texts are made by trying the lists in order of their scores.

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashSet};

use super::MAX_SUGGESTIONS;

/// A spelling with a score between 0 and 1.
pub(super) type Scored = (String, f64);

/// One thing to spell: the tokens it covers and its spellings.
pub(super) struct Unit {
    pub(super) candidates: Vec<Scored>,
}

/// A choice of one spelling per unit, ordered by its score (best first, then by choice).
#[derive(Debug, PartialEq)]
struct Choice {
    score: f64,
    picks: Vec<usize>,
}

impl Eq for Choice {}

impl Ord for Choice {
    fn cmp(&self, other: &Self) -> Ordering {
        self.score
            .total_cmp(&other.score)
            .then_with(|| other.picks.cmp(&self.picks))
    }
}

impl PartialOrd for Choice {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// The best whole texts, at most [`MAX_SUGGESTIONS`], made by trying spellings in score order.
/// Each text comes with the choice made in every unit (the index of its spelling), so the
/// caller can say where each word came from.
pub(super) fn best_texts(layout: &[Slot], units: &[Unit]) -> Vec<(String, Vec<usize>)> {
    let score_of = |picks: &[usize]| -> f64 {
        units
            .iter()
            .zip(picks)
            .map(|(unit, pick)| unit.candidates.get(*pick).map_or(0.0, |(_, s)| *s))
            .product()
    };
    let start = vec![0_usize; units.len()];
    let mut heap = BinaryHeap::new();
    let mut seen_picks: HashSet<Vec<usize>> = HashSet::new();
    seen_picks.insert(start.clone());
    heap.push(Choice {
        score: score_of(&start),
        picks: start,
    });
    let mut texts: Vec<(String, Vec<usize>)> = Vec::new();
    let mut seen_texts: HashSet<String> = HashSet::new();
    // A unit with many spellings can make the same text twice; look a little further.
    let mut budget = MAX_SUGGESTIONS.saturating_mul(8);
    while let Some(choice) = heap.pop() {
        if texts.len() >= MAX_SUGGESTIONS || budget == 0 {
            break;
        }
        budget = budget.saturating_sub(1);
        let text = render(layout, units, &choice.picks);
        if seen_texts.insert(text.clone()) {
            texts.push((text, choice.picks.clone()));
        }
        for index in 0..units.len() {
            let mut next = choice.picks.clone();
            if let Some(pick) = next.get_mut(index) {
                *pick = pick.saturating_add(1);
            }
            let has = units
                .get(index)
                .zip(next.get(index))
                .is_some_and(|(unit, pick)| *pick < unit.candidates.len());
            if has && seen_picks.insert(next.clone()) {
                heap.push(Choice {
                    score: score_of(&next),
                    picks: next,
                });
            }
        }
    }
    texts
}

/// How the text is laid out: spaces and fixed pieces between the units.
pub(super) enum Slot {
    Space,
    Fixed(String),
    Unit(usize),
}

fn render(layout: &[Slot], units: &[Unit], picks: &[usize]) -> String {
    let mut out = String::new();
    for slot in layout {
        match slot {
            Slot::Space => out.push(' '),
            Slot::Fixed(text) => out.push_str(text),
            Slot::Unit(index) => {
                let text = units
                    .get(*index)
                    .zip(picks.get(*index))
                    .and_then(|(unit, pick)| unit.candidates.get(*pick))
                    .map_or("", |(text, _)| text.as_str());
                out.push_str(text);
            }
        }
    }
    out
}
