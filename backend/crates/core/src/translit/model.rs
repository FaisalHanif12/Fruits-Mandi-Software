//! What the word list teaches the spelling rules (plan 05 Section 4).
//!
//! The list is a set of pairs, for example `shahid` and شاہد. Each pair is lined up with the
//! letter groups of its English spelling ("sh" gave ش, "a" gave ا, "i" gave nothing ...). From
//! all pairs the model counts how often each group has each Urdu spelling, by place in the
//! word. It also learns how Urdu letters follow each other, and, for the other direction,
//! where short vowels must be put back.
//!
//! Everything is counted in a fixed order, so the same list always gives the same model, on
//! any computer (Code Standards B28). No random numbers and no hash-map order is used.

use std::collections::{BTreeMap, HashMap, HashSet};

use super::graphemes::{DROPPED, Piece, Place, allowed_options, pieces};
pub(crate) use super::letters::{Bigram, END, START};
use super::tables::{BACKOFF, Tables};
use super::wordlist::Entry;

/// How many times the alignment is re-weighted.
const EM_ROUNDS: usize = 3;
/// The most ways one pair is lined up. A few are enough; more would only cost time.
const MAX_DERIVATIONS: usize = 64;

/// Where a vowel is put back going from Urdu to English: the Urdu letter before and after.
type Gap = (char, char);

/// One way a gap can read: the English letters put there (maybe none), with its count.
type Fill = Vec<(String, f64)>;

/// The trained model.
#[derive(Debug)]
pub(crate) struct Model {
    tables: Tables,
    pub(crate) urdu_letters: Bigram,
    pub(crate) roman_letters: Bigram,
    /// Urdu pieces (1 to 3 letters) with the English letters they stand for, by place.
    chunk_roman: HashMap<(String, Place), Fill>,
    chunk_roman_any: HashMap<String, Fill>,
    /// Pieces longest first, so the reverse search tries long ones first.
    pub(crate) chunks: Vec<String>,
    fill_exact: HashMap<Gap, Fill>,
    fill_class: HashMap<(u8, u8), Fill>,
    /// How many pairs of the list were lined up, and how many not.
    pub(crate) lined_up: usize,
    pub(crate) skipped: usize,
    /// The pairs that could not be lined up (for list maintenance).
    pub(crate) skipped_pairs: Vec<(String, String)>,
}

/// One step of a lining-up: the option chosen for each group.
type Derivation = Vec<usize>;

