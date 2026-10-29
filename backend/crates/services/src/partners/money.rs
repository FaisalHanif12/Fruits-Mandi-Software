//! What reaches a partner (PRD 16.2, 17.4, FR-17.9, FR-17.10, plan 17): the money he took, his
//! capital history and his dated statement with the running کل رقم.
//!
//! Plan 17: the money taken, the کل رقم (C22) and the capital history. The dated statement is in
//! `statement`.

use std::collections::HashMap;

use mandi_db::open_lines::{self, LiveLine};
use mandi_db::partners::{self as repo, PartnerRecord};
use mandi_db::rusqlite::Connection;
use mandi_domain::core::accounts::RecordRef;
use mandi_domain::core::capital::{
    CapitalHistory, CapitalHistoryInput, CapitalRow, CapitalRowKind, CapitalTotals,
};
use mandi_domain::core::dates::BusinessDate;
use mandi_domain::core::error::ErrorCode;
use mandi_domain::core::expenses::{
    PartnerTakes, PartnerTakesInput, TakeRow, TakeSource, TakesTotals,
};
use mandi_domain::core::money::Paisa;
use mandi_domain::core::paging::{SortDirection, SortInput};
use mandi_domain::expenses::{OpenAmount, days_past_return_by, open_amounts};

use crate::context::Context;
use crate::day;
use crate::error::{ServiceError, money_error};
use crate::page;
use crate::ventures::record_of;

fn paisa(value: i64) -> Result<Paisa, ServiceError> {
    Paisa::try_from_paisa(value).map_err(money_error)
}

/// A partner's کل رقم (PRD FR-17.9, C22): his opening capital, plus capital invested, less capital
/// taken back (his capital account), plus his income shares, less his expense shares, less the
/// money he took, plus what he returned, plus his gardens, trips and fruit account (credits less
/// debits). `as_of` (`YYYY-MM-DD`, included) gives it at the end of a day; `None` is now.
///
/// Read from posted records only. Month close never changes it (the sums leave its entries out).
///
/// # Errors
/// `ServiceError::Db`; `ERR_OVERFLOW`.
pub(crate) fn total_of(
    conn: &Connection,
    partner: &PartnerRecord,
    as_of: Option<&str>,
) -> Result<Paisa, ServiceError> {
    let parts = repo::total_parts(conn, partner, as_of)?;
    let capital = paisa(parts.capital_cr)?
        .checked_sub(paisa(parts.capital_dr)?)
        .map_err(money_error)?;
    let taken_net = paisa(parts.drawings_dr)?
        .checked_sub(paisa(parts.drawings_cr)?)
        .map_err(money_error)?;
    let ventures = paisa(parts.venture_cr)?
        .checked_sub(paisa(parts.venture_dr)?)
        .map_err(money_error)?;
    capital
        .checked_add(paisa(parts.shares)?)
        .and_then(|v| v.checked_sub(taken_net))
        .and_then(|v| v.checked_add(ventures))
        .map_err(money_error)
}

/// A drawings line that is money taken (a debit).
fn is_take(line: &LiveLine) -> bool {
    line.dr_paisa > 0
}

