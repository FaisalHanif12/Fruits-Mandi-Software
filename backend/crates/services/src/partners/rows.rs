//! The hook that writes the partner share rows of a posting (PRD FR-6.5, FR-17.3, FR-17.4,
//! plan 11). It lives inside `ledger::post`, so every module (جنرل, خرچہ, بکری, own goods) gets
//! the split and no module writes share rows itself.
//!
//! - A new posting: the lines of the entry make an income base and an expense base
//!   (`domain::partners::share_bases`), which are split by the shares of the entry date
//!   (`domain::partners::split_posting`). Nothing is written if both bases are zero.
//! - A reversal: the rows of the original entry are copied with base and amount negated. It
//!   never splits again, so it cancels the original exactly even if the shares changed since,
//!   and it works when the shares today do not total 100%.
//! - `month_close` writes no rows (it only moves balances, PRD FR-17.6). An `opening` must not
//!   have a line on income or expense accounts (PRD FR-10.9 refuses those), so one that does is a
//!   bug and is refused loudly.
//!
//! The rows are planned BEFORE the entry is inserted ([`plan`]), so a refusal (for example
//! `ERR_SHARES_NOT_100`) leaves nothing behind, and written right after the entry is posted
//! ([`write`]), in the same transaction.

use std::collections::{BTreeMap, HashMap};

use mandi_db::partners as repo;
use mandi_db::rusqlite::Connection;
use mandi_domain::core::dates::BusinessDate;
use mandi_domain::core::error::{ErrorCode, ErrorValue};
use mandi_domain::core::ids::{AccountId, PartnerId};
use mandi_domain::core::money::{Bps, Paisa};
use mandi_domain::ledger::{BalancedEntry, PostingType};
use mandi_domain::partners::{
    Nature, PartnersError, ShareKind, ShareRowDraft, share_bases, split_posting,
};

use crate::error::{Rejection, ServiceError, money_error};
use crate::ledger::PostingEvent;

/// The rows of one posting, ready to write.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct SharePlan {
    rows: Vec<ShareRowDraft>,
}

/// Turns a refused split into the error the user sees.
fn split_error(error: PartnersError) -> ServiceError {
    match error {
        PartnersError::SharesNot100 { total_bps } => ServiceError::Rejected(vec![Rejection {
            code: ErrorCode::ErrSharesNot100,
            field: None,
            values: BTreeMap::from([(
                "total_bps".to_owned(),
                ErrorValue::Number(i64::from(total_bps)),
            )]),
        }]),
        PartnersError::Money(money) => money_error(money),
        PartnersError::Inconsistent => ServiceError::Data(
            "the partner share rows do not add up to the amount that was split".to_owned(),
        ),
    }
}

/// The nature of each account of the entry: one query for the heads of all its accounts.
fn natures_of(
    tx: &Connection,
    entry: &BalancedEntry,
) -> Result<HashMap<AccountId, Nature>, ServiceError> {
    let mut ids: Vec<i64> = entry.lines().iter().map(|l| l.account_id.get()).collect();
    ids.sort_unstable();
    ids.dedup();
    let heads = repo::head_codes_of(tx, &ids)?;
    if heads.len() != ids.len() {
        return Err(ServiceError::Data(
            "an entry has an account that does not exist".to_owned(),
        ));
    }
    Ok(heads
        .into_iter()
        .map(|(id, head)| (AccountId::new(id), Nature::from_head_code(&head)))
        .collect())
}

/// A stored row as a draft, to be mirrored.
fn draft_of(stored: &repo::StoredShareRow) -> Result<ShareRowDraft, ServiceError> {
    let bad = |what: &str| ServiceError::Data(format!("a stored share row has a wrong {what}"));
    Ok(ShareRowDraft {
        partner_id: PartnerId::new(stored.partner_id),
        kind: ShareKind::parse(&stored.kind).ok_or_else(|| bad("kind"))?,
        base: Paisa::try_from_paisa(stored.base_paisa).map_err(|_| bad("base"))?,
        share_bps: i32::try_from(stored.share_bps)
            .ok()
            .and_then(|bps| Bps::new(bps).ok())
            .ok_or_else(|| bad("share"))?,
        amount: Paisa::try_from_paisa(stored.amount_paisa).map_err(|_| bad("amount"))?,
    })
}

/// The shares of all partners on a day, for the split.
pub(crate) fn shares_on(
    tx: &Connection,
    date_iso: &str,
) -> Result<Vec<(PartnerId, Bps)>, ServiceError> {
    repo::shares_on(tx, date_iso)?
        .into_iter()
        .map(|(partner, bps)| {
            let bps = i32::try_from(bps)
                .ok()
                .and_then(|b| Bps::new(b).ok())
                .ok_or_else(|| {
                    ServiceError::Data("a stored share is not 0 to 10,000".to_owned())
                })?;
            Ok((PartnerId::new(partner), bps))
        })
        .collect()
}

