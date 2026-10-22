//! A sale of trip goods in another shop (PRD FR-12.32, C21, C25, plan 16b): date, shop, purchase,
//! qty and the money received. The qty and its cost (C21) come off the purchase; the money comes into
//! روکڑ and goes to the trip's partners by their % (C25), on their 0604 accounts. The preview and the
//! save call the same [`plan`] (Code Standards B5).
//!
//! Example (E2E-34): 8,460 received for goods of a trip at 40 / 30 / 30 is 3,384 / 2,538 / 2,538 on
//! the partners' 0604 accounts, against روکڑ بنام 8,460.

use mandi_db::accounts as account_repo;
use mandi_db::bikri::edit::{self as edit_repo, NewMove};
use mandi_db::ledger as ledger_repo;
use mandi_db::purchases::{self as purchase_repo, PurchaseRec};
use mandi_db::rusqlite::Connection;
use mandi_db::ventures::{self as repo, NewEntry};
use mandi_domain::bikri::own_cost;
use mandi_domain::core::dates::BusinessDate;
use mandi_domain::core::error::{ErrorCode, ErrorValue, FieldError};
use mandi_domain::core::ids::AccountId;
use mandi_domain::core::money::{Paisa, Qty};
use mandi_domain::core::rules::{Working, WorkingArg, WorkingValue};
use mandi_domain::core::trips::{
    OtherSaleInput, OtherSaleNumbers, OtherSalePreview, OtherSaleSaved, TripEntryCancelInput,
    TripEntryCancelled,
};
use mandi_domain::core::validate::{self, FieldErrors};
use mandi_domain::ledger::{BalancedEntry, PostingType, SourceType};
use mandi_domain::ventures::{CreditPart, credit_parts, receipt_entry};
use serde_json::json;

use crate::audit::Event;
use crate::auth::Actor;
use crate::cash;
use crate::context::Context;
use crate::day;
use crate::error::{ServiceError, money_error};
use crate::ledger;
use crate::purchases;
use crate::ventures::{
    self, Settlement, Venture, credit_partners, credits_view, not_found, require_open_day,
    required_reason,
};
use crate::vouchers::stored_cash_change;

fn data(message: &str) -> ServiceError {
    ServiceError::Data(message.to_owned())
}

fn money_arg(name: &str, value: Paisa) -> WorkingArg {
    WorkingArg {
        name: name.to_owned(),
        value: WorkingValue::Money { paisa: value },
    }
}

/// A sale that passed every check, with what the save writes.
struct Ready {
    venture: Venture,
    purchase: PurchaseRec,
    date: BusinessDate,
    shop: Option<String>,
    note: Option<String>,
    amount: Paisa,
    qty: Qty,
    cost: Paisa,
    credits: Vec<CreditPart>,
    entry: BalancedEntry,
    numbers: OtherSaleNumbers,
}

/// Every mistake of a typed sale, and the numbers if it can be worked out.
struct Plan {
    errors: Vec<FieldError>,
    warnings: Vec<FieldError>,
    ready: Option<Ready>,
}

impl Plan {
    fn preview(&self) -> OtherSalePreview {
        OtherSalePreview {
            numbers: self.ready.as_ref().map(|r| r.numbers.clone()),
            errors: self.errors.clone(),
            warnings: self.warnings.clone(),
        }
    }
}

fn optional_text(errors: &mut FieldErrors, field: &str, text: Option<&String>) -> Option<String> {
    let text = text.map(|t| t.trim()).filter(|t| !t.is_empty())?;
    errors
        .take(validate::free_text(field, text))
        .filter(|t| !t.is_empty())
}

