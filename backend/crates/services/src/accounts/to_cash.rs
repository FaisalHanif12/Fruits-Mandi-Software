//! The one-time fix "پرانے ابتدائی بقایا روکڑ میں درج کریں" (PRD FR-10.9 v1.60, FR-6.3, FR-21.4,
//! plan 99): the openings of sellers, buyers, persons of money lent and الگ کھاتے that were saved
//! against ابتدائی فرق are reversed and posted again on the same day against روکڑ.
//!
//! One save, one transaction (Code Standards B3): the closed days are reopened from the first of
//! them, each opening is reversed (nothing is deleted) and posted again at its own place in its
//! day (its order key), the matching is made again, and the same days are closed again with the
//! checks of FR-21.1. If anything fails, nothing changes. The reason and the Owner's sign are
//! required; a wrong sign changes nothing and is audited without its text.
//!
//! No party balance changes, no partner total changes (his parts of linked money are checked to be
//! the same lines as before), no income, expense or share row. Cash changes by Σ جمع − Σ بنام, and
//! ابتدائی فرق by the opposite. A second run has nothing to move: `ERR_OPENINGS_ALREADY_IN_CASH`.
//!
//! Example (plan 99): openings بنام 125,000 and جمع 40,000 move: cash 1,057,840 becomes 972,840 and
//! ابتدائی فرق 85,000 جمع becomes 0.

use mandi_db::accounts as repo;
use mandi_db::rusqlite::Connection;
use mandi_domain::core::accounts::{
    BUYER_CONTROL, Balance, KHATA_CONTROL, MONEY_LENT_CONTROL, OPENING_DIFFERENCE_CODE,
    OpeningPartyKind, OpeningRequest, OpeningToCashRow, OpeningsToCashDone, OpeningsToCashInput,
    OpeningsToCashPreview, RecordRef, SELLER_CONTROL,
};
use mandi_domain::core::dates::{BusinessDate, format_ddmmyyyy};
use mandi_domain::core::error::ErrorCode;
use mandi_domain::core::ids::AccountId;
use mandi_domain::core::money::Paisa;
use mandi_domain::core::validate::{self, FieldErrors};
use mandi_domain::core::vouchers::DayCloseInput;
use mandi_domain::ledger::{PostingType, Side, SourceType, build_opening};
use serde_json::json;

use super::current_opening;
use crate::audit::Event;
use crate::auth::Actor;
use crate::cash::{CASH_ACCOUNT_CODE, cash_after_entry, cash_in_hand, require_confirmed};
use crate::context::Context;
use crate::day::{self, SignCheck};
use crate::error::{ServiceError, money_error};
use crate::floor::{self, check_after, is_returnable_control};
use crate::ledger::{self, PostingEvent};
use crate::matching;
use crate::search;

/// The audit action written when the sign is wrong.
const WRONG_SIGN_ACTION: &str = "openings_to_cash_sign_wrong";
/// The audit action of one opening moved.
const MOVED_ACTION: &str = "opening_moved_to_cash";
/// The narration of the reversal.
const REVERSAL_NOTE: &str = "opening moved to cash";
/// The controls of the four kinds of party (PRD FR-10.9 v1.60).
const PARTY_CONTROLS: [&str; 4] = [
    SELLER_CONTROL,
    BUYER_CONTROL,
    MONEY_LENT_CONTROL,
    KHATA_CONTROL,
];
/// The controls whose lines are the partners' side of linked money (PRD FR-16.6, FR-16.13).
const LINKED_CONTROLS: [&str; 2] = ["0605", "0307"];

fn data(message: &str) -> ServiceError {
    ServiceError::Data(message.to_owned())
}

/// One opening to move.
struct Candidate {
    facts: repo::AccountFacts,
    account: RecordRef,
    kind: OpeningPartyKind,
    opening: OpeningRequest,
    date: BusinessDate,
    entry_id: i64,
}

impl Candidate {
    /// What it does to cash: جمع comes in, بنام goes out.
    fn cash_change(&self) -> Result<Paisa, ServiceError> {
        self.opening
            .cr
            .checked_sub(self.opening.dr)
            .map_err(money_error)
    }

