//! The dead amount (ڈوبی رقم, PRD FR-10.10, plan 40): move a person who owes the shop out of the
//! daily lists, and make him active again.
//!
//! Every user action is one function and one transaction (Code Standards B3). **Nothing posts**: a
//! move only makes the account inactive (a پختہ کھاتہ person only gets his key marked, because the
//! پختہ accounts stay active, PRD FR-10.4), writes a `dead_marks` row and an audit entry. The
//! balance stays on his account, so the trial balance, the partner split and every partner's
//! کل رقم do not change (PRD A65). The preview and the move work out the amount with the same
//! function (Code Standards B5).
//!
//! Who can be moved (FR-10.10): a buyer; a seller whose ledger balance is بنام (his open advance
//! and payables are netted into that balance, A66); a پختہ کھاتہ person by his person key; a person
//! with ادھار رقم (control 0103). Partners, الگ کھاتے, gardens and system accounts never are.
//!
//! Money words (the money agent): what he owes is `max(0, Σ debit − Σ credit)` of his posted
//! lines (`mandi_domain::core::dead::owed_now`). It must be above 0 or the move is refused
//! (`ERR_NOTHING_OWED`).
//!
//! Example (PRD E2E-44): ارشد owes 18,684 on 11-10-26. The move keeps `owed_paisa` 18,684 and the
//! entry id of that moment, makes his account inactive and posts nothing; cash stays 58,484.

use mandi_db::accounts as account_repo;
use mandi_db::dead as repo;
use mandi_db::ledger as ledger_repo;
use mandi_db::rusqlite::Connection;
use mandi_db::vouchers as voucher_repo;
use mandi_domain::core::accounts::{BUYER_CONTROL, MONEY_LENT_CONTROL, RecordRef, SELLER_CONTROL};
use mandi_domain::core::dead::{
    DeadInfo, DeadLinked, DeadMarkInput, DeadMarked, DeadPartyType, DeadPreview, DeadPreviewInput,
    DeadReason, DeadUndoInput, DeadUndone, MAX_REASON_CHARS, owed_now,
};
use mandi_domain::core::error::{ErrorCode, ErrorValue, FieldError};
use mandi_domain::core::money::Paisa;
use mandi_domain::core::validate::{self, FieldErrors};
use serde_json::json;

use crate::audit::Event;
use crate::auth::Actor;
use crate::context::Context;
use crate::day;
use crate::error::{ServiceError, money_error};
use crate::ledger;
use crate::search;

fn paisa(value: i64) -> Result<Paisa, ServiceError> {
    Paisa::try_from_paisa(value).map_err(money_error)
}

/// What kind of person this account holds, or `None` for an account that is never moved.
fn kind_of(facts: &account_repo::AccountFacts) -> Option<DeadPartyType> {
    if facts.is_pukhta_khata {
        return Some(DeadPartyType::Pukhta);
    }
    // System accounts (نقدی منڈی, اپنا مال, cash, the income and expense heads) are never moved.
    if facts.is_system {
        return None;
    }
    match facts.control_code.as_str() {
        BUYER_CONTROL => Some(DeadPartyType::Buyer),
        SELLER_CONTROL => Some(DeadPartyType::Seller),
        MONEY_LENT_CONTROL => Some(DeadPartyType::MoneyLent),
        _ => None,
    }
}

/// The account (and person) a move is about, once found.
struct Target {
    facts: account_repo::AccountFacts,
    kind: Option<DeadPartyType>,
    /// The person key, for a پختہ کھاتہ account.
    key: Option<String>,
}

/// Why a person cannot be moved, with the field the screen marks.
struct Refusal {
    code: ErrorCode,
    field: &'static str,
}

