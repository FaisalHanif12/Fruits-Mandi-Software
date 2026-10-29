//! Goods that arrived from a garden (مال آیا, PRD FR-12.40, plan 33 v1.1): a stock record of the
//! garden, with its date to sell. It posts NO journal entry and changes no partner total: it is only
//! goods. A garden lot of a بکری linked to it takes its goods from it (`services::bikri`), and the
//! goods that are not sold stay in it and come again into the next new بکری.
//!
//! - [`save`], [`cancel`], [`set_sell_date`]: one transaction each (Code Standards B3).
//! - [`write_off`], [`write_off_undo`]: spoiled goods leave the arrival and nothing is posted.
//! - [`take_sold`] and [`give_back_entry`]: what a garden lot sold comes off the arrival, and goes
//!   back exactly when the posting is edited or cancelled.
//! - [`list`], [`get`]: reads, with the figures read from the stored rows.
//!
//! Example: 100 پیٹی کیلا arrive on 28-10-2026, to sell on 29-10-2026. A new بکری of 29-10-2026
//! opens with a ready lot of 100. 60 are sold: the arrival has 60 sold and 40 left, and the 40 come
//! again into the next new بکری.

use std::collections::HashMap;

use mandi_db::arrivals::{self as repo, ArrivalRec, ArrivalWriteOff, ListFilter, NewArrival};
use mandi_db::bikri as bikri_repo;
use mandi_db::bikri::edit::{self as edit_repo, NewMove};
use mandi_db::rusqlite::Connection;
use mandi_domain::core::arrivals::{
    ArrivalCancelInput, ArrivalCancelled, ArrivalEntryInput, ArrivalEntryPreview,
    ArrivalEntrySaved, ArrivalGetInput, ArrivalInput, ArrivalLinePreview, ArrivalLotRef,
    ArrivalRow, ArrivalSaved, ArrivalStateFilter, ArrivalStatus, ArrivalWriteOffDone,
    ArrivalWriteOffInput, ArrivalWriteOffUndoInput, ArrivalWriteOffUndone, ArrivalsList,
    ArrivalsListInput, ArrivalsTotals, SellDateInput, SellDateSet,
};
use mandi_domain::core::bikri::WriteOffRow;
use mandi_domain::core::codes::{CodeType, number_code};
use mandi_domain::core::dates::BusinessDate;
use mandi_domain::core::error::{ErrorCode, ErrorValue, FieldError};
use mandi_domain::core::money::Grams;
use mandi_domain::core::split::StockParts;
use mandi_domain::core::validate::{self, FieldErrors};
use mandi_domain::core::ventures::GardenRef;
use serde_json::json;

use crate::audit::Event;
use crate::auth::Actor;
use crate::context::Context;
use crate::day;
use crate::error::{Rejection, ServiceError};
use crate::ventures::{self, not_found, require_open_day, required_reason, sell_date_of};

fn data(message: &str) -> ServiceError {
    ServiceError::Data(message.to_owned())
}

/// نگ left in an arrival: arrived less sold less written off.
fn left_of(rec: &ArrivalRec) -> Result<i64, ServiceError> {
    rec.qty
        .checked_sub(rec.qty_sold)
        .and_then(|v| v.checked_sub(rec.qty_written_off))
        .ok_or_else(|| data("the quantity of an arrival does not fit"))
}

/// What is left of each part of an arrival (PRD FR-12.41).
pub(crate) fn parts_of(rec: &ArrivalRec) -> Result<StockParts, ServiceError> {
    StockParts::of(
        rec.qty,
        rec.other_qty,
        rec.qty_sold,
        rec.qty_sold_other,
        rec.qty_written_off,
    )
    .ok_or_else(|| data("the split of an arrival does not fit"))
}

/// The garden of an arrival as other screens point at it, with the garden's account.
pub(crate) fn garden_ref(rec: &ArrivalRec) -> GardenRef {
    GardenRef {
        id: rec.garden_id,
        number: rec.garden_number,
        code: number_code(CodeType::Garden, rec.garden_number),
        name_ur: rec.garden_name_ur.clone(),
        name_en: rec.garden_name_en.clone(),
        account: rec.account.clone(),
    }
}

fn write_off_rows(off: Vec<ArrivalWriteOff>) -> Vec<WriteOffRow> {
    off.into_iter()
        .map(|w| WriteOffRow {
            move_id: w.move_id,
            move_date: w.move_date,
            qty: w.qty,
            reason: w.reason,
        })
        .collect()
}

