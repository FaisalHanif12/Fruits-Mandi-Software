//! A question cut into normalised words, with the cues, the numbers and the dates it holds.
//!
//! Normalising is the same as the search keys use (`core::text`): lower case, one Urdu letter for
//! each look-alike, no diacritics. Digits of every script are plain digits. A date written in
//! digits ("12-10-2026", "12/10") becomes one word that holds the date. PRD FR-7.40, NFR-URD-18.

use chrono::{Datelike, NaiveDate};
use mandi_services::core::text;
use mandi_services::domain::figures::{QuestionNumber, question_numbers};

use super::lexicon::{Cue, Hit, table};
use super::when;

/// The most words of a question that are read. A longer question is cut here and flagged.
pub(super) const MAX_WORDS: usize = 80;
/// The most characters of a question that are read.
pub(super) const MAX_CHARS: usize = 600;

/// The prefix of the word that stands for a date written in digits.
pub(super) const DATE_WORD: &str = "zzdate";

/// "یکم" and its Roman spellings: the first day of a month.
pub(super) fn is_first_day_word(word: &str) -> bool {
    matches!(word, "یکم" | "yakum" | "yekum" | "ekam")
}

/// A date written in digits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Literal {
    pub date: NaiveDate,
    /// True when the year was written.
    pub had_year: bool,
}

/// The words of a question and what they say.
#[derive(Debug, Clone)]
pub(super) struct Scan {
    /// The normalised words in order. A date written in digits is `zzdate<n>`.
    pub words: Vec<String>,
    /// The dates written in digits, by `n`.
    pub dates: Vec<Literal>,
    /// Every cue found.
    pub hits: Vec<Hit>,
    /// The numbers the Owner typed.
    pub numbers: Vec<QuestionNumber>,
    /// True when the question was cut for length.
    pub cut: bool,
    /// The question as typed, trimmed.
    pub raw: String,
    /// A truck number the question holds, written `LES-1234`.
    pub truck: Option<String>,
}

fn ascii_digit(c: char) -> char {
    match c {
        '\u{0660}'..='\u{0669}' => {
            char::from_digit(u32::from(c).saturating_sub(0x0660), 10).unwrap_or(c)
        }
        '\u{06F0}'..='\u{06F9}' => {
            char::from_digit(u32::from(c).saturating_sub(0x06F0), 10).unwrap_or(c)
        }
        other => other,
    }
}

/// Takes the dates written in digits out of the text and leaves a marker word for each.
fn take_dates(text: &str, today: NaiveDate) -> (String, Vec<Literal>) {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let mut dates: Vec<Literal> = Vec::new();
    let mut at = 0usize;
    while let Some(&c) = chars.get(at) {
        if !c.is_ascii_digit() {
            out.push(c);
            at = at.saturating_add(1);
            continue;
        }
        // Read groups of digits joined by one separator.
        let mut groups: Vec<String> = Vec::new();
        let mut seps: Vec<char> = Vec::new();
        let mut cursor = at;
        loop {
            let start = cursor;
            while chars.get(cursor).is_some_and(char::is_ascii_digit) {
                cursor = cursor.saturating_add(1);
            }
            groups.push(
                chars
                    .get(start..cursor)
                    .map(|s| s.iter().collect())
                    .unwrap_or_default(),
            );
            let sep = chars.get(cursor).copied();
            let next_digit = chars
                .get(cursor.saturating_add(1))
                .is_some_and(char::is_ascii_digit);
            match sep {
                Some(s @ ('-' | '/' | '.')) if next_digit && groups.len() < 3 => {
                    seps.push(s);
                    cursor = cursor.saturating_add(1);
                }
                _ => break,
            }
        }
        match literal_of(&groups, &seps, today) {
            Some(literal) => {
                out.push(' ');
                out.push_str(DATE_WORD);
                out.push_str(&dates.len().to_string());
                out.push(' ');
                dates.push(literal);
            }
            None => {
                out.extend(chars.get(at..cursor).unwrap_or_default());
            }
        }
        at = cursor;
    }
    (out, dates)
}

