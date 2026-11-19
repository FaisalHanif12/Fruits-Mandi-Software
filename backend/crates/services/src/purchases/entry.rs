//! A purchase entry with many item lines (PRD FR-12.41, R1, plan 35): the preview and the save.
//!
//! Each line is its own purchase: the line is turned into the same [`PurchaseInput`] a single
//! purchase has (the trip, the day, the supplier, the day to sell and the partners' sources are the
//! entry's), checked and worked out by the same [`plan::plan`], and written by the same
//! [`write_one`]. So a line of an entry is exactly the purchase of today: same number series, same
//! posting, same cost, same shares (C23). Only three things belong to the entry:
//!
//! - the mistakes carry the line (`lines[i].qty`), and a mistake of the shared fields is listed once;
//! - the cash question (PRD FR-6.7) is asked once for the shop parts of ALL lines together;
//! - the save is ONE transaction: a mistake in any line saves nothing.
//!
//! Example (plan 35 Section 3): trip 0070, line 1 مالٹا 80 at 1,200 (50 for our shop, 30 for another
//! shop), line 2 کیلا 40 at 900. The save gives خ-1 (96,000) and خ-2 (36,000), each exactly as one
//! purchase.

use mandi_db::rusqlite::Connection;
use mandi_domain::core::cash::CashCheck;
use mandi_domain::core::dates::BusinessDate;
use mandi_domain::core::error::{ErrorCode, ErrorValue, FieldError};
use mandi_domain::core::money::Paisa;
use mandi_domain::core::purchases::{
    PurchaseEntryInput, PurchaseEntryPreview, PurchaseEntrySaved, PurchaseInput, PurchaseLineInput,
    PurchaseLinePreview,
};
use mandi_domain::core::rules::{Working, WorkingArg, WorkingValue};
use mandi_domain::core::ventures::{EntryPart, EntryParts};

use super::{plan, write_one};
use crate::auth::Actor;
use crate::cash;
use crate::context::Context;
use crate::day;
use crate::error::{ServiceError, money_error};

/// The most item lines one entry may hold.
pub(crate) const MAX_LINES: usize = 100;

/// The fields that belong to the entry, not to a line. A mistake of one of these is the same on
/// every line and is listed once.
fn is_shared(field: &str) -> bool {
    matches!(field, "trip_id" | "date" | "supplier" | "sell_date")
        || field == "parts"
        || field.starts_with("parts[")
}

/// The single purchase a line stands for.
fn line_input(entry: &PurchaseEntryInput, line: &PurchaseLineInput) -> PurchaseInput {
    PurchaseInput {
        trip_id: entry.trip_id,
        date: entry.date.clone(),
        supplier: entry.supplier.clone(),
        sell_date: entry.sell_date.clone(),
        item_id: line.item_id,
        packing_id: line.packing_id,
        qty: line.qty.clone(),
        weight: line.weight.clone(),
        rate: line.rate.clone(),
        freight: line.freight.clone(),
        labour: line.labour.clone(),
        other_cost: line.other_cost.clone(),
        note: line.note.clone(),
        parts: entry.parts.clone(),
        confirm_cash_below_zero: entry.confirm_cash_below_zero,
        other_qty: line.other_qty.clone(),
        shop_name: line.shop_name.clone(),
    }
}

/// Puts a mistake of line `i` at its path, and leaves a mistake of the entry where it is.
fn at_line(mut error: FieldError, i: usize) -> FieldError {
    if !error.field.is_empty() && !is_shared(&error.field) {
        error.field = format!("lines[{i}].{}", error.field);
    }
    error
}

fn push_once(list: &mut Vec<FieldError>, error: FieldError) {
    if !list.contains(&error) {
        list.push(error);
    }
}

/// Every line checked and worked out, against the database as it is now.
struct EntryPlan {
    lines: Vec<plan::Plan>,
    errors: Vec<FieldError>,
    warnings: Vec<FieldError>,
    /// What all the lines together do to cash in hand.
    cash: CashCheck,
}

fn plan_entry(
    conn: &Connection,
    window: (Option<BusinessDate>, BusinessDate),
    input: &PurchaseEntryInput,
) -> Result<EntryPlan, ServiceError> {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    if input.lines.is_empty() {
        errors.push(FieldError::new(ErrorCode::ErrRequired, "lines"));
    }
    if input.lines.len() > MAX_LINES {
        errors.push(FieldError::new(ErrorCode::ErrTooManyLines, "lines").with(
            "max",
            ErrorValue::Number(i64::try_from(MAX_LINES).unwrap_or(i64::MAX)),
        ));
        return Ok(EntryPlan {
            lines: Vec::new(),
            errors,
            warnings,
            cash: cash::cash_after_entry(conn, Paisa::ZERO)?,
        });
    }
    let mut lines = Vec::with_capacity(input.lines.len());
    let mut change = Paisa::ZERO;
    for (i, line) in input.lines.iter().enumerate() {
        let planned = plan::plan(conn, window, &line_input(input, line))?;
        for e in &planned.errors {
            push_once(&mut errors, at_line(e.clone(), i));
        }
        for w in &planned.warnings {
            push_once(&mut warnings, at_line(w.clone(), i));
        }
        change = change
            .checked_add(planned.cash.change_paisa)
            .map_err(money_error)?;
        lines.push(planned);
    }
    // The cash question is for the whole entry: the shop parts of all lines together.
    let cash = cash::cash_after_entry(conn, change)?;
    Ok(EntryPlan {
        lines,
        errors,
        warnings,
        cash,
    })
}

