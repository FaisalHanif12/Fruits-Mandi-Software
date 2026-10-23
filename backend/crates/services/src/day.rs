//! The open-day model (PRD FR-6.6, plan 04 Section 4). Closing and reopening days come in D2.

use std::collections::HashMap;

use chrono::NaiveDate;
use mandi_db::partners as partner_repo;
use mandi_db::rusqlite::{Connection, OptionalExtension};
use mandi_db::vouchers as repo;
use mandi_domain::core::accounts::{Balance, BalanceSide, RecordRef};
use mandi_domain::core::dates::BusinessDate;
use mandi_domain::core::error::ErrorCode;
use mandi_domain::core::money::Paisa;
use mandi_domain::core::validate;
use mandi_domain::core::vouchers::{
    DayCloseInput, DayClosePartner, DayClosePreview, DayClosePreviewInput, DayCloseSummary,
    DayReopenInput,
};
use mandi_domain::partners::ShareKind;
use serde_json::json;

use crate::audit::Event;
use crate::auth::Actor;
use crate::context::Context;
use crate::error::{ServiceError, money_error};
use crate::ledger;

/// The storage and display form of a business date.
const DATE_FORMAT: &str = "%Y-%m-%d";

/// Today and the closed-up-to date.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DayStatusInfo {
    /// Today on this computer, `YYYY-MM-DD`.
    pub today: String,
    /// The last closed day. `None` until a day is closed.
    pub closed_up_to: Option<String>,
    /// True when today is after the closed-up-to date.
    pub today_open: bool,
}

/// The open window for postings: the "closed up to" date and today (PRD FR-6.6). A posting date
/// must be after the first and not after the second.
///
/// # Errors
/// As [`status`].
pub fn window(
    ctx: &Context,
) -> Result<
    (
        Option<mandi_domain::core::dates::BusinessDate>,
        mandi_domain::core::dates::BusinessDate,
    ),
    ServiceError,
> {
    use mandi_domain::core::dates::BusinessDate;
    let info = status(ctx)?;
    let parse = |text: &str| {
        BusinessDate::parse_iso(text)
            .ok_or_else(|| ServiceError::Data(format!("a date is not YYYY-MM-DD: {text}")))
    };
    let closed = info.closed_up_to.as_deref().map(parse).transpose()?;
    Ok((closed, parse(&info.today)?))
}

/// Today and the day state. A posting can only be dated after the closed-up-to date and not
/// later than today (PRD FR-6.6).
///
/// # Errors
/// `ServiceError::Db` or `ServiceError::Data` for a date that is not `YYYY-MM-DD`.
pub fn status(ctx: &Context) -> Result<DayStatusInfo, ServiceError> {
    let today: NaiveDate = ctx.clock().local_today();
    let closed: Option<String> = ctx.db.read(|conn| {
        let value: Option<Option<String>> = conn
            .query_row(
                "SELECT closed_up_to_date FROM day_state WHERE id = 1",
                [],
                |row| row.get(0),
            )
            .optional()?;
        Ok::<_, ServiceError>(value.flatten())
    })?;
    let closed_date = closed
        .as_deref()
        .map(|text| {
            NaiveDate::parse_from_str(text, DATE_FORMAT)
                .map_err(|e| ServiceError::Data(format!("closed_up_to_date {text}: {e}")))
        })
        .transpose()?;
    Ok(DayStatusInfo {
        today: today.format(DATE_FORMAT).to_string(),
        closed_up_to: closed,
        today_open: closed_date.is_none_or(|closed| today > closed),
    })
}

/// The "closed up to" date as stored, read through the given connection.
fn closed_in(conn: &Connection) -> Result<Option<BusinessDate>, ServiceError> {
    let text: Option<Option<String>> = conn
        .query_row(
            "SELECT closed_up_to_date FROM day_state WHERE id = 1",
            [],
            |row| row.get(0),
        )
        .optional()?;
    text.flatten()
        .map(|t| {
            BusinessDate::parse_iso(&t)
                .ok_or_else(|| ServiceError::Data(format!("closed_up_to_date {t}")))
        })
        .transpose()
}

