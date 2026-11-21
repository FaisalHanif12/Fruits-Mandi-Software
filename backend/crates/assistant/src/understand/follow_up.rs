//! A follow-up that changes only the subject ("aur ehsan ki ?", "and on tea?"): plan 82, live check of
//! 21-11-2026 (D26b, Y21b, Y31b).
//!
//! After "faisal ki abhi tak ki bikri kitni hai ?" the Owner asks "aur ehsan ki ?": the same figure over the same
//! period, for another party. Left to the model, it read the new party and then cited a result of the earlier
//! question, which the app refuses (`earlier_question`), or it read "and on tea?" with no dates (all the books,
//! 115,000, not this year's 98,000).
//!
//! The app now writes the question the Owner means: the earlier question with its subject replaced by the new
//! one, "ehsan ki abhi tak ki bikri kitni hai ?", and understands that like any question. Its reads, its answer
//! and the guards are about this question's own results, with the period of the question before.
//!
//! Only a short question that names exactly one found subject of the same kind as the earlier one, and no figure
//! and no period of its own, is rewritten; the earlier one must be a sure figure that names one subject. Anything
//! else is left to the model as before.
//!
//! Example: "rahim ki is saal ki bikri kitni hai ?" then "aur karim ki ?" is "karim ki is saal ki bikri kitni
//! hai ?".

use super::scan::{DATE_WORD, Scan};
use super::{Intent, Outcome, Reader, Understanding};

/// The longest follow-up, in words.
const MOST_WORDS: usize = 6;

/// The kind of subject, and its words in the question (first, after last).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Subject {
    Party((usize, usize)),
    Item((usize, usize)),
    Record((usize, usize)),
}

impl Subject {
    fn same_kind(self, other: Subject) -> bool {
        std::mem::discriminant(&self) == std::mem::discriminant(&other)
    }

    fn span(self) -> (usize, usize) {
        match self {
            Subject::Party(s) | Subject::Item(s) | Subject::Record(s) => s,
        }
    }
}

/// The one subject an understanding names, found in the books, or `None` for none or several.
fn subject_of(u: &Understanding) -> Option<Subject> {
    let mut found: Vec<Subject> = Vec::new();
    for p in &u.parties {
        match &p.outcome {
            Outcome::Found(_) | Outcome::FoundAs(_) => found.push(Subject::Party(p.words)),
            _ => return None,
        }
    }
    for i in &u.items {
        match &i.outcome {
            Outcome::Found(_) | Outcome::FoundAs(_) => found.push(Subject::Item(i.words)),
            _ => return None,
        }
    }
    for r in &u.records {
        match &r.outcome {
            Outcome::Found(_) | Outcome::FoundAs(_) => found.push(Subject::Record(r.words)),
            _ => return None,
        }
    }
    match found.as_slice() {
        [one] => Some(*one),
        _ => None,
    }
}

/// The earlier question with the subject of the follow-up put in its place, or `None` when the follow-up is
/// not that kind of question.
#[must_use]
pub fn follow_up_text(
    reader: &Reader<'_>,
    prev_text: &str,
    prev: &Understanding,
    text: &str,
    new: &Understanding,
) -> Option<String> {
    if !prev.sure
        || prev.intent.value != Intent::Figure
        || prev.figures.value.len() != 1
        || prev.compare
        || prev.list.is_some()
        || !new.figures.value.is_empty()
        || !new.periods.is_empty()
        || new.list.is_some()
        || new.topic.is_some()
        || new.report.is_some()
        || new.outside.is_some()
        || new.unexplained
        || new.compare
        || !matches!(new.intent.value, Intent::Unclear | Intent::Figure)
    {
        return None;
    }
    let today = reader.today_date()?;
    let old_scan = Scan::new(prev_text, today);
    let new_scan = Scan::new(text, today);
    if new_scan.words.len() > MOST_WORDS || new_scan.cut || old_scan.cut {
        return None;
    }
    // A date written in digits is a marker word in a scan: it cannot be written back.
    if old_scan.words.iter().any(|w| w.starts_with(DATE_WORD)) {
        return None;
    }
    let (before, now) = (subject_of(prev)?, subject_of(new)?);
    if !before.same_kind(now) {
        return None;
    }
    let (old_from, old_to) = before.span();
    let (new_from, new_to) = now.span();
    let name = new_scan.words.get(new_from..new_to)?;
    let head = old_scan.words.get(..old_from)?;
    let tail = old_scan.words.get(old_to..)?;
    let words: Vec<&str> = head
        .iter()
        .chain(name)
        .chain(tail)
        .map(String::as_str)
        .collect();
    Some(words.join(" "))
}
