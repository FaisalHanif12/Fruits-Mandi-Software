//! Checks and works out one purchase of a trip (PRD 12.10, FR-12.24, C1, C23, plan 16b). The preview
//! and the save both call [`plan`], so the screen and the saved numbers cannot differ (Code
//! Standards B5).
//!
//! Money (PRD C1): the value is `qty × rate`, or the kilograms times the rate for an item sold by
//! weight, rounded once. Paid now is the value plus freight, labour and other cost, each paid at
//! once. It is split by the trip's shares (C23) and each partner's part comes from the shop's cash
//! or his pocket (FR-12.30). Worked numbers (E2E-34): 20 × 500 = 10,000, plus freight 400, is 10,400
//! (520 for each نگ), split 4,160 / 3,120 / 3,120.

use mandi_db::bikri as bikri_repo;
use mandi_db::rusqlite::Connection;
use mandi_domain::core::cash::CashCheck;
use mandi_domain::core::dates::BusinessDate;
use mandi_domain::core::error::{ErrorCode, ErrorValue, FieldError};
use mandi_domain::core::money::{Grams, Paisa, Qty, avg_rate, line_value, weight_value};
use mandi_domain::core::purchases::{
    PurchaseInput, PurchaseNumbers, PurchasePreview, PurchaseWorkings,
};
use mandi_domain::core::rules::{Working, WorkingArg, WorkingValue};
use mandi_domain::core::validate::{self, FieldErrors};
use mandi_domain::ledger::MAX_LINE_PAISA;

use crate::cash;
use crate::error::{ServiceError, money_error};
use crate::ventures::{self, PaymentPlan, Venture, parts_view, plan_payment, resolve_sources};

/// A purchase that passed every check, with what the save writes.
#[derive(Debug)]
pub(super) struct Plan {
    /// Every mistake, each at its field.
    pub errors: Vec<FieldError>,
    /// What only warns.
    pub warnings: Vec<FieldError>,
    /// The numbers and the rows to write; `None` while the purchase is not complete.
    pub ready: Option<Ready>,
    /// What the entry does to cash in hand.
    pub cash: CashCheck,
}

/// A complete, checked purchase.
#[derive(Debug)]
pub(super) struct Ready {
    pub venture: Venture,
    pub date: BusinessDate,
    pub sell_date: BusinessDate,
    pub supplier: Option<String>,
    pub item_id: i64,
    pub packing_id: i64,
    pub qty: Qty,
    /// نگ for another shop (PRD FR-12.41).
    pub other_qty: i64,
    pub shop_name: Option<String>,
    pub weight: Option<Grams>,
    pub rate: Paisa,
    pub value: Paisa,
    pub freight: Paisa,
    pub labour: Paisa,
    pub other_cost: Paisa,
    pub paid_now: Paisa,
    pub note: Option<String>,
    pub payment: PaymentPlan,
    pub numbers: PurchaseNumbers,
}

impl Plan {
    /// What the preview sends.
    pub(super) fn preview(&self) -> PurchasePreview {
        PurchasePreview {
            numbers: self.ready.as_ref().map(|r| r.numbers.clone()),
            cash: self.cash,
            errors: self.errors.clone(),
            warnings: self.warnings.clone(),
        }
    }
}

fn money_arg(name: &str, paisa: Paisa) -> WorkingArg {
    WorkingArg {
        name: name.to_owned(),
        value: WorkingValue::Money { paisa },
    }
}

fn count_arg(name: &str, value: i64) -> WorkingArg {
    WorkingArg {
        name: name.to_owned(),
        value: WorkingValue::Count { value },
    }
}

fn working(message_id: &str, args: Vec<WorkingArg>) -> Working {
    Working {
        message_id: message_id.to_owned(),
        args,
        rule_id: None,
    }
}

/// Optional money: empty is 0.
fn optional_money(errors: &mut FieldErrors, field: &str, text: Option<&String>) -> Paisa {
    match text.map(|t| t.trim()).filter(|t| !t.is_empty()) {
        Some(text) => errors
            .take(validate::money(field, text))
            .unwrap_or(Paisa::ZERO),
        None => Paisa::ZERO,
    }
}

