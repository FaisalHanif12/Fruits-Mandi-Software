//! Placing a partner's opening capital in روکڑ (PRD FR-16.14 v1.50, FR-10.9, FR-6.3, FR-21.4, plan 87).
//!
//! A partner's opening capital entered before PRD 17.1 v1.43 stands against ابتدائی فرق (his capital
//! account جمع, ابتدائی فرق بنام), so the money is in his total but not in روکڑ. This fix reverses that
//! opening entry and posts the same amount again on the same day with روکڑ بنام on the other side.
//! The change of an opening is the one of FR-10.9 (`accounts::openings`), so a reversal is written
//! and nothing is deleted (FR-6.3). Cash only goes up, so the cash question of FR-6.7 cannot come up.
//!
//! The opening day is usually closed. The days are reopened from the first of them, the openings are
//! moved and the same days are closed again, all in one transaction (Code Standards B3). If anything
//! fails, nothing changes. The Owner's sign is checked first, as FR-21.4: a wrong sign changes nothing
//! and is audited without its text.
//!
//! No partner total changes (his capital account is untouched), nor does any income, expense, share
//! row, matching row or analytics fact. The trial balance stays balanced.
//!
//! Example (Umar's data): two openings of 500,000 against ابتدائی فرق; after the fix روکڑ is 1,000,000
//! higher and ابتدائی فرق changes by the same amount.

use mandi_db::accounts as account_repo;
use mandi_db::partners as partner_repo;
use mandi_db::rusqlite::Connection;
use mandi_domain::core::accounts::{OPENING_DIFFERENCE_CODE, OpeningDay, RecordRef};
use mandi_domain::core::dates::{BusinessDate, format_ddmmyyyy};
use mandi_domain::core::error::{ErrorCode, FieldError};
use mandi_domain::core::money::Paisa;
use mandi_domain::core::partners::{
    PartnersOpeningCapitalPlaced, PartnersPlaceOpeningCapitalInput, PlacedOpeningCapital,
};
use mandi_domain::core::validate::{self, FieldErrors};
use mandi_domain::core::vouchers::DayCloseInput;
use serde_json::json;

use crate::accounts::{OpeningCounter, apply_opening_with, current_opening};
use crate::audit::Event;
use crate::auth::Actor;
use crate::cash::{CASH_ACCOUNT_CODE, cash_in_hand};
use crate::context::Context;
use crate::day::{self, SignCheck};
use crate::error::ServiceError;

/// The audit action written when the sign is wrong.
const WRONG_SIGN_ACTION: &str = "capital_place_sign_wrong";
/// The audit action of one opening moved.
const PLACED_ACTION: &str = "opening_capital_placed";

/// An opening capital that can be moved: the partner and the opening entry in force.
struct Candidate {
    partner_id: i64,
    capital_account_id: i64,
    entry_id: i64,
    date: BusinessDate,
    amount: Paisa,
}

fn rec(id: i64, name_ur: String, name_en: String) -> RecordRef {
    RecordRef {
        id,
        code: String::new(),
        name_ur,
        name_en,
        tag: None,
    }
}

/// The partner's opening capital in force, if it stands against ابتدائی فرق: a credit on his capital
/// account, an entry with a line on ابتدائی فرق and none on روکڑ. The amount and the day come from the
/// books (S9).
fn candidate_of(
    conn: &Connection,
    partner: &partner_repo::PartnerRecord,
    difference_id: i64,
    cash_id: i64,
) -> Result<Option<Candidate>, ServiceError> {
    let Some(facts) = account_repo::facts(conn, partner.capital_account_id)? else {
        return Ok(None);
    };
    let have = current_opening(&facts)?;
    let (Some(entry_id), Some(date)) = (facts.opening_entry_id, have.date) else {
        return Ok(None);
    };
    // His opening capital is a credit on his capital account (not a debit: that would be money taken).
    if !have.has_amount() || have.cr == Paisa::ZERO || have.dr != Paisa::ZERO {
        return Ok(None);
    }
    let Some(entry) = mandi_db::ledger::get_entry(conn, entry_id)? else {
        return Ok(None);
    };
    let on = |account: i64| entry.lines.iter().any(|l| l.account_id == account);
    if entry.status != "posted" || !on(difference_id) || on(cash_id) {
        return Ok(None);
    }
    Ok(Some(Candidate {
        partner_id: partner.id,
        capital_account_id: partner.capital_account_id,
        entry_id,
        date,
        amount: have.cr,
    }))
}