/// Checks and works out a typed sale in another shop. Nothing is written.
///
/// # Errors
/// `ServiceError::Db`, `ServiceError::Data` or `ERR_OVERFLOW` for a failure the user cannot fix.
/// The user's mistakes come back in [`Plan::errors`].
#[allow(clippy::too_many_lines)]
fn plan(
    conn: &Connection,
    window: (Option<BusinessDate>, BusinessDate),
    input: &OtherSaleInput,
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
    let date = errors.take(validate::posting_date(
        "date",
        &input.date,
        closed_up_to,
        today,
    ));
    let shop = optional_text(&mut errors, "shop", input.shop.as_ref());
    let note = optional_text(&mut errors, "note", input.note.as_ref());
    let purchase = match (input.purchase_id, &venture) {
        (None, _) => {
            errors.push(FieldError::new(ErrorCode::ErrRequired, "purchase_id"));
            None
        }
        (Some(id), Some(v)) => match purchase_repo::get(conn, id)? {
            Some(p) if p.trip_id == v.rec.id => {
                if p.status == "cancelled" {
                    errors.push(FieldError::new(
                        ErrorCode::ErrPurchaseCancelled,
                        "purchase_id",
                    ));
                    None
                } else {
                    Some(p)
                }
            }
            _ => {
                errors.push(FieldError::new(ErrorCode::ErrNotFound, "purchase_id"));
                None
            }
        },
        (Some(_), None) => None,
    };
    if let (Some(date), Some(p)) = (date, purchase.as_ref()) {
        let bought = BusinessDate::parse_iso(&p.entry_date)
            .ok_or_else(|| data("the date of a purchase is not a date"))?;
        if date < bought {
            errors.push(
                FieldError::new(ErrorCode::ErrDateBeforeReceived, "date")
                    .with("received_date", ErrorValue::Text(bought.to_iso())),
            );
        }
    }
    let qty = errors.take(validate::qty("qty", &input.qty));
    let amount = errors
        .take(validate::money("amount", &input.amount))
        .filter(|a| {
            let positive = *a > Paisa::ZERO;
            if !positive {
                errors.push(FieldError::new(ErrorCode::ErrMustBeMoreThanZero, "amount"));
            }
            positive
        });
    let qty_left = match &purchase {
        Some(p) => p
            .qty
            .checked_sub(p.qty_sold)
            .and_then(|v| v.checked_sub(p.qty_written_off))
            .ok_or_else(|| data("the quantity of a purchase does not fit"))?,
        None => 0,
    };
    if let (Some(q), Some(_)) = (qty, purchase.as_ref())
        && q.get() > qty_left
    {
        errors.push(
            FieldError::new(ErrorCode::ErrQtyExceedsLot, "qty")
                .with("available", ErrorValue::Number(qty_left)),
        );
    }
    let found = errors.finish().err().unwrap_or_default();
    let (Some(venture), Some(purchase), Some(date), Some(qty), Some(amount)) =
        (venture, purchase, date, qty, amount)
    else {
        return Ok(Plan {
            errors: found,
            warnings: Vec::new(),
            ready: None,
        });
    };
    if !found.is_empty() {
        return Ok(Plan {
            errors: found,
            warnings: Vec::new(),
            ready: None,
        });
    }
    // C21: the cost of the goods sold. The last نگ take all the cost left. Never posted.
    let cost_left = Paisa::try_from_paisa(
        purchase
            .total_cost_paisa
            .checked_sub(purchase.cost_released_paisa)
            .ok_or_else(|| data("the cost of a purchase does not fit"))?,
    )
    .map_err(money_error)?;
    let (cost, cost_working) =
        own_cost(cost_left, qty_left, qty.get()).map_err(|e| ServiceError::Data(e.to_string()))?;
    // C25: the money goes to the trip's partners by their shares.
    let partners = credit_partners(&venture)?;
    let credits = credit_parts(amount, &partners).map_err(money_error)?;
    let cash_id = account_repo::id_by_code(conn, cash::CASH_ACCOUNT_CODE)?
        .ok_or_else(|| data("the cash account is missing"))?;
    let entry =
        receipt_entry(amount, &credits, AccountId::new(cash_id)).map_err(ledger::ledger_error)?;
    let result = amount.checked_sub(cost).map_err(money_error)?;
    let qty_left_after = qty_left
        .checked_sub(qty.get())
        .ok_or_else(|| data("the quantity does not fit"))?;
    let numbers = OtherSaleNumbers {
        amount,
        qty: qty.get(),
        cost_paisa: cost,
        cost_working,
        result_paisa: result,
        result_working: Working {
            message_id: "working-other-sale-result".to_owned(),
            args: vec![
                money_arg("amount", amount),
                money_arg("cost", cost),
                money_arg("result", result),
            ],
            rule_id: None,
        },
        credits: credits_view(&venture, &credits)?,
        qty_left_after,
        cost_left_after_paisa: cost_left.checked_sub(cost).map_err(money_error)?,
    };
    Ok(Plan {
        errors: found,
        warnings: Vec::new(),
        ready: Some(Ready {
            venture,
            purchase,
            date,
            shop,
            note,
            amount,
            qty,
            cost,
            credits,
            entry,
            numbers,
        }),
    })
}

