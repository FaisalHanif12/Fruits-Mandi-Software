//! The party ledgers screen (PRD FR-12.43, plan 66, D8 Feature 6): the balance of every buyer or
//! seller, and one party's statement with a row for each date on which he has an entry. Read only
//! (Code Standards B29): nothing is saved and no money is worked out except the differences of
//! sums, which are checked (M7). The third level is the R35 day slip of the party and the date
//! (`reports_print_data` for R35), so it is not read here.
//!
//! The figures are the figures of the rest of the app:
//!
//! - A balance is `Balance::from_totals` of the posted journal sums of the account, the same as the
//!   party card and the closing balance of R13 (PRD FR-6.1).
//! - A day's `charged` and `paid` are the live debits and credits of the day, the figures of the R35
//!   all-parties table (`day_dr` and `day_cr`). A same-day edit or cancel leaves no row (the two
//!   halves of the pair are left out, as in R35).
//! - Every row is checked: his balance before the day plus what the day charged and paid is his
//!   balance after it. A pair that a rule change one day splits would be an error, never a quiet
//!   wrong row (money review of plan 66).

use std::collections::HashMap;

use mandi_db::dead;
use mandi_db::ledger as ledger_repo;
use mandi_db::party_ledgers as repo;
use mandi_db::rusqlite::Connection;
use mandi_domain::core::accounts::{BUYER_CONTROL, Balance, BalanceSide, SELLER_CONTROL};
use mandi_domain::core::dates::{BusinessDate, format_ddmmyyyy};
use mandi_domain::core::error::ErrorCode;
use mandi_domain::core::money::Paisa;
use mandi_domain::core::party_ledgers::{
    LedgerSide, PartyLedgerDay, PartyLedgerGoTo, PartyLedgerGoToInput, PartyLedgerHead,
    PartyLedgerRow, PartyLedgerStatement, PartyLedgerStatementInput, PartyLedgerStatementWhole,
    PartyLedgersList, PartyLedgersListInput,
};
use mandi_domain::core::text;
use mandi_domain::core::validate::{self, FieldErrors};

use crate::context::Context;
use crate::day;
use crate::error::{ServiceError, money_error};
use crate::ledger;
use crate::page::DEFAULT_PAGE;

/// The most rows of a page (Code Standards B31).
const MAX_PAGE: u32 = 100;
/// The days of a statement window (PRD FR-12.47).
const WINDOW_DAYS: i64 = 30;
/// How far past the asked day a go-to-date window ends: the asked day sits near its middle
/// (plan 74 W5a).
const WINDOW_AHEAD_DAYS: i64 = 15;
/// Up to this many parties the balances are read for those accounts only; above it the whole
/// control is read in one grouped query, which is cheaper than a very long list of ids.
const NAMED_BALANCES_UP_TO: usize = 500;

fn control_of(side: LedgerSide) -> &'static str {
    match side {
        LedgerSide::Buyer => BUYER_CONTROL,
        LedgerSide::Seller => SELLER_CONTROL,
    }
}

fn page_size(limit: u32) -> u32 {
    match limit {
        0 => DEFAULT_PAGE,
        n => n.min(MAX_PAGE),
    }
}

fn paisa(value: i64) -> Result<Paisa, ServiceError> {
    Paisa::try_from_paisa(value).map_err(money_error)
}

/// A date typed in a filter: empty or missing is no limit.
fn optional_date(
    errors: &mut FieldErrors,
    field: &str,
    text: Option<&String>,
    today: BusinessDate,
) -> Option<String> {
    let text = text.map(String::as_str).filter(|t| !t.trim().is_empty())?;
    errors
        .take(validate::date(field, text, today))
        .map(BusinessDate::to_iso)
}

/// The posted totals of the given parties of one control.
fn totals_of_parties(
    conn: &Connection,
    control: &str,
    ids: &[i64],
) -> Result<HashMap<i64, ledger_repo::Totals>, ServiceError> {
    if ids.len() <= NAMED_BALANCES_UP_TO {
        Ok(ledger_repo::totals_for_accounts(conn, ids)?)
    } else {
        Ok(
            ledger_repo::account_totals_in_controls(conn, &[control], None)?
                .into_iter()
                .collect(),
        )
    }
}

