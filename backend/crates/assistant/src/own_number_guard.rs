//! An answer made only of the Owner's own amount (D7 R13, money review 11).
//!
//! "kya akram ka bakaya 5000 hai" answered "ہاں، {{q.n1}}" shows his 5,000 back to him as if the books
//! had said it, with no read. A reference to a number of the question is allowed (PRD FR-7.75: a
//! threshold, a number of months), but when the answer cites NOTHING from the books, the calculator
//! or the estimator, and what it cites is an amount of the Owner's own (a number with lakh, hazar or
//! crore, or a plain number of 100 or more), it confirms his own number. The check refuses it and
//! the one rewrite has to read the books.

use mandi_services::domain::figures::{NumberUnit, QuestionNumber};

use crate::check::Failure;
use crate::model_answer::{Piece, pieces};

/// A plain number below this is a count ("top 5"), not an amount.
const COUNT_LIMIT: i64 = 100;

/// True for an amount of the Owner's own: a number with lakh, hazar or crore, or a plain number of 100
/// or more. A count, a number of months, days or years and a percent are not.
fn is_amount(n: &QuestionNumber) -> bool {
    match n.unit {
        NumberUnit::Amount => true,
        NumberUnit::Plain => n.number >= COUNT_LIMIT,
        NumberUnit::Months | NumberUnit::Days | NumberUnit::Years | NumberUnit::Percent => false,
    }
}

/// The failure for a text that cites only the Owner's own amount, or cites nothing when the question
/// holds an amount. `read` are the results of this question, offered to the rewrite to cite instead.
///
/// Example: "ہاں، {{q.n1}}" with the question number 5000 fails; so does "جی ہاں، یہی رقم ہے" (a
/// confirmation with no figure); "ہاں، {{r2.balance}} اور {{q.n1}}" does not (a book figure is
/// cited); "اگلے {{q.n1}} مہینے" with 6 months does not.
///
/// `confirm_check` is false for an answer that may hold no figure by design (general advice, or an
/// answer written after the app's own guide was read): a yes with no figure is not refused there, but
/// an answer that cites only the Owner's amount still is (money review 15, M3 and M4).
pub fn failures(
    text: &str,
    numbers: &[QuestionNumber],
    read: &[String],
    confirm_check: bool,
) -> Vec<Failure> {
    let mut amount: Option<String> = None;
    let mut cited_any = false;
    for piece in pieces(text) {
        let Piece::Ref(r) = piece else { continue };
        cited_any = true;
        let Some(place) = r
            .path
            .strip_prefix("q.n")
            .and_then(|n| n.parse::<usize>().ok())
        else {
            // A reference to a tool result, the calculator or the estimator: the books are cited.
            return Vec::new();
        };
        let own_amount = place
            .checked_sub(1)
            .and_then(|i| numbers.get(i))
            .is_some_and(is_amount);
        if own_amount && amount.is_none() {
            amount = Some(r.path.clone());
        }
    }
    // Nothing cited at all, though the question holds an amount: "جی ہاں، یہی رقم ہے" confirms the
    // Owner's number with no figure and no read.
    let confirms_without_a_figure =
        confirm_check && !cited_any && !text.trim().is_empty() && numbers.iter().any(is_amount);
    let at = amount.or_else(|| confirms_without_a_figure.then(|| "text".to_owned()));
    at.map(|at| {
        vec![Failure {
            instead: read.to_vec(),
            ..Failure::new("own_number", &at)
        }]
    })
    .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use mandi_services::domain::figures::question_numbers;

    fn failed(question: &str, text: &str) -> bool {
        !failures(text, &question_numbers(question), &[], true).is_empty()
    }

    #[test]
    fn an_answer_that_only_gives_the_owners_own_amount_back_is_refused() {
        assert!(failed("kya akram ka bakaya 5000 hai", "ہاں، {{q.n1}}"));
        assert!(failed(
            "kya akram ka bakaya 3 lakh hai",
            "ہاں، {{q.n1}} روپے"
        ));
        // The place counts: the amount is n1, the months n2.
        assert!(failed(
            "kya akram ka bakaya 5000 hai pichle 3 mahine",
            "{{q.n1}} اور {{q.n2}}"
        ));
    }

    #[test]
    fn a_book_figure_a_count_or_a_month_is_never_refused() {
        // A tool result is cited: the books were read.
        assert!(!failed(
            "kya akram ka bakaya 5000 hai",
            "{{r2.balance}} ہے، {{q.n1}} نہیں"
        ));
        // A number of months, a percent and a small count are the Owner's own and harmless.
        assert!(!failed("agle 6 mahine ka andaza", "اگلے {{q.n1}} مہینے"));
        assert!(!failed("top 5 buyers", "سرفہرست {{q.n1}} گاہک"));
        // A place the question has not got, and an empty text (a which-one or a not-found).
        assert!(!failed("kya akram ka bakaya 5000 hai", "{{q.n9}}"));
        assert!(!failed("kya akram ka bakaya 5000 hai", ""));
        // No reference at all and no amount in the question: nothing to confirm.
        assert!(!failed("akram ka bakaya", "ٹھیک ہے"));
        assert!(!failed("agle 6 mahine ka andaza", "کوئی اندازہ نہیں"));
    }

    #[test]
    fn a_yes_with_no_figure_to_an_amount_is_refused_and_the_rewrite_is_told_what_was_read() {
        let numbers = question_numbers("kya akram ka bakaya 5000 hai");
        assert!(failed("kya akram ka bakaya 5000 hai", "جی ہاں، یہی رقم ہے"));
        let read = vec!["r1".to_owned(), "r2".to_owned()];
        let f = failures("جی ہاں، یہی رقم ہے", &numbers, &read, true);
        assert_eq!(f.len(), 1);
        assert_eq!(f.first().map(|x| x.instead.clone()), Some(read.clone()));
        // Where a figure is not expected (general advice, a help answer) a bare yes passes, but the
        // Owner's own amount given back as the only reference is still refused.
        assert!(failures("جی ہاں، یہی رقم ہے", &numbers, &read, false).is_empty());
        assert!(!failures("ہاں، {{q.n1}}", &numbers, &read, false).is_empty());
    }
}
