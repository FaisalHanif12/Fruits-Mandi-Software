//! The open-day model (PRD FR-6.6, plan 04 Section 4). Closing and reopening days come in D2.

use std::collections::HashMap;

use chrono::NaiveDate;
use mandi_db::partners as partner_repo;
use mandi_db::rusqlite::{Connection, OptionalExtension};
use mandi_db::vouchers as repo;
use mandi_domain::core::accounts::{Balance, BalanceSide, RecordRef};
use mandi_domain::core::dates::{self, BusinessDate};
use mandi_domain::core::edit::{
    DayFixCloseInput, DayFixClosed, DayFixOpenInput, DayFixOpened, DayFixStayedOpen,
};
use mandi_domain::core::error::{ErrorCode, ErrorValue, FieldError};
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
use crate::error::{Rejection, ServiceError, money_error};
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

/// The refusal of a closed day (`ERR_DATE_CLOSED`) on `field`, with every value a screen needs
/// (plan 53): `closed_up_to` (the closed day shown), `today` and `today_open` (1 or 0), from
/// [`validate::closed_day_values`]. Both dates are display text, `DD-MM-YYYY`, as every other
/// `ERR_DATE_CLOSED` carries them (`validate::posting_date`). `closed_up_to` is the real
/// closed-up-to date of the books, `today` today.
pub(crate) fn closed_rejection_at(
    shown: BusinessDate,
    closed_up_to: Option<BusinessDate>,
    today: BusinessDate,
    field: &str,
) -> ServiceError {
    let mut values = std::collections::BTreeMap::from([(
        "closed_up_to".to_owned(),
        ErrorValue::Text(dates::format_ddmmyyyy(shown)),
    )]);
    for (key, value) in validate::closed_day_values(closed_up_to, today) {
        values.insert(key.to_owned(), value);
    }
    ServiceError::Rejected(vec![Rejection {
        code: ErrorCode::ErrDateClosed,
        field: Some(field.to_owned()),
        values,
    }])
}

