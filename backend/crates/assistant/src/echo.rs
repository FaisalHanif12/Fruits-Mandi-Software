//! The Owner's own number, written in words by the model, put back as his number (plan 52 P52-S1).
//!
//! The model may not type a figure, in digits or in words (PRD FR-7.76). When the question says
//! "agle 6 mahine" the model often echoes it as "چھ مہینوں", and the number-word rule refuses the
//! answer: the live run lost a rewrite call (4.8 s) to it in 5 of 369 questions. The word is the
//! Owner's own number, which the answer may show as `{{q.n1}}`. So when a number word of the answer
//! is exactly the word of a number of the question, the app writes the reference for it and checks
//! the answer again. Only the small numbers people say in words (1 to 12, 15, 20, 24, 30, 50) and
//! those with "لاکھ" or "کروڑ" ("تیس لاکھ" for "30 lakh"); every other word stays refused.
//!
//! Example: "پچھلے چھ مہینوں میں" with the question number 6 becomes "پچھلے {{q.n1}} مہینوں میں".

use mandi_services::domain::figures::QuestionNumber;

use crate::check::{padded, word_spans};
use crate::model_answer::{Piece, pieces};
use crate::results::Node;
use crate::tools::State;

/// The words for the numbers people say in words: Urdu, and English for an English answer.
const WORDS: &[(i64, &[&str])] = &[
    (1, &["ایک", "one"]),
    (2, &["دو", "two"]),
    (3, &["تین", "three"]),
    (4, &["چار", "four"]),
    (5, &["پانچ", "five"]),
    (6, &["چھ", "چھے", "چھہ", "six"]),
    (7, &["سات", "seven"]),
    (8, &["آٹھ", "eight"]),
    (9, &["نو", "nine"]),
    (10, &["دس", "ten"]),
    (11, &["گیارہ", "eleven"]),
    (12, &["بارہ", "twelve"]),
    (15, &["پندرہ", "fifteen"]),
    (20, &["بیس", "twenty"]),
    (24, &["چوبیس"]),
    (30, &["تیس", "thirty"]),
    (50, &["پچاس", "fifty"]),
];

const LAKH: i64 = 100_000;
const CRORE: i64 = 10_000_000;

fn words_of(n: i64) -> &'static [&'static str] {
    WORDS
        .iter()
        .find(|(value, _)| *value == n)
        .map_or(&[], |(_, words)| *words)
}

/// What a question number is called in words: one word sequence for each spelling. "3 lakh" is
/// 300,000: "تین لاکھ", "three lakh".
fn spellings(n: i64) -> Vec<Vec<String>> {
    let norm = |w: &str| padded(w).trim().to_lowercase();
    let mut out = Vec::new();
    for word in words_of(n) {
        out.push(vec![norm(word)]);
    }
    for (step, names) in [(LAKH, ["لاکھ", "lakh"]), (CRORE, ["کروڑ", "crore"])] {
        if n > 0 && n.checked_rem(step) == Some(0) {
            for word in words_of(n.checked_div(step).unwrap_or(0)) {
                for unit in names {
                    out.push(vec![norm(word), norm(unit)]);
                }
            }
        }
    }
    out
}

/// A text with each word of a number of the question put back as the reference to it, or `None`
/// when no word is. A longer spelling ("تین لاکھ") wins over a shorter one ("تین").
pub fn put_numbers_back(text: &str, numbers: &[QuestionNumber]) -> Option<String> {
    let mut wanted: Vec<(Vec<String>, String)> = Vec::new();
    for (i, n) in numbers.iter().enumerate() {
        for spelling in spellings(n.number) {
            wanted.push((spelling, format!("q.n{}", i.saturating_add(1))));
        }
    }
    if wanted.is_empty() {
        return None;
    }
    wanted.sort_by_key(|(words, _)| std::cmp::Reverse(words.len()));
    let mut changed = false;
    let mut out = String::new();
    for piece in pieces(text) {
        match piece {
            Piece::Ref(r) => out.push_str(text.get(r.start..r.end).unwrap_or("")),
            Piece::Words(w) => {
                let spans = word_spans(&w);
                let (mut at, mut i) = (0usize, 0usize);
                while i < spans.len() {
                    let hit = wanted.iter().find(|(words, _)| {
                        spans
                            .get(i..i.saturating_add(words.len()))
                            .is_some_and(|run| {
                                run.len() == words.len()
                                    && run.iter().zip(words).all(|((_, _, t), want)| t == want)
                                    && run.windows(2).all(|pair| match pair {
                                        [(_, end, _), (start, _, _)] => {
                                            w.get(*end..*start).is_some_and(|gap| {
                                                gap.chars().all(char::is_whitespace)
                                            })
                                        }
                                        _ => true,
                                    })
                            })
                    });
                    match hit {
                        Some((words, reference)) => {
                            let last = i.saturating_add(words.len()).saturating_sub(1);
                            let (Some(first_span), Some(last_span)) =
                                (spans.get(i), spans.get(last))
                            else {
                                break;
                            };
                            out.push_str(w.get(at..first_span.0).unwrap_or(""));
                            out.push_str(&format!("{{{{{reference}}}}}"));
                            at = last_span.1;
                            i = last.saturating_add(1);
                            changed = true;
                        }
                        None => i = i.saturating_add(1),
                    }
                }
                out.push_str(w.get(at..).unwrap_or(""));
            }
        }
    }
    changed.then_some(out)
}