struct Step {
    piece: Piece,
    options: Vec<(usize, &'static str)>,
}

/// All ways to line the groups up with the Urdu text (at most [`MAX_DERIVATIONS`]).
fn derivations(steps: &[Step], urdu: &str) -> Vec<Derivation> {
    fn walk(
        steps: &[Step],
        urdu: &str,
        chosen: &mut Derivation,
        found: &mut Vec<Derivation>,
        dead: &mut HashSet<(usize, usize)>,
    ) -> bool {
        let Some((step, rest)) = steps.split_first() else {
            if urdu.is_empty() {
                found.push(chosen.clone());
                return true;
            }
            return false;
        };
        let state = (steps.len(), urdu.len());
        if dead.contains(&state) {
            return false;
        }
        let mut any = false;
        for (id, text) in &step.options {
            if found.len() >= MAX_DERIVATIONS {
                return true;
            }
            if let Some(after) = urdu.strip_prefix(text) {
                chosen.push(*id);
                if walk(rest, after, chosen, found, dead) {
                    any = true;
                }
                chosen.pop();
            }
        }
        if !any {
            dead.insert(state);
        }
        any
    }
    let mut found = Vec::new();
    let mut dead = HashSet::new();
    walk(steps, urdu, &mut Vec::new(), &mut found, &mut dead);
    found
}

/// The class of an Urdu letter for the reverse model: 0 start or end, 1 vowel letter, 2 other.
fn letter_class(c: char) -> u8 {
    match c {
        START | END => 0,
        'ا' | 'آ' | 'و' | 'ی' | 'ے' | 'ع' | 'ؤ' | 'ئ' | 'ہ' => 1,
        _ => 2,
    }
}

/// A map of choices turned into a list, most counted first, ties in text order.
fn ranked(map: BTreeMap<String, f64>) -> Fill {
    let mut list: Vec<(String, f64)> = map.into_iter().collect();
    list.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    list
}

impl Model {
    /// Trains on the single-word pairs of the list.
    pub(crate) fn train(entries: &[Entry]) -> Model {
        // 1. Cut every English word into groups once.
        let mut prepared: Vec<(&Entry, Vec<Step>)> = Vec::new();
        for entry in entries {
            if entry.roman.contains(' ') || entry.urdu.contains(' ') {
                continue;
            }
            if let Some(list) = pieces(&entry.roman) {
                let steps = list
                    .into_iter()
                    .map(|piece| Step {
                        options: allowed_options(&piece),
                        piece,
                    })
                    .collect();
                prepared.push((entry, steps));
            }
        }
        // 2. All the ways each pair can be lined up.
        let mut lined: Vec<(&Entry, &[Step], Vec<Derivation>)> = Vec::new();
        let mut skipped = 0_usize;
        let mut skipped_pairs: Vec<(String, String)> = Vec::new();
        for (entry, steps) in &prepared {
            let found = derivations(steps, &entry.urdu);
            if found.is_empty() {
                skipped = skipped.saturating_add(1);
                skipped_pairs.push((entry.roman.clone(), entry.urdu.clone()));
            } else {
                lined.push((entry, steps.as_slice(), found));
            }
        }
        // 3. Count, weighting each way by how likely it is under the counts so far.
        let mut tables = Tables::default();
        for round in 0..EM_ROUNDS {
            let mut next = Tables::default();
            for (_, steps, found) in &lined {
                let weights: Vec<f64> = found
                    .iter()
                    .map(|chosen| {
                        if round == 0 {
                            1.0
                        } else {
                            chosen_chance(&tables, steps, chosen)
                        }
                    })
                    .collect();
                let sum: f64 = weights.iter().sum();
                if sum <= 0.0 {
                    continue;
                }
                for (chosen, weight) in found.iter().zip(&weights) {
                    for (step, option) in steps.iter().zip(chosen) {
                        next.add(&step.piece, *option, weight / sum);
                    }
                }
            }
            tables = next;
        }
        // 4. The best way for each pair, for the Urdu letter model and the reverse counts.
        let single = |e: &&Entry| !e.roman.contains(' ') && !e.urdu.contains(' ');
        let urdu_letters = Bigram::train(
            entries
                .iter()
                .filter(|e| single(e))
                .map(|e| e.urdu.as_str()),
        );
        let roman_letters = Bigram::train(
            entries
                .iter()
                .filter(|e| single(e))
                .map(|e| e.roman.as_str()),
        );
        let mut chunk_place: BTreeMap<(String, Place), BTreeMap<String, f64>> = BTreeMap::new();
        let mut chunk_any: BTreeMap<String, BTreeMap<String, f64>> = BTreeMap::new();
        let mut gap_exact: BTreeMap<Gap, BTreeMap<String, f64>> = BTreeMap::new();
        let mut gap_class: BTreeMap<(u8, u8), BTreeMap<String, f64>> = BTreeMap::new();
        let mut chunk_set: BTreeMap<String, ()> = BTreeMap::new();
        let mut lined_up = 0_usize;
        for (_, steps, found) in &lined {
            let best = found
                .iter()
                .max_by(|a, b| {
                    chosen_chance(&tables, steps, a).total_cmp(&chosen_chance(&tables, steps, b))
                })
                .or_else(|| found.first());
            let Some(best) = best else { continue };
            lined_up = lined_up.saturating_add(1);
            // Walk the steps: pieces with Urdu text are chunks; the others are gaps.
            let mut parts: Vec<(String, String, Place)> = Vec::new(); // (urdu chunk, english, place)
            let mut gaps: Vec<String> = vec![String::new()]; // gaps[i] is before chunk i; last is the end
            for (step, option) in steps.iter().zip(best) {
                let text = if *option == DROPPED {
                    ""
                } else {
                    step.piece
                        .grapheme
                        .options
                        .get(*option)
                        .copied()
                        .unwrap_or("")
                };
                if text.is_empty() {
                    if let Some(gap) = gaps.last_mut() {
                        gap.push_str(step.piece.grapheme.roman);
                    }
                } else {
                    parts.push((
                        text.to_owned(),
                        step.piece.grapheme.roman.to_owned(),
                        step.piece.place,
                    ));
                    gaps.push(String::new());
                }
            }
            for (chunk, english, place) in &parts {
                chunk_place
                    .entry((chunk.clone(), *place))
                    .or_default()
                    .entry(english.clone())
                    .and_modify(|c| *c += 1.0)
                    .or_insert(1.0);
                chunk_any
                    .entry(chunk.clone())
                    .or_default()
                    .entry(english.clone())
                    .and_modify(|c| *c += 1.0)
                    .or_insert(1.0);
                chunk_set.insert(chunk.clone(), ());
            }
            let mut before = START;
            for (index, gap) in gaps.iter().enumerate() {
                let next = parts
                    .get(index)
                    .map_or(END, |(chunk, _, _)| chunk.chars().next().unwrap_or(END));
                let key = (before, next);
                for map in [
                    gap_exact.entry(key).or_default(),
                    gap_class
                        .entry((letter_class(before), letter_class(next)))
                        .or_default(),
                ] {
                    map.entry(gap.clone())
                        .and_modify(|c| *c += 1.0)
                        .or_insert(1.0);
                }
                before = parts
                    .get(index)
                    .and_then(|(chunk, _, _)| chunk.chars().last())
                    .unwrap_or(END);
            }
        }
        let mut chunks: Vec<String> = chunk_set.into_keys().collect();
        chunks.sort_by(|a, b| {
            b.chars()
                .count()
                .cmp(&a.chars().count())
                .then_with(|| a.cmp(b))
        });
        Model {
            tables,
            urdu_letters,
            roman_letters,
            chunk_roman: chunk_place
                .into_iter()
                .map(|(k, v)| (k, ranked(v)))
                .collect(),
            chunk_roman_any: chunk_any.into_iter().map(|(k, v)| (k, ranked(v))).collect(),
            chunks,
            fill_exact: gap_exact.into_iter().map(|(k, v)| (k, ranked(v))).collect(),
            fill_class: gap_class.into_iter().map(|(k, v)| (k, ranked(v))).collect(),
            lined_up,
            skipped,
            skipped_pairs,
        }
    }

