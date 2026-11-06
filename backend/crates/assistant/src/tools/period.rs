//! The tool `period`: from and to dates, and a label, for a period word (PRD FR-7.77).
//!
//! Periods come from a fixed list, worked out from today's Pakistan date. A period the list does
//! not hold is not guessed: the model asks for the dates (`need_dates`). A number of days, months
//! or years must be a number of the question with that word after it (D6-system-prompt "Rules for
//! the inputs"). A date that does not exist (31-02) is refused.
//!
//! Example (today 10-10-26): `last_month` is 01-09-26 to 30-09-26; `last_n_years` with 2 on 01-11-26
//! is 01-11-24 to 01-11-26; `last_n_months` with 1 on 31-03-26 is 28-02-26 to 31-03-26 (a month end
//! moves to the last day of a shorter month).

use chrono::{Datelike, Duration, Months, NaiveDate};
use mandi_services::domain::figures::NumberUnit;
use serde::Deserialize;
use serde_json::Value;

use super::{Draft, Env, Stop, ToolError, input};
use crate::results::{Node, Val};

/// The period words.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    /// آج.
    Today,
    /// کل.
    Yesterday,
    /// یہ ہفتہ: Monday to today.
    ThisWeek,
    /// پچھلا ہفتہ.
    LastWeek,
    /// یہ مہینہ: the 1st to today.
    ThisMonth,
    /// پچھلا مہینہ.
    LastMonth,
    /// یہ سال: 1 January to today.
    ThisYear,
    /// پچھلا سال.
    LastYear,
    /// اب تک: from the first day of the books to today.
    UpToNow,
    /// A named month: the latest one that has started.
    Month,
    /// A day, DD-MM: this year, or last year if it has not come yet.
    Day,
    /// Given dates.
    Range,
    /// The last N days.
    LastNDays,
    /// The last N months.
    LastNMonths,
    /// The last N years.
    LastNYears,
    /// The next N months (estimates only): this month and the following ones.
    NextNMonths,
}

/// The input of `period` (D6-system-prompt Section 3).
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Input {
    /// The period word.
    pub kind: Kind,
    /// 1 to 12, for `month`.
    pub month: Option<i64>,
    /// DD-MM, for `day`.
    pub day: Option<String>,
    /// YYYY-MM-DD, for `range`.
    pub from: Option<String>,
    /// YYYY-MM-DD, for `range`.
    pub to: Option<String>,
    /// For the `last_n` and `next_n` kinds: a number of the question with that unit.
    pub n: Option<i64>,
}

/// From and to. `from` is `None` for "up to now": from the first day of the books.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Range {
    /// First day, or `None` for the first day of the books.
    pub from: Option<NaiveDate>,
    /// Last day.
    pub to: NaiveDate,
    /// The month the period is, when it is exactly one month (for the label).
    pub month: Option<(i32, u32)>,
}

fn day(y: i32, m: u32, d: u32) -> Option<NaiveDate> {
    NaiveDate::from_ymd_opt(y, m, d)
}

fn first_of(date: NaiveDate) -> Option<NaiveDate> {
    date.with_day(1)
}

fn last_of_month(y: i32, m: u32) -> Option<NaiveDate> {
    let (ny, nm) = if m == 12 {
        (y.checked_add(1)?, 1)
    } else {
        (y, m.checked_add(1)?)
    };
    day(ny, nm, 1)?.pred_opt()
}

/// The month a range is, when it is exactly one whole calendar month (first day to last day), so
/// the result carries its `label` like the month periods do. The app plans "vegetables in
/// september" as the range 01-09 to 30-09, and the answer must name the month from the label and
/// not by typing it (hidden round 4, H394).
///
/// Example: 2026-09-01 to 2026-09-30 is September 2026; 2026-09-01 to 2026-09-29 is no month.
fn whole_month(from: NaiveDate, to: NaiveDate) -> Option<(i32, u32)> {
    (from.day() == 1 && last_of_month(from.year(), from.month()) == Some(to))
        .then(|| (from.year(), from.month()))
}