/// [`closed_rejection_at`] for the books as they are now.
pub(crate) fn closed_rejection(ctx: &Context, shown: BusinessDate, field: &str) -> ServiceError {
    match window(ctx) {
        Ok((closed_up_to, today)) => closed_rejection_at(shown, closed_up_to, today, field),
        Err(error) => error,
    }
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

/// The first day of the books: the oldest date of a posted entry, a trip or garden, a saved
/// purchase or a stock move, or `None` when there is none yet. The assistant's period `up_to_now`
/// starts here (plan 52 item 2). Reads only.
///
/// # Errors
/// `ServiceError::Db`; `ServiceError::Data` for a date that is not `YYYY-MM-DD`.
pub fn first_book_day(ctx: &Context) -> Result<Option<BusinessDate>, ServiceError> {
    let text: Option<String> = ctx
        .db
        .read(|conn| repo::first_book_date(conn).map_err(ServiceError::from))?;
    text.map(|t| {
        BusinessDate::parse_iso(&t).ok_or_else(|| ServiceError::Data(format!("a date is {t}")))
    })
    .transpose()
}

/// The "closed up to" date as stored, read through the given connection.
pub(crate) fn closed_in(conn: &Connection) -> Result<Option<BusinessDate>, ServiceError> {
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
                tag: None,
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
        partners_locked: false,
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
    ctx.db.write(|tx| close_in(ctx, tx, actor, input))?;
    status(ctx)
}

/// The writes of [`close`] inside a transaction the caller owns, so a reopen-and-fix can close its
/// days in the same one (PRD FR-21.4, Code Standards B3). A refusal comes before any write.
pub(crate) fn close_in(
    ctx: &Context,
    tx: &Connection,
    actor: &Actor,
    input: &DayCloseInput,
) -> Result<(), ServiceError> {
    let today = ctx.clock().local_today();
    let today = BusinessDate::parse_iso(&today.format(DATE_FORMAT).to_string())
        .ok_or_else(|| ServiceError::Data("today is not a date".to_owned()))?;
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
    settle_manual_reopen(tx, to, actor.user_id)?;
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
}

/// Reopens a day and every later day (PRD FR-21.2): "closed up to" moves back to the day before.
/// A reason is required and goes to the audit log with the old and new date. Only the Owner's sign
/// opens the day, as in a reopen to fix an entry (PRD FR-21.4, Umar 19-11-2026, plan 80 Q-R2-2): a
/// wrong sign opens nothing and is audited without the typed text, and the right sign is kept in the
/// audit entry beside the reason. A month that was month-closed cannot be reopened until its month
/// close is reversed. The days stay held against the automatic close until a close reaches the last
/// of them ([`auto_close_hold`], plan 85); the fact is kept in the `day_manual_reopen` setting.
///
/// # Errors
/// `ERR_REASON_REQUIRED` on `reason`; `ERR_SIGN_REQUIRED` and `ERR_SIGN_WRONG` on `sign`; the date
/// codes on `date`; `ERR_DAY_NOT_CLOSED` on `date`; `ERR_MONTH_CLOSED` on `date`.
pub fn reopen(
    ctx: &Context,
    actor: &Actor,
    input: &DayReopenInput,
) -> Result<DayStatusInfo, ServiceError> {
    check_sign(
        ctx,
        actor,
        &SignCheck {
            reason: &input.reason,
            sign: &input.sign,
            date: &input.date,
            wrong_action: "day_reopen_sign_wrong",
        },
    )?;
    let (date, reason) = parse_reopen(ctx, &input.date, &input.reason)?;
    ctx.db.write(|tx| {
        let closed_before = reopen_in(ctx, tx, actor, date, &reason, true)?;
        // The Owner opened these days by hand: the automatic close leaves them alone until he closes
        // them (PRD FR-21.4 v1.49, plan 85).
        note_manual_reopen(tx, actor.user_id, date, closed_before)?;
        Ok::<(), ServiceError>(())
    })?;
    status(ctx)
}

/// The date and the reason of a reopen, checked (nothing is read but today).
fn parse_reopen(
    ctx: &Context,
    date_text: &str,
    reason_text: &str,
) -> Result<(BusinessDate, String), ServiceError> {
    let (_, today) = window(ctx)?;
    let mut errors = validate::FieldErrors::default();
    let date = errors.take(validate::date("date", date_text, today));
    let reason = errors
        .take(validate::free_text("reason", reason_text))
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
    match (date, reason) {
        (Some(date), Some(reason)) => Ok((date, reason)),
        _ => Err(ServiceError::reject(ErrorCode::ErrInternal)),
    }
}

/// The writes of a reopen inside a transaction the caller owns: the "closed up to" date moves back,
/// one day-close row and one audit entry say who, why and (for a reopen to fix) with which sign.
/// Returns the day that was the "closed up to" day before it. The state is read inside the
/// transaction (see [`close_in`]).
pub(crate) fn reopen_in(
    ctx: &Context,
    tx: &Connection,
    actor: &Actor,
    date: BusinessDate,
    reason: &str,
    sign_given: bool,
) -> Result<BusinessDate, ServiceError> {
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
    let new_iso = date.previous_day().map(BusinessDate::to_iso);
    repo::set_closed_up_to(tx, new_iso.as_deref(), actor.user_id, &now)?;
    let mut numbers = json!({
        "reopened_from": date.to_iso(),
        "closed_up_to_before": closed_day.to_iso(),
        "closed_up_to_after": new_iso,
    });
    // The sign is a secret: only the fact that it was given is kept (PRD FR-21.4, FR-7.71 v1.58).
    if let (true, Some(object)) = (sign_given, numbers.as_object_mut()) {
        object.insert("sign_given".to_owned(), json!(true));
    }
    repo::insert_day_close(
        tx,
        &date.to_iso(),
        "reopen",
        actor.user_id,
        &now,
        Some(reason),
        &numbers.to_string(),
    )?;
    ctx.db.auditor().write(
        tx,
        &Event::new(Some(actor.user_id), "day_reopen", "day")
            .old(json!({ "closed_up_to": closed_day.to_iso() }))
            .new_value(json!({ "closed_up_to": new_iso }))
            .sign_given(sign_given)
            .reason(reason)
            .into_entry(),
    )?;
    Ok(closed_day)
}

/// The setting that remembers a fix in progress: `{ "token", "first", "last", "sign_given", "opened_at" }`
/// (PRD FR-21.4, plan 73, plan 85).
const KEY_FIX_PENDING: &str = "day_fix_pending";

/// The setting that remembers the days the Owner reopened by hand and has not closed again:
/// `{ "first", "last" }` (PRD FR-21.4 v1.49, plan 85). It lives in the data file, so a restart keeps it.
const KEY_MANUAL_REOPEN: &str = "day_manual_reopen";

/// How long the automatic close waits for a fix in progress, in hours. A fix is a few minutes of work
/// on one screen; one that is older than this was left behind (the app was closed in the middle of
/// it), and the days it reopened are closed by the automatic close like any other open day (plan 85).
pub const FIX_HOLD_HOURS: i64 = 12;

/// The fix in progress, if any: the token, the first and last day it reopened, whether the Owner's sign
/// opened it (the sign itself is never kept, PRD FR-7.71 v1.58) and when (UTC text; empty in a record
/// made before plan 85).
struct Pending {
    token: String,
    first: BusinessDate,
    last: BusinessDate,
    sign_given: bool,
    opened_at: Option<String>,
}

fn pending_of(conn: &Connection) -> Result<Option<Pending>, ServiceError> {
    let Some(value) = crate::store::setting(conn, KEY_FIX_PENDING)? else {
        return Ok(None);
    };
    let day = |key: &str| {
        value
            .get(key)
            .and_then(serde_json::Value::as_str)
            .and_then(BusinessDate::parse_iso)
    };
    let token = value
        .get("token")
        .and_then(serde_json::Value::as_str)
        .map(str::to_owned);
    // A record made before plan 96 holds the sign text under `sign`: it only counts as "given" and the
    // text is never read.
    let sign_given = value
        .get("sign_given")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false)
        || value
            .get("sign")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|sign| !sign.is_empty());
    let opened_at = value
        .get("opened_at")
        .and_then(serde_json::Value::as_str)
        .map(str::to_owned);
    Ok(match (token, day("first"), day("last")) {
        (Some(token), Some(first), Some(last)) => Some(Pending {
            token,
            first,
            last,
            sign_given,
            opened_at,
        }),
        _ => None,
    })
}

