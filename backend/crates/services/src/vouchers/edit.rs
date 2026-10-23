//! Edit and cancel of a voucher by reversal (PRD FR-15.3, FR-6.3, Architecture 12, plan 15). All
//! three types go through here.
//!
//! - **Edit**: while the posting's day is open, the posting is taken back by a reversal on the
//!   same day, the matching of the party is worked out again for the books without the voucher,
//!   the new version is checked and posted, and the voucher row is updated, all in one
//!   transaction. The voucher keeps its number and its type. If anything is refused the whole
//!   edit is rolled back.
//! - **Cancel**: the posting is taken back, the party is matched again (a buyer's receipt that
//!   settled bills is credit again, and is matched to his oldest bills, PRD FR-12.22), and the
//!   voucher is marked منسوخ with the reason. It keeps its number (Code Standards D3).
//!
//! After a reversal the matching rows are no longer canonical, so only the full workings
//! (`matching::rematch`, `matching::rematch_advances`) are used here, never the fast paths
//! (money agent review of plan 15, problem 3).
//!
//! The cash question of PRD FR-6.7 uses the net change of the whole action: the reversal gives
//! the old cash back and the new entry takes its own, so "edit a payment 3,000 to 2,000" is
//! +1,000 and never asks, and "cancel a receipt of 10,000" is −10,000 and asks if cash would
//! end below 0.

use mandi_db::ledger as ledger_repo;
use mandi_db::rusqlite::Connection;
use mandi_db::vouchers::{self as repo, EditedVoucher, VoucherExtra};
use mandi_domain::cash;
use mandi_domain::core::accounts::Balance;
use mandi_domain::core::error::ErrorCode;
use mandi_domain::core::money::Paisa;
use mandi_domain::core::validate::{self, FieldErrors};
use mandi_domain::core::vouchers::{
    VoucherCancelInput, VoucherCancelled, VoucherEditInput, VoucherEntry, VoucherRow, VoucherSaved,
    VoucherType,
};
use serde_json::{Value, json};

use super::common::{
    PostingSpec, Role, post_entry, rematch_party, role_of_account, stored_cash_change,
};
use super::general_kind::rematch_buyers;
use super::kinds::Planned;
use super::receipt::check_seller_limit;
use crate::audit::Event;
use crate::auth::Actor;
use crate::cash::{cash_in_hand, require_confirmed};
use crate::context::Context;
use crate::day;
use crate::error::{ServiceError, money_error};
use crate::floor::{AMOUNT_FIELD, check_after, floor_before};
use crate::ledger;

/// A voucher read for an edit or a cancel, with what its old posting touched.
struct Old {
    row: VoucherRow,
    entry: ledger_repo::StoredEntry,
    /// The account the voucher is on. None for a voucher with no single account (جنرل).
    account_id: Option<i64>,
    person_key: Option<String>,
    /// Its matching role, for a buyer or a seller. None for every other account: the خرچہ,
    /// ادھار رقم, سرمایہ and الگ کھاتہ kinds have no matching.
    role: Option<(Role, bool)>,
    /// The columns only the vouchers of plan 17 use.
    extra: VoucherExtra,
}

/// Reads the voucher and checks it can be changed: it exists, it is not cancelled, and the screen
/// has its latest version (PRD FR-15.3).
fn load(tx: &Connection, id: i64, version: i64) -> Result<Old, ServiceError> {
    let row = repo::get(tx, id)?
        .ok_or_else(|| ServiceError::reject_field(ErrorCode::ErrNotFound, "id"))?;
    if row.status != "saved" {
        return Err(ServiceError::reject_field(
            ErrorCode::ErrVoucherCancelled,
            "id",
        ));
    }
    if row.version != version {
        return Err(ServiceError::reject_field(
            ErrorCode::ErrVersionChanged,
            "version",
        ));
    }
    let entry_id = row
        .entry_id
        .ok_or_else(|| ServiceError::Data(format!("voucher {id} has no posting")))?;
    let entry = ledger_repo::get_entry(tx, entry_id)?
        .ok_or_else(|| ServiceError::Data(format!("the posting of voucher {id} is missing")))?;
    let account_id = row.account.as_ref().map(|a| a.id);
    let role = match account_id {
        Some(account_id) => role_of_account(tx, account_id)?,
        None => None,
    };
    // A receipt, a payment and a پیشگی are on a buyer, a seller or a payable account.
    if matches!(
        row.voucher_type,
        VoucherType::Receipt | VoucherType::Payment | VoucherType::Advance
    ) && role.is_none()
    {
        return Err(ServiceError::Data(format!(
            "voucher {id}: account {account_id:?}"
        )));
    }
    Ok(Old {
        person_key: row.person.as_ref().map(|p| p.person_key.clone()),
        extra: repo::extras(tx, id)?,
        row,
        entry,
        account_id,
        role,
    })
}

