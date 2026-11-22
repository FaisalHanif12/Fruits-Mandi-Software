//! The پیشگی the sellers take with their lots (PRD FR-12.52 v1.57, FR-15.8, FR-6.7, plan 95 item 11).
//!
//! A lot may carry an advance: money its seller takes when he brings the goods. It is a real advance,
//! so it is its own journal entry of the posting type `advance` with the بکری as its source: a seller
//! debit flagged `is_advance` for each lot that has one, and one روکڑ credit. It is written in the SAME
//! transaction as the بکری and just BEFORE the بکری's first entry. That order puts the advance ahead
//! of the seller's credit in his canonical order (date, order key, id, line), so a lot whose tick
//! "پیشگی سے کاٹیں" is on is adjusted by its own credit, and the fast and the canonical matching agree
//! (PRD FR-15.8). The amount is also kept on the lot (`bikri_lots.advance_paisa`) so an edit opens the
//! lot with it. It is NOT part of the lot's deductions, net or seller credit (C6 to C8).
//!
//! * A new بکری: the entry is posted before the first entry ([`post_new`]).
//! * An edit of the first posting reverses the old entry (if any) with the old first entry, then
//!   posts the new one before the new first entry, with the key of the old advance entry or, when
//!   there was none, of the old first entry, so the new entries keep their place in the seller's
//!   order ([`reverse_live`], [`post_edited`]).
//! * A cancel and an undo reverse it too ([`reverse_live`]).
//!
//! The entry has no income or expense line, so it writes no partner share rows (PRD FR-6.5). The
//! partner totals follow through the advance working (PRD FR-17.13), which reads every live
//! `is_advance` line.
//!
//! Worked numbers (plan 95): عدنان, lot credit 52,900, نقد 30,000, advance 20,000: entry A is
//! عدنان بنام 20,000 (advance), روکڑ جمع 20,000; the بکری entry is 94,400 = 94,400 as without it.

use mandi_db::bikri::edit as edit_repo;
use mandi_db::ledger as ledger_repo;
use mandi_db::rusqlite::Connection;
use mandi_domain::core::dates::BusinessDate;
use mandi_domain::ledger::{BalancedEntry, PostingType, SourceType};

use crate::context::Context;
use crate::error::ServiceError;
use crate::ledger::{self, PostingEvent};

/// The live پیشگی entry of a بکری, if any: posted, not reversed, of the posting type `advance`.
///
/// # Errors
/// `ServiceError::Db`.
pub(super) fn live_entry(
    conn: &Connection,
    bikri_id: i64,
) -> Result<Option<edit_repo::PostingRef>, ServiceError> {
    Ok(edit_repo::advance_entry_in_force(conn, bikri_id)?)
}

/// Posts the پیشگی entry of a new بکری, just before its first entry. Returns the entry id.
///
/// # Errors
/// The errors of `ledger::post` (an inactive or dead seller, a closed day, ...).
pub(super) fn post_new(
    ctx: &Context,
    tx: &Connection,
    user_id: i64,
    bikri_id: i64,
    date: BusinessDate,
    entry: &BalancedEntry,
) -> Result<i64, ServiceError> {
    post(ctx, tx, user_id, bikri_id, date, entry, None)
}

/// Posts the پیشگی entry of an edited first posting, before its new first entry. `key` is the
/// order key it takes (see the module text).
///
/// # Errors
/// As [`post_new`].
pub(super) fn post_edited(
    ctx: &Context,
    tx: &Connection,
    user_id: i64,
    bikri_id: i64,
    date: BusinessDate,
    entry: &BalancedEntry,
    key: i64,
) -> Result<i64, ServiceError> {
    post(ctx, tx, user_id, bikri_id, date, entry, Some(key))
}

fn post(
    ctx: &Context,
    tx: &Connection,
    user_id: i64,
    bikri_id: i64,
    date: BusinessDate,
    entry: &BalancedEntry,
    key: Option<i64>,
) -> Result<i64, ServiceError> {
    let posted = ledger::post_ordered(
        ctx,
        tx,
        user_id,
        &PostingEvent {
            date,
            posting_type: PostingType::Advance,
            source_type: SourceType::Bikri,
            source_id: bikri_id,
            narration: None,
            reverses: None,
            entry: entry.clone(),
        },
        key,
    )?;
    Ok(posted.entry_id)
}

/// What the live پیشگی entry of a بکری did to cash in hand (debit on روکڑ is plus), 0 when there
/// is none (PRD FR-6.7).
///
/// # Errors
/// `ServiceError::Db`, `ERR_OVERFLOW`.
pub(super) fn cash_of_live(
    conn: &Connection,
    bikri_id: i64,
) -> Result<mandi_domain::core::money::Paisa, ServiceError> {
    let Some(live) = live_entry(conn, bikri_id)? else {
        return Ok(mandi_domain::core::money::Paisa::ZERO);
    };
    let entry = ledger_repo::get_entry(conn, live.entry_id)?
        .ok_or_else(|| super::shared::data("the پیشگی entry of a بکری cannot be read"))?;
    crate::vouchers::stored_cash_change(conn, &entry)
}