/// The days the Owner reopened by hand: the first day he opened and the last day that was closed
/// when he did.
struct ManualReopen {
    first: BusinessDate,
    last: BusinessDate,
}

fn manual_reopen_of(conn: &Connection) -> Result<Option<ManualReopen>, ServiceError> {
    let Some(value) = crate::store::setting(conn, KEY_MANUAL_REOPEN)? else {
        return Ok(None);
    };
    let day = |key: &str| {
        value
            .get(key)
            .and_then(serde_json::Value::as_str)
            .and_then(BusinessDate::parse_iso)
    };
    Ok(match (day("first"), day("last")) {
        (Some(first), Some(last)) => Some(ManualReopen { first, last }),
        _ => None,
    })
}

/// Remembers a reopen made by hand (PRD FR-21.2, plan 85). `closed_before` is the "closed up to" day
/// before the reopen. A hold still in force is widened to the earlier first day and the later last day.
fn note_manual_reopen(
    tx: &Connection,
    user_id: i64,
    first: BusinessDate,
    closed_before: BusinessDate,
) -> Result<(), ServiceError> {
    let kept = manual_reopen_of(tx)?.filter(|held| closed_before < held.last);
    let (first, last) = kept.map_or((first, closed_before), |held| {
        (held.first.min(first), held.last.max(closed_before))
    });
    crate::store::set_setting(
        tx,
        KEY_MANUAL_REOPEN,
        &json!({ "first": first.to_iso(), "last": last.to_iso() }),
        Some(user_id),
    )
}