/// One arrival as the screens show it.
fn row_of(
    rec: &ArrivalRec,
    lots: Vec<ArrivalLotRef>,
    write_offs: Vec<WriteOffRow>,
    commission: &mandi_db::commission::Commission,
    closed_up_to: Option<BusinessDate>,
    today: BusinessDate,
) -> Result<ArrivalRow, ServiceError> {
    let (commission_paisa, commission_working) = ventures::commission_view(commission)?;
    let status =
        ArrivalStatus::parse(&rec.status).ok_or_else(|| data("an unknown arrival status"))?;
    let qty_left = if status == ArrivalStatus::Cancelled {
        0
    } else {
        left_of(rec)?
    };
    let entry = BusinessDate::parse_iso(&rec.entry_date)
        .ok_or_else(|| data("the day of an arrival is not a date"))?;
    let sell = BusinessDate::parse_iso(&rec.sell_date)
        .ok_or_else(|| data("the day to sell of an arrival is not a date"))?;
    let parts = parts_of(rec)?;
    let saved = status == ArrivalStatus::Saved;
    let day_open = closed_up_to.is_none_or(|closed| entry > closed);
    Ok(ArrivalRow {
        id: rec.id,
        version: rec.version,
        number: rec.number,
        garden: garden_ref(rec),
        entry_date: rec.entry_date.clone(),
        sell_date: rec.sell_date.clone(),
        status,
        cancel_reason: rec.cancel_reason.clone(),
        item: rec.item.clone(),
        packing: rec.packing.clone(),
        qty: rec.qty,
        weight_grams: rec.weight_grams,
        note: rec.note.clone(),
        qty_sold: rec.qty_sold,
        qty_written_off: rec.qty_written_off,
        qty_left,
        qty_other: rec.other_qty,
        shop_name: rec.shop_name.clone(),
        qty_our_sold: parts.our_sold,
        qty_other_sold: parts.other_sold,
        qty_our_left: if status == ArrivalStatus::Cancelled {
            0
        } else {
            parts.our_left
        },
        qty_other_left: if status == ArrivalStatus::Cancelled {
            0
        } else {
            parts.other_left
        },
        can_change_split: saved && qty_left > 0,
        days_waiting: if qty_left > 0 {
            today.days_after(sell).max(0)
        } else {
            0
        },
        can_cancel: saved && rec.qty_sold == 0 && rec.qty_written_off == 0 && day_open,
        can_change_sell_date: saved && qty_left > 0,
        lots,
        write_offs,
        commission_paisa,
        commission_working,
    })
}

/// The rows of arrivals, with their lots and write-offs from two grouped reads.
fn rows_of(
    conn: &Connection,
    recs: &[ArrivalRec],
    closed_up_to: Option<BusinessDate>,
    today: BusinessDate,
) -> Result<Vec<ArrivalRow>, ServiceError> {
    let ids: Vec<i64> = recs.iter().map(|r| r.id).collect();
    let mut offs = repo::write_offs_of(conn, &ids)?;
    let commissions = mandi_db::commission::by_arrivals(conn, &ids)?;
    recs.iter()
        .map(|rec| {
            let lots = repo::lots_of(conn, rec.id)?
                .into_iter()
                .map(|l| ArrivalLotRef {
                    bikri_id: l.bikri_id,
                    bikri_number: l.bikri_number,
                    lot_id: l.lot_id,
                    lot_no: l.lot_no,
                    entry_date: l.entry_date,
                    qty_sold: l.qty_sold,
                })
                .collect();
            let off = write_off_rows(offs.remove(&rec.id).unwrap_or_default());
            row_of(
                rec,
                lots,
                off,
                &commissions.get(&rec.id).copied().unwrap_or_default(),
                closed_up_to,
                today,
            )
        })
        .collect()
}

/// What the entry (or the single arrival) decides for every line: the garden and the two days.
struct Shared {
    garden: ventures::Venture,
    date: BusinessDate,
    sell_date: BusinessDate,
}

/// A line checked: ready to be written.
struct Line {
    item_id: i64,
    packing_id: i64,
    qty: i64,
    weight: Option<Grams>,
    note: Option<String>,
    other_qty: i64,
    shop_name: Option<String>,
}

/// The fields every line shares: the garden (open, a garden), the day (open, not before the
/// garden) and the day to sell. Mistakes go to `errors` at the entry's own paths.
fn check_shared(
    tx: &Connection,
    window: (Option<BusinessDate>, BusinessDate),
    errors: &mut FieldErrors,
    garden_id: Option<i64>,
    date_text: &str,
    sell_text: Option<&String>,
) -> Result<Option<Shared>, ServiceError> {
    let (closed_up_to, today) = window;
    let garden = match garden_id {
        None => {
            errors.push(FieldError::new(ErrorCode::ErrRequired, "garden_id"));
            None
        }
        Some(id) => match ventures::load(tx, id)? {
            Some(v) if v.rec.kind == "garden" => Some(v),
            _ => {
                errors.push(FieldError::new(ErrorCode::ErrNotFound, "garden_id"));
                None
            }
        },
    };
    if garden.as_ref().is_some_and(|g| !g.is_open()) {
        errors.push(FieldError::new(ErrorCode::ErrGardenClosed, "garden_id"));
    }
    let date = errors.take(validate::posting_date(
        "date",
        date_text,
        closed_up_to,
        today,
    ));
    // An arrival is not dated before its garden (PRD FR-12.34).
    if let (Some(date), Some(g)) = (date, garden.as_ref()) {
        let garden_date = g.date()?;
        if date < garden_date {
            errors.push(
                FieldError::new(ErrorCode::ErrDateBeforeGarden, "date")
                    .with("garden_date", ErrorValue::Text(garden_date.to_iso())),
            );
        }
    }
    let sell_date = sell_date_of(errors, sell_text, date, today);
    Ok(match (garden, date, sell_date) {
        (Some(garden), Some(date), Some(sell_date)) => Some(Shared {
            garden,
            date,
            sell_date,
        }),
        _ => None,
    })
}

