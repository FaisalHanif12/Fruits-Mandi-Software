//! Words and phrases (plan 05 Section 4, plan 99).
//!
//! A text is cut into words, spaces and everything else. Each word gets a ranked list of
//! spellings: learned pairs first, then the word the word before asks for (a two-word phrase
//! of the everyday dictionary: after کہا, `ke` is کہ), then the word list by its standing in
//! the field (in a name field the names come first, in free text the everyday words), then
//! the spelling rules. Phrases that are in the word list as a whole (`abdul rehman`, `ly kr`)
//! are taken as one unit. The best three whole texts are made by trying the lists in order of
//! their scores.

use std::collections::HashSet;

use super::combine::{Scored, Slot, Unit, best_texts};
use super::compound;
use super::loose;
use super::source::{self, Script};
use super::wordlist::{Entry, Field, WordList, plain_urdu};
use super::{
    Direction, Hint, Kind, LearnedPair, MAX_INPUT_CHARS, MAX_SUGGESTIONS, Source, SuggestedWord,
    Suggestion, forward, reverse,
};
use crate::text;

/// Spelling candidates kept for each word.
const PER_WORD: usize = 5;
/// Spellings the rules make before the best are kept.
const RULE_POOL: usize = 12;
/// How much more likely a rule spelling is if it is a word of the list.
const KNOWN_WORD_BOOST: f64 = 6.0;
/// Words in a text that are converted. The rest are kept as typed.
const MAX_WORDS: usize = 16;
/// The longest word the rules try, in letters. Longer words are kept as typed.
const MAX_WORD_LETTERS: usize = 40;
/// How far back in the text before the word the word before is looked for.
const CONTEXT_CHARS: usize = 60;
/// The score of the word that the word before asks for: after learned pairs, before the list.
const AFTER_WORD_SCORE: f64 = 0.97;
/// The score of the best other spelling of a known word, when the word itself is not listed.
const OTHER_SPELLING_ALONE: f64 = 0.92;
/// The same, when the list has the typed spelling too.
const OTHER_SPELLING_AFTER: f64 = 0.55;

/// The field the text is typed in. English names are always spelled as names.
fn field_of(direction: Direction, hint: &Hint<'_>) -> Field {
    if direction == Direction::ToUrdu && hint.kind == Some(Kind::Word) {
        Field::Text
    } else {
        Field::Name
    }
}

/// The Urdu word at the very end of the text, if only spaces follow it. Punctuation or a
/// number after it ends the phrase, so there is no word before.
fn last_urdu_word(before: &str) -> Option<String> {
    let skip = before.chars().count().saturating_sub(CONTEXT_CHARS);
    let tail: String = before.chars().skip(skip).collect();
    match tokenize(&tail, Direction::ToEnglish).pop() {
        Some(Token::Word(word)) => Some(word),
        _ => None,
    }
}

/// A piece of the text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Token {
    /// A word of letters of the script being converted.
    Word(String),
    /// One space (any run of white space).
    Space,
    /// Anything else, kept as typed: digits, punctuation, other scripts, emoji.
    Fixed(String),
}

fn is_letter(direction: Direction, c: char) -> bool {
    match direction {
        Direction::ToUrdu => c.is_ascii_alphabetic(),
        Direction::ToEnglish => text::is_urdu_letter(c),
    }
}

pub(super) fn tokenize(input: &str, direction: Direction) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();
    for c in input.chars().take(MAX_INPUT_CHARS) {
        if c.is_whitespace() {
            if !matches!(tokens.last(), Some(Token::Space)) && !tokens.is_empty() {
                tokens.push(Token::Space);
            }
        } else if is_letter(direction, c)
            || (direction == Direction::ToEnglish
                && text::is_urdu_mark(c)
                && matches!(tokens.last(), Some(Token::Word(_))))
        {
            match tokens.last_mut() {
                Some(Token::Word(word)) => word.push(c),
                _ => tokens.push(Token::Word(c.to_string())),
            }
        } else {
            match tokens.last_mut() {
                Some(Token::Fixed(text)) => text.push(c),
                _ => tokens.push(Token::Fixed(c.to_string())),
            }
        }
    }
    while matches!(tokens.last(), Some(Token::Space)) {
        tokens.pop();
    }
    tokens
}

