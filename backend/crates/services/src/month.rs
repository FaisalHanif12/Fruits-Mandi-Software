//! Month close and its reversal (plan 22 round 7, PRD FR-17.6, FR-21.2, FR-17.9, Architecture
//! diagram 10).
//!
//! The Owner closes a finished month. One journal entry (posting type and source type
//! `month_close`) brings every income account (04) and every shop expense account (05) of the
//! month to 0 and puts each partner's share of the month on his capital account (0602):
//!
//! - an income account with a net credit gets a debit of that net (a net debit gets a credit);
//! - an expense account with a net debit gets a credit of that net (a net credit gets a debit);
//! - a partner whose share rows of the month add up to more than 0 gets a credit of that sum
//!   (a debit of its size if the sum is below 0); a partner with a sum of 0 gets no line.
//!
//! The entry balances because the shares of a month add up to its income less its expenses
//! (PRD FR-6.5); `build_entry` checks it and a difference writes nothing. A month close writes no
//! partner share rows and does not change any partner's `کل رقم` (PRD FR-17.9, C22): it only
//! moves amounts between his accounts.
//!
//! The month is the entries dated in it, posted, that are not month-close entries. The entry is
//! DATED TODAY: the days of the month are closed by then (they must be, PRD FR-17.6), and the
//! open-day model (FR-6.6) forbids a posting in a closed day. Reports that read by date
//! (R28, R30, the partner statement) leave month-close entries out, so the month still shows
//! its income and expenses.
//!
//! The close is refused while the month is not over (`ERR_DATE_IN_FUTURE`), while one of its
//! days is open (`ERR_DAY_NOT_CLOSED`, value `closed_up_to`), when it is closed already
//! (`ERR_MONTH_CLOSED`) and when there is nothing to move (`ERR_NOTHING_TO_CLOSE`). The
//! reversal (FR-21.2) is a mirror entry dated today with a reason, and frees the month for a new
//! close and for reopening its days.
//!
//! Example (PRD Section 4, October 2026 after Events 1 to 5): debits 04010001 2,484, 04010002
//! 2,484, 04010003 480, 04010004 40; credits 05010001 3,000, Alif 996, Bay 746, Jeem 746; each
//! side 5,488.

use mandi_db::ledger as ledger_repo;
use mandi_db::month as repo;
use mandi_db::rusqlite::Connection;
use mandi_domain::core::accounts::RecordRef;
use mandi_domain::core::dates::BusinessDate;
use mandi_domain::core::error::ErrorCode;
use mandi_domain::core::ids::{AccountId, EntryId};
use mandi_domain::core::money::Paisa;
use mandi_domain::core::validate;
use mandi_domain::ledger::{
    BalancedEntry, LedgerLine, PostingType, Side, SourceType, build_entry, build_reversal,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use ts_rs::TS;

use crate::audit::Event;
use crate::auth::Actor;
use crate::context::Context;
use crate::day;
use crate::error::{Rejection, ServiceError, money_error};
use crate::ledger::{self, PostingEvent};

/// The input of the preview and of the close.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct MonthInput {
    /// The month, `YYYY-MM`.
    pub month: String,
}

/// The input of the reversal.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct MonthReverseInput {
    /// The month, `YYYY-MM`.
    pub month: String,
    /// Why (required, goes to the audit log).
    pub reason: String,
}

/// What a line of the close is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum MonthLineKind {
    /// An income account brought to 0.
    Income,
    /// A shop expense account brought to 0.
    Expense,
    /// A partner's capital account.
    PartnerCapital,
}

/// One line of the close entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct MonthLine {
    /// The account.
    pub account: RecordRef,
    /// What it is.
    pub kind: MonthLineKind,
    /// The partner, for a capital line.
    #[ts(type = "number | null")]
    pub partner_id: Option<i64>,
    /// The debit.
    #[ts(type = "Paisa")]
    pub dr_paisa: Paisa,
    /// The credit.
    #[ts(type = "Paisa")]
    pub cr_paisa: Paisa,
}