/// One line: item, packing, qty, weight, note and the split. `prefix` is `""` for a single arrival
/// and `"lines[i]."` for a line of an entry.
#[allow(clippy::too_many_arguments)]
fn check_line(
    tx: &Connection,
    errors: &mut FieldErrors,
    prefix: &str,
    item_id: Option<i64>,
    packing_id: Option<i64>,
    qty_text: &str,
    weight_text: Option<&String>,
    note_text: Option<&String>,
    other_text: Option<&String>,
    shop_text: Option<&String>,
) -> Result<Option<Line>, ServiceError> {
    let at = |field: &str| format!("{prefix}{field}");
    let item = match item_id {
        None => {
            errors.push(FieldError::new(ErrorCode::ErrRequired, &at("item_id")));
            None
        }
        Some(id) => match bikri_repo::items(tx, &[id])?.into_iter().next() {
            None => {
                errors.push(FieldError::new(ErrorCode::ErrNotFound, &at("item_id")));
                None
            }
            Some(item) if !item.is_active => {
                errors.push(FieldError::new(
                    ErrorCode::ErrAccountInactive,
                    &at("item_id"),
                ));
                None
            }
            Some(item) => Some(item),
        },
    };
    let packing = match packing_id {
        None => {
            errors.push(FieldError::new(ErrorCode::ErrRequired, &at("packing_id")));
            None
        }
        Some(id) => {
            let found = bikri_repo::existing_ids(tx, bikri_repo::MasterTable::Packings, &[id])?;
            if found.is_empty() {
                errors.push(FieldError::new(ErrorCode::ErrNotFound, &at("packing_id")));
                None
            } else {
                Some(id)
            }
        }
    };
    let qty = errors.take(validate::qty(&at("qty"), qty_text));
    let by_weight = item.as_ref().is_some_and(|i| i.sold_by == "weight");
    let weight = if by_weight {
        match weight_text.filter(|w| !w.trim().is_empty()) {
            Some(text) => errors.take(validate::weight(&at("weight"), text)),
            None => {
                errors.push(FieldError::new(ErrorCode::ErrWeightRequired, &at("weight")));
                None
            }
        }
    } else {
        None
    };
    let note = note_text
        .map(|t| t.trim())
        .filter(|t| !t.is_empty())
        .and_then(|t| errors.take(validate::free_text(&at("note"), t)))
        .filter(|t| !t.is_empty());
    let (other_qty, shop_name) = ventures::split_of(
        errors,
        prefix,
        other_text,
        shop_text,
        qty.map(mandi_domain::core::money::Qty::get),
    );
    Ok(match (item, packing, qty) {
        (Some(item), Some(packing_id), Some(qty)) => Some(Line {
            item_id: item.id,
            packing_id,
            qty: qty.get(),
            weight,
            note,
            other_qty,
            shop_name,
        }),
        _ => None,
    })
}

/// Writes one checked line as an arrival: the row, its received move and the audit row. Nothing is
/// posted. Used for a single arrival and for each line of an entry (PRD FR-12.41).
fn write_line(
    ctx: &Context,
    tx: &Connection,
    user: i64,
    now: &str,
    shared: &Shared,
    line: &Line,
) -> Result<i64, ServiceError> {
    let date_iso = shared.date.to_iso();
    let sell_iso = shared.sell_date.to_iso();
    let number = repo::next_number(tx)?;
    let id = repo::insert(
        tx,
        &NewArrival {
            venture_id: shared.garden.rec.id,
            entry_date: &date_iso,
            sell_date: &sell_iso,
            item_id: line.item_id,
            packing_id: line.packing_id,
            qty: line.qty,
            weight_grams: line.weight.map(Grams::get),
            note: line.note.as_deref(),
            other_qty: line.other_qty,
            shop_name: line.shop_name.as_deref(),
        },
        number,
        user,
        now,
    )?;
    // The arrival's own "received" move, so a cancel can reverse it.
    edit_repo::insert_move(
        tx,
        &NewMove {
            lot_id: None,
            purchase_id: None,
            arrival_id: Some(id),
            cost_paisa: 0,
            entry_id: None,
            move_date: &date_iso,
            kind: "received",
            qty: line.qty,
            reason: None,
            reverses_move_id: None,
        },
        user,
        now,
    )?;
    ctx.db.auditor().write(
        tx,
        &Event::new(Some(user), "create", "garden_arrival")
            .entity_id(id)
            .new_value(json!({
                "number": number,
                "garden_id": shared.garden.rec.id,
                "date": date_iso,
                "sell_date": sell_iso,
                "item_id": line.item_id,
                "packing_id": line.packing_id,
                "qty": line.qty,
                "other_qty": line.other_qty,
            }))
            .into_entry(),
    )?;
    Ok(id)
}

