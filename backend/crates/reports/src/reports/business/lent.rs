//! R32 ادھار رقم رپورٹ (the money lent report) (PRD 19.2, 16.5, FR-16.4, FR-16.5, FR-7.44).
//!
//! One row for each person who was lent money: what was lent, what came back in cash and from
//! salary, what is still open, the oldest open loan, the earliest return-by day and how many days
//! it is past. Those with something open come first, the oldest open loan first; then the settled
//! ones. The report is as of one day (PRD FR-19.10: a balance list takes an "as of" day), today
//! when none is given.
//!
//! The open amount of a loan is worked out at read time, oldest first, from the live lines of
//! the person's account (money agent 8.2b). Nothing is stored. The working is the one of
//! `mandi_services::lent::report`, kept here as a copy because `mandi-reports` may not depend on
//! the services (see `workings`); a test holds the two side by side on the PRD example.
//!
//! Example (E2E-30, اسلم, as of 12-10-26): lent 5,000, returned 3,000 in cash and 2,000 from
//! salary, open 0. Example (E2E-30b, as of 16-10-26, no salary deduction): lent 5,000, returned
//! 3,000, open 2,000, return by 15-10-26, 1 day past.

use std::collections::HashMap;

use mandi_core::dates::BusinessDate;
use mandi_core::money::Paisa;
use mandi_db::open_lines::{self, LiveLine, PersonFacts};
use mandi_db::rusqlite::Connection;

use super::super::PageSpec;
use crate::error::ReportsError;
use crate::reports::common::{add, col, count_of, date_cell, load_parties, paisa};
use crate::reports::filters::Typed;
use crate::reports::ids;
use crate::reports::model::{
    Cell, ColumnKind, ReportId, ReportPage, Row, RowKind, RowSource, SourceKind, SummaryLine,
};
use crate::reports::workings::{OpenAmount, days_past_return_by, open_amounts};

/// A loan (or an opening debit), with what was returned of it.
pub(crate) struct Loan<'a> {
    pub(crate) line: &'a LiveLine,
    pub(crate) figures: OpenAmount,
    pub(crate) return_by: Option<BusinessDate>,
}

/// One account's loans with what its credits returned, split into cash and salary.
pub(crate) struct Loans<'a> {
    /// The loans, oldest first.
    pub(crate) loans: Vec<Loan<'a>>,
    /// Σ credits that are not salary deductions.
    returned_cash: Paisa,
    /// Σ credits that are salary deductions.
    returned_salary: Paisa,
}

/// The loans of each person with their open amounts, oldest first. `as_of` (`YYYY-MM-DD`,
/// included) leaves out the lines dated after it.
pub(crate) fn loans_by_person<'a>(
    lines: &'a [LiveLine],
    as_of: &str,
) -> Result<HashMap<i64, Loans<'a>>, ReportsError> {
    let mut by_account: HashMap<i64, Vec<&'a LiveLine>> = HashMap::new();
    for line in lines.iter().filter(|l| l.entry_date.as_str() <= as_of) {
        by_account.entry(line.account_id).or_default().push(line);
    }
    let mut out = HashMap::new();
    for (account, own) in by_account {
        let mut bills = Vec::new();
        for line in own.iter().filter(|l| l.dr_paisa > 0) {
            bills.push((line.line_id, paisa(line.dr_paisa)?));
        }
        let (mut returned_cash, mut returned_salary) = (Paisa::ZERO, Paisa::ZERO);
        let mut credits = Vec::new();
        for line in own.iter().filter(|l| l.cr_paisa > 0) {
            let amount = paisa(line.cr_paisa)?;
            credits.push((line.line_id, amount));
            // Any credit that is not a salary deduction counts as cash (a جنرل credit, an
            // opening credit).
            add(
                if line.salary {
                    &mut returned_salary
                } else {
                    &mut returned_cash
                },
                amount,
            )?;
        }
        let figures: HashMap<i64, OpenAmount> = open_amounts(&bills, &credits)?
            .into_iter()
            .map(|o| (o.line_id, o))
            .collect();
        let loans = own
            .iter()
            .filter(|l| l.dr_paisa > 0)
            .filter_map(|line| {
                figures.get(&line.line_id).map(|figures| Loan {
                    line,
                    figures: *figures,
                    return_by: line.return_by.as_deref().and_then(BusinessDate::parse_iso),
                })
            })
            .collect();
        out.insert(
            account,
            Loans {
                loans,
                returned_cash,
                returned_salary,
            },
        );
    }
    Ok(out)
}