/// The cash in hand (روکڑ, PRD FR-10.4).
const CASH_ACCOUNT_CODE: &str = "01010001";

/// The days a close would cover: from the oldest open day to `up_to` (PRD FR-21.1).
///
/// - The oldest open day is the day after "closed up to"; if nothing was ever closed, the day of
///   the oldest posting (or today if there is none).
/// - An empty `up_to` closes that one day. A date closes every open day up to and including it.
/// - A date that is already closed (the "out of order" refusal) is `ERR_DATE_CLOSED`; a date after
///   today is `ERR_DATE_IN_FUTURE`.
fn close_range(
    conn: &Connection,
    window: (Option<BusinessDate>, BusinessDate),
    up_to: &str,
) -> Result<(BusinessDate, BusinessDate), ServiceError> {
    let (closed, today) = window;
    let oldest = match closed {
        Some(closed_day) => closed_day.next_day(),
        None => repo::first_posting_date(conn)?
            .as_deref()
            .and_then(BusinessDate::parse_iso),
    }
    .unwrap_or(today);
    let to = if up_to.trim().is_empty() {
        // The oldest open day itself; nothing is open when it is after today.
        if oldest > today {
            return Err(ServiceError::reject_field(
                ErrorCode::ErrNothingToClose,
                "up_to",
            ));
        }
        validate::posting_date(
            "up_to",
            &mandi_domain::core::dates::format_ddmmyyyy(oldest),
            closed,
            today,
        )?
    } else {
        validate::posting_date("up_to", up_to, closed, today)?
    };
    Ok((oldest.min(to), to))
}

fn day_count(from: BusinessDate, to: BusinessDate) -> Result<u32, ServiceError> {
    let parse = |d: BusinessDate| {
        NaiveDate::parse_from_str(&d.to_iso(), DATE_FORMAT)
            .map_err(|e| ServiceError::Data(format!("date {}: {e}", d.to_iso())))
    };
    let days = parse(to)?.signed_duration_since(parse(from)?).num_days();
    u32::try_from(days.saturating_add(1))
        .map_err(|_| ServiceError::Data("a day range with a negative length".to_owned()))
}

/// The numbers shown before a close and written to the history after it.
fn close_summary(
    conn: &Connection,
    from: BusinessDate,
    to: BusinessDate,
) -> Result<(DayClosePreview, bool), ServiceError> {
    // A draft entry never stays after a save: one found here is a bug, not something to close.
    if repo::unposted_entries(conn)? > 0 {
        return Err(ServiceError::Data(
            "a journal entry is still a draft".to_owned(),
        ));
    }
    let trial = ledger::trial_balance_unchecked(conn, Some(&to.to_iso()))?;
    let difference = Balance::from_totals(trial.total_dr, trial.total_cr).map_err(money_error)?;
    let cash = crate::ledger::balance_in(
        conn,
        mandi_db::accounts::id_by_code(conn, CASH_ACCOUNT_CODE)?
            .ok_or_else(|| ServiceError::Data("the account روکڑ is missing".to_owned()))?,
        None,
        Some(&to.to_iso()),
    )?;
    let balanced = trial.total_dr == trial.total_cr;
    Ok((
        DayClosePreview {
            from_date: from.to_iso(),
            up_to: to.to_iso(),
            days: day_count(from, to)?,
            trial_dr_paisa: trial.total_dr,
            trial_cr_paisa: trial.total_cr,
            trial_difference: difference,
            cash_in_hand: cash,
            entries: repo::entries_between(conn, &from.to_iso(), &to.to_iso())?,
            summary: day_summary(conn, from, to)?,
        },
        balanced,
    ))
}

