//! Opening balances (PRD FR-10.9): one account at a time from the form, many at once from the
//! grid. The preview and the save use the same plan (Code Standards B5).
//!
//! An opening is a posted entry against ابتدائی فرق (06019999). Changing it reverses the old
//! entry (on its own date, which must still be open) and posts a new one; an opening that did
//! not change writes nothing.

use mandi_db::accounts as repo;
use mandi_db::rusqlite::Connection;
use mandi_domain::core::accounts::{
    BUYER_CONTROL, Balance, LineError, OPENING_DIFFERENCE_CODE, OpeningPreviewRow, OpeningRequest,
    OpeningsInput, OpeningsPreview, OpeningsSaved, RecordRef, SELLER_CONTROL,
    opening_request_in_force,
};
use mandi_domain::core::dates::{BusinessDate, format_ddmmyyyy};
use mandi_domain::core::error::{ErrorCode, FieldError};
use mandi_domain::core::ids::AccountId;
use mandi_domain::core::money::Paisa;
use mandi_domain::core::validate;
use mandi_domain::ledger::{PostingType, Side, SourceType, build_opening};
use serde_json::json;

use super::{current_opening, opening_allowed};
use crate::audit::Event;
use crate::auth::Actor;
use crate::context::Context;
use crate::day;
use crate::error::{Rejection, ServiceError, money_error};
use crate::floor::{self, check_after, is_returnable_control};
use crate::ledger::{self, PostingEvent};
use crate::matching;
use crate::search;

fn difference_id(conn: &Connection) -> Result<i64, ServiceError> {
    repo::id_by_code(conn, OPENING_DIFFERENCE_CODE)?
        .ok_or_else(|| ServiceError::Data("the account ابتدائی فرق is missing".to_owned()))
}

/// True if the opening in force and the wanted one are the same (date, both amounts, advance).
fn same_opening(have: &OpeningRequest, wanted: &OpeningRequest) -> bool {
    if !have.has_amount() && !wanted.has_amount() {
        return true;
    }
    have.date == wanted.date
        && have.dr == wanted.dr
        && have.cr == wanted.cr
        && have.is_advance == wanted.is_advance
}

/// Moves a date error from the posting date field to the opening date field, and an
/// inactive-account refusal of the ledger (which has no field) to the opening amount field.
fn on_opening_date(error: ServiceError) -> ServiceError {
    match error {
        ServiceError::Rejected(items) => ServiceError::Rejected(
            items
                .into_iter()
                .map(|mut item| {
                    if item.field.as_deref() == Some("entry_date") {
                        item.field = Some("opening_date".to_owned());
                    } else if item.field.is_none() && item.code == ErrorCode::ErrAccountInactive {
                        item.field = Some("opening_dr".to_owned());
                    }
                    item
                })
                .collect(),
        ),
        other => other,
    }
}

/// Puts the path of a grid line in front of a field that has none yet, so every error of the
/// grid names `lines[i].<field>` (the screen cannot show an error that has no row).
fn on_line(error: ServiceError, index: usize) -> ServiceError {
    match error {
        ServiceError::Rejected(items) => ServiceError::Rejected(
            items
                .into_iter()
                .map(|mut item| {
                    item.field = Some(match item.field.take() {
                        Some(field) if field.starts_with("lines[") => field,
                        Some(field) => format!("lines[{index}].{field}"),
                        None => format!("lines[{index}]"),
                    });
                    item
                })
                .collect(),
        ),
        other => other,
    }
}