/// A close up to `closed_up_to` ends the hold of a reopen by hand when it reaches the last day he
/// opened. A close that stops short leaves it: the days after it are still his.
fn settle_manual_reopen(
    tx: &Connection,
    closed_up_to: BusinessDate,
    user_id: i64,
) -> Result<(), ServiceError> {
    if manual_reopen_of(tx)?.is_some_and(|held| held.last <= closed_up_to) {
        crate::store::set_setting(tx, KEY_MANUAL_REOPEN, &json!({}), Some(user_id))?;
    }
    Ok(())
}

/// Why the automatic close must not close today (PRD FR-21.4 v1.49, plan 85): the day is in use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutoCloseHold {
    /// A reopen to fix an entry, or to save a new one, is in progress (`day_fix_open` without its
    /// `day_fix_close` yet).
    FixInProgress,
    /// The Owner reopened days by hand (`day_reopen`) and has not closed them again.
    ReopenedByHand,
}

/// Whether the automatic close has to leave the open days alone. A fix in progress holds it while the
/// fix still fits the books (the "closed up to" day is the day before its first day) and is younger
/// than [`FIX_HOLD_HOURS`]. A reopen by hand holds it until a close reaches the last day he opened.
/// Reads only.
///
/// # Errors
/// `ServiceError::Db`; `ServiceError::Data` for a date that is not `YYYY-MM-DD`.
pub fn auto_close_hold(ctx: &Context) -> Result<Option<AutoCloseHold>, ServiceError> {
    let now = ctx.now_utc();
    ctx.db.read(|conn| {
        let closed = closed_in(conn)?;
        if let Some(pending) = pending_of(conn)? {
            let fits = closed == pending.first.previous_day();
            let fresh = pending
                .opened_at
                .as_deref()
                .and_then(|text| chrono::DateTime::parse_from_rfc3339(text).ok())
                .is_some_and(|at| {
                    now.signed_duration_since(at) < chrono::Duration::hours(FIX_HOLD_HOURS)
                });
            if fits && fresh {
                return Ok::<_, ServiceError>(Some(AutoCloseHold::FixInProgress));
            }
        }
        let by_hand = manual_reopen_of(conn)?
            .is_some_and(|held| closed.is_none_or(|closed| closed < held.last));
        Ok(by_hand.then_some(AutoCloseHold::ReopenedByHand))
    })
}

/// Whether the Owner's sign opened the fix in progress that reopened the day of an entry (PRD FR-21.4,
/// plan 73): the edit service saves `sign_given: true` beside the reason in the audit entry of the
/// edit. The sign itself is a secret and is never kept (PRD FR-7.71 v1.58, plan 96). False when no fix
/// is in progress, when the record is stale (the closed-up-to day is no longer the day before its
/// first), or when the entry's day is not one the fix reopened: an edit of an open day has no sign.
///
/// # Errors
/// `ServiceError::Db`.
pub(crate) fn fix_sign_given(conn: &Connection, date_iso: &str) -> Result<bool, ServiceError> {
    let Some(pending) = pending_of(conn)? else {
        return Ok(false);
    };
    let (true, Some(date)) = (pending.sign_given, BusinessDate::parse_iso(date_iso)) else {
        return Ok(false);
    };
    if closed_in(conn)? != pending.first.previous_day() {
        return Ok(false);
    }
    Ok(pending.first <= date && date <= pending.last)
}

fn clear_pending(tx: &Connection, user_id: i64) -> Result<(), ServiceError> {
    crate::store::set_setting(tx, KEY_FIX_PENDING, &json!({}), Some(user_id))
}

/// A token nobody can guess: 16 random bytes in hex.
fn new_token(ctx: &Context) -> Result<String, ServiceError> {
    let mut bytes = [0_u8; 16];
    ctx.random.fill(&mut bytes)?;
    Ok(format!(
        "fix-{}",
        bytes.iter().map(|b| format!("{b:02x}")).collect::<String>()
    ))
}

/// The days from `first` to `last`, both included.
fn days_between(first: BusinessDate, last: BusinessDate) -> Vec<BusinessDate> {
    let mut days = Vec::new();
    let mut day = first;
    while day <= last {
        days.push(day);
        match day.next_day() {
            Some(next) => day = next,
            None => break,
        }
    }
    days
}

