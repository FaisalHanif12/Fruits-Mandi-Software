//! Making a trip and changing its partners before its first entry (PRD 12.10, FR-12.29): the
//! partners and their shares first, then the money source of each, then the details. One action is
//! one transaction with its audit row (Code Standards B3, S7).

use std::collections::{BTreeSet, HashSet};

use mandi_db::rusqlite::Connection;
use mandi_db::ventures::{self as repo, NewVenture};
use mandi_domain::core::dates::format_ddmmyyyy;
use mandi_domain::core::error::{ErrorCode, ErrorValue, FieldError};
use mandi_domain::core::money::{BPS_FULL, Bps};
use mandi_domain::core::text;
use mandi_domain::core::trips::{TripInput, TripRow, TripSetPartnersInput};
use mandi_domain::core::validate::{self, FieldErrors};
use mandi_domain::core::ventures::{MoneySource, VenturePartnerInput};
use serde_json::json;

use super::read::row_by_id;
use crate::audit::Event;
use crate::auth::Actor;
use crate::context::Context;
use crate::day;
use crate::error::ServiceError;
use crate::ventures::not_found;

/// A partner of a new trip after the checks.
struct Parsed {
    partner_id: i64,
    share: Bps,
    source: MoneySource,
}

/// An optional free text: empty is none.
fn optional_text(errors: &mut FieldErrors, field: &str, text: Option<&str>) -> Option<String> {
    let text = text.map(str::trim).filter(|t| !t.is_empty())?;
    errors
        .take(validate::free_text(field, text))
        .filter(|t| !t.is_empty())
}

/// Reads the partners of a trip as typed (PRD FR-12.29): each share with the percent reader, rows with
/// a share of 0 skipped, every other partner active and listed once, and the shares totalling exactly
/// 100%. The mistakes go to `errors`, each at its field.
fn read_partners(
    conn: &Connection,
    errors: &mut Vec<FieldError>,
    rows: &[VenturePartnerInput],
) -> Result<Vec<Parsed>, ServiceError> {
    let active: HashSet<i64> = repo::active_partner_ids(conn)?.into_iter().collect();
    let mut seen = BTreeSet::new();
    let mut out = Vec::with_capacity(rows.len());
    let mut total: i64 = 0;
    for (index, row) in rows.iter().enumerate() {
        let share = match validate::percent(&format!("partners[{index}].share"), &row.share) {
            Ok(share) => share,
            Err(error) => {
                errors.push(error);
                continue;
            }
        };
        if share.get() == 0 {
            continue;
        }
        if !active.contains(&row.partner_id) || !seen.insert(row.partner_id) {
            errors.push(FieldError::new(
                ErrorCode::ErrPartnerNotListed,
                &format!("partners[{index}].partner_id"),
            ));
            continue;
        }
        total = total.saturating_add(i64::from(share.get()));
        out.push(Parsed {
            partner_id: row.partner_id,
            share,
            source: row.source,
        });
    }
    if total != i64::from(BPS_FULL) {
        errors.push(
            FieldError::new(ErrorCode::ErrSharesNot100, "partners")
                .with("total_bps", ErrorValue::Number(total)),
        );
    }
    out.sort_by_key(|p| p.partner_id);
    Ok(out)
}

fn write_partners(
    tx: &Connection,
    trip_id: i64,
    partners: &[Parsed],
    user_id: i64,
    now: &str,
) -> Result<(), ServiceError> {
    for p in partners {
        repo::insert_partner(
            tx,
            trip_id,
            p.partner_id,
            i64::from(p.share.get()),
            p.source.as_str(),
            user_id,
            now,
        )?;
    }
    Ok(())
}

fn partners_json(partners: &[Parsed]) -> serde_json::Value {
    json!(
        partners
            .iter()
            .map(|p| json!({
                "partner_id": p.partner_id,
                "share_bps": p.share.get(),
                "source": p.source.as_str(),
            }))
            .collect::<Vec<_>>()
    )
}