/// The accounts and people the lines of the old posting touched, for a جنرل. The other kinds have
/// one account, which `rematch_party` handles.
fn old_general_lines(old: &Old) -> Vec<(i64, Option<String>)> {
    if old.row.voucher_type != VoucherType::General {
        return Vec::new();
    }
    old.entry
        .lines
        .iter()
        .map(|l| (l.account_id, l.person_key.clone()))
        .collect()
}

/// A voucher as the audit log keeps it, from its stored row.
fn row_json(row: &VoucherRow, extra: &VoucherExtra) -> Value {
    let mut value = json!({
        "type": row.voucher_type.as_str(),
        "number": row.number,
        "entry_date": row.entry_date,
        "account_id": row.account.as_ref().map(|a| a.id),
        "person_key": row.person.as_ref().map(|p| p.person_key.as_str()),
        "amount_paisa": row.amount_paisa.as_paisa(),
        "mode": row.mode.as_str(),
        "bank_account_id": row.bank_account_id,
        "cheque_no": row.cheque_no,
        "cheque_date": row.cheque_date,
        "collector_staff_id": row.collector_staff_id,
        "reason": row.reason,
        "expected_goods": row.expected_goods,
        "narration": row.narration,
        "status": row.status,
    });
    super::common::add_extra_json(&mut value, extra);
    value
}

/// Asks the cash question for the whole action. `old_cash` is what the old posting did to cash;
/// `new_cash` is what the new entry does (0 for a cancel).
fn confirm_cash(
    tx: &Connection,
    old_cash: Paisa,
    new_cash: Paisa,
    confirmed: bool,
) -> Result<(), ServiceError> {
    // Cash now already has the reversal in it, so cash before the action is cash now plus what
    // the old posting took out (or less what it put in).
    let now = cash_in_hand(tx)?;
    let before = now.checked_add(old_cash).map_err(money_error)?;
    let net = new_cash.checked_sub(old_cash).map_err(money_error)?;
    let check = cash::check(before, net).map_err(money_error)?;
    require_confirmed(&check, confirmed)
}