/// Puts the wanted opening on an account inside the caller's transaction. Returns true if
/// anything was written. `amount_field` names the field to blame if the account cannot take an
/// opening (`opening_dr` on the form, `lines[i].opening_dr` in the grid).
///
/// # Errors
/// `ERR_OPENING_NOT_ALLOWED`; the date codes (the old entry's date must be open too);
/// `ERR_AMOUNT_TOO_LARGE`; `ERR_JOURNAL_UNBALANCED` (a bug signal).
pub(crate) fn apply(
    ctx: &Context,
    tx: &Connection,
    user_id: i64,
    now: &str,
    facts: &repo::AccountFacts,
    wanted: &OpeningRequest,
    amount_field: &str,
) -> Result<bool, ServiceError> {
    let have = current_opening(facts)?;
    if same_opening(&have, wanted) {
        return Ok(false);
    }
    if wanted.has_amount() && !opening_allowed(facts) {
        return Err(ServiceError::reject_field(
            ErrorCode::ErrOpeningNotAllowed,
            amount_field,
        ));
    }
    // Money taken by a partner (0603) and money lent (0103) can only start with a debit: an
    // opening credit would be a return with nothing taken (PRD FR-10.9, FR-16.2, FR-16.5).
    if wanted.cr != Paisa::ZERO && is_returnable_control(&facts.control_code) {
        return Err(ServiceError::reject_field(
            ErrorCode::ErrOpeningNotAllowed,
            &amount_field.replacen("opening_dr", "opening_cr", 1),
        ));
    }
    // The lowest day-end balance of such an account before the change: a change or a removal of
    // an opening may not leave returns above what was taken or lent (money agent, plan 17).
    let floor_before = if is_returnable_control(&facts.control_code) {
        Some(floor::floor_of(tx, facts.id)?)
    } else {
        None
    };
    // A new opening entry is a new posting, so the account must be active (PRD FR-10.2). Taking
    // an opening away (no amount wanted) only reverses the old entry, which is always allowed.
    if wanted.has_amount() && !facts.is_active {
        return Err(ServiceError::reject_field(
            ErrorCode::ErrAccountInactive,
            amount_field,
        ));
    }
    if have.has_amount() {
        let old_entry = facts
            .opening_entry_id
            .ok_or_else(|| ServiceError::Data("an opening without its entry".to_owned()))?;
        ledger::reverse(ctx, tx, user_id, old_entry, "opening changed").map_err(on_opening_date)?;
    }
    if wanted.has_amount() && wanted.date.is_none() {
        // opening_request refuses this first; reaching here would clear an opening by mistake.
        return Err(ServiceError::Data(
            "an opening with an amount has no date".to_owned(),
        ));
    }
    match wanted.date.filter(|_| wanted.has_amount()) {
        Some(date) => {
            let (side, amount) = if wanted.dr == Paisa::ZERO {
                (Side::Cr, wanted.cr)
            } else {
                (Side::Dr, wanted.dr)
            };
            let entry = build_opening(
                AccountId::new(facts.id),
                side,
                amount,
                wanted.is_advance,
                AccountId::new(difference_id(tx)?),
            )
            .map_err(ledger::ledger_error)?;
            let posted = ledger::post(
                ctx,
                tx,
                user_id,
                &PostingEvent {
                    date,
                    posting_type: PostingType::Opening,
                    source_type: SourceType::Account,
                    source_id: facts.id,
                    narration: None,
                    reverses: None,
                    entry,
                },
            )
            .map_err(on_opening_date)?;
            repo::set_opening(
                tx,
                facts.id,
                Some(&date.to_iso()),
                wanted.dr.as_paisa(),
                wanted.cr.as_paisa(),
                wanted.is_advance,
                Some(posted.entry_id),
                user_id,
                now,
            )?;
        }
        None => repo::set_opening(tx, facts.id, None, 0, 0, false, None, user_id, now)?,
    }
    check_after(tx, floor_before, amount_field)?;
    // An opening is a line of the buyer's key (a bill or a prepayment) or a seller's advance, so
    // the matching rows are made right again: they stay the canonical ones (plan 13). A one-time
    // person has no opening (PRD FR-10.9).
    if !facts.is_pukhta_khata {
        if facts.control_code == BUYER_CONTROL {
            matching::rematch(tx, user_id, now, facts.id, None)?;
        } else if facts.control_code == SELLER_CONTROL {
            matching::rematch_advances(tx, user_id, now, facts.id)?;
        }
    }
    Ok(true)
}

/// One line of the grid, checked.
struct PlannedLine {
    facts: Option<repo::AccountFacts>,
    account: Option<RecordRef>,
    request: Option<OpeningRequest>,
    errors: Vec<LineError>,
}

fn line_error(code: ErrorCode, field: String) -> LineError {
    LineError {
        code,
        field,
        values: std::collections::BTreeMap::new(),
    }
}