/// What the close of a month would post.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct MonthPreview {
    /// The month, `YYYY-MM`.
    pub month: String,
    /// Its first day.
    pub first_day: String,
    /// Its last day.
    pub last_day: String,
    /// The date the entry will have: today.
    pub entry_date: String,
    /// The lines, in the order they are posted.
    pub lines: Vec<MonthLine>,
    /// The income of the month (credits less debits of the income accounts).
    #[ts(type = "Paisa")]
    pub income_paisa: Paisa,
    /// The shop expenses of the month.
    #[ts(type = "Paisa")]
    pub expense_paisa: Paisa,
    /// Income less expenses: what is shared between the partners.
    #[ts(type = "Paisa")]
    pub profit_paisa: Paisa,
    /// The debit total of the entry, equal to its credit total.
    #[ts(type = "Paisa")]
    pub total_paisa: Paisa,
}

/// A month closed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct MonthClosed {
    /// The month.
    pub month: String,
    /// The close entry.
    #[ts(type = "number")]
    pub entry_id: i64,
    /// Its total.
    #[ts(type = "Paisa")]
    pub total_paisa: Paisa,
}

/// A month close reversed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct MonthReversed {
    /// The month.
    pub month: String,
    /// The entry that reversed the close.
    #[ts(type = "number")]
    pub reversal_entry_id: i64,
}

/// Where a month stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum MonthState {
    /// Closed, and in force.
    Closed,
    /// Over, its days closed, not closed yet.
    Ready,
    /// Over, but some of its days are still open: close the days first.
    DaysOpen,
    /// Not over yet.
    NotFinished,
}

/// One month of the list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct MonthRow {
    /// The month, `YYYY-MM`.
    pub month: String,
    /// Where it stands.
    pub state: MonthState,
    /// The day the close was made, when it is closed.
    pub closed_on: Option<String>,
    /// The close entry, when it is closed.
    #[ts(type = "number | null")]
    pub closed_entry_id: Option<i64>,
    /// How many times a close of this month was reversed.
    #[ts(type = "number")]
    pub reversed_count: i64,
}

/// The months from the first entry to this month, newest first.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct MonthList {
    /// The months.
    pub rows: Vec<MonthRow>,
}

/// A month text read: its first and last day.
struct Span {
    month: String,
    first: BusinessDate,
    last: BusinessDate,
}

fn span_of(text: &str) -> Result<Span, ServiceError> {
    let bad = || ServiceError::reject_field(ErrorCode::ErrFormatInvalid, "month");
    let text = text.trim();
    let (year, month) = text.split_once('-').ok_or_else(bad)?;
    if year.len() != 4 || month.len() != 2 {
        return Err(bad());
    }
    let year: i32 = year.parse().map_err(|_| bad())?;
    let month: u32 = month.parse().map_err(|_| bad())?;
    let first = BusinessDate::from_ymd(year, month, 1).ok_or_else(bad)?;
    let (next_year, next_month) = if month == 12 {
        (year.checked_add(1).ok_or_else(bad)?, 1)
    } else {
        (year, month.checked_add(1).ok_or_else(bad)?)
    };
    let last = BusinessDate::from_ymd(next_year, next_month, 1)
        .and_then(BusinessDate::previous_day)
        .ok_or_else(bad)?;
    Ok(Span {
        month: format!("{year:04}-{month:02}"),
        first,
        last,
    })
}

fn paisa(value: i64) -> Result<Paisa, ServiceError> {
    Paisa::try_from_paisa(value).map_err(money_error)
}

fn record(account_id: i64, code: &str, name_ur: &str, name_en: &str) -> RecordRef {
    RecordRef {
        id: account_id,
        code: code.to_owned(),
        name_ur: name_ur.to_owned(),
        name_en: name_en.to_owned(),
        // The accounts of a month close are income, expense and capital accounts: none has a tag.
        tag: None,
    }
}

/// The lines of the close and the figures behind them, from one read of the books.
struct Plan {
    lines: Vec<MonthLine>,
    income: Paisa,
    expense: Paisa,
    entry: BalancedEntry,
}

/// A line that brings `net` (a signed amount) to 0: it goes on the side opposite to the balance.
/// `positive_is_debit_line` says a positive `net` is closed by a debit line (an income account
/// with a credit balance) and a negative `net` by a credit line; false is the other way round (an
/// expense account with a debit balance, a partner's capital).
fn closing_line(
    account: RecordRef,
    kind: MonthLineKind,
    net: Paisa,
    positive_is_debit_line: bool,
) -> Result<Option<MonthLine>, ServiceError> {
    if net == Paisa::ZERO {
        return Ok(None);
    }
    let size = if net.is_negative() {
        net.checked_neg().map_err(money_error)?
    } else {
        net
    };
    // A positive net is closed by a line on `positive_is_debit_line`'s side; a negative net by the
    // other side.
    let debit = positive_is_debit_line == !net.is_negative();
    let (dr, cr) = if debit {
        (size, Paisa::ZERO)
    } else {
        (Paisa::ZERO, size)
    };
    Ok(Some(MonthLine {
        account,
        kind,
        partner_id: None,
        dr_paisa: dr,
        cr_paisa: cr,
    }))
}