/// Saves a garden arrival (PRD FR-12.40).
///
/// # Errors
/// `ERR_REQUIRED` and `ERR_NOT_FOUND` on `garden_id`, `item_id`, `packing_id`; `ERR_GARDEN_CLOSED` on
/// `garden_id`; the date codes on `date`; `ERR_DATE_BEFORE_GARDEN` on `date`;
/// `ERR_SELL_DATE_BEFORE_ENTRY` on `sell_date`; the quantity and weight codes on `qty` and `weight`.
/// Nothing is written when any of them is returned.
pub fn save(
    ctx: &Context,
    actor: &Actor,
    input: &ArrivalInput,
) -> Result<ArrivalSaved, ServiceError> {
    let window = day::window(ctx)?;
    let (closed_up_to, today) = window;
    let now = ctx.now_text();
    ctx.db.write(|tx| {
        let mut errors = FieldErrors::default();
        let shared = check_shared(
            tx,
            window,
            &mut errors,
            input.garden_id,
            &input.date,
            input.sell_date.as_ref(),
        )?;
        let line = check_line(
            tx,
            &mut errors,
            "",
            input.item_id,
            input.packing_id,
            &input.qty,
            input.weight.as_ref(),
            input.note.as_ref(),
            None,
            None,
        )?;
        errors.finish()?;
        let (Some(shared), Some(line)) = (shared, line) else {
            return Err(ServiceError::reject(ErrorCode::ErrInternal));
        };
        let id = write_line(ctx, tx, actor.user_id, &now, &shared, &line)?;
        let rec =
            repo::get(tx, id)?.ok_or_else(|| data("the arrival is missing after the save"))?;
        let row = row_of(
            &rec,
            Vec::new(),
            Vec::new(),
            // A new arrival has no sale yet, so no commission.
            &mandi_db::commission::Commission::default(),
            closed_up_to,
            today,
        )?;
        Ok(ArrivalSaved {
            row,
            warnings: Vec::new(),
        })
    })
}

/// The checked parts of an entry: the shared fields and one checked line per input line.
type CheckedEntry = (Option<Shared>, Vec<Option<Line>>);

/// Checks every line of an entry. Mistakes go to `errors` at `lines[i].field`, the entry's own at
/// their paths, each once.
fn check_entry(
    tx: &Connection,
    window: (Option<BusinessDate>, BusinessDate),
    errors: &mut FieldErrors,
    input: &ArrivalEntryInput,
) -> Result<Option<CheckedEntry>, ServiceError> {
    if input.lines.is_empty() {
        errors.push(FieldError::new(ErrorCode::ErrRequired, "lines"));
    }
    if input.lines.len() > crate::purchases::MAX_ENTRY_LINES {
        errors.push(FieldError::new(ErrorCode::ErrTooManyLines, "lines").with(
            "max",
            ErrorValue::Number(
                i64::try_from(crate::purchases::MAX_ENTRY_LINES).unwrap_or(i64::MAX),
            ),
        ));
        return Ok(None);
    }
    let shared = check_shared(
        tx,
        window,
        errors,
        input.garden_id,
        &input.date,
        input.sell_date.as_ref(),
    )?;
    let mut lines = Vec::with_capacity(input.lines.len());
    for (i, line) in input.lines.iter().enumerate() {
        let checked = check_line(
            tx,
            errors,
            &format!("lines[{i}]."),
            line.item_id,
            line.packing_id,
            &line.qty,
            line.weight.as_ref(),
            line.note.as_ref(),
            line.other_qty.as_ref(),
            line.shop_name.as_ref(),
        )?;
        lines.push(checked);
    }
    Ok(Some((shared, lines)))
}

/// Works out a goods-arrived entry with many item lines (PRD FR-12.41, R2): only the mistakes, each
/// at `lines[i].field`. Nothing is saved.
///
/// # Errors
/// `ServiceError::Db` or `ServiceError::Data` for a failure the user cannot fix.
pub fn entry_preview(
    ctx: &Context,
    input: &ArrivalEntryInput,
) -> Result<ArrivalEntryPreview, ServiceError> {
    let window = day::window(ctx)?;
    ctx.db.read(|conn| {
        let mut errors = FieldErrors::default();
        let checked = check_entry(conn, window, &mut errors, input)?;
        let lines = input
            .lines
            .iter()
            .enumerate()
            .map(|(i, line)| {
                let one = checked
                    .as_ref()
                    .and_then(|(_, lines)| lines.get(i))
                    .and_then(Option::as_ref);
                ArrivalLinePreview {
                    key: line.key.clone(),
                    qty: one.map(|l| l.qty),
                    qty_our: one.and_then(|l| l.qty.checked_sub(l.other_qty)),
                }
            })
            .collect();
        Ok(ArrivalEntryPreview {
            lines,
            errors: errors.finish().err().unwrap_or_default(),
            warnings: Vec::new(),
        })
    })
}

/// Saves a goods-arrived entry with many item lines in ONE transaction (PRD FR-12.41, R2). Each line
/// is its own arrival with its own number. A mistake in any line saves nothing.
///
/// # Errors
/// Every mistake of the preview at its field. Nothing is written when any is returned.
pub fn entry_save(
    ctx: &Context,
    actor: &Actor,
    input: &ArrivalEntryInput,
) -> Result<ArrivalEntrySaved, ServiceError> {
    let window = day::window(ctx)?;
    let (closed_up_to, today) = window;
    let now = ctx.now_text();
    ctx.db.write(|tx| {
        let mut errors = FieldErrors::default();
        let checked = check_entry(tx, window, &mut errors, input)?;
        errors.finish()?;
        let Some((Some(shared), lines)) = checked else {
            return Err(ServiceError::reject(ErrorCode::ErrInternal));
        };
        let mut rows = Vec::with_capacity(lines.len());
        for line in lines.iter().flatten() {
            let id = write_line(ctx, tx, actor.user_id, &now, &shared, line)?;
            let rec =
                repo::get(tx, id)?.ok_or_else(|| data("the arrival is missing after the save"))?;
            rows.push(row_of(
                &rec,
                Vec::new(),
                Vec::new(),
                // A new arrival has no sale yet, so no commission.
                &mandi_db::commission::Commission::default(),
                closed_up_to,
                today,
            )?);
        }
        Ok(ArrivalEntrySaved {
            rows,
            warnings: Vec::new(),
        })
    })
}

