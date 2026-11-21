//! Opening balances (PRD FR-10.9): one account at a time from the form, many at once from the
//! grid. The preview and the save use the same plan (Code Standards B5).
//!
//! An opening is a posted entry against ابتدائی فرق (06019999), or, for a partner's opening capital
//! that is in the shop's cash (PRD 17.1 v1.43), against روکڑ. Changing it reverses the old entry (on
//! its own date, which must still be open) and posts a new one on the same other side; an opening
//! that did not change writes nothing.

use mandi_db::accounts as repo;
use mandi_db::rusqlite::Connection;
use mandi_domain::core::accounts::{
    BUYER_CONTROL, Balance, LineError, OPENING_DIFFERENCE_CODE, OpeningDay, OpeningPreviewRow,
    OpeningRequest, OpeningsInput, OpeningsPreview, OpeningsSaved, RecordRef, SELLER_CONTROL,
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
use crate::cash::{CASH_ACCOUNT_CODE, cash_after_entry, require_confirmed};
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

/// Gives a refusal that names no field the field of the amount that caused it.
fn on_field(error: ServiceError, field: &str) -> ServiceError {
    match error {
        ServiceError::Rejected(items) => ServiceError::Rejected(
            items
                .into_iter()
                .map(|mut item| {
                    item.field.get_or_insert_with(|| field.to_owned());
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
// Eight plain arguments: the transaction, who and when, the account, the wanted opening, the field
// to blame and the days it may be dated on. A struct would only move them into a type used by
// three callers.
#[allow(clippy::too_many_arguments)]
pub(crate) fn apply(
    ctx: &Context,
    tx: &Connection,
    user_id: i64,
    now: &str,
    facts: &repo::AccountFacts,
    wanted: &OpeningRequest,
    amount_field: &str,
    day: OpeningDay,
    confirmed: bool,
) -> Result<bool, ServiceError> {
    apply_with(
        ctx,
        tx,
        user_id,
        now,
        facts,
        wanted,
        amount_field,
        day,
        OpeningCounter::AsBefore,
        confirmed,
    )
}

/// The account on the other side of an opening entry (PRD FR-10.9, PRD 17.1 v1.43).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OpeningCounter {
    /// ابتدائی فرق: the money is already counted in other openings. The only way before v1.43.
    Difference,
    /// روکڑ: the money is in the shop's cash (a partner's opening capital put into cash, plan 78 E).
    Cash,
    /// The same as the opening in force: an opening that came into cash stays in cash when it is
    /// changed, and one against ابتدائی فرق stays against it. A first opening is against
    /// ابتدائی فرق.
    AsBefore,
}

/// True when the opening entry in force of an account has a line on روکڑ, so its other side is the
/// shop's cash (a partner's opening capital that came into cash, plan 78 E). The opening typed on
/// the cash account itself is not "in cash": it is cash by its own account.
fn opening_is_in_cash(
    tx: &Connection,
    facts: &repo::AccountFacts,
    cash_id: i64,
) -> Result<bool, ServiceError> {
    let Some(entry_id) = facts.opening_entry_id.filter(|_| facts.id != cash_id) else {
        return Ok(false);
    };
    let found: bool = tx.query_row(
        "SELECT EXISTS (SELECT 1 FROM journal_lines WHERE entry_id = ?1 AND account_id = ?2)",
        [entry_id, cash_id],
        |r| r.get(0),
    )?;
    Ok(found)
}

/// True when the same opening is asked to go into روکڑ (`OpeningCounter::Cash`), the account has an
/// opening in force and that opening does not have روکڑ on its other side yet. The cash account's own
/// opening never moves (it is cash by its own account).
fn moves_to_cash(
    tx: &Connection,
    facts: &repo::AccountFacts,
    have: &OpeningRequest,
    counter: OpeningCounter,
) -> Result<bool, ServiceError> {
    if counter != OpeningCounter::Cash || !have.has_amount() {
        return Ok(false);
    }
    let cash_id = repo::id_by_code(tx, CASH_ACCOUNT_CODE)?
        .ok_or_else(|| ServiceError::Data("the cash account is missing".to_owned()))?;
    Ok(facts.id != cash_id && !opening_is_in_cash(tx, facts, cash_id)?)
}

/// What cash gains from an opening that is in cash: a credit opening on the account puts the same
/// amount into روکڑ (Dr روکڑ), a debit opening takes it out (Cr روکڑ).
fn cash_effect(opening: &OpeningRequest) -> Result<Paisa, ServiceError> {
    opening.cr.checked_sub(opening.dr).map_err(money_error)
}

/// What changing an opening does to cash in hand: the new entry's effect less the old entry's, each
/// only when that opening is in cash. The one formula of the save and of the preview (Code Standards
/// B5). Negative means the change takes cash out (PRD FR-6.7).
fn opening_cash_change(
    have: &OpeningRequest,
    was_in_cash: bool,
    wanted: &OpeningRequest,
    will_be_in_cash: bool,
) -> Result<Paisa, ServiceError> {
    let old = if was_in_cash {
        cash_effect(have)?
    } else {
        Paisa::ZERO
    };
    let new = if will_be_in_cash {
        cash_effect(wanted)?
    } else {
        Paisa::ZERO
    };
    new.checked_sub(old).map_err(money_error)
}

/// [`apply`] with the account on the other side chosen (plan 78 E).
///
/// An opening that is in cash and is changed or taken away changes cash: when the change would
/// leave cash below zero it is refused with `ERR_CASH_BELOW_ZERO_NOT_CONFIRMED` (value
/// `cash_after_paisa`, PRD FR-6.7) unless `confirmed` is true, the answer "ہاں، محفوظ کریں" of the
/// cash box (plan 80 gate R4-4).
///
/// # Errors
/// As [`apply`]; `ERR_CASH_BELOW_ZERO_NOT_CONFIRMED` (value `cash_after_paisa`).
#[allow(clippy::too_many_arguments)]
pub(crate) fn apply_with(
    ctx: &Context,
    tx: &Connection,
    user_id: i64,
    now: &str,
    facts: &repo::AccountFacts,
    wanted: &OpeningRequest,
    amount_field: &str,
    day: OpeningDay,
    counter: OpeningCounter,
    confirmed: bool,
) -> Result<bool, ServiceError> {
    let have = current_opening(facts)?;
    // The same opening is not written again, except when it is asked to move into روکڑ and now stands
    // against ابتدائی فرق (plan 87, PRD FR-16.14 v1.50): then the old entry is reversed and the same
    // amount is posted again on the same date with روکڑ on the other side.
    if same_opening(&have, wanted) && !moves_to_cash(tx, facts, &have, counter)? {
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
    // An opening advance is split between the partners by their shares (PRD FR-17.13). With
    // partners but no share set at all it would have no partner parts and the check of FR-17.9
    // would miss it, so it is refused until the shares are set (plan 80 gate Q-R1-3, R1-3). With no
    // partner yet it is allowed (PRD 4.3).
    if wanted.has_amount() && wanted.is_advance && mandi_db::partners::partners_without_shares(tx)?
    {
        return Err(ServiceError::reject_field(
            ErrorCode::ErrSharesNot100,
            amount_field,
        ));
    }
    // Which account takes the other side, and what the change does to cash (plan 78 E).
    let cash_id = repo::id_by_code(tx, CASH_ACCOUNT_CODE)?
        .ok_or_else(|| ServiceError::Data("the cash account is missing".to_owned()))?;
    // The opening typed on the cash account itself is cash by its own account, never "in cash" as
    // the other side, and it keeps the rules it always had.
    let was_in_cash = opening_is_in_cash(tx, facts, cash_id)?;
    let in_cash = facts.id != cash_id
        && match counter {
            OpeningCounter::Difference => false,
            OpeningCounter::Cash => true,
            OpeningCounter::AsBefore => was_in_cash,
        };
    let counter_id = if in_cash { cash_id } else { difference_id(tx)? };
    // PRD FR-6.7: changing an opening that is in cash takes cash out (the old entry goes, the new
    // one comes). When cash would end below zero the user is asked first: the refusal carries the
    // cash after the change, and the save goes through when the form is sent again confirmed.
    let cash_change = opening_cash_change(&have, was_in_cash, wanted, in_cash)?;
    if cash_change.is_negative() {
        let check = cash_after_entry(tx, cash_change)?;
        require_confirmed(&check, confirmed).map_err(|e| on_field(e, amount_field))?;
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
                AccountId::new(counter_id),
            )
            .map_err(ledger::ledger_error)?;
            let event = PostingEvent {
                date,
                posting_type: PostingType::Opening,
                source_type: SourceType::Account,
                source_id: facts.id,
                narration: None,
                reverses: None,
                entry,
            };
            // A new account is dated on any day up to today, a closed one too (PRD FR-10.11); it
            // has no old entry (`have` is empty), so no closed entry is touched.
            let new_account = day == OpeningDay::NewAccount && !have.has_amount();
            let posted = if new_account {
                ledger::post_new_opening(ctx, tx, user_id, &event)
            } else {
                ledger::post(ctx, tx, user_id, &event)
            }
            .map_err(on_opening_date)?;
            if new_account {
                audit_closed_day_opening(ctx, tx, user_id, facts.id, date, wanted)?;
            }
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
    if !facts.is_chukta_khata {
        if facts.control_code == BUYER_CONTROL {
            matching::rematch(tx, user_id, now, facts.id, None)?;
        } else if facts.control_code == SELLER_CONTROL {
            matching::rematch_advances(tx, user_id, now, facts.id)?;
        }
    }
    Ok(true)
}

/// The audit row of an opening dated on a closed day (S7): who, which account, which day and how
/// much. An opening on an open day needs none: it is an ordinary posting.
fn audit_closed_day_opening(
    ctx: &Context,
    tx: &Connection,
    user_id: i64,
    account_id: i64,
    date: BusinessDate,
    wanted: &OpeningRequest,
) -> Result<(), ServiceError> {
    let (closed_up_to, _) = day::window(ctx)?;
    if closed_up_to.is_none_or(|closed| date > closed) {
        return Ok(());
    }
    ctx.db.auditor().write(
        tx,
        &Event::new(Some(user_id), "opening_on_closed_day", "account")
            .entity_id(account_id)
            .new_value(json!({
                "date": date.to_iso(),
                "dr_paisa": wanted.dr.as_paisa(),
                "cr_paisa": wanted.cr.as_paisa(),
                "is_advance": wanted.is_advance,
            }))
            .reason(CLOSED_DAY_OPENING_REASON)
            .into_entry(),
    )?;
    Ok(())
}

/// The reason written with an opening on a closed day (plan 53 T1).
pub const CLOSED_DAY_OPENING_REASON: &str = "opening on a closed day";

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
    confirmed: bool,
) -> Result<Vec<PlannedLine>, ServiceError> {
    let (closed, today) = window;
    let cash_id = repo::id_by_code(conn, CASH_ACCOUNT_CODE)?
        .ok_or_else(|| ServiceError::Data("the cash account is missing".to_owned()))?;
    // Cash in hand while the lines are checked in order, as the save sees it (read only when a line
    // changes an opening that is in cash).
    let mut cash_running: Option<Paisa> = None;
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
            tag: row.tag,
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
                        // PRD FR-6.7 (plan 78 E): changing an opening that is in cash takes cash out;
                        // refused when cash would end below zero, as the save refuses it (B5).
                        if !same_opening(&have, &found) && opening_is_in_cash(conn, facts, cash_id)?
                        {
                            let change = opening_cash_change(&have, true, &found, true)?;
                            let before = match cash_running {
                                Some(cash) => cash,
                                None => crate::cash::cash_in_hand(conn)?,
                            };
                            let check =
                                mandi_domain::cash::check(before, change).map_err(money_error)?;
                            if check.needs_confirm && !confirmed {
                                errors.push(LineError {
                                    code: ErrorCode::ErrCashBelowZeroNotConfirmed,
                                    field: format!("{prefix}opening_dr"),
                                    values: std::collections::BTreeMap::from([(
                                        "cash_after_paisa".to_owned(),
                                        mandi_domain::core::error::ErrorValue::Number(
                                            check.cash_after_paisa.as_paisa(),
                                        ),
                                    )]),
                                });
                            }
                            cash_running = Some(check.cash_after_paisa);
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
    let cash_id = repo::id_by_code(conn, CASH_ACCOUNT_CODE)?
        .ok_or_else(|| ServiceError::Data("the cash account is missing".to_owned()))?;
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
            // An opening that is in cash stays in cash: it never moves ابتدائی فرق (plan 78 E).
            if !same_opening(&have, wanted) && !opening_is_in_cash(conn, facts, cash_id)? {
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
    openings_preview_with_confirm(ctx, input, false)
}

/// [`openings_preview`] with the answer of the cash box (PRD FR-6.7, plan 80 gate R4-4): with
/// `confirm_cash_below_zero` true a line that takes cash below 0 has no
/// `ERR_CASH_BELOW_ZERO_NOT_CONFIRMED`. The save uses the same plan (B5).
///
/// # Errors
/// As [`openings_preview`].
pub fn openings_preview_with_confirm(
    ctx: &Context,
    input: &OpeningsInput,
    confirm_cash_below_zero: bool,
) -> Result<OpeningsPreview, ServiceError> {
    // The day window is read first: the read connection is not re-entrant.
    let window = day::window(ctx)?;
    ctx.db.read(|conn| {
        let planned = plan(window, conn, input, confirm_cash_below_zero)?;
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
    openings_save_with_confirm(ctx, actor, input, false)
}

/// [`openings_save`] with the answer of the cash box (PRD FR-6.7, plan 80 gate R4-4).
///
/// # Errors
/// As [`openings_save`].
pub fn openings_save_with_confirm(
    ctx: &Context,
    actor: &Actor,
    input: &OpeningsInput,
    confirm_cash_below_zero: bool,
) -> Result<OpeningsSaved, ServiceError> {
    let ids: Vec<i64> = input.lines.iter().map(|l| l.account_id).collect();
    let window = day::window(ctx)?;
    ctx.db.write(|tx| {
        let planned = plan(window, tx, input, confirm_cash_below_zero)?;
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
            if apply(
                ctx,
                tx,
                actor.user_id,
                &now,
                facts,
                wanted,
                &field,
                OpeningDay::Open,
                confirm_cash_below_zero,
            )
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
