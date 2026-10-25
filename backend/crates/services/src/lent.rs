//! ادھار رقم: people who are lent money, their loans and the report R32 (PRD 16.5, plan 17).
//! The saves of a loan and its return are in `vouchers`.
//!
//! The open amount of each loan is worked out at read time, oldest first, from the live lines of
//! the person's account (money agent 8.2b); nothing is stored. Σ open is the account's debit
//! balance. A loan past its return-by day is in the attention data (PRD FR-7.44).

use std::collections::HashMap;

use mandi_db::open_lines::{self, LiveLine, PersonFacts};
use mandi_db::rusqlite::Connection;
use mandi_domain::core::accounts::{AccountType, RecordRef};
use mandi_domain::core::dates::BusinessDate;
use mandi_domain::core::error::ErrorCode;
use mandi_domain::core::lent::{
    AttentionOverdue, LentList, LentListInput, LentPerson, LentPersonCreateInput, LentReport,
    LentReportInput, LentReportRow, LentReportTotals, LentRow, LentTotals, MAX_OVERDUE_ITEMS,
    OverdueItem, OverdueKind,
};
use mandi_domain::core::money::Paisa;
use mandi_domain::core::validate;
use mandi_domain::expenses::{OpenAmount, days_past_return_by, open_amounts};
use serde_json::json;

use crate::accounts;
use crate::audit::Event;
use crate::auth::Actor;
use crate::context::Context;
use crate::day;
use crate::error::{ServiceError, money_error};
use crate::ledger;
use crate::page;
use crate::search;
use crate::ventures::record_of;

fn paisa(value: i64) -> Result<Paisa, ServiceError> {
    Paisa::try_from_paisa(value).map_err(money_error)
}

fn add(sum: &mut Paisa, value: Paisa) -> Result<(), ServiceError> {
    *sum = sum.checked_add(value).map_err(money_error)?;
    Ok(())
}

fn person_ref(person: &PersonFacts) -> RecordRef {
    RecordRef {
        id: person.account_id,
        code: person.code.clone(),
        name_ur: person.name_ur.clone(),
        name_en: person.name_en.clone(),
        tag: None,
    }
}

fn lent_person(person: &PersonFacts) -> LentPerson {
    LentPerson {
        account: person_ref(person),
        staff: person
            .staff_id
            .zip(person.staff_names.as_ref())
            .map(|(id, (ur, en))| record_of(id, ur, en)),
        phone: person.phone.clone(),
    }
}

/// Makes the account of a person who can be lent money (control 0103), in one transaction. When
/// he is a staff member, his staff record is linked to the account in the same transaction
/// (PRD FR-16.4). A staff member who already has a person account is refused: his account is the
/// one to use.
///
/// # Errors
/// The name codes, `ERR_SPELLING_UNCONFIRMED`, the phone code on `phone`, `ERR_NOT_FOUND` on
/// `staff_id`, `ERR_FORMAT_INVALID` on `staff_id` for a staff member who has an account already.
/// Nothing is written when any of them is returned.
pub fn person_create(
    ctx: &Context,
    actor: &Actor,
    input: &LentPersonCreateInput,
) -> Result<LentPerson, ServiceError> {
    let window = day::window(ctx)?;
    let account_input = mandi_domain::core::accounts::AccountInput {
        id: None,
        version: None,
        account_type: AccountType::MoneyLent,
        control_id: None,
        code: String::new(),
        name_ur: input.name_ur.clone(),
        name_en: input.name_en.clone(),
        confirmed_rules_words: input.confirmed_rules_words.clone(),
        father_or_firm_ur: String::new(),
        father_or_firm_en: String::new(),
        city_id: None,
        adda_id: None,
        phone1: input.phone.clone().unwrap_or_default(),
        phone2: String::new(),
        address: String::new(),
        linked_account_id: None,
        opening_date: String::new(),
        opening_dr: String::new(),
        opening_cr: String::new(),
        opening_is_advance: false,
        credit_allowed: true,
        credit_limit: String::new(),
        default_marka_id: None,
        collector_staff_id: None,
        collection_weekday: None,
        is_active: true,
        note: String::new(),
        tag: String::new(),
    };
    let (id, learned) = ctx
        .db
        .write(|tx| {
            if let Some(staff_id) = input.staff_id {
                match open_lines::staff(tx, staff_id)? {
                    None => {
                        return Err(ServiceError::reject_field(
                            ErrorCode::ErrNotFound,
                            "staff_id",
                        ));
                    }
                    Some(staff) if staff.account_id.is_some() => {
                        return Err(ServiceError::reject_field(
                            ErrorCode::ErrFormatInvalid,
                            "staff_id",
                        ));
                    }
                    Some(_) => {}
                }
            }
            let (id, learned) = accounts::create_in(ctx, actor, tx, &account_input, window)?;
            if let Some(staff_id) = input.staff_id {
                open_lines::link_staff_account(tx, staff_id, id, actor.user_id, &ctx.now_text())?;
                ctx.db.auditor().write(
                    tx,
                    &Event::new(Some(actor.user_id), "update", "staff")
                        .entity_id(staff_id)
                        .new_value(json!({ "account_id": id }))
                        .into_entry(),
                )?;
            }
            Ok((id, learned))
        })
        .map_err(phone_field)?;
    for learned in learned {
        learned.after_commit(ctx);
    }
    search::refresh_account(ctx, id);
    ctx.db.read(|conn| {
        open_lines::persons(conn, &[id])?
            .remove(&id)
            .map(|person| lent_person(&person))
            .ok_or_else(|| ServiceError::Data("the new person cannot be read back".to_owned()))
    })
}