/// The sales, shares and stock of the days from `from` to `to` (PRD FR-21.1, 4.9). Every number
/// is read from stored records (analytics facts, partner share rows, lots), never worked out
/// again: for 10-10-26 it is 2 بکری, خام 41,400, commission 5,448, shared income 5,488, shares
/// 996 / 746 / 746 and no lots left.
fn day_summary(
    conn: &Connection,
    from: BusinessDate,
    to: BusinessDate,
) -> Result<DayCloseSummary, ServiceError> {
    let (from_iso, to_iso) = (from.to_iso(), to.to_iso());
    let paisa = |value: i64| Paisa::try_from_paisa(value).map_err(money_error);
    let sales = repo::sales_of_days(conn, &from_iso, &to_iso)?;
    // PRD 4.9 "commission": seller commission + buyer commission + لاگا.
    let commission = paisa(sales.seller_comm_paisa)?
        .checked_add(paisa(sales.buyer_comm_paisa)?)
        .map_err(money_error)?
        .checked_add(paisa(sales.laga_paisa)?)
        .map_err(money_error)?;
    let sums = partner_repo::share_sums(conn, &from_iso, &to_iso)?;
    let mut partners = Vec::new();
    let mut shared = Paisa::ZERO;
    let mut records = partner_repo::list(conn)?;
    // The order the partners were created (PRD C10 tie rule).
    records.sort_by_key(|r| r.id);
    for record in &records {
        let sum_of = |kind: &str| -> Result<Paisa, ServiceError> {
            sums.iter()
                .filter(|s| s.partner_id == record.id && s.kind == kind)
                .try_fold(Paisa::ZERO, |acc, s| {
                    acc.checked_add(paisa(s.total_paisa)?).map_err(money_error)
                })
        };
        let income = sum_of(ShareKind::Income.as_str())?;
        // Expense rows are stored negative; the summary shows the expense as a positive number.
        let expense = sum_of(ShareKind::Expense.as_str())?
            .checked_neg()
            .map_err(money_error)?;
        // An inactive partner is shown only if he has a number in these days.
        if !record.is_active && income == Paisa::ZERO && expense == Paisa::ZERO {
            continue;
        }
        shared = shared.checked_add(income).map_err(money_error)?;
        partners.push(DayClosePartner {
            partner: RecordRef {
                id: record.id,
                code: String::new(),
                name_ur: record.name_ur.clone(),
                name_en: record.name_en.clone(),
            },
            income_paisa: income,
            expense_paisa: expense,
            profit_paisa: income.checked_sub(expense).map_err(money_error)?,
        });
    }
    let (open_lots, open_qty) = repo::open_stock(conn, &to_iso)?;
    Ok(DayCloseSummary {
        bikri_count: sales.bikri_count,
        gross_paisa: paisa(sales.gross_paisa)?,
        commission_paisa: commission,
        shared_income_paisa: shared,
        partners,
        open_lots,
        open_qty,
    })
}

/// The numbers of one closed day: the trial balance and the cash in hand at the end of that
/// day, and the entries dated that day.
struct DaySummary {
    date: BusinessDate,
    trial_dr: Paisa,
    trial_cr: Paisa,
    cash: Balance,
    entries: i64,
}

/// A balance as the part it adds to the debit total and the part it adds to the credit total of
/// a trial balance.
fn trial_parts(totals: mandi_db::ledger::Totals) -> Result<(Paisa, Paisa), ServiceError> {
    let balance = ledger::balance_of(totals)?;
    Ok(match balance.side {
        BalanceSide::Dr => (balance.amount_paisa, Paisa::ZERO),
        BalanceSide::Cr => (Paisa::ZERO, balance.amount_paisa),
        BalanceSide::Zero => (Paisa::ZERO, Paisa::ZERO),
    })
}

