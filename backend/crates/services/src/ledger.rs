//! The one way any module writes to the books (PRD FR-6.2, FR-6.3, plan 07): post an entry that
//! was built and balanced in `mandi_domain::ledger`, reverse a posted entry, and read balances.
//!
//! - [`post`] inserts the entry as a draft with its lines, then makes ONE update that sets
//!   `posted`, the total and the keyed chain hash, so the database trigger checks the balance
//!   too (Code Standards B16). It is the only function that writes journal rows; vouchers, the
//!   بکری, openings and month close all call it, inside their own transaction.
//! - A posted entry never changes. [`reverse`] writes its mirror on the same date.
//! - Reads look at posted entries only (PRD FR-6.1). Sums are `SUM()` on INTEGER columns in SQL;
//!   the differences are worked out here with checked maths (money agent Section 9).

use std::collections::HashMap;

use mandi_db::accounts as account_repo;
use mandi_db::ledger as repo;
use mandi_db::rusqlite::Connection;
use mandi_domain::core::accounts::{Balance, BalanceSide};
use mandi_domain::core::dates::{BusinessDate, format_ddmmyyyy};
use mandi_domain::core::error::{ErrorCode, ErrorValue};
use mandi_domain::core::ids::{AccountId, EntryId};
use mandi_domain::core::money::Paisa;
use mandi_domain::core::validate;
use mandi_domain::ledger::{
    BalancedEntry, LedgerError, LedgerLine, MAX_LINE_PAISA, PostingType, Side, SourceType,
    build_reversal,
};

use crate::context::Context;
use crate::day;
use crate::error::{Rejection, ServiceError, money_error};

/// Everything needed to post one entry.
#[derive(Debug, Clone)]
pub struct PostingEvent {
    /// The posting date. It must be after the "closed up to" date and not after today.
    pub date: BusinessDate,
    /// What the entry is for.
    pub posting_type: PostingType,
    /// What kind of record it belongs to.
    pub source_type: SourceType,
    /// The id of that record.
    pub source_id: i64,
    /// A note.
    pub narration: Option<String>,
    /// The entry this one reverses (a reversal), if any.
    pub reverses: Option<EntryId>,
    /// The lines, already balanced.
    pub entry: BalancedEntry,
}

/// A posted entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Posted {
    /// The entry id.
    pub entry_id: i64,
    /// The debit total (also the credit total).
    pub total: Paisa,
    /// The keyed hash that chains it to the entry before it.
    pub chain_hash: String,
}

/// Turns a refused entry into the error the user sees. `Unbalanced` is a bug signal: it is
/// logged with both totals and nothing is written.
pub fn ledger_error(error: LedgerError) -> ServiceError {
    match error {
        LedgerError::Unbalanced { dr, cr } => {
            tracing::error!(
                dr = dr.as_paisa(),
                cr = cr.as_paisa(),
                "a journal entry does not balance; nothing was written"
            );
            ServiceError::reject(ErrorCode::ErrJournalUnbalanced)
        }
        LedgerError::TooLarge => ServiceError::Rejected(vec![Rejection {
            code: ErrorCode::ErrAmountTooLarge,
            field: None,
            values: std::collections::BTreeMap::from([(
                "max".to_owned(),
                ErrorValue::Number(MAX_LINE_PAISA.checked_div(100).unwrap_or(i64::MAX)),
            )]),
        }]),
        LedgerError::Overflow => ServiceError::reject(ErrorCode::ErrOverflow),
        other => ServiceError::Data(format!("a journal entry was built wrong: {other}")),
    }
}

/// Checks a posting date against the open-day rule (PRD FR-6.6).
///
/// # Errors
/// `ERR_DATE_CLOSED` (value `closed_up_to`) or `ERR_DATE_IN_FUTURE` (value `today`).
pub fn check_posting_date(ctx: &Context, date: BusinessDate) -> Result<(), ServiceError> {
    let (closed, today) = day::window(ctx)?;
    validate::posting_date("entry_date", &format_ddmmyyyy(date), closed, today)?;
    Ok(())
}

/// A line on a پختہ کھاتہ account must name its person, and a line on any other account must
/// not (PRD FR-13.5): balances, matching and aging are kept per person there. The checks are
/// bug signals, so they are `ServiceError::Data`, not user errors.
fn check_person_keys(tx: &Connection, entry: &BalancedEntry) -> Result<(), ServiceError> {
    let pukhta = repo::pukhta_khata_ids(tx)?;
    for (index, line) in entry.lines().iter().enumerate() {
        let is_pukhta = pukhta.contains(&line.account_id.get());
        let key = line.person_key.as_deref().map(str::trim);
        let wrong = match key {
            Some("") => true,
            Some(_) => !is_pukhta,
            None => is_pukhta,
        };
        if wrong {
            return Err(ServiceError::Data(format!(
                "line {} of an entry has a wrong person key for account {}",
                index.saturating_add(1),
                line.account_id.get()
            )));
        }
    }
    Ok(())
}

