//! The Owner's sign (دستخط) that opens a closed day to fix an entry (PRD FR-21.4, FR-7.71, plan 73).
//!
//! Only the Owner's sign opens the day. The sign is the settings key `owner_sign`; until the Owner
//! changes it, it is `adnan` (a constant, no row is written). It is compared after trimming and
//! ignoring case, and in Urdu letters with the letter variants of the search keys. While the sign is
//! still the default, the Urdu spelling `عدنان` is accepted too. Once the Owner has chosen his own sign,
//! only that sign opens the day.
//!
//! The typed sign is never written to the audit log when it is wrong. A right sign is saved in the
//! form the Owner set it (not the typed case), beside the reason, so the stamp of an edit made in the
//! fix shows it.
//!
//! Example: the sign `adnan` accepts `adnan`, `ADNAN ` and `عدنان`; `munshi` is refused.

use mandi_db::rusqlite::Connection;
use mandi_domain::core::error::{ErrorCode, ErrorValue, FieldError};
use mandi_domain::core::text;
use mandi_domain::core::validate;
use serde_json::json;

use crate::audit::Event;
use crate::auth::{Actor, prove_owner};
use crate::context::Context;
use crate::error::{Rejection, ServiceError};
use crate::secret::Secret;
use crate::store;

/// The settings key of the Owner's sign.
pub const KEY_OWNER_SIGN: &str = "owner_sign";

/// The sign until the Owner changes it (PRD FR-21.4).
pub const DEFAULT_SIGN: &str = "adnan";

/// The Urdu spelling of the default sign, accepted while the sign is the default.
const DEFAULT_SIGN_URDU: &str = "عدنان";

/// The longest sign, in characters.
pub const MAX_SIGN_CHARS: usize = 40;

/// The sign the Owner chose, or `None` while it is the default.
///
/// # Errors
/// `ServiceError::Db`.
pub(crate) fn configured(conn: &Connection) -> Result<Option<String>, ServiceError> {
    Ok(store::setting(conn, KEY_OWNER_SIGN)?
        .and_then(|value| value.as_str().map(|s| s.trim().to_owned()))
        .filter(|sign| !sign.is_empty()))
}

/// The key two signs are compared by: Urdu letters through the Urdu search key (ي and ی are one
/// letter), anything else lower case with one space between words.
fn key(typed: &str) -> String {
    let trimmed = typed.trim();
    if trimmed.chars().any(text::is_urdu_letter) {
        text::search_ur(trimmed)
    } else {
        text::search_en(trimmed)
    }
}

/// The sign to save when `typed` is the Owner's sign, or `None` when it is not. `configured` is the
/// sign he chose, or `None` for the default.
pub(crate) fn accepted(typed: &str, configured: Option<&str>) -> Option<String> {
    let wanted = configured.unwrap_or(DEFAULT_SIGN);
    let typed = key(typed);
    if typed.is_empty() {
        return None;
    }
    let right = typed == key(wanted) || (configured.is_none() && typed == key(DEFAULT_SIGN_URDU));
    right.then(|| wanted.to_owned())
}

/// Changes the Owner's sign, proved by his password (PRD FR-21.4, plan 73). The change is in the
/// audit log; the sign itself is not.
///
/// # Errors
/// `ERR_REQUIRED` and `ERR_TEXT_TOO_LONG` (value `max`) and `ERR_CONTROL_CHARS` on `sign`;
/// `ERR_REQUIRED` or `ERR_WRONG_PASSWORD` on `password`; `ERR_LOCKED_WAIT`.
pub fn set_owner_sign(
    ctx: &Context,
    actor: &Actor,
    password: &Secret,
    sign: &str,
) -> Result<(), ServiceError> {
    let cleaned = validate::free_text("sign", sign).map_err(ServiceError::from)?;
    if cleaned.is_empty() {
        return Err(ServiceError::reject_field(ErrorCode::ErrRequired, "sign"));
    }
    if cleaned.chars().count() > MAX_SIGN_CHARS {
        let max = i64::try_from(MAX_SIGN_CHARS).unwrap_or(i64::MAX);
        return Err(ServiceError::Rejected(vec![Rejection::from(
            FieldError::new(ErrorCode::ErrTextTooLong, "sign").with("max", ErrorValue::Number(max)),
        )]));
    }
    prove_owner(ctx, actor, password, "password", false)?;
    ctx.db.write(|tx| {
        let was_default = configured(tx)?.is_none();
        store::set_setting(tx, KEY_OWNER_SIGN, &json!(cleaned), Some(actor.user_id))?;
        ctx.db.auditor().write(
            tx,
            &Event::new(Some(actor.user_id), "edit", "settings")
                .old(json!({ "owner_sign": if was_default { "default" } else { "set" } }))
                .new_value(json!({ "owner_sign": "changed" }))
                .into_entry(),
        )?;
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_default_sign_is_adnan_in_any_case_with_spaces_at_the_ends_or_in_urdu() {
        for typed in ["adnan", "ADNAN ", "  Adnan", "عدنان", " عدنان "] {
            assert_eq!(accepted(typed, None).as_deref(), Some("adnan"), "{typed}");
        }
        for typed in ["munshi", "", "  ", "adn an", "adnaan", "اکرم"] {
            assert_eq!(accepted(typed, None), None, "{typed}");
        }
    }

    #[test]
    fn a_sign_the_owner_chose_replaces_the_default_and_its_urdu_spelling() {
        assert_eq!(accepted("SANA ", Some("sana")).as_deref(), Some("sana"));
        assert_eq!(accepted("adnan", Some("sana")), None);
        assert_eq!(accepted("عدنان", Some("sana")), None);
        // An Urdu sign is compared with the letter variants of the search keys.
        assert_eq!(accepted("سنا", Some("سنا")).as_deref(), Some("سنا"));
    }
}
