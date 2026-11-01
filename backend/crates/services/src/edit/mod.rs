//! Edit everywhere (PRD FR-7.70, FR-7.71, plan 44): the edit of every kind of saved entry, with the change
//! list and the effect a preview shows before the save.
//!
//! One model for every entry (PRD FR-7.70):
//! - **The date never changes** (`ERR_EDIT_DATE_FIXED`). A wrong date is a cancel and a new entry.
//! - **A reason** is needed when the entry is not of today (`ERR_REASON_REQUIRED` on `reason`), and
//!   optional for an entry of today (PRD A78).
//! - **An edit is a reversal plus a new posting** on the same day, in one transaction (PRD FR-12.19,
//!   FR-6.3), written to the audit log with the old and the new values.
//! - **The preview is a rehearsal of the save** ([`rehearse`]): the real edit function runs in a
//!   transaction that is rolled back, so the change list and the effect are exactly what the save
//!   will do (Code Standards B5).
//!
//! Plan 44 T2a: every kind of entry has its review and its save. The edits of a trip or garden entry
//! (`entries`) change rows the triggers keep read-only, inside the edit window of that one entry
//! (migration 12, `mandi_db::edit_windows`); the window is opened and closed inside the one
//! transaction, so a refused or failed save leaves nothing.
//!
//! Example (PRD E2E-45): بکری 1 is edited, اکرم's rate 600 → 610. The review has the changes of the
//! line (rate, commission, total) and the effect: اکرم 12,880 → 13,092 بنام.

mod bikri;
mod diff;
pub(crate) mod entries;
mod rehearse;

pub(crate) use rehearse::{Facts, Rehearsed, Scope, rehearse};

use mandi_domain::core::dates::BusinessDate;
use mandi_domain::core::edit::{EditChange, EditEffect, EditReview};
use mandi_domain::core::error::{ErrorValue, FieldError};
use mandi_domain::core::validate;

/// The error of an edit that tries to change the date of the entry (PRD FR-7.70). The value
/// `entry_date` (`YYYY-MM-DD`) is the date the entry keeps.
pub(crate) fn date_fixed(field: &str, keeps: BusinessDate) -> ServiceError {
    ServiceError::from(date_fixed_error(field, keeps))
}

/// The same as a field error, for the lists of errors of a بکری.
pub(crate) fn date_fixed_error(field: &str, keeps: BusinessDate) -> FieldError {
    FieldError::new(ErrorCode::ErrEditDateFixed, field)
        .with("entry_date", ErrorValue::Text(keeps.to_iso()))
}

/// The reason of an edit (PRD FR-7.70, A78): optional for an entry of today, required for any other
/// day. Returns the trimmed text, or `None` when none was given.
///
/// # Errors
/// The text codes on `reason`; `ERR_REASON_REQUIRED` on `reason`.
pub(crate) fn reason_for(
    today: BusinessDate,
    entry_date: BusinessDate,
    typed: &str,
) -> Result<Option<String>, ServiceError> {
    reason_for_at("reason", today, entry_date, typed)
}

/// [`reason_for`] with the field the reason is typed in. A trip cost and a garden payment have a reason
/// of their own (why the money was paid), so the reason of their EDIT is `edit_reason` and the two
/// cannot be mixed up on the screen.
///
/// # Errors
/// The text codes on `field`; `ERR_REASON_REQUIRED` on `field`.
pub(crate) fn reason_for_at(
    field: &str,
    today: BusinessDate,
    entry_date: BusinessDate,
    typed: &str,
) -> Result<Option<String>, ServiceError> {
    let text = validate::free_text(field, typed).map_err(ServiceError::from)?;
    if text.is_empty() {
        if entry_date != today {
            return Err(ServiceError::reject_field(
                ErrorCode::ErrReasonRequired,
                field,
            ));
        }
        return Ok(None);
    }
    Ok(Some(text))
}

fn field_error(rejection: crate::error::Rejection) -> FieldError {
    FieldError {
        code: rejection.code,
        field: rejection.field.unwrap_or_default(),
        values: rejection.values,
    }
}

/// The effect the screen shows: the parties, the cash and the partners (which stay out while the
/// partner section is locked, PRD FR-17.11).
fn effect_of(ctx: &Context, facts: Facts) -> Result<EditEffect, ServiceError> {
    let locked = crate::partner_code::is_locked(ctx)?;
    Ok(EditEffect {
        parties: facts.parties,
        cash: facts.cash,
        partners: if locked { Vec::new() } else { facts.partners },
        partners_locked: locked,
    })
}