fn optional_text(errors: &mut FieldErrors, field: &str, text: Option<&String>) -> Option<String> {
    let text = text.map(|t| t.trim()).filter(|t| !t.is_empty())?;
    errors
        .take(validate::free_text(field, text))
        .filter(|t| !t.is_empty())
}

/// Checks and works out a purchase. Nothing is written.
///
/// # Errors
/// `ServiceError::Db`, `ServiceError::Data` or `ERR_OVERFLOW` for a failure the user cannot fix.
/// The user's mistakes come back in [`Plan::errors`].
#[allow(clippy::too_many_lines)]
pub(super) fn plan(
    conn: &Connection,
    window: (Option<BusinessDate>, BusinessDate),
    input: &PurchaseInput,
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
    // A purchase is not dated before its trip (PRD FR-12.24).
    if let (Some(date), Some(v)) = (date, venture.as_ref()) {
        let trip_date = v.date()?;
        if date < trip_date {
            errors.push(
                FieldError::new(ErrorCode::ErrPurchaseBeforeTrip, "date")
                    .with("trip_date", ErrorValue::Text(trip_date.to_iso())),
            );
        }
    }

    let sell_date = ventures::sell_date_of(&mut errors, input.sell_date.as_ref(), date, today);

    // Item and packing; a weight item needs its kilograms.
    let item = match input.item_id {
        None => {
            errors.push(FieldError::new(ErrorCode::ErrRequired, "item_id"));
            None
        }
        Some(id) => {
            let found = bikri_repo::items(conn, &[id])?.into_iter().next();
            match found {
                None => {
                    errors.push(FieldError::new(ErrorCode::ErrNotFound, "item_id"));
                    None
                }
                Some(item) if !item.is_active => {
                    errors.push(FieldError::new(ErrorCode::ErrAccountInactive, "item_id"));
                    None
                }
                Some(item) => Some(item),
            }
        }
    };
    let packing = match input.packing_id {
        None => {
            errors.push(FieldError::new(ErrorCode::ErrRequired, "packing_id"));
            None
        }
        Some(id) => {
            let found = bikri_repo::existing_ids(conn, bikri_repo::MasterTable::Packings, &[id])?;
            if found.is_empty() {
                errors.push(FieldError::new(ErrorCode::ErrNotFound, "packing_id"));
                None
            } else {
                Some(id)
            }
        }
    };
    let by_weight = item.as_ref().is_some_and(|i| i.sold_by == "weight");
    let qty = errors.take(validate::qty("qty", &input.qty));
    let weight = if by_weight {
        match input.weight.as_ref().filter(|w| !w.trim().is_empty()) {
            Some(text) => errors.take(validate::weight("weight", text)),
            None => {
                errors.push(FieldError::new(ErrorCode::ErrWeightRequired, "weight"));
                None
            }
        }
    } else {
        None
    };
    let (other_qty, shop_name) = ventures::split_of(
        &mut errors,
        "",
        input.other_qty.as_ref(),
        input.shop_name.as_ref(),
        qty.map(Qty::get),
    );
    let rate = errors.take(validate::rate("rate", &input.rate));
    let freight = optional_money(&mut errors, "freight", input.freight.as_ref());
    let labour = optional_money(&mut errors, "labour", input.labour.as_ref());
    let other_cost = optional_money(&mut errors, "other_cost", input.other_cost.as_ref());
    let note = optional_text(&mut errors, "note", input.note.as_ref());
    let supplier = optional_text(&mut errors, "supplier", input.supplier.as_ref());
    let partners = match &venture {
        Some(v) => resolve_sources(v, &input.parts, &mut errors)?,
        None => None,
    };

    // C1: the value of the goods.
    let value = match (qty, rate) {
        (Some(q), Some(r)) => {
            let worked = match weight {
                Some(w) => weight_value(w, r).map(|v| {
                    (
                        v,
                        working(
                            "working-weight-value",
                            vec![
                                count_arg("grams", w.get()),
                                money_arg("rate", r),
                                money_arg("result", v),
                            ],
                        ),
                    )
                }),
                None => line_value(q, r).map(|v| {
                    (
                        v,
                        working(
                            "working-line-value",
                            vec![
                                count_arg("qty", q.get()),
                                money_arg("rate", r),
                                money_arg("result", v),
                            ],
                        ),
                    )
                }),
            };
            match worked {
                Ok((v, w)) if v > Paisa::ZERO => Some((v, w)),
                Ok(_) => {
                    // A weight item can round to 0 rupees.
                    errors.push(FieldError::new(ErrorCode::ErrMustBeMoreThanZero, "rate"));
                    None
                }
                Err(e) => return Err(money_error(e)),
            }
        }
        _ => None,
    };

    let errors_so_far = errors.finish().err().unwrap_or_default();
    let (
        Some(venture),
        Some(date),
        Some(sell_date),
        Some(item),
        Some(packing),
        Some(qty),
        Some(rate),
        Some((value, value_working)),
        Some(partners),
    ) = (
        venture, date, sell_date, item, packing, qty, rate, value, partners,
    )
    else {
        return Ok(Plan {
            errors: errors_so_far,
            warnings: Vec::new(),
            ready: None,
            cash: cash::cash_after_entry(conn, Paisa::ZERO)?,
        });
    };
    let mut errors = errors_so_far;

    // Paid now: the value plus freight, labour and other cost, all paid at once.
    let paid_now = value
        .checked_add(freight)
        .and_then(|t| t.checked_add(labour))
        .and_then(|t| t.checked_add(other_cost))
        .map_err(money_error)?;
    if paid_now.as_paisa() > MAX_LINE_PAISA {
        errors.push(FieldError::new(ErrorCode::ErrAmountTooLarge, "rate").with(
            "max",
            ErrorValue::Number(MAX_LINE_PAISA.checked_div(100).unwrap_or(i64::MAX)),
        ));
    }
    if !errors.is_empty() {
        return Ok(Plan {
            errors,
            warnings: Vec::new(),
            ready: None,
            cash: cash::cash_after_entry(conn, Paisa::ZERO)?,
        });
    }
    // C23: the parts, and the entry for the parts paid from the shop's cash.
    let payment = plan_payment(conn, paid_now, &partners)?;
    let avg = avg_rate(paid_now, qty).map_err(money_error)?;
    let numbers = PurchaseNumbers {
        value,
        freight,
        labour,
        other_cost,
        paid_now,
        total_cost: paid_now,
        avg_cost: avg,
        parts: parts_view(&venture, &payment.parts)?,
        workings: PurchaseWorkings {
            value: value_working,
            paid_now: working(
                "working-purchase-paid-now",
                vec![
                    money_arg("value", value),
                    money_arg("freight", freight),
                    money_arg("labour", labour),
                    money_arg("other_cost", other_cost),
                    money_arg("result", paid_now),
                ],
            ),
            total_cost: working(
                "working-purchase-total-cost",
                vec![
                    money_arg("paid_now", paid_now),
                    money_arg("trip_cost", Paisa::ZERO),
                    money_arg("result", paid_now),
                ],
            ),
            avg_cost: working(
                "working-purchase-avg-cost",
                vec![
                    money_arg("total", paid_now),
                    count_arg("qty", qty.get()),
                    money_arg("result", avg),
                ],
            ),
        },
    };
    let cash_check = payment.cash;
    Ok(Plan {
        errors,
        warnings: Vec::new(),
        ready: Some(Ready {
            venture,
            date,
            sell_date,
            supplier,
            item_id: item.id,
            packing_id: packing,
            qty,
            other_qty,
            shop_name,
            weight,
            rate,
            value,
            freight,
            labour,
            other_cost,
            paid_now,
            note,
            payment,
            numbers,
        }),
        cash: cash_check,
    })
}

/// A rejection that carries the list of errors of a plan.
pub(super) fn rejected(errors: Vec<FieldError>) -> ServiceError {
    ServiceError::from(errors)
}
