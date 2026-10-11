//! Letter-pair models (plan 05 Section 4): how likely one letter is to follow another. They
//! rank spellings so that words that look like real names come first.

use std::collections::{HashMap, HashSet};

/// Small count added to every letter pair, so a new pair is rare but not impossible.
const LM_SMOOTHING: f64 = 0.05;

pub(super) fn count_as_f64(n: usize) -> f64 {
    f64::from(u32::try_from(n).unwrap_or(u32::MAX))
}

/// A letter-pair (bigram) model with start and end marks.
#[derive(Debug, Default)]
pub(crate) struct Bigram {
    pairs: HashMap<(char, char), u32>,
    totals: HashMap<char, u32>,
    /// How often each letter is seen at all (after anything).
    singles: HashMap<char, u32>,
    all: u32,
    letters: usize,
}

/// Start of a word, for the letter-pair models.
pub(crate) const START: char = '^';
/// End of a word, for the letter-pair models.
pub(crate) const END: char = '$';

impl Bigram {
    pub(super) fn train<'a>(words: impl Iterator<Item = &'a str>) -> Bigram {
        let mut model = Bigram::default();
        let mut seen: HashSet<char> = HashSet::new();
        for word in words {
            let mut previous = START;
            for c in word.chars().chain(std::iter::once(END)) {
                let pair = model.pairs.entry((previous, c)).or_insert(0);
                *pair = pair.saturating_add(1);
                let total = model.totals.entry(previous).or_insert(0);
                *total = total.saturating_add(1);
                let single = model.singles.entry(c).or_insert(0);
                *single = single.saturating_add(1);
                model.all = model.all.saturating_add(1);
                seen.insert(c);
                previous = c;
            }
        }
        model.letters = seen.len().max(1);
        model
    }

    /// Chance that `c` follows `previous`.
    pub(crate) fn chance(&self, previous: char, c: char) -> f64 {
        let pair = f64::from(self.pairs.get(&(previous, c)).copied().unwrap_or(0));
        let total = f64::from(self.totals.get(&previous).copied().unwrap_or(0));
        (pair + LM_SMOOTHING) / (total + LM_SMOOTHING * count_as_f64(self.letters))
    }

    /// How much more (or less) likely `c` is after `previous` than anywhere. Above 1 means the
    /// two letters like each other. Used to rank spellings without favouring short words.
    pub(crate) fn lift(&self, previous: char, c: char) -> f64 {
        let single = f64::from(self.singles.get(&c).copied().unwrap_or(0));
        let anywhere = (single + LM_SMOOTHING)
            / (f64::from(self.all) + LM_SMOOTHING * count_as_f64(self.letters));
        (self.chance(previous, c) / anywhere).sqrt()
    }
}