/// Posts an entry inside the caller's transaction `tx`.
///
/// # Errors
/// The date codes of [`check_posting_date`]; `ERR_JOURNAL_UNBALANCED` (a bug signal) if the
/// database refuses the balance; `ServiceError::Db` for anything else. Nothing is left behind
/// when the caller rolls back.
pub fn post(
    ctx: &Context,
    tx: &Connection,
    user_id: i64,
    event: &PostingEvent,
) -> Result<Posted, ServiceError> {
    check_posting_date(ctx, event.date)?;
    check_person_keys(tx, &event.entry)?;
    let now = ctx.now_text();
    let iso = event.date.to_iso();
    let entry_id = repo::insert_draft(
        tx,
        &repo::EntryHeader {
            entry_date: &iso,
            posting_type: event.posting_type.as_str(),
            source_type: event.source_type.as_str(),
            source_id: event.source_id,
            reverses_entry_id: event.reverses.map(EntryId::get),
            narration: event.narration.as_deref(),
        },
        user_id,
        &now,
    )?;
    for (index, line) in event.entry.lines().iter().enumerate() {
        let (dr, cr) = match line.side {
            Side::Dr => (line.amount.as_paisa(), 0),
            Side::Cr => (0, line.amount.as_paisa()),
        };
        repo::insert_line(
            tx,
            entry_id,
            &repo::NewLine {
                line_no: i64::try_from(index.saturating_add(1)).unwrap_or(i64::MAX),
                account_id: line.account_id.get(),
                person_key: line.person_key.as_deref(),
                dr_paisa: dr,
                cr_paisa: cr,
                is_advance: line.is_advance,
                narration: line.narration.as_deref(),
            },
            user_id,
            &now,
        )?;
    }
    let key = ctx.db.auditor().key()?;
    let chain_hash = repo::post(
        tx,
        &key,
        entry_id,
        event.entry.total().as_paisa(),
        user_id,
        &now,
    )
    .map_err(|error| match error {
        mandi_db::DbError::JournalUnbalanced => {
            tracing::error!(entry_id, "the database refused an unbalanced entry");
            ServiceError::reject(ErrorCode::ErrJournalUnbalanced)
        }
        other => ServiceError::Db(other),
    })?;
    Ok(Posted {
        entry_id,
        total: event.entry.total(),
        chain_hash,
    })
}

/// Reverses a posted entry: its mirror, on the same date, with the same source (PRD FR-6.3,
/// FR-12.19). The original date must still be open. An entry can be reversed once.
///
/// # Errors
/// The date codes of [`check_posting_date`] for the original date; `ERR_NOT_FOUND` on `id` if
/// there is no posted entry; `ServiceError::Data` if it is already reversed or is itself a
/// reversal (a bug in the caller).
pub fn reverse(
    ctx: &Context,
    tx: &Connection,
    user_id: i64,
    entry_id: i64,
    reason: &str,
) -> Result<Posted, ServiceError> {
    let stored = repo::get_entry(tx, entry_id)?
        .filter(|e| e.status == "posted")
        .ok_or_else(|| ServiceError::reject_field(ErrorCode::ErrNotFound, "id"))?;
    if stored.reverses_entry_id.is_some() || repo::is_reversed(tx, entry_id)? {
        return Err(ServiceError::Data(format!(
            "entry {entry_id} is a reversal or is already reversed"
        )));
    }
    let date = BusinessDate::parse_iso(&stored.entry_date)
        .ok_or_else(|| ServiceError::Data(format!("entry date {}", stored.entry_date)))?;
    let mut lines = Vec::with_capacity(stored.lines.len());
    for line in &stored.lines {
        let (side, amount) = if line.dr_paisa > 0 {
            (Side::Dr, line.dr_paisa)
        } else {
            (Side::Cr, line.cr_paisa)
        };
        let mut built = LedgerLine::new(
            AccountId::new(line.account_id),
            side,
            Paisa::try_from_paisa(amount).map_err(money_error)?,
        );
        built.person_key.clone_from(&line.person_key);
        built.is_advance = line.is_advance;
        built.narration.clone_from(&line.narration);
        lines.push(built);
    }
    let original = mandi_domain::ledger::build_entry(lines).map_err(ledger_error)?;
    let mirror = build_reversal(&original);
    let source_type = SourceType::parse(&stored.source_type)
        .ok_or_else(|| ServiceError::Data(format!("source type {}", stored.source_type)))?;
    let event = PostingEvent {
        date,
        posting_type: PostingType::Reversal,
        source_type,
        source_id: stored.source_id,
        narration: Some(reason.to_owned()),
        reverses: Some(EntryId::new(entry_id)),
        entry: mirror,
    };
    post(ctx, tx, user_id, &event)
}

