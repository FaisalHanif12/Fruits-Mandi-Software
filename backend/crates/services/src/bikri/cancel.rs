//! Cancel and undo of a بکری (PRD FR-12.11, FR-12.22, FR-7.47, FR-6.3, FR-18.7, plan 16).
//!
//! A cancel reverses EVERY posting of the بکری, in one transaction. Every posting day must be open
//! (the Owner reopens a closed day first, PRD FR-21.2); the reason is required. For each posting:
//! the journal is reversed (and through the hook the partner share rows), its stock moves and
//! analytics facts get opposite rows (so a cancelled بکری adds nothing, PRD FR-18.7), and its sale
//! lines end as `cancelled`. The write-offs of its lots are reversed too, so the stock of a
//! cancelled lot nets to zero. The بکری is marked منسوخ; its number is kept and never used again
//! (PRD FR-12.2), and its lot columns stay as the record of what was cancelled.
//!
//! A receipt that was matched to a bill of the بکری becomes unmatched credit on the buyer and is
//! matched again to his next oldest bills (PRD FR-12.22, E15): every buyer key and seller the
//! postings touch is matched again from the start, never by the fast paths.
//!
//! The cash question (PRD FR-6.7) is asked for the whole action: the reversals give back what the
//! postings took from cash and take back what they put in.
//!
//! An undo (PRD FR-7.47) is a cancel with a fixed reason, straight after a save, and gives back a
//! draft with the same values.
//!
//! Example (PRD E2E-15): cancelling بکری 2 reverses its entry of 25,600, its split of 616 / 462 /
//! 462 and its 12,800 of cash, and the سیب facts net to zero.

use std::collections::BTreeSet;

use mandi_db::bikri::edit::{self as edit_repo, Ended};
use mandi_db::ledger as ledger_repo;
use mandi_db::rusqlite::Connection;
use mandi_domain::bikri::PostingKind;
use mandi_domain::core::bikri::{
    BikriCancelled, BikriDetail, BikriStatus, CancelInput, CancelPreview, CancelPreviewInput,
    SavedPosting, UNDO_REASON_ID, UndoInput, UndoResult,
};
use mandi_domain::core::dates::BusinessDate;
use mandi_domain::core::error::{ErrorCode, ErrorValue};
use mandi_domain::core::money::Paisa;
use mandi_domain::core::validate;
use serde_json::json;

use super::load::{detail_of, draft_of};
use super::read::mark_editable;
use super::shared::data;
use crate::audit::Event;
use crate::auth::Actor;
use crate::cash;
use crate::context::Context;
use crate::day;
use crate::error::{Rejection, ServiceError, money_error};
use crate::ledger;
use crate::vouchers::{rematch_party, role_of_account, stored_cash_change};

/// The postings of a بکری that are in force, with their days.
struct Live {
    entry_id: i64,
    date: BusinessDate,
}

fn live_postings(conn: &Connection, bikri_id: i64) -> Result<Vec<Live>, ServiceError> {
    edit_repo::postings_in_force(conn, bikri_id)?
        .into_iter()
        .map(|p| {
            Ok(Live {
                entry_id: p.entry_id,
                date: BusinessDate::parse_iso(&p.entry_date)
                    .ok_or_else(|| data("the date of a posting is not a date"))?,
            })
        })
        .collect()
}

/// The days of these postings that are on or before the "closed up to" day, oldest first.
fn closed_days(live: &[Live], closed_up_to: Option<BusinessDate>) -> Vec<BusinessDate> {
    let mut days: Vec<BusinessDate> = live
        .iter()
        .map(|p| p.date)
        .filter(|date| closed_up_to.is_some_and(|closed| *date <= closed))
        .collect();
    days.sort();
    days.dedup();
    days
}

/// What the postings of a بکری did to cash in hand, added up (debit on روکڑ is plus).
fn cash_of(conn: &Connection, live: &[Live]) -> Result<Paisa, ServiceError> {
    let mut total = Paisa::ZERO;
    for posting in live {
        let entry = ledger_repo::get_entry(conn, posting.entry_id)?
            .ok_or_else(|| data("a posting of a بکری cannot be read"))?;
        total = total
            .checked_add(stored_cash_change(conn, &entry)?)
            .map_err(money_error)?;
    }
    Ok(total)
}

