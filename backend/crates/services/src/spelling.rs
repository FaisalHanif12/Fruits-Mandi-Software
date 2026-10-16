//! The spelling check before a named record is saved (PRD FR-7.66, E49, plan 05 Section 4).
//!
//! A spelling that only the spelling rules could have made may be wrong, so it is never saved
//! without the user looking at it. Every service that saves a named record (accounts, items,
//! cities, partners, staff, ...) calls [`check`] with the two names and the words the screen
//! says the user confirmed (`confirmed_rules_words`):
//!
//! 1. [`check`] finds the words of each name that are neither a learned pair nor in the word
//!    list. A word that is not in `confirmed_rules_words` is refused with
//!    `ERR_SPELLING_UNCONFIRMED` on that name field, with `values.words` (the words, separated
//!    by one space). Both fields are reported together.
//! 2. Inside the save's own transaction, [`SpellingCheck::learn`] learns each confirmed word as
//!    a pair (English, Urdu), so the same word is not asked about again (PRD FR-7.26).
//! 3. After the commit, [`Learned::after_commit`] puts the pairs into memory.
//!
//! A confirmed word is paired with the word at the same place in the other name. If the two
//! names have a different number of words there is no safe pairing, so the save goes through
//! and nothing is learned. Free text (notes, reasons) is never checked. A word of one letter
//! is an initial and is never checked.
//!
//! The judging of a word is `mandi_core::translit::rules_only_words`: pure, the same on the
//! screen and here.

use std::collections::HashSet;

use mandi_db::rusqlite::Connection;
use mandi_domain::core::error::{ErrorCode, ErrorValue};
use mandi_domain::core::translit::{
    Direction, Kind, Script, WordList, confirmation_key, rules_only_words,
};

use crate::auth::Actor;
use crate::context::Context;
use crate::error::{Rejection, ServiceError};
use crate::translit::{self, LearnedRow, NewPair};

/// The two names of one record, as they are about to be saved. They are already cleaned and
/// checked for their script (`validate::name_ur`, `validate::name_en`).
#[derive(Debug, Clone, Copy)]
pub struct NamePair<'a> {
    /// Field path of the Urdu name, for the error, for example `name_ur`.
    pub field_ur: &'a str,
    /// The Urdu name.
    pub name_ur: &'a str,
    /// Field path of the English name, for example `name_en`.
    pub field_en: &'a str,
    /// The English name.
    pub name_en: &'a str,
    /// What kind of word the pairs learned from this record are (person, place, item).
    pub kind: Kind,
}

/// A name that passed the check, and the pairs its confirmed words teach.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SpellingCheck {
    to_learn: Vec<NewPair>,
}

/// The pairs written by [`SpellingCheck::learn`], waiting for the commit.
#[derive(Debug, Default)]
#[must_use = "call after_commit once the save has committed"]
pub struct Learned {
    rows: Vec<LearnedRow>,
}

/// The words of a name in order, split the way the check splits them.
fn words_of(name: &str, script: Script) -> Vec<String> {
    let direction = match script {
        Script::Urdu => Direction::ToEnglish,
        Script::English => Direction::ToUrdu,
    };
    mandi_domain::core::translit::words(name, direction)
}

fn rejection(field: &str, words: &[String]) -> Rejection {
    let mut values = std::collections::BTreeMap::new();
    values.insert("words".to_owned(), ErrorValue::Text(words.join(" ")));
    Rejection {
        code: ErrorCode::ErrSpellingUnconfirmed,
        field: Some(field.to_owned()),
        values,
    }
}