/// The account form calls the phone `phone1`; this screen calls it `phone`.
fn phone_field(error: ServiceError) -> ServiceError {
    match error {
        ServiceError::Rejected(mut items) => {
            for item in &mut items {
                if item.field.as_deref() == Some("phone1") {
                    item.field = Some("phone".to_owned());
                }
            }
            ServiceError::Rejected(items)
        }
        other => other,
    }
}

/// A loan (or an opening debit) of the list, with what was returned of it.
struct Loan<'a> {
    line: &'a LiveLine,
    figures: OpenAmount,
    return_by: Option<BusinessDate>,
}

/// One account's loans with what its credits returned, split into cash and salary.
struct Loans<'a> {
    /// The loans, oldest first.
    loans: Vec<Loan<'a>>,
    /// Σ credits that are not salary deductions.
    returned_cash: Paisa,
    /// Σ credits that are salary deductions.
    returned_salary: Paisa,
}

/// The loans of each person with their open amounts, oldest first. `as_of` (`YYYY-MM-DD`,
/// included) leaves out the lines dated after it. One read of the lines of all the accounts.
fn loans_by_person<'a>(
    lines: &'a [LiveLine],
    as_of: Option<&str>,
) -> Result<HashMap<i64, Loans<'a>>, ServiceError> {
    let mut by_account: HashMap<i64, Vec<&'a LiveLine>> = HashMap::new();
    for line in lines
        .iter()
        .filter(|l| as_of.is_none_or(|day| l.entry_date.as_str() <= day))
    {
        by_account.entry(line.account_id).or_default().push(line);
    }
    let mut out = HashMap::new();
    for (account, own) in by_account {
        let bills = own
            .iter()
            .filter(|l| l.dr_paisa > 0)
            .map(|l| Ok((l.line_id, paisa(l.dr_paisa)?)))
            .collect::<Result<Vec<_>, ServiceError>>()?;
        let mut credit_cash = Paisa::ZERO;
        let mut credit_salary = Paisa::ZERO;
        let mut credits = Vec::new();
        for line in own.iter().filter(|l| l.cr_paisa > 0) {
            let amount = paisa(line.cr_paisa)?;
            credits.push((line.line_id, amount));
            // Any credit that is not a salary deduction counts as cash (a جنرل credit, an
            // opening credit).
            add(
                if line.salary {
                    &mut credit_salary
                } else {
                    &mut credit_cash
                },
                amount,
            )?;
        }
        let figures: HashMap<i64, OpenAmount> = open_amounts(&bills, &credits)
            .map_err(money_error)?
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
                returned_cash: credit_cash,
                returned_salary: credit_salary,
            },
        );
    }
    Ok(out)
}

