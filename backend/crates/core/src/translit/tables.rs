//! The counts behind the letter-group spellings (plan 05 Section 4): for each letter group, how
//! often each Urdu spelling was seen, at three levels of detail.

use std::collections::HashMap;

use super::graphemes::{DROPPED, Piece, Place};
use super::letters::count_as_f64;

/// How strongly a less specific count is trusted when a more specific one is thin.
pub(super) const BACKOFF: f64 = 1.0;

/// Counts of the choices for one situation.
#[derive(Debug, Clone, Default)]
struct Dist {
    counts: Vec<f64>,
    total: f64,
}

impl Dist {
    fn add(&mut self, slot: usize, amount: f64) {
        if self.counts.len() <= slot {
            self.counts.resize(slot.saturating_add(1), 0.0);
        }
        if let Some(count) = self.counts.get_mut(slot) {
            *count += amount;
        }
        self.total += amount;
    }

    fn count(&self, slot: usize) -> f64 {
        self.counts.get(slot).copied().unwrap_or(0.0)
    }
}

/// The place in the counts of the option "dropped" (it is not in the group's list).
fn slot_of(option: usize, option_count: usize) -> usize {
    if option == DROPPED {
        option_count
    } else {
        option
    }
}

/// Counts at three levels of detail, from the most specific to the least.
#[derive(Debug, Default)]
pub(super) struct Tables {
    fine: HashMap<(usize, Place, bool, bool), Dist>,
    place: HashMap<(usize, Place), Dist>,
    group: HashMap<usize, Dist>,
}

impl Tables {
    pub(super) fn add(&mut self, piece: &Piece, option: usize, amount: f64) {
        let slot = slot_of(option, piece.grapheme.options.len());
        self.fine
            .entry((piece.index, piece.place, piece.after_vowel, piece.doubled))
            .or_default()
            .add(slot, amount);
        self.place
            .entry((piece.index, piece.place))
            .or_default()
            .add(slot, amount);
        self.group.entry(piece.index).or_default().add(slot, amount);
    }

    /// Chance of one option, blending the three levels (and an even guess under them).
    pub(super) fn chance(&self, piece: &Piece, option: usize, allowed: usize) -> f64 {
        let slot = slot_of(option, piece.grapheme.options.len());
        let even = 1.0 / count_as_f64(allowed.max(1));
        let blend = |dist: Option<&Dist>, below: f64| -> f64 {
            match dist {
                Some(d) => (d.count(slot) + BACKOFF * below) / (d.total + BACKOFF),
                None => below,
            }
        };
        let level1 = blend(self.group.get(&piece.index), even);
        let level2 = blend(self.place.get(&(piece.index, piece.place)), level1);
        blend(
            self.fine
                .get(&(piece.index, piece.place, piece.after_vowel, piece.doubled)),
            level2,
        )
    }
}
