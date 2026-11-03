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