/// The summary of every day from `from` to `to`, oldest first, from one pass over the journal:
/// the totals of each account the day before, then each day's movements added in turn, with the
/// trial balance kept as a running pair of sums. Days with nothing posted repeat the day before.
fn day_summaries(
    conn: &Connection,
    from: BusinessDate,
    to: BusinessDate,
) -> Result<Vec<DaySummary>, ServiceError> {
    let cash_id = mandi_db::accounts::id_by_code(conn, CASH_ACCOUNT_CODE)?
        .ok_or_else(|| ServiceError::Data("the account روکڑ is missing".to_owned()))?;
    // Nothing can be posted before the first date that exists, so the start is empty there. It
    // must never be "all time" (`None`), which would count every line twice.
    let mut totals = match from.previous_day() {
        Some(before) => ledger::balances_by_account(conn, Some(&before.to_iso()))?,
        None => HashMap::new(),
    };
    let (mut trial_dr, mut trial_cr) = (Paisa::ZERO, Paisa::ZERO);
    for found in totals.values() {
        let (dr, cr) = trial_parts(*found)?;
        trial_dr = trial_dr.checked_add(dr).map_err(money_error)?;
        trial_cr = trial_cr.checked_add(cr).map_err(money_error)?;
    }
    let (from_iso, to_iso) = (from.to_iso(), to.to_iso());
    let mut movements = repo::movements_between(conn, &from_iso, &to_iso)?
        .into_iter()
        .peekable();
    let mut entries = repo::entries_per_day(conn, &from_iso, &to_iso)?
        .into_iter()
        .peekable();
    let mut out = Vec::new();
    let mut day = from;
    loop {
        let iso = day.to_iso();
        while let Some(moved) = movements.next_if(|m| m.date == iso) {
            let slot = totals
                .entry(moved.account_id)
                .or_insert(mandi_db::ledger::Totals {
                    dr_paisa: 0,
                    cr_paisa: 0,
                });
            let (old_dr, old_cr) = trial_parts(*slot)?;
            slot.dr_paisa = slot
                .dr_paisa
                .checked_add(moved.dr_paisa)
                .ok_or_else(|| ServiceError::reject(ErrorCode::ErrOverflow))?;
            slot.cr_paisa = slot
                .cr_paisa
                .checked_add(moved.cr_paisa)
                .ok_or_else(|| ServiceError::reject(ErrorCode::ErrOverflow))?;
            let (new_dr, new_cr) = trial_parts(*slot)?;
            trial_dr = trial_dr
                .checked_sub(old_dr)
                .and_then(|v| v.checked_add(new_dr))
                .map_err(money_error)?;
            trial_cr = trial_cr
                .checked_sub(old_cr)
                .and_then(|v| v.checked_add(new_cr))
                .map_err(money_error)?;
        }
        let count = entries.next_if(|(d, _)| *d == iso).map_or(0, |(_, n)| n);
        let cash = match totals.get(&cash_id) {
            Some(found) => ledger::balance_of(*found)?,
            None => Balance::ZERO,
        };
        out.push(DaySummary {
            date: day,
            trial_dr,
            trial_cr,
            cash,
            entries: count,
        });
        if day >= to {
            break;
        }
        day = day
            .next_day()
            .ok_or_else(|| ServiceError::Data("the day after the last day".to_owned()))?;
    }
    // Both lists are sorted by date inside the range, so each must be used up by now. A leftover
    // line would mean a date the loop never reached, and its money would be missing from the rows.
    if movements.peek().is_some() || entries.peek().is_some() {
        return Err(ServiceError::Data(
            "journal lines dated outside the days being closed".to_owned(),
        ));
    }
    Ok(out)
}

/// The last day's row must carry exactly the numbers of the preview (the trial totals and the
/// cash), and the days' entry counts must add up to the range's count. Two different ways of
/// reaching the same figures; a difference is a bug, so nothing is written.
fn check_rows_match_summary(
    days: &[DaySummary],
    summary: &DayClosePreview,
) -> Result<(), ServiceError> {
    let Some(last) = days.last() else {
        return Err(ServiceError::Data("a close without days".to_owned()));
    };
    let counted = days
        .iter()
        .try_fold(0_i64, |sum, d| sum.checked_add(d.entries))
        .ok_or_else(|| ServiceError::reject(ErrorCode::ErrOverflow))?;
    let same = last.trial_dr == summary.trial_dr_paisa
        && last.trial_cr == summary.trial_cr_paisa
        && last.cash == summary.cash_in_hand
        && counted == summary.entries;
    if same {
        Ok(())
    } else {
        Err(ServiceError::Data(
            "the day rows and the close summary disagree".to_owned(),
        ))
    }
}

