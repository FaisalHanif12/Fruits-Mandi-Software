//! Words and phrases (plan 05 Section 4).
//!
//! A text is cut into words, spaces and everything else. Each word gets a ranked list of
//! spellings: learned pairs first, then the word list by weight, then the spelling rules.
//! Phrases that are in the word list as a whole (`abdul rehman`) are taken as one unit. The
//! best three whole texts are made by trying the lists in order of their scores.

use std::collections::HashSet;

use super::combine::{Scored, Slot, Unit, best_texts};
use super::compound;
use super::source::{self, Script};
use super::wordlist::{WordList, plain_urdu};
use super::{
    Direction, LearnedPair, MAX_INPUT_CHARS, MAX_SUGGESTIONS, Source, SuggestedWord, Suggestion,
    forward, reverse,
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

/// The ranked spellings of one word (or one phrase of the list).
fn word_candidates(
    word: &str,
    learned: &[LearnedPair],
    list: &WordList,
    direction: Direction,
) -> Vec<Scored> {
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
    // 2. The word list, the heaviest first.
    let entries = match direction {
        Direction::ToUrdu => list.by_roman(&lower),
        Direction::ToEnglish => list.by_urdu(word),
    };
    let top = entries.first().map_or(1, |e| e.weight.max(1));
    for entry in &entries {
        let text = match direction {
            Direction::ToUrdu => entry.urdu.clone(),
            Direction::ToEnglish => capitalise(&entry.roman),
        };
        let score = 0.95 * f64::from(entry.weight) / f64::from(top);
        push_unique(&mut found, &mut seen, text, score);
    }
    let model = list.model();
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
            let known = match direction {
                Direction::ToUrdu => !list.by_urdu(text).is_empty(),
                Direction::ToEnglish => list.contains_roman(&text.to_lowercase()),
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
    learned: &[LearnedPair],
    list: &WordList,
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
    let tails = word_candidates(next, learned, list, Direction::ToUrdu);
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
) -> Vec<Suggestion> {
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
                    let found = phrase_candidates(&key, list, direction);
                    if !found.is_empty() {
                        candidates = found;
                        taken = span;
                        break;
                    }
                }
                if candidates.is_empty()
                    && direction == Direction::ToUrdu
                    && let Some((found, span)) = joined_with_next(&tokens, index, learned, list)
                {
                    candidates = found;
                    taken = span;
                }
                if candidates.is_empty() {
                    words_done = words_done.saturating_add(1);
                    candidates = if words_done > MAX_WORDS {
                        Vec::new()
                    } else {
                        word_candidates(word, learned, list, direction)
                    };
                }
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
fn phrase_candidates(key: &str, list: &WordList, direction: Direction) -> Vec<Scored> {
    let entries = match direction {
        Direction::ToUrdu => list.by_roman(&key.to_lowercase()),
        Direction::ToEnglish => list.by_urdu(key),
    };
    let top = entries.first().map_or(1, |e| e.weight.max(1));
    let mut seen = HashSet::new();
    let mut found = Vec::new();
    for entry in entries {
        let text = match direction {
            Direction::ToUrdu => entry.urdu.clone(),
            Direction::ToEnglish => capitalise(&entry.roman),
        };
        push_unique(
            &mut found,
            &mut seen,
            text,
            0.95 * f64::from(entry.weight) / f64::from(top),
        );
    }
    monotone(found)
}
