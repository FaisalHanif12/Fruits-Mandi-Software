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
    AttentionOverdue, LentDues, LentDuesInput, LentDuesPerson, LentDuesRow, LentDuesRowKind,
    LentDuesTotals, LentList, LentListInput, LentPerson, LentPersonCreateInput,
    LentPersonCreateRequest, LentPersonUpdateInput, LentReport, LentReportInput, LentReportRow,
    LentReportTotals, LentRow, LentTotals, MAX_OVERDUE_ITEMS, OverdueItem, OverdueKind,
};
use mandi_domain::core::links::{Link, LinkKind, PersonType};
use mandi_domain::core::money::Paisa;
use mandi_domain::core::months::Month;
use mandi_domain::core::paging::SortDirection;
use mandi_domain::core::text;
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
use crate::linked;
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
        tag: person.tag.clone(),
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
        person_type: PersonType::parse(&person.person_type).unwrap_or(PersonType::Other),
        link: person_link(&person.link_kind, person.link_partner_id),
    }
}

/// The link of a person who is lent money. A loan with no link belongs to the shop, as it always
/// did, so a stored `none` reads as the shop (plan 76 point 10, PRD FR-16.13).
fn person_link(kind: &str, partner_id: Option<i64>) -> Link {
    match linked::link_from_stored(kind, partner_id) {
        Link {
            kind: LinkKind::None,
            ..
        } => Link {
            kind: LinkKind::Shop,
            partner_id: None,
        },
        other => other,
    }
}