    fn row(&self, closed: Option<BusinessDate>) -> Result<OpeningToCashRow, ServiceError> {
        Ok(OpeningToCashRow {
            account: self.account.clone(),
            kind: self.kind,
            date: self.date.to_iso(),
            dr_paisa: self.opening.dr,
            cr_paisa: self.opening.cr,
            cash_change_paisa: self.cash_change()?,
            day_closed: closed.is_some_and(|c| self.date <= c),
        })
    }
}

fn kind_of(control_code: &str) -> Result<OpeningPartyKind, ServiceError> {
    match control_code {
        SELLER_CONTROL => Ok(OpeningPartyKind::Seller),
        BUYER_CONTROL => Ok(OpeningPartyKind::Buyer),
        MONEY_LENT_CONTROL => Ok(OpeningPartyKind::MoneyLent),
        KHATA_CONTROL => Ok(OpeningPartyKind::SeparateKhata),
        _ => Err(data("an opening of an account that is not a party")),
    }
}

fn cash_id(conn: &Connection) -> Result<i64, ServiceError> {
    repo::id_by_code(conn, CASH_ACCOUNT_CODE)?.ok_or_else(|| data("the cash account is missing"))
}

fn difference_id(conn: &Connection) -> Result<i64, ServiceError> {
    repo::id_by_code(conn, OPENING_DIFFERENCE_CODE)?
        .ok_or_else(|| data("the account ابتدائی فرق is missing"))
}

/// Every opening to move, by day then account. Read inside the caller's connection, so the save
/// sees the state it changes.
fn candidates(conn: &Connection) -> Result<Vec<Candidate>, ServiceError> {
    let ids = repo::openings_not_in_cash(conn, &PARTY_CONTROLS, cash_id(conn)?)?;
    let mut found = Vec::with_capacity(ids.len());
    for id in ids {
        let facts = repo::facts(conn, id)?.ok_or_else(|| data("an account is missing"))?;
        let record = repo::get(conn, id)?.ok_or_else(|| data("an account is missing"))?;
        let opening = current_opening(&facts)?;
        let (Some(date), Some(entry_id)) = (opening.date, facts.opening_entry_id) else {
            return Err(data("an opening without its day or its entry"));
        };
        found.push(Candidate {
            kind: kind_of(&facts.control_code)?,
            account: RecordRef {
                id: record.id,
                code: record.code,
                name_ur: record.name_ur,
                name_en: record.name_en,
                tag: record.tag,
            },
            facts,
            opening,
            date,
            entry_id,
        });
    }
    Ok(found)
}

/// The totals of the openings: Σ بنام, Σ جمع and the change of cash.
fn totals(found: &[Candidate]) -> Result<(Paisa, Paisa, Paisa), ServiceError> {
    let (mut dr, mut cr) = (Paisa::ZERO, Paisa::ZERO);
    for c in found {
        dr = dr.checked_add(c.opening.dr).map_err(money_error)?;
        cr = cr.checked_add(c.opening.cr).map_err(money_error)?;
    }
    Ok((dr, cr, cr.checked_sub(dr).map_err(money_error)?))
}

/// What the fix would do (the same openings as the save, Code Standards B5). Read only.
///
/// # Errors
/// `ServiceError::Db`.
pub fn openings_to_cash_preview(ctx: &Context) -> Result<OpeningsToCashPreview, ServiceError> {
    let (closed, _) = day::window(ctx)?;
    ctx.db.read(|conn| {
        let found = candidates(conn)?;
        let (total_dr, total_cr, change) = totals(&found)?;
        let before = cash_in_hand(conn)?;
        let check = mandi_domain::cash::check(before, change).map_err(money_error)?;
        let difference = ledger::balance_in(conn, difference_id(conn)?, None, None)?;
        // Each moved opening counted `cr − dr` on ابتدائی فرق (debit positive); it leaves.
        let after = ledger::signed(difference)?
            .checked_sub(change)
            .map_err(money_error)?;
        let rows = found
            .iter()
            .map(|c| c.row(closed))
            .collect::<Result<Vec<_>, _>>()?;
        let reopens_from = rows.iter().find(|r| r.day_closed).map(|r| r.date.clone());
        Ok(OpeningsToCashPreview {
            rows,
            total_dr_paisa: total_dr,
            total_cr_paisa: total_cr,
            cash_change_paisa: change,
            cash_before_paisa: before,
            cash_after_paisa: check.cash_after_paisa,
            needs_cash_confirm: check.needs_confirm,
            opening_difference_before: difference,
            opening_difference_after: Balance::from_net(after).map_err(money_error)?,
            reopens_from,
        })
    })
}