/// Edits a voucher (PRD FR-15.3): see the module text.
///
/// # Errors
/// `ERR_NOT_FOUND` (`id`), `ERR_VOUCHER_CANCELLED` (`id`), `ERR_VERSION_CHANGED` (`version`),
/// `ERR_FORMAT_INVALID` (`entry.type`, a voucher cannot change its type), `ERR_DATE_CLOSED` (the
/// old day or the new one), every error of the type's own save, `ERR_RETURN_EXCEEDS_OPEN` and
/// `ERR_CASH_BELOW_ZERO_NOT_CONFIRMED`. Nothing is written when any of them is returned.
pub fn edit(
    ctx: &Context,
    actor: &Actor,
    input: &VoucherEditInput,
) -> Result<VoucherSaved, ServiceError> {
    let window = day::window(ctx)?;
    let mut errors = FieldErrors::default();
    let note = errors
        .take(validate::free_text("reason", &input.reason))
        .filter(|t| !t.is_empty());
    errors.finish()?;
    ctx.db.write(|tx| {
        let old = load(tx, input.id, input.version)?;
        if input.entry.voucher_type() != old.row.voucher_type {
            return Err(ServiceError::reject_field(
                ErrorCode::ErrFormatInvalid,
                "entry.type",
            ));
        }
        let now = ctx.now_text();
        let old_cash = stored_cash_change(tx, &old.entry)?;
        let old_key = ledger::order_key_of(tx, old.entry.id)?;
        // A take or a loan: the lowest day-end balance of its account before the edit, so the
        // edit is refused only if it makes returns stand above what was taken or lent (PRD
        // FR-16.2, FR-16.5). See `crate::floor`.
        let old_floor = floor_before(tx, old.row.voucher_type, old.account_id)?;
        // A seller's return is judged against the books with the old version still in force, at
        // the place the new one keeps (migration 0005, plan 16): the same return, or a smaller
        // one, stays allowed although a later entry of its day used the advance. It is worked out
        // here, with the same function as the preview (Code Standards B5), and refused below after
        // the date and the field checks, so those errors keep coming first.
        let seller_check = match &input.entry {
            VoucherEntry::Receipt(_) => {
                let first = Planned::plan(tx, window, &input.entry, None)?;
                match &first {
                    Planned::Receipt(receipt) if first.errors().is_empty() => {
                        check_seller_limit(tx, receipt, Some(old.entry.id), Some(old.entry.id))
                    }
                    _ => Ok(()),
                }
            }
            _ => Ok(()),
        };
        // 1. Take the old posting back, on its own day (ERR_DATE_CLOSED if that day is closed).
        ledger::reverse(
            ctx,
            tx,
            actor.user_id,
            old.entry.id,
            note.as_deref().unwrap_or(""),
        )?;
        if let (Some(account_id), Some((role, is_pukhta))) = (old.account_id, old.role) {
            rematch_party(
                tx,
                actor.user_id,
                &now,
                account_id,
                old.person_key.as_deref(),
                role,
                is_pukhta,
            )?;
        }
        rematch_buyers(tx, actor.user_id, &now, &old_general_lines(&old))?;
        // 2. Check the new version against the books without the old one.
        let planned = Planned::plan(tx, window, &input.entry, None)?;
        if !planned.errors().is_empty() {
            return Err(planned.errors().to_vec().into());
        }
        seller_check?;
        let entry = planned.entry()?;
        let new_cash = super::common::cash_change(tx, &entry)?;
        confirm_cash(tx, old_cash, new_cash, input.confirm_cash_below_zero)?;
        // An edit that moves a take or a loan to another partner or person must check that
        // account too; the reversal does not touch it, so it is read now.
        let new_account = planned.account().map(|a| a.id);
        let new_floor = if new_account == old.account_id {
            None
        } else {
            floor_before(tx, old.row.voucher_type, new_account)?
        };
        // 3. Post it and update the voucher, which keeps its number.
        let row = planned.row(old.row.number)?;
        let date = planned
            .date()
            .ok_or_else(|| ServiceError::Data("an edited voucher has no date".to_owned()))?;
        let posted = post_entry(
            ctx,
            tx,
            actor.user_id,
            old.row.id,
            PostingSpec {
                posting_type: planned.posting_type(),
                date,
                narration: row.narration.clone(),
            },
            entry,
            // The edited voucher keeps its place among the entries of its day (migration 0005).
            Some(old_key),
        )?;
        repo::update_for_edit(
            tx,
            old.row.id,
            &EditedVoucher {
                entry_date: row.entry_date.clone(),
                account_id: row.account_id,
                person_key: row.person_key.clone(),
                amount_paisa: row.amount_paisa,
                mode: row.mode,
                bank_account_id: row.bank_account_id,
                cheque_no: row.cheque_no.clone(),
                cheque_date: row.cheque_date.clone(),
                collector_staff_id: row.collector_staff_id,
                reason: row.reason.clone(),
                expected_goods: row.expected_goods.clone(),
                narration: row.narration.clone(),
                extra: row.extra.clone(),
                current_entry_id: posted.entry_id,
            },
            actor.user_id,
            &now,
        )?;
        // The edited voucher is not the newest line of its day: it kept its place (migration 0005),
        // so the fast paths of a new posting do not hold. The full working decides, as for a
        // cancel (plan 16: a same-day seller return edited after a later same-day ticked بکری
        // stays allowed, and an edited پیشگی keeps its adjustment).
        if let (Some(account), Some((role, is_pukhta))) = (planned.account(), planned.role()) {
            rematch_party(
                tx,
                actor.user_id,
                &now,
                account.id,
                planned.person_key(),
                role,
                is_pukhta,
            )?;
        }
        rematch_buyers(tx, actor.user_id, &now, &planned.general_lines())?;
        // A take or a loan that was returned in part cannot be made smaller than its returns,
        // moved to a later day than its returns, or moved to another partner or person while
        // returns stand on it: the OLD account is checked too (money agent, plan 17: الف took
        // 2,000 and returned 2,000; the take edited to ب would leave الف's drawings 2,000 on the
        // credit side).
        check_after(tx, old_floor, AMOUNT_FIELD)?;
        check_after(tx, new_floor, AMOUNT_FIELD)?;
        let mut new_value =
            super::common::voucher_json(old.row.number, old.row.voucher_type.as_str(), &row);
        if let (Some(lines), Some(map)) = (planned.general_lines_json(), new_value.as_object_mut())
        {
            map.insert("lines".to_owned(), lines);
        }
        let mut event = Event::new(Some(actor.user_id), "edit", "voucher")
            .entity_id(old.row.id)
            .old(row_json(&old.row, &old.extra))
            .new_value(new_value);
        if let Some(note) = &note {
            event = event.reason(note);
        }
        ctx.db.auditor().write(tx, &event.into_entry())?;
        let new_balance = match planned.account() {
            Some(account) => ledger::balance_in(tx, account.id, planned.person_key(), None)?,
            None => Balance::ZERO,
        };
        Ok(VoucherSaved {
            voucher_id: old.row.id,
            voucher_type: old.row.voucher_type,
            number: old.row.number,
            entry_id: posted.entry_id,
            new_balance,
        })
    })
}