/// The money a partner took, oldest first, with what was returned against each, one page
/// (PRD FR-16.2). Worked out at read time, oldest first (money agent 8.2b); nothing is stored.
/// A drawings opening debit is a take dated its opening date (PRD FR-10.9). Σ open is the debit
/// balance of his drawings account.
///
/// # Errors
/// `ERR_NOT_FOUND` on `partner_id`.
pub fn takes(ctx: &Context, input: &PartnerTakesInput) -> Result<PartnerTakes, ServiceError> {
    let (_, today) = day::window(ctx)?;
    // The sort is checked first: an unknown column is refused (plan 38 R4).
    if let Some(sort) = &input.sort {
        sort.check(&TAKE_COLUMNS)
            .map_err(|e| ServiceError::reject_field(e.code, &e.field))?;
    }
    ctx.db.read(|conn| {
        let partner = repo::get(conn, input.partner_id)?
            .ok_or_else(|| ServiceError::reject_field(ErrorCode::ErrNotFound, "partner_id"))?;
        let lines = open_lines::live_lines(conn, &[partner.drawings_account_id])?;
        let bills = lines
            .iter()
            .filter(|l| is_take(l))
            .map(|l| Ok((l.line_id, paisa(l.dr_paisa)?)))
            .collect::<Result<Vec<_>, ServiceError>>()?;
        let credits = lines
            .iter()
            .filter(|l| l.cr_paisa > 0)
            .map(|l| Ok((l.line_id, paisa(l.cr_paisa)?)))
            .collect::<Result<Vec<_>, ServiceError>>()?;
        let open: HashMap<i64, OpenAmount> = open_amounts(&bills, &credits)
            .map_err(money_error)?
            .into_iter()
            .map(|o| (o.line_id, o))
            .collect();
        let mut totals = TakesTotals {
            taken_paisa: Paisa::ZERO,
            returned_paisa: Paisa::ZERO,
            open_paisa: Paisa::ZERO,
        };
        let mut rows = Vec::with_capacity(open.len());
        let mut month_acc = page::MonthAcc::new(3);
        for line in lines.iter().filter(|l| is_take(l)) {
            let Some(figures) = open.get(&line.line_id) else {
                continue;
            };
            let amount = paisa(line.dr_paisa)?;
            totals.taken_paisa = totals
                .taken_paisa
                .checked_add(amount)
                .map_err(money_error)?;
            totals.returned_paisa = totals
                .returned_paisa
                .checked_add(figures.returned)
                .map_err(money_error)?;
            totals.open_paisa = totals
                .open_paisa
                .checked_add(figures.open)
                .map_err(money_error)?;
            month_acc.add(
                &line.entry_date,
                &[
                    amount.as_paisa(),
                    figures.returned.as_paisa(),
                    figures.open.as_paisa(),
                ],
            )?;
            let return_by = line.return_by.as_deref().and_then(BusinessDate::parse_iso);
            rows.push(TakeRow {
                source: if line.is_opening {
                    TakeSource::Opening
                } else {
                    TakeSource::Voucher
                },
                voucher_id: line.voucher_id,
                number: line.number,
                entry_date: line.entry_date.clone(),
                amount_paisa: amount,
                reason: line.reason.clone(),
                taken_by: line.taken_by.clone(),
                return_by: line.return_by.clone(),
                returned_paisa: figures.returned,
                open_paisa: figures.open,
                days_past_return_by: days_past_return_by(figures.open, return_by, today),
            });
        }
        let total_count = i64::try_from(rows.len()).unwrap_or(i64::MAX);
        if let Some(sort) = &input.sort {
            sort_takes(&mut rows, &sort.column, sort.direction);
        }
        let (in_date_order, newest_first) = date_order(input.sort.as_ref());
        let months = if input.group_by_month && in_date_order {
            month_acc.finish(
                &[
                    ("taken_paisa", true),
                    ("returned_paisa", true),
                    ("open_paisa", true),
                ],
                newest_first,
            )
        } else {
            Vec::new()
        };
        Ok(PartnerTakes {
            partner: record_of(partner.id, &partner.name_ur, &partner.name_en),
            rows: page::slice(rows, input.offset, input.limit),
            total_count,
            totals,
            months,
        })
    })
}

/// A partner's capital history, oldest first, one page (PRD 17.4, FR-19.11). Read only. The rows
/// are the live lines of his capital account: a voucher that was cancelled or edited leaves no
/// trace (the reversal and the old line cancel). Month close shows as a row of its own, because it
/// moves profit into capital; his کل رقم does not change by it (PRD FR-17.6).
///
/// Example (PRD E2E-35): ب invests 50,000: one row `invested` +50,000, balance 50,000.
///
/// # Errors
/// `ERR_NOT_FOUND` on `partner_id`.
pub fn capital_history(
    ctx: &Context,
    input: &CapitalHistoryInput,
) -> Result<CapitalHistory, ServiceError> {
    // The sort is checked first: an unknown column is refused (plan 38 R4).
    if let Some(sort) = &input.sort {
        sort.check(&CAPITAL_COLUMNS)
            .map_err(|e| ServiceError::reject_field(e.code, &e.field))?;
    }
    ctx.db.read(|conn| {
        let partner = repo::get(conn, input.partner_id)?
            .ok_or_else(|| ServiceError::reject_field(ErrorCode::ErrNotFound, "partner_id"))?;
        let lines = open_lines::live_lines(conn, &[partner.capital_account_id])?;
        let mut month_acc = page::MonthAcc::new(2);
        let mut balance = Paisa::ZERO;
        let mut totals = CapitalTotals {
            invested_paisa: Paisa::ZERO,
            taken_back_paisa: Paisa::ZERO,
        };
        let mut rows = Vec::with_capacity(lines.len());
        for line in &lines {
            let (dr, cr) = (paisa(line.dr_paisa)?, paisa(line.cr_paisa)?);
            let amount = cr.checked_sub(dr).map_err(money_error)?;
            let kind = match line.posting_type.as_str() {
                "opening" => CapitalRowKind::Opening,
                "month_close" => CapitalRowKind::MonthClose,
                _ if cr > Paisa::ZERO => CapitalRowKind::Invested,
                _ => CapitalRowKind::TakenBack,
            };
            // Month close moves profit between his accounts; it is neither put in nor taken back.
            if kind != CapitalRowKind::MonthClose {
                totals.invested_paisa =
                    totals.invested_paisa.checked_add(cr).map_err(money_error)?;
                totals.taken_back_paisa = totals
                    .taken_back_paisa
                    .checked_add(dr)
                    .map_err(money_error)?;
            }
            balance = balance.checked_add(amount).map_err(money_error)?;
            // Month close moves profit between his accounts, so it adds nothing to the month's
            // money (the same rule as the totals).
            let (put_in, put_out) = if kind == CapitalRowKind::MonthClose {
                (0, 0)
            } else {
                (cr.as_paisa(), dr.as_paisa())
            };
            month_acc.add(&line.entry_date, &[put_in, put_out])?;
            let from_voucher = matches!(kind, CapitalRowKind::Invested | CapitalRowKind::TakenBack);
            rows.push(CapitalRow {
                kind,
                voucher_id: line.voucher_id.filter(|_| from_voucher),
                number: line.number.filter(|_| from_voucher),
                entry_date: line.entry_date.clone(),
                amount_paisa: amount,
                narration: line.narration.clone(),
                running_paisa: balance,
            });
        }
        let total_count = i64::try_from(rows.len()).unwrap_or(i64::MAX);
        if let Some(sort) = &input.sort {
            sort_capital(&mut rows, &sort.column, sort.direction);
        }
        let (in_date_order, newest_first) = date_order(input.sort.as_ref());
        let months = if input.group_by_month && in_date_order {
            month_acc.finish(
                &[("invested_paisa", true), ("taken_back_paisa", true)],
                newest_first,
            )
        } else {
            Vec::new()
        };
        Ok(CapitalHistory {
            partner: RecordRef {
                id: partner.id,
                code: partner.capital_account_code.clone(),
                name_ur: partner.name_ur.clone(),
                name_en: partner.name_en.clone(),
                tag: None,
            },
            balance_paisa: balance,
            rows: page::slice(rows, input.offset, input.limit),
            total_count,
            totals,
            months,
        })
    })
}