/// One partner's part of the entry while the lines are added up.
struct PartSum {
    part: EntryPart,
    /// How many lines added to it.
    lines: i64,
}

/// The entry's parts: the sums, or null when no line works out. A part that only one line added to
/// keeps that line's working, so a one-line entry shows exactly what the single purchase shows.
fn entry_parts(sums: Vec<PartSum>, shop: Paisa, pocket: Paisa) -> Option<EntryParts> {
    if sums.is_empty() {
        return None;
    }
    let parts = sums
        .into_iter()
        .map(|sum| {
            let mut part = sum.part;
            if sum.lines > 1 {
                part.working = Working {
                    message_id: "working-entry-part-sum".to_owned(),
                    args: vec![
                        WorkingArg {
                            name: "lines".to_owned(),
                            value: WorkingValue::Count { value: sum.lines },
                        },
                        WorkingArg {
                            name: "amount".to_owned(),
                            value: WorkingValue::Money {
                                paisa: part.amount_paisa,
                            },
                        },
                    ],
                    rule_id: None,
                };
            }
            part
        })
        .collect();
    Some(EntryParts {
        parts,
        shop_total_paisa: shop,
        pocket_total_paisa: pocket,
    })
}

/// Works out an entry for the live screen. Nothing is saved.
///
/// # Errors
/// `ServiceError::Db` or `ServiceError::Data` for a failure the user cannot fix. The mistakes come
/// back inside the preview.
pub(super) fn preview(
    ctx: &Context,
    input: &PurchaseEntryInput,
) -> Result<PurchaseEntryPreview, ServiceError> {
    let window = day::window(ctx)?;
    ctx.db.read(|conn| {
        let planned = plan_entry(conn, window, input)?;
        let mut total = Paisa::ZERO;
        let mut lines = Vec::with_capacity(planned.lines.len());
        // Each partner's part of the whole entry: the sum of his parts of the lines.
        let mut sums: Vec<PartSum> = Vec::new();
        let (mut shop_total, mut pocket_total) = (Paisa::ZERO, Paisa::ZERO);
        for (line, input_line) in planned.lines.iter().zip(&input.lines) {
            let numbers = line.ready.as_ref().map(|r| r.numbers.clone());
            if let Some(n) = &numbers {
                total = total.checked_add(n.paid_now).map_err(money_error)?;
                shop_total = shop_total
                    .checked_add(n.parts.shop_total_paisa)
                    .map_err(money_error)?;
                pocket_total = pocket_total
                    .checked_add(n.parts.pocket_total_paisa)
                    .map_err(money_error)?;
                for part in &n.parts.parts {
                    match sums
                        .iter_mut()
                        .find(|s| s.part.partner.id == part.partner.id)
                    {
                        Some(sum) => {
                            sum.part.amount_paisa = sum
                                .part
                                .amount_paisa
                                .checked_add(part.amount_paisa)
                                .map_err(money_error)?;
                            sum.lines = sum.lines.saturating_add(1);
                        }
                        None => sums.push(PartSum {
                            part: part.clone(),
                            lines: 1,
                        }),
                    }
                }
            }
            let qty_our = line
                .ready
                .as_ref()
                .and_then(|r| r.qty.get().checked_sub(r.other_qty));
            lines.push(PurchaseLinePreview {
                key: input_line.key.clone(),
                qty_our,
                numbers,
            });
        }
        Ok(PurchaseEntryPreview {
            lines,
            paid_now_total: total,
            parts: entry_parts(sums, shop_total, pocket_total),
            cash: planned.cash,
            errors: planned.errors,
            warnings: planned.warnings,
        })
    })
}

/// Saves an entry in ONE transaction.
///
/// # Errors
/// Every mistake of the preview, at its field (`lines[i].qty` ...), and
/// `ERR_CASH_BELOW_ZERO_NOT_CONFIRMED` (value `cash_after_paisa`) when the shop parts of all lines
/// together take cash below 0 and `confirm_cash_below_zero` is false. Nothing is written when any
/// of them is returned.
pub(super) fn save(
    ctx: &Context,
    actor: &Actor,
    input: &PurchaseEntryInput,
) -> Result<PurchaseEntrySaved, ServiceError> {
    let window = day::window(ctx)?;
    let now = ctx.now_text();
    ctx.db.write(|tx| {
        let planned = plan_entry(tx, window, input)?;
        if !planned.errors.is_empty() {
            return Err(plan::rejected(planned.errors));
        }
        cash::require_confirmed(&planned.cash, input.confirm_cash_below_zero)?;
        let mut saved = Vec::with_capacity(input.lines.len());
        let mut total = Paisa::ZERO;
        for line in &input.lines {
            // Each line is worked out again just before it is written, against the books with the
            // lines before it in them, so its numbers and its cash note are the real ones.
            let one = plan::plan(tx, window, &line_input(input, line))?;
            if !one.errors.is_empty() {
                return Err(plan::rejected(one.errors));
            }
            let preview = one.preview();
            // The audit row says whether the ENTRY needed the question (one line: the same as today).
            let cash = mandi_domain::core::cash::CashCheck {
                needs_confirm: planned.cash.needs_confirm,
                ..one.cash
            };
            let ready = one
                .ready
                .ok_or_else(|| ServiceError::Data("a line without errors has no numbers".into()))?;
            let done = write_one(ctx, tx, actor, &now, ready, preview, cash)?;
            total = total.checked_add(done.total_cost).map_err(money_error)?;
            saved.push(done);
        }
        Ok(PurchaseEntrySaved {
            saved,
            paid_now_total: total,
        })
    })
}