/// Turns the outcome of a rehearsal into the answer of a preview. A refusal of the edit is not an
/// error of the call: its mistakes come back in `errors`, each at its field, with a review that
/// holds only the date and the two flags.
///
/// # Errors
/// `ServiceError::Db` and `ServiceError::Data`; a refusal that is not about the user's entry.
pub(crate) fn preview_of<R>(
    ctx: &Context,
    entry_date: BusinessDate,
    outcome: Result<Rehearsed<R>, ServiceError>,
    changes_of: impl Fn(&R, &R) -> Vec<EditChange>,
) -> Result<EditPreview, ServiceError> {
    let (closed_up_to, today) = day::window(ctx)?;
    let reason_required = entry_date != today;
    let needs_reopen = closed_up_to.is_some_and(|closed| entry_date <= closed);
    match outcome {
        Ok(done) => Ok(EditPreview {
            errors: Vec::new(),
            review: EditReview {
                entry_date: entry_date.to_iso(),
                changes: changes_of(&done.before, &done.after),
                effect: effect_of(ctx, done.facts)?,
                reason_required,
                needs_reopen,
            },
        }),
        Err(ServiceError::Rejected(items)) if !items.is_empty() => Ok(EditPreview {
            errors: items.into_iter().map(field_error).collect(),
            review: EditReview {
                entry_date: entry_date.to_iso(),
                changes: Vec::new(),
                effect: effect_of(ctx, Facts::default())?,
                reason_required,
                needs_reopen,
            },
        }),
        Err(other) => Err(other),
    }
}

use mandi_db::rusqlite::OptionalExtension;
use mandi_db::vouchers as voucher_repo;
use mandi_domain::core::bikri::{EditPreviewInput, EditSaveInput};
use mandi_domain::core::edit::{
    EditEntity, EditPreview, EntryEdited, GardenArrivalEditInput, GardenEntryEditInput,
    GardenHeaderEditInput, PurchaseEditInput, TripCostEditInput, TripHeaderEditInput,
    TripOtherSaleEditInput,
};
use mandi_domain::core::error::ErrorCode;
use mandi_domain::core::fruit::FruitEditInput;
use mandi_domain::core::vouchers::VoucherEditInput;

use crate::auth::Actor;
use crate::context::Context;
use crate::day;
use crate::error::ServiceError;

/// The review of an edit of a بکری posting: what changes and what it does, from a rehearsal of
/// `bikri::edit_save` (PRD FR-7.70). The same input as `bikri_edit_preview`.
///
/// # Errors
/// `ERR_NOT_FOUND` on `entry_id`. The mistakes of the edit come back inside the preview.
pub fn bikri_review(ctx: &Context, input: &EditPreviewInput) -> Result<EditPreview, ServiceError> {
    let actor = crate::auth::require(ctx)?;
    let window = day::window(ctx)?;
    let entry_id = input.entry_id;
    // The posting and its بکری: the posting's day is the day of the edit.
    let (bikri_id, posting_date, version): (i64, String, i64) = ctx.db.read(|conn| {
        conn.query_row(
            "SELECT e.source_id, e.entry_date, b.version FROM journal_entries e
             JOIN bikri_entries b ON b.id = e.source_id
             WHERE e.id = ?1 AND e.source_type = 'bikri'",
            [entry_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()?
        .ok_or_else(|| ServiceError::reject_field(ErrorCode::ErrNotFound, "entry_id"))
    })?;
    let entry_date = BusinessDate::parse_iso(&posting_date)
        .ok_or_else(|| ServiceError::Data("the date of a posting is not a date".to_owned()))?;
    let rule_set = crate::rules::rule_set(ctx)?;
    let switches = crate::bikri::edit::switches_of(ctx)?;
    // The reason and the cash question are the screen's: the review shows them
    // (`reason_required`, `effect.cash.needs_confirm`).
    let asked = EditSaveInput {
        entry_id,
        version,
        draft: input.draft.clone(),
        reason: Some("-".to_owned()),
        confirm_cash_below_zero: true,
    };
    let outcome = rehearse(
        ctx,
        Scope {
            source_type: "bikri",
            source_id: bikri_id,
            posting_id: entry_id,
        },
        |conn| {
            let detail = mandi_db::bikri::saved::get(conn, bikri_id)?
                .ok_or_else(|| ServiceError::reject_field(ErrorCode::ErrNotFound, "entry_id"))?;
            let mut statement = conn.prepare(
                "SELECT id, replaced_by_sale_id FROM bikri_sales
                 WHERE bikri_id = ?1 AND replaced_by_sale_id IS NOT NULL",
            )?;
            let replaced_by = statement
                .query_map([bikri_id], |r| {
                    Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?))
                })?
                .collect::<Result<_, _>>()?;
            Ok(bikri::Snapshot {
                detail,
                replaced_by,
            })
        },
        |tx| crate::bikri::edit::edit_in(ctx, &actor, tx, window, &rule_set, switches, &asked),
    );
    preview_of(ctx, entry_date, outcome, bikri::changes)
}

