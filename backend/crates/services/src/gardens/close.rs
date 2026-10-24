//! Closing a garden and undoing the closing (PRD FR-12.38, C26, C11, E64, plan 17c).
//!
//! When the goods stop coming the Owner closes the garden. The balance B of its account (the money
//! from its lots sold in our shop and in other shops, less any negative credit) is split by the
//! garden's shares with `split_by_bps` (C26); a loss is split as a positive amount and every part is
//! made negative (C11). The garden's account ends at 0 and each partner's 0604 account takes his part.
//! A closing with nothing to move (B = 0) posts nothing and still closes (E64).
//!
//! The closing is stored as one `close` entry of the garden: `amount_paisa` is B without its sign (the
//! column cannot be negative) and the signed parts carry the sign, so Σ parts = B. The preview and the
//! save call the same [`plan`] (Code Standards B5). The undo reverses the closing entry and opens the
//! garden again; the cancelled closing keeps its parts (append-only) and every read of the parts looks
//! only at saved entries.

use mandi_db::gardens as repo;
use mandi_db::rusqlite::Connection;
use mandi_db::ventures::{self as vrepo, NewEntry};
use mandi_domain::core::dates::BusinessDate;
use mandi_domain::core::error::{ErrorCode, ErrorValue, FieldError};
use mandi_domain::core::gardens::{
    GardenCloseInput, GardenCloseNumbers, GardenClosePreview, GardenClosed, GardenReopenInput,
    GardenReopened,
};
use mandi_domain::core::ids::AccountId;
use mandi_domain::core::money::Paisa;
use mandi_domain::core::validate::{self, FieldErrors};
use mandi_domain::ledger::{BalancedEntry, PostingType, SourceType};
use mandi_domain::ventures::{CreditPart, close_entry, close_parts};
use serde_json::json;

use super::data;
use super::read::{Garden, Money, load, result_of, row_by_id};
use crate::audit::Event;
use crate::auth::Actor;
use crate::context::Context;
use crate::day;
use crate::error::{ServiceError, money_error};
use crate::ledger;
use crate::search;
use crate::ventures::{
    self, Settlement, credit_partners, credits_view, not_found, require_open_day, required_reason,
};

/// A closing after the checks.
struct Ready {
    garden: Garden,
    date: BusinessDate,
    balance: Paisa,
    parts: Vec<CreditPart>,
    entry: Option<BalancedEntry>,
    numbers: GardenCloseNumbers,
}

/// What [`plan`] found.
struct Plan {
    errors: Vec<FieldError>,
    earliest_date: Option<String>,
    ready: Option<Ready>,
}

impl Plan {
    fn preview(&self) -> GardenClosePreview {
        GardenClosePreview {
            numbers: self.ready.as_ref().map(|r| r.numbers.clone()),
            earliest_date: self.earliest_date.clone(),
            errors: self.errors.clone(),
            warnings: Vec::new(),
        }
    }
}

