//! A cost of the whole trip, such as the truck or the food (PRD FR-12.31, C23, C24, plan 16b). It
//! is paid in as any payment of a trip (each partner's part from the shop's cash or his pocket,
//! FR-12.30) and spread, when it is saved, over the trip's purchases that still have goods left, by
//! the نگ bought. The preview and the save call the same [`plan`] (Code Standards B5).
//!
//! Example (E2E-41): a truck of 4,000 on a trip whose only purchase is 80 نگ: the purchase carries
//! all 4,000, its total cost goes from 96,000 to 100,000, and the parts are 2,000 / 2,000.

use mandi_db::ledger as ledger_repo;
use mandi_db::purchases as purchase_repo;
use mandi_db::rusqlite::Connection;
use mandi_db::ventures::{self as repo, NewEntry};
use mandi_domain::core::cash::CashCheck;
use mandi_domain::core::dates::BusinessDate;
use mandi_domain::core::error::{ErrorCode, ErrorValue, FieldError};
use mandi_domain::core::money::Paisa;
use mandi_domain::core::trips::{
    CostSpreadPart, TripCostInput, TripCostNumbers, TripCostPreview, TripCostSaved,
    TripEntryCancelInput, TripEntryCancelled,
};
use mandi_domain::core::validate::{self, FieldErrors};
use mandi_domain::ledger::{PostingType, SourceType};
use mandi_domain::ventures::{SpreadPart, spread_cost};
use serde_json::json;

use crate::audit::Event;
use crate::auth::Actor;
use crate::cash;
use crate::context::Context;
use crate::day;
use crate::error::{Rejection, ServiceError, money_error};
use crate::ledger;
use crate::purchases;
use crate::ventures::{
    self, PaymentPlan, Settlement, Venture, not_found, parts_view, plan_payment, require_open_day,
    required_reason,
};
use crate::vouchers::stored_cash_change;

fn data(message: &str) -> ServiceError {
    ServiceError::Data(message.to_owned())
}

/// A trip cost that passed every check, with what the save writes.
struct Ready {
    venture: Venture,
    date: BusinessDate,
    amount: Paisa,
    reason: String,
    payment: PaymentPlan,
    spread: Vec<SpreadPart>,
    numbers: TripCostNumbers,
}

/// Every mistake of a typed cost, and the numbers if it can be worked out.
struct Plan {
    errors: Vec<FieldError>,
    warnings: Vec<FieldError>,
    ready: Option<Ready>,
    cash: CashCheck,
}

impl Plan {
    fn preview(&self) -> TripCostPreview {
        TripCostPreview {
            numbers: self.ready.as_ref().map(|r| r.numbers.clone()),
            cash: self.cash,
            errors: self.errors.clone(),
            warnings: self.warnings.clone(),
        }
    }
}

