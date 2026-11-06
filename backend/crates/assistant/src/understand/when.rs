//! The period a question names, as exact dates (plan 54 Section 6.2, plan 55 step 1).
//!
//! Words: aaj, kal (yesterday in a past-tense question, tomorrow in a future one), parson, is hafte,
//! pichle hafte, is mahine, pichle mahine, a month name in three scripts, "1 se 15 tak", "12-10-2026",
//! is saal, pichle saal, pichle 3 mahine, ab tak. The dates come from today's Pakistan date. Nothing
//! is guessed: a word that is not a period gives no period. کل / kal is a day, never "total".
//!
//! PRD FR-19.2 (the date shortcuts of the reports), FR-7.77 (the period tool of the assistant) and
//! FR-6.6 (no posting date after today: a day that has not come yet is last year's or last month's).
//!
//! Example (today 04-11-2026): "pichle mahine" is 01-10-2026 to 31-10-2026; "1 se 15 tak" is
//! 01-10-2026 to 15-10-2026, because 15-11-2026 has not come yet.

use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

use chrono::{Datelike, Duration, Months, NaiveDate};

use super::lexicon::key;
use super::scan::Scan;
use super::{Confidence, PeriodKind, PeriodRef};
use crate::tools::cues;

fn set(words: &[&str]) -> HashSet<String> {
    words.iter().map(|w| key(w)).collect()
}

struct Words {
    this: HashSet<String>,
    last: HashSet<String>,
    next: HashSet<String>,
    week: HashSet<String>,
    month: HashSet<String>,
    year: HashSet<String>,
    day: HashSet<String>,
    today: HashSet<String>,
    kal: HashSet<String>,
    parson: HashSet<String>,
    yesterday: HashSet<String>,
    tomorrow: HashSet<String>,
    from: HashSet<String>,
    until: HashSet<String>,
    tarikh: HashSet<String>,
    past: HashSet<String>,
    future: HashSet<String>,
    may_before: HashSet<String>,
    may_after: HashSet<String>,
    months: HashMap<String, u32>,
    numbers: HashMap<String, i64>,
}

#[rustfmt::skip]
fn words() -> &'static Words {
    static WORDS: OnceLock<Words> = OnceLock::new();
    WORDS.get_or_init(|| {
        let month_names: [(u32, &[&str]); 12] = [
            (1, &["january", "jan", "janwari", "janvari", "جنوری"]),
            (2, &["february", "feb", "farwari", "farvari", "فروری"]),
            (3, &["march", "مارچ"]),
            (4, &["april", "apr", "aprail", "اپریل"]),
            (5, &["مئی", "mei"]),
            (6, &["june", "jun", "جون"]),
            (7, &["july", "jul", "julai", "جولائی"]),
            (8, &["august", "aug", "agast", "اگست"]),
            (
                9,
                &["september", "sep", "sept", "sitambar", "sitember", "ستمبر"],
            ),
            (
                10,
                &["october", "oct", "aktubar", "aktoobar", "october", "اکتوبر"],
            ),
            (11, &["november", "nov", "navambar", "november", "نومبر"]),
            (12, &["december", "dec", "disambar", "disember", "دسمبر"]),
        ];
        let mut months = HashMap::new();
        for (number, names) in month_names {
            for name in names {
                months.insert(key(name), number);
            }
        }
        let number_words: [(i64, &[&str]); 12] = [
            (1, &["ek", "one", "ایک"]),
            (2, &["do", "two", "دو"]),
            (3, &["teen", "three", "تین"]),
            (4, &["char", "chaar", "four", "چار"]),
            (5, &["panch", "paanch", "five", "پانچ"]),
            (6, &["che", "chay", "chhe", "six", "چھ"]),
            (7, &["saat", "seven", "سات"]),
            (8, &["aath", "eight", "آٹھ"]),
            (9, &["nau", "nine", "نو"]),
            (10, &["das", "ten", "دس"]),
            (11, &["gyarah", "eleven", "گیارہ"]),
            (12, &["barah", "twelve", "بارہ"]),
        ];
        let mut numbers = HashMap::new();
        for (value, names) in number_words {
            for name in names {
                numbers.insert(key(name), value);
            }
        }
        Words {
            this: set(&["is", "ye", "yeh", "this", "اس", "یہ"]),
            last: set(&[
                "pichle", "pichla", "pichli", "pichlay", "pichly", "pichlee", "pichhle", "pichhla",
                "last", "previous", "guzishta", "guzashta", "gaye", "gaya", "gayi", "گزشتہ",
                "پچھلے", "پچھلا", "پچھلی", "گئے",
            ]),
            next: set(&[
                "agle", "agla", "agli", "aglay", "next", "aane", "aanay", "اگلے", "اگلا", "اگلی",
                "آنے",
            ]),
            week: set(&[
                "hafta", "hafte", "haftay", "haftah", "week", "weeks", "ہفتہ", "ہفتے",
            ]),
            month: set(&[
                "mahina", "mahine", "mahinay", "mahiny", "mahiney", "mahinah", "mahino", "mahinon",
                "month", "months", "مہینہ", "مہینے", "مہینوں", "ماہ",
            ]),
            year: set(&["saal", "sal", "saalon", "year", "years", "سال", "سالوں"]),
            day: set(&["din", "dino", "day", "days", "دن", "دنوں"]),
            today: set(&["aaj", "aj", "today", "آج"]),
            kal: set(&["kal", "کل"]),
            parson: set(&["parson", "parsoon", "پرسوں", "narson"]),
            yesterday: set(&["yesterday"]),
            tomorrow: set(&["tomorrow"]),
            from: set(&["se", "say", "to", "till", "until", "from", "سے", "تا"]),
            until: set(&["tak", "tk", "till", "تک"]),
            tarikh: set(&["tarikh", "tareekh", "date", "تاریخ"]),
            past: set(&[
                "kiya", "kia", "kiye", "hua", "hui", "hue", "huwa", "huay", "tha", "thi", "the",
                "thay", "becha", "beche", "bechi", "bika", "bike", "biki", "khareeda", "khareede",
                "khareedi", "kharida", "aaya", "aayi", "aya", "aai", "gaya", "gayi", "gaye", "diya",
                "dia", "liya", "lia", "mila", "mili", "mile", "bought", "sold", "did", "was",
                "were", "had", "paid", "got", "beecha", "beeche", "ہوا", "ہوئی", "ہوئے", "تھا",
                "تھی", "تھے", "بیچا", "بیچے", "بیچی", "بکا", "بکے", "خریدا", "خریدے", "آیا", "آئی",
                "گیا", "گئی", "گئے", "دیا", "لیا", "ملا", "ملی", "کیا", "کیے",
            ]),
            future: set(&[
                "hoga", "hogi", "honge", "hongi", "karunga", "karungi", "karenge", "karengi",
                "aayega", "aayegi", "ayega", "ayegi", "aaega", "jayega", "jayegi", "jaega",
                "milega", "milegi", "bikega", "bikegi", "will", "shall", "going", "tomorrow",
                "hona", "bhejna", "dunga", "dungi", "lunga", "lungi", "ہوگا", "ہوگی", "ہوں گے",
                "کروں گا", "آئے گا", "آئے گی", "جائے گا", "ملے گا", "بکے گا",
            ]),
            may_before: set(&[
                "in", "of", "for", "during", "since", "from", "is", "ye", "this",
            ]),
            may_after: set(&["ka", "ki", "ke", "mein", "me", "ko", "tak", "se", "say"]),
            months,
            numbers,
        }
    })
}