fn one_month(first: NaiveDate, to: NaiveDate) -> Range {
    Range {
        from: Some(first),
        to,
        month: Some((first.year(), first.month())),
    }
}

/// The period for a kind, worked out from today.
///
/// # Errors
/// `ERR_OUT_OF_RANGE` / `ERR_REQUIRED` on the field the kind needs; `ERR_DATE_INVALID`.
pub fn compute(
    today: NaiveDate,
    input: &Input,
    numbers: &[mandi_services::domain::figures::QuestionNumber],
) -> Result<Range, ToolError> {
    let out = |field: &str| ToolError::new("ERR_OUT_OF_RANGE", field, "outside the list");
    let need = |field: &str| ToolError::new("ERR_REQUIRED", field, "this kind needs it");
    let date_err = |field: &str, text: &str| ToolError::new("ERR_DATE_INVALID", field, text);
    let overflow = || ToolError::new("ERR_OUT_OF_RANGE", "kind", "the date is out of range");
    let this_first = first_of(today).ok_or_else(overflow)?;
    let range = match input.kind {
        Kind::Today => Range {
            from: Some(today),
            to: today,
            month: None,
        },
        Kind::Yesterday => {
            let y = today.pred_opt().ok_or_else(overflow)?;
            Range {
                from: Some(y),
                to: y,
                month: None,
            }
        }
        Kind::ThisWeek | Kind::LastWeek => {
            let back = i64::from(today.weekday().num_days_from_monday());
            let monday = today
                .checked_sub_signed(Duration::days(back))
                .ok_or_else(overflow)?;
            if input.kind == Kind::ThisWeek {
                Range {
                    from: Some(monday),
                    to: today,
                    month: None,
                }
            } else {
                let from = monday
                    .checked_sub_signed(Duration::days(7))
                    .ok_or_else(overflow)?;
                let to = monday.pred_opt().ok_or_else(overflow)?;
                Range {
                    from: Some(from),
                    to,
                    month: None,
                }
            }
        }
        Kind::ThisMonth => one_month(this_first, today),
        Kind::LastMonth => {
            let first = this_first
                .checked_sub_months(Months::new(1))
                .ok_or_else(overflow)?;
            one_month(first, this_first.pred_opt().ok_or_else(overflow)?)
        }
        Kind::ThisYear => Range {
            from: day(today.year(), 1, 1),
            to: today,
            month: None,
        },
        Kind::LastYear => {
            let y = today.year().checked_sub(1).ok_or_else(overflow)?;
            Range {
                from: day(y, 1, 1),
                to: day(y, 12, 31).ok_or_else(overflow)?,
                month: None,
            }
        }
        Kind::UpToNow => Range {
            from: None,
            to: today,
            month: None,
        },
        Kind::Month => {
            let m = input.month.ok_or_else(|| need("month"))?;
            let m = u32::try_from(m)
                .ok()
                .filter(|m| (1..=12).contains(m))
                .ok_or_else(|| out("month"))?;
            // The latest month of that name that has started: this year's, or last year's.
            let y = if m <= today.month() {
                today.year()
            } else {
                today.year().checked_sub(1).ok_or_else(overflow)?
            };
            let first = day(y, m, 1).ok_or_else(overflow)?;
            let last = last_of_month(y, m).ok_or_else(overflow)?;
            one_month(first, last.min(today))
        }
        Kind::Day => {
            let text = input.day.as_deref().ok_or_else(|| need("day"))?;
            let (d, m) = text.split_once('-').ok_or_else(|| date_err("day", text))?;
            let (d, m): (u32, u32) = (
                d.trim().parse().map_err(|_| date_err("day", text))?,
                m.trim().parse().map_err(|_| date_err("day", text))?,
            );
            let mut date = day(today.year(), m, d).ok_or_else(|| date_err("day", text))?;
            if date > today {
                date = day(today.year().checked_sub(1).ok_or_else(overflow)?, m, d)
                    .ok_or_else(|| date_err("day", text))?;
            }
            Range {
                from: Some(date),
                to: date,
                month: None,
            }
        }
        Kind::Range => {
            let parse = |field: &str, text: Option<&str>| -> Result<NaiveDate, ToolError> {
                let text = text.ok_or_else(|| need(field))?;
                NaiveDate::parse_from_str(text, "%Y-%m-%d").map_err(|_| date_err(field, text))
            };
            let from = parse("from", input.from.as_deref())?;
            let to = parse("to", input.to.as_deref())?;
            if from > to {
                return Err(ToolError::new("ERR_DATE_RANGE", "to", "to is before from"));
            }
            Range {
                from: Some(from),
                to,
                month: whole_month(from, to),
            }
        }
        Kind::LastNDays | Kind::LastNMonths | Kind::LastNYears | Kind::NextNMonths => {
            let (unit, field) = match input.kind {
                Kind::LastNDays => (NumberUnit::Days, "n"),
                Kind::LastNYears => (NumberUnit::Years, "n"),
                _ => (NumberUnit::Months, "n"),
            };
            let n = input.n.ok_or_else(|| need(field))?;
            let typed = numbers.iter().any(|q| q.number == n && q.unit == unit);
            if !typed || n <= 0 {
                return Err(ToolError::new(
                    "ERR_OUT_OF_RANGE",
                    field,
                    "n must be a number of the question with this unit",
                ));
            }
            let n = u32::try_from(n).map_err(|_| out(field))?;
            match input.kind {
                Kind::LastNDays => Range {
                    from: Some(
                        today
                            .checked_sub_signed(Duration::days(i64::from(n)))
                            .ok_or_else(overflow)?,
                    ),
                    to: today,
                    month: None,
                },
                Kind::LastNMonths => Range {
                    from: Some(
                        today
                            .checked_sub_months(Months::new(n))
                            .ok_or_else(overflow)?,
                    ),
                    to: today,
                    month: None,
                },
                Kind::LastNYears => Range {
                    from: Some(
                        today
                            .checked_sub_months(Months::new(
                                n.checked_mul(12).ok_or_else(overflow)?,
                            ))
                            .ok_or_else(overflow)?,
                    ),
                    to: today,
                    month: None,
                },
                _ => {
                    let end = this_first
                        .checked_add_months(Months::new(n))
                        .and_then(|d| d.pred_opt())
                        .ok_or_else(overflow)?;
                    Range {
                        from: Some(this_first),
                        to: end,
                        month: None,
                    }
                }
            }
        }
    };
    Ok(range)
}