/// Cancels an arrival with a reason, only while nothing of it is sold or written off (PRD
/// FR-12.40). Its received move is reversed.
///
/// # Errors
/// `ERR_NOT_FOUND`, `ERR_ARRIVAL_CANCELLED` and `ERR_ARRIVAL_HAS_SALES` (values `qty_sold`,
/// `qty_written_off`) on `id`; `ERR_VERSION_CHANGED` on `version`; `ERR_REASON_REQUIRED` on
/// `reason`; `ERR_DATE_CLOSED` on `id`.
pub fn cancel(
    ctx: &Context,
    actor: &Actor,
    input: &ArrivalCancelInput,
) -> Result<ArrivalCancelled, ServiceError> {
    let reason = required_reason(&input.reason)?;
    let now = ctx.now_text();
    ctx.db.write(|tx| {
        let rec = repo::get(tx, input.id)?.ok_or_else(|| not_found("id"))?;
        if rec.status == "cancelled" {
            return Err(ServiceError::reject_field(
                ErrorCode::ErrArrivalCancelled,
                "id",
            ));
        }
        if rec.version != input.version {
            return Err(ServiceError::reject_field(
                ErrorCode::ErrVersionChanged,
                "version",
            ));
        }
        if rec.qty_sold > 0 || rec.qty_written_off > 0 {
            return Err(ServiceError::Rejected(vec![Rejection {
                code: ErrorCode::ErrArrivalHasSales,
                field: Some("id".to_owned()),
                values: std::collections::BTreeMap::from([
                    ("qty_sold".to_owned(), ErrorValue::Number(rec.qty_sold)),
                    (
                        "qty_written_off".to_owned(),
                        ErrorValue::Number(rec.qty_written_off),
                    ),
                ]),
            }]));
        }
        let entry = BusinessDate::parse_iso(&rec.entry_date)
            .ok_or_else(|| data("the day of an arrival is not a date"))?;
        require_open_day(ctx, entry, "id")?;
        let user = actor.user_id;
        if let Some(received) = repo::received_move(tx, rec.id)? {
            edit_repo::insert_move(
                tx,
                &NewMove {
                    lot_id: None,
                    purchase_id: None,
                    arrival_id: Some(rec.id),
                    cost_paisa: 0,
                    entry_id: None,
                    move_date: &rec.entry_date,
                    kind: "reversal",
                    qty: rec
                        .qty
                        .checked_neg()
                        .ok_or_else(|| data("the quantity does not fit"))?,
                    reason: Some(&reason),
                    reverses_move_id: Some(received),
                },
                user,
                &now,
            )?;
        }
        repo::mark_cancelled(tx, rec.id, &reason, user, &now)?;
        ctx.db.auditor().write(
            tx,
            &Event::new(Some(user), "cancel", "garden_arrival")
                .entity_id(rec.id)
                .new_value(json!({ "number": rec.number, "qty": rec.qty }))
                .reason(&reason)
                .into_entry(),
        )?;
        Ok(ArrivalCancelled {
            id: rec.id,
            number: rec.number,
        })
    })
}

/// Changes the day to sell of an arrival with goods left (PRD FR-12.40). No money effect.
///
/// # Errors
/// `ERR_NOT_FOUND`, `ERR_ARRIVAL_CANCELLED`, `ERR_NOTHING_TO_SELL` on `id`; `ERR_VERSION_CHANGED` on
/// `version`; `ERR_SELL_DATE_BEFORE_ENTRY` and the date codes on `sell_date`.
pub fn set_sell_date(
    ctx: &Context,
    actor: &Actor,
    input: &SellDateInput,
) -> Result<SellDateSet, ServiceError> {
    let (_, today) = day::window(ctx)?;
    let now = ctx.now_text();
    ctx.db.write(|tx| {
        let rec = repo::get(tx, input.id)?.ok_or_else(|| not_found("id"))?;
        if rec.status == "cancelled" {
            return Err(ServiceError::reject_field(
                ErrorCode::ErrArrivalCancelled,
                "id",
            ));
        }
        if rec.version != input.version {
            return Err(ServiceError::reject_field(
                ErrorCode::ErrVersionChanged,
                "version",
            ));
        }
        if left_of(&rec)? == 0 {
            return Err(ServiceError::reject_field(
                ErrorCode::ErrNothingToSell,
                "id",
            ));
        }
        let entry = BusinessDate::parse_iso(&rec.entry_date)
            .ok_or_else(|| data("the day of an arrival is not a date"))?;
        let mut errors = FieldErrors::default();
        let sell = sell_date_of(&mut errors, Some(&input.sell_date), Some(entry), today);
        errors.finish()?;
        let sell = sell.ok_or_else(|| data("a sell date without errors is missing"))?;
        let sell_iso = sell.to_iso();
        let user = actor.user_id;
        repo::set_sell_date(tx, rec.id, &sell_iso, user, &now)?;
        ctx.db.auditor().write(
            tx,
            &Event::new(Some(user), "set_sell_date", "garden_arrival")
                .entity_id(rec.id)
                .old(json!({ "sell_date": rec.sell_date }))
                .new_value(json!({ "sell_date": sell_iso }))
                .into_entry(),
        )?;
        let after = repo::get(tx, rec.id)?.ok_or_else(|| data("the arrival is missing"))?;
        Ok(SellDateSet {
            id: after.id,
            version: after.version,
            sell_date: after.sell_date,
        })
    })
}