/// One person's figures as of the day.
struct PersonRow {
    account_id: i64,
    is_staff: bool,
    lent: Paisa,
    returned_cash: Paisa,
    returned_salary: Paisa,
    open: Paisa,
    oldest_open: Option<String>,
    return_by: Option<String>,
    days_past: Option<i64>,
}

/// Builds R32.
///
/// # Errors
/// A database error; `ReportsError::Money` if a stored amount is not whole rupees, a total does
/// not fit, or more is returned than was lent in a way the matching cannot place.
pub(crate) fn run(
    conn: &Connection,
    typed: &Typed,
    spec: PageSpec,
) -> Result<ReportPage, ReportsError> {
    let today = typed
        .today
        .ok_or_else(|| ReportsError::Data("the report has no today".to_owned()))?;
    // The day the report is as of; today when none was given.
    let as_of_iso = typed.as_of.clone().unwrap_or_else(|| today.to_iso());
    let as_of = BusinessDate::parse_iso(&as_of_iso)
        .ok_or_else(|| ReportsError::Data("the report day is not a date".to_owned()))?;

    let wanted: Vec<i64> = typed.party_id.into_iter().collect();
    let persons: HashMap<i64, PersonFacts> = open_lines::persons(conn, &wanted)?;
    // A picked account that is not under control 0103 has no loans: the report is empty.
    if persons.is_empty() && !wanted.is_empty() {
        return Ok(with_day(empty(typed, spec), &as_of_iso));
    }
    let ids: Vec<i64> = persons.keys().copied().collect();
    let lines = open_lines::live_lines(conn, &ids)?;
    let grouped = loans_by_person(&lines, &as_of_iso)?;

    let mut totals = [Paisa::ZERO; 5];
    let mut keyed: Vec<(bool, String, i64, PersonRow)> = Vec::new();
    for (account, person) in &persons {
        let Some(group) = grouped.get(account) else {
            continue;
        };
        let (mut lent, mut open) = (Paisa::ZERO, Paisa::ZERO);
        let mut oldest: Option<&Loan<'_>> = None;
        let mut earliest_return_by: Option<BusinessDate> = None;
        let mut overdue = Paisa::ZERO;
        for loan in &group.loans {
            add(&mut lent, paisa(loan.line.dr_paisa)?)?;
            add(&mut open, loan.figures.open)?;
            if loan.figures.open > Paisa::ZERO {
                if oldest.is_none() {
                    oldest = Some(loan);
                }
                if let Some(by) = loan.return_by {
                    earliest_return_by = Some(earliest_return_by.map_or(by, |e| e.min(by)));
                }
                if days_past_return_by(loan.figures.open, loan.return_by, as_of).is_some() {
                    add(&mut overdue, loan.figures.open)?;
                }
            }
        }
        // A person with no loan and no return in the period has no row.
        if lent == Paisa::ZERO
            && group.returned_cash == Paisa::ZERO
            && group.returned_salary == Paisa::ZERO
        {
            continue;
        }
        for (slot, amount) in totals.iter_mut().zip([
            lent,
            group.returned_cash,
            group.returned_salary,
            open,
            overdue,
        ]) {
            add(slot, amount)?;
        }
        let oldest_open = oldest.map(|l| l.line.entry_date.clone());
        keyed.push((
            open == Paisa::ZERO,
            oldest_open.clone().unwrap_or_default(),
            *account,
            PersonRow {
                account_id: *account,
                is_staff: person.staff_id.is_some(),
                lent,
                returned_cash: group.returned_cash,
                returned_salary: group.returned_salary,
                open,
                oldest_open,
                return_by: earliest_return_by.map(BusinessDate::to_iso),
                days_past: days_past_return_by(open, earliest_return_by, as_of),
            },
        ));
    }
    keyed.sort_by(|a, b| (a.0, &a.1, a.2).cmp(&(b.0, &b.1, b.2)));

    let account_ids: Vec<i64> = keyed.iter().map(|k| k.2).collect();
    let parties = load_parties(conn, &account_ids)?;
    let mut all_rows = Vec::with_capacity(keyed.len());
    for (_, _, _, row) in keyed {
        let party = parties
            .get(&row.account_id)
            .cloned()
            .ok_or_else(|| ReportsError::Data("a loan is on an unknown account".to_owned()))?;
        let money = |paisa| Cell::Money { paisa };
        all_rows.push(Row {
            cells: vec![
                Cell::Party { party },
                if row.is_staff {
                    Cell::Label {
                        id: ids::STAFF_YES.to_owned(),
                    }
                } else {
                    Cell::Empty
                },
                money(row.lent),
                money(row.returned_cash),
                money(row.returned_salary),
                money(row.open),
                date_cell(row.oldest_open),
                date_cell(row.return_by),
                row.days_past
                    .map_or(Cell::Empty, |value| Cell::Number { value }),
            ],
            source: Some(RowSource {
                kind: SourceKind::Account,
                id: row.account_id,
            }),
            kind: RowKind::Normal,
        });
    }

    let [lent, returned_cash, returned_salary, open, overdue] = totals;
    let money = |paisa| Cell::Money { paisa };
    let mut page = with_day(empty(typed, spec), &as_of_iso);
    page.rows = spec.slice(&all_rows);
    page.total_count = count_of(all_rows.len());
    page.totals = vec![
        Cell::Empty,
        Cell::Empty,
        money(lent),
        money(returned_cash),
        money(returned_salary),
        money(open),
        Cell::Empty,
        Cell::Empty,
        Cell::Empty,
    ];
    page.summary = vec![SummaryLine {
        label_id: ids::R32_OVERDUE.to_owned(),
        cell: money(overdue),
    }];
    Ok(page)
}

