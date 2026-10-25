//! Reading fruit entries (PRD FR-16.12, FR-19.11, plan 17d): one entry, one page of entries with
//! the totals over the whole filter, and the lots fruit can be taken from. Read only.

use mandi_db::fruit::{self as frepo, FruitRec, ListFilter};
use mandi_db::rusqlite::Connection;
use mandi_domain::core::bikri::SavedPukhta;
use mandi_domain::core::dates::BusinessDate;
use mandi_domain::core::error::ErrorCode;
use mandi_domain::core::fruit::{
    FruitGetInput, FruitList, FruitListInput, FruitLotRef, FruitLotRow, FruitLots, FruitLotsInput,
    FruitPartRow, FruitRow, FruitSource, FruitStatus, FruitTotals,
};
use mandi_domain::core::money::Paisa;
use mandi_domain::core::validate::{self, FieldErrors};

use super::act::lot_of_closed_garden;
use crate::context::Context;
use crate::day;
use crate::error::{ServiceError, money_error};
use crate::page::limit_of;
use crate::ventures::not_found;

fn data(message: &str) -> ServiceError {
    ServiceError::Data(message.to_owned())
}

fn paisa(value: i64) -> Result<Paisa, ServiceError> {
    Paisa::try_from_paisa(value).map_err(money_error)
}

fn saved_pukhta(tuple: Option<&(String, String, String, String)>) -> Option<SavedPukhta> {
    tuple.map(|(name_ur, name_en, phone, key)| SavedPukhta {
        name_ur: name_ur.clone(),
        name_en: name_en.clone(),
        phone: phone.clone(),
        person_key: key.clone(),
    })
}

fn row_of(
    conn: &Connection,
    rec: &FruitRec,
    closed_up_to: Option<BusinessDate>,
) -> Result<FruitRow, ServiceError> {
    // The parts of the posting in force (a cancelled entry shows the posting it had).
    let parts = match rec.current_entry_id {
        Some(entry) => frepo::parts_of_entry(conn, entry)?,
        None => Vec::new(),
    };
    let mut part_rows = Vec::with_capacity(parts.len());
    for part in parts {
        part_rows.push(FruitPartRow {
            partner: part.partner,
            amount_paisa: paisa(part.amount_paisa)?,
        });
    }
    let date = BusinessDate::parse_iso(&rec.entry_date)
        .ok_or_else(|| data("the date of a fruit entry is not a date"))?;
    let status = FruitStatus::parse(&rec.status)
        .ok_or_else(|| data("a fruit entry has a status that is not saved or cancelled"))?;
    let source = FruitSource::parse(&rec.source)
        .ok_or_else(|| data("a fruit entry has a source that is not shop or other_shop"))?;
    let day_is_open = closed_up_to.is_none_or(|closed| date > closed);
    let can_change =
        status == FruitStatus::Saved && day_is_open && !lot_of_closed_garden(conn, rec)?;
    Ok(FruitRow {
        id: rec.id,
        version: rec.version,
        number: rec.number,
        entry_date: rec.entry_date.clone(),
        partner: rec.partner.clone(),
        source,
        lot: rec
            .lot
            .map(|(lot_id, lot_no, bikri_id, bikri_number)| FruitLotRef {
                lot_id,
                lot_no,
                bikri_id,
                bikri_number,
            }),
        credit_account: rec.credit.clone(),
        pukhta: saved_pukhta(rec.pukhta.as_ref()),
        item: rec.item.clone(),
        packing: rec.packing.clone(),
        qty: rec.qty,
        weight_grams: rec.weight_grams,
        rate_paisa: paisa(rec.rate_paisa)?,
        value_paisa: paisa(rec.value_paisa)?,
        note: rec.note.clone(),
        status,
        cancel_reason: rec.cancel_reason.clone(),
        parts: part_rows,
        journal_entry_id: if status == FruitStatus::Saved {
            rec.current_entry_id
        } else {
            None
        },
        can_change,
    })
}

/// One entry. Read only.
pub(super) fn get(ctx: &Context, input: &FruitGetInput) -> Result<FruitRow, ServiceError> {
    let (closed_up_to, _) = day::window(ctx)?;
    ctx.db.read(|conn| {
        let rec = frepo::get(conn, input.id)?.ok_or_else(|| not_found("id"))?;
        row_of(conn, &rec, closed_up_to)
    })
}

fn optional_date(
    errors: &mut FieldErrors,
    field: &str,
    text: Option<&String>,
    today: BusinessDate,
) -> Option<String> {
    let text = text.map(|t| t.trim()).filter(|t| !t.is_empty())?;
    errors
        .take(validate::date(field, text, today))
        .map(BusinessDate::to_iso)
}

/// One page of entries with the totals over the whole filter (B31). Read only.
pub(super) fn list(ctx: &Context, input: &FruitListInput) -> Result<FruitList, ServiceError> {
    let (closed_up_to, today) = day::window(ctx)?;
    let mut errors = FieldErrors::default();
    let from = optional_date(&mut errors, "from", input.from.as_ref(), today);
    let to = optional_date(&mut errors, "to", input.to.as_ref(), today);
    if input.guests_only && input.partner_id.is_some() {
        errors.push(mandi_domain::core::error::FieldError::new(
            ErrorCode::ErrFormatInvalid,
            "partner_id",
        ));
    }
    errors.finish()?;
    let filter = ListFilter {
        partner_id: input.partner_id,
        guests_only: input.guests_only,
        item_id: input.item_id,
        source: input.source.map(|s| s.as_str().to_owned()),
        from,
        to,
        status: input.status.map(|s| s.as_str().to_owned()),
    };
    ctx.db.read(|conn| {
        let (count, qty, value) = frepo::totals(conn, &filter)?;
        let recs = frepo::list(
            conn,
            &filter,
            i64::from(input.offset),
            limit_of(input.limit),
        )?;
        let mut rows = Vec::with_capacity(recs.len());
        for rec in &recs {
            rows.push(row_of(conn, rec, closed_up_to)?);
        }
        Ok(FruitList {
            rows,
            total_count: count,
            totals: FruitTotals {
                qty,
                value_paisa: paisa(value)?,
            },
        })
    })
}

/// The lots fruit can be taken from. Read only.
pub(super) fn lots(ctx: &Context, input: &FruitLotsInput) -> Result<FruitLots, ServiceError> {
    let search = input
        .search
        .as_ref()
        .map(|t| t.trim().to_lowercase())
        .filter(|t| !t.is_empty());
    ctx.db.read(|conn| {
        let (recs, total) = frepo::lots(conn, search.as_deref(), limit_of(input.limit))?;
        Ok(FruitLots {
            rows: recs
                .into_iter()
                .map(|rec| FruitLotRow {
                    lot_id: rec.lot_id,
                    lot_no: rec.lot_no,
                    bikri_id: rec.bikri_id,
                    bikri_number: rec.bikri_number,
                    received_date: rec.received_date,
                    seller: rec.seller,
                    pukhta: saved_pukhta(rec.pukhta.as_ref()),
                    item: rec.item,
                    packing: rec.packing,
                    sold_by_weight: rec.sold_by_weight,
                    balance: rec.balance,
                })
                .collect(),
            total_count: total,
        })
    })
}