/// Finds the account and checks who it is: it exists, its kind can be moved, a پختہ کھاتہ account
/// names a person who is on the books, any other account names none, and he is not moved already.
///
/// `Ok((target, Some(refusal)))` is a person who cannot be moved (the preview shows it as
/// `allowed: false`, the move turns it into an error). `Err` is a mistake in the input or a
/// database error.
fn resolve(
    conn: &Connection,
    account_id: i64,
    person_key: Option<&str>,
) -> Result<(Target, Option<Refusal>), ServiceError> {
    let facts = account_repo::facts(conn, account_id)?
        .ok_or_else(|| ServiceError::reject_field(ErrorCode::ErrNotFound, "account_id"))?;
    let typed_key = person_key.map(str::trim).filter(|k| !k.is_empty());
    let kind = kind_of(&facts);
    let key = match (kind, typed_key) {
        (Some(DeadPartyType::Pukhta), None) => {
            return Err(ServiceError::reject_field(
                ErrorCode::ErrPersonRequired,
                "pukhta_person_key",
            ));
        }
        (Some(DeadPartyType::Pukhta), Some(key)) => {
            if !voucher_repo::person_has_lines(conn, facts.id, key)? {
                return Err(ServiceError::reject_field(
                    ErrorCode::ErrNotFound,
                    "pukhta_person_key",
                ));
            }
            Some(key.to_owned())
        }
        (_, Some(_)) => {
            return Err(ServiceError::reject_field(
                ErrorCode::ErrFormatInvalid,
                "pukhta_person_key",
            ));
        }
        (_, None) => None,
    };
    let refusal = if kind.is_none() {
        Some(Refusal {
            code: ErrorCode::ErrDeadNotAllowed,
            field: "account_id",
        })
    } else if repo::active_for(conn, facts.id, key.as_deref())?.is_some() {
        Some(if key.is_some() {
            Refusal {
                code: ErrorCode::ErrPersonInDeadAmount,
                field: "pukhta_person_key",
            }
        } else {
            Refusal {
                code: ErrorCode::ErrAlreadyDead,
                field: "account_id",
            }
        })
    } else {
        None
    };
    Ok((Target { facts, kind, key }, refusal))
}

/// What he owes now: `max(0, Σ debit − Σ credit)` of his posted lines.
fn owed_of(conn: &Connection, target: &Target) -> Result<Paisa, ServiceError> {
    still_owed(conn, target.facts.id, target.key.as_deref())
}

/// What moving a person would do: what he owes, whether he can be moved and the code that says why
/// not, the day of his last posting and his linked account with its balance. Read only.
///
/// # Errors
/// `ERR_NOT_FOUND` on `account_id`; `ERR_PERSON_REQUIRED`, `ERR_NOT_FOUND` and `ERR_FORMAT_INVALID`
/// on `pukhta_person_key`. A person who cannot be moved is not an error: `allowed` is false.
pub fn preview(ctx: &Context, input: &DeadPreviewInput) -> Result<DeadPreview, ServiceError> {
    ctx.db.read(|conn| {
        let (target, refusal) =
            resolve(conn, input.account_id, input.pukhta_person_key.as_deref())?;
        let owed = owed_of(conn, &target)?;
        let reason_not_allowed = refusal
            .map(|r| r.code)
            .or_else(|| (owed == Paisa::ZERO).then_some(ErrorCode::ErrNothingOwed));
        Ok(DeadPreview {
            owed_paisa: owed,
            allowed: reason_not_allowed.is_none(),
            reason_not_allowed,
            last_posting_date: repo::last_posting_date(
                conn,
                target.facts.id,
                target.key.as_deref(),
            )?,
            linked: linked_of(conn, target.facts.id)?,
        })
    })
}

/// The account linked to this one (PRD E10) and his balance. It is not moved with him (A68).
fn linked_of(conn: &Connection, account_id: i64) -> Result<Option<DeadLinked>, ServiceError> {
    let Some(row) = account_repo::get(conn, account_id)? else {
        return Ok(None);
    };
    let Some(RecordRef {
        id,
        code,
        name_ur,
        name_en,
        tag,
    }) = row.linked_account
    else {
        return Ok(None);
    };
    let balance = ledger::balance_in(conn, id, None, None)?;
    Ok(Some(DeadLinked {
        account_id: id,
        code,
        name_ur,
        name_en,
        tag,
        balance,
    }))
}