/// Makes a trip with its partners, shares and default sources (PRD FR-12.29). The trip gets the next
/// number of its series, and names made from the number and the date for the search.
///
/// Example: trip 1 on 10-10-2026 is named "سفر-1 10-10-2026" and "TRP-1 10-10-2026".
///
/// # Errors
/// The date codes on `date`; the text codes on `market`, `sent_person` and `note`; the percent codes
/// on `partners[i].share`; `ERR_PARTNER_NOT_LISTED` on `partners[i].partner_id`; `ERR_SHARES_NOT_100`
/// on `partners` (value `total_bps`). All the problems found together come back together.
pub(super) fn create(
    ctx: &Context,
    actor: &Actor,
    input: &TripInput,
) -> Result<TripRow, ServiceError> {
    let (closed_up_to, today) = day::window(ctx)?;
    let now = ctx.now_text();
    ctx.db.write(|tx| {
        let mut errors = FieldErrors::default();
        let date = errors.take(validate::posting_date(
            "date",
            &input.date,
            closed_up_to,
            today,
        ));
        let market = optional_text(&mut errors, "market", input.market.as_deref());
        let sent_person = optional_text(&mut errors, "sent_person", input.sent_person.as_deref());
        let note = optional_text(&mut errors, "note", input.note.as_deref());
        let mut found = errors.finish().err().unwrap_or_default();
        let partners = read_partners(tx, &mut found, &input.partners)?;
        let Some(date) = date.filter(|_| found.is_empty()) else {
            return Err(ServiceError::from(found));
        };
        let user = actor.user_id;
        let number = repo::next_number(tx, "trip")?;
        let day_text = format_ddmmyyyy(date);
        let name_ur = format!("سفر-{number} {day_text}");
        let name_en = format!("TRP-{number} {day_text}");
        let id = repo::insert(
            tx,
            &NewVenture {
                kind: "trip",
                number,
                entry_date: &date.to_iso(),
                name_ur: &name_ur,
                name_en: &name_en,
                search_ur: &text::search_ur(&name_ur),
                search_en: &text::search_en(&name_en),
                search_phonetic: &text::search_phonetic(&name_en),
                market: market.as_deref(),
                sent_person: sent_person.as_deref(),
                note: note.as_deref(),
            },
            user,
            &now,
        )?;
        write_partners(tx, id, &partners, user, &now)?;
        ctx.db.auditor().write(
            tx,
            &Event::new(Some(user), "create", "trip")
                .entity_id(id)
                .new_value(json!({
                    "number": number,
                    "entry_date": date.to_iso(),
                    "market": market,
                    "sent_person": sent_person,
                    "partners": partners_json(&partners),
                }))
                .into_entry(),
        )?;
        row_by_id(tx, id)
    })
}

/// Changes the partners and shares of a trip that has no entry yet (PRD FR-12.29). After its first
/// entry they are fixed.
///
/// # Errors
/// As [`create`] for the partners; `ERR_NOT_FOUND` on `id`; `ERR_VERSION_CHANGED` on `version`;
/// `ERR_VENTURE_SHARES_LOCKED` on `partners` once the trip has an entry.
pub(super) fn set_partners(
    ctx: &Context,
    actor: &Actor,
    input: &TripSetPartnersInput,
) -> Result<TripRow, ServiceError> {
    let now = ctx.now_text();
    ctx.db.write(|tx| {
        let rec = repo::get(tx, input.id)?
            .filter(|v| v.kind == "trip")
            .ok_or_else(|| not_found("id"))?;
        if rec.version != input.version {
            return Err(ServiceError::reject_field(
                ErrorCode::ErrVersionChanged,
                "version",
            ));
        }
        if repo::has_entries(tx, rec.id)? {
            return Err(ServiceError::reject_field(
                ErrorCode::ErrVentureSharesLocked,
                "partners",
            ));
        }
        let mut found = Vec::new();
        let partners = read_partners(tx, &mut found, &input.partners)?;
        if !found.is_empty() {
            return Err(ServiceError::from(found));
        }
        let old = repo::partners_of(tx, rec.id)?;
        repo::delete_partners(tx, rec.id)?;
        write_partners(tx, rec.id, &partners, actor.user_id, &now)?;
        repo::touch(tx, rec.id, actor.user_id, &now)?;
        ctx.db.auditor().write(
            tx,
            &Event::new(Some(actor.user_id), "edit", "trip")
                .entity_id(rec.id)
                .old(json!({
                    "partners": old
                        .iter()
                        .map(|p| json!({
                            "partner_id": p.partner_id,
                            "share_bps": p.share_bps,
                            "source": p.source,
                        }))
                        .collect::<Vec<_>>(),
                }))
                .new_value(json!({ "partners": partners_json(&partners) }))
                .into_entry(),
        )?;
        row_by_id(tx, rec.id)
    })
}