/// Works out a typed sale in another shop for the live screen. Nothing is saved.
///
/// # Errors
/// `ServiceError::Db` or `ServiceError::Data` for a failure the user cannot fix; the user's mistakes
/// come back inside the preview.
pub(super) fn preview(
    ctx: &Context,
    input: &OtherSaleInput,
) -> Result<OtherSalePreview, ServiceError> {
    let window = day::window(ctx)?;
    ctx.db.read(|conn| Ok(plan(conn, window, input)?.preview()))
}

/// Saves a sale in another shop in one transaction (PRD FR-12.32): the entry with each partner's
/// part, the journal entry (روکڑ بنام, each partner's 0604 جمع), the stock move with its cost, the
/// purchase's qty and cost released, the trip's status and the audit row.
///
/// # Errors
/// Every error of the preview, each at its field and all together, with nothing written;
/// `ERR_QTY_EXCEEDS_LOT` (value `available`) on `qty`; `ERR_DATE_CLOSED` and `ERR_DATE_IN_FUTURE` on
/// `date`.
pub(super) fn save(
    ctx: &Context,
    actor: &Actor,
    input: &OtherSaleInput,
) -> Result<OtherSaleSaved, ServiceError> {
    let window = day::window(ctx)?;
    let now = ctx.now_text();
    ctx.db.write(|tx| {
        let planned = plan(tx, window, input)?;
        if !planned.errors.is_empty() {
            return Err(ServiceError::from(planned.errors.clone()));
        }
        let preview = planned.preview();
        let ready = planned
            .ready
            .ok_or_else(|| data("a sale without errors has no numbers"))?;
        let user = actor.user_id;
        let date_iso = ready.date.to_iso();
        let number = repo::next_entry_number(tx, "other_sale")?;
        let id = repo::insert_entry(
            tx,
            &NewEntry {
                venture_id: ready.venture.rec.id,
                kind: "other_sale",
                entry_date: &date_iso,
                amount_paisa: ready.amount.as_paisa(),
                reason: ready.note.as_deref(),
                purchase_id: Some(ready.purchase.id),
                qty: Some(ready.qty.get()),
                shop_name: ready.shop.as_deref(),
                cost_paisa: ready.cost.as_paisa(),
            },
            number,
            user,
            &now,
        )?;
        ventures::write_credits(tx, user, &now, id, &ready.credits)?;
        let entry_id = ventures::post_settlement(
            ctx,
            tx,
            user,
            &now,
            &Settlement {
                venture_entry_id: id,
                date: ready.date,
                entry: Some(&ready.entry),
                posting_type: PostingType::VentureReceipt,
                source_type: SourceType::Venture,
                source_id: id,
            },
        )?
        .ok_or_else(|| data("a sale in another shop has no journal entry"))?;
        // The goods go out of the purchase with their cost (C21): the move keeps both, so a cancel
        // gives back exactly these.
        let qty_out = ready
            .qty
            .get()
            .checked_neg()
            .ok_or_else(|| data("the quantity does not fit"))?;
        edit_repo::insert_move(
            tx,
            &NewMove {
                lot_id: None,
                purchase_id: Some(ready.purchase.id),
                cost_paisa: ready.cost.as_paisa(),
                entry_id: Some(entry_id),
                move_date: &date_iso,
                kind: "sold",
                qty: qty_out,
                reason: None,
                reverses_move_id: None,
            },
            user,
            &now,
        )?;
        purchases::take_sold(
            tx,
            ready.purchase.id,
            ready.qty.get(),
            ready.cost,
            user,
            &now,
        )?;
        let change =
            ventures::refresh_trip_status(tx, ready.venture.rec.id, &date_iso, user, &now)?;
        ctx.db.auditor().write(
            tx,
            &Event::new(Some(user), "create", "trip_other_sale")
                .entity_id(id)
                .new_value(json!({
                    "number": number,
                    "trip_id": ready.venture.rec.id,
                    "purchase_id": ready.purchase.id,
                    "entry_date": date_iso,
                    "entry_id": entry_id,
                    "shop": ready.shop,
                    "qty": ready.qty.get(),
                    "amount_paisa": ready.amount.as_paisa(),
                    "cost_paisa": ready.cost.as_paisa(),
                }))
                .into_entry(),
        )?;
        let version = repo::get_entry(tx, id)?
            .map(|e| e.version)
            .ok_or_else(|| data("a sale is missing after its save"))?;
        Ok(OtherSaleSaved {
            id,
            number,
            version,
            entry_id,
            trip_closed: change == ventures::TripChange::Closed,
            preview,
        })
    })
}