/// The one plan both the preview and the close use (Code Standards B5).
fn plan(conn: &Connection, span: &Span) -> Result<Plan, ServiceError> {
    let (first, last) = (span.first.to_iso(), span.last.to_iso());
    let mut lines = Vec::new();
    let (mut income, mut expense) = (Paisa::ZERO, Paisa::ZERO);
    for net in repo::head_nets(conn, &first, &last)? {
        let (dr, cr) = (paisa(net.dr_paisa)?, paisa(net.cr_paisa)?);
        let account = record(net.account_id, &net.code, &net.name_ur, &net.name_en);
        if net.head_code == "04" {
            // Income: credits less debits. A net credit is closed by a debit.
            let balance = cr.checked_sub(dr).map_err(money_error)?;
            income = income.checked_add(balance).map_err(money_error)?;
            lines.extend(closing_line(account, MonthLineKind::Income, balance, true)?);
        } else {
            // Shop expense: debits less credits. A net debit is closed by a credit.
            let balance = dr.checked_sub(cr).map_err(money_error)?;
            expense = expense.checked_add(balance).map_err(money_error)?;
            lines.extend(closing_line(
                account,
                MonthLineKind::Expense,
                balance,
                false,
            )?);
        }
    }
    let profit = income.checked_sub(expense).map_err(money_error)?;
    let mut shares_sum = Paisa::ZERO;
    for partner in repo::partner_months(conn, &first, &last)? {
        let share = paisa(partner.total_paisa)?;
        shares_sum = shares_sum.checked_add(share).map_err(money_error)?;
        if let Some(mut line) = closing_line(
            record(
                partner.capital_account_id,
                &partner.capital_code,
                &partner.name_ur,
                &partner.name_en,
            ),
            MonthLineKind::PartnerCapital,
            share,
            // A share above 0 is a credit on his capital account.
            false,
        )? {
            line.partner_id = Some(partner.partner_id);
            lines.push(line);
        }
    }
    // "Nothing to close" means no line at all. A month whose profit is 0 still moves amounts.
    if lines.is_empty() {
        return Err(ServiceError::reject_field(
            ErrorCode::ErrNothingToClose,
            "month",
        ));
    }
    // The shares of a month add up to its income less its expenses (PRD FR-6.5). Checked here
    // with both numbers in the log; `build_entry` below is the second guard.
    let shared = profit.checked_sub(shares_sum).map_err(money_error)?;
    if shared != Paisa::ZERO {
        tracing::error!(
            month = %span.month,
            income_less_expenses = profit.as_paisa(),
            partner_sums = shares_sum.as_paisa(),
            "the partner shares of the month do not match its income and expenses"
        );
        return Err(ServiceError::reject(ErrorCode::ErrJournalUnbalanced));
    }
    let entry = build_entry(
        lines
            .iter()
            .map(|line| {
                let (side, amount) = if line.dr_paisa > Paisa::ZERO {
                    (Side::Dr, line.dr_paisa)
                } else {
                    (Side::Cr, line.cr_paisa)
                };
                LedgerLine::new(AccountId::new(line.account.id), side, amount)
            })
            .collect(),
    )
    .map_err(ledger::ledger_error)?;
    Ok(Plan {
        lines,
        income,
        expense,
        entry,
    })
}

/// A rejection of the month with one number in its message.
fn month_rejection(code: ErrorCode, key: &str, value: &str) -> ServiceError {
    ServiceError::Rejected(vec![Rejection {
        code,
        field: Some("month".to_owned()),
        values: std::collections::BTreeMap::from([(
            key.to_owned(),
            mandi_domain::core::error::ErrorValue::Text(value.to_owned()),
        )]),
    }])
}