/// Checks the whole journal chain. Returns the id of the first entry that does not match as a
/// `ServiceError::Data`.
///
/// # Errors
/// `ServiceError::Data` naming the entry; `ServiceError::Db`.
pub fn verify_journal_chain(ctx: &Context) -> Result<(), ServiceError> {
    let key = ctx.db.auditor().key()?;
    ctx.db.read(|conn| {
        repo::verify_journal_chain(conn, &key).map_err(|error| match error {
            repo::JournalVerifyError::Broken(found) => ServiceError::Data(found.to_string()),
            repo::JournalVerifyError::Db(db) => ServiceError::Db(db),
        })
    })
}

fn paisa(value: i64) -> Result<Paisa, ServiceError> {
    Paisa::try_from_paisa(value).map_err(money_error)
}

/// The balance of posted totals: debits minus credits with the side it stands on (PRD C13).
pub fn balance_of(totals: repo::Totals) -> Result<Balance, ServiceError> {
    Balance::from_totals(paisa(totals.dr_paisa)?, paisa(totals.cr_paisa)?).map_err(money_error)
}

/// The balance of one account now, from the posted journal, all persons of a پختہ کھاتہ account
/// together (PRD FR-6.1).
///
/// # Errors
/// `ServiceError::Db`.
pub fn account_balance(ctx: &Context, account_id: i64) -> Result<Balance, ServiceError> {
    ctx.db.read(|conn| balance_in(conn, account_id, None, None))
}

/// The balance of one person of a پختہ کھاتہ account (PRD FR-13.5).
///
/// # Errors
/// `ServiceError::Db`.
pub fn person_balance(
    ctx: &Context,
    account_id: i64,
    person_key: &str,
) -> Result<Balance, ServiceError> {
    ctx.db
        .read(|conn| balance_in(conn, account_id, Some(person_key), None))
}

/// The balance of an account inside a read, optionally for one person and up to a date.
pub(crate) fn balance_in(
    conn: &Connection,
    account_id: i64,
    person: Option<&str>,
    as_of: Option<&str>,
) -> Result<Balance, ServiceError> {
    balance_of(repo::totals_of(conn, account_id, person, as_of)?)
}

/// All account balances as a map, for lists and the chart (one query for all accounts).
pub(crate) fn balances_by_account(
    conn: &Connection,
    as_of: Option<&str>,
) -> Result<HashMap<i64, repo::Totals>, ServiceError> {
    Ok(repo::account_totals(conn, as_of)?.into_iter().collect())
}

/// A date given to a query must be `YYYY-MM-DD`: SQL compares it as text, so a wrong form would
/// give a wrong range without any error.
fn check_iso_date(date: Option<&str>) -> Result<(), ServiceError> {
    match date {
        Some(text) if BusinessDate::parse_iso(text).is_none() => {
            Err(ServiceError::Data(format!("date {text}")))
        }
        _ => Ok(()),
    }
}

/// One account in the trial balance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrialRow {
    /// The account.
    pub account_id: i64,
    /// Its code.
    pub code: String,
    /// Urdu name.
    pub name_ur: String,
    /// English name.
    pub name_en: String,
    /// Its balance and side. An account with lines whose balance is zero is listed with zero.
    pub balance: Balance,
}

/// The trial balance (ٹرائل بیلنس, PRD R18): the balance of every account that has posted lines,
/// the debit total and the credit total.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrialBalance {
    /// By account code.
    pub rows: Vec<TrialRow>,
    /// Σ of the debit balances.
    pub total_dr: Paisa,
    /// Σ of the credit balances.
    pub total_cr: Paisa,
}