/// A date from groups of digits: `12-10-2026`, `2026-10-12`, `12/10/26`, or `12-10` (this year, or
/// last year when that day has not come).
fn literal_of(groups: &[String], seps: &[char], today: NaiveDate) -> Option<Literal> {
    let number = |s: &String| s.parse::<u32>().ok();
    match groups {
        [a, b, c] if seps.iter().all(|s| matches!(s, '-' | '/' | '.')) => {
            let (a, b, c) = (number(a)?, number(b)?, number(c)?);
            let len_c = groups.get(2)?.len();
            let len_a = groups.first()?.len();
            let (y, m, d) = if len_a == 4 {
                (i32::try_from(a).ok()?, b, c)
            } else if len_c == 4 {
                (i32::try_from(c).ok()?, b, a)
            } else if len_c == 2 {
                (i32::try_from(c.saturating_add(2000)).ok()?, b, a)
            } else {
                return None;
            };
            Some(Literal {
                date: NaiveDate::from_ymd_opt(y, m, d)?,
                had_year: true,
            })
        }
        [a, b] if seps.iter().all(|s| matches!(s, '-' | '/')) => {
            let (d, m) = (number(a)?, number(b)?);
            if groups.first()?.len() > 2 || groups.get(1)?.len() > 2 {
                return None;
            }
            let mut date = NaiveDate::from_ymd_opt(today.year(), m, d)?;
            if date > today {
                date = NaiveDate::from_ymd_opt(today.year().checked_sub(1)?, m, d)?;
            }
            Some(Literal {
                date,
                had_year: false,
            })
        }
        _ => None,
    }
}

/// Makes "kitnaaaa" into "kitnaa": a letter repeated three times or more is written twice.
fn squeeze(word: &str) -> String {
    if !word.chars().all(|c| c.is_ascii_alphabetic()) {
        return word.to_owned();
    }
    let mut out = String::with_capacity(word.len());
    let mut run = 0usize;
    let mut last: Option<char> = None;
    for c in word.chars() {
        if last == Some(c) {
            run = run.saturating_add(1);
        } else {
            run = 1;
            last = Some(c);
        }
        if run <= 2 {
            out.push(c);
        }
    }
    out
}

/// "15th", "1st", "2nd" are the number.
fn without_ordinal(word: &str) -> String {
    let digits: String = word.chars().take_while(char::is_ascii_digit).collect();
    let rest = word.get(digits.len()..).unwrap_or_default();
    if !digits.is_empty() && matches!(rest, "st" | "nd" | "rd" | "th") {
        digits
    } else {
        word.to_owned()
    }
}

/// A truck number in the words: two or three letters and three or four digits, glued ("lhr0077"),
/// joined by a hyphen ("LES-1234") or, with the letters in capitals, by a space ("LES 1234").
/// Returns the first and the last-plus-one word and the number written `LES-1234`.
///
/// Example: "LES-1234 ki bikri" gives words 0 to 2 and `LES-1234`; "ali 1000 ki bikri" gives none
/// (a name and an amount).
fn truck_number(raw: &str, words: &[String]) -> Option<(usize, usize, String)> {
    let letters_then_digits = |w: &str| -> Option<(String, String)> {
        let at = w.find(|c: char| c.is_ascii_digit())?;
        let (l, d) = w.split_at(at);
        ((2..=3).contains(&l.chars().count())
            && l.chars().all(|c| c.is_ascii_lowercase())
            && (3..=4).contains(&d.chars().count())
            && d.chars().all(|c| c.is_ascii_digit()))
        .then(|| (l.to_owned(), d.to_owned()))
    };
    let lower = raw.to_ascii_lowercase();
    for (at, word) in words.iter().enumerate() {
        if let Some((l, d)) = letters_then_digits(word) {
            return Some((
                at,
                at.saturating_add(1),
                format!("{}-{d}", l.to_uppercase()),
            ));
        }
        let next = words.get(at.saturating_add(1))?;
        let letters = (2..=3).contains(&word.chars().count())
            && word.chars().all(|c| c.is_ascii_lowercase())
            && !table().is_known(word);
        let digits =
            (3..=4).contains(&next.chars().count()) && next.chars().all(|c| c.is_ascii_digit());
        if letters && digits {
            let hyphen = lower.contains(&format!("{word}-{next}"));
            let capitals = raw.contains(&format!("{} {next}", word.to_ascii_uppercase()));
            if hyphen || capitals {
                return Some((
                    at,
                    at.saturating_add(2),
                    format!("{}-{next}", word.to_uppercase()),
                ));
            }
        }
    }
    None
}