/// Moves a person to the dead amount (PRD FR-10.10). One transaction: the checks, the account made
/// inactive (not for a پختہ person), the move with what he owes that day and the entry id of that
/// moment, and the audit row. Nothing posts.
///
/// # Errors
/// `ERR_NOT_FOUND` (`account_id`); `ERR_NOTHING_OWED`, `ERR_DEAD_NOT_ALLOWED`, `ERR_ALREADY_DEAD`
/// (`account_id`); `ERR_PERSON_REQUIRED` and `ERR_PERSON_IN_DEAD_AMOUNT` (`pukhta_person_key`);
/// `ERR_NOTE_REQUIRED` (`note`); the text codes on `note`; `ERR_DATE_CLOSED`, `ERR_DATE_IN_FUTURE`
/// and the date codes on `entry_date`; `ERR_DATE_BEFORE_LAST_POSTING` (`entry_date`, value
/// `last_posting_date`).
pub fn mark(
    ctx: &Context,
    actor: &Actor,
    input: &DeadMarkInput,
) -> Result<DeadMarked, ServiceError> {
    let (closed, today) = day::window(ctx)?;
    let mut errors = FieldErrors::default();
    let date = errors.take(validate::posting_date(
        "entry_date",
        &input.entry_date,
        closed,
        today,
    ));
    let note = errors
        .take(validate::free_text("note", &input.note))
        .unwrap_or_default();
    if input.reason == DeadReason::Other && note.is_empty() {
        errors.push(FieldError::new(ErrorCode::ErrNoteRequired, "note"));
    }

    let (mark_id, owed, account_id) = ctx.db.write(|tx| {
        let (target, refusal) = resolve(tx, input.account_id, input.pukhta_person_key.as_deref())?;
        let mut owed = Paisa::ZERO;
        if let Some(refusal) = refusal {
            errors.push(FieldError::new(refusal.code, refusal.field));
        } else {
            owed = owed_of(tx, &target)?;
            if owed == Paisa::ZERO {
                errors.push(FieldError::new(ErrorCode::ErrNothingOwed, "account_id"));
            }
            // Plan 40 D6: not before his last posting, so the move never lands in his past.
            if let Some(date) = date
                && let Some(last) =
                    repo::last_posting_date(tx, target.facts.id, target.key.as_deref())?
                && date.to_iso() < last
            {
                errors.push(
                    FieldError::new(ErrorCode::ErrDateBeforeLastPosting, "entry_date")
                        .with("last_posting_date", ErrorValue::Text(last)),
                );
            }
        }
        errors.finish()?;
        let date = date.ok_or_else(|| {
            ServiceError::Data("the date of a move was checked but is missing".to_owned())
        })?;

        let now = ctx.now_text();
        let mark_id = repo::insert(
            tx,
            &repo::NewMark {
                account_id: target.facts.id,
                person_key: target.key.as_deref(),
                entry_date: &date.to_iso(),
                reason: input.reason.as_str(),
                note: &note,
                owed_paisa: owed.as_paisa(),
            },
            actor.user_id,
            &now,
        )?;
        // A پختہ person: the account stays active, only his key is marked (PRD FR-10.10).
        let was_active = target.facts.is_active;
        let made_inactive = target.kind != Some(DeadPartyType::Pukhta) && was_active;
        if made_inactive {
            account_repo::set_active(tx, target.facts.id, false, actor.user_id, &now)?;
        }
        ctx.db.auditor().write(
            tx,
            &Event::new(Some(actor.user_id), "dead_mark", "account")
                .entity_id(target.facts.id)
                .old(json!({ "is_active": was_active, "person_key": target.key }))
                .new_value(json!({
                    "mark_id": mark_id,
                    "entry_date": date.to_iso(),
                    "reason": input.reason.as_str(),
                    "note": note,
                    "owed_paisa": owed.as_paisa(),
                    "is_active": was_active && !made_inactive,
                }))
                .into_entry(),
        )?;
        Ok::<_, ServiceError>((mark_id, owed, target.facts.id))
    })?;
    search::refresh_account(ctx, account_id);
    Ok(DeadMarked {
        mark_id,
        owed_paisa: owed,
    })
}

/// Makes a person active again, with a reason ("دوبارہ فعال کریں", PRD FR-10.10). One transaction:
/// the move ends (its figures stop here), the account is active again (not a پختہ person: only his
/// key is free), and the audit row. His balance goes back into the normal tiles and lists.
///
/// # Errors
/// `ERR_REASON_REQUIRED` and the text codes on `reason`; `ERR_NOT_FOUND` on `mark_id`, also for a
/// move that was undone already.
pub fn undo(
    ctx: &Context,
    actor: &Actor,
    input: &DeadUndoInput,
) -> Result<DeadUndone, ServiceError> {
    let reason = validate::free_text("reason", &input.reason).map_err(ServiceError::from)?;
    if reason.is_empty() {
        return Err(ServiceError::reject_field(
            ErrorCode::ErrReasonRequired,
            "reason",
        ));
    }
    if reason.chars().count() > MAX_REASON_CHARS {
        return Err(ServiceError::reject_field(
            ErrorCode::ErrTextTooLong,
            "reason",
        ));
    }
    let (_, today) = day::window(ctx)?;
    let account_id = ctx.db.write(|tx| {
        let row = repo::get(tx, input.mark_id)?
            .filter(|m| m.is_active)
            .ok_or_else(|| ServiceError::reject_field(ErrorCode::ErrNotFound, "mark_id"))?;
        let now = ctx.now_text();
        if !repo::undo(tx, row.id, &reason, &today.to_iso(), actor.user_id, &now)? {
            return Err(ServiceError::reject_field(
                ErrorCode::ErrNotFound,
                "mark_id",
            ));
        }
        if row.person_key.is_none() {
            account_repo::set_active(tx, row.account_id, true, actor.user_id, &now)?;
        }
        ctx.db.auditor().write(
            tx,
            &Event::new(Some(actor.user_id), "dead_undo", "account")
                .entity_id(row.account_id)
                .old(json!({
                    "mark_id": row.id,
                    "entry_date": row.entry_date,
                    "reason": row.reason,
                    "owed_paisa": row.owed_paisa,
                    "person_key": row.person_key,
                }))
                .new_value(json!({ "mark_id": row.id, "is_active": row.person_key.is_none() }))
                .reason(&reason)
                .into_entry(),
        )?;
        Ok::<_, ServiceError>(row.account_id)
    })?;
    search::refresh_account(ctx, account_id);
    Ok(DeadUndone { ok: true })
}