/// Cancels a voucher (PRD FR-15.3): see the module text.
///
/// # Errors
/// `ERR_NOT_FOUND` (`id`), `ERR_VOUCHER_CANCELLED` (`id`), `ERR_VERSION_CHANGED` (`version`),
/// `ERR_REASON_REQUIRED` (`reason`), `ERR_DATE_CLOSED`, `ERR_CASH_BELOW_ZERO_NOT_CONFIRMED`.
/// Nothing is written when any of them is returned.
pub fn cancel(
    ctx: &Context,
    actor: &Actor,
    input: &VoucherCancelInput,
) -> Result<VoucherCancelled, ServiceError> {
    let mut errors = FieldErrors::default();
    let reason = errors
        .take(validate::free_text("reason", &input.reason))
        .filter(|t| !t.is_empty());
    if reason.is_none() && errors.is_empty() {
        return Err(ServiceError::reject_field(
            ErrorCode::ErrReasonRequired,
            "reason",
        ));
    }
    errors.finish()?;
    let reason = reason.unwrap_or_default();
    ctx.db.write(|tx| {
        let old = load(tx, input.id, input.version)?;
        let now = ctx.now_text();
        let old_cash = stored_cash_change(tx, &old.entry)?;
        let old_floor = floor_before(tx, old.row.voucher_type, old.account_id)?;
        let reversal = ledger::reverse(ctx, tx, actor.user_id, old.entry.id, &reason)?;
        if let (Some(account_id), Some((role, is_pukhta))) = (old.account_id, old.role) {
            rematch_party(
                tx,
                actor.user_id,
                &now,
                account_id,
                old.person_key.as_deref(),
                role,
                is_pukhta,
            )?;
        }
        rematch_buyers(tx, actor.user_id, &now, &old_general_lines(&old))?;
        // A take or a loan that was returned in part cannot be cancelled: its returns would be
        // more than what is open.
        check_after(tx, old_floor, AMOUNT_FIELD)?;
        confirm_cash(tx, old_cash, Paisa::ZERO, input.confirm_cash_below_zero)?;
        repo::mark_cancelled(tx, old.row.id, &reason, actor.user_id, &now)?;
        ctx.db.auditor().write(
            tx,
            &Event::new(Some(actor.user_id), "cancel", "voucher")
                .entity_id(old.row.id)
                .old(row_json(&old.row, &old.extra))
                .reason(&reason)
                .into_entry(),
        )?;
        let balance = match old.account_id {
            Some(account_id) => {
                ledger::balance_in(tx, account_id, old.person_key.as_deref(), None)?
            }
            None => Balance::ZERO,
        };
        Ok(VoucherCancelled {
            voucher_id: old.row.id,
            voucher_type: old.row.voucher_type,
            number: old.row.number,
            reversal_entry_id: reversal.entry_id,
            new_balance: balance,
        })
    })
}

/// One voucher by id, with whether it can still be edited (PRD FR-15.3, FR-6.6).
///
/// # Errors
/// `ERR_NOT_FOUND` on `id`.
pub fn get(ctx: &Context, id: i64) -> Result<VoucherRow, ServiceError> {
    let (closed, _) = day::window(ctx)?;
    let mut rows = vec![ctx.db.read(|conn| {
        repo::get(conn, id)?.ok_or_else(|| ServiceError::reject_field(ErrorCode::ErrNotFound, "id"))
    })?];
    super::mark_editable(&mut rows, closed);
    rows.pop()
        .ok_or_else(|| ServiceError::reject_field(ErrorCode::ErrNotFound, "id"))
}
