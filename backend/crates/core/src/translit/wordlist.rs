//! The word list: reviewed pairs of an English spelling and an Urdu spelling (PRD FR-7.27).
//!
//! The list ships inside the app, embedded at build time: `data/translit_seed.tsv` (names,
//! places, items and shop words) and `data/translit_everyday.tsv` (the checked everyday words,
//! NFR-URD-30). One line is one pair: `roman <TAB> urdu <TAB> kind <TAB> weight <TAB> review <TAB> source`. The last
//! two columns are optional when reading. Several Urdu spellings
//! may share one English spelling; the higher weight comes first. A key with spaces (for
//! example `abdul rehman`) is a phrase that is matched as a whole.

use std::collections::HashMap;
use std::sync::OnceLock;

use thiserror::Error;

use super::Kind;
use super::loose;
use super::model::Model;
use crate::text::{is_urdu_mark, is_zero_width};

/// Names, places, items and shop words.
const SEED: &str = include_str!("../../data/translit_seed.tsv");
/// The everyday dictionary, built from `data/everyday/src` (plan 99).
const EVERYDAY: &str = include_str!("../../data/translit_everyday.tsv");
/// The `source` of the everyday dictionary.
const EVERYDAY_SOURCE: &str = "everyday";

/// Which words come first: in a name field the names, in free text the everyday words.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Field {
    Name,
    Text,
}

/// In a name field every name list pair comes before every everyday word: an everyday word
/// counts so little that even the lightest name stands above it. (Everyday words still come
/// before the spelling rules.)
const EVERYDAY_IN_NAMES: f64 = 0.005;
/// How much a shop word or an item of the first list counts in free text.
const SHOP_WORD_IN_TEXT: f64 = 0.80;
/// How much a name or a place counts in free text. A weak or hurried everyday spelling (weight
/// 50 and below) stays under a name's first spelling.
const NAME_IN_TEXT: f64 = 0.55;
/// A word no checker has read yet counts a little less than a checked one.
const NOT_CHECKED: f64 = 0.98;
/// A pair only our own rules proposed counts far below every other pair.
const PROPOSED: f64 = 0.4;
/// How much each letter that differs takes from a word typed almost like it.
const PER_EDIT: f64 = 0.08;
/// The least a word typed almost like it counts, however many letters differ.
const LEAST_ALIKE: f64 = 0.4;
/// The most words of an everyday entry that one typed word can stand for.
const MAX_JOINED_WORDS: usize = 3;
/// A skeleton match may differ from the typed letters by this many edits, or by half the typed
/// length and one more if that is larger, so `bht` meets بہت but `thank` never meets تنخواہ.
const MIN_SKELETON_EDITS: usize = 3;
/// How much a skeleton match counts if the typed `h` is not in it (`yhn` is یہاں, not یوں).
const NO_TYPED_H: f64 = 0.7;

/// One pair of the list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// English spelling, lower case, words separated by one space.
    pub roman: String,
    /// Urdu spelling.
    pub urdu: String,
    /// What the word is.
    pub kind: Kind,
    /// Which Urdu spelling is preferred for the same English spelling (higher first).
    pub weight: u32,
    /// `pending`; `checked` once a second, independent check has passed the row; `reviewed`
    /// once Umar's Urdu reviewer has read it (NFR-URD-4).
    pub review: String,
    /// Where the pair came from: `team` (written by us), `wikidata` (CC0), `geonames`
    /// (CC BY 4.0), `everyday` (the everyday dictionary) or `engine` (the Urdu was proposed
    /// by our spelling rules). Credits are in `data/SOURCES.md`. Several sources are joined
    /// with `,`.
    pub source: String,
}

impl Entry {
    /// True if a person wrote this spelling (or a reviewer checked it). A pair whose Urdu was
    /// only proposed by our own rules (`source` `engine`) is kept in the file for the reviewer
    /// but does not count as known until it is `reviewed`: the screen still marks it "check
    /// the spelling" and the save still asks (PRD FR-7.66).
    pub fn counts_as_known(&self) -> bool {
        self.source != "engine" || self.review == "reviewed"
    }

    /// True for a pair of the everyday dictionary.
    pub fn is_everyday(&self) -> bool {
        self.source == EVERYDAY_SOURCE
    }