/// Checks and works out a closing. Nothing is written.
///
/// # Errors
/// `ServiceError::Db`, `ServiceError::Data` or `ERR_OVERFLOW` for a failure the user cannot fix. The
/// user's mistakes come back in [`Plan::errors`]; a garden that is not found or has changed is
/// refused at once.
fn plan(
    conn: &Connection,
    window: (Option<BusinessDate>, BusinessDate),
    input: &GardenCloseInput,
) -> Result<Plan, ServiceError> {
    let (closed_up_to, today) = window;
    let garden = load(conn, input.id)?.ok_or_else(|| not_found("id"))?;
    if garden.rec.version != input.version {
        return Err(ServiceError::reject_field(
            ErrorCode::ErrVersionChanged,
            "version",
        ));
    }
    let mut errors = FieldErrors::default();
    if !garden.is_open() {
        errors.push(FieldError::new(ErrorCode::ErrGardenClosed, "id"));
    }
    let date = errors.take(validate::posting_date(
        "date",
        &input.date,
        closed_up_to,
        today,
    ));
    let figures = repo::figures(conn, &garden.rec)?;
    let money = Money::of(&garden.rec, &figures)?;
    // The closing cannot be dated before the garden's latest entry or lot posting (plan 17c Q1).
    let earliest = money.last_date.clone();
    if let (Some(date), Some(last)) = (date, earliest.as_deref()) {
        let last_day = BusinessDate::parse_iso(last)
            .ok_or_else(|| data("the last day of a garden is not a date"))?;
        if date < last_day {
            errors.push(
                FieldError::new(ErrorCode::ErrCloseBeforeLastEntry, "date")
                    .with("last_date", ErrorValue::Text(last_day.to_iso())),
            );
        }
    }
    let found = errors.finish().err().unwrap_or_default();
    let Some(date) = date.filter(|_| found.is_empty()) else {
        return Ok(Plan {
            errors: found,
            earliest_date: earliest,
            ready: None,
        });
    };
    // B is what stands on the garden's account now. While the garden is open it is the money in
    // (the closing and its undo net to 0); the two must agree.
    let balance = money.account_balance;
    if balance != money.money_in {
        return Err(data(
            "the account of an open garden does not hold exactly its money in",
        ));
    }
    let partners = credit_partners(&garden.venture)?;
    let parts = close_parts(balance, &partners).map_err(|error| match error {
        mandi_domain::core::money::MoneyError::SharesNot100 => {
            data("the shares of a garden do not total 100%")
        }
        other => money_error(other),
    })?;
    let entry = close_entry(AccountId::new(garden.rec.account.id), balance, &parts)
        .map_err(ledger::ledger_error)?;
    let credits = credits_view(&garden.venture, &parts)?;
    // What each partner paid is read from the stored parts of the live payments; his result is his
    // closing part less that (derived, so the results add up to the garden's result exactly).
    let mut partner_results_view = Vec::with_capacity(parts.len());
    let paid = repo::paid_by_partner(conn, garden.rec.id)?;
    for part in &parts {
        let partner = garden
            .venture
            .partners
            .iter()
            .find(|p| p.partner_id == part.partner_id.get())
            .ok_or_else(|| data("a closing part names a partner who is not in the garden"))?;
        let (shop, pocket) = paid.get(&partner.partner_id).copied().unwrap_or((0, 0));
        partner_results_view.push(result_of(
            partner.partner_id,
            &partner.name_ur,
            &partner.name_en,
            partner.share_bps,
            Paisa::try_from_paisa(shop).map_err(money_error)?,
            Paisa::try_from_paisa(pocket).map_err(money_error)?,
            part.amount,
        )?);
    }
    let result = money
        .money_in
        .checked_sub(money.paid)
        .map_err(money_error)?;
    let numbers = GardenCloseNumbers {
        balance_paisa: balance,
        total_cost_paisa: money.paid,
        money_in_paisa: money.money_in,
        result_paisa: result,
        credits,
        partner_results: partner_results_view,
        posts_entry: entry.is_some(),
    };
    Ok(Plan {
        errors: found,
        earliest_date: earliest,
        ready: Some(Ready {
            garden,
            date,
            balance,
            parts,
            entry,
            numbers,
        }),
    })
}

/// Works out the closing for the confirm box. Nothing is saved.
pub(super) fn preview(
    ctx: &Context,
    input: &GardenCloseInput,
) -> Result<GardenClosePreview, ServiceError> {
    let window = day::window(ctx)?;
    ctx.db.read(|conn| Ok(plan(conn, window, input)?.preview()))
}