/// One page of arrivals with the totals over the whole filter. Read only.
///
/// # Errors
/// The date codes on `from` and `to`.
pub fn list(ctx: &Context, input: &ArrivalsListInput) -> Result<ArrivalsList, ServiceError> {
    let (closed_up_to, today) = day::window(ctx)?;
    let mut errors = FieldErrors::default();
    let mut day_of = |field: &str, text: Option<&String>| {
        let text = text.map(|t| t.trim()).filter(|t| !t.is_empty())?;
        errors
            .take(validate::date(field, text, today))
            .map(BusinessDate::to_iso)
    };
    let from = day_of("from", input.from.as_ref());
    let to = day_of("to", input.to.as_ref());
    errors.finish()?;
    let filter = ListFilter {
        garden_id: input.garden_id,
        from,
        to,
        item_id: input.item_id,
        state: input.state.map(|s| match s {
            ArrivalStateFilter::InStock => repo::StateFilter::InStock,
            ArrivalStateFilter::SoldOut => repo::StateFilter::SoldOut,
            ArrivalStateFilter::Cancelled => repo::StateFilter::Cancelled,
        }),
    };
    ctx.db.read(|conn| {
        let recs = repo::list(
            conn,
            &filter,
            i64::from(input.offset),
            crate::page::limit_of(input.limit),
        )?;
        let rows = rows_of(conn, &recs, closed_up_to, today)?;
        let sums = repo::totals(conn, &filter)?;
        let qty_left = sums
            .qty
            .checked_sub(sums.qty_sold)
            .and_then(|v| v.checked_sub(sums.qty_written_off))
            .ok_or_else(|| data("the quantity of the arrivals does not fit"))?;
        Ok(ArrivalsList {
            rows,
            total_count: sums.count,
            totals: ArrivalsTotals {
                qty: sums.qty,
                qty_sold: sums.qty_sold,
                qty_written_off: sums.qty_written_off,
                qty_left,
                commission_paisa: ventures::commission_view(&sums.commission)?.0,
            },
        })
    })
}

/// One arrival with its lots and write-offs. Read only.
///
/// # Errors
/// `ERR_NOT_FOUND` on `id`.
pub fn get(ctx: &Context, input: &ArrivalGetInput) -> Result<ArrivalRow, ServiceError> {
    let (closed_up_to, today) = day::window(ctx)?;
    ctx.db.read(|conn| {
        let rec = repo::get(conn, input.id)?.ok_or_else(|| not_found("id"))?;
        let mut rows = rows_of(conn, std::slice::from_ref(&rec), closed_up_to, today)?;
        rows.pop().ok_or_else(|| data("an arrival has no row"))
    })
}

/// Writes off spoiled goods of an arrival (PRD FR-12.40). Nothing is posted: an arrival has no cost.
///
/// # Errors
/// `ERR_NOT_FOUND` and `ERR_ARRIVAL_CANCELLED` on `arrival_id`; the quantity codes on `qty`;
/// `ERR_WRITE_OFF_EXCEEDS_BALANCE` (value `available`) on `qty`; `ERR_REASON_REQUIRED` on `reason`;
/// `ERR_DATE_CLOSED` on `date`.
pub fn write_off(
    ctx: &Context,
    actor: &Actor,
    input: &ArrivalWriteOffInput,
) -> Result<ArrivalWriteOffDone, ServiceError> {
    let (_, today) = day::window(ctx)?;
    let mut errors = FieldErrors::default();
    let qty = errors.take(validate::qty("qty", &input.qty));
    let reason = errors.take(validate::free_text("reason", &input.reason));
    if reason.as_ref().is_some_and(String::is_empty) {
        errors.push(FieldError::new(ErrorCode::ErrReasonRequired, "reason"));
    }
    errors.finish()?;
    let (Some(qty), Some(reason)) = (qty, reason) else {
        return Err(ServiceError::reject_field(
            ErrorCode::ErrReasonRequired,
            "reason",
        ));
    };
    require_open_day(ctx, today, "date")?;
    let now = ctx.now_text();
    ctx.db.write(|tx| {
        let rec = repo::get(tx, input.arrival_id)?.ok_or_else(|| not_found("arrival_id"))?;
        if rec.status == "cancelled" {
            return Err(ServiceError::reject_field(
                ErrorCode::ErrArrivalCancelled,
                "arrival_id",
            ));
        }
        let qty_left = left_of(&rec)?;
        if qty.get() > qty_left {
            return Err(ServiceError::Rejected(vec![Rejection {
                code: ErrorCode::ErrWriteOffExceedsBalance,
                field: Some("qty".to_owned()),
                values: std::collections::BTreeMap::from([(
                    "available".to_owned(),
                    ErrorValue::Number(qty_left),
                )]),
            }]));
        }
        let user = actor.user_id;
        let date_iso = today.to_iso();
        let neg_qty = qty
            .get()
            .checked_neg()
            .ok_or_else(|| data("the quantity does not fit"))?;
        let move_id = edit_repo::insert_move(
            tx,
            &NewMove {
                lot_id: None,
                purchase_id: None,
                arrival_id: Some(rec.id),
                cost_paisa: 0,
                entry_id: None,
                move_date: &date_iso,
                kind: "written_off",
                qty: neg_qty,
                reason: Some(&reason),
                reverses_move_id: None,
            },
            user,
            &now,
        )?;
        repo::add_written_off(tx, rec.id, qty.get(), user, &now)?;
        ctx.db.auditor().write(
            tx,
            &Event::new(Some(user), "write_off", "garden_arrival")
                .entity_id(rec.id)
                .new_value(json!({
                    "number": rec.number,
                    "move_id": move_id,
                    "qty": qty.get(),
                    "date": date_iso,
                }))
                .reason(&reason)
                .into_entry(),
        )?;
        Ok(ArrivalWriteOffDone {
            arrival_id: rec.id,
            move_id,
            qty: qty.get(),
            qty_written_off: rec
                .qty_written_off
                .checked_add(qty.get())
                .ok_or_else(|| data("the quantity does not fit"))?,
            qty_left: qty_left
                .checked_sub(qty.get())
                .ok_or_else(|| data("the quantity does not fit"))?,
        })
    })
}

