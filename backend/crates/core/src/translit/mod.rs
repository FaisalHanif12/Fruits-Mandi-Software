//! English letters to Urdu and back, for names (PRD 7.4, Tech Stack 4.4, plan 05).
//!
//! Pure: no database, no files, no clock (Code Standards B28). The same word list, learned
//! pairs and input always give the same suggestions.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

mod combine;
mod compound;
mod forward;
mod glued;
mod graphemes;
mod letters;
mod loose;
mod model;
mod phrase;
mod reverse;
mod source;
mod tables;
mod wordlist;

pub use phrase::words;
pub use source::{
    Script, Source, confirmation_key, english_spelling_source, rules_only_words,
    urdu_spelling_source,
};
pub use wordlist::{Entry, ParseError, WordList};

/// The most suggestions given (PRD FR-7.22).
pub const MAX_SUGGESTIONS: usize = 3;
/// The most characters of a text that are read. The rest is ignored, never an error: the engine
/// takes any input (Code Standards B28).
pub const MAX_INPUT_CHARS: usize = 500;

/// A pair the user chose, offered first the next time (PRD FR-7.26). The services load these
/// from the `translit_pairs` table and pass them in; this module never reads the database.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LearnedPair {
    /// English spelling, lower case, one word.
    pub roman: String,
    /// Urdu spelling, one word.
    pub urdu: String,
    /// How often it was chosen. More often means earlier.
    pub use_count: u32,
}

/// Up to three Urdu spellings for English letters (a word or a phrase). Learned pairs come
/// first, then the word list by weight, then the spelling rules. Urdu letters in the text are
/// kept as they are, and so is anything that is not a letter. Phrases are converted word by
/// word. Any text is fine: it never panics (Code Standards B28).
///
/// ```
/// use mandi_core::translit::{WordList, suggest_to_urdu};
/// let list = WordList::parse("roman\turdu\tkind\tweight\treview\nali\tعلی\tname\t100\tpending").unwrap();
/// assert_eq!(suggest_to_urdu("ali", &[], &list)[0], "علی");
/// assert!(suggest_to_urdu("", &[], &list).is_empty());
/// ```
pub fn suggest_to_urdu(text: &str, learned: &[LearnedPair], list: &WordList) -> Vec<String> {
    texts(suggest(text, learned, list, Direction::ToUrdu))
}

/// Up to three English spellings for Urdu letters, with a capital first letter on every word
/// (PRD FR-7.24). Same order and same promises as [`suggest_to_urdu`].
pub fn suggest_to_english(text: &str, learned: &[LearnedPair], list: &WordList) -> Vec<String> {
    texts(suggest(text, learned, list, Direction::ToEnglish))
}

/// The same suggestions as [`suggest_to_urdu`] and [`suggest_to_english`], each with where it
/// came from (Tech Stack 4.4 v1.2, PRD FR-7.66). The screen marks a word whose source is
/// `rules` with "check the spelling". Names come first, as in a name field; a free-text field
/// uses [`suggest_for`].
///
/// ```
/// use mandi_core::translit::{Direction, Source, WordList, suggest};
/// let list = WordList::parse("roman\turdu\tkind\tweight\treview\nali\tعلی\tname\t100\tpending").unwrap();
/// let got = suggest("ali", &[], &list, Direction::ToUrdu);
/// assert_eq!((got[0].text.as_str(), got[0].source), ("علی", Source::WordList));
/// ```
pub fn suggest(
    text: &str,
    learned: &[LearnedPair],
    list: &WordList,
    direction: Direction,
) -> Vec<Suggestion> {
    suggest_for(text, learned, list, direction, &Hint::default())
}

/// What the screen knows about the field being typed in (plan 99).
#[derive(Debug, Clone, Copy, Default)]
pub struct Hint<'a> {
    /// The kind of the field: [`Kind::Word`] for free text (reasons, narrations, notes),
    /// where everyday words come first. A name kind, or none, means a name field, where
    /// names come first.
    pub kind: Option<Kind>,
    /// The text of the field before the word being typed. Its last word helps to choose:
    /// after کہا, `ke` is کہ.
    pub before: Option<&'a str>,
}

/// [`suggest`] for a field the screen describes (PRD FR-7.22, NFR-URD-30).
///
/// ```
/// use mandi_core::translit::{Direction, Hint, Kind, WordList, suggest_for};
/// let free_text = Hint { kind: Some(Kind::Word), before: None };
/// let got = suggest_for("hai", &[], WordList::embedded(), Direction::ToUrdu, &free_text);
/// assert_eq!(got[0].text, "ہے");
/// ```
pub fn suggest_for(
    text: &str,
    learned: &[LearnedPair],
    list: &WordList,
    direction: Direction,
    hint: &Hint<'_>,
) -> Vec<Suggestion> {
    phrase::suggest(text, learned, list, direction, hint)
}

fn texts(suggestions: Vec<Suggestion>) -> Vec<String> {
    suggestions.into_iter().map(|s| s.text).collect()
}

/// One word of a suggestion and where it came from. A phrase of the word list counts as one
/// word.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct SuggestedWord {
    /// The spelling of this word.
    pub text: String,
    /// Where it came from.
    pub source: Source,
}

/// One suggested spelling of the typed text (PRD FR-7.22, FR-7.66).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct Suggestion {
    /// The whole text, for example `ارشد ملتانی`.
    pub text: String,
    /// The weakest source of its words: `rules` if any word rests on the spelling rules alone.
    pub source: Source,
    /// The converted words of the text, in order, each with its own source. Spaces, digits and
    /// punctuation are not words.
    pub words: Vec<SuggestedWord>,
}

/// Which way a word is converted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    /// English letters typed, Urdu suggested (PRD FR-7.22).
    ToUrdu,
    /// Urdu letters typed, an English name suggested (PRD FR-7.24).
    ToEnglish,
}

/// The kind of a word in the word list and in learned pairs (Architecture 15.3 `translit_pairs`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    /// A person, family or firm name.
    Name,
    /// A city, town or truck stand.
    Place,
    /// A fruit, vegetable or packing.
    Item,
    /// Any other word, for example a glossary word.
    Word,
}

impl Kind {
    /// The storage form, as in `translit_pairs.kind`.
    pub const fn as_str(self) -> &'static str {
        match self {
            Kind::Name => "name",
            Kind::Place => "place",
            Kind::Item => "item",
            Kind::Word => "word",
        }
    }

    /// Reads the storage form.
    pub fn parse(text: &str) -> Option<Kind> {
        match text {
            "name" => Some(Kind::Name),
            "place" => Some(Kind::Place),
            "item" => Some(Kind::Item),
            "word" => Some(Kind::Word),
            _ => None,
        }
    }
}