    /// A name list pair: known, and not an everyday word.
    fn is_name_list(&self) -> bool {
        self.counts_as_known() && !self.is_everyday()
    }

    /// How strongly this pair stands for its spelling in a field. Higher is offered first.
    pub(crate) fn standing(&self, field: Field) -> f64 {
        let weight = f64::from(self.weight);
        if !self.counts_as_known() {
            return PROPOSED * weight;
        }
        if self.is_everyday() {
            let checked = if self.review == "pending" {
                NOT_CHECKED
            } else {
                1.0
            };
            let here = match field {
                Field::Text => 1.0,
                Field::Name => EVERYDAY_IN_NAMES,
            };
            return weight * checked * here;
        }
        match field {
            Field::Name => weight,
            Field::Text if matches!(self.kind, Kind::Word | Kind::Item) => {
                weight * SHOP_WORD_IN_TEXT
            }
            Field::Text => weight * NAME_IN_TEXT,
        }
    }
}

/// A line of the word list that cannot be read.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("word list line {line}: {reason}")]
pub struct ParseError {
    /// Line number, from 1.
    pub line: usize,
    /// What is wrong.
    pub reason: String,
}

/// The Urdu text without marks (zabar, zer, khari zabar ...) and invisible characters, the
/// form the lookups use.
pub(crate) fn plain_urdu(text: &str) -> String {
    text.chars()
        .filter(|&c| !is_urdu_mark(c) && !is_zero_width(c))
        .collect()
}

/// The words, indexed both ways, and the models trained on them.
#[derive(Debug)]
pub struct WordList {
    entries: Vec<Entry>,
    by_roman: HashMap<String, Vec<usize>>,
    by_urdu: HashMap<String, Vec<usize>>,
    longest_phrase: usize,
    /// (the Urdu word before, the English spelling) -> the Urdu word, from two-word phrases.
    after: HashMap<(String, String), String>,
    /// Trained on the name list: names, places, items and shop words.
    name_model: OnceLock<Model>,
    /// Trained on the name list and the usual spelling of every everyday word.
    text_model: OnceLock<Model>,
    /// The everyday words by their loose and skeleton keys, built the first time needed.
    variants: OnceLock<Variants>,
}

/// The everyday words by the keys of the `loose` module.
#[derive(Debug, Default)]
struct Variants {
    loose: HashMap<String, Vec<usize>>,
    skeleton: HashMap<String, Vec<usize>>,
}

/// For each pair of an Urdu word and a spelling that a two-word phrase of the everyday
/// dictionary holds: the Urdu of the second word, the heaviest phrase first.
fn words_after(entries: &[Entry]) -> HashMap<(String, String), String> {
    let mut best: HashMap<(String, String), (u32, String)> = HashMap::new();
    for entry in entries.iter().filter(|e| e.is_everyday()) {
        let mut roman = entry.roman.split(' ');
        let mut urdu = entry.urdu.split(' ');
        let (Some(_), Some(second), None) = (roman.next(), roman.next(), roman.next()) else {
            continue;
        };
        let (Some(first_ur), Some(second_ur), None) = (urdu.next(), urdu.next(), urdu.next())
        else {
            continue;
        };
        let key = (plain_urdu(first_ur), second.to_owned());
        let stronger = best.get(&key).is_none_or(|(w, u)| {
            (entry.weight, std::cmp::Reverse(second_ur)) > (*w, std::cmp::Reverse(u.as_str()))
        });
        if stronger {
            best.insert(key, (entry.weight, second_ur.to_owned()));
        }
    }
    best.into_iter().map(|(k, (_, u))| (k, u)).collect()
}