/// What closing the days up to a day would do (PRD FR-21.1): the days, the trial balance (the
/// difference must be 0) and the cash in hand. Only reads. Plans 12 and 13 add the بکری and
/// stock lines of the summary.
///
/// # Errors
/// `ERR_DATE_CLOSED` (value `closed_up_to`) or `ERR_DATE_IN_FUTURE` on `up_to`.
pub fn close_preview(
    ctx: &Context,
    input: &DayClosePreviewInput,
) -> Result<DayClosePreview, ServiceError> {
    let window = window(ctx)?;
    ctx.db.read(|conn| {
        // Every day up to today is closed and no date was asked for: the answer is "0 days" with
        // today's numbers, so the screen can say so (the close itself is refused with
        // ERR_NOTHING_TO_CLOSE).
        if input.up_to.trim().is_empty() && window.0.is_some_and(|closed| closed >= window.1) {
            let mut summary = close_summary(conn, window.1, window.1)?.0;
            summary.days = 0;
            summary.entries = 0;
            return Ok(summary);
        }
        let (from, to) = close_range(conn, window, &input.up_to)?;
        Ok(close_summary(conn, from, to)?.0)
    })
}

/// Closes the days up to a day, oldest first, in one step (PRD FR-21.1): the "closed up to" date
/// moves to `up_to`, one row goes to the close history for each day closed (with that day's trial
/// totals, cash in hand and entries), and one audit row says who did it. Refuses a day that is
/// already closed, a day after today, and a trial balance that does not balance on any of the
/// days. All in one transaction. The cloud backup that follows a close comes in plan D5.
///
/// # Errors
/// `ERR_DATE_CLOSED`, `ERR_DATE_IN_FUTURE` on `up_to`; `ERR_TRIAL_NOT_BALANCED`.
pub fn close(
    ctx: &Context,
    actor: &Actor,
    input: &DayCloseInput,
) -> Result<DayStatusInfo, ServiceError> {
    let today = ctx.clock().local_today();
    let today = BusinessDate::parse_iso(&today.format(DATE_FORMAT).to_string())
        .ok_or_else(|| ServiceError::Data("today is not a date".to_owned()))?;
    ctx.db.write(|tx| {
        // The "closed up to" date is read inside the transaction, so two calls queued on the
        // writer cannot work from an old value (PRD FR-21.2: closing never moves it back).
        let window = (closed_in(tx)?, today);
        let (from, to) = close_range(tx, window, &input.up_to)?;
        let (summary, balanced) = close_summary(tx, from, to)?;
        // Every day must balance on its own, not only the last one.
        let days = day_summaries(tx, from, to)?;
        check_rows_match_summary(&days, &summary)?;
        if !balanced || days.iter().any(|d| d.trial_dr != d.trial_cr) {
            return Err(ServiceError::reject_field(
                ErrorCode::ErrTrialNotBalanced,
                "up_to",
            ));
        }
        let now = ctx.now_text();
        repo::set_closed_up_to(tx, Some(&to.to_iso()), actor.user_id, &now)?;
        // One history row for each day closed, each with its own numbers (Architecture 15.3).
        for day in &days {
            let numbers = json!({
                "trial_dr_paisa": day.trial_dr.as_paisa(),
                "trial_cr_paisa": day.trial_cr.as_paisa(),
                "cash_in_hand_paisa": day.cash.amount_paisa.as_paisa(),
                "cash_in_hand_side": match day.cash.side {
                    BalanceSide::Dr => "dr",
                    BalanceSide::Cr => "cr",
                    BalanceSide::Zero => "zero",
                },
                "entries": day.entries,
                "closed_in_one_step_up_to": summary.up_to,
            });
            repo::insert_day_close(
                tx,
                &day.date.to_iso(),
                "close",
                actor.user_id,
                &now,
                None,
                &numbers.to_string(),
            )?;
        }
        let numbers = json!({
            "from_date": summary.from_date,
            "up_to": summary.up_to,
            "days": summary.days,
            "trial_dr_paisa": summary.trial_dr_paisa.as_paisa(),
            "trial_cr_paisa": summary.trial_cr_paisa.as_paisa(),
            "cash_in_hand_paisa": summary.cash_in_hand.amount_paisa.as_paisa(),
            "entries": summary.entries,
        });
        ctx.db.auditor().write(
            tx,
            &Event::new(Some(actor.user_id), "day_close", "day")
                .old(json!({ "closed_up_to": window.0.map(BusinessDate::to_iso) }))
                .new_value(numbers)
                .into_entry(),
        )?;
        Ok(())
    })?;
    status(ctx)
}