/// The tool.
pub fn run(env: &Env<'_>, _state: &super::State, args: &Value) -> Result<Draft, Stop> {
    let parsed: Input = input(args)?;
    let today = NaiveDate::parse_from_str(&env.today.to_iso(), "%Y-%m-%d")
        .map_err(|_| ToolError::new("ERR_INTERNAL", "today", "no date"))?;
    let mut range = compute(today, &parsed, env.numbers)?;
    let iso = |d: NaiveDate| d.format("%Y-%m-%d").to_string();
    let mut note = None;
    // "Up to now" starts at the first day of the books (plan 52 item 2), so every tool, and a
    // comparison, gets a real start date. With nothing posted yet it stays open.
    if parsed.kind == Kind::UpToNow
        && range.from.is_none()
        && let Some(first) = books_start(env)
        && first <= range.to
    {
        range.from = Some(first);
        note = Some(format!(
            "up_to_now starts at the first day of the books, {}",
            iso(first)
        ));
    }
    let mut fields = vec![
        ("kind", Node::Str(format!("{:?}", parsed.kind))),
        (
            "from",
            range
                .from
                .map_or(Node::Null, |d| Node::Val(Val::Date(iso(d)))),
        ),
        ("to", Node::Val(Val::Date(iso(range.to)))),
    ];
    if let Some((year, month)) = range.month {
        fields.push(("label", Node::Val(Val::Month { year, month })));
    }
    if let Some(note) = note {
        fields.push(("note", Node::Str(note)));
    }
    Ok(Draft::new("period", args.clone(), Node::obj(fields)))
}