impl Scan {
    /// Reads a question. `today` is needed only to place a date written without a year.
    pub(super) fn new(question: &str, today: NaiveDate) -> Self {
        let raw: String = question.trim().chars().take(MAX_CHARS).collect();
        let cut = question.trim().chars().count() > MAX_CHARS;
        let digits: String = raw.chars().map(ascii_digit).collect();
        let (with_markers, dates) = take_dates(&digits, today);
        let normal = text::search_ur(&text::search_en(&with_markers));
        let mut words: Vec<String> = normal
            .split(|c: char| !c.is_alphanumeric())
            .filter(|w| !w.is_empty())
            .map(|w| without_ordinal(&squeeze(w)))
            .collect();
        let cut = cut || words.len() > MAX_WORDS;
        words.truncate(MAX_WORDS);
        let mut hits = table().hits(&words);
        // A word of time that is close to a word of the table is still a word of time (tarikh is
        // not tarika).
        hits.retain(|h| !(h.fuzzy && words.get(h.start).is_some_and(|w| when::is_time_word(w))));
        let numbers = question_numbers(&digits);
        // A truck number is no name and no amount: the words it covers are known (a hit of its own).
        let truck = truck_number(&raw, &words);
        if let Some((start, end, _)) = &truck {
            hits.push(Hit {
                cue: Cue::TruckNo,
                start: *start,
                end: *end,
                fuzzy: false,
            });
        }
        Self {
            words,
            dates,
            hits,
            numbers,
            cut,
            truck: truck.map(|(_, _, text)| text),
            raw,
        }
    }

    /// True when the question has the cue, exact or soft.
    pub(super) fn has(&self, cue: Cue) -> bool {
        self.hits.iter().any(|h| h.cue == cue)
    }

    /// True when the question has the cue as the table's own word or phrase.
    pub(super) fn has_exact(&self, cue: Cue) -> bool {
        self.hits.iter().any(|h| h.cue == cue && !h.fuzzy)
    }

    /// True when it has any of the cues.
    pub(super) fn has_any(&self, cues: &[Cue]) -> bool {
        cues.iter().any(|c| self.has(*c))
    }

    /// The words from `start` to `end`, joined by a space.
    pub(super) fn phrase(&self, start: usize, end: usize) -> String {
        self.words
            .get(start..end)
            .map(|w| w.join(" "))
            .unwrap_or_default()
    }

    /// The word at a place.
    pub(super) fn word(&self, at: usize) -> Option<&str> {
        self.words.get(at).map(String::as_str)
    }

    /// The number a word is, if it is plain digits.
    pub(super) fn digits_at(&self, at: usize) -> Option<i64> {
        let word = self.words.get(at)?;
        // The Urdu word for the first day of a month is the number 1 wherever a day is read ("یکم نومبر",
        // "یکم سے 15 تک"): a date word is a date, never the name of a party (D7, plan 59 F3b, guard 3).
        if is_first_day_word(word) {
            return Some(1);
        }
        // "pehli tarikh" ("the first date") and "from the first till today": the first day of a month.
        if matches!(word.as_str(), "pehli" | "pehla" | "پہلی" | "پہلا" | "first") {
            let next = self.words.get(at.saturating_add(1)).map(String::as_str);
            let before = at
                .checked_sub(1)
                .and_then(|b| self.words.get(b))
                .map(String::as_str);
            let before2 = at
                .checked_sub(2)
                .and_then(|b| self.words.get(b))
                .map(String::as_str);
            let after_from = matches!(before, Some("from" | "since"))
                || (before == Some("the") && matches!(before2, Some("from" | "since")));
            if matches!(next, Some("tarikh" | "tareekh" | "تاریخ" | "date")) || after_from {
                return Some(1);
            }
        }
        if word.is_empty() || !word.chars().all(|c| c.is_ascii_digit()) || word.len() > 9 {
            return None;
        }
        word.parse().ok()
    }

    /// The date literal a word is, if it is one.
    pub(super) fn date_at(&self, at: usize) -> Option<Literal> {
        let word = self.words.get(at)?;
        let n: usize = word.strip_prefix(DATE_WORD)?.parse().ok()?;
        self.dates.get(n).copied()
    }

    /// The question with the cues inside the given word spans left out (a name that holds a cue
    /// word: "Imran Fruit", "Akram Toll Tax").
    pub(super) fn without_spans(&self, spans: &[(usize, usize)]) -> Self {
        let mut copy = self.clone();
        // A place word stays: "lahore mein kele ka rate" is about a market outside the books even when
        // the shop has a city or an اڈا of that name.
        copy.hits.retain(|h| {
            h.cue == Cue::ExternalPlace || !spans.iter().any(|(s, e)| *s <= h.start && h.end <= *e)
        });
        copy
    }