/// Checks and works out a trip cost. Nothing is written.
///
/// # Errors
/// `ServiceError::Db`, `ServiceError::Data` or `ERR_OVERFLOW` for a failure the user cannot fix.
/// The user's mistakes come back in [`Plan::errors`].
fn plan(
    conn: &Connection,
    window: (Option<BusinessDate>, BusinessDate),
    input: &TripCostInput,
) -> Result<Plan, ServiceError> {
    let (closed_up_to, today) = window;
    let mut errors = FieldErrors::default();
    let venture = match input.trip_id {
        None => {
            errors.push(FieldError::new(ErrorCode::ErrRequired, "trip_id"));
            None
        }
        Some(id) => match ventures::load(conn, id)? {
            Some(v) if v.rec.kind == "trip" => Some(v),
            _ => {
                errors.push(FieldError::new(ErrorCode::ErrNotFound, "trip_id"));
                None
            }
        },
    };
    if venture.as_ref().is_some_and(|v| !v.is_open()) {
        errors.push(FieldError::new(ErrorCode::ErrTripClosed, "trip_id"));
    }
    let date = errors.take(validate::posting_date(
        "date",
        &input.date,
        closed_up_to,
        today,
    ));
    if let (Some(date), Some(v)) = (date, venture.as_ref()) {
        let trip_date = v.date()?;
        if date < trip_date {
            errors.push(
                FieldError::new(ErrorCode::ErrPurchaseBeforeTrip, "date")
                    .with("trip_date", ErrorValue::Text(trip_date.to_iso())),
            );
        }
    }
    let amount = errors
        .take(validate::money("amount", &input.amount))
        .filter(|a| {
            let positive = *a > Paisa::ZERO;
            if !positive {
                errors.push(FieldError::new(ErrorCode::ErrMustBeMoreThanZero, "amount"));
            }
            positive
        });
    let reason = errors
        .take(validate::free_text("reason", &input.reason))
        .filter(|r| {
            let given = !r.is_empty();
            if !given {
                errors.push(FieldError::new(ErrorCode::ErrReasonRequired, "reason"));
            }
            given
        });
    let carriers = match (&venture, input.trip_id) {
        (Some(v), Some(id)) if v.is_open() => {
            let carriers = purchase_repo::with_stock_of_trip(conn, id)?;
            if carriers.is_empty() {
                errors.push(FieldError::new(ErrorCode::ErrTripNoStock, "trip_id"));
            }
            carriers
        }
        _ => Vec::new(),
    };
    let partners = match &venture {
        Some(v) => ventures::resolve_sources(v, &input.parts, &mut errors)?,
        None => None,
    };
    let found = errors.finish().err().unwrap_or_default();
    let (Some(venture), Some(date), Some(amount), Some(reason), Some(partners)) =
        (venture, date, amount, reason, partners)
    else {
        return Ok(Plan {
            errors: found,
            warnings: Vec::new(),
            ready: None,
            cash: cash::cash_after_entry(conn, Paisa::ZERO)?,
        });
    };
    if carriers.is_empty() || !found.is_empty() {
        return Ok(Plan {
            errors: found,
            warnings: Vec::new(),
            ready: None,
            cash: cash::cash_after_entry(conn, Paisa::ZERO)?,
        });
    }
    let payment = plan_payment(conn, amount, &partners)?;
    let weights: Vec<(i64, i64)> = carriers.iter().map(|p| (p.id, p.qty)).collect();
    let spread = spread_cost(amount, &weights).map_err(money_error)?;
    let spread_view = spread
        .iter()
        .map(|part| {
            let rec = carriers
                .iter()
                .find(|p| p.id == part.purchase_id)
                .ok_or_else(|| data("a spread names a purchase that is not in the trip"))?;
            Ok(CostSpreadPart {
                purchase_id: rec.id,
                purchase_number: rec.number,
                item: rec.item.clone(),
                qty: part.qty,
                amount_paisa: part.amount,
                working: part.working.clone(),
            })
        })
        .collect::<Result<Vec<_>, ServiceError>>()?;
    let numbers = TripCostNumbers {
        amount,
        parts: parts_view(&venture, &payment.parts)?,
        spread: spread_view,
    };
    let cash = payment.cash;
    Ok(Plan {
        errors: found,
        warnings: Vec::new(),
        ready: Some(Ready {
            venture,
            date,
            amount,
            reason,
            payment,
            spread,
            numbers,
        }),
        cash,
    })
}

/// Works out a typed trip cost for the live screen. Nothing is saved.
///
/// # Errors
/// `ServiceError::Db` or `ServiceError::Data` for a failure the user cannot fix; the user's mistakes
/// come back inside the preview.
pub(super) fn preview(
    ctx: &Context,
    input: &TripCostInput,
) -> Result<TripCostPreview, ServiceError> {
    let window = day::window(ctx)?;
    ctx.db.read(|conn| Ok(plan(conn, window, input)?.preview()))
}