/// The move of an account, as the party card and the ledger heading show it (PRD FR-7.38). Null for
/// an account that is not moved, and for a پختہ کھاتہ account (its people are moved one by one).
///
/// # Errors
/// `ServiceError::Db`.
pub fn info_of(conn: &Connection, account_id: i64) -> Result<Option<DeadInfo>, ServiceError> {
    Ok(repo::active_for_account(conn, account_id)?.map(|row| row.info()))
}

/// The same, for one person of a پختہ کھاتہ account.
///
/// # Errors
/// `ServiceError::Db`.
pub fn info_of_person(
    conn: &Connection,
    account_id: i64,
    person_key: &str,
) -> Result<Option<DeadInfo>, ServiceError> {
    Ok(repo::active_for(conn, account_id, Some(person_key))?.map(|row| row.info()))
}

/// True when this account (or this person of a پختہ کھاتہ account) is in the dead amount.
///
/// # Errors
/// `ServiceError::Db`.
pub fn is_dead(
    conn: &Connection,
    account_id: i64,
    person_key: Option<&str>,
) -> Result<bool, ServiceError> {
    Ok(repo::active_for(conn, account_id, person_key)?.is_some())
}

/// What a person still owes: `max(0, Σ debit − Σ credit)` of his posted lines. A recovery may not
/// be more than this for a person in the dead amount (PRD FR-10.10, plan 40 D2).
///
/// # Errors
/// `ServiceError::Db`; `ServiceError::Rejected` with a money code if a sum does not fit.
pub fn still_owed(
    conn: &Connection,
    account_id: i64,
    person_key: Option<&str>,
) -> Result<Paisa, ServiceError> {
    let totals = ledger_repo::totals_of(conn, account_id, person_key, None)?;
    owed_now(paisa(totals.dr_paisa)?, paisa(totals.cr_paisa)?).map_err(money_error)
}

#[cfg(test)]
mod tests {
    use mandi_db::accounts::AccountFacts;

    use super::*;

    fn facts(control: &str, system: bool, pukhta: bool) -> AccountFacts {
        AccountFacts {
            id: 1,
            version: 1,
            code: "x".to_owned(),
            control_code: control.to_owned(),
            head_code: "03".to_owned(),
            is_system: system,
            is_pukhta_khata: pukhta,
            is_active: true,
            opening_entry_id: None,
            opening_date: None,
            opening_dr_paisa: 0,
            opening_cr_paisa: 0,
            opening_is_advance: false,
        }
    }

    #[test]
    fn only_buyers_sellers_pukhta_and_lent_persons_can_be_moved() {
        assert_eq!(
            kind_of(&facts("0304", false, false)),
            Some(DeadPartyType::Buyer)
        );
        assert_eq!(
            kind_of(&facts("03011", false, false)),
            Some(DeadPartyType::Seller)
        );
        assert_eq!(
            kind_of(&facts("0103", false, false)),
            Some(DeadPartyType::MoneyLent)
        );
        assert_eq!(
            kind_of(&facts("0304", false, true)),
            Some(DeadPartyType::Pukhta)
        );
        // A system account (the cash buyer, the firm's own goods, cash) never is.
        assert_eq!(kind_of(&facts("0304", true, false)), None);
        // A الگ کھاتہ, a garden, a partner's account and an income head never are.
        for control in ["0305", "0306", "0602", "0603", "0604", "0401"] {
            assert_eq!(kind_of(&facts(control, false, false)), None, "{control}");
        }
    }
}
