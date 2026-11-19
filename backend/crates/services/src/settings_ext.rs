//! The settings row of PRD Section 21 that plan 04 left out: the number grouping (PRD FR-19.7).
//! The Owner picks `1,316,040` or `13,16,040`; prints and report print data read it
//! (`print::load::load_grouping`). A change writes an audit row with the old and the new value
//! in the same transaction (Code Standards S7), and asking for the value it has writes nothing.

use mandi_domain::core::money::Grouping;
use serde_json::json;

use crate::audit::Event;
use crate::auth::Actor;
use crate::context::Context;
use crate::error::ServiceError;
use crate::settings::KEY_NUMBER_GROUPING;
use crate::store;

fn name(grouping: Grouping) -> &'static str {
    match grouping {
        Grouping::International => "international",
        Grouping::Lakh => "lakh",
    }
}

/// The saved grouping; the international one before the Owner chooses.
///
/// # Errors
/// `ServiceError::Db`.
pub fn number_grouping(ctx: &Context) -> Result<Grouping, ServiceError> {
    ctx.db.read(|conn| {
        Ok(store::setting(conn, KEY_NUMBER_GROUPING)?
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or_default())
    })
}

/// Saves the number grouping (Owner).
///
/// # Errors
/// `ServiceError::Db`.
pub fn set_number_grouping(
    ctx: &Context,
    actor: &Actor,
    grouping: Grouping,
) -> Result<Grouping, ServiceError> {
    ctx.db.write(|tx| {
        let old: Grouping = store::setting(tx, KEY_NUMBER_GROUPING)?
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or_default();
        if old != grouping {
            let value =
                serde_json::to_value(grouping).map_err(|e| ServiceError::Data(e.to_string()))?;
            store::set_setting(tx, KEY_NUMBER_GROUPING, &value, Some(actor.user_id))?;
            ctx.db.auditor().write(
                tx,
                &Event::new(Some(actor.user_id), "edit", "settings")
                    .old(json!({ "number_grouping": name(old) }))
                    .new_value(json!({ "number_grouping": name(grouping) }))
                    .into_entry(),
            )?;
        }
        Ok(grouping)
    })
}