/// Moves the opening capital of the listed partners into روکڑ.
///
/// # Errors
/// `ERR_REASON_REQUIRED` on `reason`; `ERR_SIGN_REQUIRED` and `ERR_SIGN_WRONG` on `sign`;
/// `ERR_REQUIRED` and `ERR_NOT_FOUND` on `partner_ids`; `ERR_NOTHING_TO_PLACE` on `partner_ids`
/// (none of them has an opening capital against ابتدائی فرق); `ERR_MONTH_CLOSED` on `date` (a month
/// close sits on the days to reopen); `ERR_TRIAL_NOT_BALANCED` and the other refusals of closing a
/// day again. Nothing is written when any of them is returned. The caller checks the partner code.
pub fn place(
    ctx: &Context,
    actor: &Actor,
    input: &PartnersPlaceOpeningCapitalInput,
) -> Result<PartnersOpeningCapitalPlaced, ServiceError> {
    // The sign first: a wrong sign writes only its own audit row (the refusal must not undo it).
    let sign = day::check_sign(
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
    if input.partner_ids.is_empty() {
        errors.push(FieldError::new(ErrorCode::ErrRequired, "partner_ids"));
    }
    errors.finish()?;
    let (placed, skipped, reopened_from) = ctx.db.write(|tx| {
        let difference_id = account_repo::id_by_code(tx, OPENING_DIFFERENCE_CODE)?
            .ok_or_else(|| ServiceError::Data("the account ابتدائی فرق is missing".to_owned()))?;
        let cash_id = account_repo::id_by_code(tx, CASH_ACCOUNT_CODE)?
            .ok_or_else(|| ServiceError::Data("the cash account is missing".to_owned()))?;
        // Every listed partner is read inside the transaction, so the state cannot move under us.
        let mut candidates: Vec<(Candidate, partner_repo::PartnerRecord)> = Vec::new();
        let mut skipped: Vec<RecordRef> = Vec::new();
        let mut seen: Vec<i64> = Vec::new();
        for (index, id) in input.partner_ids.iter().enumerate() {
            if seen.contains(id) {
                continue;
            }
            seen.push(*id);
            let Some(partner) = partner_repo::get(tx, *id)? else {
                return Err(ServiceError::reject_field(
                    ErrorCode::ErrNotFound,
                    &format!("partner_ids[{index}]"),
                ));
            };
            match candidate_of(tx, &partner, difference_id, cash_id)? {
                Some(candidate) => candidates.push((candidate, partner)),
                None => skipped.push(rec(partner.id, partner.name_ur, partner.name_en)),
            }
        }
        let Some(earliest) = candidates.iter().map(|(c, _)| c.date).min() else {
            return Err(ServiceError::reject_field(
                ErrorCode::ErrNothingToPlace,
                "partner_ids",
            ));
        };
        // The days to reopen: from the first opening day, when it is closed.
        let closed_before = day::closed_in(tx)?;
        let closed_up_to = closed_before.filter(|closed| earliest <= *closed);
        if closed_up_to.is_some() {
            day::reopen_in(ctx, tx, actor, earliest, &reason, Some(&sign))?;
        }
        let now = ctx.now_text();
        let mut placed: Vec<PlacedOpeningCapital> = Vec::new();
        for (candidate, partner) in candidates {
            let facts = account_repo::facts(tx, candidate.capital_account_id)?
                .ok_or_else(|| ServiceError::Data("the capital account is missing".to_owned()))?;
            // The same opening (day, amount) again, with روکڑ on the other side.
            let wanted = current_opening(&facts)?;
            let written = apply_opening_with(
                ctx,
                tx,
                actor.user_id,
                &now,
                &facts,
                &wanted,
                "partner_ids",
                OpeningDay::Open,
                OpeningCounter::Cash,
                // Cash goes up, so no cash question can come up (PRD FR-6.7).
                false,
            )?;
            if !written {
                return Err(ServiceError::Data(
                    "an opening capital was not moved".to_owned(),
                ));
            }
            let new_entry = account_repo::facts(tx, candidate.capital_account_id)?
                .and_then(|f| f.opening_entry_id)
                .ok_or_else(|| ServiceError::Data("the new opening entry is missing".to_owned()))?;
            ctx.db.auditor().write(
                tx,
                &Event::new(Some(actor.user_id), PLACED_ACTION, "account")
                    .entity_id(candidate.capital_account_id)
                    .old(json!({
                        "opening_entry_id": candidate.entry_id,
                        "other_side": "opening_difference",
                        "date": candidate.date.to_iso(),
                        "amount_paisa": candidate.amount.as_paisa(),
                    }))
                    .new_value(json!({
                        "opening_entry_id": new_entry,
                        "other_side": "cash",
                        "date": candidate.date.to_iso(),
                        "amount_paisa": candidate.amount.as_paisa(),
                        "partner_id": candidate.partner_id,
                    }))
                    .sign(Some(&sign))
                    .reason(&reason)
                    .into_entry(),
            )?;
            placed.push(PlacedOpeningCapital {
                partner: rec(partner.id, partner.name_ur, partner.name_en),
                date: candidate.date.to_iso(),
                amount_paisa: candidate.amount,
                old_entry_id: candidate.entry_id,
                new_entry_id: new_entry,
            });
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
        Ok::<_, ServiceError>((placed, skipped, closed_up_to.map(|_| earliest)))
    })?;
    let cash_after = ctx.db.read(cash_in_hand)?;
    Ok(PartnersOpeningCapitalPlaced {
        placed,
        skipped,
        cash_after_paisa: cash_after,
        reopened_from: reopened_from.map(BusinessDate::to_iso),
        closed_up_to: day::status(ctx)?.closed_up_to,
    })
}