/// One person's loans, oldest first, one page (PRD FR-16.5). An opening debit is a loan dated
/// its opening date; its row has `voucher_id` and `number` 0.
///
/// # Errors
/// `ERR_NOT_FOUND` on `person_id`.
pub fn list(ctx: &Context, input: &LentListInput) -> Result<LentList, ServiceError> {
    let (_, today) = day::window(ctx)?;
    ctx.db.read(|conn| {
        let person = open_lines::persons(conn, &[input.person_id])?
            .remove(&input.person_id)
            .ok_or_else(|| ServiceError::reject_field(ErrorCode::ErrNotFound, "person_id"))?;
        let lines = open_lines::live_lines(conn, &[person.account_id])?;
        let mut grouped = loans_by_person(&lines, None)?;
        let loans = grouped
            .remove(&person.account_id)
            .map_or_else(Vec::new, |g| g.loans);
        let mut totals = LentTotals {
            lent_paisa: Paisa::ZERO,
            returned_paisa: Paisa::ZERO,
            open_paisa: Paisa::ZERO,
        };
        let mut rows = Vec::with_capacity(loans.len());
        for loan in &loans {
            let amount = paisa(loan.line.dr_paisa)?;
            add(&mut totals.lent_paisa, amount)?;
            add(&mut totals.returned_paisa, loan.figures.returned)?;
            add(&mut totals.open_paisa, loan.figures.open)?;
            rows.push(LentRow {
                voucher_id: loan.line.voucher_id.unwrap_or(0),
                number: loan.line.number.unwrap_or(0),
                entry_date: loan.line.entry_date.clone(),
                amount_paisa: amount,
                reason: loan.line.reason.clone(),
                return_by: loan.line.return_by.clone(),
                returned_paisa: loan.figures.returned,
                open_paisa: loan.figures.open,
                days_past_return_by: days_past_return_by(loan.figures.open, loan.return_by, today),
            });
        }
        let total_count = i64::try_from(rows.len()).unwrap_or(i64::MAX);
        Ok(LentList {
            person: lent_person(&person),
            balance: ledger::balance_in(conn, person.account_id, None, None)?,
            rows: page::slice(rows, input.offset, input.limit),
            total_count,
            totals,
        })
    })
}

/// R32: everyone who was lent money, one page (PRD FR-16.5, FR-19.10). Those with something open
/// come first, the oldest open loan first; then the settled ones. `as_of` leaves out entries
/// dated after it; without it the report is as of today. The totals are over every person.
///
/// # Errors
/// The date codes on `as_of`.
pub fn report(ctx: &Context, input: &LentReportInput) -> Result<LentReport, ServiceError> {
    let (_, today) = day::window(ctx)?;
    let as_of = match input
        .as_of
        .as_deref()
        .map(str::trim)
        .filter(|t| !t.is_empty())
    {
        Some(text) => validate::date("as_of", text, today)?,
        None => today,
    };
    let as_of_iso = as_of.to_iso();
    ctx.db.read(|conn| {
        let (rows, totals) = report_rows(conn, &as_of_iso, as_of)?;
        let total_count = i64::try_from(rows.len()).unwrap_or(i64::MAX);
        Ok(LentReport {
            as_of: as_of_iso,
            rows: page::slice(rows, input.offset, input.limit),
            total_count,
            totals,
        })
    })
}