/// The review of an edit of a voucher of any type: what changes and what it does, from a rehearsal of
/// `vouchers::edit` (PRD FR-7.70).
///
/// # Errors
/// `ERR_NOT_FOUND` on `id`. The mistakes of the edit come back inside the preview.
pub fn voucher_review(
    ctx: &Context,
    input: &VoucherEditInput,
) -> Result<EditPreview, ServiceError> {
    let actor = crate::auth::require(ctx)?;
    let window = day::window(ctx)?;
    let id = input.id;
    let old = ctx.db.read(|conn| {
        voucher_repo::get(conn, id)?
            .ok_or_else(|| ServiceError::reject_field(ErrorCode::ErrNotFound, "id"))
    })?;
    let entry_date = BusinessDate::parse_iso(&old.entry_date)
        .ok_or_else(|| ServiceError::Data("the date of a voucher is not a date".to_owned()))?;
    // The cash question and the reason are the screen's: the review shows them
    // (`effect.cash.needs_confirm`, `reason_required`).
    let mut asked = input.clone();
    asked.confirm_cash_below_zero = true;
    if asked.reason.trim().is_empty() {
        asked.reason = "-".to_owned();
    }
    let outcome = rehearse(
        ctx,
        Scope {
            source_type: "voucher",
            source_id: id,
            posting_id: old
                .entry_id
                .ok_or_else(|| ServiceError::Data("a saved voucher has no posting".to_owned()))?,
        },
        |conn| crate::vouchers::edit::row_value(conn, id),
        |tx| crate::vouchers::edit::edit_in(ctx, &actor, tx, window, &asked),
    );
    preview_of(ctx, entry_date, outcome, diff::changes_of_rows)
}

/// The review of an edit of fruit taken, from a rehearsal of `fruit::edit` (PRD FR-7.70).
///
/// # Errors
/// `ERR_NOT_FOUND` on `id`. The mistakes of the edit come back inside the preview.
pub fn fruit_review(ctx: &Context, input: &FruitEditInput) -> Result<EditPreview, ServiceError> {
    let actor = crate::auth::require(ctx)?;
    let window = day::window(ctx)?;
    let id = input.id;
    let old = ctx.db.read(|conn| {
        mandi_db::fruit::get(conn, id)?
            .ok_or_else(|| ServiceError::reject_field(ErrorCode::ErrNotFound, "id"))
    })?;
    let entry_date = BusinessDate::parse_iso(&old.entry_date)
        .ok_or_else(|| ServiceError::Data("the date of a fruit entry is not a date".to_owned()))?;
    let mut asked = input.clone();
    if asked.reason.trim().is_empty() {
        asked.reason = "-".to_owned();
    }
    let outcome = rehearse(
        ctx,
        Scope {
            source_type: "fruit",
            source_id: id,
            posting_id: old.current_entry_id.ok_or_else(|| {
                ServiceError::Data("a saved fruit entry has no posting".to_owned())
            })?,
        },
        |conn| crate::fruit::row_value(conn, id),
        |tx| crate::fruit::act::edit_in(ctx, &actor, tx, window, &asked),
    );
    preview_of(ctx, entry_date, outcome, diff::changes_of_rows)
}