fn clean_key(text: &str) -> String {
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

impl WordList {
    /// A list with no words. Suggestions then come from the spelling rules alone.
    pub fn empty() -> WordList {
        Self::from_entries(Vec::new())
    }

    /// The list that ships with the app, read once: the name list and the everyday words.
    pub fn embedded() -> &'static WordList {
        static LIST: OnceLock<WordList> = OnceLock::new();
        LIST.get_or_init(|| {
            let mut entries = Vec::new();
            for text in [SEED, EVERYDAY] {
                if let Ok(list) = WordList::parse(text) {
                    entries.extend(list.entries);
                }
            }
            WordList::from_entries(entries)
        })
    }

    /// Reads a list in the file form. Blank lines, lines starting with `#` and the header line
    /// (starting with `roman`) are skipped.
    ///
    /// # Errors
    /// The first line that does not have 4 to 6 columns, a known kind and a whole-number weight.
    pub fn parse(text: &str) -> Result<WordList, ParseError> {
        let mut entries = Vec::new();
        for (index, line) in text.lines().enumerate() {
            let number = index.saturating_add(1);
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with("roman\t") {
                continue;
            }
            let fail = |reason: &str| ParseError {
                line: number,
                reason: reason.to_owned(),
            };
            let mut columns = line.split('\t');
            let (Some(roman), Some(urdu), Some(kind), Some(weight)) = (
                columns.next(),
                columns.next(),
                columns.next(),
                columns.next(),
            ) else {
                return Err(fail("fewer than 4 columns"));
            };
            let review = columns.next().unwrap_or("pending");
            let source = columns.next().unwrap_or("team");
            if columns.next().is_some() {
                return Err(fail("more than 6 columns"));
            }
            let roman = clean_key(roman);
            let urdu = urdu.split_whitespace().collect::<Vec<_>>().join(" ");
            if roman.is_empty() || urdu.is_empty() {
                return Err(fail("an empty spelling"));
            }
            let kind = Kind::parse(kind.trim()).ok_or_else(|| fail("unknown kind"))?;
            let weight: u32 = weight
                .trim()
                .parse()
                .map_err(|_| fail("weight is not a whole number"))?;
            entries.push(Entry {
                roman,
                urdu,
                kind,
                weight,
                review: review.trim().to_owned(),
                source: source.trim().to_owned(),
            });
        }
        Ok(Self::from_entries(entries))
    }

    /// A list from entries already made.
    pub fn from_entries(entries: Vec<Entry>) -> WordList {
        let mut by_roman: HashMap<String, Vec<usize>> = HashMap::new();
        let mut by_urdu: HashMap<String, Vec<usize>> = HashMap::new();
        let mut longest_phrase = 1;
        for (index, entry) in entries.iter().enumerate() {
            by_roman.entry(entry.roman.clone()).or_default().push(index);
            by_urdu
                .entry(plain_urdu(&entry.urdu))
                .or_default()
                .push(index);
            longest_phrase = longest_phrase.max(entry.roman.split(' ').count());
        }
        // Heavier first; the same weight in Urdu order, so the order never depends on the file.
        let order = |a: &usize, b: &usize| -> std::cmp::Ordering {
            match (entries.get(*a), entries.get(*b)) {
                (Some(x), Some(y)) => y
                    .weight
                    .cmp(&x.weight)
                    .then_with(|| x.urdu.cmp(&y.urdu))
                    .then_with(|| x.roman.cmp(&y.roman)),
                _ => std::cmp::Ordering::Equal,
            }
        };
        for list in by_roman.values_mut().chain(by_urdu.values_mut()) {
            list.sort_by(order);
        }
        let after = words_after(&entries);
        WordList {
            entries,
            by_roman,
            by_urdu,
            longest_phrase,
            after,
            name_model: OnceLock::new(),
            text_model: OnceLock::new(),
            variants: OnceLock::new(),
        }
    }

    fn variants(&self) -> &Variants {
        self.variants.get_or_init(|| {
            let mut made = Variants::default();
            for (index, entry) in self.entries.iter().enumerate() {
                if !entry.is_everyday() || entry.roman.split(' ').count() > MAX_JOINED_WORDS {
                    continue;
                }
                let loose_key = loose::loose_key(&entry.roman);
                made.loose.entry(loose_key).or_default().push(index);
                let skeleton = loose::skeleton_key(&entry.roman);
                made.skeleton.entry(skeleton).or_default().push(index);
            }
            made
        })
    }

    /// Everyday words typed almost like `word`: the same loose key, or with `by_skeleton` the
    /// same skeleton. The strongest and closest first, each Urdu word once, with its score.
    pub(crate) fn typed_like(&self, word: &str, by_skeleton: bool) -> Vec<(&str, f64)> {
        let variants = self.variants();
        let found = if by_skeleton {
            variants.skeleton.get(&loose::skeleton_key(word))
        } else {
            variants.loose.get(&loose::loose_key(word))
        };
        let mut best: HashMap<&str, f64> = HashMap::new();
        let typed_h = by_skeleton && word.chars().skip(1).any(|c| c == 'h');
        let most_edits = (word.chars().count() / 2)
            .saturating_add(1)
            .max(MIN_SKELETON_EDITS);
        for entry in found
            .into_iter()
            .flatten()
            .filter_map(|i| self.entries.get(*i))
        {
            let roman: String = entry.roman.split(' ').collect();
            let edits = loose::distance(word, &roman);
            if by_skeleton && edits > most_edits {
                continue;
            }
            let edits = u32::try_from(edits).unwrap_or(u32::MAX);
            let alike = (1.0 - PER_EDIT * f64::from(edits)).max(LEAST_ALIKE);
            let keeps_h = !typed_h || roman.chars().skip(1).any(|c| c == 'h');
            let score =
                entry.standing(Field::Text) * alike * if keeps_h { 1.0 } else { NO_TYPED_H };
            let slot = best.entry(entry.urdu.as_str()).or_insert(score);
            *slot = slot.max(score);
        }
        let mut ranked: Vec<(&str, f64)> = best.into_iter().collect();
        ranked.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(b.0)));
        ranked
    }

    /// All entries, in file order.
    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    /// How many entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// True if there are no entries.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// The most words in one English key. Phrases up to this long are tried.
    pub(crate) fn longest_phrase(&self) -> usize {
        self.longest_phrase
    }

    /// Entries whose English spelling is `key` (lower case, one space between words), the
    /// preferred Urdu spelling first.
    pub fn by_roman(&self, key: &str) -> Vec<&Entry> {
        self.by_roman
            .get(key)
            .map(|list| list.iter().filter_map(|i| self.entries.get(*i)).collect())
            .unwrap_or_default()
    }

    /// Entries whose Urdu spelling is `urdu` (marks such as zabar and khari zabar do not count),
    /// the preferred English spelling first.
    pub fn by_urdu(&self, urdu: &str) -> Vec<&Entry> {
        self.by_urdu
            .get(&plain_urdu(urdu))
            .map(|list| list.iter().filter_map(|i| self.entries.get(*i)).collect())
            .unwrap_or_default()
    }

    /// True if the English word (or phrase) is in the list.
    pub fn contains_roman(&self, key: &str) -> bool {
        self.by_roman.contains_key(key)
    }

    /// True if the English word (or phrase) is in the list as a known pair (see
    /// [`Entry::counts_as_known`]).
    pub fn contains_known_roman(&self, key: &str) -> bool {
        self.by_roman(key).iter().any(|e| e.counts_as_known())
    }

    /// True if the Urdu word (or phrase) is in the list as a known pair.
    pub fn contains_known_urdu(&self, urdu: &str) -> bool {
        self.by_urdu(urdu).iter().any(|e| e.counts_as_known())
    }

    /// The entries for an English spelling with their standing in a field, the strongest
    /// first.
    pub(crate) fn ranked_by_roman(&self, key: &str, field: Field) -> Vec<(&Entry, f64)> {
        rank(self.by_roman(key), field)
    }

    /// The entries for an Urdu spelling with their standing in a field, the strongest first.
    pub(crate) fn ranked_by_urdu(&self, urdu: &str, field: Field) -> Vec<(&Entry, f64)> {
        rank(self.by_urdu(urdu), field)
    }

    /// The name list entries of an English spelling (not everyday words, not proposed pairs),
    /// the heaviest first. Names written together are made from these.
    pub(crate) fn names_by_roman(&self, key: &str) -> Vec<&Entry> {
        let mut found = self.by_roman(key);
        found.retain(|e| e.is_name_list());
        found
    }

    /// The name list entries of an Urdu spelling, the heaviest first.
    pub(crate) fn names_by_urdu(&self, urdu: &str) -> Vec<&Entry> {
        let mut found = self.by_urdu(urdu);
        found.retain(|e| e.is_name_list());
        found
    }

    /// The Urdu word a two-word phrase gives for `roman` after the Urdu word `before`.
    pub(crate) fn word_after(&self, before: &str, roman: &str) -> Option<&str> {
        self.after
            .get(&(plain_urdu(before), roman.to_owned()))
            .map(String::as_str)
    }

    /// How many pairs of the name list the spelling model could line up with letter groups,
    /// and how many it could not (those still work as plain lookups). Also trains the model of
    /// free text, so the first key press is not the slow one.
    pub fn training_report(&self) -> (usize, usize) {
        let _ = self.variants();
        let _ = self.model(Field::Text);
        let model = self.model(Field::Name);
        (model.lined_up, model.skipped)
    }

    /// The pairs the spelling model could not line up, for list maintenance.
    pub fn unlined_pairs(&self) -> &[(String, String)] {
        &self.model(Field::Name).skipped_pairs
    }

    /// The model of a field, trained the first time it is needed.
    pub(crate) fn model(&self, field: Field) -> &Model {
        match field {
            Field::Name => self.name_model.get_or_init(|| {
                let names: Vec<&Entry> = self.entries.iter().filter(|e| e.is_name_list()).collect();
                Model::train(&names)
            }),
            Field::Text => self.text_model.get_or_init(|| {
                let mut words: Vec<&Entry> =
                    self.entries.iter().filter(|e| e.is_name_list()).collect();
                words.extend(usual_everyday(&self.entries));
                Model::train(&words)
            }),
        }
    }
}