/// The page with the day it is as of in its filters, so the print header says which day it is
/// (also when no day was typed and today was taken).
fn with_day(mut page: ReportPage, as_of: &str) -> ReportPage {
    page.filters.as_of = Some(as_of.to_owned());
    page
}

/// The report with its columns and no rows.
fn empty(typed: &Typed, spec: PageSpec) -> ReportPage {
    ReportPage {
        group_above: None,
        id: ReportId::R32,
        title_id: ids::title_id("R32"),
        filters: typed.resolved.clone(),
        columns: vec![
            col("person", ids::COL_PERSON, ColumnKind::Party),
            col("staff", ids::COL_STAFF, ColumnKind::Label),
            col("lent", ids::COL_LENT, ColumnKind::Money),
            col("returned_cash", ids::COL_RETURNED_CASH, ColumnKind::Money),
            col(
                "returned_salary",
                ids::COL_RETURNED_SALARY,
                ColumnKind::Money,
            ),
            col("open", ids::COL_OPEN, ColumnKind::Money),
            col("oldest_open", ids::COL_OLDEST_DATE, ColumnKind::Date),
            col("return_by", ids::COL_RETURN_BY, ColumnKind::Date),
            col("days_past", ids::COL_DAYS_PAST, ColumnKind::Number),
        ],
        rows: Vec::new(),
        offset: spec.offset,
        limit: spec.reported_limit(),
        total_count: 0,
        totals_label_id: ids::TOTAL.to_owned(),
        totals: vec![Cell::Empty; 9],
        summary: Vec::new(),
    }
}
