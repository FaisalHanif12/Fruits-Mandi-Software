//! A later sale of goods left in a lot (PRD FR-12.17, plan 16).
//!
//! It adds new sale lines to an existing lot as a NEW posting on the day it happens (an open day,
//! not before the goods were received). The original posting and its day are not touched, even if
//! that day is closed. The lines look the rules up by the date of the later sale. The seller is
//! credited `Σ new seller amounts − Σ new seller commission` (C8 without lot charges): freight,
//! store rent, مزدوری, منشیانہ, فون, نقد and the per-lot commission belong to the first posting and
//! are NOT charged again. The partner split uses the shares of the later day.
//!
//! The preview and the save work the draft out with the same function (`work_out`, Code Standards
//! B5), as a lot whose quantity available is what the lot has left.
//!
//! Example (PRD E2E-16b): 5 نگ left of رحیم's کیلا are sold to ارشد at 560 on 11-10-26: value 2,800,
//! seller commission 168, buyer commission 168, لاگا 40, ارشد pays 3,008, seller credit 2,632,
//! shared income 376 split 152 / 112 / 112; the lot then shows خام 29,300, کل اخراجات 10,078 and
//! صافی 19,222, and nothing is left in باقی مال.

use mandi_db::bikri::edit::{self as edit_repo, LotState};
use mandi_db::rusqlite::Connection;
use mandi_domain::bikri::{PostingKind, PostingTarget, StoredLot, StoredPosting};
use mandi_domain::core::bikri::{
    BikriDraft, BikriPreview, LaterSaleInput, LaterSaleSaveInput, LaterSaleSaved, PreparedDraft,
    SaleInput, prepare_draft,
};
use mandi_domain::core::dates::BusinessDate;
use mandi_domain::core::error::{ErrorCode, ErrorValue, FieldError};
use mandi_domain::core::money::Paisa;
use mandi_domain::core::validate;
use mandi_domain::ledger::{PostingType, SourceType};
use serde_json::json;

use super::load::{later_header, later_lot_input};
use super::rows::{
    LotCharges, Posting, check_shares, insert_sales_and_facts, lot_running_totals, match_posting,
    posting_sums, shape, stored_line_ids,
};
use super::shared::{Basis, data, switches, work_out};
use crate::audit::Event;
use crate::auth::Actor;
use crate::cash;
use crate::context::Context;
use crate::day;
use crate::error::{ServiceError, money_error};
use crate::ledger::{self, PostingEvent};
use crate::rules;

/// The charges of a lot as its first posting stored them, for the running totals of the lot.
///
/// # Errors
/// `ServiceError::Data` for an amount that is not a whole rupee.
pub(super) fn lot_charges_of(state: &LotState) -> Result<LotCharges, ServiceError> {
    let paisa = |value: i64| Paisa::try_from_paisa(value).map_err(money_error);
    Ok(LotCharges {
        freight: if state.freight_by_firm {
            paisa(state.freight_paisa)?
        } else {
            Paisa::ZERO
        },
        labour: paisa(state.labour_paisa)?,
        munshiana: paisa(state.munshiana_paisa)?,
        cash_paid: paisa(state.cash_paid_paisa)?,
        store_rent: paisa(state.store_rent_paisa)?,
        phone: paisa(state.phone_paisa)?,
    })
}

/// Everything a later sale works from, read once.
struct LaterPlan {
    state: LotState,
    date: Option<BusinessDate>,
    prepared: PreparedDraft,
    basis: Basis,
    extra: Vec<FieldError>,
}

/// The date of a later sale is `date` on this screen, not `header.entry_date`.
fn on_date(mut errors: Vec<FieldError>) -> Vec<FieldError> {
    for error in &mut errors {
        if error.field == "header.entry_date" {
            error.field = "date".to_owned();
        }
    }
    errors
}