/// Reopens the date of an entry to fix and every later closed date, and gives a token to close
/// the same days again (PRD FR-21.4, plan 44). It is the reopen of PRD FR-21.2 (a reason, the
/// month-close check, the audit row) and it remembers the days, so [`fix_close`] can close exactly
/// them. A fix already in progress is widened: its days stay in the token.
///
/// Only the Owner's sign opens the day (plan 73): it is checked first, a wrong sign opens nothing
/// and writes an audit entry without the typed text, and the sign is saved with the reason in the
/// audit entry of the reopen and, through the fix record, of every edit saved in this fix.
///
/// The reopen, the fix record and the audit entry are one save (Code Standards B3, plan 80 T3-1):
/// if any of them fails, nothing is reopened.
///
/// It names no entry, only a date: the same call serves an edit of a saved entry and the save of a NEW
/// entry dated on a closed day (PRD FR-21.4 v1.49, plan 85). While the fix is open the automatic close
/// leaves the days alone ([`auto_close_hold`]).
///
/// # Errors
/// `ERR_REASON_REQUIRED` on `reason`; `ERR_SIGN_REQUIRED` and `ERR_SIGN_WRONG` on `sign`; the date
/// codes on `date`; `ERR_DAY_NOT_CLOSED` on `date`; `ERR_MONTH_CLOSED` on `date`.
pub fn fix_open(
    ctx: &Context,
    actor: &Actor,
    input: &DayFixOpenInput,
) -> Result<DayFixOpened, ServiceError> {
    check_sign(
        ctx,
        actor,
        &SignCheck {
            reason: &input.reason,
            sign: &input.sign,
            date: &input.date,
            wrong_action: "day_fix_sign_wrong",
        },
    )?;
    let (first, reason) = parse_reopen(ctx, &input.date, &input.reason)?;
    let opened = ctx.db.write(|tx| {
        let closed_before = closed_in(tx)?;
        // A record of a fix that no longer fits the closed-up-to day is stale (a day was closed or
        // reopened some other way since): it is ignored, so its old last day is not closed again.
        let pending = pending_of(tx)?.filter(|p| closed_before == p.first.previous_day());
        let before = reopen_in(ctx, tx, actor, first, &reason, true)?;
        // A fix in progress keeps its last day: the days between stay open and are closed with it.
        let last = pending.as_ref().map_or(before, |p| p.last.max(before));
        let token = new_token(ctx)?;
        let days: Vec<String> = days_between(first, last)
            .into_iter()
            .map(BusinessDate::to_iso)
            .collect();
        crate::store::set_setting(
            tx,
            KEY_FIX_PENDING,
            &json!({
                "token": token,
                "first": first.to_iso(),
                "last": last.to_iso(),
                "sign_given": true,
                "opened_at": ctx.now_text(),
            }),
            Some(actor.user_id),
        )?;
        ctx.db.auditor().write(
            tx,
            &Event::new(Some(actor.user_id), "day_fix_open", "day")
                .new_value(json!({ "days": days }))
                .sign_given(true)
                .reason(&reason)
                .into_entry(),
        )?;
        Ok::<(String, Vec<String>), ServiceError>((token, days))
    })?;
    Ok(DayFixOpened {
        token: opened.0,
        days: opened.1,
        closed_up_to: status(ctx)?.closed_up_to,
    })
}

/// What [`check_sign`] looks at.
pub(crate) struct SignCheck<'a> {
    pub(crate) reason: &'a str,
    pub(crate) sign: &'a str,
    pub(crate) date: &'a str,
    /// The audit action written when the sign is wrong.
    pub(crate) wrong_action: &'static str,
}