/// The value of a digit of any script the app takes (English, Arabic-Indic, Urdu), or `None`.
fn digit_value(c: char) -> Option<u64> {
    match c {
        '0'..='9' => c.to_digit(10).map(u64::from),
        '\u{0660}'..='\u{0669}' => u64::from(c).checked_sub(0x0660),
        '\u{06F0}'..='\u{06F9}' => u64::from(c).checked_sub(0x06F0),
        _ => None,
    }
}

/// True for a character that joins digits into one figure (`30,000`, `3.5`) when a digit follows.
fn joins_digits(c: char) -> bool {
    matches!(c, ',' | '.' | '٬' | '٫')
}

/// A text with each standalone number written in digits that is exactly a number of the question put
/// back as the reference to it, or `None` when no number is. The model wrote "3" for the Owner's own
/// "agle 3 mahine"; it is his number, not a figure of the books (the number-word echo, for digits).
/// A number that is part of a longer figure (`30,000`, `3.5`) is never touched.
///
/// Example: "اگلے 3 مہینے" with the question number 3 becomes "اگلے {{q.n1}} مہینے".
pub fn put_digits_back(text: &str, numbers: &[QuestionNumber]) -> Option<String> {
    if numbers.is_empty() {
        return None;
    }
    let mut changed = false;
    let mut out = String::new();
    for piece in pieces(text) {
        match piece {
            Piece::Ref(r) => out.push_str(text.get(r.start..r.end).unwrap_or("")),
            Piece::Words(w) => {
                let chars: Vec<(usize, char)> = w.char_indices().collect();
                let (mut at, mut i) = (0usize, 0usize);
                while i < chars.len() {
                    let Some(&(start, c)) = chars.get(i) else {
                        break;
                    };
                    if digit_value(c).is_none() {
                        i = i.saturating_add(1);
                        continue;
                    }
                    let mut j = i;
                    let mut value: Option<u64> = Some(0);
                    while let Some(&(_, d)) = chars.get(j) {
                        let Some(v) = digit_value(d) else { break };
                        value = value
                            .and_then(|x| x.checked_mul(10))
                            .and_then(|x| x.checked_add(v));
                        j = j.saturating_add(1);
                    }
                    let end = chars.get(j).map_or(w.len(), |&(b, _)| b);
                    let before_joins = i
                        .checked_sub(2)
                        .and_then(|k| chars.get(k))
                        .zip(i.checked_sub(1).and_then(|k| chars.get(k)))
                        .is_some_and(|(&(_, d), &(_, sep))| {
                            digit_value(d).is_some() && joins_digits(sep)
                        });
                    let after_joins = chars
                        .get(j)
                        .zip(chars.get(j.saturating_add(1)))
                        .is_some_and(|(&(_, sep), &(_, d))| {
                            joins_digits(sep) && digit_value(d).is_some()
                        });
                    let hit = value
                        .filter(|_| !before_joins && !after_joins)
                        .and_then(|v| {
                            numbers
                                .iter()
                                .position(|n| u64::try_from(n.number).is_ok_and(|x| x == v))
                        });
                    if let Some(k) = hit {
                        out.push_str(w.get(at..start).unwrap_or(""));
                        out.push_str(&format!("{{{{q.n{}}}}}", k.saturating_add(1)));
                        at = end;
                        changed = true;
                    }
                    i = j;
                }
                out.push_str(w.get(at..).unwrap_or(""));
            }
        }
    }
    changed.then_some(out)
}