/// Saves a trip cost in one transaction (PRD FR-12.31): the entry with each partner's part, the
/// journal entry for the shop parts (if any), the spread over the purchases and the audit row.
///
/// # Errors
/// Every error of the preview, each at its field and all together, with nothing written;
/// `ERR_TRIP_CLOSED` and `ERR_TRIP_NO_STOCK` on `trip_id`; `ERR_DATE_CLOSED` and `ERR_DATE_IN_FUTURE`
/// on `date`; `ERR_CASH_BELOW_ZERO_NOT_CONFIRMED` (value `cash_after_paisa`) when the shop parts take
/// cash below 0 and `confirm_cash_below_zero` is false.
pub(super) fn save(
    ctx: &Context,
    actor: &Actor,
    input: &TripCostInput,
) -> Result<TripCostSaved, ServiceError> {
    let window = day::window(ctx)?;
    let now = ctx.now_text();
    ctx.db.write(|tx| {
        let planned = plan(tx, window, input)?;
        if !planned.errors.is_empty() {
            return Err(ServiceError::from(planned.errors.clone()));
        }
        cash::require_confirmed(&planned.cash, input.confirm_cash_below_zero)?;
        let preview = planned.preview();
        let ready = planned
            .ready
            .ok_or_else(|| data("a cost without errors has no numbers"))?;
        let user = actor.user_id;
        let date_iso = ready.date.to_iso();
        let number = repo::next_entry_number(tx, "cost")?;
        let id = repo::insert_entry(
            tx,
            &NewEntry {
                venture_id: ready.venture.rec.id,
                kind: "cost",
                entry_date: &date_iso,
                amount_paisa: ready.amount.as_paisa(),
                reason: Some(&ready.reason),
                purchase_id: None,
                qty: None,
                shop_name: None,
                cost_paisa: 0,
            },
            number,
            user,
            &now,
        )?;
        ventures::write_parts(tx, user, &now, id, &ready.payment.parts)?;
        let entry_id = ventures::post_settlement(
            ctx,
            tx,
            user,
            &now,
            &Settlement {
                venture_entry_id: id,
                date: ready.date,
                entry: ready.payment.entry.as_ref(),
                posting_type: PostingType::TripCost,
                source_type: SourceType::Venture,
                source_id: id,
            },
        )?;
        // C24: each purchase carries its part. Its total cost and cost left go up with it.
        for part in &ready.spread {
            purchase_repo::insert_cost_part(tx, id, part.purchase_id, part.amount.as_paisa(), user, &now)?;
            purchase_repo::add_trip_cost(tx, part.purchase_id, part.amount.as_paisa(), user, &now)?;
            purchases::ensure_consistent(tx, part.purchase_id)?;
        }
        ctx.db.auditor().write(
            tx,
            &Event::new(Some(user), "create", "trip_cost")
                .entity_id(id)
                .new_value(json!({
                    "number": number,
                    "trip_id": ready.venture.rec.id,
                    "entry_date": date_iso,
                    "entry_id": entry_id,
                    "amount_paisa": ready.amount.as_paisa(),
                    "reason": ready.reason,
                    "spread": ready
                        .spread
                        .iter()
                        .map(|s| json!({ "purchase_id": s.purchase_id, "amount_paisa": s.amount.as_paisa() }))
                        .collect::<Vec<_>>(),
                    "cash_after_paisa": planned.cash.cash_after_paisa.as_paisa(),
                    "cash_below_zero_confirmed": planned.cash.needs_confirm,
                }))
                .into_entry(),
        )?;
        let version = repo::get_entry(tx, id)?
            .map(|e| e.version)
            .ok_or_else(|| data("a trip cost is missing after its save"))?;
        Ok(TripCostSaved {
            id,
            number,
            version,
            entry_id,
            preview,
        })
    })
}