/// Moves every party opening that is not in cash into روکڑ.
///
/// # Errors
/// `ERR_SIGN_REQUIRED`, `ERR_SIGN_WRONG` on `sign`; `ERR_REASON_REQUIRED` on `reason`;
/// `ERR_OPENINGS_ALREADY_IN_CASH`; `ERR_CASH_BELOW_ZERO_NOT_CONFIRMED` (value `cash_after_paisa`,
/// one check for the whole fix); `ERR_MONTH_CLOSED` on `date`; the refusals of closing a day
/// again. Nothing is written when any of them is returned.
pub fn openings_to_cash(
    ctx: &Context,
    actor: &Actor,
    input: &OpeningsToCashInput,
) -> Result<OpeningsToCashDone, ServiceError> {
    // The sign first: a wrong sign writes only its own audit row (the refusal must not undo it).
    day::check_sign(
        ctx,
        actor,
        &SignCheck {
            reason: &input.reason,
            sign: &input.sign,
            date: "",
            wrong_action: WRONG_SIGN_ACTION,
        },
    )?;
    let mut errors = FieldErrors::default();
    let reason = errors
        .take(validate::free_text("reason", &input.reason))
        .unwrap_or_default();
    errors.finish()?;
    let (rows, totals_moved, reopened_from, ids) = ctx.db.write(|tx| {
        let found = candidates(tx)?;
        let Some(earliest) = found.iter().map(|c| c.date).min() else {
            return Err(ServiceError::reject(ErrorCode::ErrOpeningsAlreadyInCash));
        };
        let (total_dr, total_cr, change) = totals(&found)?;
        // PRD FR-6.7: one check for the whole fix, never one account at a time.
        if change.is_negative() {
            let check = cash_after_entry(tx, change)?;
            require_confirmed(&check, input.confirm_cash_below_zero)?;
        }
        // The days to reopen: from the first opening day, when it is closed.
        let closed_before = day::closed_in(tx)?;
        let closed_up_to = closed_before.filter(|closed| earliest <= *closed);
        if closed_up_to.is_some() {
            day::reopen_in(ctx, tx, actor, earliest, &reason, true)?;
        }
        let now = ctx.now_text();
        let cash = cash_id(tx)?;
        let mut rows = Vec::with_capacity(found.len());
        let mut ids = Vec::with_capacity(found.len());
        for candidate in &found {
            rows.push(candidate.row(closed_before)?);
            ids.push(candidate.facts.id);
            let new_entry = move_one(ctx, tx, actor, &now, cash, candidate)?;
            ctx.db.auditor().write(
                tx,
                &Event::new(Some(actor.user_id), MOVED_ACTION, "account")
                    .entity_id(candidate.facts.id)
                    .old(json!({
                        "opening_entry_id": candidate.entry_id,
                        "other_side": "opening_difference",
                    }))
                    .new_value(json!({
                        "opening_entry_id": new_entry,
                        "other_side": "cash",
                        "date": candidate.date.to_iso(),
                        "dr_paisa": candidate.opening.dr.as_paisa(),
                        "cr_paisa": candidate.opening.cr.as_paisa(),
                        "is_advance": candidate.opening.is_advance,
                    }))
                    .sign_given(true)
                    .reason(&reason)
                    .into_entry(),
            )?;
        }
        // Close the same days again, with the checks of FR-21.1. A failed check refuses the whole
        // change: nothing stays open or half done.
        if let Some(closed) = closed_up_to {
            day::close_in(
                ctx,
                tx,
                actor,
                &DayCloseInput {
                    up_to: format_ddmmyyyy(closed),
                },
            )?;
        }
        Ok::<_, ServiceError>((
            rows,
            (total_dr, total_cr),
            closed_up_to.map(|_| earliest),
            ids,
        ))
    })?;
    for id in ids {
        search::refresh_account(ctx, id);
    }
    let (cash_after, difference) = ctx.db.read(|conn| {
        Ok::<_, ServiceError>((
            cash_in_hand(conn)?,
            ledger::balance_in(conn, difference_id(conn)?, None, None)?,
        ))
    })?;
    Ok(OpeningsToCashDone {
        rows,
        total_dr_paisa: totals_moved.0,
        total_cr_paisa: totals_moved.1,
        cash_after_paisa: cash_after,
        opening_difference_after: difference,
        reopened_from: reopened_from.map(BusinessDate::to_iso),
        closed_up_to: day::status(ctx)?.closed_up_to,
    })
}