/// Cancels a sale in another shop by reversal (PRD FR-12.32): the money goes out of روکڑ again, and
/// the qty and its stored cost go back to the purchase. The trip opens again if it had closed.
///
/// # Errors
/// `ERR_NOT_FOUND` and `ERR_TRIP_ENTRY_CANCELLED` on `id`; `ERR_VERSION_CHANGED` on `version`;
/// `ERR_REASON_REQUIRED` on `reason`; `ERR_DATE_CLOSED` on `id`; `ERR_CASH_BELOW_ZERO_NOT_CONFIRMED`.
/// Nothing is written when any of them is returned.
pub(super) fn cancel(
    ctx: &Context,
    actor: &Actor,
    input: &TripEntryCancelInput,
) -> Result<TripEntryCancelled, ServiceError> {
    let reason = required_reason(&input.reason)?;
    let now = ctx.now_text();
    ctx.db.write(|tx| {
        let entry = repo::get_entry(tx, input.id)?
            .filter(|e| e.kind == "other_sale")
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
            .ok_or_else(|| data("the date of a sale is not a date"))?;
        require_open_day(ctx, date, "id")?;
        let journal_id = entry
            .journal_entry_id
            .ok_or_else(|| data("a sale in another shop has no journal entry"))?;
        let purchase_id = entry
            .purchase_id
            .ok_or_else(|| data("a sale in another shop has no purchase"))?;
        // The money goes out of cash again (PRD FR-6.7).
        let stored = ledger_repo::get_entry(tx, journal_id)?
            .ok_or_else(|| data("the entry of a sale cannot be read"))?;
        let change = stored_cash_change(tx, &stored)?
            .checked_neg()
            .map_err(money_error)?;
        cash::require_confirmed(
            &cash::cash_after_entry(tx, change)?,
            input.confirm_cash_below_zero,
        )?;
        let user = actor.user_id;
        let reversal = ledger::reverse(ctx, tx, user, journal_id, &reason)?;
        edit_repo::reverse_moves(tx, journal_id, reversal.entry_id, user, &now)?;
        let qty_back = entry
            .qty
            .and_then(i64::checked_neg)
            .ok_or_else(|| data("the quantity of a sale does not fit"))?;
        let cost_back = entry
            .cost_paisa
            .checked_neg()
            .ok_or_else(|| data("the cost of a sale does not fit"))?;
        purchase_repo::add_sold(tx, purchase_id, qty_back, cost_back, user, &now)?;
        purchases::ensure_consistent(tx, purchase_id)?;
        repo::mark_entry_cancelled(tx, entry.id, &reason, user, &now)?;
        let change =
            ventures::refresh_trip_status(tx, entry.venture_id, &entry.entry_date, user, &now)?;
        ctx.db.auditor().write(
            tx,
            &Event::new(Some(user), "cancel", "trip_other_sale")
                .entity_id(entry.id)
                .old(json!({
                    "number": entry.number,
                    "status": "saved",
                    "amount_paisa": entry.amount_paisa,
                }))
                .new_value(json!({
                    "status": "cancelled",
                    "reversal_entry_id": reversal.entry_id,
                }))
                .reason(&reason)
                .into_entry(),
        )?;
        Ok(TripEntryCancelled {
            id: entry.id,
            number: entry.number,
            reversal_entry_id: Some(reversal.entry_id),
            trip_reopened: change == ventures::TripChange::Reopened,
        })
    })
}