/// Sorts entries by their standing in a field, then weight, then Urdu order.
fn rank(entries: Vec<&Entry>, field: Field) -> Vec<(&Entry, f64)> {
    let mut found: Vec<(&Entry, f64)> = entries
        .into_iter()
        .map(|e| (e, e.standing(field)))
        .collect();
    found.sort_by(|a, b| {
        b.1.total_cmp(&a.1)
            .then_with(|| b.0.weight.cmp(&a.0.weight))
            .then_with(|| a.0.urdu.cmp(&b.0.urdu))
            .then_with(|| a.0.roman.cmp(&b.0.roman))
    });
    found
}

/// The usual spelling of every single everyday word (its heaviest pair), in file order. The
/// hurried spellings would teach the model wrong letters.
fn usual_everyday(entries: &[Entry]) -> Vec<&Entry> {
    let mut best: HashMap<&str, usize> = HashMap::new();
    for (index, entry) in entries.iter().enumerate() {
        if !entry.is_everyday() || entry.urdu.contains(' ') || entry.roman.contains(' ') {
            continue;
        }
        let heavier = best
            .get(entry.urdu.as_str())
            .and_then(|i| entries.get(*i))
            .is_none_or(|e| entry.weight > e.weight);
        if heavier {
            best.insert(entry.urdu.as_str(), index);
        }
    }
    let mut picked: Vec<usize> = best.into_values().collect();
    picked.sort_unstable();
    picked.into_iter().filter_map(|i| entries.get(i)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "roman\turdu\tkind\tweight\treview\n# a comment\n\
        shahid\tشاہد\tname\t100\tpending\n\
        shahid\tشاھد\tname\t60\tpending\n\
        abdul rehman\tعبدالرحمان\tname\t100\tpending\n\
        multan\tملتان\tplace\t100\treviewed\n";

    #[test]
    fn reads_the_file_form_and_orders_by_weight() {
        let list = WordList::parse(SAMPLE).unwrap();
        assert_eq!(list.len(), 4);
        let spellings: Vec<&str> = list
            .by_roman("shahid")
            .iter()
            .map(|e| e.urdu.as_str())
            .collect();
        assert_eq!(spellings, ["شاہد", "شاھد"]);
        assert_eq!(list.longest_phrase(), 2);
        assert_eq!(list.by_urdu("ملتان")[0].roman, "multan");
        assert_eq!(list.by_urdu("ملتان")[0].review, "reviewed");
        assert!(list.contains_roman("abdul rehman") && !list.contains_roman("abdul"));
    }

    #[test]
    fn bad_lines_are_errors_not_panics() {
        for (text, line) in [
            ("a\tb\tname", 1),
            ("a\tب\tcolour\t1", 1),
            ("a\tب\tname\tmany", 1),
            ("\n\na\tب\tname\t1\tp\ts\textra", 3),
            ("a\t\tname\t1", 1),
        ] {
            assert_eq!(WordList::parse(text).unwrap_err().line, line, "{text:?}");
        }
    }

    #[test]
    fn the_shipped_list_is_readable() {
        let list = WordList::embedded();
        assert!(!list.is_empty(), "translit_seed.tsv could not be read");
    }
}