/// Reverses one opening and posts it again against روکڑ at the same place in its day. Returns the
/// new entry.
fn move_one(
    ctx: &Context,
    tx: &Connection,
    actor: &Actor,
    now: &str,
    cash: i64,
    candidate: &Candidate,
) -> Result<i64, ServiceError> {
    let facts = &candidate.facts;
    let floor_before = if is_returnable_control(&facts.control_code) {
        Some(floor::floor_of(tx, facts.id)?)
    } else {
        None
    };
    let opening = &candidate.opening;
    let (side, amount) = if opening.dr == Paisa::ZERO {
        (Side::Cr, opening.cr)
    } else {
        (Side::Dr, opening.dr)
    };
    let linked_before = linked_lines(tx, candidate.entry_id)?;
    let old_key = ledger::order_key_of(tx, candidate.entry_id)?;
    ledger::reverse(ctx, tx, actor.user_id, candidate.entry_id, REVERSAL_NOTE)?;
    let entry = build_opening(
        AccountId::new(facts.id),
        side,
        amount,
        opening.is_advance,
        AccountId::new(cash),
    )
    .map_err(ledger::ledger_error)?;
    let posted = ledger::post_moved_opening(
        ctx,
        tx,
        actor.user_id,
        &PostingEvent {
            date: candidate.date,
            posting_type: PostingType::Opening,
            source_type: SourceType::Account,
            source_id: facts.id,
            narration: None,
            reverses: None,
            entry,
        },
        old_key,
    )?;
    // The partners' parts of linked money must stay exactly as they were (FR-17.9).
    if linked_lines(tx, posted.entry_id)? != linked_before {
        return Err(data(
            "moving an opening would change the partners' linked parts",
        ));
    }
    repo::set_opening(
        tx,
        facts.id,
        Some(&candidate.date.to_iso()),
        opening.dr.as_paisa(),
        opening.cr.as_paisa(),
        opening.is_advance,
        Some(posted.entry_id),
        actor.user_id,
        now,
    )?;
    check_after(tx, floor_before, "opening_dr")?;
    if !facts.is_chukta_khata {
        if facts.control_code == BUYER_CONTROL {
            matching::rematch(tx, actor.user_id, now, facts.id, None)?;
        } else if facts.control_code == SELLER_CONTROL {
            matching::rematch_advances(tx, actor.user_id, now, facts.id)?;
        }
    }
    Ok(posted.entry_id)
}

/// The lines of an entry on the linked accounts, as sorted `(account, dr, cr)`.
fn linked_lines(tx: &Connection, entry_id: i64) -> Result<Vec<(i64, i64, i64)>, ServiceError> {
    let entry = mandi_db::ledger::get_entry(tx, entry_id)?
        .ok_or_else(|| data("an opening entry is missing"))?;
    let mut found = Vec::new();
    for line in &entry.lines {
        let facts =
            repo::facts(tx, line.account_id)?.ok_or_else(|| data("an account is missing"))?;
        if LINKED_CONTROLS.contains(&facts.control_code.as_str()) {
            found.push((line.account_id, line.dr_paisa, line.cr_paisa));
        }
    }
    found.sort_unstable();
    Ok(found)
}