/// Every person's row and the totals over all of them.
fn report_rows(
    conn: &Connection,
    as_of_iso: &str,
    as_of: BusinessDate,
) -> Result<(Vec<LentReportRow>, LentReportTotals), ServiceError> {
    let persons = open_lines::persons(conn, &[])?;
    let ids: Vec<i64> = persons.keys().copied().collect();
    let lines = open_lines::live_lines(conn, &ids)?;
    let grouped = loans_by_person(&lines, Some(as_of_iso))?;
    let mut totals = LentReportTotals {
        lent_paisa: Paisa::ZERO,
        returned_cash_paisa: Paisa::ZERO,
        returned_salary_paisa: Paisa::ZERO,
        open_paisa: Paisa::ZERO,
        overdue_paisa: Paisa::ZERO,
    };
    let mut keyed: Vec<(bool, String, i64, LentReportRow)> = Vec::new();
    for (account, person) in &persons {
        let Some(Loans {
            loans,
            returned_cash,
            returned_salary,
        }) = grouped.get(account)
        else {
            continue;
        };
        let mut lent = Paisa::ZERO;
        let mut open = Paisa::ZERO;
        let mut oldest: Option<&Loan<'_>> = None;
        let mut earliest_return_by: Option<BusinessDate> = None;
        for loan in loans {
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
                    add(&mut totals.overdue_paisa, loan.figures.open)?;
                }
            }
        }
        if lent == Paisa::ZERO && returned_cash == &Paisa::ZERO && returned_salary == &Paisa::ZERO {
            continue;
        }
        add(&mut totals.lent_paisa, lent)?;
        add(&mut totals.returned_cash_paisa, *returned_cash)?;
        add(&mut totals.returned_salary_paisa, *returned_salary)?;
        add(&mut totals.open_paisa, open)?;
        let oldest_date = oldest.map(|l| l.line.entry_date.clone());
        keyed.push((
            open == Paisa::ZERO,
            oldest_date.clone().unwrap_or_default(),
            *account,
            LentReportRow {
                person: person_ref(person),
                is_staff: person.staff_id.is_some(),
                lent_paisa: lent,
                returned_cash_paisa: *returned_cash,
                returned_salary_paisa: *returned_salary,
                open_paisa: open,
                oldest_open_date: oldest_date,
                return_by: earliest_return_by.map(BusinessDate::to_iso),
                days_past_return_by: days_past_return_by(open, earliest_return_by, as_of),
            },
        ));
    }
    keyed.sort_by(|a, b| (a.0, &a.1, a.2).cmp(&(b.0, &b.1, b.2)));
    Ok((keyed.into_iter().map(|k| k.3).collect(), totals))
}

/// The overdue loans and partner takes: open amounts past their return-by day, most overdue first,
/// at most [`MAX_OVERDUE_ITEMS`] (PRD FR-7.44). Read only.
///
/// # Errors
/// `ServiceError::Db`.
pub fn attention_overdue(ctx: &Context) -> Result<AttentionOverdue, ServiceError> {
    let (_, today) = day::window(ctx)?;
    ctx.db.read(|conn| {
        let persons = open_lines::persons(conn, &[])?;
        let partners = mandi_db::partners::list(conn)?;
        let mut ids: Vec<i64> = persons.keys().copied().collect();
        ids.extend(partners.iter().map(|p| p.drawings_account_id));
        let lines = open_lines::live_lines(conn, &ids)?;
        let grouped = loans_by_person(&lines, None)?;
        let by_drawings: HashMap<i64, &mandi_db::partners::PartnerRecord> = partners
            .iter()
            .map(|p| (p.drawings_account_id, p))
            .collect();
        let mut items = Vec::new();
        for (account, Loans { loans, .. }) in &grouped {
            let (kind, who) = match (persons.get(account), by_drawings.get(account)) {
                (Some(person), _) => (OverdueKind::Loan, person_ref(person)),
                (None, Some(partner)) => (
                    OverdueKind::Take,
                    record_of(partner.id, &partner.name_ur, &partner.name_en),
                ),
                (None, None) => continue,
            };
            for loan in loans {
                let (Some(voucher_id), Some(number), Some(return_by)) = (
                    loan.line.voucher_id,
                    loan.line.number,
                    loan.line.return_by.clone(),
                ) else {
                    continue;
                };
                let Some(days_past) = days_past_return_by(loan.figures.open, loan.return_by, today)
                else {
                    continue;
                };
                items.push(OverdueItem {
                    kind,
                    who: who.clone(),
                    voucher_id,
                    number,
                    entry_date: loan.line.entry_date.clone(),
                    return_by,
                    open_paisa: loan.figures.open,
                    days_past,
                });
            }
        }
        items.sort_by(|a, b| {
            b.days_past
                .cmp(&a.days_past)
                .then(a.entry_date.cmp(&b.entry_date))
                .then(a.voucher_id.cmp(&b.voucher_id))
        });
        let total_count = i64::try_from(items.len()).unwrap_or(i64::MAX);
        let truncated = items.len() > MAX_OVERDUE_ITEMS;
        items.truncate(MAX_OVERDUE_ITEMS);
        Ok(AttentionOverdue {
            items,
            total_count,
            truncated,
        })
    })
}