/// The checks every close needs, inside the caller's connection: the month is over, all its days
/// are closed, and no close is in force for it.
fn check_ready(conn: &Connection, span: &Span, today: BusinessDate) -> Result<(), ServiceError> {
    if span.last >= today {
        return Err(ServiceError::reject_field(
            ErrorCode::ErrDateInFuture,
            "month",
        ));
    }
    let closed = repo::closed_up_to(conn)?
        .as_deref()
        .and_then(BusinessDate::parse_iso);
    if !closed.is_some_and(|c| c >= span.last) {
        let shown = closed.map_or_else(String::new, BusinessDate::to_iso);
        return Err(month_rejection(
            ErrorCode::ErrDayNotClosed,
            "closed_up_to",
            &shown,
        ));
    }
    // The entry is dated today, so today must still be open (PRD FR-6.6).
    if closed.is_some_and(|c| c >= today) {
        return Err(ServiceError::reject_field(
            ErrorCode::ErrDateClosed,
            "month",
        ));
    }
    if repo::active(conn, &span.month)?.is_some() {
        return Err(ServiceError::reject_field(
            ErrorCode::ErrMonthClosed,
            "month",
        ));
    }
    Ok(())
}

fn preview_of(plan: &Plan, span: &Span, today: BusinessDate) -> Result<MonthPreview, ServiceError> {
    Ok(MonthPreview {
        month: span.month.clone(),
        first_day: span.first.to_iso(),
        last_day: span.last.to_iso(),
        entry_date: today.to_iso(),
        lines: plan.lines.clone(),
        income_paisa: plan.income,
        expense_paisa: plan.expense,
        profit_paisa: plan.income.checked_sub(plan.expense).map_err(money_error)?,
        total_paisa: plan.entry.total(),
    })
}

/// What closing a month would post. Refused for the same reasons as the close itself, so the
/// screen can say why before the Owner presses the button. Reads only.
///
/// # Errors
/// `ERR_FORMAT_INVALID`, `ERR_DATE_IN_FUTURE`, `ERR_DAY_NOT_CLOSED` (value `closed_up_to`),
/// `ERR_MONTH_CLOSED`, `ERR_NOTHING_TO_CLOSE`, all on `month`; `ERR_JOURNAL_UNBALANCED` if the
/// shares do not match the month (a bug signal).
pub fn preview(ctx: &Context, input: &MonthInput) -> Result<MonthPreview, ServiceError> {
    let span = span_of(&input.month)?;
    let (_, today) = day::window(ctx)?;
    ctx.db.read(|conn| {
        check_ready(conn, &span, today)?;
        let plan = plan(conn, &span)?;
        preview_of(&plan, &span, today)
    })
}

/// Closes a finished month: posts the close entry, notes it, writes the audit row, all in one
/// transaction (PRD FR-17.6).
///
/// # Errors
/// As [`preview`].
pub fn close(
    ctx: &Context,
    actor: &Actor,
    input: &MonthInput,
) -> Result<MonthClosed, ServiceError> {
    let span = span_of(&input.month)?;
    let (_, today) = day::window(ctx)?;
    ctx.db.write(|tx| {
        // Everything is read inside the transaction that writes, so nothing moves in between.
        check_ready(tx, &span, today)?;
        let plan = plan(tx, &span)?;
        let total = plan.entry.total();
        let posted = ledger::post(
            ctx,
            tx,
            actor.user_id,
            &PostingEvent {
                date: today,
                posting_type: PostingType::MonthClose,
                source_type: SourceType::MonthClose,
                source_id: source_id_of(&span),
                narration: Some(span.month.clone()),
                reverses: None,
                entry: plan.entry,
            },
        )?;
        repo::insert(
            tx,
            &span.month,
            posted.entry_id,
            actor.user_id,
            &ctx.now_text(),
        )?;
        ctx.db.auditor().write(
            tx,
            &Event::new(Some(actor.user_id), "month_close", "month")
                .new_value(json!({
                    "month": span.month,
                    "entry_id": posted.entry_id,
                    "total_paisa": total.as_paisa(),
                    "lines": plan.lines.len(),
                }))
                .into_entry(),
        )?;
        Ok(MonthClosed {
            month: span.month.clone(),
            entry_id: posted.entry_id,
            total_paisa: total,
        })
    })
}

/// The source id of a close entry: the month as a number (`202610`), so every close of a month
/// points at the same place.
fn source_id_of(span: &Span) -> i64 {
    let year = i64::from(span.first.year());
    let month = i64::from(span.first.month());
    year.saturating_mul(100).saturating_add(month)
}