fn closed_error(day: BusinessDate, field: &str) -> ServiceError {
    ServiceError::Rejected(vec![Rejection {
        code: ErrorCode::ErrDateClosed,
        field: Some(field.to_owned()),
        values: std::collections::BTreeMap::from([(
            "closed_up_to".to_owned(),
            ErrorValue::Text(day.to_iso()),
        )]),
    }])
}

/// What a cancel would do, for the confirm box (PRD FR-7.8). Read only.
///
/// # Errors
/// `ERR_NOT_FOUND` on `id`.
pub fn preview(ctx: &Context, input: &CancelPreviewInput) -> Result<CancelPreview, ServiceError> {
    let (closed_up_to, _) = day::window(ctx)?;
    ctx.db.read(|conn| {
        let mut detail = detail_of(conn, input.id, "id")?;
        mark_editable(&mut detail, closed_up_to);
        let live = live_postings(conn, detail.id)?;
        let closed = closed_days(&live, closed_up_to);
        let postings: Vec<SavedPosting> = detail
            .postings
            .iter()
            .filter(|p| live.iter().any(|l| l.entry_id == p.entry_id))
            .cloned()
            .collect();
        // A cancel gives back what the postings took from cash.
        let change = cash_of(conn, &live)?.checked_neg().map_err(money_error)?;
        Ok(CancelPreview {
            bikri_id: detail.id,
            number: detail.number,
            version: detail.version,
            status: detail.status,
            postings,
            cash: cash::cash_after_entry(conn, change)?,
            closed_dates: closed.iter().map(|d| d.to_iso()).collect(),
            can_cancel: detail.status == BikriStatus::Saved && closed.is_empty(),
        })
    })
}

/// Cancels a بکری inside the caller's transaction and returns the reversal entries.
fn cancel_in(
    ctx: &Context,
    tx: &Connection,
    actor: &Actor,
    detail: &BikriDetail,
    reason: &str,
    confirmed: bool,
) -> Result<Vec<i64>, ServiceError> {
    let (closed_up_to, _) = day::window(ctx)?;
    if detail.status == BikriStatus::Cancelled {
        return Err(ServiceError::reject_field(
            ErrorCode::ErrBikriCancelled,
            "id",
        ));
    }
    let user = actor.user_id;
    let now = ctx.now_text();
    let live = live_postings(tx, detail.id)?;
    if let Some(day) = closed_days(&live, closed_up_to).first() {
        return Err(closed_error(*day, "id"));
    }
    // The cash question of the whole action (PRD FR-6.7).
    let change = cash_of(tx, &live)?.checked_neg().map_err(money_error)?;
    cash::require_confirmed(&cash::cash_after_entry(tx, change)?, confirmed)?;

    let mut touched: BTreeSet<(i64, Option<String>)> = BTreeSet::new();
    let mut reversals = Vec::with_capacity(live.len());
    for posting in &live {
        let entry = ledger_repo::get_entry(tx, posting.entry_id)?
            .ok_or_else(|| data("a posting of a بکری cannot be read"))?;
        touched.extend(
            entry
                .lines
                .iter()
                .map(|l| (l.account_id, l.person_key.clone())),
        );
        let reversal = ledger::reverse(ctx, tx, user, posting.entry_id, reason)?;
        edit_repo::reverse_moves(tx, posting.entry_id, reversal.entry_id, user, &now)?;
        edit_repo::reverse_facts(tx, posting.entry_id, reversal.entry_id, user, &now)?;
        for sale in edit_repo::active_sales_of_entry(tx, posting.entry_id)? {
            edit_repo::end_sale(tx, sale.id, Ended::Cancelled, None, user, &now)?;
        }
        reversals.push(reversal.entry_id);
    }
    // The write-offs of its lots are taken back too, so the stock of a cancelled lot nets to zero.
    let lot_ids: Vec<i64> = detail.lots.iter().map(|l| l.id).collect();
    for write_off in edit_repo::write_offs_of_lots(tx, &lot_ids)? {
        edit_repo::insert_move(
            tx,
            &edit_repo::NewMove {
                lot_id: write_off.lot_id,
                entry_id: None,
                move_date: &write_off.move_date,
                kind: "reversal",
                qty: write_off.qty,
                reason: Some(reason),
                reverses_move_id: Some(write_off.move_id),
            },
            user,
            &now,
        )?;
    }
    edit_repo::mark_cancelled(tx, detail.id, reason, user, &now)?;

    // Receipts matched to its bills are credit again, matched to the next oldest bills; advances
    // it adjusted are open again (PRD FR-12.22).
    for (account_id, person_key) in touched {
        if let Some((role, is_pukhta)) = role_of_account(tx, account_id)? {
            rematch_party(
                tx,
                user,
                &now,
                account_id,
                person_key.as_deref(),
                role,
                is_pukhta,
            )?;
        }
    }
    ctx.db.auditor().write(
        tx,
        &Event::new(Some(user), "cancel", "bikri")
            .entity_id(detail.id)
            .old(json!({
                "number": detail.number,
                "status": "saved",
                "postings": live.iter().map(|p| p.entry_id).collect::<Vec<_>>(),
                "gross_paisa": detail.totals.gross.as_paisa(),
                "buyer_total_paisa": detail.totals.buyer_total.as_paisa(),
            }))
            .new_value(json!({
                "status": "cancelled",
                "reversal_entries": reversals,
                "cash_change_paisa": change.as_paisa(),
            }))
            .reason(reason)
            .into_entry(),
    )?;
    Ok(reversals)
}