    /// The Urdu spellings a group can have here, each with its chance. The chances add up to 1.
    pub(crate) fn options(&self, piece: &Piece) -> Vec<(&'static str, f64)> {
        let allowed = allowed_options(piece);
        let mut list: Vec<(&'static str, f64)> = allowed
            .iter()
            .map(|(id, text)| (*text, self.tables.chance(piece, *id, allowed.len())))
            .collect();
        let sum: f64 = list.iter().map(|(_, p)| *p).sum();
        if sum > 0.0 {
            for (_, p) in &mut list {
                *p /= sum;
            }
        }
        list
    }

    /// English letters an Urdu piece may stand for, with chances that add up to 1.
    pub(crate) fn english_for(&self, chunk: &str, place: Place) -> Vec<(String, f64)> {
        let any = self.chunk_roman_any.get(chunk).cloned().unwrap_or_default();
        let any_total: f64 = any.iter().map(|(_, c)| *c).sum();
        let here = self
            .chunk_roman
            .get(&(chunk.to_owned(), place))
            .cloned()
            .unwrap_or_default();
        let here_total: f64 = here.iter().map(|(_, c)| *c).sum();
        let mut list: Vec<(String, f64)> = any
            .iter()
            .map(|(roman, count)| {
                let base = if any_total > 0.0 {
                    count / any_total
                } else {
                    0.0
                };
                let local = here
                    .iter()
                    .find(|(r, _)| r == roman)
                    .map_or(0.0, |(_, c)| *c);
                (
                    roman.clone(),
                    (local + BACKOFF * base) / (here_total + BACKOFF),
                )
            })
            .collect();
        let sum: f64 = list.iter().map(|(_, p)| *p).sum();
        if sum > 0.0 {
            for (_, p) in &mut list {
                *p /= sum;
            }
        }
        list
    }

    /// English letters that may sit between two Urdu letters (often a short vowel, often
    /// nothing), with chances that add up to 1.
    pub(crate) fn fill_between(&self, before: char, after: char) -> Vec<(String, f64)> {
        let class = self
            .fill_class
            .get(&(letter_class(before), letter_class(after)));
        let exact = self.fill_exact.get(&(before, after));
        let class_total: f64 = class.map_or(0.0, |l| l.iter().map(|(_, c)| *c).sum());
        let exact_total: f64 = exact.map_or(0.0, |l| l.iter().map(|(_, c)| *c).sum());
        let mut names: BTreeMap<String, ()> = BTreeMap::new();
        for list in [class, exact].into_iter().flatten() {
            for (text, _) in list {
                names.insert(text.clone(), ());
            }
        }
        let mut list: Vec<(String, f64)> = names
            .into_keys()
            .map(|text| {
                let base = if class_total > 0.0 {
                    class
                        .and_then(|l| l.iter().find(|(t, _)| *t == text))
                        .map_or(0.0, |(_, c)| *c)
                        / class_total
                } else {
                    0.0
                };
                let local = exact
                    .and_then(|l| l.iter().find(|(t, _)| *t == text))
                    .map_or(0.0, |(_, c)| *c);
                let p = (local + BACKOFF * base) / (exact_total + BACKOFF);
                (text, p)
            })
            .collect();
        let sum: f64 = list.iter().map(|(_, p)| *p).sum();
        if sum > 0.0 {
            for (_, p) in &mut list {
                *p /= sum;
            }
        }
        list
    }
}

/// Chance of one lining-up under the counts so far.
fn chosen_chance(tables: &Tables, steps: &[Step], chosen: &Derivation) -> f64 {
    steps
        .iter()
        .zip(chosen)
        .map(|(step, option)| tables.chance(&step.piece, *option, step.options.len()))
        .product()
}

#[cfg(test)]
mod tests {
    use super::super::graphemes::GRAPHEMES;
    use super::*;