/// Undoes a write-off of an arrival, on a day that is still open (PRD FR-12.40). A closed garden
/// takes no goods back (`ERR_GARDEN_CLOSED`): its arrivals are sold or written off for good.
///
/// # Errors
/// `ERR_NOT_FOUND` on `move_id` (not a write-off of an arrival, or already undone);
/// `ERR_ARRIVAL_CANCELLED` and `ERR_GARDEN_CLOSED` on `move_id`; `ERR_REASON_REQUIRED` on `reason`;
/// `ERR_DATE_CLOSED` on `move_id`.
pub fn write_off_undo(
    ctx: &Context,
    actor: &Actor,
    input: &ArrivalWriteOffUndoInput,
) -> Result<ArrivalWriteOffUndone, ServiceError> {
    let reason = required_reason(&input.reason)?;
    let now = ctx.now_text();
    ctx.db.write(|tx| {
        let off = repo::write_off(tx, input.move_id)?.ok_or_else(|| not_found("move_id"))?;
        let rec = repo::get(tx, off.arrival_id)?
            .ok_or_else(|| data("the arrival of a write-off is missing"))?;
        if rec.status == "cancelled" {
            return Err(ServiceError::reject_field(
                ErrorCode::ErrArrivalCancelled,
                "move_id",
            ));
        }
        if rec.garden_status == "closed" {
            return Err(ServiceError::reject_field(
                ErrorCode::ErrGardenClosed,
                "move_id",
            ));
        }
        let date = BusinessDate::parse_iso(&off.move_date)
            .ok_or_else(|| data("the date of a stock move is not a date"))?;
        require_open_day(ctx, date, "move_id")?;
        let user = actor.user_id;
        let reversal_move_id = edit_repo::insert_move(
            tx,
            &NewMove {
                lot_id: None,
                purchase_id: None,
                arrival_id: Some(rec.id),
                cost_paisa: 0,
                entry_id: None,
                move_date: &off.move_date,
                kind: "reversal",
                qty: off.qty,
                reason: Some(&reason),
                reverses_move_id: Some(off.move_id),
            },
            user,
            &now,
        )?;
        let back = off
            .qty
            .checked_neg()
            .ok_or_else(|| data("the quantity does not fit"))?;
        repo::add_written_off(tx, rec.id, back, user, &now)?;
        ctx.db.auditor().write(
            tx,
            &Event::new(Some(user), "write_off_undo", "garden_arrival")
                .entity_id(rec.id)
                .new_value(json!({
                    "number": rec.number,
                    "move_id": off.move_id,
                    "reversal_move_id": reversal_move_id,
                    "qty": off.qty,
                }))
                .reason(&reason)
                .into_entry(),
        )?;
        let after = repo::get(tx, rec.id)?.ok_or_else(|| data("the arrival is missing"))?;
        Ok(ArrivalWriteOffUndone {
            arrival_id: after.id,
            reversal_move_id,
            qty_written_off: after.qty_written_off,
            qty_left: left_of(&after)?,
        })
    })
}

/// The arrival gives up what a garden lot sold: `qty` نگ (PRD FR-12.40). The table checks that sold
/// plus written off is never more than arrived.
///
/// # Errors
/// `ServiceError::Db`.
pub(crate) fn take_sold(
    tx: &Connection,
    arrival_id: i64,
    qty: i64,
    user_id: i64,
    now: &str,
) -> Result<(), ServiceError> {
    repo::add_sold(tx, arrival_id, qty, user_id, now)?;
    Ok(())
}