/// The review of an edit of a trip purchase: what changes and what it does, from a rehearsal of
/// the real edit (PRD FR-7.70).
///
/// # Errors
/// `ERR_NOT_FOUND` on `id`. The mistakes of the edit come back inside the preview.
pub fn purchase_preview(
    ctx: &Context,
    input: &PurchaseEditInput,
) -> Result<EditPreview, ServiceError> {
    let actor = crate::auth::require(ctx)?;
    let window = day::window(ctx)?;
    let id = input.id;
    let old = ctx.db.read(|conn| {
        mandi_db::purchases::get(conn, id)?
            .ok_or_else(|| ServiceError::reject_field(ErrorCode::ErrNotFound, "id"))
    })?;
    let entry_date = BusinessDate::parse_iso(&old.entry_date)
        .ok_or_else(|| ServiceError::Data("the date of a purchase is not a date".to_owned()))?;
    // The cash question and the reason are the screen's: the review shows them.
    let mut asked = input.clone();
    asked.purchase.confirm_cash_below_zero = true;
    if asked.reason.trim().is_empty() {
        asked.reason = "-".to_owned();
    }
    entries::review(
        ctx,
        Scope {
            source_type: "purchase",
            source_id: id,
            posting_id: old.entry_id.unwrap_or(0),
        },
        entry_date,
        |conn| crate::purchases::edit::row_value(conn, id),
        |tx| crate::purchases::edit::edit_in(ctx, &actor, tx, window, &asked),
    )
}

/// Saves the edit of a trip purchase: the old posting is reversed and the new one is made on the
/// same day, in one transaction (PRD FR-12.19, FR-6.3).
///
/// # Errors
/// The mistakes of [`purchase_preview`], each at its field; `ERR_CASH_BELOW_ZERO_NOT_CONFIRMED`.
pub fn purchase_save(
    ctx: &Context,
    actor: &Actor,
    input: &PurchaseEditInput,
) -> Result<EntryEdited, ServiceError> {
    let preview = purchase_preview(ctx, input)?;
    let window = day::window(ctx)?;
    entries::save(ctx, EditEntity::Purchase, preview, |tx| {
        crate::purchases::edit::edit_in(ctx, actor, tx, window, input)
    })
}

/// The review of an edit of a trip cost: what changes and what it does, from a rehearsal of the real
/// edit (PRD FR-7.70).
///
/// # Errors
/// `ERR_NOT_FOUND` on `id`. The mistakes of the edit come back inside the preview.
pub fn trip_cost_preview(
    ctx: &Context,
    input: &TripCostEditInput,
) -> Result<EditPreview, ServiceError> {
    let actor = crate::auth::require(ctx)?;
    let window = day::window(ctx)?;
    let id = input.id;
    let old = ctx.db.read(|conn| {
        mandi_db::ventures::get_entry(conn, id)?
            .filter(|e| e.kind == "cost")
            .ok_or_else(|| ServiceError::reject_field(ErrorCode::ErrNotFound, "id"))
    })?;
    let entry_date = BusinessDate::parse_iso(&old.entry_date)
        .ok_or_else(|| ServiceError::Data("the date of a trip cost is not a date".to_owned()))?;
    let mut asked = input.clone();
    asked.cost.confirm_cash_below_zero = true;
    if asked.reason.trim().is_empty() {
        asked.reason = "-".to_owned();
    }
    entries::review(
        ctx,
        Scope {
            source_type: "venture",
            source_id: id,
            posting_id: old.journal_entry_id.unwrap_or(0),
        },
        entry_date,
        |conn| crate::trips::cost_edit::row_value(conn, id),
        |tx| crate::trips::cost_edit::edit_in(ctx, &actor, tx, window, &asked),
    )
}

/// Saves the edit of a trip cost: the old posting is reversed and the new one is made on the same
/// day, in one transaction (PRD FR-12.19, FR-6.3).
///
/// # Errors
/// The mistakes of [`trip_cost_preview`], each at its field; `ERR_CASH_BELOW_ZERO_NOT_CONFIRMED`.
pub fn trip_cost_save(
    ctx: &Context,
    actor: &Actor,
    input: &TripCostEditInput,
) -> Result<EntryEdited, ServiceError> {
    let preview = trip_cost_preview(ctx, input)?;
    let window = day::window(ctx)?;
    entries::save(ctx, EditEntity::TripCost, preview, |tx| {
        crate::trips::cost_edit::edit_in(ctx, actor, tx, window, input)
    })
}