/// Checks the link of a person: a partner link needs an active partner; the shop, and no link,
/// both mean the shop and need nothing (a shop-linked loan posts as before, so the shares are not
/// needed until a statement splits its note, PRD FR-16.13).
///
/// # Errors
/// `ERR_LINK_PARTNER_REQUIRED`, `ERR_NOT_FOUND` and `ERR_PARTNER_INACTIVE` on `link.partner_id`;
/// `ERR_FORMAT_INVALID` on `link.partner_id` for a shop link or no link that names a partner.
pub(crate) fn check_person_link(
    tx: &Connection,
    link: &Link,
    shares_day: &str,
) -> Result<linked::CheckedLink, ServiceError> {
    linked::refuse_stray_partner(link)?;
    match link.kind {
        LinkKind::Partner => linked::check_link(tx, link, shares_day),
        LinkKind::None | LinkKind::Shop => Ok(linked::CheckedLink {
            kind: "shop",
            partner_id: None,
        }),
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
    person_create_with(ctx, actor, input, None, None)
}

/// Makes a person with his type and link (plan 76 point 10, PRD FR-16.13), in one transaction. No
/// type means staff when he is a staff member, else other; no link means the shop.
///
/// # Errors
/// As [`person_create`], with `ERR_LINK_PARTNER_REQUIRED`, `ERR_NOT_FOUND` and
/// `ERR_PARTNER_INACTIVE` on `link.partner_id`.
pub fn person_create_linked(
    ctx: &Context,
    actor: &Actor,
    request: &LentPersonCreateRequest,
) -> Result<LentPerson, ServiceError> {
    person_create_with(
        ctx,
        actor,
        &request.base,
        request.person_type,
        request.link.as_ref(),
    )
}

fn person_create_with(
    ctx: &Context,
    actor: &Actor,
    input: &LentPersonCreateInput,
    person_type: Option<PersonType>,
    link: Option<&Link>,
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
        is_active: true,
        note: String::new(),
        tag: String::new(),
    };
    let (id, learned) = ctx
        .db
        .write(|tx| {
            // Checked first, so a bad link writes nothing.
            let checked_link = check_person_link(
                tx,
                link.unwrap_or(&Link {
                    kind: LinkKind::Shop,
                    partner_id: None,
                }),
                &window.1.to_iso(),
            )?;
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
            let kind = person_type.unwrap_or(if input.staff_id.is_some() {
                PersonType::Staff
            } else {
                PersonType::Other
            });
            mandi_db::links::set_person_type(tx, id, kind.as_str())?;
            linked::apply_link(
                ctx,
                actor,
                tx,
                id,
                &checked_link,
                ErrorCode::ErrLinkFixedWhileOwed,
            )?;
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

/// Changes the type and the link of a person (plan 76 point 10, PRD FR-16.13), in one transaction.
/// The type is a label and changes any time; the link changes only while he owes nothing.
///
/// # Errors
/// `ERR_NOT_FOUND` on `person_account_id`; `ERR_LINK_FIXED_WHILE_OWED` on `link`;
/// `ERR_LINK_PARTNER_REQUIRED`, `ERR_NOT_FOUND` and `ERR_PARTNER_INACTIVE` on `link.partner_id`.
pub fn person_update(
    ctx: &Context,
    actor: &Actor,
    input: &LentPersonUpdateInput,
) -> Result<LentPerson, ServiceError> {
    let (_, today) = day::window(ctx)?;
    let id = input.person_account_id;
    ctx.db.write(|tx| -> Result<(), ServiceError> {
        let person = open_lines::persons(tx, &[id])?.remove(&id).ok_or_else(|| {
            ServiceError::reject_field(ErrorCode::ErrNotFound, "person_account_id")
        })?;
        if let Some(kind) = input.person_type
            && kind.as_str() != person.person_type
        {
            mandi_db::links::set_person_type(tx, id, kind.as_str())?;
            ctx.db.auditor().write(
                tx,
                &Event::new(Some(actor.user_id), "update", "account")
                    .entity_id(id)
                    .old(json!({ "person_type": person.person_type }))
                    .new_value(json!({ "person_type": kind.as_str() }))
                    .into_entry(),
            )?;
        }
        if let Some(link) = &input.link {
            let checked = check_person_link(tx, link, &today.to_iso())?;
            linked::apply_link(
                ctx,
                actor,
                tx,
                id,
                &checked,
                ErrorCode::ErrLinkFixedWhileOwed,
            )?;
        }
        Ok(())
    })?;
    ctx.db.read(|conn| {
        open_lines::persons(conn, &[id])?
            .remove(&id)
            .map(|person| lent_person(&person))
            .ok_or_else(|| ServiceError::reject_field(ErrorCode::ErrNotFound, "person_account_id"))
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
    let (from, to) = page::date_range(input.from.as_ref(), input.to.as_ref(), today)?;
    // The sort is checked first: an unknown column is refused (plan 38 R4).
    if let Some(sort) = &input.sort {
        sort.check(&LOAN_COLUMNS)
            .map_err(|e| ServiceError::reject_field(e.code, &e.field))?;
    }
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
        let mut month_acc = page::MonthAcc::new(3);
        for loan in &loans {
            // A fully returned loan is finished: it steps aside, with its money, unless asked for
            // (plan 38 R5). The totals and the months are over the rows listed.
            if !input.show_finished && loan.figures.open == Paisa::ZERO {
                continue;
            }
            // The range keeps the loans dated inside it (plan 76 point 8); what has been returned
            // against a loan is still its true figure, whenever the return was made.
            if !page::in_range(&loan.line.entry_date, from.as_deref(), to.as_deref()) {
                continue;
            }
            let amount = paisa(loan.line.dr_paisa)?;
            add(&mut totals.lent_paisa, amount)?;
            add(&mut totals.returned_paisa, loan.figures.returned)?;
            add(&mut totals.open_paisa, loan.figures.open)?;
            month_acc.add(
                &loan.line.entry_date,
                &[
                    amount.as_paisa(),
                    loan.figures.returned.as_paisa(),
                    loan.figures.open.as_paisa(),
                ],
            )?;
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
        if let Some(sort) = &input.sort {
            sort_loans(&mut rows, &sort.column, sort.direction);
        }
        let in_date_order = input.sort.as_ref().is_none_or(|s| s.column == "entry_date");
        let newest_first = input
            .sort
            .as_ref()
            .is_some_and(|s| s.column == "entry_date" && s.direction == SortDirection::Desc);
        let months = if input.group_by_month && in_date_order {
            month_acc.finish(
                &[
                    ("lent_paisa", true),
                    ("returned_paisa", true),
                    ("open_paisa", true),
                ],
                newest_first,
            )
        } else {
            Vec::new()
        };
        Ok(LentList {
            person: lent_person(&person),
            balance: ledger::balance_in(conn, person.account_id, None, None)?,
            rows: page::slice(rows, input.offset, input.limit),
            total_count,
            totals,
            months,
        })
    })
}

/// The types the dashboard tile and the dues page show when none is chosen: everyone but staff
/// (plan 76 point 10, PRD FR-16.13).
const DEFAULT_DUES_TYPES: [PersonType; 3] = [
    PersonType::GardenOwner,
    PersonType::TripPerson,
    PersonType::Other,
];

/// The page behind the tile of garden owners and others, one month at a time (plan 76 point 10,
/// PRD FR-16.13, FR-19.11): each amount given and each return of the month with the person, his
/// type and his link, the totals of the month, and the people with their open amounts as of today.
/// People in the dead amount are left out, as on the tile, so with the default filter the open
/// total equals `others_dues_paisa` of the dashboard. Read only.
///
/// Example: نواز (garden owner, linked to الف) was given 20,000 on 01-11-26 and returned 5,000 on
/// 10-11-26. For November: given 20,000, returned 5,000, open 15,000, 1 person.
///
/// # Errors
/// `ERR_FORMAT_INVALID` on `month`; `ERR_DATE_IN_FUTURE` on `month` for a month after this one.
pub fn dues(ctx: &Context, input: &LentDuesInput) -> Result<LentDues, ServiceError> {
    let (_, today) = day::window(ctx)?;
    let current = Month::of(today);
    let month = match input
        .month
        .as_deref()
        .map(str::trim)
        .filter(|m| !m.is_empty())
    {
        None => current,
        Some(text) => Month::parse(text)
            .ok_or_else(|| ServiceError::reject_field(ErrorCode::ErrFormatInvalid, "month"))?,
    };
    if month > current {
        return Err(ServiceError::reject_field(
            ErrorCode::ErrDateInFuture,
            "month",
        ));
    }
    let (Some(first), Some(last)) = (month.first_day(), month.last_day()) else {
        return Err(ServiceError::reject_field(
            ErrorCode::ErrFormatInvalid,
            "month",
        ));
    };
    let (from, to) = (first.to_iso(), last.to_iso());
    let types: &[PersonType] = if input.person_types.is_empty() {
        &DEFAULT_DUES_TYPES
    } else {
        &input.person_types
    };
    // Which partner a loan is linked to is a partner fact (PRD FR-17.11, NFR-SEC-14). While the
    // section is locked, a filter by partner is refused and a partner link carries no partner.
    let locked = crate::partner_code::is_locked(ctx)?;
    if locked && input.partner_id.is_some() {
        return Err(ServiceError::reject_field(
            ErrorCode::ErrPartnerLocked,
            "partner_id",
        ));
    }
    ctx.db.read(|conn| {
        let dead = mandi_db::dead::dead_set(conn, None)?;
        let mut persons = open_lines::persons(conn, &[])?;
        persons.retain(|id, p| {
            let link = person_link(&p.link_kind, p.link_partner_id);
            PersonType::parse(&p.person_type).is_some_and(|t| types.contains(&t))
                && !dead.has_account(*id)
                && (input.link_kinds.is_empty() || input.link_kinds.contains(&link.kind))
                && input
                    .partner_id
                    .is_none_or(|partner| link.partner_id == Some(partner))
        });
        let ids: Vec<i64> = persons.keys().copied().collect();
        let lines = open_lines::live_lines(conn, &ids)?;
        let grouped = loans_by_person(&lines, None)?;
        let mut rows: Vec<(i64, i64, LentDuesRow)> = Vec::new();
        let mut totals = LentDuesTotals {
            given_paisa: Paisa::ZERO,
            returned_paisa: Paisa::ZERO,
            open_paisa: Paisa::ZERO,
            open_people_count: 0,
        };
        let mut people: Vec<LentDuesPerson> = Vec::new();
        for (account, person) in &persons {
            let mut link = person_link(&person.link_kind, person.link_partner_id);
            if locked && link.kind == LinkKind::Partner {
                link.partner_id = None;
            }
            let person_type = PersonType::parse(&person.person_type).unwrap_or(PersonType::Other);
            let (mut lent, mut returned, mut open) = (Paisa::ZERO, Paisa::ZERO, Paisa::ZERO);
            if let Some(loans) = grouped.get(account) {
                for loan in &loans.loans {
                    add(&mut lent, paisa(loan.line.dr_paisa)?)?;
                    add(&mut returned, loan.figures.returned)?;
                    add(&mut open, loan.figures.open)?;
                }
            }
            let mut in_month = false;
            for line in lines.iter().filter(|l| l.account_id == *account) {
                if line.entry_date.as_str() < from.as_str()
                    || line.entry_date.as_str() > to.as_str()
                {
                    continue;
                }
                in_month = true;
                let (kind, amount) = if line.dr_paisa > 0 {
                    (LentDuesRowKind::Given, paisa(line.dr_paisa)?)
                } else {
                    (LentDuesRowKind::Returned, paisa(line.cr_paisa)?)
                };
                add(
                    if kind == LentDuesRowKind::Given {
                        &mut totals.given_paisa
                    } else {
                        &mut totals.returned_paisa
                    },
                    amount,
                )?;
                rows.push((
                    line.entry_id,
                    line.line_id,
                    LentDuesRow {
                        kind,
                        entry_date: line.entry_date.clone(),
                        person: person_ref(person),
                        person_type,
                        link,
                        voucher_id: line.voucher_id,
                        number: line.number,
                        reason: line.reason.clone(),
                        amount_paisa: amount,
                    },
                ));
            }
            if open > Paisa::ZERO {
                add(&mut totals.open_paisa, open)?;
                totals.open_people_count += 1;
            }
            if open > Paisa::ZERO || in_month {
                people.push(LentDuesPerson {
                    person: person_ref(person),
                    person_type,
                    link,
                    lent_paisa: lent,
                    returned_paisa: returned,
                    open_paisa: open,
                });
            }
        }
        rows.sort_by(|a, b| {
            (a.2.entry_date.as_str(), a.0, a.1).cmp(&(b.2.entry_date.as_str(), b.0, b.1))
        });
        let total_count = i64::try_from(rows.len()).unwrap_or(i64::MAX);
        let rows: Vec<LentDuesRow> = rows.into_iter().map(|(_, _, row)| row).collect();
        people.sort_by(|a, b| {
            b.open_paisa
                .cmp(&a.open_paisa)
                .then_with(|| a.person.code.cmp(&b.person.code))
        });
        let people_total_count = i64::try_from(people.len()).unwrap_or(i64::MAX);
        Ok(LentDues {
            month: month.key(),
            rows: page::slice(rows, input.offset, input.limit),
            total_count,
            totals,
            people: page::slice(people, input.people_offset, input.people_limit),
            people_total_count,
            partner_locked: locked,
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
    // The sort is checked first: an unknown column is refused (plan 38 R4).
    if let Some(sort) = &input.sort {
        sort.check(&REPORT_COLUMNS)
            .map_err(|e| ServiceError::reject_field(e.code, &e.field))?;
    }
    let search = input
        .search
        .as_deref()
        .map(str::trim)
        .filter(|t| !t.is_empty());
    ctx.db.read(|conn| {
        let (mut rows, totals) = report_rows(conn, &as_of_iso, as_of, search)?;
        if let Some(sort) = &input.sort {
            sort_report(&mut rows, &sort.column, sort.direction);
        }
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
    search: Option<&str>,
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
        // A search keeps only the people it names, before anything is added to the totals.
        if let Some(text) = search {
            let typed_ur = text::search_ur(text);
            let typed_en = text::search_en(text);
            let found = person.code.to_lowercase().contains(&text.to_lowercase())
                || (!typed_ur.is_empty() && text::search_ur(&person.name_ur).contains(&typed_ur))
                || (!typed_en.is_empty() && text::search_en(&person.name_en).contains(&typed_en));
            if !found {
                continue;
            }
        }
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

/// The columns the list of one person's loans can be sorted by: the names of the fields of the row.
const LOAN_COLUMNS: [&str; 8] = [
    "entry_date",
    "number",
    "amount_paisa",
    "reason",
    "return_by",
    "returned_paisa",
    "open_paisa",
    "days_past_return_by",
];

/// Sorts the whole list by a column (checked by [`LOAN_COLUMNS`]). A stable sort, so equal rows keep
/// the order by day.
fn sort_loans(rows: &mut [LentRow], column: &str, direction: SortDirection) {
    use std::cmp::Ordering;
    let cmp: fn(&LentRow, &LentRow) -> Ordering = match column {
        "entry_date" => |a, b| a.entry_date.cmp(&b.entry_date),
        "number" => |a, b| a.number.cmp(&b.number),
        "amount_paisa" => |a, b| a.amount_paisa.cmp(&b.amount_paisa),
        "reason" => |a, b| a.reason.cmp(&b.reason),
        "return_by" => |a, b| a.return_by.cmp(&b.return_by),
        "returned_paisa" => |a, b| a.returned_paisa.cmp(&b.returned_paisa),
        "open_paisa" => |a, b| a.open_paisa.cmp(&b.open_paisa),
        "days_past_return_by" => |a, b| a.days_past_return_by.cmp(&b.days_past_return_by),
        _ => |_, _| Ordering::Equal,
    };
    rows.sort_by(|a, b| direction.apply(cmp(a, b)));
}

/// The columns the report R32 can be sorted by: the names of the fields of the row.
const REPORT_COLUMNS: [&str; 9] = [
    "person",
    "is_staff",
    "lent_paisa",
    "returned_cash_paisa",
    "returned_salary_paisa",
    "open_paisa",
    "oldest_open_date",
    "return_by",
    "days_past_return_by",
];

/// Sorts the whole report by a column (checked by [`REPORT_COLUMNS`]). A stable sort, so equal rows
/// keep the natural order (open first, the oldest open loan first).
fn sort_report(rows: &mut [LentReportRow], column: &str, direction: SortDirection) {
    use std::cmp::Ordering;
    let cmp: fn(&LentReportRow, &LentReportRow) -> Ordering = match column {
        "person" => |a, b| {
            a.person
                .name_en
                .to_lowercase()
                .cmp(&b.person.name_en.to_lowercase())
        },
        "is_staff" => |a, b| a.is_staff.cmp(&b.is_staff),
        "lent_paisa" => |a, b| a.lent_paisa.cmp(&b.lent_paisa),
        "returned_cash_paisa" => |a, b| a.returned_cash_paisa.cmp(&b.returned_cash_paisa),
        "returned_salary_paisa" => |a, b| a.returned_salary_paisa.cmp(&b.returned_salary_paisa),
        "open_paisa" => |a, b| a.open_paisa.cmp(&b.open_paisa),
        "oldest_open_date" => |a, b| a.oldest_open_date.cmp(&b.oldest_open_date),
        "return_by" => |a, b| a.return_by.cmp(&b.return_by),
        "days_past_return_by" => |a, b| a.days_past_return_by.cmp(&b.days_past_return_by),
        _ => |_, _| Ordering::Equal,
    };
    rows.sort_by(|a, b| direction.apply(cmp(a, b)));
}