/// The first day of the books (the oldest dated record of any kind), or `None` with nothing yet.
pub(super) fn books_start(env: &Env<'_>) -> Option<NaiveDate> {
    mandi_services::day::first_book_day(env.ctx)
        .ok()
        .flatten()
        .and_then(|d| NaiveDate::parse_from_str(&d.to_iso(), "%Y-%m-%d").ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    use mandi_services::domain::figures::question_numbers;

    fn d(y: i32, m: u32, dd: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, dd).unwrap()
    }

    fn input(kind: Kind) -> Input {
        Input {
            kind,
            month: None,
            day: None,
            from: None,
            to: None,
            n: None,
        }
    }

    fn range(today: NaiveDate, kind: Kind) -> (Option<NaiveDate>, NaiveDate) {
        let r = compute(today, &input(kind), &[]).unwrap();
        (r.from, r.to)
    }

    #[test]
    fn the_simple_words() {
        let today = d(2026, 10, 10); // a Saturday
        assert_eq!(range(today, Kind::Today), (Some(today), today));
        assert_eq!(
            range(today, Kind::Yesterday),
            (Some(d(2026, 10, 9)), d(2026, 10, 9))
        );
        assert_eq!(range(today, Kind::ThisWeek), (Some(d(2026, 10, 5)), today));
        assert_eq!(
            range(today, Kind::LastWeek),
            (Some(d(2026, 9, 28)), d(2026, 10, 4))
        );
        assert_eq!(range(today, Kind::ThisMonth), (Some(d(2026, 10, 1)), today));
        assert_eq!(
            range(today, Kind::LastMonth),
            (Some(d(2026, 9, 1)), d(2026, 9, 30))
        );
        assert_eq!(range(today, Kind::ThisYear), (Some(d(2026, 1, 1)), today));
        assert_eq!(
            range(today, Kind::LastYear),
            (Some(d(2025, 1, 1)), d(2025, 12, 31))
        );
        assert_eq!(range(today, Kind::UpToNow), (None, today));
    }

    #[test]
    fn month_and_year_ends() {
        // January: last month is December of the year before; last week crosses the year.
        let jan = d(2027, 1, 3);
        assert_eq!(
            range(jan, Kind::LastMonth),
            (Some(d(2026, 12, 1)), d(2026, 12, 31))
        );
        assert_eq!(
            range(jan, Kind::LastWeek),
            (Some(d(2026, 12, 21)), d(2026, 12, 27))
        );
        // March after a leap day and after a short February.
        assert_eq!(
            range(d(2028, 3, 1), Kind::LastMonth),
            (Some(d(2028, 2, 1)), d(2028, 2, 29))
        );
        assert_eq!(
            range(d(2026, 3, 1), Kind::LastMonth),
            (Some(d(2026, 2, 1)), d(2026, 2, 28))
        );
    }

    #[test]
    fn a_named_month_is_the_latest_that_has_started() {
        let today = d(2026, 10, 10);
        let mut i = input(Kind::Month);
        i.month = Some(8);
        let r = compute(today, &i, &[]).unwrap();
        assert_eq!(
            (r.from, r.to, r.month),
            (Some(d(2026, 8, 1)), d(2026, 8, 31), Some((2026, 8)))
        );
        // December has not started this year: it is last December.
        i.month = Some(12);
        let r = compute(today, &i, &[]).unwrap();
        assert_eq!((r.from, r.to), (Some(d(2025, 12, 1)), d(2025, 12, 31)));
        // The current month stops at today.
        i.month = Some(10);
        assert_eq!(compute(today, &i, &[]).unwrap().to, today);
        i.month = Some(13);
        assert!(compute(today, &i, &[]).is_err());
        i.month = None;
        assert!(compute(today, &i, &[]).is_err());
    }

    #[test]
    fn last_n_needs_a_number_of_the_question_with_the_right_unit() {
        let today = d(2026, 11, 1);
        let numbers = question_numbers("pichle 2 saal ka hisab");
        let mut i = input(Kind::LastNYears);
        i.n = Some(2);
        let r = compute(today, &i, &numbers).unwrap();
        assert_eq!((r.from, r.to), (Some(d(2024, 11, 1)), today));
        // The wrong unit, a number that is not in the question, zero.
        i.kind = Kind::LastNMonths;
        assert!(compute(today, &i, &numbers).is_err());
        i.kind = Kind::LastNYears;
        i.n = Some(3);
        assert!(compute(today, &i, &numbers).is_err());
        i.n = Some(0);
        assert!(compute(today, &i, &question_numbers("0 saal")).is_err());
        // Days: the example of the prompt, n = 10000 is refused when 10 is the days number.
        let numbers = question_numbers("pichle 10 din mein 10,000 se zyada");
        let mut i = input(Kind::LastNDays);
        i.n = Some(10);
        assert_eq!(
            compute(today, &i, &numbers).unwrap().from,
            Some(d(2026, 10, 22))
        );
        i.n = Some(10_000);
        assert!(compute(today, &i, &numbers).is_err());
    }

    #[test]
    fn a_month_end_moves_to_the_last_day_of_a_shorter_month() {
        let numbers = question_numbers("pichle 1 mahine");
        let mut i = input(Kind::LastNMonths);
        i.n = Some(1);
        let r = compute(d(2026, 3, 31), &i, &numbers).unwrap();
        assert_eq!(r.from, Some(d(2026, 2, 28)));
        let r = compute(d(2028, 3, 31), &i, &numbers).unwrap();
        assert_eq!(r.from, Some(d(2028, 2, 29)));
    }

    #[test]
    fn next_months_start_with_the_current_month() {
        let numbers = question_numbers("agle 6 mahine");
        let mut i = input(Kind::NextNMonths);
        i.n = Some(6);
        let r = compute(d(2026, 10, 10), &i, &numbers).unwrap();
        assert_eq!((r.from, r.to), (Some(d(2026, 10, 1)), d(2027, 3, 31)));
    }

    #[test]
    fn a_range_that_is_one_whole_month_is_labelled_with_it() {
        // H394: "vegetables in september" is read as the range 01-09 to 30-09, and the answer
        // writes the month from the label.
        let today = d(2026, 11, 5);
        let range = |from: &str, to: &str| {
            let mut i = input(Kind::Range);
            i.from = Some(from.to_owned());
            i.to = Some(to.to_owned());
            compute(today, &i, &[]).unwrap().month
        };
        assert_eq!(range("2026-09-01", "2026-09-30"), Some((2026, 9)));
        assert_eq!(range("2024-02-01", "2024-02-29"), Some((2024, 2)));
        assert_eq!(range("2026-12-01", "2026-12-31"), Some((2026, 12)));
        // Not a whole month: no label.
        assert_eq!(range("2026-09-01", "2026-09-29"), None);
        assert_eq!(range("2026-09-02", "2026-09-30"), None);
        assert_eq!(range("2026-08-01", "2026-09-30"), None);
        assert_eq!(range("2026-09-01", "2026-09-01"), None);
    }

    #[test]
    fn dates_are_checked() {
        let today = d(2026, 10, 10);
        let mut i = input(Kind::Range);
        i.from = Some("2026-02-31".to_owned());
        i.to = Some("2026-03-01".to_owned());
        assert_eq!(
            compute(today, &i, &[]).unwrap_err().code,
            "ERR_DATE_INVALID"
        );
        i.from = Some("2026-03-05".to_owned());
        assert_eq!(compute(today, &i, &[]).unwrap_err().code, "ERR_DATE_RANGE");
        i.from = Some("2026-03-01".to_owned());
        assert!(compute(today, &i, &[]).is_ok());
        let mut day = input(Kind::Day);
        day.day = Some("31-02".to_owned());
        assert!(compute(today, &day, &[]).is_err());
        day.day = Some("05-10".to_owned());
        assert_eq!(compute(today, &day, &[]).unwrap().to, d(2026, 10, 5));
        // A day that has not come yet this year is last year's.
        day.day = Some("25-12".to_owned());
        assert_eq!(compute(today, &day, &[]).unwrap().to, d(2025, 12, 25));
    }
}