/// Reads the lot and the draft of a later sale and checks them against each other.
fn plan_later(
    conn: &Connection,
    lot_id: i64,
    date_text: &str,
    sales: &[SaleInput],
    window: (Option<BusinessDate>, BusinessDate),
) -> Result<LaterPlan, ServiceError> {
    let state = edit_repo::lot_state(conn, lot_id)?
        .ok_or_else(|| ServiceError::reject_field(ErrorCode::ErrLotNotFound, "lot_id"))?;
    if state.bikri_status != "saved" {
        return Err(ServiceError::reject_field(
            ErrorCode::ErrBikriCancelled,
            "lot_id",
        ));
    }
    let taken = state
        .qty_sold
        .checked_add(state.qty_written_off)
        .and_then(|v| v.checked_add(state.qty_taken))
        .ok_or_else(|| data("the quantity of a lot does not fit"))?;
    // A lot of own goods is sold in full in its بکری and never has a later sale (PRD FR-12.26). The
    // same holds for a garden lot linked to an arrival (PRD FR-12.40).
    if state.qty_received <= taken || state.purchase_id.is_some() || state.arrival_id.is_some() {
        return Err(ServiceError::reject_field(
            ErrorCode::ErrNothingToSell,
            "lot_id",
        ));
    }
    let (closed_up_to, today) = window;
    let mut extra = Vec::new();
    // Not before the goods were received.
    if let Ok(date) = validate::date("date", date_text, today)
        && date.to_iso() < state.received_date
    {
        extra.push(
            FieldError::new(ErrorCode::ErrDateBeforeReceived, "date").with(
                "received_date",
                ErrorValue::Text(state.received_date.clone()),
            ),
        );
    }
    let sales: Vec<SaleInput> = sales
        .iter()
        .cloned()
        .map(|mut sale| {
            // A later sale has only new lines.
            sale.sale_id = None;
            sale
        })
        .collect();
    let mut header = later_header(today);
    header.entry_date = date_text.to_owned();
    let draft = BikriDraft {
        header,
        lots: vec![later_lot_input(&state, sales)],
    };
    let prepared = prepare_draft(&draft, today, closed_up_to);
    let date = prepared.header.entry_date;
    let stored = StoredPosting {
        kind: PostingKind::Later,
        lots: std::iter::once((
            lot_id,
            StoredLot {
                elsewhere_qty: taken,
                ..StoredLot::default()
            },
        ))
        .collect(),
        sales: std::collections::HashMap::new(),
    };
    Ok(LaterPlan {
        state,
        date,
        prepared,
        basis: Basis {
            target: PostingTarget::LaterSale { lot_id },
            stored,
            replaces: None,
        },
        extra,
    })
}

/// Works out a later sale for the live summary (PRD FR-12.17). Nothing is saved.
///
/// # Errors
/// `ERR_LOT_NOT_FOUND`, `ERR_BIKRI_CANCELLED` and `ERR_NOTHING_TO_SELL` on `lot_id`. The mistakes in
/// the lines and the date come back inside the preview (`errors`).
pub fn preview(ctx: &Context, input: &LaterSaleInput) -> Result<BikriPreview, ServiceError> {
    let window = day::window(ctx)?;
    let rule_set = rules::rule_set(ctx)?;
    let settings = rules::overrides(ctx)?;
    let switches = switches(&settings);
    ctx.db.read(|conn| {
        let plan = plan_later(conn, input.lot_id, &input.date, &input.sales, window)?;
        let mut worked = work_out(conn, plan.prepared, &rule_set, switches, &plan.basis)?;
        worked.errors = on_date(worked.errors);
        worked.errors.extend(plan.extra);
        Ok(worked.preview())
    })
}