    fn entry(roman: &str, urdu: &str) -> Entry {
        Entry {
            roman: roman.to_owned(),
            urdu: urdu.to_owned(),
            kind: super::super::Kind::Name,
            weight: 100,
            review: "pending".to_owned(),
            source: "team".to_owned(),
        }
    }

    #[test]
    fn a_pair_is_lined_up_with_its_groups() {
        let list = pieces("shahid").unwrap();
        let steps: Vec<Step> = list
            .into_iter()
            .map(|piece| Step {
                options: allowed_options(&piece),
                piece,
            })
            .collect();
        let found = derivations(&steps, "شاہد");
        assert!(!found.is_empty());
        // sh, a, h, i, d: ش + ا + ہ + (nothing) + د
        assert!(found.iter().any(|d| d.len() == 5));
        assert!(derivations(&steps, "xyz").is_empty());
    }

    #[test]
    fn the_model_learns_the_common_spelling() {
        let entries: Vec<Entry> = [
            ("shahid", "شاہد"),
            ("rashid", "راشد"),
            ("majid", "ماجد"),
            ("tariq", "طارق"),
            ("nadeem", "ندیم"),
            ("saleem", "سلیم"),
            ("kareem", "کریم"),
            ("ali", "علی"),
            ("akram", "اکرم"),
            ("arshad", "ارشد"),
        ]
        .iter()
        .map(|(r, u)| entry(r, u))
        .collect();
        let model = Model::train(&entries);
        assert_eq!(model.lined_up, 10);
        assert_eq!(model.skipped, 0);
        let piece = pieces("kareem").unwrap().into_iter().nth(3).unwrap(); // "ee"
        let options = model.options(&piece);
        let best = options.iter().max_by(|a, b| a.1.total_cmp(&b.1)).unwrap();
        assert_eq!(best.0, "ی");
        let sum: f64 = options.iter().map(|(_, p)| *p).sum();
        assert!((sum - 1.0).abs() < 1e-9);
    }

    #[test]
    fn training_twice_gives_the_same_model() {
        let entries: Vec<Entry> = [("shahid", "شاہد"), ("rashid", "راشد"), ("ali", "علی")]
            .iter()
            .map(|(r, u)| entry(r, u))
            .collect();
        let a = Model::train(&entries);
        let b = Model::train(&entries);
        let piece = pieces("shahid").unwrap().into_iter().nth(1).unwrap();
        assert_eq!(a.options(&piece), b.options(&piece));
        assert_eq!(a.chunks, b.chunks);
        assert_eq!(a.fill_between('ش', 'د'), b.fill_between('ش', 'د'));
    }

    #[test]
    fn every_group_has_at_least_one_option_and_a_unique_roman() {
        let mut seen = HashSet::new();
        for group in GRAPHEMES {
            assert!(!group.options.is_empty(), "{}", group.roman);
            assert!(seen.insert(group.roman), "{} twice", group.roman);
        }
    }
}