/// The path a mistyped reference should have been, or `None` when it needs no change or none is
/// certain: `n1` is the question number `q.n1`, and a place in a keyed list is that row's key
/// (`r3.rows.0.gross` is `r3.rows.i9.gross` when the first row of `r3` is `i9`). The model reads a
/// keyed list as a list of objects in order, each with its `key`, so "the place 0" is the first
/// object it was shown: the same row. A place past the last row, or a key that does not exist, is
/// never guessed: the answer then fails and the rewrite is told the keys (hidden round 4, H363 and
/// Q924: `by_for.0.home_value`).
///
/// Example: `r2.by_for.0.home_value` with the rows `s1` and `guests` is `r2.by_for.s1.home_value`.
fn fixed_path(path: &str, state: &State, numbers: usize) -> Option<String> {
    let segments: Vec<&str> = path.split('.').collect();
    let [first, rest @ ..] = segments.as_slice() else {
        return None;
    };
    if rest.is_empty() {
        let k = first.strip_prefix('n')?.parse::<usize>().ok()?;
        return (k >= 1 && k <= numbers).then(|| format!("q.n{k}"));
    }
    let mut node = &state.find(first)?.node;
    let mut out = vec![(*first).to_owned()];
    let mut changed = false;
    for segment in rest {
        match node {
            Node::Keyed(rows)
                if !rows.iter().any(|(key, _)| key == segment)
                    && segment.parse::<usize>().is_ok_and(|at| at < rows.len()) =>
            {
                let (key, next) = rows.get(segment.parse::<usize>().ok()?)?;
                out.push(key.clone());
                node = next;
                changed = true;
            }
            _ => {
                node = node.get(&[segment])?;
                out.push((*segment).to_owned());
            }
        }
    }
    changed.then(|| out.join("."))
}

/// A text with the references the model mistyped written right where that is certain (see
/// `fixed_path`), or `None` when no reference needed it. The answer is checked again either way.
pub fn fix_references(text: &str, state: &State, numbers: usize) -> Option<String> {
    let mut changed = false;
    let mut out = String::new();
    for piece in pieces(text) {
        match piece {
            Piece::Words(w) => out.push_str(&w),
            Piece::Ref(r) => match fixed_path(&r.path, state, numbers) {
                Some(path) => {
                    out.push_str(&format!("{{{{{path}}}}}"));
                    changed = true;
                }
                None => out.push_str(text.get(r.start..r.end).unwrap_or("")),
            },
        }
    }
    changed.then_some(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use mandi_services::domain::figures::question_numbers;

    fn back(question: &str, text: &str) -> Option<String> {
        put_numbers_back(text, &question_numbers(question))
    }

    #[test]
    fn a_number_word_that_is_the_owners_own_number_becomes_his_number() {
        assert_eq!(
            back(
                "پچھلے 6 مہینوں میں کل کمیشن کتنا رہا",
                "پچھلے چھ مہینوں میں {{r2.totals.total_comm}}"
            )
            .as_deref(),
            Some("پچھلے {{q.n1}} مہینوں میں {{r2.totals.total_comm}}")
        );
        assert_eq!(
            back("average over the last 3 months", "پچھلے تین مکمل مہینوں").as_deref(),
            Some("پچھلے {{q.n1}} مکمل مہینوں")
        );
        // "30 lakh" is two words; the longer spelling wins over "تیس" alone.
        assert_eq!(
            back(
                "when will we pass a target of 30 lakh",
                "کمیشن کا تیس لاکھ ہدف"
            )
            .as_deref(),
            Some("کمیشن کا {{q.n1}} ہدف")
        );
        assert_eq!(
            back("next 12 months", "the next twelve months").as_deref(),
            Some("the next {{q.n1}} months")
        );
    }

    #[test]
    fn a_digit_that_is_the_owners_own_number_becomes_his_number_and_no_other_does() {
        assert_eq!(
            back_digits("agle 3 mahine ka andaza", "اگلے 3 مہینوں کا اندازہ").as_deref(),
            Some("اگلے {{q.n1}} مہینوں کا اندازہ")
        );
        // Urdu digits too.
        assert_eq!(
            back_digits("agle 3 mahine ka andaza", "اگلے ۳ مہینے").as_deref(),
            Some("اگلے {{q.n1}} مہینے")
        );
        // Not his number, part of a longer figure, or inside a reference: left for the rewrite.
        assert_eq!(back_digits("agle 3 mahine ka andaza", "اگلے 6 مہینے"), None);
        assert_eq!(
            back_digits("agle 3 mahine ka andaza", "کل 30,000 اور 3.5 اور 1,3"),
            None
        );
        assert_eq!(
            back_digits("agle 3 mahine ka andaza", "{{r1.rows.a3.days}}"),
            None
        );
        assert_eq!(back_digits("agle mahine ka andaza", "اگلے 3 مہینے"), None);
    }

    fn back_digits(question: &str, text: &str) -> Option<String> {
        put_digits_back(text, &question_numbers(question))
    }

    #[test]
    fn a_word_that_is_not_a_number_of_the_question_stays_for_the_rewrite() {
        // The question says 6; "تین" (3) is not his number, nor a word of another number.
        assert_eq!(back("agle 6 mahine ka commission", "تین مہینے"), None);
        // No number in the question at all.
        assert_eq!(back("agle mahine ka commission", "چھ مہینے"), None);
        // A longer word that only contains the word ("تینوں" = all three) is not the word.
        assert_eq!(back("last 3 months", "تینوں شراکت دار"), None);
        // A number with no word in the table (7,500) is never written back.
        assert_eq!(back("khata of 7500", "ساڑھے سات ہزار"), None);
    }
}