/// Reopens a day and every later day (PRD FR-21.2): "closed up to" moves back to the day before.
/// A reason is required and goes to the audit log with the old and new date. One user, so no
/// role check beyond sign-in. A month that was month-closed cannot be reopened until its month
/// close is reversed.
///
/// # Errors
/// `ERR_REASON_REQUIRED` on `reason`; the date codes on `date`; `ERR_DAY_NOT_CLOSED` on `date`;
/// `ERR_MONTH_CLOSED` on `date`.
pub fn reopen(
    ctx: &Context,
    actor: &Actor,
    input: &DayReopenInput,
) -> Result<DayStatusInfo, ServiceError> {
    let (_, today) = window(ctx)?;
    let mut errors = validate::FieldErrors::default();
    let date = errors.take(validate::date("date", &input.date, today));
    let reason = errors
        .take(validate::free_text("reason", &input.reason))
        .and_then(|r| {
            if r.is_empty() {
                errors.push(mandi_domain::core::error::FieldError::new(
                    ErrorCode::ErrReasonRequired,
                    "reason",
                ));
                None
            } else {
                Some(r)
            }
        });
    errors.finish()?;
    let (Some(date), Some(reason)) = (date, reason) else {
        return Err(ServiceError::reject(ErrorCode::ErrInternal));
    };
    let new_closed = date.previous_day();
    ctx.db.write(|tx| {
        // The state is read inside the transaction (see `close`).
        let Some(closed_day) = closed_in(tx)?.filter(|c| date <= *c) else {
            return Err(ServiceError::reject_field(
                ErrorCode::ErrDayNotClosed,
                "date",
            ));
        };
        let month = date.to_iso().chars().take(7).collect::<String>();
        if repo::month_closed_from(tx, &month)? {
            return Err(ServiceError::reject_field(
                ErrorCode::ErrMonthClosed,
                "date",
            ));
        }
        let now = ctx.now_text();
        let new_iso = new_closed.map(BusinessDate::to_iso);
        repo::set_closed_up_to(tx, new_iso.as_deref(), actor.user_id, &now)?;
        let numbers = json!({
            "reopened_from": date.to_iso(),
            "closed_up_to_before": closed_day.to_iso(),
            "closed_up_to_after": new_iso,
        });
        repo::insert_day_close(
            tx,
            &date.to_iso(),
            "reopen",
            actor.user_id,
            &now,
            Some(&reason),
            &numbers.to_string(),
        )?;
        ctx.db.auditor().write(
            tx,
            &Event::new(Some(actor.user_id), "day_reopen", "day")
                .old(json!({ "closed_up_to": closed_day.to_iso() }))
                .new_value(json!({ "closed_up_to": new_iso }))
                .reason(&reason)
                .into_entry(),
        )?;
        Ok(())
    })?;
    status(ctx)
}
