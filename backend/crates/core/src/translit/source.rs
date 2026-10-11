//! Where a spelling came from, and which words of a saved name still rest on the spelling
//! rules alone (Tech Stack 4.4 v1.2, PRD FR-7.66, E49, plan 05 Section 4).
//!
//! A spelling is judged by the spelling itself, not by the path that made it:
//!
//! - `learned`: a learned pair has it (the user chose or confirmed it on this computer),
//! - `word_list`: the word list has it,
//! - `rules`: neither. Only the spelling rules could have made it, so it may be wrong.
//!
//! Judging the text gives the same answer when a suggestion is made and when a name is saved,
//! so a word that was marked "check the spelling" on the screen is the word the save asks
//! about. Pure: the learned pairs and the list are passed in (Code Standards B28).

use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::phrase::{Token, tokenize};
use super::wordlist::{WordList, plain_urdu};
use super::{Direction, LearnedPair};

/// Where a suggested spelling came from. `Rules` is the weakest, `Learned` the strongest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    /// Only the spelling rules made it. The screen marks it "check the spelling" (PRD FR-7.66).
    Rules,
    /// It is in the word list.
    WordList,
    /// A learned pair has it (PRD FR-7.26).
    Learned,
}

impl Source {
    /// The word for the screen and the logs: `rules`, `word_list` or `learned`.
    pub const fn as_str(self) -> &'static str {
        match self {
            Source::Rules => "rules",
            Source::WordList => "word_list",
            Source::Learned => "learned",
        }
    }
}

/// The script of a name: which of the two spellings of a record is being looked at.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Script {
    /// `name_ur`: Urdu letters.
    Urdu,
    /// `name_en`: English letters.
    English,
}

/// The shortest word, in letters, that the spelling check looks at. A single letter is an
/// initial (`M Aslam`), not a spelling.
const MIN_CHECKED_LETTERS: usize = 2;

/// The Urdu spelling of a word or phrase, judged as described in the module text. Marks such
/// as zabar do not count, as in the word list.
///
/// ```
/// use mandi_core::translit::{Source, WordList, urdu_spelling_source};
/// let list = WordList::parse("roman\turdu\tkind\tweight\treview\nali\tعلی\tname\t100\tpending").unwrap();
/// assert_eq!(urdu_spelling_source("علی", &[], &list), Source::WordList);
/// assert_eq!(urdu_spelling_source("زیڈ", &[], &list), Source::Rules);
/// ```
pub fn urdu_spelling_source(urdu: &str, learned: &[LearnedPair], list: &WordList) -> Source {
    let plain = plain_urdu(urdu);
    if learned.iter().any(|p| plain_urdu(&p.urdu) == plain) {
        Source::Learned
    } else if list.contains_known_urdu(urdu) {
        Source::WordList
    } else {
        Source::Rules
    }
}

/// The English spelling of a word or phrase, judged as described in the module text. Case does
/// not matter.
pub fn english_spelling_source(english: &str, learned: &[LearnedPair], list: &WordList) -> Source {
    let lower = english
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase();
    if learned.iter().any(|p| p.roman == lower) {
        Source::Learned
    } else if list.contains_known_roman(&lower) {
        Source::WordList
    } else {
        Source::Rules
    }
}

fn source_in(script: Script, text: &str, learned: &[LearnedPair], list: &WordList) -> Source {
    match script {
        Script::Urdu => urdu_spelling_source(text, learned, list),
        Script::English => english_spelling_source(text, learned, list),
    }
}

/// The key a confirmed word is compared by: Urdu without marks, English in lower case.
pub fn confirmation_key(script: Script, word: &str) -> String {
    match script {
        Script::Urdu => plain_urdu(word).trim().to_owned(),
        Script::English => word.trim().to_lowercase(),
    }
}

/// The words of a name that only the spelling rules could have made: each word (or phrase of
/// the list) that is neither a learned pair nor in the word list, in the order they stand in
/// the name, each once. Words of one letter are initials and are skipped. Digits, punctuation
/// and letters of the other script are not words and are skipped. Never panics on any text.
///
/// ```
/// use mandi_core::translit::{Script, WordList, rules_only_words};
/// let list = WordList::parse("roman\turdu\tkind\tweight\treview\nali\tعلی\tname\t100\tpending").unwrap();
/// assert_eq!(rules_only_words("Ali Khichi", Script::English, &[], &list), ["Khichi"]);
/// assert!(rules_only_words("علی", Script::Urdu, &[], &list).is_empty());
/// ```
pub fn rules_only_words(
    name: &str,
    script: Script,
    learned: &[LearnedPair],
    list: &WordList,
) -> Vec<String> {
    let direction = match script {
        Script::Urdu => Direction::ToEnglish,
        Script::English => Direction::ToUrdu,
    };
    let tokens = tokenize(name, direction);
    let mut found: Vec<String> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    let mut index = 0_usize;
    while let Some(token) = tokens.get(index) {
        let Token::Word(word) = token else {
            index = index.saturating_add(1);
            continue;
        };
        // A phrase of the list (`abdul rehman`) covers its words, longest first.
        let mut covered = 1_usize;
        let mut known = false;
        for words in (2..=list.longest_phrase()).rev() {
            let Some((key, span)) = super::phrase::phrase_at(&tokens, index, words) else {
                continue;
            };
            if source_in(script, &key, learned, list) != Source::Rules {
                covered = span;
                known = true;
                break;
            }
        }
        if !known {
            let letters = plain_urdu(word).chars().count();
            known = letters < MIN_CHECKED_LETTERS
                || source_in(script, word, learned, list) != Source::Rules;
        }
        if !known && seen.insert(confirmation_key(script, word)) {
            found.push(word.clone());
        }
        index = index.saturating_add(covered);
    }
    found
}