/// The review of an edit of a sale in another shop of a trip: what changes and what it does, from a
/// rehearsal of the real edit (PRD FR-7.70).
///
/// # Errors
/// `ERR_NOT_FOUND` on `id`. The mistakes of the edit come back inside the preview.
pub fn trip_other_sale_preview(
    ctx: &Context,
    input: &TripOtherSaleEditInput,
) -> Result<EditPreview, ServiceError> {
    let actor = crate::auth::require(ctx)?;
    let window = day::window(ctx)?;
    let id = input.id;
    let old = ctx.db.read(|conn| {
        mandi_db::ventures::get_entry(conn, id)?
            .filter(|e| e.kind == "other_sale")
            .ok_or_else(|| ServiceError::reject_field(ErrorCode::ErrNotFound, "id"))
    })?;
    let entry_date = BusinessDate::parse_iso(&old.entry_date)
        .ok_or_else(|| ServiceError::Data("the date of a sale is not a date".to_owned()))?;
    let mut asked = input.clone();
    asked.confirm_cash_below_zero = true;
    if asked.reason.trim().is_empty() {
        asked.reason = "-".to_owned();
    }
    entries::review(
        ctx,
        Scope {
            source_type: "venture",
            source_id: id,
            posting_id: old.journal_entry_id.unwrap_or(0),
        },
        entry_date,
        |conn| crate::trips::sale_edit::row_value(conn, id),
        |tx| crate::trips::sale_edit::edit_in(ctx, &actor, tx, window, &asked),
    )
}

/// Saves the edit of a sale in another shop of a trip: the old posting is reversed and the new one is
/// made on the same day, in one transaction (PRD FR-12.19, FR-6.3).
///
/// # Errors
/// The mistakes of [`trip_other_sale_preview`], each at its field;
/// `ERR_CASH_BELOW_ZERO_NOT_CONFIRMED`.
pub fn trip_other_sale_save(
    ctx: &Context,
    actor: &Actor,
    input: &TripOtherSaleEditInput,
) -> Result<EntryEdited, ServiceError> {
    let preview = trip_other_sale_preview(ctx, input)?;
    let window = day::window(ctx)?;
    entries::save(ctx, EditEntity::TripOtherSale, preview, |tx| {
        crate::trips::sale_edit::edit_in(ctx, actor, tx, window, input)
    })
}

/// The review of an edit of a garden installment, running cost or sale in another shop: what changes
/// and what it does, from a rehearsal of the real edit (PRD FR-7.70).
///
/// # Errors
/// `ERR_NOT_FOUND` on `id`. The mistakes of the edit come back inside the preview.
pub fn garden_entry_preview(
    ctx: &Context,
    input: &GardenEntryEditInput,
) -> Result<EditPreview, ServiceError> {
    let actor = crate::auth::require(ctx)?;
    let window = day::window(ctx)?;
    let id = input.id;
    let old = ctx.db.read(|conn| {
        mandi_db::ventures::get_entry(conn, id)?
            .filter(|e| matches!(e.kind.as_str(), "installment" | "cost" | "other_sale"))
            .ok_or_else(|| ServiceError::reject_field(ErrorCode::ErrNotFound, "id"))
    })?;
    let entry_date = BusinessDate::parse_iso(&old.entry_date)
        .ok_or_else(|| ServiceError::Data("the date of a garden entry is not a date".to_owned()))?;
    let mut asked = input.clone();
    asked.confirm_cash_below_zero = true;
    match &mut asked.entry {
        mandi_domain::core::gardens::GardenEntryInput::Installment(p)
        | mandi_domain::core::gardens::GardenEntryInput::Cost(p) => {
            p.confirm_cash_below_zero = true;
        }
        mandi_domain::core::gardens::GardenEntryInput::OtherSale(_) => {}
    }
    if asked.reason.trim().is_empty() {
        asked.reason = "-".to_owned();
    }
    entries::review(
        ctx,
        Scope {
            source_type: "venture",
            source_id: id,
            posting_id: old.journal_entry_id.unwrap_or(0),
        },
        entry_date,
        |conn| crate::gardens::entry_edit::row_value(conn, id),
        |tx| crate::gardens::entry_edit::edit_in(ctx, &actor, tx, window, &asked),
    )
}