/// Level 1: a page of buyers or sellers by code, active or not, with each balance now and the
/// footer over every party the text matches (PRD FR-12.43, FR-19.11).
///
/// # Errors
/// `ServiceError::Db`; a money step that overflows is `ERR_INTERNAL`.
pub fn list(
    ctx: &Context,
    input: &PartyLedgersListInput,
) -> Result<PartyLedgersList, ServiceError> {
    let control = control_of(input.side);
    let typed = input.text.as_deref().map_or("", str::trim);
    let (text_ur, text_en) = (text::search_ur(typed), text::search_en(typed));
    let limit = page_size(input.limit);
    ctx.db.read(|conn| {
        let page = repo::list_page(
            conn,
            control,
            &text_ur,
            &text_en,
            i64::from(limit),
            i64::from(input.offset),
        )?;
        let ids = repo::matching_ids(conn, control, &text_ur, &text_en)?;
        let totals = totals_of_parties(conn, control, &ids)?;
        let dead = dead::dead_set(conn, None)?;
        let balance = |id: i64| match totals.get(&id) {
            Some(found) => ledger::balance_of(*found),
            None => Ok(Balance::ZERO),
        };
        // The footer is over every matching party, never over the page (B31).
        let (mut total_dr, mut total_cr) = (Paisa::ZERO, Paisa::ZERO);
        for id in &ids {
            let found = balance(*id)?;
            match found.side {
                BalanceSide::Dr => {
                    total_dr = total_dr
                        .checked_add(found.amount_paisa)
                        .map_err(money_error)?;
                }
                BalanceSide::Cr => {
                    total_cr = total_cr
                        .checked_add(found.amount_paisa)
                        .map_err(money_error)?;
                }
                BalanceSide::Zero => {}
            }
        }
        let rows = page
            .into_iter()
            .map(|party| {
                let found = balance(party.id)?;
                Ok(PartyLedgerRow {
                    account_id: party.id,
                    code: party.code,
                    name_ur: party.name_ur,
                    name_en: party.name_en,
                    tag: party.tag,
                    balance_paisa: found.amount_paisa,
                    side: found.side,
                    is_active: party.is_active,
                    is_dead: dead.has_account(party.id),
                })
            })
            .collect::<Result<Vec<_>, ServiceError>>()?;
        Ok(PartyLedgersList {
            rows,
            count: u32::try_from(ids.len())
                .map_err(|_| ServiceError::Data("too many parties".to_owned()))?,
            total_dr_paisa: total_dr,
            total_cr_paisa: total_cr,
        })
    })
}

/// A statement the balances of which do not add up is a fault in the books or in this reading, and
/// the screen shows an error, never a wrong figure.
fn broken(account_id: i64, date: &str) -> ServiceError {
    ServiceError::Data(format!(
        "party ledger of account {account_id}: the day {date} does not add up"
    ))
}

/// The debit side minus the credit side, as one signed amount.
fn net(dr: i64, cr: i64) -> Result<Paisa, ServiceError> {
    paisa(dr)?.checked_sub(paisa(cr)?).map_err(money_error)
}

/// The two figures of a day for this kind of party: a buyer is charged with debits and pays with
/// credits; a seller is charged with credits (what the shop owes him more) and is paid with
/// debits.
fn charged_and_paid(kind: LedgerSide, dr: i64, cr: i64) -> (i64, i64) {
    match kind {
        LedgerSide::Buyer => (dr, cr),
        LedgerSide::Seller => (cr, dr),
    }
}

fn balance_of_net(value: Paisa) -> Result<Balance, ServiceError> {
    Balance::from_net(value).map_err(money_error)
}