/// Splits an entry from its natures: the one place that works out the bases and the shares of
/// the date.
fn split_with(
    tx: &Connection,
    date: BusinessDate,
    entry: &BalancedEntry,
    natures: &HashMap<AccountId, Nature>,
) -> Result<Vec<ShareRowDraft>, ServiceError> {
    let nature_of = |id: AccountId| natures.get(&id).copied().unwrap_or(Nature::Other);
    let bases = share_bases(entry, nature_of).map_err(money_error)?;
    if bases.is_zero() {
        return Ok(Vec::new());
    }
    let shares = shares_on(tx, &date.to_iso())?;
    split_posting(&shares, bases).map_err(split_error)
}

/// The share rows a NEW posting with these lines, dated `date`, would write: the heads of its
/// accounts, its income and expense bases, and the shares of that date. The save ([`plan`]) and
/// any preview of the split call this one function, so the two can never differ (Code Standards
/// B5). The date is a `BusinessDate`, never text, so a badly formed date cannot pick the wrong
/// shares. It reads only. The `بکری` preview of plan 13 passes the same `BalancedEntry` that the
/// save will post.
///
/// # Errors
/// `ERR_SHARES_NOT_100` (value `total_bps`) when a base is not zero and the shares of that date
/// do not total 100%; `ERR_OVERFLOW`; `ServiceError::Data` for an account that does not exist.
pub fn split_entry(
    conn: &Connection,
    date: BusinessDate,
    entry: &BalancedEntry,
) -> Result<Vec<ShareRowDraft>, ServiceError> {
    let natures = natures_of(conn, entry)?;
    split_with(conn, date, entry, &natures)
}

/// Works out the rows of a posting before it is written.
///
/// # Errors
/// `ERR_SHARES_NOT_100` (value `total_bps`) when a base is not zero and the shares on the entry
/// date do not total 100%; `ERR_OVERFLOW`; `ServiceError::Data` for an opening with a line on an
/// income or expense account, an event whose `reverses` and posting type disagree, or a stored
/// row or split that is not as written.
pub(crate) fn plan(tx: &Connection, event: &PostingEvent) -> Result<SharePlan, ServiceError> {
    // A reversal has `reverses` and the posting type `reversal`, or neither (the database
    // checks the same). Anything else is a bug in the caller.
    if event.reverses.is_some() != (event.posting_type == PostingType::Reversal) {
        return Err(ServiceError::Data(
            "an entry is a reversal or it is not: reverses and the posting type disagree"
                .to_owned(),
        ));
    }
    // PRD FR-17.6: month close only moves amounts between accounts.
    if event.posting_type == PostingType::MonthClose {
        return Ok(SharePlan::default());
    }
    // A reversal copies; it never splits again (PRD FR-17.4).
    if let Some(original) = event.reverses {
        let stored = repo::share_rows_of_entry(tx, original.get())?;
        // A reversal has the date of the entry it reverses (PRD FR-17.4, Architecture 12). The
        // copied rows carry that date, and `db::partners::taken_between` and the monthly sums
        // rely on it, so it is checked for EVERY reversal, also one of an entry with no rows.
        // The one exception is the reversal of a month close (PRD FR-17.6, FR-21.2), which may
        // have another date; it copies no rows, because a month close writes none.
        let (original_date, original_type) = repo::entry_date_and_type(tx, original.get())?
            .ok_or_else(|| ServiceError::Data("the entry to reverse does not exist".to_owned()))?;
        if original_type != PostingType::MonthClose.as_str() && original_date != event.date.to_iso()
        {
            return Err(ServiceError::Data(
                "a reversal must have the date of the entry it reverses".to_owned(),
            ));
        }
        let rows = stored
            .iter()
            .map(|row| draft_of(row)?.mirrored().map_err(money_error))
            .collect::<Result<Vec<_>, _>>()?;
        return Ok(SharePlan { rows });
    }
    let natures = natures_of(tx, &event.entry)?;
    if event.posting_type == PostingType::Opening
        && event.entry.lines().iter().any(|l| {
            natures
                .get(&l.account_id)
                .is_some_and(|nature| *nature != Nature::Other)
        })
    {
        // PRD FR-10.9 refuses an opening on these accounts; reaching here is a bug.
        return Err(ServiceError::Data(
            "an opening entry has a line on an income or shop expense account".to_owned(),
        ));
    }
    let rows = split_with(tx, event.date, &event.entry, &natures)?;
    Ok(SharePlan { rows })
}

/// Writes the planned rows for the entry that was just posted.
///
/// # Errors
/// `ServiceError::Db` (the CHECKs of `partner_share_rows`).
pub(crate) fn write(
    tx: &Connection,
    user_id: i64,
    now: &str,
    entry_id: i64,
    entry_date_iso: &str,
    plan: &SharePlan,
) -> Result<(), ServiceError> {
    for row in &plan.rows {
        repo::insert_share_row(
            tx,
            &repo::NewShareRow {
                journal_entry_id: entry_id,
                entry_date: entry_date_iso,
                partner_id: row.partner_id.get(),
                kind: row.kind.as_str(),
                base_paisa: row.base.as_paisa(),
                share_bps: i64::from(row.share_bps.get()),
                amount_paisa: row.amount.as_paisa(),
            },
            user_id,
            now,
        )?;
    }
    Ok(())
}