/// Checks the reason and the sign of a reopen (PRD FR-21.2, FR-21.4, plan 73, plan 80 Q-R2-2) and
/// says nothing more than yes: the sign is a secret and is never saved (PRD FR-7.71 v1.58, plan 96).
/// A missing reason or sign is refused first, both together. A wrong sign writes one audit entry that
/// holds no typed text, and refuses with `ERR_SIGN_WRONG`: nothing opens.
pub(crate) fn check_sign(
    ctx: &Context,
    actor: &Actor,
    check: &SignCheck<'_>,
) -> Result<(), ServiceError> {
    let mut errors = validate::FieldErrors::default();
    if check.reason.trim().is_empty() {
        errors.push(FieldError::new(ErrorCode::ErrReasonRequired, "reason"));
    }
    if check.sign.trim().is_empty() {
        errors.push(FieldError::new(ErrorCode::ErrSignRequired, "sign"));
    }
    errors.finish()?;
    let configured = ctx.db.read(crate::owner_sign::configured)?;
    if crate::owner_sign::accepted(check.sign, configured.as_deref()).is_some() {
        return Ok(());
    }
    // The refused try is audited on its own: the refusal must not roll the entry back.
    ctx.db.write(|tx| {
        ctx.db.auditor().write(
            tx,
            &Event::new(Some(actor.user_id), check.wrong_action, "day")
                .new_value(
                    json!({ "date": check.date.trim().chars().take(10).collect::<String>() }),
                )
                .into_entry(),
        )?;
        Ok::<(), ServiceError>(())
    })?;
    Err(ServiceError::reject_field(ErrorCode::ErrSignWrong, "sign"))
}

/// Closes the days that `fix_open` reopened, oldest first, each with the checks of PRD FR-21.1. A
/// day whose checks fail stays open and is listed with the reason; the days after it stay open
/// because days are closed in order. A day that was closed or opened some other way since the
/// token was given makes the token invalid.
///
/// The days closed, the clearing of the fix record and the audit entry are one save (Code Standards
/// B3, plan 80 T3-2). A day's refusal comes before any write of that day, so the days before it stay
/// closed and the fix record is cleared with them; any other error undoes the whole save.
///
/// # Errors
/// `ERR_FIX_TOKEN_INVALID` on `token`.
pub fn fix_close(
    ctx: &Context,
    actor: &Actor,
    input: &DayFixCloseInput,
) -> Result<DayFixClosed, ServiceError> {
    let invalid = || ServiceError::reject_field(ErrorCode::ErrFixTokenInvalid, "token");
    let (closed, stayed_open) = ctx.db.write(|tx| {
        let closed_now = closed_in(tx)?;
        let pending = pending_of(tx)?.ok_or_else(invalid)?;
        // The days must still be exactly as the reopen left them: the day before the first is the
        // last closed one.
        if pending.token != input.token || closed_now != pending.first.previous_day() {
            return Err(invalid());
        }
        let mut closed = Vec::new();
        let mut stayed_open: Vec<DayFixStayedOpen> = Vec::new();
        for day in days_between(pending.first, pending.last) {
            if !stayed_open.is_empty() {
                stayed_open.push(DayFixStayedOpen {
                    date: day.to_iso(),
                    errors: Vec::new(),
                });
                continue;
            }
            let result = close_in(
                ctx,
                tx,
                actor,
                &DayCloseInput {
                    up_to: mandi_domain::core::dates::format_ddmmyyyy(day),
                },
            );
            match result {
                Ok(()) => closed.push(day.to_iso()),
                Err(ServiceError::Rejected(items)) if !items.is_empty() => {
                    stayed_open.push(DayFixStayedOpen {
                        date: day.to_iso(),
                        errors: items
                            .into_iter()
                            .map(|r| mandi_domain::core::error::FieldError {
                                code: r.code,
                                field: r.field.unwrap_or_default(),
                                values: r.values,
                            })
                            .collect(),
                    });
                }
                Err(other) => return Err(other),
            }
        }
        clear_pending(tx, actor.user_id)?;
        ctx.db.auditor().write(
            tx,
            &Event::new(Some(actor.user_id), "day_fix_close", "day")
                .new_value(json!({
                    "closed": closed,
                    "stayed_open": stayed_open.iter().map(|d| d.date.clone()).collect::<Vec<_>>(),
                }))
                .into_entry(),
        )?;
        Ok::<(Vec<String>, Vec<DayFixStayedOpen>), ServiceError>((closed, stayed_open))
    })?;
    Ok(DayFixClosed {
        closed,
        stayed_open,
        closed_up_to: status(ctx)?.closed_up_to,
    })
}