/// Checks every line of the grid against the database. Used by the preview and the save.
fn plan(
    window: (Option<BusinessDate>, BusinessDate),
    conn: &Connection,
    input: &OpeningsInput,
) -> Result<Vec<PlannedLine>, ServiceError> {
    let (closed, today) = window;
    let mut seen: Vec<i64> = Vec::new();
    let mut lines = Vec::with_capacity(input.lines.len());
    for (index, line) in input.lines.iter().enumerate() {
        let prefix = format!("lines[{index}].");
        let mut errors: Vec<LineError> = Vec::new();
        let facts = repo::facts(conn, line.account_id)?;
        let account = repo::get(conn, line.account_id)?.map(|row| RecordRef {
            id: row.id,
            code: row.code,
            name_ur: row.name_ur,
            name_en: row.name_en,
        });
        let mut request = None;
        match &facts {
            None => errors.push(line_error(
                ErrorCode::ErrNotFound,
                format!("{prefix}account_id"),
            )),
            Some(facts) => {
                if seen.contains(&facts.id) {
                    errors.push(line_error(
                        ErrorCode::ErrFormatInvalid,
                        format!("{prefix}account_id"),
                    ));
                }
                seen.push(facts.id);
                if facts.version != line.version {
                    errors.push(line_error(
                        ErrorCode::ErrVersionChanged,
                        format!("{prefix}version"),
                    ));
                }
                let have = current_opening(facts)?;
                match opening_request_in_force(
                    &prefix,
                    &line.opening_date,
                    &line.opening_dr,
                    &line.opening_cr,
                    line.opening_is_advance,
                    facts.control_code == SELLER_CONTROL,
                    (closed, today),
                    Some(&have),
                ) {
                    Ok(found) => {
                        if found.has_amount() && !opening_allowed(facts) {
                            errors.push(line_error(
                                ErrorCode::ErrOpeningNotAllowed,
                                format!("{prefix}opening_dr"),
                            ));
                        }
                        // The save refuses a new opening on an inactive account (the same
                        // rule as `apply`, so the preview says what the save will say, B5).
                        if found.has_amount() && !same_opening(&have, &found) && !facts.is_active {
                            errors.push(line_error(
                                ErrorCode::ErrAccountInactive,
                                format!("{prefix}opening_dr"),
                            ));
                        }
                        // A change reverses the old entry on its own date, which must be open
                        // too. Checked here, so the preview says what the save will say (B5).
                        if !same_opening(&have, &found)
                            && let Some(old_date) = have.date.filter(|_| have.has_amount())
                            && let Err(e) = validate::posting_date(
                                &format!("{prefix}opening_date"),
                                &format_ddmmyyyy(old_date),
                                closed,
                                today,
                            )
                        {
                            errors.push(LineError {
                                code: e.code,
                                field: e.field,
                                values: e.values,
                            });
                        }
                        // Money taken (0603) and money lent (0103): only a debit opening, and a
                        // change may not leave returns above it. Says what the save will say (B5).
                        if is_returnable_control(&facts.control_code)
                            && !same_opening(&have, &found)
                        {
                            if found.cr != Paisa::ZERO {
                                errors.push(line_error(
                                    ErrorCode::ErrOpeningNotAllowed,
                                    format!("{prefix}opening_cr"),
                                ));
                            } else {
                                let old_entry =
                                    match facts.opening_entry_id.filter(|_| have.has_amount()) {
                                        Some(id) => mandi_db::ledger::get_entry(conn, id)?,
                                        None => None,
                                    };
                                let day = found
                                    .date
                                    .filter(|_| found.has_amount())
                                    .map(|d| d.to_iso());
                                if let Some(e) = floor::what_if(
                                    conn,
                                    old_entry.as_ref().map(|entry| (facts.id, entry)),
                                    day.as_deref().map(|d| (facts.id, d, found.dr)),
                                    &format!("{prefix}opening_dr"),
                                )? {
                                    errors.push(LineError {
                                        code: e.code,
                                        field: e.field,
                                        values: e.values,
                                    });
                                }
                            }
                        }
                        request = Some(found);
                    }
                    Err(found) => {
                        errors.extend(found.into_iter().map(|e: FieldError| LineError {
                            code: e.code,
                            field: e.field,
                            values: e.values,
                        }));
                    }
                }
            }
        }
        lines.push(PlannedLine {
            facts,
            account,
            request: if errors.is_empty() { request } else { None },
            errors,
        });
    }
    Ok(lines)
}

/// The signed effect on ابتدائی فرق (debit minus credit, in paisa) of changing an account's
/// opening: an opening debit puts a credit on ابتدائی فرق, so its net moves by `cr − dr` of the
/// account's opening.
fn difference_delta(have: &OpeningRequest, wanted: &OpeningRequest) -> Result<Paisa, ServiceError> {
    let net = |o: &OpeningRequest| o.cr.checked_sub(o.dr).map_err(money_error);
    net(wanted)?.checked_sub(net(have)?).map_err(money_error)
}