/// Checks the two names of a record about to be saved.
///
/// # Errors
/// `ERR_SPELLING_UNCONFIRMED` on `field_ur` and/or `field_en` (value `words`): the words that
/// only the spelling rules could have made and that are not in `confirmed_rules_words`.
/// Nothing is written.
pub fn check(
    ctx: &Context,
    names: &NamePair<'_>,
    confirmed_rules_words: &[String],
) -> Result<SpellingCheck, ServiceError> {
    let learned = translit::learned_pairs(ctx)?;
    let list = WordList::embedded();
    let ur_only = rules_only_words(names.name_ur, Script::Urdu, &learned, list);
    let en_only = rules_only_words(names.name_en, Script::English, &learned, list);

    // A confirmed word may be in either script. Comparing keys of both scripts is safe: an Urdu
    // word never equals an English one.
    let confirmed: HashSet<String> = confirmed_rules_words
        .iter()
        .flat_map(|w| {
            [
                confirmation_key(Script::Urdu, w),
                confirmation_key(Script::English, w),
            ]
        })
        .collect();
    let is_confirmed =
        |script: Script, word: &str| confirmed.contains(&confirmation_key(script, word));

    let open_ur: Vec<String> = ur_only
        .iter()
        .filter(|w| !is_confirmed(Script::Urdu, w))
        .cloned()
        .collect();
    let open_en: Vec<String> = en_only
        .iter()
        .filter(|w| !is_confirmed(Script::English, w))
        .cloned()
        .collect();
    let mut refused = Vec::new();
    if !open_ur.is_empty() {
        refused.push(rejection(names.field_ur, &open_ur));
    }
    if !open_en.is_empty() {
        refused.push(rejection(names.field_en, &open_en));
    }
    if !refused.is_empty() {
        return Err(ServiceError::Rejected(refused));
    }

    // Pair the words of the two names by place, but only when they line up.
    let ur_words = words_of(names.name_ur, Script::Urdu);
    let en_words = words_of(names.name_en, Script::English);
    let mut to_learn: Vec<NewPair> = Vec::new();
    if ur_words.len() == en_words.len() {
        let mut taken: HashSet<(String, String)> = HashSet::new();
        for (urdu, english) in ur_words.iter().zip(&en_words) {
            let was_rules_only = ur_only
                .iter()
                .any(|w| confirmation_key(Script::Urdu, w) == confirmation_key(Script::Urdu, urdu))
                || en_only.iter().any(|w| {
                    confirmation_key(Script::English, w)
                        == confirmation_key(Script::English, english)
                });
            let pair = (english.to_lowercase(), urdu.clone());
            if was_rules_only && taken.insert(pair.clone()) {
                to_learn.push(NewPair {
                    roman: pair.0,
                    urdu: pair.1,
                    kind: names.kind,
                });
            }
        }
    }
    Ok(SpellingCheck { to_learn })
}

/// Checks several name pairs of one record together, so every refused field comes back in one
/// answer (PRD FR-7.66). The checks are in the order of `pairs`.
///
/// # Errors
/// `ERR_SPELLING_UNCONFIRMED` on every field with an open word, from all the pairs.
pub fn check_all(
    ctx: &Context,
    pairs: &[NamePair<'_>],
    confirmed_rules_words: &[String],
) -> Result<Vec<SpellingCheck>, ServiceError> {
    let mut checks = Vec::new();
    let mut refused: Vec<Rejection> = Vec::new();
    for pair in pairs {
        match check(ctx, pair, confirmed_rules_words) {
            Ok(found) => checks.push(found),
            Err(ServiceError::Rejected(items)) => refused.extend(items),
            Err(other) => return Err(other),
        }
    }
    if refused.is_empty() {
        Ok(checks)
    } else {
        Err(ServiceError::Rejected(refused))
    }
}

impl SpellingCheck {
    /// True if confirming these names teaches nothing.
    pub fn is_empty(&self) -> bool {
        self.to_learn.is_empty()
    }

    /// The pairs that will be learned, as (English, Urdu).
    pub fn pairs(&self) -> Vec<(&str, &str)> {
        self.to_learn
            .iter()
            .map(|p| (p.roman.as_str(), p.urdu.as_str()))
            .collect()
    }

    /// Learns the confirmed pairs inside the save's transaction `tx` (PRD FR-7.26). A pair the
    /// table cannot take (for example a word longer than a learned word may be) is skipped: it
    /// is not a reason to refuse the save.
    ///
    /// # Errors
    /// `ServiceError::Db`.
    pub fn learn(
        &self,
        ctx: &Context,
        actor: &Actor,
        tx: &Connection,
    ) -> Result<Learned, ServiceError> {
        let mut rows = Vec::new();
        for pair in &self.to_learn {
            match translit::learn_in(ctx, actor, tx, pair) {
                Ok(row) => rows.push(row),
                Err(ServiceError::Rejected(_)) => {}
                Err(other) => return Err(other),
            }
        }
        Ok(Learned { rows })
    }
}

impl Learned {
    /// Puts the pairs into the copy in memory. Call it after the save has committed.
    pub fn after_commit(self, ctx: &Context) {
        for row in &self.rows {
            translit::remember(ctx, row);
        }
    }

    /// How many pairs were written.
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    /// True if no pair was written.
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
}
