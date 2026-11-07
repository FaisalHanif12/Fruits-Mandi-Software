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

use mandi_services::domain::figures::{NumberUnit, QuestionNumber};

use crate::check::{padded, word_spans};
use crate::model_answer::{Piece, pieces};
use crate::results::{Node, Val};
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
/// A number that is part of a longer figure (`30,000`, `3.5`) is never touched. A number that reads as an
/// amount (a plain number of 100 or more, or one with lakh, hazar or crore) is never put back either:
/// "kya akram ka bakaya 5000 hai" answered "ہاں، 5000" would confirm the Owner's own number as if the
/// books had said it, with no read (D7 R13). The answer then fails the digit check and the rewrite
/// has to cite the books.
///
/// Example: "اگلے 3 مہینے" with the question number 3 becomes "اگلے {{q.n1}} مہینے".
pub fn put_digits_back(text: &str, numbers: &[QuestionNumber]) -> Option<String> {
    const COUNT_LIMIT: i64 = 100;
    // Each number keeps its place in the question's list: `{{q.nK}}` points into the FULL list, so a
    // number put back after an amount was left out must still be named by its own place (money review 11:
    // "bakaya 5000 hai pichle 3 mahine" has n1 = 5000 and n2 = 3, and the 3 is `q.n2`).
    let usable: Vec<(usize, &QuestionNumber)> = numbers
        .iter()
        .enumerate()
        .filter(|(_, n)| match n.unit {
            NumberUnit::Months | NumberUnit::Days | NumberUnit::Years | NumberUnit::Percent => true,
            NumberUnit::Plain => n.number < COUNT_LIMIT,
            NumberUnit::Amount => false,
        })
        .collect();
    if usable.is_empty() {
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
                            usable
                                .iter()
                                .find(|(_, n)| u64::try_from(n.number).is_ok_and(|x| x == v))
                                .map(|(place, _)| *place)
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

/// The month names the check refuses (`words.tsv`, kind month), by month number: Urdu, English in
/// full and the English short forms. A month is told by its exact spelling, as the check does
/// (`September` is refused, `september` is not).
const MONTH_NAMES: [(u32, &[&str]); 12] = [
    (1, &["جنوری", "January", "Jan"]),
    (2, &["فروری", "February", "Feb"]),
    (3, &["مارچ", "March", "Mar"]),
    (4, &["اپریل", "April", "Apr"]),
    (5, &["مئی", "May"]),
    (6, &["جون", "June", "Jun"]),
    (7, &["جولائی", "July", "Jul"]),
    (8, &["اگست", "August", "Aug"]),
    (9, &["ستمبر", "September", "Sep", "Sept"]),
    (10, &["اکتوبر", "October", "Oct"]),
    (11, &["نومبر", "November", "Nov"]),
    (12, &["دسمبر", "December", "Dec"]),
];

/// The number of the month a word names, by its exact spelling, or `None`.
fn month_of_word(word: &str) -> Option<u32> {
    let norm = |w: &str| padded(w).trim().to_owned();
    let word = norm(word);
    MONTH_NAMES
        .iter()
        .find(|(_, names)| names.iter().any(|n| norm(n) == word))
        .map(|(month, _)| *month)
}

/// Every month value of a result, with the path that holds it: (month, year, path).
fn walk_months(path: &str, node: &Node, out: &mut Vec<(u32, i32, String)>) {
    match node {
        Node::Val(Val::Month { year, month }) => out.push((*month, *year, path.to_owned())),
        Node::Obj(fields) | Node::Keyed(fields) => {
            for (key, value) in fields {
                walk_months(&format!("{path}.{key}"), value, out);
            }
        }
        Node::List(items) => {
            for (i, value) in items.iter().enumerate() {
                walk_months(&format!("{path}.{i}"), value, out);
            }
        }
        _ => {}
    }
}

/// The one path for each month the results hold, when every value of that month is of one year:
/// the `label` of a period if it is one of them, else the first. A month held in two years (a
/// series over more than a year) has no one path and is left to the rewrite.
fn one_path_per_month(held: &[(u32, i32, String)]) -> Vec<(u32, i32, String)> {
    let mut out: Vec<(u32, i32, String)> = Vec::new();
    for month in 1..=12u32 {
        let of_month: Vec<&(u32, i32, String)> = held.iter().filter(|h| h.0 == month).collect();
        let Some(first) = of_month.first() else {
            continue;
        };
        if of_month.iter().any(|h| h.1 != first.1) {
            continue;
        }
        let chosen = of_month
            .iter()
            .find(|h| h.2.ends_with(".label"))
            .unwrap_or(first);
        out.push((month, chosen.1, chosen.2.clone()));
    }
    out
}

/// A text with each month name typed in it put back as the reference to that month in the results,
/// and the year typed right after it with it (the reference shows month and year), or `None` when no
/// month is. The model types "ستمبر 2026" for the period it was given as a `label`: the month rule
/// refuses it and a rewrite call (3 to 6 seconds) follows. Only a month the results hold in ONE year
/// is put back; any other digit stays and is refused as before.
///
/// Example: "ستمبر 2026 میں بکری" with the label of period `r1` (September 2026) becomes
/// "{{r1.label}} میں بکری".
pub fn put_months_back(text: &str, state: &State) -> Option<String> {
    let mut held = Vec::new();
    for result in state.results.iter().filter(|r| r.turn == state.turn) {
        walk_months(&result.id, &result.node, &mut held);
    }
    put_months(text, &one_path_per_month(&held))
}

/// The length in bytes of the white space and the 4 digits of `year` at the start of `rest`, or
/// `None` when what follows the month is not exactly that year (`2026`, `۲۰۲۶`; not `20265`, not
/// `2026.5`, not another year).
fn year_after(rest: &str, year: i32) -> Option<usize> {
    let body = rest.trim_start_matches(char::is_whitespace);
    let lead = rest.len().saturating_sub(body.len());
    let digits: Vec<u64> = body.chars().map_while(digit_value).collect();
    if digits.len() != 4 {
        return None;
    }
    let value = digits
        .iter()
        .fold(0u64, |a, d| a.saturating_mul(10).saturating_add(*d));
    if i32::try_from(value).ok() != Some(year) {
        return None;
    }
    let taken: usize = body.chars().take(4).map(char::len_utf8).sum();
    // A mark that joins digits into a longer figure ends it as not a year.
    let mut after = body.get(taken..).unwrap_or("").chars();
    if let (Some(mark), next) = (after.next(), after.next())
        && joins_digits(mark)
        && next.is_some_and(|c| digit_value(c).is_some())
    {
        return None;
    }
    Some(lead.saturating_add(taken))
}

/// [`put_months_back`] for the months already chosen: (month, year, path).
fn put_months(text: &str, wanted: &[(u32, i32, String)]) -> Option<String> {
    if wanted.is_empty() {
        return None;
    }
    let mut changed = false;
    let mut out = String::new();
    for piece in pieces(text) {
        match piece {
            Piece::Ref(r) => out.push_str(text.get(r.start..r.end).unwrap_or("")),
            Piece::Words(w) => {
                let spans = word_spans(&w);
                let (mut at, mut i) = (0usize, 0usize);
                while i < spans.len() {
                    let Some((start, end, _)) = spans.get(i).cloned() else {
                        break;
                    };
                    let hit = w
                        .get(start..end)
                        .and_then(month_of_word)
                        .and_then(|m| wanted.iter().find(|(wm, _, _)| *wm == m));
                    let Some((_, year, path)) = hit else {
                        i = i.saturating_add(1);
                        continue;
                    };
                    out.push_str(w.get(at..start).unwrap_or(""));
                    out.push_str(&format!("{{{{{path}}}}}"));
                    at = end;
                    changed = true;
                    // The year typed after the month, when it is this month's year.
                    if let Some(len) = w.get(end..).and_then(|rest| year_after(rest, *year)) {
                        at = end.saturating_add(len);
                    }
                    i = i.saturating_add(1);
                }
                out.push_str(w.get(at..).unwrap_or(""));
            }
        }
    }
    changed.then_some(out)
}

/// The model's answer JSON with its kind `answer` changed to `estimate`, or `None` when it has another
/// kind or is not JSON. Used when the app itself read an estimate for the question (the Owner asked
/// for one) and the model cited its figures under the plain kind: the figures are the app's own
/// estimate, so the label is the one the panel needs for the amber band (PRD FR-7.86). The check of
/// the kind stays as it was for every answer the app did not plan an estimate for.
///
/// Example: `{"kind":"answer","text":"{{e1.same_pace.total}}"}` becomes `{"kind":"estimate", ...}`.
pub fn as_estimate(content: &str) -> Option<String> {
    let mut value: serde_json::Value = serde_json::from_str(content).ok()?;
    let kind = value.get_mut("kind")?;
    if kind.as_str() != Some("answer") {
        return None;
    }
    *kind = serde_json::Value::String("estimate".to_owned());
    Some(value.to_string())
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

    #[test]
    fn a_bare_amount_of_the_owner_is_never_put_back_as_if_the_books_had_said_it() {
        // D7 R13: "ہاں، 5000" for "kya akram ka bakaya 5000 hai" confirms his own number with no read.
        assert_eq!(
            back_digits("kya akram ka bakaya 5000 hai", "ہاں، 5000"),
            None
        );
        assert_eq!(back_digits("khata 100 hai", "ہاں 100"), None);
        assert_eq!(back_digits("3 lakh ka target", "300000"), None);
        // A count, a month and a percent are still his number.
        assert_eq!(
            back_digits("top 5 buyers", "سرفہرست 5 گاہک").as_deref(),
            Some("سرفہرست {{q.n1}} گاہک")
        );
        assert_eq!(
            back_digits("10% zyada ho to", "10% زیادہ").as_deref(),
            Some("{{q.n1}}% زیادہ")
        );
        // A number left out must not shift the place of the next one: the model's 3 is `q.n2`, never
        // `q.n1` (which would show 5000), and the same for an amount in lakh (money review 11).
        assert_eq!(
            back_digits(
                "kya akram ka bakaya 5000 hai pichle 3 mahine",
                "پچھلے 3 مہینے"
            )
            .as_deref(),
            Some("پچھلے {{q.n2}} مہینے")
        );
        assert_eq!(
            back_digits("3 lakh ka target agle 6 mahine", "اگلے 6 مہینے").as_deref(),
            Some("اگلے {{q.n2}} مہینے")
        );
    }

    #[test]
    fn the_place_of_a_number_put_back_is_the_place_in_the_questions_own_list() {
        let q = question_numbers("kya akram ka bakaya 5000 hai pichle 3 mahine");
        assert_eq!(q.len(), 2, "{q:?}");
        assert_eq!(q.first().map(|n| n.number), Some(5000));
        assert_eq!(q.get(1).map(|n| n.number), Some(3));
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

    fn september() -> Vec<(u32, i32, String)> {
        vec![(9, 2026, "r1.label".to_owned())]
    }

    #[test]
    fn a_month_typed_with_its_year_becomes_the_label_of_the_period() {
        assert_eq!(
            put_months("ستمبر 2026 میں سبزی کی بکری", &september()).as_deref(),
            Some("{{r1.label}} میں سبزی کی بکری")
        );
        // The year in Urdu digits too, and an English month name.
        assert_eq!(
            put_months("ستمبر ۲۰۲۶ میں", &september()).as_deref(),
            Some("{{r1.label}} میں")
        );
        assert_eq!(
            put_months("Sales in September 2026 were", &september()).as_deref(),
            Some("Sales in {{r1.label}} were")
        );
        // A month alone, twice, and with the year of another month: the digits stay for the check.
        assert_eq!(
            put_months("ستمبر کی بکری، ستمبر کا کمیشن", &september()).as_deref(),
            Some("{{r1.label}} کی بکری، {{r1.label}} کا کمیشن")
        );
        assert_eq!(
            put_months("ستمبر 2025 میں", &september()).as_deref(),
            Some("{{r1.label}} 2025 میں")
        );
    }

    #[test]
    fn a_month_the_results_do_not_hold_or_a_word_the_check_accepts_is_left_alone() {
        // October is not in the results; "september" in lower case is not refused by the check.
        assert_eq!(put_months("اکتوبر میں", &september()), None);
        assert_eq!(put_months("in september", &september()), None);
        assert_eq!(put_months("ستمبر میں", &[]), None);
        // The references already in the text are never touched.
        assert_eq!(put_months("{{r2.rows.i1.month}} میں", &september()), None);
    }

    #[test]
    fn a_month_held_in_two_years_has_no_one_path() {
        let held = vec![
            (9, 2025, "r1.rows.a.month".to_owned()),
            (9, 2026, "r1.rows.b.month".to_owned()),
            (10, 2026, "r2.label".to_owned()),
            (10, 2026, "r1.rows.c.month".to_owned()),
        ];
        let one = one_path_per_month(&held);
        // September is of two years: left out. October is of one: the label of the period.
        assert_eq!(one, vec![(10, 2026, "r2.label".to_owned())]);
    }

    #[test]
    fn only_a_plain_answer_becomes_an_estimate() {
        let plain = r#"{"kind":"answer","text":"{{e1.same_pace.total}}","show":[]}"#;
        let fixed = as_estimate(plain).unwrap();
        let value: serde_json::Value = serde_json::from_str(&fixed).unwrap();
        assert_eq!(value["kind"], "estimate");
        assert_eq!(value["text"], "{{e1.same_pace.total}}");
        // Another kind, no kind, or not JSON: nothing to do.
        assert_eq!(as_estimate(r#"{"kind":"estimate","text":"x"}"#), None);
        assert_eq!(as_estimate(r#"{"kind":"advice","text":"x"}"#), None);
        assert_eq!(as_estimate(r#"{"text":"x"}"#), None);
        assert_eq!(as_estimate("not json"), None);
    }
}