fn difference_after(conn: &Connection, lines: &[PlannedLine]) -> Result<Balance, ServiceError> {
    let now = ledger::balance_in(conn, difference_id(conn)?, None, None)?;
    // Balance -> signed net (debit positive).
    let mut net = match now.side {
        mandi_domain::core::accounts::BalanceSide::Cr => {
            now.amount_paisa.checked_neg().map_err(money_error)?
        }
        _ => now.amount_paisa,
    };
    for line in lines {
        if let (Some(facts), Some(wanted)) = (&line.facts, &line.request) {
            let have = current_opening(facts)?;
            if !same_opening(&have, wanted) {
                net = net
                    .checked_add(difference_delta(&have, wanted)?)
                    .map_err(money_error)?;
            }
        }
    }
    Balance::from_net(net).map_err(money_error)
}

/// Checks the grid and says what ابتدائی فرق would be after it. Nothing is written.
///
/// # Errors
/// `ServiceError::Db`. Mistakes on lines are returned in the rows, not as an error.
pub fn openings_preview(
    ctx: &Context,
    input: &OpeningsInput,
) -> Result<OpeningsPreview, ServiceError> {
    // The day window is read first: the read connection is not re-entrant.
    let window = day::window(ctx)?;
    ctx.db.read(|conn| {
        let planned = plan(window, conn, input)?;
        let (mut total_dr, mut total_cr) = (Paisa::ZERO, Paisa::ZERO);
        let mut rows = Vec::with_capacity(planned.len());
        for line in &planned {
            let (dr, cr) = line
                .request
                .map_or((Paisa::ZERO, Paisa::ZERO), |r| (r.dr, r.cr));
            total_dr = total_dr.checked_add(dr).map_err(money_error)?;
            total_cr = total_cr.checked_add(cr).map_err(money_error)?;
            rows.push(OpeningPreviewRow {
                account: line.account.clone(),
                errors: line.errors.clone(),
                opening_dr_paisa: dr,
                opening_cr_paisa: cr,
            });
        }
        Ok(OpeningsPreview {
            rows,
            total_dr_paisa: total_dr,
            total_cr_paisa: total_cr,
            opening_difference_after: difference_after(conn, &planned)?,
        })
    })
}

/// Saves the grid in one transaction: all lines or none. Each line that changed reverses its old
/// opening entry and posts the new one; a line that did not change writes nothing.
///
/// # Errors
/// Every mistake of every line, with the path `lines[i].<field>`; nothing is written then.
pub fn openings_save(
    ctx: &Context,
    actor: &Actor,
    input: &OpeningsInput,
) -> Result<OpeningsSaved, ServiceError> {
    let ids: Vec<i64> = input.lines.iter().map(|l| l.account_id).collect();
    let window = day::window(ctx)?;
    ctx.db.write(|tx| {
        let planned = plan(window, tx, input)?;
        let problems: Vec<Rejection> = planned
            .iter()
            .flat_map(|line| line.errors.iter())
            .map(|e| Rejection {
                code: e.code,
                field: Some(e.field.clone()),
                values: e.values.clone(),
            })
            .collect();
        if !problems.is_empty() {
            return Err(ServiceError::Rejected(problems));
        }
        let now = ctx.now_text();
        for (index, line) in planned.iter().enumerate() {
            let (Some(facts), Some(wanted)) = (&line.facts, &line.request) else {
                continue;
            };
            let have = current_opening(facts)?;
            let field = format!("lines[{index}].opening_dr");
            if apply(ctx, tx, actor.user_id, &now, facts, wanted, &field)
                .map_err(|e| on_line(e, index))?
            {
                ctx.db.auditor().write(
                    tx,
                    &Event::new(Some(actor.user_id), "edit", "account")
                        .entity_id(facts.id)
                        .old(opening_json(&have))
                        .new_value(opening_json(wanted))
                        .into_entry(),
                )?;
            }
        }
        Ok::<_, ServiceError>(())
    })?;
    for id in &ids {
        search::refresh_account(ctx, *id);
    }
    ctx.db.read(|conn| {
        let mut rows = Vec::with_capacity(ids.len());
        for id in &ids {
            rows.push(super::one_with_balance(conn, *id)?);
        }
        let diff = ledger::balance_in(conn, difference_id(conn)?, None, None)?;
        Ok(OpeningsSaved {
            rows,
            opening_difference: diff,
        })
    })
}

fn opening_json(o: &OpeningRequest) -> serde_json::Value {
    json!({
        "opening": {
            "date": o.date.map(|d| d.to_iso()),
            "dr_paisa": o.dr.as_paisa(),
            "cr_paisa": o.cr.as_paisa(),
            "is_advance": o.is_advance,
        }
    })
}