/// The trial balance up to `as_of` (`YYYY-MM-DD`, included), or all time. The two totals must be
/// equal (NFR-REL-8). If they are not, that is a bug: the function logs it and returns an error;
/// it never adjusts a total.
///
/// # Errors
/// `ServiceError::Data` if the totals differ; `ServiceError::Db`.
pub fn trial_balance(ctx: &Context, as_of: Option<&str>) -> Result<TrialBalance, ServiceError> {
    check_iso_date(as_of)?;
    ctx.db.read(|conn| {
        let totals = balances_by_account(conn, as_of)?;
        let mut rows = Vec::new();
        let (mut total_dr, mut total_cr) = (Paisa::ZERO, Paisa::ZERO);
        for account in account_repo::tree_accounts(conn)? {
            let Some(found) = totals.get(&account.id) else {
                continue;
            };
            let balance = balance_of(*found)?;
            match balance.side {
                BalanceSide::Dr => {
                    total_dr = total_dr
                        .checked_add(balance.amount_paisa)
                        .map_err(money_error)?;
                }
                BalanceSide::Cr => {
                    total_cr = total_cr
                        .checked_add(balance.amount_paisa)
                        .map_err(money_error)?;
                }
                BalanceSide::Zero => {}
            }
            rows.push(TrialRow {
                account_id: account.id,
                code: account.code,
                name_ur: account.name_ur,
                name_en: account.name_en,
                balance,
            });
        }
        if total_dr != total_cr {
            tracing::error!(
                dr = total_dr.as_paisa(),
                cr = total_cr.as_paisa(),
                "the trial balance does not balance"
            );
            return Err(ServiceError::Data(format!(
                "the trial balance does not balance: {} against {}",
                total_dr.as_paisa(),
                total_cr.as_paisa()
            )));
        }
        Ok(TrialBalance {
            rows,
            total_dr,
            total_cr,
        })
    })
}

/// One line of an account ledger with the balance after it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LedgerEntry {
    /// The entry.
    pub entry_id: i64,
    /// Its date, `YYYY-MM-DD`.
    pub entry_date: String,
    /// Line number in the entry.
    pub line_no: i64,
    /// The posting type, as stored.
    pub posting_type: String,
    /// The note.
    pub narration: Option<String>,
    /// The person inside a پختہ کھاتہ account.
    pub person_key: Option<String>,
    /// Debit.
    pub dr: Paisa,
    /// Credit.
    pub cr: Paisa,
    /// The balance after this line.
    pub balance: Balance,
}

/// An account ledger (کھاتہ, PRD R13) without the items; plan 11 adds them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ledger {
    /// The balance before the first line (everything dated before `from`).
    pub opening: Balance,
    /// The lines with their running balance.
    pub lines: Vec<LedgerEntry>,
    /// The balance after the last line.
    pub closing: Balance,
}

/// The ledger of an account from `from` to `to` (`YYYY-MM-DD`, both included, either may be
/// open), for the whole account or one person. The running balance is kept as two checked sums,
/// so a change of side (جمع to بنام) needs no signed value.
///
/// # Errors
/// `ServiceError::Db`; `ServiceError::Data` for a `from` or `to` that is not a date.
pub fn ledger(
    ctx: &Context,
    account_id: i64,
    person: Option<&str>,
    from: Option<&str>,
    to: Option<&str>,
) -> Result<Ledger, ServiceError> {
    check_iso_date(from)?;
    check_iso_date(to)?;
    ctx.db.read(|conn| {
        let before = match from {
            Some(from) => {
                let date = BusinessDate::parse_iso(from)
                    .ok_or_else(|| ServiceError::Data(format!("date {from}")))?;
                match date.previous_day() {
                    Some(day) => repo::totals_of(conn, account_id, person, Some(&day.to_iso()))?,
                    None => repo::Totals {
                        dr_paisa: 0,
                        cr_paisa: 0,
                    },
                }
            }
            None => repo::Totals {
                dr_paisa: 0,
                cr_paisa: 0,
            },
        };
        let (mut dr_sum, mut cr_sum) = (paisa(before.dr_paisa)?, paisa(before.cr_paisa)?);
        let opening = Balance::from_totals(dr_sum, cr_sum).map_err(money_error)?;
        let mut lines = Vec::new();
        for row in repo::ledger_rows(conn, account_id, person, from, to)? {
            let (dr, cr) = (paisa(row.dr_paisa)?, paisa(row.cr_paisa)?);
            dr_sum = dr_sum.checked_add(dr).map_err(money_error)?;
            cr_sum = cr_sum.checked_add(cr).map_err(money_error)?;
            lines.push(LedgerEntry {
                entry_id: row.entry_id,
                entry_date: row.entry_date,
                line_no: row.line_no,
                posting_type: row.posting_type,
                narration: row.narration,
                person_key: row.person_key,
                dr,
                cr,
                balance: Balance::from_totals(dr_sum, cr_sum).map_err(money_error)?,
            });
        }
        let closing = Balance::from_totals(dr_sum, cr_sum).map_err(money_error)?;
        Ok(Ledger {
            opening,
            lines,
            closing,
        })
    })
}