/// Whether a sort keeps the list in date order, and whether it is newest first.
fn date_order(sort: Option<&SortInput>) -> (bool, bool) {
    (
        sort.is_none_or(|s| s.column == "entry_date"),
        sort.is_some_and(|s| s.column == "entry_date" && s.direction == SortDirection::Desc),
    )
}

/// The columns the list of money taken can be sorted by: the names of the fields of the row.
const TAKE_COLUMNS: [&str; 10] = [
    "entry_date",
    "source",
    "number",
    "amount_paisa",
    "reason",
    "taken_by",
    "return_by",
    "returned_paisa",
    "open_paisa",
    "days_past_return_by",
];

/// Sorts the whole list by a column (checked by [`TAKE_COLUMNS`]). A stable sort, so equal rows keep
/// the order by day.
fn sort_takes(rows: &mut [TakeRow], column: &str, direction: SortDirection) {
    use std::cmp::Ordering;
    let cmp: fn(&TakeRow, &TakeRow) -> Ordering = match column {
        "entry_date" => |a, b| a.entry_date.cmp(&b.entry_date),
        "source" => |a, b| (a.source as u8).cmp(&(b.source as u8)),
        "number" => |a, b| a.number.cmp(&b.number),
        "amount_paisa" => |a, b| a.amount_paisa.cmp(&b.amount_paisa),
        "reason" => |a, b| a.reason.cmp(&b.reason),
        "taken_by" => |a, b| a.taken_by.cmp(&b.taken_by),
        "return_by" => |a, b| a.return_by.cmp(&b.return_by),
        "returned_paisa" => |a, b| a.returned_paisa.cmp(&b.returned_paisa),
        "open_paisa" => |a, b| a.open_paisa.cmp(&b.open_paisa),
        "days_past_return_by" => |a, b| a.days_past_return_by.cmp(&b.days_past_return_by),
        _ => |_, _| Ordering::Equal,
    };
    rows.sort_by(|a, b| direction.apply(cmp(a, b)));
}

/// The columns the capital history can be sorted by: the names of the fields of the row.
const CAPITAL_COLUMNS: [&str; 6] = [
    "entry_date",
    "kind",
    "number",
    "amount_paisa",
    "narration",
    "running_paisa",
];

/// Sorts the whole history by a column (checked by [`CAPITAL_COLUMNS`]). A stable sort, so equal
/// rows keep the order by day.
fn sort_capital(rows: &mut [CapitalRow], column: &str, direction: SortDirection) {
    use std::cmp::Ordering;
    let cmp: fn(&CapitalRow, &CapitalRow) -> Ordering = match column {
        "entry_date" => |a, b| a.entry_date.cmp(&b.entry_date),
        "kind" => |a, b| (a.kind as u8).cmp(&(b.kind as u8)),
        "number" => |a, b| a.number.cmp(&b.number),
        "amount_paisa" => |a, b| a.amount_paisa.cmp(&b.amount_paisa),
        "narration" => |a, b| a.narration.cmp(&b.narration),
        "running_paisa" => |a, b| a.running_paisa.cmp(&b.running_paisa),
        _ => |_, _| Ordering::Equal,
    };
    rows.sort_by(|a, b| direction.apply(cmp(a, b)));
}
