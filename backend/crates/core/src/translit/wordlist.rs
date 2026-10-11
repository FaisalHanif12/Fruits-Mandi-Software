//! The word list: reviewed pairs of an English spelling and an Urdu spelling (PRD FR-7.27).
//!
//! The list ships inside the app (`data/translit_seed.tsv`, embedded at build time). One line
//! is one pair: `roman <TAB> urdu <TAB> kind <TAB> weight <TAB> review <TAB> source`. The last
//! two columns are optional when reading. Several Urdu spellings
//! may share one English spelling; the higher weight comes first. A key with spaces (for
//! example `abdul rehman`) is a phrase that is matched as a whole.

use std::collections::HashMap;
use std::sync::OnceLock;

use thiserror::Error;

use super::Kind;
use super::model::Model;
use crate::text::{is_urdu_mark, is_zero_width};

/// The list that ships with the app.
const EMBEDDED: &str = include_str!("../../data/translit_seed.tsv");

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
    /// `pending` until Umar's Urdu reviewer has read it (NFR-URD-4), then `reviewed`.
    pub review: String,
    /// Where the pair came from: `team` (written by us), `wikidata` (CC0), `geonames`
    /// (CC BY 4.0) or `engine` (the Urdu was proposed by our spelling rules). Credits are in
    /// `data/SOURCES.md`. Several sources are joined with `,`.
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

/// The words, indexed both ways, and the model trained on them.
#[derive(Debug)]
pub struct WordList {
    entries: Vec<Entry>,
    by_roman: HashMap<String, Vec<usize>>,
    by_urdu: HashMap<String, Vec<usize>>,
    longest_phrase: usize,
    model: OnceLock<Model>,
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

    /// The list that ships with the app, read once.
    pub fn embedded() -> &'static WordList {
        static LIST: OnceLock<WordList> = OnceLock::new();
        LIST.get_or_init(|| WordList::parse(EMBEDDED).unwrap_or_else(|_| WordList::empty()))
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
        WordList {
            entries,
            by_roman,
            by_urdu,
            longest_phrase,
            model: OnceLock::new(),
        }
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

    /// How many pairs of the list the spelling model could line up with letter groups, and how
    /// many it could not (those still work as plain lookups).
    pub fn training_report(&self) -> (usize, usize) {
        let model = self.model();
        (model.lined_up, model.skipped)
    }

    /// The pairs the spelling model could not line up, for list maintenance.
    pub fn unlined_pairs(&self) -> &[(String, String)] {
        &self.model().skipped_pairs
    }

    /// The model trained on this list, made the first time it is needed.
    pub(crate) fn model(&self) -> &Model {
        self.model.get_or_init(|| Model::train(&self.entries))
    }
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