/// Reverses the close in force for a month (PRD FR-21.2): a mirror entry dated today. The month
/// can then be closed again and its days reopened. A reason is required.
///
/// # Errors
/// `ERR_FORMAT_INVALID` on `month`; `ERR_REASON_REQUIRED` (or a text code) on `reason`;
/// `ERR_NOT_FOUND` on `month` when no close is in force.
pub fn reverse(
    ctx: &Context,
    actor: &Actor,
    input: &MonthReverseInput,
) -> Result<MonthReversed, ServiceError> {
    let span = span_of(&input.month)?;
    let reason = validate::free_text("reason", &input.reason)?;
    if reason.is_empty() {
        return Err(ServiceError::reject_field(
            ErrorCode::ErrReasonRequired,
            "reason",
        ));
    }
    let (_, today) = day::window(ctx)?;
    ctx.db.write(|tx| {
        let found = repo::active(tx, &span.month)?
            .ok_or_else(|| ServiceError::reject_field(ErrorCode::ErrNotFound, "month"))?;
        let stored = ledger_repo::get_entry(tx, found.journal_entry_id)?.ok_or_else(|| {
            ServiceError::Data(format!(
                "month close entry {} is missing",
                found.journal_entry_id
            ))
        })?;
        let mut lines = Vec::with_capacity(stored.lines.len());
        for line in &stored.lines {
            let (side, amount) = if line.dr_paisa > 0 {
                (Side::Dr, line.dr_paisa)
            } else {
                (Side::Cr, line.cr_paisa)
            };
            lines.push(LedgerLine::new(
                AccountId::new(line.account_id),
                side,
                paisa(amount)?,
            ));
        }
        let original = build_entry(lines).map_err(ledger::ledger_error)?;
        let posted = ledger::post(
            ctx,
            tx,
            actor.user_id,
            &PostingEvent {
                date: today,
                posting_type: PostingType::Reversal,
                source_type: SourceType::MonthClose,
                source_id: source_id_of(&span),
                narration: Some(reason.clone()),
                reverses: Some(EntryId::new(found.journal_entry_id)),
                entry: build_reversal(&original),
            },
        )?;
        repo::deactivate(
            tx,
            found.id,
            posted.entry_id,
            actor.user_id,
            &ctx.now_text(),
        )?;
        ctx.db.auditor().write(
            tx,
            &Event::new(Some(actor.user_id), "month_close_reverse", "month")
                .old(json!({ "month": span.month, "entry_id": found.journal_entry_id }))
                .new_value(json!({ "reversal_entry_id": posted.entry_id }))
                .reason(&reason)
                .into_entry(),
        )?;
        Ok(MonthReversed {
            month: span.month.clone(),
            reversal_entry_id: posted.entry_id,
        })
    })
}

/// The months from the first entry to the current month, newest first, each with where it
/// stands. Reads only.
///
/// # Errors
/// `ServiceError::Db` or `ServiceError::Data`.
pub fn list(ctx: &Context) -> Result<MonthList, ServiceError> {
    let (_, today) = day::window(ctx)?;
    ctx.db.read(|conn| {
        let closed = repo::closed_up_to(conn)?
            .as_deref()
            .and_then(BusinessDate::parse_iso);
        let closes = repo::closes(conn)?;
        let Some(first) = repo::first_month(conn)? else {
            return Ok(MonthList { rows: Vec::new() });
        };
        let mut cursor = span_of(&first)?;
        let current = span_of(&format!("{:04}-{:02}", today.year(), today.month()))?;
        let mut rows = Vec::new();
        loop {
            let in_force = closes
                .iter()
                .find(|c| c.month == cursor.month && c.is_active);
            let reversed_count = closes
                .iter()
                .filter(|c| c.month == cursor.month && !c.is_active)
                .count();
            let state = if in_force.is_some() {
                MonthState::Closed
            } else if cursor.last >= today {
                MonthState::NotFinished
            } else if !closed.is_some_and(|c| c >= cursor.last) {
                MonthState::DaysOpen
            } else {
                MonthState::Ready
            };
            rows.push(MonthRow {
                month: cursor.month.clone(),
                state,
                closed_on: in_force.map(|c| c.entry_date.clone()),
                closed_entry_id: in_force.map(|c| c.journal_entry_id),
                reversed_count: i64::try_from(reversed_count).unwrap_or(i64::MAX),
            });
            if cursor.month >= current.month {
                break;
            }
            let next = cursor
                .last
                .next_day()
                .ok_or_else(|| ServiceError::Data("the month after the last".to_owned()))?;
            cursor = span_of(&format!("{:04}-{:02}", next.year(), next.month()))?;
        }
        rows.reverse();
        Ok(MonthList { rows })
    })
}