/// True when a word is a month, a unit of time or a day word written wrong by a letter ("septmber",
/// "mahine" for "mahina"): a time word spelt wrong, never a person.
pub(super) fn is_near_time_word(word: &str) -> bool {
    near_month(word).is_some() || {
        let w = words();
        let length = word.chars().count();
        length >= 5
            && word.chars().all(|c| c.is_ascii_lowercase())
            && [&w.week, &w.month, &w.year, &w.day, &w.today, &w.parson]
                .iter()
                .any(|set| {
                    set.iter().any(|candidate| {
                        candidate.chars().count().abs_diff(length) <= 1
                            && super::scan::edit_distance(word, candidate, 1) <= 1
                    })
                })
    }
}

/// The month a word is one letter off ("septmber" is September), for words of five letters or more.
fn near_month(word: &str) -> Option<u32> {
    let length = word.chars().count();
    if length < 5 || !word.chars().all(|c| c.is_ascii_lowercase()) {
        return None;
    }
    // A word of an item ("mirch", a chilli) is no spelling of a month ("march").
    if super::items::is_item_word(word) {
        return None;
    }
    let mut found: Option<u32> = None;
    for (name, number) in &words().months {
        if name.chars().count() < 5 || !name.chars().all(|c| c.is_ascii_lowercase()) {
            continue;
        }
        if name.chars().count().abs_diff(length) <= 1
            && super::scan::edit_distance(word, name, 1) <= 1
        {
            if found.is_some_and(|f| f != *number) {
                return None;
            }
            found = Some(*number);
        }
    }
    found
}

/// True for a word that names a time and never a person or a thing.
pub(super) fn is_time_word(word: &str) -> bool {
    let w = words();
    w.today.contains(word)
        || w.kal.contains(word)
        || w.parson.contains(word)
        || w.yesterday.contains(word)
        || w.tomorrow.contains(word)
        || w.week.contains(word)
        || w.month.contains(word)
        || w.year.contains(word)
        || w.day.contains(word)
        || w.last.contains(word)
        || w.next.contains(word)
        || w.tarikh.contains(word)
        || w.months.contains_key(word)
}

/// How the question speaks of time: past, future, both or neither (decides what کل / kal means).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tense {
    Past,
    Future,
    Mixed,
    None,
}

fn tense_of(scan: &Scan) -> Tense {
    let w = words();
    let past = scan.words.iter().any(|x| w.past.contains(x));
    let future =
        scan.words.iter().any(|x| w.future.contains(x)) || scan.has(super::lexicon::Cue::Forecast);
    match (past, future) {
        (true, false) => Tense::Past,
        (false, true) => Tense::Future,
        (true, true) => Tense::Mixed,
        (false, false) => Tense::None,
    }
}

// ---- date arithmetic (checked) ---------------------------------------------------------------------

fn first_of_month(date: NaiveDate) -> Option<NaiveDate> {
    date.with_day(1)
}

fn last_of_month(year: i32, month: u32) -> Option<NaiveDate> {
    let first = NaiveDate::from_ymd_opt(year, month, 1)?;
    first.checked_add_months(Months::new(1))?.pred_opt()
}

fn monday_of(date: NaiveDate) -> Option<NaiveDate> {
    let back = i64::from(date.weekday().num_days_from_monday());
    date.checked_sub_signed(Duration::days(back))
}