/// Saves the edit of a garden entry: the old posting is reversed and the new one is made on the same
/// day, in one transaction (PRD FR-12.19, FR-6.3).
///
/// # Errors
/// The mistakes of [`garden_entry_preview`], each at its field;
/// `ERR_CASH_BELOW_ZERO_NOT_CONFIRMED`.
pub fn garden_entry_save(
    ctx: &Context,
    actor: &Actor,
    input: &GardenEntryEditInput,
) -> Result<EntryEdited, ServiceError> {
    let preview = garden_entry_preview(ctx, input)?;
    let window = day::window(ctx)?;
    entries::save(ctx, EditEntity::GardenEntry, preview, |tx| {
        crate::gardens::entry_edit::edit_in(ctx, actor, tx, window, input)
    })
}

/// The review of an edit of a garden arrival: what changes, from a rehearsal of the real edit (PRD
/// FR-7.70). An arrival posts nothing, so the effect has no party and no cash.
///
/// # Errors
/// `ERR_NOT_FOUND` on `id`. The mistakes of the edit come back inside the preview.
pub fn garden_arrival_preview(
    ctx: &Context,
    input: &GardenArrivalEditInput,
) -> Result<EditPreview, ServiceError> {
    let actor = crate::auth::require(ctx)?;
    let window = day::window(ctx)?;
    let id = input.id;
    let old = ctx.db.read(|conn| {
        mandi_db::arrivals::get(conn, id)?
            .ok_or_else(|| ServiceError::reject_field(ErrorCode::ErrNotFound, "id"))
    })?;
    let entry_date = BusinessDate::parse_iso(&old.entry_date)
        .ok_or_else(|| ServiceError::Data("the day of an arrival is not a date".to_owned()))?;
    let mut asked = input.clone();
    if asked.reason.trim().is_empty() {
        asked.reason = "-".to_owned();
    }
    entries::review(
        ctx,
        Scope {
            source_type: "garden_arrival",
            source_id: id,
            posting_id: 0,
        },
        entry_date,
        |conn| crate::gardens::arrival_edit::row_value(conn, id),
        |tx| crate::gardens::arrival_edit::edit_in(ctx, &actor, tx, window, &asked),
    )
}

/// Saves the edit of a garden arrival, in one transaction.
///
/// # Errors
/// The mistakes of [`garden_arrival_preview`], each at its field.
pub fn garden_arrival_save(
    ctx: &Context,
    actor: &Actor,
    input: &GardenArrivalEditInput,
) -> Result<EntryEdited, ServiceError> {
    let preview = garden_arrival_preview(ctx, input)?;
    let window = day::window(ctx)?;
    entries::save(ctx, EditEntity::GardenArrival, preview, |tx| {
        crate::gardens::arrival_edit::edit_in(ctx, actor, tx, window, input)
    })
}

/// The review of an edit of a trip's header (market, person sent, note): what changes, from a
/// rehearsal of the real edit (PRD FR-7.70). It posts nothing.
///
/// # Errors
/// `ERR_NOT_FOUND` on `id`. The mistakes of the edit come back inside the preview.
pub fn trip_header_preview(
    ctx: &Context,
    input: &TripHeaderEditInput,
) -> Result<EditPreview, ServiceError> {
    let actor = crate::auth::require(ctx)?;
    let window = day::window(ctx)?;
    let id = input.id;
    let old = ctx.db.read(|conn| {
        mandi_db::ventures::get(conn, id)?
            .filter(|v| v.kind == "trip")
            .ok_or_else(|| ServiceError::reject_field(ErrorCode::ErrNotFound, "id"))
    })?;
    let entry_date = BusinessDate::parse_iso(&old.entry_date)
        .ok_or_else(|| ServiceError::Data("the date of a trip is not a date".to_owned()))?;
    let mut asked = input.clone();
    if asked.reason.trim().is_empty() {
        asked.reason = "-".to_owned();
    }
    entries::review(
        ctx,
        Scope {
            source_type: "trip",
            source_id: id,
            posting_id: 0,
        },
        entry_date,
        |conn| crate::trips::header_edit::row_value(conn, id),
        |tx| crate::trips::header_edit::edit_in(ctx, &actor, tx, window, &asked),
    )
}