/// Level 2: one party's statement, a row for each date on which he has an entry, newest first, with
/// the totals over every date the filter matches (PRD FR-12.43, FR-19.11). With no `to`, the
/// closing balance is his balance now: the figure of his party card and the closing balance of R13.
///
/// # Errors
/// `ERR_NOT_FOUND` on `account_id` for an account that is not a buyer or a seller (the shared چکتا
/// account, نقدی منڈی and the other system accounts have no ledger, PRD FR-13.6); `ERR_DATE_*` on a
/// bad `from` or `to`; `ERR_DATE_RANGE` on `to` when it is before `from`.
pub fn statement(
    ctx: &Context,
    input: &PartyLedgerStatementInput,
) -> Result<PartyLedgerStatement, ServiceError> {
    let (_, today) = day::window(ctx)?;
    let mut errors = FieldErrors::default();
    let from = optional_date(&mut errors, "from", input.from.as_ref(), today);
    let to = optional_date(&mut errors, "to", input.to.as_ref(), today);
    errors.finish()?;
    if let (Some(first), Some(last)) = (&from, &to) {
        // `YYYY-MM-DD` sorts as the dates do.
        if first > last {
            return Err(ServiceError::reject_field(ErrorCode::ErrDateRange, "to"));
        }
    }
    read_statement(
        ctx,
        input.account_id,
        from.as_deref(),
        to.as_deref(),
        i64::from(page_size(input.limit)),
        i64::from(input.offset),
    )
}

/// One party's statement for the ISO dates `from` and `to` (a missing one is no limit): `limit` rows
/// from `offset`, and a negative `limit` for every row (the print, Code Standards B31). The page and
/// the totals come from one reading of his lines.
fn read_statement(
    ctx: &Context,
    account_id: i64,
    from: Option<&str>,
    to: Option<&str>,
    limit: i64,
    offset: i64,
) -> Result<PartyLedgerStatement, ServiceError> {
    ctx.db.read(|conn| {
        let not_found = || ServiceError::reject_field(ErrorCode::ErrNotFound, "account_id");
        let head = repo::head(conn, account_id)?
            .filter(|h| !h.is_system)
            .ok_or_else(not_found)?;
        let kind = match head.control_code.as_str() {
            BUYER_CONTROL => LedgerSide::Buyer,
            SELLER_CONTROL => LedgerSide::Seller,
            _ => return Err(not_found()),
        };
        // The page and the totals come from one reading of his lines (B31, NFR-SPD-24).
        let (days, totals) = repo::statement_page(conn, account_id, from, to, limit, offset)?;
        let is_dead = dead::dead_set(conn, None)?.has_account(account_id);

        let mut rows = Vec::with_capacity(days.len());
        for day in &days {
            let previous = net(day.prev_dr, day.prev_cr)?;
            let after = previous
                .checked_add(net(day.all_dr, day.all_cr)?)
                .map_err(money_error)?;
            // What the day charged and paid must carry him from his balance before to his balance
            // after (the check of the R35 table, `Activity::check`).
            let by_live = previous
                .checked_add(net(day.live_dr, day.live_cr)?)
                .map_err(money_error)?;
            if by_live != after {
                return Err(broken(account_id, &day.date));
            }
            let (charged, paid) = charged_and_paid(kind, day.live_dr, day.live_cr);
            let (previous_balance, balance) = (balance_of_net(previous)?, balance_of_net(after)?);
            rows.push(PartyLedgerDay {
                date: day.date.clone(),
                previous_paisa: previous_balance.amount_paisa,
                previous_side: previous_balance.side,
                charged_paisa: paisa(charged)?,
                paid_paisa: paisa(paid)?,
                balance_paisa: balance.amount_paisa,
                balance_side: balance.side,
            });
        }

        let (charged_total, paid_total) = charged_and_paid(kind, totals.live_dr, totals.live_cr);
        let opening = net(totals.opening_dr, totals.opening_cr)?;
        let closing = net(totals.closing_dr, totals.closing_cr)?;
        // The whole filter adds up too: opening, plus what every date charged and paid, is closing.
        let by_live = opening
            .checked_add(net(totals.live_dr, totals.live_cr)?)
            .map_err(money_error)?;
        if by_live != closing {
            return Err(broken(account_id, "all dates"));
        }
        let (opening, closing) = (balance_of_net(opening)?, balance_of_net(closing)?);
        Ok(PartyLedgerStatement {
            head: PartyLedgerHead {
                account_id,
                code: head.party.code,
                name_ur: head.party.name_ur,
                name_en: head.party.name_en,
                tag: head.party.tag,
                kind,
                is_active: head.party.is_active,
                is_dead,
            },
            rows,
            count: u32::try_from(totals.count)
                .map_err(|_| ServiceError::Data("too many dates".to_owned()))?,
            charged_total_paisa: paisa(charged_total)?,
            paid_total_paisa: paisa(paid_total)?,
            opening_paisa: opening.amount_paisa,
            opening_side: opening.side,
            closing_paisa: closing.amount_paisa,
            closing_side: closing.side,
        })
    })
}