/// Cancels a بکری (PRD FR-12.22): see the module text.
///
/// # Errors
/// `ERR_NOT_FOUND` and `ERR_BIKRI_CANCELLED` on `id`; `ERR_VERSION_CHANGED` on `version`;
/// `ERR_REASON_REQUIRED` on `reason`; `ERR_DATE_CLOSED` on `id` (value `closed_up_to`);
/// `ERR_CASH_BELOW_ZERO_NOT_CONFIRMED` (value `cash_after_paisa`). Nothing is written when any of
/// them is returned.
pub fn cancel(
    ctx: &Context,
    actor: &Actor,
    input: &CancelInput,
) -> Result<BikriCancelled, ServiceError> {
    let reason = validate::free_text("reason", &input.reason)?;
    if reason.is_empty() {
        return Err(ServiceError::reject_field(
            ErrorCode::ErrReasonRequired,
            "reason",
        ));
    }
    ctx.db.write(|tx| {
        let detail = detail_of(tx, input.id, "id")?;
        if detail.status == BikriStatus::Cancelled {
            return Err(ServiceError::reject_field(
                ErrorCode::ErrBikriCancelled,
                "id",
            ));
        }
        if detail.version != input.version {
            return Err(ServiceError::reject_field(
                ErrorCode::ErrVersionChanged,
                "version",
            ));
        }
        let reversals = cancel_in(
            ctx,
            tx,
            actor,
            &detail,
            &reason,
            input.confirm_cash_below_zero,
        )?;
        Ok(BikriCancelled {
            bikri_id: detail.id,
            number: detail.number,
            reversal_entry_ids: reversals,
        })
    })
}

/// Undoes a بکری straight after its save (PRD FR-7.47): a cancel with the fixed reason
/// `UNDO_REASON_ID`, giving back a draft with the same values, so the screen opens a new unsaved
/// entry to correct. The cancelled number is not used again. Works only while the day is open.
///
/// # Errors
/// As [`cancel`], without the reason and the version.
pub fn undo(ctx: &Context, actor: &Actor, input: &UndoInput) -> Result<UndoResult, ServiceError> {
    ctx.db.write(|tx| {
        let detail = detail_of(tx, input.id, "id")?;
        // The values to give back are those of the first posting.
        let first = edit_repo::postings_in_force(tx, detail.id)?
            .into_iter()
            .find(|p| p.posting_type == "bikri_first")
            .ok_or_else(|| ServiceError::reject_field(ErrorCode::ErrBikriCancelled, "id"))?;
        let date = BusinessDate::parse_iso(&first.entry_date)
            .ok_or_else(|| data("the date of a posting is not a date"))?;
        let draft = draft_of(&detail, first.entry_id, date, PostingKind::First, false)?;
        let reversals = cancel_in(
            ctx,
            tx,
            actor,
            &detail,
            UNDO_REASON_ID,
            input.confirm_cash_below_zero,
        )?;
        Ok(UndoResult {
            bikri_id: detail.id,
            number: detail.number,
            reversal_entry_ids: reversals,
            draft,
        })
    })
}
