//! The edit of a trip's header (PRD FR-7.70, FR-12.29): the market, the person sent and the note.
//! The date, the name (made from the number and the day) and the shares never change, and the edit
//! posts nothing, so there is no reversal and no edit window.
//!
//! Example: the market "Lahore" is edited to "Multan": one change, `market`, nothing else moves.

use mandi_db::ventures as repo;
use mandi_domain::core::dates::BusinessDate;
use mandi_domain::core::edit::TripHeaderEditInput;
use mandi_domain::core::error::ErrorCode;
use mandi_domain::core::validate::{self, FieldErrors};
use serde_json::{Value, json};

use crate::audit::Event;
use crate::auth::Actor;
use crate::context::Context;
use crate::edit::entries::Edited;
use crate::edit::{date_fixed, reason_for};
use crate::error::ServiceError;
use crate::ventures::{not_found, read_partners};

fn data(message: &str) -> ServiceError {
    ServiceError::Data(message.to_owned())
}

/// An optional free text: empty is none.
fn optional_text(errors: &mut FieldErrors, field: &str, text: Option<&str>) -> Option<String> {
    let text = text.map(str::trim).filter(|t| !t.is_empty())?;
    errors
        .take(validate::free_text(field, text))
        .filter(|t| !t.is_empty())
}

/// A trip's header as the change list of an edit compares it.
///
/// # Errors
/// `ERR_NOT_FOUND` on `id`; `ServiceError::Db`.
pub(crate) fn row_value(
    conn: &mandi_db::rusqlite::Connection,
    id: i64,
) -> Result<Value, ServiceError> {
    let trip = repo::get(conn, id)?
        .filter(|v| v.kind == "trip")
        .ok_or_else(|| not_found("id"))?;
    Ok(json!({
        "market": trip.market,
        "sent_person": trip.sent_person,
        "note": trip.note,
    }))
}

/// The edit inside the transaction of the caller: the save and the rehearsal of the review are this
/// one function (Code Standards B5).
///
/// # Errors
/// `ERR_NOT_FOUND` on `id`; `ERR_VERSION_CHANGED` on `version`; `ERR_EDIT_DATE_FIXED` on `date`;
/// `ERR_REASON_REQUIRED` on `reason`; `ERR_VENTURE_SHARES_LOCKED` on `partners` when the shares are
/// not the trip's own; the text codes on `market`, `sent_person` and `note`. Nothing is written when
/// any of them is returned.
pub(crate) fn edit_in(
    ctx: &Context,
    actor: &Actor,
    tx: &mandi_db::rusqlite::Connection,
    window: (Option<BusinessDate>, BusinessDate),
    input: &TripHeaderEditInput,
) -> Result<Edited, ServiceError> {
    let trip = repo::get(tx, input.id)?
        .filter(|v| v.kind == "trip")
        .ok_or_else(|| not_found("id"))?;
    if trip.version != input.version {
        return Err(ServiceError::reject_field(
            ErrorCode::ErrVersionChanged,
            "version",
        ));
    }
    let old_date = BusinessDate::parse_iso(&trip.entry_date)
        .ok_or_else(|| data("the date of a trip is not a date"))?;
    let typed = &input.trip;
    if validate::date("date", &typed.date, window.1).is_ok_and(|d| d != old_date) {
        return Err(date_fixed("date", old_date));
    }
    let reason = reason_for(window.1, old_date, &input.reason)?;
    let mut errors = FieldErrors::default();
    let market = optional_text(&mut errors, "market", typed.market.as_deref());
    let sent_person = optional_text(&mut errors, "sent_person", typed.sent_person.as_deref());
    let note = optional_text(&mut errors, "note", typed.note.as_deref());
    let mut found = errors.finish().err().unwrap_or_default();
    // The partners and their shares are the trip's own (PRD FR-12.29).
    let parsed = read_partners(tx, &mut found, &typed.partners)?;
    if !found.is_empty() {
        return Err(ServiceError::from(found));
    }
    let own = repo::partners_of(tx, trip.id)?;
    let same = own.len() == parsed.len()
        && own.iter().all(|o| {
            parsed
                .iter()
                .any(|p| p.partner_id == o.partner_id && i64::from(p.share.get()) == o.share_bps)
        });
    if !same {
        return Err(ServiceError::reject_field(
            ErrorCode::ErrVentureSharesLocked,
            "partners",
        ));
    }
    let user = actor.user_id;
    let now = ctx.now_text();
    let old_value = row_value(tx, trip.id)?;
    repo::update_trip_header(
        tx,
        trip.id,
        market.as_deref(),
        sent_person.as_deref(),
        note.as_deref(),
        user,
        &now,
    )?;
    let mut event = Event::new(Some(user), "edit", "trip")
        .entity_id(trip.id)
        .old(json!({ "number": trip.number, "row": old_value }))
        .new_value(json!({ "number": trip.number, "row": row_value(tx, trip.id)? }));
    if let Some(reason) = &reason {
        event = event.reason(reason);
    }
    event = event.sign(crate::day::fix_sign(tx, &trip.entry_date)?.as_deref());
    ctx.db.auditor().write(tx, &event.into_entry())?;
    let version = repo::get(tx, trip.id)?
        .map(|v| v.version)
        .ok_or_else(|| data("a trip is missing after its edit"))?;
    Ok(Edited {
        id: trip.id,
        number: Some(trip.number),
        version,
        entry_id: None,
        reversal_entry_id: None,
    })
}