/// Closes a garden in one transaction: the closing entry with each partner's signed part, the journal
/// entry (the garden's account against the partners' 0604 accounts, if there is anything to move),
/// the garden's status and the audit row.
///
/// Example (E2E-40): a balance of 66,400 at 60 / 40 gives 39,840 / 26,560; the garden's account goes
/// to 0 and الف's total changes by +39,840 less the 33,000 he paid from the shop.
pub(super) fn close(
    ctx: &Context,
    actor: &Actor,
    input: &GardenCloseInput,
) -> Result<GardenClosed, ServiceError> {
    let window = day::window(ctx)?;
    let now = ctx.now_text();
    let (closed, account_id) = ctx.db.write(|tx| {
        let planned = plan(tx, window, input)?;
        if !planned.errors.is_empty() {
            return Err(ServiceError::from(planned.errors.clone()));
        }
        let preview = planned.preview();
        let ready = planned
            .ready
            .ok_or_else(|| data("a closing without errors has no numbers"))?;
        let user = actor.user_id;
        let date_iso = ready.date.to_iso();
        let magnitude = if ready.balance.is_negative() {
            ready.balance.checked_neg().map_err(money_error)?
        } else {
            ready.balance
        };
        let number = vrepo::next_entry_number(tx, "close")?;
        let entry_id = vrepo::insert_entry(
            tx,
            &NewEntry {
                venture_id: ready.garden.rec.id,
                kind: "close",
                entry_date: &date_iso,
                amount_paisa: magnitude.as_paisa(),
                reason: None,
                purchase_id: None,
                qty: None,
                shop_name: None,
                cost_paisa: 0,
            },
            number,
            user,
            &now,
        )?;
        // The parts are signed (a loss has negative parts); their source is `shop`: the money
        // comes from the garden's account, not from a pocket.
        ventures::write_credits(tx, user, &now, entry_id, &ready.parts)?;
        let journal_entry_id = ventures::post_settlement(
            ctx,
            tx,
            user,
            &now,
            &Settlement {
                venture_entry_id: entry_id,
                date: ready.date,
                entry: ready.entry.as_ref(),
                posting_type: PostingType::GardenClose,
                source_type: SourceType::Venture,
                source_id: entry_id,
            },
        )?;
        repo::set_closed(
            tx,
            ready.garden.rec.id,
            &date_iso,
            journal_entry_id,
            user,
            &now,
        )?;
        ctx.db.auditor().write(
            tx,
            &Event::new(Some(user), "close", "garden")
                .entity_id(ready.garden.rec.id)
                .old(json!({ "status": "open" }))
                .new_value(json!({
                    "status": "closed",
                    "closed_date": date_iso,
                    "close_entry_id": entry_id,
                    "journal_entry_id": journal_entry_id,
                    "balance_paisa": ready.balance.as_paisa(),
                    "parts": ready.parts.iter().map(|p| json!({
                        "partner_id": p.partner_id.get(),
                        "share_bps": p.share.get(),
                        "amount_paisa": p.amount.as_paisa(),
                    })).collect::<Vec<_>>(),
                }))
                .into_entry(),
        )?;
        let row = row_by_id(tx, ready.garden.rec.id)?;
        Ok((
            GardenClosed {
                row,
                close_entry_id: entry_id,
                journal_entry_id,
                preview,
            },
            ready.garden.rec.account.id,
        ))
    })?;
    search::refresh_account(ctx, account_id);
    Ok(closed)
}

/// Undoes the closing of a garden by reversal while the closing day is open (PRD FR-12.38): the
/// closing entry is cancelled, its journal entry reversed, and the garden takes entries and lots
/// again. A later closing is a new entry with a new number.
///
/// # Errors
/// `ERR_NOT_FOUND`, `ERR_GARDEN_NOT_CLOSED` and `ERR_DATE_CLOSED` on `id`; `ERR_VERSION_CHANGED` on
/// `version`; `ERR_REASON_REQUIRED` on `reason`. Nothing is written when any of them is returned.
pub(super) fn reopen(
    ctx: &Context,
    actor: &Actor,
    input: &GardenReopenInput,
) -> Result<GardenReopened, ServiceError> {
    let reason = required_reason(&input.reason)?;
    let now = ctx.now_text();
    let (reopened, account_id) = ctx.db.write(|tx| {
        let garden = load(tx, input.id)?.ok_or_else(|| not_found("id"))?;
        if garden.is_open() {
            return Err(ServiceError::reject_field(
                ErrorCode::ErrGardenNotClosed,
                "id",
            ));
        }
        if garden.rec.version != input.version {
            return Err(ServiceError::reject_field(
                ErrorCode::ErrVersionChanged,
                "version",
            ));
        }
        let close_id = repo::saved_close_entry(tx, garden.rec.id)?
            .ok_or_else(|| data("a closed garden has no saved closing"))?;
        let entry = vrepo::get_entry(tx, close_id)?
            .ok_or_else(|| data("the closing of a garden cannot be read"))?;
        let date = BusinessDate::parse_iso(&entry.entry_date)
            .ok_or_else(|| data("the date of a closing is not a date"))?;
        require_open_day(ctx, date, "id")?;
        let user = actor.user_id;
        let reversal_entry_id = match entry.journal_entry_id {
            Some(journal_id) => Some(ledger::reverse(ctx, tx, user, journal_id, &reason)?.entry_id),
            None => None,
        };
        vrepo::mark_entry_cancelled(tx, entry.id, &reason, user, &now)?;
        repo::set_open(tx, garden.rec.id, user, &now)?;
        ctx.db.auditor().write(
            tx,
            &Event::new(Some(user), "reopen", "garden")
                .entity_id(garden.rec.id)
                .old(json!({ "status": "closed", "close_entry_id": entry.id }))
                .new_value(json!({
                    "status": "open",
                    "reversal_entry_id": reversal_entry_id,
                }))
                .reason(&reason)
                .into_entry(),
        )?;
        let row = row_by_id(tx, garden.rec.id)?;
        Ok((
            GardenReopened {
                row,
                reversal_entry_id,
            },
            garden.rec.account.id,
        ))
    })?;
    search::refresh_account(ctx, account_id);
    Ok(reopened)
}
