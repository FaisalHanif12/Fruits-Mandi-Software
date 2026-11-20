//! Money lent and partner money taken, for the tile ادھار رقم باقی and the two attention items
//! (PRD FR-16.4, FR-16.5, FR-7.44, money agent 8.2b). The open amount of each loan is the one of
//! R32: worked at read time, oldest first, from the live lines of the account
//! (`business::lent::loans_by_person`), so the tile, R32 and the bell can never differ.
//!
//! A loan is overdue only when it has a return-by day, the day has passed and something is still
//! open (`days_past_return_by`, E2E-30b). A drawing with no return-by day is never overdue.
//!
//! Example (E2E-30b, today 16-10-26): اسلم was lent 5,000 with return by 15-10-26 and returned
//! 3,000. Open 2,000, overdue 2,000, 1 day past.

use mandi_core::dates::BusinessDate;
use mandi_core::money::Paisa;
use mandi_db::open_lines;
use mandi_db::rusqlite::Connection;

use crate::error::ReportsError;
use crate::reports::business::lent::{Loans, loans_by_person};
use crate::reports::common::add;
use crate::reports::workings::days_past_return_by;

/// One loan or take that is past its return-by day.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Overdue {
    /// The person's account (a loan) or the partner (a take).
    pub(crate) who_id: i64,
    /// Urdu name.
    pub(crate) name_ur: String,
    /// English name.
    pub(crate) name_en: String,
    /// Still open.
    pub(crate) open: Paisa,
    /// Days past the return-by day. 1 or more.
    pub(crate) days_past: i64,
}

/// One person who borrowed and still owes (plan 79 item 2b): the row of the page ادھار رقم باقی.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PersonOpen {
    /// The person's ادھار رقم account.
    pub(crate) account_id: i64,
    /// His type word as stored (`staff`, `garden_owner`, `trip_person`, `other`).
    pub(crate) person_type: String,
    /// What he still owes: Σ open of his loans. More than 0.
    pub(crate) open: Paisa,
    /// The day of his oldest loan that still has something open, `YYYY-MM-DD`.
    pub(crate) oldest_open_date: Option<String>,
    /// How much of it is past its return-by day.
    pub(crate) overdue: Paisa,
}

/// What is lent and what is overdue.
#[derive(Debug, Clone, Default)]
pub(crate) struct LentFacts {
    /// Every person with something open, in the order of his account id. Their opens add up to
    /// `open`, and the people not in the dead amount only.
    pub(crate) people: Vec<PersonOpen>,
    /// Σ open of the money lent to persons (control 0103). Partner takes are not in it.
    pub(crate) open: Paisa,
    /// Σ open of the loans past their return-by day.
    pub(crate) overdue: Paisa,
    /// Σ open of the money lent to persons who are not staff: garden owners, market-trip people
    /// and others (plan 76 point 10, PRD FR-16.13). Part of `open`.
    pub(crate) others_open: Paisa,
    /// How many of those persons have something open.
    pub(crate) others_people: i64,
    /// The overdue loans, most days past first.
    pub(crate) loans: Vec<Overdue>,
    /// The overdue partner takes, most days past first.
    pub(crate) takes: Vec<Overdue>,
}

/// Reads the loans and the takes as of today.
///
/// # Errors
/// A database error; `ReportsError::Money` if a stored amount is not whole rupees or a sum does
/// not fit.
pub(crate) fn facts(conn: &Connection, today: BusinessDate) -> Result<LentFacts, ReportsError> {
    let mut persons = open_lines::persons(conn, &[])?;
    // A person in the dead amount is in R38 and the tile ڈوبی رقم, not in the money lent (PRD
    // FR-10.10, A66).
    let dead = mandi_db::dead::dead_set(conn, None)?;
    persons.retain(|id, _| !dead.has_account(*id));
    let partners = mandi_db::partners::list(conn)?;
    let mut ids: Vec<i64> = persons.keys().copied().collect();
    ids.extend(partners.iter().map(|p| p.drawings_account_id));
    let lines = open_lines::live_lines(conn, &ids)?;
    let grouped = loans_by_person(&lines, &today.to_iso())?;
    let by_drawings: std::collections::HashMap<i64, &mandi_db::partners::PartnerRecord> = partners
        .iter()
        .map(|p| (p.drawings_account_id, p))
        .collect();
    let mut out = LentFacts::default();
    for (account, Loans { loans, .. }) in &grouped {
        let person = persons.get(account);
        let take = by_drawings.get(account);
        if let Some(p) = person {
            let (mut open, mut overdue) = (Paisa::ZERO, Paisa::ZERO);
            let mut oldest: Option<&str> = None;
            for loan in loans {
                add(&mut open, loan.figures.open)?;
                if loan.figures.open > Paisa::ZERO {
                    oldest = oldest.or(Some(loan.line.entry_date.as_str()));
                    if days_past_return_by(loan.figures.open, loan.return_by, today).is_some() {
                        add(&mut overdue, loan.figures.open)?;
                    }
                }
            }
            if open > Paisa::ZERO {
                out.people.push(PersonOpen {
                    account_id: *account,
                    person_type: p.person_type.clone(),
                    open,
                    oldest_open_date: oldest.map(str::to_owned),
                    overdue,
                });
            }
        }
        if person.is_some_and(|p| p.person_type != "staff") {
            let mut open = Paisa::ZERO;
            for loan in loans {
                add(&mut open, loan.figures.open)?;
            }
            if open > Paisa::ZERO {
                add(&mut out.others_open, open)?;
                out.others_people = out.others_people.checked_add(1).ok_or_else(|| {
                    ReportsError::Data("the number of people does not fit".to_owned())
                })?;
            }
        }
        for loan in loans {
            if person.is_some() {
                add(&mut out.open, loan.figures.open)?;
            }
            let Some(days_past) = days_past_return_by(loan.figures.open, loan.return_by, today)
            else {
                continue;
            };
            let (row, is_loan) = match (person, take) {
                (Some(p), _) => (
                    Overdue {
                        who_id: *account,
                        name_ur: p.name_ur.clone(),
                        name_en: p.name_en.clone(),
                        open: loan.figures.open,
                        days_past,
                    },
                    true,
                ),
                (None, Some(partner)) => (
                    Overdue {
                        who_id: partner.id,
                        name_ur: partner.name_ur.clone(),
                        name_en: partner.name_en.clone(),
                        open: loan.figures.open,
                        days_past,
                    },
                    false,
                ),
                (None, None) => continue,
            };
            if is_loan {
                add(&mut out.overdue, row.open)?;
                out.loans.push(row);
            } else {
                out.takes.push(row);
            }
        }
    }
    let by_age = |a: &Overdue, b: &Overdue| {
        b.days_past
            .cmp(&a.days_past)
            .then(b.open.cmp(&a.open))
            .then(a.who_id.cmp(&b.who_id))
    };
    out.people.sort_by_key(|p| p.account_id);
    out.loans.sort_by(by_age);
    out.takes.sort_by(by_age);
    Ok(out)
}

/// The open money lent to people not in the dead amount as of today: the tile ادھار رقم باقی and
/// the fourth part of کل اگراہی.
///
/// # Errors
/// As [`facts`].
pub(crate) fn open_now(conn: &Connection, today: BusinessDate) -> Result<Paisa, ReportsError> {
    Ok(facts(conn, today)?.open)
}