/// Saves the edit of a trip's header, in one transaction.
///
/// # Errors
/// The mistakes of [`trip_header_preview`], each at its field.
pub fn trip_header_save(
    ctx: &Context,
    actor: &Actor,
    input: &TripHeaderEditInput,
) -> Result<EntryEdited, ServiceError> {
    let preview = trip_header_preview(ctx, input)?;
    let window = day::window(ctx)?;
    entries::save(ctx, EditEntity::Trip, preview, |tx| {
        crate::trips::header_edit::edit_in(ctx, actor, tx, window, input)
    })
}

/// The spelling check of the new names of a garden whose header is edited. A name that was not
/// changed needs none.
///
/// # Errors
/// `ERR_SPELLING_UNCONFIRMED` on `name_ur` and/or `name_en` (value `words`); the name codes.
fn garden_spelling(
    ctx: &Context,
    input: &GardenHeaderEditInput,
) -> Result<Option<crate::spelling::SpellingCheck>, ServiceError> {
    let Some((name_ur, name_en)) = crate::gardens::header_edit::changed_names(ctx, input)? else {
        return Ok(None);
    };
    crate::spelling::check(
        ctx,
        &crate::spelling::NamePair {
            field_ur: "name_ur",
            name_ur: &name_ur,
            field_en: "name_en",
            name_en: &name_en,
            kind: mandi_domain::core::translit::Kind::Place,
        },
        &input.garden.confirmed_rules_words,
    )
    .map(Some)
}

/// The review of an edit of a garden's header (names, item, rent, season end, note): what changes,
/// from a rehearsal of the real edit (PRD FR-7.70). It posts nothing.
///
/// # Errors
/// `ERR_NOT_FOUND` on `id`. The mistakes of the edit come back inside the preview.
pub fn garden_header_preview(
    ctx: &Context,
    input: &GardenHeaderEditInput,
) -> Result<EditPreview, ServiceError> {
    let actor = crate::auth::require(ctx)?;
    let window = day::window(ctx)?;
    let id = input.id;
    let old = ctx.db.read(|conn| {
        mandi_db::ventures::get(conn, id)?
            .filter(|v| v.kind == "garden")
            .ok_or_else(|| ServiceError::reject_field(ErrorCode::ErrNotFound, "id"))
    })?;
    let entry_date = BusinessDate::parse_iso(&old.entry_date)
        .ok_or_else(|| ServiceError::Data("the date of a garden is not a date".to_owned()))?;
    // A new spelling that needs the user's word is a mistake of the edit, shown with the others.
    if let Err(refused) = garden_spelling(ctx, input) {
        return preview_of(
            ctx,
            entry_date,
            Err::<Rehearsed<serde_json::Value>, _>(refused),
            diff::changes_of_rows,
        );
    }
    let mut asked = input.clone();
    if asked.reason.trim().is_empty() {
        asked.reason = "-".to_owned();
    }
    entries::review(
        ctx,
        Scope {
            source_type: "garden",
            source_id: id,
            posting_id: 0,
        },
        entry_date,
        |conn| crate::gardens::header_edit::row_value(conn, id),
        |tx| {
            crate::gardens::header_edit::edit_in(ctx, &actor, tx, window, &asked, None)
                .map(|(edited, _)| edited)
        },
    )
}

/// Saves the edit of a garden's header, in one transaction. New names teach the spelling pairs they
/// confirm and reach the garden's account and the search index.
///
/// # Errors
/// The mistakes of [`garden_header_preview`], each at its field.
pub fn garden_header_save(
    ctx: &Context,
    actor: &Actor,
    input: &GardenHeaderEditInput,
) -> Result<EntryEdited, ServiceError> {
    let preview = garden_header_preview(ctx, input)?;
    if !preview.errors.is_empty() {
        return Err(ServiceError::from(preview.errors));
    }
    let window = day::window(ctx)?;
    let spelled = garden_spelling(ctx, input)?;
    let account_id = ctx
        .db
        .read(|conn| mandi_db::gardens::get(conn, input.id).map_err(ServiceError::from))?
        .map(|g| g.account.id);
    let (edited, learned) = ctx.db.write(|tx| {
        crate::gardens::header_edit::edit_in(ctx, actor, tx, window, input, spelled.as_ref())
    })?;
    if let Some(learned) = learned {
        learned.after_commit(ctx);
    }
    if let Some(account_id) = account_id {
        crate::search::refresh_account(ctx, account_id);
    }
    entries::finish(ctx, EditEntity::Garden, preview, edited)
}