/// Capital first letter of every word: `abdul rehman` becomes `Abdul Rehman`.
fn capitalise(text: &str) -> String {
    text.split(' ')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().chain(chars).collect::<String>(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Makes the scores fall from first to last, whatever they were.
fn monotone(mut list: Vec<Scored>) -> Vec<Scored> {
    let mut ceiling = 1.0_f64;
    for (_, score) in &mut list {
        ceiling = ceiling.min(*score);
        *score = ceiling;
    }
    list
}

/// Adds a spelling unless it is already there.
fn push_unique(list: &mut Vec<Scored>, seen: &mut HashSet<String>, text: String, score: f64) {
    // Never an empty spelling: a word whose letters are all unwritten gives nothing to offer.
    if !text.trim().is_empty() && seen.insert(text.clone()) {
        list.push((text, score));
    }
}

/// The standing the other spellings are measured against: the first one's, else 1.
fn top_standing(entries: &[(&Entry, f64)]) -> f64 {
    entries
        .first()
        .map(|(_, s)| *s)
        .filter(|s| *s > 0.0)
        .unwrap_or(1.0)
}

/// What a word is spelled with: the learned pairs, the list and the field.
#[derive(Clone, Copy)]
struct Spelling<'a> {
    learned: &'a [LearnedPair],
    list: &'a WordList,
    direction: Direction,
    field: Field,
}

/// The ranked spellings of one word (or one phrase of the list). `before` is the Urdu word
/// just before it, if there is one.
fn word_candidates(word: &str, before: Option<&str>, spelling: Spelling<'_>) -> Vec<Scored> {
    let Spelling {
        learned,
        list,
        direction,
        field,
    } = spelling;
    // Marks (zabar, khari zabar) do not change which word it is.
    let plain = if direction == Direction::ToEnglish {
        plain_urdu(word)
    } else {
        word.to_owned()
    };
    let word = plain.as_str();
    let lower = word.to_lowercase();
    let mut found: Vec<Scored> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    // 1. Learned pairs, the most used first.
    let mut mine: Vec<&LearnedPair> = learned
        .iter()
        .filter(|pair| match direction {
            Direction::ToUrdu => pair.roman == lower,
            Direction::ToEnglish => plain_urdu(&pair.urdu) == word,
        })
        .collect();
    mine.sort_by(|a, b| {
        b.use_count
            .cmp(&a.use_count)
            .then_with(|| a.urdu.cmp(&b.urdu))
            .then_with(|| a.roman.cmp(&b.roman))
    });
    for (rank, pair) in mine.iter().enumerate() {
        let text = match direction {
            Direction::ToUrdu => pair.urdu.clone(),
            Direction::ToEnglish => capitalise(&pair.roman),
        };
        let score = 1.0 - 0.001 * f64::from(u32::try_from(rank).unwrap_or(u32::MAX));
        push_unique(&mut found, &mut seen, text, score);
    }
    // 1b. The word the word before asks for.
    if direction == Direction::ToUrdu
        && let Some(urdu) = before.and_then(|b| list.word_after(b, &lower))
    {
        push_unique(&mut found, &mut seen, urdu.to_owned(), AFTER_WORD_SCORE);
    }
    // 2. The word list, the strongest in this field first.
    let entries = match direction {
        Direction::ToUrdu => list.ranked_by_roman(&lower, field),
        Direction::ToEnglish => list.ranked_by_urdu(word, field),
    };
    let top = top_standing(&entries);
    for (entry, standing) in &entries {
        let text = match direction {
            Direction::ToUrdu => entry.urdu.clone(),
            Direction::ToEnglish => capitalise(&entry.roman),
        };
        push_unique(&mut found, &mut seen, text, 0.95 * standing / top);
    }
    // 2a. In free text, a word we know typed another way: `accha` for `acha`, `aarahe` for
    // `aa rahe`, or a hurried `mjhy`.
    if direction == Direction::ToUrdu && field == Field::Text && found.len() < MAX_SUGGESTIONS {
        let share = if entries.is_empty() {
            OTHER_SPELLING_ALONE
        } else {
            OTHER_SPELLING_AFTER
        };
        for (text, score) in loose::other_spellings(list, &lower, !entries.is_empty())
            .into_iter()
            .take(MAX_SUGGESTIONS)
        {
            push_unique(&mut found, &mut seen, text, share * score);
        }
    }
    let model = list.model(field);
    // 2b. A word that is not in the list but is made of parts that are.
    if entries.is_empty() && !word.contains(' ') && word.chars().count() <= MAX_WORD_LETTERS {
        for (text, score) in compound_candidates(model, list, word, &lower, direction) {
            push_unique(&mut found, &mut seen, text, 0.9 * score);
        }
    }
    // 3. The spelling rules, only while there are fewer than the most that are shown.
    if (found.len() < MAX_SUGGESTIONS || entries.is_empty())
        && !word.contains(' ')
        && word.chars().count() <= MAX_WORD_LETTERS
    {
        let mut made = match direction {
            Direction::ToUrdu => forward::candidates(model, &lower, RULE_POOL),
            Direction::ToEnglish => reverse::candidates(model, word, RULE_POOL),
        };
        // A spelling the rules make that is also a word of the list is much more likely to be
        // right (the user typed another way of writing a word we know).
        for (text, score) in &mut made {
            let known = match (direction, field) {
                (Direction::ToUrdu, Field::Name) => !list.names_by_urdu(text).is_empty(),
                (Direction::ToUrdu, Field::Text) => list.contains_known_urdu(text),
                (Direction::ToEnglish, _) => !list.names_by_roman(&text.to_lowercase()).is_empty(),
            };
            if known {
                *score *= KNOWN_WORD_BOOST;
            }
        }
        made.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        let best = made.first().map_or(1.0, |(_, s)| *s);
        for (text, score) in made {
            let text = match direction {
                Direction::ToUrdu => text,
                Direction::ToEnglish => capitalise(&text),
            };
            let relative = if best > 0.0 { score / best } else { 0.0 };
            push_unique(&mut found, &mut seen, text, 0.5 * relative);
        }
    }
    found.truncate(PER_WORD);
    monotone(found)
}

/// Spellings of a word made of known parts (see the `compound` module).
fn compound_candidates(
    model: &super::model::Model,
    list: &WordList,
    word: &str,
    lower: &str,
    direction: Direction,
) -> Vec<Scored> {
    match direction {
        Direction::ToUrdu => compound::urdu_compounds(model, list, lower),
        Direction::ToEnglish => {
            let mut made: Vec<Scored> = compound::roman_compounds(model, list, word)
                .into_iter()
                .map(|(text, score)| (capitalise(&text), score))
                .collect();
            // `الرحمن` on its own: `Ur Rehman`.
            if let Some(tail) = word
                .strip_prefix(compound::AL_URDU)
                .filter(|t| t.chars().count() >= 2)
            {
                let particle = compound::particle_for(tail);
                for (rank, part) in list
                    .by_urdu(tail)
                    .iter()
                    .map(|e| e.roman.clone())
                    .chain(
                        reverse::candidates(model, tail, 2)
                            .into_iter()
                            .map(|(text, _)| text),
                    )
                    .take(2)
                    .enumerate()
                {
                    let drop = 0.02 * f64::from(u32::try_from(rank).unwrap_or(0));
                    made.push((capitalise(&format!("{particle} {part}")), 0.9 - drop));
                }
            }
            made.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
            made
        }
    }
}

/// `ur rehman` and `abdul samad`: a joining word and the name after it make one unit. The
/// unit and how many tokens it covers, if the word at `index` is one.
fn joined_with_next(
    tokens: &[Token],
    index: usize,
    spelling: Spelling<'_>,
) -> Option<(Vec<Scored>, usize)> {
    let Some(Token::Word(word)) = tokens.get(index) else {
        return None;
    };
    let lower = word.to_lowercase();
    let is_particle = compound::PARTICLES.contains(&lower.as_str())
        && index > 0
        && matches!(tokens.get(index.saturating_sub(1)), Some(Token::Space));
    let is_abdul = compound::ABDUL.contains(&lower.as_str());
    if !is_particle && !is_abdul {
        return None;
    }
    let Some(Token::Space) = tokens.get(index.saturating_add(1)) else {
        return None;
    };
    let Some(Token::Word(next)) = tokens.get(index.saturating_add(2)) else {
        return None;
    };
    let tails = word_candidates(next, None, spelling);
    let mut made: Vec<Scored> = Vec::new();
    for (tail, score) in tails.iter().take(3) {
        if is_particle {
            made.push((format!("{}{tail}", compound::AL_URDU), *score));
        } else {
            made.push((format!("{}{tail}", compound::ABDUL_URDU), 0.9 * score));
            made.push((format!("عبد {}{tail}", compound::AL_URDU), 0.5 * score));
        }
    }
    made.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    Some((monotone(made), 3))
}

/// The words of a text in order, the way the engine cuts it: letters of the script being
/// converted (for `ToUrdu` the English letters, for `ToEnglish` the Urdu letters). Spaces,
/// digits, punctuation and the other script are not words. At most [`MAX_INPUT_CHARS`]
/// characters are read.
pub fn words(input: &str, direction: Direction) -> Vec<String> {
    tokenize(input, direction)
        .into_iter()
        .filter_map(|token| match token {
            Token::Word(word) => Some(word),
            _ => None,
        })
        .collect()
}

/// The suggestions for a text (see `suggest_to_urdu` and `suggest_to_english`), each with
/// where its words came from.
pub(super) fn suggest(
    input: &str,
    learned: &[LearnedPair],
    list: &WordList,
    direction: Direction,
    hint: &Hint<'_>,
) -> Vec<Suggestion> {
    let spelling = Spelling {
        learned,
        list,
        direction,
        field: field_of(direction, hint),
    };
    let mut before: Option<String> = match direction {
        Direction::ToUrdu => hint.before.and_then(last_urdu_word),
        Direction::ToEnglish => None,
    };
    let tokens = tokenize(input, direction);
    let mut layout: Vec<Slot> = Vec::new();
    let mut units: Vec<Unit> = Vec::new();
    let mut words_done = 0_usize;
    let mut index = 0_usize;
    while let Some(token) = tokens.get(index) {
        match token {
            Token::Space => {
                layout.push(Slot::Space);
                index = index.saturating_add(1);
            }
            Token::Fixed(text) => {
                // Urdu already in the text counts as the word before; anything else ends
                // the phrase.
                before = match direction {
                    Direction::ToUrdu => last_urdu_word(text),
                    Direction::ToEnglish => None,
                };
                layout.push(Slot::Fixed(text.clone()));
                index = index.saturating_add(1);
            }
            Token::Word(word) => {
                // A phrase of the list: several words with single spaces between them.
                let mut taken = 1_usize;
                let mut candidates: Vec<Scored> = Vec::new();
                for words in (2..=list.longest_phrase()).rev() {
                    let Some((key, span)) = phrase_at(&tokens, index, words) else {
                        continue;
                    };
                    let found = phrase_candidates(&key, spelling);
                    if !found.is_empty() {
                        candidates = found;
                        taken = span;
                        break;
                    }
                }
                if candidates.is_empty()
                    && direction == Direction::ToUrdu
                    && let Some((found, span)) = joined_with_next(&tokens, index, spelling)
                {
                    candidates = found;
                    taken = span;
                }
                if candidates.is_empty() {
                    words_done = words_done.saturating_add(1);
                    candidates = if words_done > MAX_WORDS {
                        Vec::new()
                    } else {
                        word_candidates(word, before.as_deref(), spelling)
                    };
                }
                before = candidates
                    .first()
                    .filter(|_| direction == Direction::ToUrdu)
                    .and_then(|(text, _)| text.split(' ').next_back())
                    .map(str::to_owned);
                if candidates.is_empty() {
                    // Nothing to offer: the word stays as typed.
                    layout.push(Slot::Fixed(word.clone()));
                } else {
                    layout.push(Slot::Unit(units.len()));
                    units.push(Unit { candidates });
                }
                index = index.saturating_add(taken);
            }
        }
    }
    if units.is_empty() {
        return Vec::new();
    }
    let script = match direction {
        Direction::ToUrdu => Script::Urdu,
        Direction::ToEnglish => Script::English,
    };
    best_texts(&layout, &units)
        .into_iter()
        .map(|(text, picks)| {
            // The source of each word is judged by its spelling (see `source`), so it is the
            // same answer as at save time.
            let words: Vec<SuggestedWord> = units
                .iter()
                .zip(&picks)
                .filter_map(|(unit, pick)| unit.candidates.get(*pick))
                .map(|(word, _)| SuggestedWord {
                    text: word.clone(),
                    source: match script {
                        Script::Urdu => source::urdu_spelling_source(word, learned, list),
                        Script::English => source::english_spelling_source(word, learned, list),
                    },
                })
                .collect();
            // The whole suggestion is as strong as its weakest word.
            let source = words
                .iter()
                .map(|w| w.source)
                .min()
                .unwrap_or(Source::Rules);
            Suggestion {
                text,
                source,
                words,
            }
        })
        .collect()
}

/// `words` words starting at token `start`, with one space between each: the key and how many
/// tokens it covers.
pub(super) fn phrase_at(tokens: &[Token], start: usize, words: usize) -> Option<(String, usize)> {
    let mut parts: Vec<&str> = Vec::new();
    let mut cursor = start;
    for n in 0..words {
        if n > 0 {
            if !matches!(tokens.get(cursor), Some(Token::Space)) {
                return None;
            }
            cursor = cursor.saturating_add(1);
        }
        match tokens.get(cursor) {
            Some(Token::Word(word)) => parts.push(word),
            _ => return None,
        }
        cursor = cursor.saturating_add(1);
    }
    Some((parts.join(" "), cursor.saturating_sub(start)))
}

/// The spellings of a phrase that is in the list as a whole.
fn phrase_candidates(key: &str, spelling: Spelling<'_>) -> Vec<Scored> {
    let entries = match spelling.direction {
        Direction::ToUrdu => spelling
            .list
            .ranked_by_roman(&key.to_lowercase(), spelling.field),
        Direction::ToEnglish => spelling.list.ranked_by_urdu(key, spelling.field),
    };
    let top = top_standing(&entries);
    let mut seen = HashSet::new();
    let mut found = Vec::new();
    for (entry, standing) in entries {
        let text = match spelling.direction {
            Direction::ToUrdu => entry.urdu.clone(),
            Direction::ToEnglish => capitalise(&entry.roman),
        };
        push_unique(&mut found, &mut seen, text, 0.95 * standing / top);
    }
    monotone(found)
}