    /// True when the word at a place can be a part of a name: not a number, a date, a function
    /// word, a time word or a word the table knows (except the usual parts of a firm's name).
    pub(super) fn can_start_name(&self, at: usize) -> bool {
        let Some(word) = self.words.get(at) else {
            return false;
        };
        let t = table();
        if word.is_empty()
            || word.starts_with(DATE_WORD)
            || word.chars().any(|c| c.is_ascii_digit())
            || t.is_stop(word)
            || is_first_day_word(word)
            || when::is_time_word(word)
            // "jama" beside the trial balance is its credit side, not a name.
            || (super::reads::is_trial_side_word(word)
                && self.hits.iter().any(|h| h.cue == Cue::TrialBalance))
        {
            return false;
        }
        // Inside a phrase of the table ("baqi maal", "kul raqam").
        let in_phrase = self
            .hits
            .iter()
            .any(|h| !h.fuzzy && h.end.saturating_sub(h.start) > 1 && h.start <= at && at < h.end);
        if in_phrase && !t.is_nameish(word) {
            return false;
        }
        !t.is_known(word) || t.is_nameish(word)
    }
}

/// Damerau-Levenshtein distance (a swap of two letters is one change), counted up to `cap`. A larger
/// distance is returned as `cap + 1`.
pub(super) fn edit_distance(a: &str, b: &str, cap: usize) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let over = cap.saturating_add(1);
    if a.len().abs_diff(b.len()) > cap {
        return over;
    }
    let width = b.len().saturating_add(1);
    let mut before: Vec<usize> = vec![0; width];
    let mut prev: Vec<usize> = (0..width).collect();
    for (i, ca) in a.iter().enumerate() {
        let mut cur: Vec<usize> = vec![0; width];
        if let Some(first) = cur.first_mut() {
            *first = i.saturating_add(1);
        }
        for (j, cb) in b.iter().enumerate() {
            let next = j.saturating_add(1);
            let cost = usize::from(ca != cb);
            let up = prev.get(next).copied().unwrap_or(over).saturating_add(1);
            let left = cur.get(j).copied().unwrap_or(over).saturating_add(1);
            let diag = prev.get(j).copied().unwrap_or(over).saturating_add(cost);
            let mut best = up.min(left).min(diag);
            if let (Some(i0), Some(j0)) = (i.checked_sub(1), j.checked_sub(1))
                && a.get(i0) == Some(cb)
                && b.get(j0) == Some(ca)
            {
                let swap = before.get(j0).copied().unwrap_or(over).saturating_add(1);
                best = best.min(swap);
            }
            if let Some(slot) = cur.get_mut(next) {
                *slot = best;
            }
        }
        before = prev;
        prev = cur;
    }
    prev.last().copied().unwrap_or(over).min(over)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn today() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 11, 4).unwrap()
    }

    #[test]
    fn the_distance_counts_a_swap_as_one() {
        assert_eq!(edit_distance("commission", "commision", 2), 1);
        assert_eq!(edit_distance("bakaya", "bakyaa", 2), 1);
        assert_eq!(edit_distance("akram", "akarm", 2), 1);
        assert_eq!(edit_distance("abc", "xyz", 1), 2);
        assert_eq!(edit_distance("same", "same", 1), 0);
        assert_eq!(edit_distance("short", "muchlongerword", 2), 3);
    }

    #[test]
    fn digits_of_every_script_and_dates_in_digits_are_read() {
        let s = Scan::new("۱۲-۱۰-۲۰۲۶ ka print", today());
        assert_eq!(s.dates.len(), 1);
        assert_eq!(
            s.dates.first().map(|l| l.date),
            NaiveDate::from_ymd_opt(2026, 10, 12)
        );
        assert!(s.words.iter().any(|w| w.starts_with("zzdate")));
        let s = Scan::new("15/10 ki bikri", today());
        assert_eq!(
            s.dates.first().map(|l| (l.date, l.had_year)),
            Some((NaiveDate::from_ymd_opt(2026, 10, 15).unwrap(), false))
        );
        // A day that has not come yet this year is last year's.
        let s = Scan::new("25-12", today());
        assert_eq!(
            s.dates.first().map(|l| l.date),
            NaiveDate::from_ymd_opt(2025, 12, 25)
        );
        // Not dates: an amount with commas, a phone, a bare range.
        assert!(Scan::new("10,000 wasooli", today()).dates.is_empty());
        assert!(Scan::new("1-15 tak", today()).dates.is_empty());
    }

    #[test]
    fn repeated_letters_and_ordinals_are_squeezed() {
        let s = Scan::new("kitnaaaa 15th", today());
        assert_eq!(s.words, ["kitnaa", "15"]);
    }
}