/// Saves a later sale as a new posting of the lot's بکری (PRD FR-12.17): see the module text.
///
/// # Errors
/// The errors of [`preview`], each at its field and all together (`ERR_DATE_CLOSED` on `date`,
/// `ERR_QTY_EXCEEDS_LOT` on `lots[0].sales[j].qty`, `ERR_DATE_BEFORE_RECEIVED` on `date`);
/// `ERR_CASH_BELOW_ZERO_NOT_CONFIRMED` (value `cash_after_paisa`). Nothing is written when any of
/// them is returned.
pub fn save(
    ctx: &Context,
    actor: &Actor,
    input: &LaterSaleSaveInput,
) -> Result<LaterSaleSaved, ServiceError> {
    let window = day::window(ctx)?;
    let rule_set = rules::rule_set(ctx)?;
    let settings = rules::overrides(ctx)?;
    let switches = switches(&settings);
    let now = ctx.now_text();
    ctx.db.write(|tx| {
        let plan = plan_later(tx, input.lot_id, &input.date, &input.sales, window)?;
        let LaterPlan {
            state,
            date,
            prepared,
            basis,
            extra,
        } = plan;
        let worked = work_out(tx, prepared, &rule_set, switches, &basis)?;
        let mut errors = on_date(worked.errors.clone());
        errors.extend(extra);
        if !errors.is_empty() {
            return Err(ServiceError::from(errors));
        }
        cash::require_confirmed(&worked.cash, input.confirm_cash_below_zero)?;
        let date = date.ok_or_else(|| data("a later sale without errors has no date"))?;
        let (entry, layout) = worked
            .entry
            .as_ref()
            .ok_or_else(|| data("a later sale without errors has no journal entry"))?;
        let lots = shape(&worked)?;
        let user = actor.user_id;

        // The posting, then its lines, stock moves and facts. No `received` move: the goods came
        // with the first posting.
        let posted = ledger::post(
            ctx,
            tx,
            user,
            &PostingEvent {
                date,
                posting_type: PostingType::BikriLater,
                source_type: SourceType::Bikri,
                source_id: state.bikri_id,
                narration: None,
                reverses: None,
                entry: entry.clone(),
            },
        )?;
        let line_ids = stored_line_ids(tx, posted.entry_id, entry.lines().len())?;
        let date_iso = date.to_iso();
        let post = Posting {
            user_id: user,
            now: &now,
            bikri_id: state.bikri_id,
            entry_id: posted.entry_id,
            date_iso: &date_iso,
            year: date.year(),
            month: date.month(),
            adda_id: state.adda_id,
        };
        let new_ids = insert_sales_and_facts(tx, &post, &lots, &[state.id], false)?;

        // The lot's running totals, from every line in force.
        let shaped = lots
            .first()
            .ok_or_else(|| data("a later sale has no lot"))?;
        if edit_repo::lot_sums_of_entry(tx, state.id, posted.entry_id)? != posting_sums(shaped)? {
            return Err(data("the saved lines of a lot differ from the preview"));
        }
        let sums = edit_repo::lot_sums(tx, state.id, None)?;
        let running = lot_running_totals(&lot_charges_of(&state)?, sums)?;
        edit_repo::update_lot_totals(tx, state.id, &running, user, &now)?;
        if running
            .qty_sold
            .checked_add(state.qty_written_off)
            .and_then(|v| v.checked_add(state.qty_taken))
            > Some(state.qty_received)
        {
            return Err(data("a lot sold more than it received"));
        }

        // Matching of a new posting, which is the newest line of its keys.
        match_posting(tx, &post, &lots, (entry, layout), &line_ids)?;
        check_shares(tx, posted.entry_id, worked.calc.summary.shared_income)?;
        edit_repo::bump_version(tx, state.bikri_id, user, &now)?;
        ctx.db.auditor().write(
            tx,
            &Event::new(Some(user), "later_sale", "bikri")
                .entity_id(state.bikri_id)
                .new_value(json!({
                    "number": state.bikri_number,
                    "lot_id": state.id,
                    "entry_id": posted.entry_id,
                    "entry_date": date_iso,
                    "lines": new_ids.iter().map(Vec::len).sum::<usize>(),
                    "gross_paisa": worked.calc.summary.gross.as_paisa(),
                    "seller_credit_paisa": worked.calc.summary.seller_credit.as_paisa(),
                    "buyer_total_paisa": worked.calc.summary.buyer_total.as_paisa(),
                    "shared_income_paisa": worked.calc.summary.shared_income.as_paisa(),
                    "cash_after_paisa": worked.cash.cash_after_paisa.as_paisa(),
                    "cash_below_zero_confirmed": worked.cash.needs_confirm,
                }))
                .into_entry(),
        )?;
        Ok(LaterSaleSaved {
            bikri_id: state.bikri_id,
            number: state.bikri_number,
            lot_id: state.id,
            entry_id: posted.entry_id,
            preview: {
                let mut preview = worked.preview();
                preview.errors = on_date(preview.errors);
                preview
            },
        })
    })
}