fn period(
    kind: PeriodKind,
    from: Option<NaiveDate>,
    to: NaiveDate,
    confidence: Confidence,
    words: String,
    today: NaiveDate,
) -> PeriodRef {
    PeriodRef {
        kind,
        from,
        to,
        confidence,
        future: to > today || from.is_some_and(|f| f > today),
        words,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Unit {
    Week,
    Month,
    Year,
    Day,
}

fn unit_at(scan: &Scan, at: usize) -> Option<Unit> {
    let w = words();
    let word = scan.word(at)?;
    if w.week.contains(word) {
        Some(Unit::Week)
    } else if w.month.contains(word) {
        Some(Unit::Month)
    } else if w.year.contains(word) {
        Some(Unit::Year)
    } else if w.day.contains(word) {
        Some(Unit::Day)
    } else {
        None
    }
}

/// A number written in digits or in words, at a place.
fn count_at(scan: &Scan, at: usize) -> Option<i64> {
    if let Some(n) = scan.digits_at(at) {
        return Some(n);
    }
    words().numbers.get(scan.word(at)?).copied()
}

fn next_index(at: usize, taken: usize) -> usize {
    at.saturating_add(taken)
}

// ---- days written with a month ---------------------------------------------------------------------

/// A day of the month, with the month and the year when they were written.
#[derive(Debug, Clone, Copy)]
struct DaySpec {
    day: u32,
    month: Option<u32>,
    year: Option<i32>,
    /// A full date written in digits.
    literal: Option<NaiveDate>,
    end: usize,
}

fn year_at(scan: &Scan, at: usize) -> Option<i32> {
    let n = scan.digits_at(at)?;
    (2000..=2100)
        .contains(&n)
        .then(|| i32::try_from(n).ok())
        .flatten()
}

fn month_at(scan: &Scan, at: usize) -> Option<u32> {
    let word = scan.word(at)?;
    let w = words();
    let month = w.months.get(word).copied().or_else(|| near_month(word))?;
    if month == 5 {
        return None;
    }
    Some(month)
}

/// "may" is a month only beside the words that go with a month (in may, may ka, may 2026).
fn may_at(scan: &Scan, at: usize) -> bool {
    let w = words();
    if scan.word(at) != Some("may") {
        return false;
    }
    let before = at
        .checked_sub(1)
        .and_then(|b| scan.word(b))
        .is_some_and(|x| w.may_before.contains(x));
    let after = scan
        .word(next_index(at, 1))
        .is_some_and(|x| w.may_after.contains(x))
        || year_at(scan, next_index(at, 1)).is_some();
    before || after
}

fn any_month_at(scan: &Scan, at: usize) -> Option<u32> {
    month_at(scan, at).or_else(|| may_at(scan, at).then_some(5))
}

/// "12 october", "12 oct 2026", "october 12", a date in digits, or (when `plain` is true) a bare
/// day number.
fn day_spec(scan: &Scan, at: usize, plain: bool) -> Option<DaySpec> {
    if let Some(literal) = scan.date_at(at) {
        return Some(DaySpec {
            day: literal.date.day(),
            month: Some(literal.date.month()),
            year: literal.had_year.then(|| literal.date.year()),
            literal: Some(literal.date),
            end: next_index(at, 1),
        });
    }
    if let Some(n) = scan.digits_at(at)
        && (1..=31).contains(&n)
        && let Ok(day) = u32::try_from(n)
    {
        let after = next_index(at, 1);
        if let Some(month) = any_month_at(scan, after) {
            let year = year_at(scan, next_index(after, 1));
            return Some(DaySpec {
                day,
                month: Some(month),
                year,
                literal: None,
                end: next_index(after, if year.is_some() { 2 } else { 1 }),
            });
        }
        if plain {
            return Some(DaySpec {
                day,
                month: None,
                year: None,
                literal: None,
                end: after,
            });
        }
        return None;
    }
    if let Some(month) = any_month_at(scan, at) {
        let after = next_index(at, 1);
        if let Some(n) = scan.digits_at(after)
            && (1..=31).contains(&n)
            && let Ok(day) = u32::try_from(n)
        {
            return Some(DaySpec {
                day,
                month: Some(month),
                year: year_at(scan, next_index(after, 1)),
                literal: None,
                end: next_index(after, 1),
            });
        }
    }
    None
}

/// The date of a spec: this year, or last year when that day has not come yet.
fn date_of(
    spec: &DaySpec,
    today: NaiveDate,
    month_default: Option<(i32, u32)>,
) -> Option<NaiveDate> {
    if let Some(date) = spec.literal {
        return Some(date);
    }
    match (spec.month, spec.year) {
        (Some(month), Some(year)) => NaiveDate::from_ymd_opt(year, month, spec.day),
        (Some(month), None) => {
            let date = NaiveDate::from_ymd_opt(today.year(), month, spec.day)?;
            if date > today {
                NaiveDate::from_ymd_opt(today.year().checked_sub(1)?, month, spec.day)
            } else {
                Some(date)
            }
        }
        (None, _) => {
            let (year, month) = month_default?;
            NaiveDate::from_ymd_opt(year, month, spec.day)
        }
    }
}

fn typed(scan: &Scan, from: usize, to: usize) -> String {
    scan.phrase(from, to)
}

/// "1 se 15 tak", "1 october se 15 october tak", "12-10-2026 se 15-10-2026 tak", "from 1 to 15 october".
fn range_at(scan: &Scan, at: usize, today: NaiveDate) -> Option<(PeriodRef, usize)> {
    let w = words();
    let first = day_spec(scan, at, true)?;
    let connector = scan.word(first.end)?;
    if !w.from.contains(connector) {
        return None;
    }
    let second = day_spec(scan, next_index(first.end, 1), true)?;
    let ends_with_until = scan.word(second.end).is_some_and(|x| w.until.contains(x));
    let has_month = first.month.is_some() || second.month.is_some();
    let has_tarikh = scan.word(second.end).is_some_and(|x| w.tarikh.contains(x));
    // Two bare numbers ("20 se 25") are a quantity, not days, unless "tak" or "tarikh" follows.
    if !(has_month || ends_with_until || has_tarikh) {
        return None;
    }
    let mut end = second.end;
    if ends_with_until || has_tarikh {
        end = next_index(end, 1);
    }
    let mut straddles = false;
    let (year, month) = match (second.month.or(first.month), second.year.or(first.year)) {
        (Some(m), Some(y)) => (y, m),
        (Some(m), None) => {
            let probe = DaySpec {
                month: Some(m),
                year: None,
                ..second
            };
            let date = date_of(&probe, today, None)?;
            // "1 se 15 november" on 4 November: its first day has come and its last has not. It
            // looks ahead, so it is not last year's range; the app is not sure of it (money review
            // R-h).
            let first_day =
                NaiveDate::from_ymd_opt(today.year(), first.month.unwrap_or(m), first.day);
            let last_day = NaiveDate::from_ymd_opt(today.year(), m, second.day);
            match (first_day, last_day) {
                (Some(f), Some(l))
                    if first.literal.is_none()
                        && second.literal.is_none()
                        && f <= today
                        && l > today =>
                {
                    straddles = true;
                    (today.year(), m)
                }
                _ => (date.year(), m),
            }
        }
        (None, _) => {
            // Bare days: this month, or last month when the last day has not come yet.
            let this = (today.year(), today.month());
            let candidate = NaiveDate::from_ymd_opt(this.0, this.1, second.day)?;
            if candidate > today && first.literal.is_none() && second.literal.is_none() {
                let back = first_of_month(today)?.checked_sub_months(Months::new(1))?;
                (back.year(), back.month())
            } else {
                this
            }
        }
    };
    let from = match first.literal {
        Some(d) => d,
        None => {
            let m = first.month.unwrap_or(month);
            let y = first.year.unwrap_or(year);
            NaiveDate::from_ymd_opt(y, m, first.day)?
        }
    };
    let to = match second.literal {
        Some(d) => d,
        None => NaiveDate::from_ymd_opt(
            second.year.unwrap_or(year),
            second.month.unwrap_or(month),
            second.day,
        )?,
    };
    if from > to {
        return None;
    }
    let confidence = if straddles {
        Confidence::Low
    } else if has_month || first.literal.is_some() {
        Confidence::High
    } else {
        Confidence::Medium
    };
    Some((
        period(
            PeriodKind::Range,
            Some(from),
            to,
            confidence,
            typed(scan, at, end),
            today,
        ),
        end,
    ))
}

/// "12 october", "october 12", "12-10-2026".
fn date_period_at(scan: &Scan, at: usize, today: NaiveDate) -> Option<(PeriodRef, usize)> {
    let spec = day_spec(scan, at, false)?;
    let date = date_of(&spec, today, None)?;
    let confidence = if spec.literal.is_some() && spec.year.is_none() {
        Confidence::Medium
    } else {
        Confidence::High
    };
    Some((
        period(
            PeriodKind::Date,
            Some(date),
            date,
            confidence,
            typed(scan, at, spec.end),
            today,
        ),
        spec.end,
    ))
}

/// "january se march tak": from the first day of one month to the last day of the other.
fn month_range_at(scan: &Scan, at: usize, today: NaiveDate) -> Option<(PeriodRef, usize)> {
    let w = words();
    let first = month_period_at(scan, at, today)?;
    let connector = scan.word(first.1)?;
    if !w.from.contains(connector) {
        return None;
    }
    let second = month_period_at(scan, next_index(first.1, 1), today)?;
    let mut end = second.1;
    if scan.word(end).is_some_and(|x| w.until.contains(x)) {
        end = next_index(end, 1);
    }
    let from = first.0.from?;
    if from > second.0.to {
        return None;
    }
    Some((
        period(
            PeriodKind::Range,
            Some(from),
            second.0.to,
            first.0.confidence.min(second.0.confidence),
            typed(scan, at, end),
            today,
        ),
        end,
    ))
}

/// "15 tarikh ko": that day of this month, or of last month when it has not come yet.
fn tarikh_at(scan: &Scan, at: usize, today: NaiveDate) -> Option<(PeriodRef, usize)> {
    let w = words();
    let n = scan.digits_at(at)?;
    let day = u32::try_from(n).ok().filter(|d| (1..=31).contains(d))?;
    if !scan
        .word(next_index(at, 1))
        .is_some_and(|x| w.tarikh.contains(x))
    {
        return None;
    }
    let mut end = next_index(at, 2);
    if matches!(scan.word(end), Some("ko" | "کو")) {
        end = next_index(end, 1);
    }
    let this = NaiveDate::from_ymd_opt(today.year(), today.month(), day);
    let date = match this {
        Some(d) if d <= today => d,
        _ => {
            let back = first_of_month(today)?.checked_sub_months(Months::new(1))?;
            NaiveDate::from_ymd_opt(back.year(), back.month(), day)?
        }
    };
    Some((
        period(
            PeriodKind::Date,
            Some(date),
            date,
            Confidence::Medium,
            typed(scan, at, end),
            today,
        ),
        end,
    ))
}

/// A year by its number: "2025 mein", "commission in 2025".
fn year_period_at(scan: &Scan, at: usize, today: NaiveDate) -> Option<(PeriodRef, usize)> {
    let year = year_at(scan, at)?;
    // 2000 is also an amount ("faisal ko 2000 de do"): a bare number is a year only beside the words
    // that go with a year.
    let before = at
        .checked_sub(1)
        .and_then(|b| scan.word(b))
        .unwrap_or_default();
    let after = scan.word(next_index(at, 1)).unwrap_or_default();
    let said_year = matches!(
        before,
        "in" | "for" | "of" | "during" | "year" | "saal" | "sal" | "since" | "سال"
    ) || matches!(
        after,
        "mein" | "me" | "men" | "ka" | "ki" | "ke" | "میں" | "کا" | "کی" | "کے"
    );
    if !said_year || !(2015..=2100).contains(&year) {
        return None;
    }
    let first = NaiveDate::from_ymd_opt(year, 1, 1)?;
    let last = NaiveDate::from_ymd_opt(year, 12, 31)?;
    let to = if year == today.year() { today } else { last };
    let mut end = next_index(at, 1);
    if matches!(
        scan.word(end),
        Some("mein" | "me" | "men" | "ka" | "ki" | "ke" | "میں")
    ) {
        end = next_index(end, 1);
    }
    Some((
        period(
            PeriodKind::Year(year),
            Some(first),
            to,
            Confidence::High,
            typed(scan, at, end),
            today,
        ),
        end,
    ))
}

/// "10 din ka", "2 hafte ki": the last so many days, weeks, months or years (a little less sure
/// than "pichle 10 din"). "30 din se purana" is not a period.
fn rolling_at(scan: &Scan, at: usize, today: NaiveDate) -> Option<(PeriodRef, usize)> {
    let n = scan.digits_at(at)?;
    let unit = unit_at(scan, next_index(at, 1))?;
    if !matches!(scan.word(next_index(at, 2)), Some("ka" | "ki" | "ke")) {
        return None;
    }
    let count = u32::try_from(n).ok().filter(|c| (1..=1200).contains(c))?;
    let from = match unit {
        Unit::Day => today.checked_sub_signed(Duration::days(i64::from(count)))?,
        Unit::Week => today.checked_sub_signed(Duration::days(i64::from(count).checked_mul(7)?))?,
        Unit::Month => today.checked_sub_months(Months::new(count))?,
        Unit::Year => today.checked_sub_months(Months::new(count.checked_mul(12)?))?,
    };
    let kind = match unit {
        Unit::Day => PeriodKind::LastDays(count),
        Unit::Week => PeriodKind::LastWeeks(count),
        Unit::Month => PeriodKind::LastMonths(count),
        Unit::Year => PeriodKind::LastYears(count),
    };
    let end = next_index(at, 3);
    Some((
        period(
            kind,
            Some(from),
            today,
            Confidence::Medium,
            typed(scan, at, end),
            today,
        ),
        end,
    ))
}

/// A month name, with a year when one follows: the whole month, or up to today for this month.
fn month_period_at(scan: &Scan, at: usize, today: NaiveDate) -> Option<(PeriodRef, usize)> {
    let month = any_month_at(scan, at)?;
    // A month name before a day number is a date ("october 12"), read by `date_period_at`.
    if scan
        .digits_at(next_index(at, 1))
        .is_some_and(|n| (1..=31).contains(&n))
    {
        return None;
    }
    let year_written = year_at(scan, next_index(at, 1));
    let end = next_index(at, if year_written.is_some() { 2 } else { 1 });
    let year = match year_written {
        Some(y) => y,
        None if month <= today.month() => today.year(),
        None => today.year().checked_sub(1)?,
    };
    let first = NaiveDate::from_ymd_opt(year, month, 1)?;
    let last = last_of_month(year, month)?;
    let to = if year_written.is_none() {
        last.min(today)
    } else {
        last
    };
    Some((
        period(
            PeriodKind::Month { year, month },
            Some(first),
            to,
            Confidence::High,
            typed(scan, at, end),
            today,
        ),
        end,
    ))
}

// ---- relative periods ------------------------------------------------------------------------------

fn this_unit(unit: Unit, today: NaiveDate) -> Option<(PeriodKind, NaiveDate, NaiveDate)> {
    Some(match unit {
        Unit::Week => (PeriodKind::ThisWeek, monday_of(today)?, today),
        Unit::Month => (PeriodKind::ThisMonth, first_of_month(today)?, today),
        Unit::Year => (
            PeriodKind::ThisYear,
            NaiveDate::from_ymd_opt(today.year(), 1, 1)?,
            today,
        ),
        Unit::Day => (PeriodKind::Today, today, today),
    })
}

fn last_unit(unit: Unit, today: NaiveDate) -> Option<(PeriodKind, NaiveDate, NaiveDate)> {
    Some(match unit {
        Unit::Week => {
            let monday = monday_of(today)?;
            (
                PeriodKind::LastWeek,
                monday.checked_sub_signed(Duration::days(7))?,
                monday.pred_opt()?,
            )
        }
        Unit::Month => {
            let first = first_of_month(today)?.checked_sub_months(Months::new(1))?;
            (
                PeriodKind::LastMonth,
                first,
                first_of_month(today)?.pred_opt()?,
            )
        }
        Unit::Year => {
            let year = today.year().checked_sub(1)?;
            (
                PeriodKind::LastYear,
                NaiveDate::from_ymd_opt(year, 1, 1)?,
                NaiveDate::from_ymd_opt(year, 12, 31)?,
            )
        }
        Unit::Day => {
            let y = today.pred_opt()?;
            (PeriodKind::Yesterday, y, y)
        }
    })
}

fn next_unit(unit: Unit, today: NaiveDate) -> Option<(PeriodKind, NaiveDate, NaiveDate)> {
    Some(match unit {
        Unit::Week => {
            let monday = monday_of(today)?.checked_add_signed(Duration::days(7))?;
            (
                PeriodKind::NextWeek,
                monday,
                monday.checked_add_signed(Duration::days(6))?,
            )
        }
        Unit::Month => {
            let first = first_of_month(today)?.checked_add_months(Months::new(1))?;
            (
                PeriodKind::NextMonth,
                first,
                last_of_month(first.year(), first.month())?,
            )
        }
        Unit::Year => {
            let year = today.year().checked_add(1)?;
            (
                PeriodKind::NextYear,
                NaiveDate::from_ymd_opt(year, 1, 1)?,
                NaiveDate::from_ymd_opt(year, 12, 31)?,
            )
        }
        Unit::Day => {
            let t = today.succ_opt()?;
            (PeriodKind::Tomorrow, t, t)
        }
    })
}

/// "pichle 3 mahine", "last 30 days", "agle 6 mahine".
fn counted_at(scan: &Scan, at: usize, today: NaiveDate) -> Option<(PeriodRef, usize)> {
    let w = words();
    let head = scan.word(at)?;
    let backward = w.last.contains(head);
    let forward = w.next.contains(head);
    if !backward && !forward {
        return None;
    }
    let n = count_at(scan, next_index(at, 1))?;
    let unit = unit_at(scan, next_index(at, 2))?;
    let count = u32::try_from(n).ok().filter(|c| (1..=1200).contains(c))?;
    let end = next_index(at, 3);
    let words_typed = typed(scan, at, end);
    let (kind, from, to) = if backward {
        let from = match unit {
            Unit::Day => today.checked_sub_signed(Duration::days(i64::from(count)))?,
            Unit::Week => {
                today.checked_sub_signed(Duration::days(i64::from(count).checked_mul(7)?))?
            }
            Unit::Month => today.checked_sub_months(Months::new(count))?,
            Unit::Year => today.checked_sub_months(Months::new(count.checked_mul(12)?))?,
        };
        let kind = match unit {
            Unit::Day => PeriodKind::LastDays(count),
            Unit::Week => PeriodKind::LastWeeks(count),
            Unit::Month => PeriodKind::LastMonths(count),
            Unit::Year => PeriodKind::LastYears(count),
        };
        (kind, from, today)
    } else {
        let first = first_of_month(today)?;
        let to = match unit {
            Unit::Month => first.checked_add_months(Months::new(count))?.pred_opt()?,
            Unit::Year => first
                .checked_add_months(Months::new(count.checked_mul(12)?))?
                .pred_opt()?,
            Unit::Week => {
                today.checked_add_signed(Duration::days(i64::from(count).checked_mul(7)?))?
            }
            Unit::Day => today.checked_add_signed(Duration::days(i64::from(count)))?,
        };
        let kind = match unit {
            Unit::Year => PeriodKind::NextYears(count),
            Unit::Month => PeriodKind::NextMonths(count),
            Unit::Week | Unit::Day => PeriodKind::NextDays(count),
        };
        (
            kind,
            if unit == Unit::Month || unit == Unit::Year {
                first
            } else {
                today
            },
            to,
        )
    };
    Some((
        period(kind, Some(from), to, Confidence::High, words_typed, today),
        end,
    ))
}

/// "is hafte", "pichle mahine", "agle saal".
fn relative_at(scan: &Scan, at: usize, today: NaiveDate) -> Option<(PeriodRef, usize)> {
    let w = words();
    let head = scan.word(at)?;
    let unit = unit_at(scan, next_index(at, 1))?;
    let (kind, from, to) = if w.this.contains(head) {
        this_unit(unit, today)?
    } else if w.last.contains(head) {
        last_unit(unit, today)?
    } else if w.next.contains(head) {
        next_unit(unit, today)?
    } else {
        return None;
    };
    let end = next_index(at, 2);
    // "is din" (this day) and "is hafte" read well; "is" before a bare day word is the weakest.
    let confidence = if unit == Unit::Day && w.this.contains(head) {
        Confidence::Medium
    } else {
        Confidence::High
    };
    Some((
        period(
            kind,
            Some(from),
            to,
            confidence,
            typed(scan, at, end),
            today,
        ),
        end,
    ))
}

/// A single day named by a word: aaj, kal, parson, yesterday, tomorrow.
fn day_word_at(
    scan: &Scan,
    at: usize,
    today: NaiveDate,
    tense: Tense,
) -> Option<(PeriodRef, usize)> {
    let w = words();
    let word = scan.word(at)?;
    let one = |kind: PeriodKind, date: Option<NaiveDate>, confidence: Confidence| {
        let date = date?;
        Some((
            period(kind, Some(date), date, confidence, word.to_owned(), today),
            next_index(at, 1),
        ))
    };
    if w.today.contains(word) {
        return one(PeriodKind::Today, Some(today), Confidence::High);
    }
    if w.yesterday.contains(word) {
        return one(PeriodKind::Yesterday, today.pred_opt(), Confidence::High);
    }
    if w.tomorrow.contains(word) {
        return one(PeriodKind::Tomorrow, today.succ_opt(), Confidence::High);
    }
    if w.kal.contains(word) {
        // "کل رقم" (a partner's total) and "کل ملا کر" are phrases of the table: the کل in them is
        // "total", not a day.
        let in_phrase = scan
            .hits
            .iter()
            .any(|h| !h.fuzzy && h.end.saturating_sub(h.start) > 1 && h.start <= at && at < h.end);
        if in_phrase {
            return None;
        }
        // کل / kal: yesterday after a past verb, tomorrow after a future one. With no verb a data
        // question is about what happened, so yesterday, a little less sure.
        // In Urdu letters, "کل" straight before a figure noun ("کل کمیشن", "کل بکری") is "total" as
        // often as it is "yesterday's": the genitive ("کل کا کمیشن") is what makes it a day. With no
        // verb to settle it, the app is not sure (money review, 05-11-2026).
        let before_noun = word == "کل"
            && scan.hits.iter().any(|h| {
                h.start == next_index(at, 1)
                    && matches!(
                        h.cue,
                        super::lexicon::Cue::Sale
                            | super::lexicon::Cue::SalePlural
                            | super::lexicon::Cue::Commission
                            | super::lexicon::Cue::Laga
                            | super::lexicon::Cue::Dues
                            | super::lexicon::Cue::Collection
                            | super::lexicon::Cue::Payable
                            | super::lexicon::Cue::Advance
                            | super::lexicon::Cue::Balance
                            | super::lexicon::Cue::Cash
                            | super::lexicon::Cue::Stock
                    )
            });
        // "اس سال کی کل بکری": with another time word in the question, "کل" is "total".
        if before_noun
            && scan
                .words
                .iter()
                .enumerate()
                .any(|(i, x)| i != at && !w.kal.contains(x) && is_time_word(x))
        {
            return None;
        }
        return match tense {
            // Only "کل کا / کی / کے" makes it a day: straight before a figure noun it is "total".
            _ if before_noun => one(PeriodKind::Yesterday, today.pred_opt(), Confidence::Low),
            Tense::Past => one(PeriodKind::Yesterday, today.pred_opt(), Confidence::High),
            Tense::Future => one(PeriodKind::Tomorrow, today.succ_opt(), Confidence::High),
            Tense::None => one(PeriodKind::Yesterday, today.pred_opt(), Confidence::Medium),
            Tense::Mixed => one(PeriodKind::Yesterday, today.pred_opt(), Confidence::Low),
        };
    }
    if w.parson.contains(word) {
        let before = today.checked_sub_signed(Duration::days(2));
        let after = today.checked_add_signed(Duration::days(2));
        return match tense {
            Tense::Future => one(PeriodKind::DayAfterTomorrow, after, Confidence::High),
            Tense::Past => one(PeriodKind::DayBeforeYesterday, before, Confidence::High),
            Tense::None => one(PeriodKind::DayBeforeYesterday, before, Confidence::Medium),
            Tense::Mixed => one(PeriodKind::DayBeforeYesterday, before, Confidence::Low),
        };
    }
    None
}

/// "ab tak", "abhi tak", "aaj tak", "till now", "so far".
fn up_to_now_at(scan: &Scan, at: usize, today: NaiveDate) -> Option<(PeriodRef, usize)> {
    let first = scan.word(at)?;
    let second = scan.word(next_index(at, 1)).unwrap_or_default();
    let w = words();
    let two = matches!(
        (first, second),
        ("ab", "tak")
            | ("abhi", "tak")
            | ("aaj", "tak")
            | ("aj", "tak")
            | ("now", "tak")
            | ("till", "now")
            | ("till", "date")
            | ("up", "to")
            | ("upto", "now")
            | ("to", "date")
            | ("so", "far")
            | ("shuru", "se")
            | ("start", "se")
            | ("اب", "تک")
            | ("ابھی", "تک")
            | ("آج", "تک")
            | ("شروع", "سے")
    ) || (w.today.contains(first) && w.until.contains(second));
    if !two {
        return None;
    }
    // "up to now" and "up to date" are "up to" + a third word.
    let taken = if first == "up" && matches!(scan.word(next_index(at, 2)), Some("now" | "date")) {
        3
    } else {
        2
    };
    let end = next_index(at, taken);
    Some((
        period(
            PeriodKind::UpToNow,
            None,
            today,
            Confidence::High,
            typed(scan, at, end),
            today,
        ),
        end,
    ))
}

fn read_at(scan: &Scan, at: usize, today: NaiveDate, tense: Tense) -> Option<(PeriodRef, usize)> {
    up_to_now_at(scan, at, today)
        .or_else(|| range_at(scan, at, today))
        .or_else(|| month_range_at(scan, at, today))
        .or_else(|| date_period_at(scan, at, today))
        .or_else(|| tarikh_at(scan, at, today))
        .or_else(|| month_period_at(scan, at, today))
        .or_else(|| year_period_at(scan, at, today))
        .or_else(|| counted_at(scan, at, today))
        .or_else(|| rolling_at(scan, at, today))
        .or_else(|| relative_at(scan, at, today))
        .or_else(|| day_word_at(scan, at, today, tense))
}

/// A day word joined to another by "se": "kal se aaj tak", "parson se kal".
fn join_day_range(
    first: PeriodRef,
    start: usize,
    scan: &Scan,
    next: usize,
    today: NaiveDate,
    tense: Tense,
) -> Option<(PeriodRef, usize)> {
    let w = words();
    let single = |p: &PeriodRef| p.from == Some(p.to);
    if !single(&first) || !scan.word(next).is_some_and(|x| w.from.contains(x)) {
        return None;
    }
    let after = next_index(next, 1);
    let (second, mut end) =
        day_word_at(scan, after, today, tense).or_else(|| date_period_at(scan, after, today))?;
    if !single(&second) {
        return None;
    }
    if scan.word(end).is_some_and(|x| w.until.contains(x)) {
        end = next_index(end, 1);
    }
    let from = first.from?;
    if from > second.to {
        return None;
    }
    let confidence = first.confidence.min(second.confidence);
    Some((
        period(
            PeriodKind::Range,
            Some(from),
            second.to,
            confidence,
            typed(scan, start, end),
            today,
        ),
        end,
    ))
}

/// This year, from 1 January to today, for a ranking that named no period (plan 55 ruling 2). It is
/// a period the app chose, so it is not sure of it.
pub(super) fn this_year(today: NaiveDate) -> Option<PeriodRef> {
    let first = NaiveDate::from_ymd_opt(today.year(), 1, 1)?;
    Some(period(
        PeriodKind::ThisYear,
        Some(first),
        today,
        Confidence::Medium,
        String::new(),
        today,
    ))
}

/// This month so far (the 1st to today), the base of "pichle mahine se zyada" when the question names
/// only the month it is compared with.
pub(super) fn this_month_so_far(today: NaiveDate) -> Option<PeriodRef> {
    let first = NaiveDate::from_ymd_opt(today.year(), today.month(), 1)?;
    Some(period(
        PeriodKind::ThisMonth,
        Some(first),
        today,
        Confidence::Medium,
        String::new(),
        today,
    ))
}

/// The two days of a range written with no month and no full date ("1 se 15 tak"), if it is one.
fn bare_days(scan: &Scan, at: usize) -> Option<(u32, u32)> {
    let first = day_spec(scan, at, true)?;
    let connector = scan.word(first.end)?;
    if !words().from.contains(connector) {
        return None;
    }
    let second = day_spec(scan, next_index(first.end, 1), true)?;
    let bare = |s: &DaySpec| s.month.is_none() && s.literal.is_none();
    (bare(&first) && bare(&second)).then_some((first.day, second.day))
}

/// A range of days with no month of its own, and a month word in the same question ("pichle mahine
/// ki 1 se 15 tak"): the range is inside that month. The words win (lead ruling of 04-11-2026), so
/// the two periods become one, cut at today. With more than one month word or more than one such
/// range the question is a comparison and is left as it is.
///
/// Example (today 05-11-2026): "pichle mahine ki 1 se 15 tak" is 01-10-2026 to 15-10-2026.
fn merge_month_and_days(out: &mut Vec<PeriodRef>, bare: &[Option<(u32, u32)>], today: NaiveDate) {
    let is_month = |p: &PeriodRef| {
        matches!(
            p.kind,
            PeriodKind::ThisMonth
                | PeriodKind::LastMonth
                | PeriodKind::NextMonth
                | PeriodKind::Month { .. }
        )
    };
    let months: Vec<usize> = out
        .iter()
        .enumerate()
        .filter(|(_, p)| is_month(p))
        .map(|(i, _)| i)
        .collect();
    let ranges: Vec<usize> = bare
        .iter()
        .enumerate()
        .filter(|(_, b)| b.is_some())
        .map(|(i, _)| i)
        .collect();
    let (&[month_at], &[range_at]) = (months.as_slice(), ranges.as_slice()) else {
        return;
    };
    let (Some(month), Some(Some((d1, d2)))) = (out.get(month_at), bare.get(range_at)) else {
        return;
    };
    let Some(first) = month.from else { return };
    let (year, m) = (first.year(), first.month());
    let Some(last) = last_of_month(year, m) else {
        return;
    };
    let Some(from) = NaiveDate::from_ymd_opt(year, m, *d1) else {
        return;
    };
    // "1 se 31" in a month of 30 days ends on the last day; never after the month's own end (this
    // month stops at today).
    let to = NaiveDate::from_ymd_opt(year, m, *d2)
        .unwrap_or(last)
        .min(month.to);
    if from > to {
        return;
    }
    let merged = period(
        PeriodKind::Range,
        Some(from),
        to,
        month.confidence,
        format!(
            "{} {}",
            month.words,
            out.get(range_at).map_or("", |r| r.words.as_str())
        ),
        today,
    );
    let (keep, drop) = if month_at < range_at {
        (month_at, range_at)
    } else {
        (range_at, month_at)
    };
    if let Some(slot) = out.get_mut(keep) {
        *slot = merged;
    }
    out.remove(drop);
}

/// "kal tak", "30 september tak": from the first day of the books up to and including that day, not
/// that one day (money review BUG-3). The first day of the books is filled in by the caller, which
/// can read them.
///
/// Example (today 04-11-2026): "kal tak" is the books up to 03-11-2026.
fn until_day(
    found: &PeriodRef,
    start: usize,
    end: usize,
    scan: &Scan,
    today: NaiveDate,
) -> Option<(PeriodRef, usize)> {
    let named_day = found.from == Some(found.to)
        && matches!(
            found.kind,
            PeriodKind::Date | PeriodKind::Yesterday | PeriodKind::DayBeforeYesterday
        );
    if !named_day || !scan.word(end).is_some_and(|x| words().until.contains(x)) {
        return None;
    }
    let end = next_index(end, 1);
    Some((
        period(
            PeriodKind::UpToDay,
            None,
            found.to,
            found.confidence,
            typed(scan, start, end),
            today,
        ),
        end,
    ))
}

/// Words that stand before a unit of time to count it or to spread it ("do mahine", "har hafte"): the
/// unit is then not "this" one.
const COUNTING: [&str; 14] = [
    "har", "poora", "pura", "pure", "puri", "poori", "aadha", "adha", "kai", "kuch", "ek", "aik",
    "ہر", "پورا",
];

/// A word that points at a period before or after the one now ("pichla", "agle"), without the verbs
/// that only end in the same letters ("gaye", "aane").
fn is_head(word: &str) -> bool {
    let w = words();
    (w.last.contains(word) && !matches!(word, "gaye" | "gaya" | "gayi" | "گئے"))
        || (w.next.contains(word) && !matches!(word, "aane" | "aanay" | "آنے"))
}

/// The words after "last" that make it the last sale of a rate, with a rate word in the question.
const LAST_RATE_NEXT: [&str; 14] = [
    "rate", "bhao", "price", "time", "bar", "baar", "dafa", "dafah", "martaba", "ریٹ", "بھاؤ",
    "قیمت", "بار", "دفعہ",
];

/// The time words that no period took ("mahine ka", "hafte ki", "pichli", "din"). A unit of a week,
/// a month or a year with "ka / ki / ke" and nothing counting it is this one: "faisal ka mahine ka
/// commission" is this month (lead ruling of 04-11-2026), added to `out`. Anything else is a time
/// the app cannot place; the answer is `true`, and the question is not sure (money review BUG-4).
fn stray_time_words(
    scan: &Scan,
    used: &[(usize, usize)],
    today: NaiveDate,
    out: &mut Vec<PeriodRef>,
) -> bool {
    let mut stray = false;
    for at in 0..scan.words.len() {
        let in_period = used.iter().any(|(s, e)| *s <= at && at < *e);
        // A word of a phrase of the cue table is that phrase's, not a time.
        let in_phrase = scan
            .hits
            .iter()
            .any(|h| !h.fuzzy && h.start <= at && at < h.end);
        let Some(word) = scan.word(at) else { continue };
        if in_period || in_phrase {
            continue;
        }
        let Some(unit) = unit_at(scan, at) else {
            // "last rate", "pichli bar ... kis rate pe": the last sale of a rate, no time to place.
            let last_rate = scan.has(super::lexicon::Cue::Rate)
                && scan
                    .word(next_index(at, 1))
                    .is_some_and(|x| LAST_RATE_NEXT.contains(&x));
            stray |= is_head(word) && !last_rate;
            continue;
        };
        let before = at.checked_sub(1);
        let counted = before.is_some_and(|b| {
            count_at(scan, b).is_some()
                || scan
                    .word(b)
                    .is_some_and(|x| COUNTING.contains(&x) || is_head(x))
        });
        let possessive = scan
            .word(next_index(at, 1))
            .is_some_and(|x| matches!(x, "ka" | "ki" | "ke" | "kay" | "کا" | "کی" | "کے"));
        match this_unit(unit, today) {
            Some((kind, from, to)) if unit != Unit::Day && possessive && !counted => {
                out.push(period(
                    kind,
                    Some(from),
                    to,
                    Confidence::Medium,
                    word.to_owned(),
                    today,
                ));
            }
            _ => stray = true,
        }
    }
    stray
}

/// The periods a question names and whether a time word was left that no period took.
pub(super) struct PeriodsRead {
    /// Every period, in order, as exact dates.
    pub periods: Vec<PeriodRef>,
    /// True when a time word stands that the app cannot place.
    pub stray: bool,
}

/// Every period the question names, in order, as exact dates.
///
/// Example (today 04-11-2026): "kal ki bikri kitni thi" is yesterday, 03-11-2026 (a past verb).
pub(super) fn periods(scan: &Scan, today: NaiveDate) -> Vec<PeriodRef> {
    read_periods(scan, today).periods
}

/// The periods of a question, and whether a time word was left over.
pub(super) fn read_periods(scan: &Scan, today: NaiveDate) -> PeriodsRead {
    let tense = tense_of(scan);
    let mut out: Vec<PeriodRef> = Vec::new();
    // For each period: the days of a range with no month of its own.
    let mut bare: Vec<Option<(u32, u32)>> = Vec::new();
    // The words each period took.
    let mut used: Vec<(usize, usize)> = Vec::new();
    let mut at = 0usize;
    while at < scan.words.len() {
        match read_at(scan, at, today, tense) {
            Some((found, end)) => {
                if let Some((joined, joined_end)) =
                    join_day_range(found.clone(), at, scan, end, today, tense)
                {
                    out.push(joined);
                    bare.push(None);
                    used.push((at, joined_end));
                    at = joined_end;
                } else if let Some((upto, upto_end)) = until_day(&found, at, end, scan, today) {
                    out.push(upto);
                    bare.push(None);
                    used.push((at, upto_end));
                    at = upto_end;
                } else {
                    bare.push(if found.kind == PeriodKind::Range {
                        bare_days(scan, at)
                    } else {
                        None
                    });
                    out.push(found);
                    used.push((at, end));
                    at = end;
                }
            }
            None => at = next_index(at, 1),
        }
    }
    merge_month_and_days(&mut out, &bare, today);
    // "is mahine ab tak": the "so far" is the month up to today, not a second period.
    if out.len() > 1
        && out.iter().any(|p| p.kind == PeriodKind::UpToNow)
        && out
            .iter()
            .any(|p| p.kind != PeriodKind::UpToNow && p.from.is_some() && p.to == today)
    {
        out.retain(|p| p.kind != PeriodKind::UpToNow);
    }
    // "saal ke aakhir tak": from today to 31 December. The year it belongs to is not a second
    // period, and its "saal" is not a stray time word.
    let year_end = cues::year_end(&scan.raw);
    // A question that has a period already ("whole day's bikri for this month") is not unsure of its
    // time for a stray word: only a question with none reads the stray word as its period.
    let stray = !year_end && out.is_empty() && stray_time_words(scan, &used, today, &mut out);
    if year_end {
        out.retain(|p| !matches!(p.kind, PeriodKind::ThisYear));
        if let Some(end) = NaiveDate::from_ymd_opt(today.year(), 12, 31) {
            out.push(period(
                PeriodKind::YearEnd,
                Some(today),
                end,
                Confidence::High,
                "saal ke aakhir tak".to_owned(),
                today,
            ));
        }
    }
    PeriodsRead {
        periods: out,
        stray,
    }
}
