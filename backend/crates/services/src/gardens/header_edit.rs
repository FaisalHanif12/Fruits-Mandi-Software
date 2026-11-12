//! The edit of a garden's header (PRD FR-7.70, FR-12.34): the names, the main item, the rent, the end
//! of the season and the note. The date, the account, the code and the shares never change. The edit
//! posts nothing: the rent is owed by the garden's partners, not by the shop (FR-12.34). The rent is
//! changed inside the garden's edit window (migration 12). If the names change, the garden's account
//! gets them too, and the search index is refreshed after the commit.
//!
//! Example: rent 50,000 with 30,000 paid is edited to 60,000: the rent still due goes from 20,000 to
//! 30,000 and nothing is posted.

use mandi_db::edit_windows::{self, Window};
use mandi_db::gardens as repo;
use mandi_db::rusqlite::Connection;
use mandi_db::ventures::{self as vrepo, GardenHeaderEdit};
use mandi_domain::core::dates::BusinessDate;
use mandi_domain::core::edit::GardenHeaderEditInput;
use mandi_domain::core::error::{ErrorCode, ErrorValue, FieldError};
use mandi_domain::core::masters::AccountToMake;
use mandi_domain::core::money::Paisa;
use mandi_domain::core::text;
use mandi_domain::core::validate::{self, FieldErrors};
use serde_json::{Value, json};

use super::data;
use crate::audit::Event;
use crate::auth::Actor;
use crate::context::Context;
use crate::edit::entries::Edited;
use crate::edit::{date_fixed, reason_for};
use crate::error::ServiceError;
use crate::spelling::SpellingCheck;
use crate::ventures::{not_found, read_partners};

/// An optional free text: empty is none.
fn optional_text(errors: &mut FieldErrors, field: &str, text: Option<&str>) -> Option<String> {
    let text = text.map(str::trim).filter(|t| !t.is_empty())?;
    errors
        .take(validate::free_text(field, text))
        .filter(|t| !t.is_empty())
}

/// A garden's header as the change list of an edit compares it.
///
/// # Errors
/// `ERR_NOT_FOUND` on `id`; `ServiceError::Db`.
pub(crate) fn row_value(conn: &Connection, id: i64) -> Result<Value, ServiceError> {
    let garden = repo::get(conn, id)?.ok_or_else(|| not_found("id"))?;
    Ok(json!({
        "name": { "name_ur": garden.name_ur, "name_en": garden.name_en },
        "item": garden.item.as_ref().map(|i| json!({ "name_ur": i.name_ur, "name_en": i.name_en })),
        "rent_paisa": garden.rent_paisa,
        "season_end_date": garden.season_end_date,
        "note": garden.note,
    }))
}

/// The names the edit asks for, when they differ from the garden's own. The spelling check of a new
/// name runs on them before the transaction, as it does when a garden is made.
///
/// # Errors
/// `ERR_NOT_FOUND` on `id`; the name codes on `name_ur` and `name_en`.
pub(crate) fn changed_names(
    ctx: &Context,
    input: &GardenHeaderEditInput,
) -> Result<Option<(String, String)>, ServiceError> {
    let garden = ctx
        .db
        .read(|conn| repo::get(conn, input.id).map_err(ServiceError::from))?
        .ok_or_else(|| not_found("id"))?;
    let mut errors = FieldErrors::default();
    let name_ur = errors.take(validate::name_ur("name_ur", &input.garden.name_ur));
    let name_en = errors.take(validate::name_en("name_en", &input.garden.name_en));
    errors.finish()?;
    Ok(match (name_ur, name_en) {
        (Some(ur), Some(en)) if ur != garden.name_ur || en != garden.name_en => Some((ur, en)),
        _ => None,
    })
}