/// Cancels a trip cost by reversal (PRD FR-12.31): only while nothing of the purchases it was
/// spread over has been sold or written off. Its parts come off those purchases again.
///
/// # Errors
/// `ERR_NOT_FOUND` and `ERR_TRIP_ENTRY_CANCELLED` on `id`; `ERR_TRIP_COST_HAS_SALES` on `id` (values
/// `qty_sold`, `qty_written_off`); `ERR_VERSION_CHANGED` on `version`; `ERR_REASON_REQUIRED` on
/// `reason`; `ERR_DATE_CLOSED` on `id`; `ERR_CASH_BELOW_ZERO_NOT_CONFIRMED`. Nothing is written when
/// any of them is returned.
pub(super) fn cancel(
    ctx: &Context,
    actor: &Actor,
    input: &TripEntryCancelInput,
) -> Result<TripEntryCancelled, ServiceError> {
    let reason = required_reason(&input.reason)?;
    let now = ctx.now_text();
    ctx.db.write(|tx| {
        let entry = repo::get_entry(tx, input.id)?
            .filter(|e| e.kind == "cost")
            .ok_or_else(|| not_found("id"))?;
        if entry.status == "cancelled" {
            return Err(ServiceError::reject_field(
                ErrorCode::ErrTripEntryCancelled,
                "id",
            ));
        }
        if entry.version != input.version {
            return Err(ServiceError::reject_field(
                ErrorCode::ErrVersionChanged,
                "version",
            ));
        }
        let date = BusinessDate::parse_iso(&entry.entry_date)
            .ok_or_else(|| data("the date of a trip cost is not a date"))?;
        require_open_day(ctx, date, "id")?;
        let parts = purchase_repo::cost_parts_of_entry(tx, entry.id)?;
        let ids: Vec<i64> = parts.iter().map(|p| p.purchase_id).collect();
        let (mut sold, mut written_off) = (0_i64, 0_i64);
        for rec in purchase_repo::get_many(tx, &ids)? {
            sold = sold
                .checked_add(rec.qty_sold)
                .ok_or_else(|| data("the quantity of a purchase does not fit"))?;
            written_off = written_off
                .checked_add(rec.qty_written_off)
                .ok_or_else(|| data("the quantity of a purchase does not fit"))?;
        }
        if sold != 0 || written_off != 0 {
            return Err(ServiceError::Rejected(vec![Rejection {
                code: ErrorCode::ErrTripCostHasSales,
                field: Some("id".to_owned()),
                values: std::collections::BTreeMap::from([
                    ("qty_sold".to_owned(), ErrorValue::Number(sold)),
                    (
                        "qty_written_off".to_owned(),
                        ErrorValue::Number(written_off),
                    ),
                ]),
            }]));
        }
        let user = actor.user_id;
        let reversal_entry_id = match entry.journal_entry_id {
            Some(journal_id) => {
                // A cancel gives back what the shop parts took from cash (PRD FR-6.7).
                let stored = ledger_repo::get_entry(tx, journal_id)?
                    .ok_or_else(|| data("the entry of a trip cost cannot be read"))?;
                let change = stored_cash_change(tx, &stored)?
                    .checked_neg()
                    .map_err(money_error)?;
                cash::require_confirmed(
                    &cash::cash_after_entry(tx, change)?,
                    input.confirm_cash_below_zero,
                )?;
                Some(ledger::reverse(ctx, tx, user, journal_id, &reason)?.entry_id)
            }
            None => None,
        };
        for part in &parts {
            let back = part
                .amount_paisa
                .checked_neg()
                .ok_or_else(|| data("the cost does not fit"))?;
            purchase_repo::add_trip_cost(tx, part.purchase_id, back, user, &now)?;
            purchases::ensure_consistent(tx, part.purchase_id)?;
        }
        repo::mark_entry_cancelled(tx, entry.id, &reason, user, &now)?;
        ctx.db.auditor().write(
            tx,
            &Event::new(Some(user), "cancel", "trip_cost")
                .entity_id(entry.id)
                .old(json!({
                    "number": entry.number,
                    "status": "saved",
                    "amount_paisa": entry.amount_paisa,
                }))
                .new_value(json!({
                    "status": "cancelled",
                    "reversal_entry_id": reversal_entry_id,
                }))
                .reason(&reason)
                .into_entry(),
        )?;
        Ok(TripEntryCancelled {
            id: entry.id,
            number: entry.number,
            reversal_entry_id,
            trip_reopened: false,
        })
    })
}