/// One party's statement for a whole period, every date of it with no page limit, for the print (PRD
/// FR-12.47, plan 74 T1, Code Standards B31). `from` and `to` are typed like any date (`DD-MM-YYYY` or
/// a shortcut) and both are required. The rows are newest first, as [`statement`] gives them; the
/// print lays them out oldest first. Read only.
///
/// # Errors
/// `ERR_REQUIRED` and `ERR_DATE_*` on `from` or `to`; `ERR_DATE_RANGE` on `to` when it is before
/// `from`; `ERR_NOT_FOUND` on `account_id` for an account that is not a buyer or a seller.
pub fn statement_whole(
    ctx: &Context,
    account_id: i64,
    from: &str,
    to: &str,
) -> Result<PartyLedgerStatementWhole, ServiceError> {
    let (_, today) = day::window(ctx)?;
    let mut errors = FieldErrors::default();
    let first = errors.take(validate::date("from", from, today));
    let last = errors.take(validate::date("to", to, today));
    errors.finish()?;
    let invalid = |field: &str| ServiceError::reject_field(ErrorCode::ErrDateInvalid, field);
    let first = first.ok_or_else(|| invalid("from"))?;
    let last = last.ok_or_else(|| invalid("to"))?;
    if first > last {
        return Err(ServiceError::reject_field(ErrorCode::ErrDateRange, "to"));
    }
    let (from_iso, to_iso) = (first.to_iso(), last.to_iso());
    let statement = read_statement(ctx, account_id, Some(&from_iso), Some(&to_iso), -1, 0)?;
    Ok(PartyLedgerStatementWhole {
        from: from_iso,
        to: to_iso,
        statement,
    })
}

/// Go to a date on one party's statement (PRD FR-12.47, plan 74 W5a): the 30-day window that ends on
/// the earlier of the day plus 15 days and today, with its page of dates and totals (the same answer
/// as [`statement`] for that `from` and `to`, from the same query), and whether the day has an entry
/// and where its row is.
///
/// Example: today 05-12-2026 and the day 11-10-2026: the window is 27-09-2026 to 26-10-2026. The day
/// 02-12-2026 would end on 17-12-2026, which is after today, so its window ends on 05-12-2026.
///
/// # Errors
/// `ERR_REQUIRED` and `ERR_DATE_INVALID` on `date`; `ERR_DATE_IN_FUTURE` on `date` for a day after
/// today; `ERR_NOT_FOUND` on `account_id` for an account that is not a buyer or a seller.
pub fn go_to_date(
    ctx: &Context,
    input: &PartyLedgerGoToInput,
) -> Result<PartyLedgerGoTo, ServiceError> {
    let (_, today) = day::window(ctx)?;
    let mut errors = FieldErrors::default();
    let asked = errors.take(validate::date("date", &input.date, today));
    errors.finish()?;
    let invalid = || ServiceError::reject_field(ErrorCode::ErrDateInvalid, "date");
    let asked = asked.ok_or_else(invalid)?;
    if asked > today {
        return Err(ServiceError::reject_field(
            ErrorCode::ErrDateInFuture,
            "date",
        ));
    }
    let end = asked
        .plus_days(WINDOW_AHEAD_DAYS)
        .map_or(today, |day| day.min(today));
    let start = end
        .plus_days(1_i64.saturating_sub(WINDOW_DAYS))
        .ok_or_else(invalid)?;
    let statement = statement(
        ctx,
        &PartyLedgerStatementInput {
            account_id: input.account_id,
            from: Some(format_ddmmyyyy(start)),
            to: Some(format_ddmmyyyy(end)),
            offset: 0,
            limit: MAX_PAGE,
        },
    )?;
    let iso = asked.to_iso();
    let row = statement
        .rows
        .iter()
        .position(|r| r.date == iso)
        .and_then(|at| u32::try_from(at).ok());
    Ok(PartyLedgerGoTo {
        from: start.to_iso(),
        to: end.to_iso(),
        has_entry: row.is_some(),
        row,
        statement,
    })
}