/// Gives back to the arrivals exactly what the lines in force of a posting took, before the posting
/// is reversed or replaced (PRD FR-12.40).
///
/// # Errors
/// `ServiceError::Db`, `ServiceError::Data`.
pub(crate) fn give_back_entry(
    tx: &Connection,
    entry_id: i64,
    user_id: i64,
    now: &str,
) -> Result<(), ServiceError> {
    for taken in edit_repo::arrival_release_of_entry(tx, entry_id)? {
        let qty = taken
            .qty
            .checked_neg()
            .ok_or_else(|| data("the quantity does not fit"))?;
        repo::add_sold(tx, taken.arrival_id, qty, user_id, now)?;
    }
    Ok(())
}

/// The saved arrivals with goods left, for the باقی مال list (PRD FR-12.40), as a map by id is not
/// needed: the rows keep the order oldest arrival first, then number.
pub(crate) fn stock_rows(
    conn: &Connection,
    from: Option<&str>,
    to: Option<&str>,
    item_id: Option<i64>,
    account_id: Option<i64>,
    today: BusinessDate,
) -> Result<Vec<mandi_domain::core::bikri::GardenStockRow>, ServiceError> {
    let recs = repo::with_stock(conn, from, to, item_id, account_id)?;
    let ids: Vec<i64> = recs.iter().map(|r| r.id).collect();
    let mut offs: HashMap<i64, Vec<ArrivalWriteOff>> = repo::write_offs_of(conn, &ids)?;
    recs.iter()
        .map(|rec| {
            let balance = left_of(rec)?;
            let parts = parts_of(rec)?;
            let sell = BusinessDate::parse_iso(&rec.sell_date)
                .ok_or_else(|| data("the day to sell of an arrival is not a date"))?;
            Ok(mandi_domain::core::bikri::GardenStockRow {
                arrival_id: rec.id,
                number: rec.number,
                received_date: rec.entry_date.clone(),
                sell_date: rec.sell_date.clone(),
                garden: garden_ref(rec),
                item: rec.item.clone(),
                packing: rec.packing.clone(),
                qty_arrived: rec.qty,
                qty_sold: rec.qty_sold,
                qty_written_off: rec.qty_written_off,
                balance,
                qty_our_left: parts.our_left,
                qty_other_left: parts.other_left,
                shop_name: rec.shop_name.clone(),
                days_waiting: today.days_after(sell).max(0),
                write_offs: write_off_rows(offs.remove(&rec.id).unwrap_or_default()),
            })
        })
        .collect()
}

/// Changes the split of an arrival between our shop and another shop, while goods are left (PRD
/// FR-12.41, R3). No money effect.
///
/// # Errors
/// `ERR_NOT_FOUND`, `ERR_ARRIVAL_CANCELLED`, `ERR_NOTHING_TO_SELL` on `id`; `ERR_VERSION_CHANGED` on
/// `version`; the quantity codes and `ERR_SPLIT_ABOVE_QTY` (value `qty`) on `other_qty`;
/// `ERR_SPLIT_BELOW_SOLD` (values `min_other`, `max_other`) on `other_qty`.
pub fn change_split(
    ctx: &Context,
    actor: &Actor,
    input: &mandi_domain::core::split::SplitInput,
) -> Result<mandi_domain::core::split::SplitSet, ServiceError> {
    use mandi_domain::core::split::SplitSet;
    let now = ctx.now_text();
    ctx.db.write(|tx| {
        let rec = repo::get(tx, input.id)?.ok_or_else(|| not_found("id"))?;
        if rec.status == "cancelled" {
            return Err(ServiceError::reject_field(
                ErrorCode::ErrArrivalCancelled,
                "id",
            ));
        }
        if rec.version != input.version {
            return Err(ServiceError::reject_field(
                ErrorCode::ErrVersionChanged,
                "version",
            ));
        }
        if parts_of(&rec)?.total_left == 0 {
            return Err(ServiceError::reject_field(
                ErrorCode::ErrNothingToSell,
                "id",
            ));
        }
        let mut errors = FieldErrors::default();
        let (other, shop) = ventures::split_of(
            &mut errors,
            "",
            Some(&input.other_qty),
            input.shop_name.as_ref(),
            Some(rec.qty),
        );
        errors.finish()?;
        let (min, max) = StockParts::other_range(rec.qty, rec.qty_sold, rec.qty_sold_other)
            .ok_or_else(|| data("the split of an arrival does not fit"))?;
        ventures::check_split_range(other, min, max)?;
        let user = actor.user_id;
        repo::set_split(tx, rec.id, other, shop.as_deref(), user, &now)?;
        ctx.db.auditor().write(
            tx,
            &Event::new(Some(user), "change_split", "garden_arrival")
                .entity_id(rec.id)
                .old(json!({ "other_qty": rec.other_qty, "shop_name": rec.shop_name }))
                .new_value(json!({
                    "number": rec.number,
                    "other_qty": other,
                    "shop_name": shop,
                }))
                .into_entry(),
        )?;
        let after = repo::get(tx, rec.id)?.ok_or_else(|| data("the arrival is missing"))?;
        let parts = parts_of(&after)?;
        Ok(SplitSet {
            id: after.id,
            version: after.version,
            qty_other: after.other_qty,
            shop_name: after.shop_name,
            qty_our_left: parts.our_left,
            qty_other_left: parts.other_left,
        })
    })
}