/// The edit inside the transaction of the caller: the save and the rehearsal of the review are this
/// one function (Code Standards B5). `learned` is the spelling check of changed names, whose pairs
/// are learned inside this transaction (PRD FR-7.26); the review passes `None`.
///
/// # Errors
/// `ERR_NOT_FOUND` on `id`; `ERR_VERSION_CHANGED` on `version`; `ERR_EDIT_DATE_FIXED` on `date`;
/// `ERR_REASON_REQUIRED` on `reason`; `ERR_VENTURE_SHARES_LOCKED` on `partners` when the shares are
/// not the garden's own; the codes of a new garden on `name_ur`, `name_en`, `rent`, `season_end`,
/// `note` and `item_id`. Nothing is written when any of them is returned.
#[allow(clippy::too_many_lines)]
pub(crate) fn edit_in(
    ctx: &Context,
    actor: &Actor,
    tx: &Connection,
    window: (Option<BusinessDate>, BusinessDate),
    input: &GardenHeaderEditInput,
    learned: Option<&SpellingCheck>,
) -> Result<(Edited, Option<crate::spelling::Learned>), ServiceError> {
    let garden = repo::get(tx, input.id)?.ok_or_else(|| not_found("id"))?;
    if garden.version != input.version {
        return Err(ServiceError::reject_field(
            ErrorCode::ErrVersionChanged,
            "version",
        ));
    }
    let old_date = BusinessDate::parse_iso(&garden.entry_date)
        .ok_or_else(|| data("the date of a garden is not a date"))?;
    let typed = &input.garden;
    if validate::date("date", &typed.date, window.1).is_ok_and(|d| d != old_date) {
        return Err(date_fixed("date", old_date));
    }
    let reason = reason_for(window.1, old_date, &input.reason)?;
    let (_, today) = window;
    let mut errors = FieldErrors::default();
    let name_ur = errors.take(validate::name_ur("name_ur", &typed.name_ur));
    let name_en = errors.take(validate::name_en("name_en", &typed.name_en));
    let rent = errors
        .take(validate::money("rent", &typed.rent))
        .filter(|rent| {
            let positive = *rent > Paisa::ZERO;
            if !positive {
                errors.push(FieldError::new(ErrorCode::ErrMustBeMoreThanZero, "rent"));
            }
            positive
        });
    let season_end = match typed
        .season_end
        .as_deref()
        .map(str::trim)
        .filter(|t| !t.is_empty())
    {
        Some(text) => errors.take(validate::date("season_end", text, today)),
        None => None,
    };
    if let Some(end) = season_end
        && end < old_date
    {
        errors.push(
            FieldError::new(ErrorCode::ErrDateBeforeGarden, "season_end")
                .with("garden_date", ErrorValue::Text(old_date.to_iso())),
        );
    }
    let note = optional_text(&mut errors, "note", typed.note.as_deref());
    if let Some(item_id) = typed.item_id {
        match mandi_db::bikri::items(tx, &[item_id])?.into_iter().next() {
            None => errors.push(FieldError::new(ErrorCode::ErrNotFound, "item_id")),
            Some(item) if !item.is_active => {
                errors.push(FieldError::new(ErrorCode::ErrAccountInactive, "item_id"));
            }
            Some(_) => {}
        }
    }
    let mut found = errors.finish().err().unwrap_or_default();
    // The partners and their shares are the garden's own (PRD FR-12.29).
    let parsed = read_partners(tx, &mut found, &typed.partners)?;
    if !found.is_empty() {
        return Err(ServiceError::from(found));
    }
    let own = vrepo::partners_of(tx, garden.id)?;
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
    let (Some(name_ur), Some(name_en), Some(rent)) = (name_ur, name_en, rent) else {
        return Err(ServiceError::reject(ErrorCode::ErrInternal));
    };
    let user = actor.user_id;
    let now = ctx.now_text();
    let old_value = row_value(tx, garden.id)?;
    let search_ur = text::search_ur(&name_ur);
    let search_en = text::search_en(&name_en);
    let search_phonetic = text::search_phonetic(&name_en);
    let season_iso = season_end.map(BusinessDate::to_iso);
    edit_windows::open(tx, Window::Venture, garden.id, user, &now)?;
    vrepo::update_garden_header(
        tx,
        garden.id,
        &GardenHeaderEdit {
            name_ur: &name_ur,
            name_en: &name_en,
            search_ur: &search_ur,
            search_en: &search_en,
            search_phonetic: &search_phonetic,
            item_id: typed.item_id,
            rent_paisa: rent.as_paisa(),
            season_end_date: season_iso.as_deref(),
            note: note.as_deref(),
        },
        user,
        &now,
    )?;
    edit_windows::close(tx, Window::Venture, garden.id)?;
    // The garden's account carries its names (PRD FR-12.34).
    if name_ur != garden.name_ur || name_en != garden.name_en {
        mandi_db::masters::rename_account(
            tx,
            garden.account.id,
            &AccountToMake {
                control_code: "0306",
                search_ur: search_ur.clone(),
                search_en: search_en.clone(),
                search_phonetic: search_phonetic.clone(),
                name_ur: name_ur.clone(),
                name_en: name_en.clone(),
            },
            user,
            &now,
        )?;
    }
    let learned = match learned {
        Some(spelled) => Some(spelled.learn(ctx, actor, tx)?),
        None => None,
    };
    let mut event = Event::new(Some(user), "edit", "garden")
        .entity_id(garden.id)
        .old(json!({ "number": garden.number, "row": old_value }))
        .new_value(json!({ "number": garden.number, "row": row_value(tx, garden.id)? }));
    if let Some(reason) = &reason {
        event = event.reason(reason);
    }
    event = event.sign(crate::day::fix_sign(tx, &garden.entry_date)?.as_deref());
    ctx.db.auditor().write(tx, &event.into_entry())?;
    let version = repo::get(tx, garden.id)?
        .map(|g| g.version)
        .ok_or_else(|| data("a garden is missing after its edit"))?;
    Ok((
        Edited {
            id: garden.id,
            number: Some(garden.number),
            version,
            entry_id: None,
            reversal_entry_id: None,
        },
        learned,
    ))
}
