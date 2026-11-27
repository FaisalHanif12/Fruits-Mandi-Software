//! "بعد میں" (later) on the bell panel (PRD FR-7.44 v1.61, plan 100 J): the hidden items are kept
//! in the Owner's preference `attention.snoozed` and come back with the next بکری save. Only the
//! preference is written: nothing in the books.

use mandi_db::rusqlite::{Connection, params};
use mandi_domain::core::error::ErrorCode;
use serde_json::{Value, json};

use crate::auth::Actor;
use crate::context::Context;
use crate::error::ServiceError;
use crate::store;

/// The preference key.
const KEY: &str = "attention.snoozed";
/// The most hidden items kept.
const MAX_HIDDEN: usize = 500;
/// The kinds of item that name an account.
const ACCOUNT_KINDS: [&str; 3] = ["seller", "buyer", "payable"];
/// The longest name of a reminder kind.
const MAX_KIND_CHARS: usize = 40;

/// True for a key the panel gives: `seller:12`, `buyer:7`, `payable:3` or `kind:drafts`.
fn is_item_key(key: &str) -> bool {
    let Some((prefix, rest)) = key.split_once(':') else {
        return false;
    };
    if ACCOUNT_KINDS.contains(&prefix) {
        return !rest.is_empty() && rest.len() <= 18 && rest.bytes().all(|b| b.is_ascii_digit());
    }
    prefix == "kind"
        && !rest.is_empty()
        && rest.len() <= MAX_KIND_CHARS
        && rest.bytes().all(|b| b.is_ascii_lowercase() || b == b'_')
}

fn keys_of(value: Option<Value>) -> Vec<String> {
    value
        .as_ref()
        .and_then(|v| v.get("keys"))
        .and_then(Value::as_array)
        .map(|keys| {
            keys.iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

/// The items the signed-in Owner hid since the last بکری save. Read only.
///
/// # Errors
/// `ServiceError::Db`.
pub fn snoozed_keys(ctx: &Context, actor: &Actor) -> Result<Vec<String>, ServiceError> {
    ctx.db
        .read(|conn| Ok(keys_of(store::pref(conn, actor.user_id, KEY)?)))
}

/// Hides one item of the bell panel until the next بکری save.
///
/// # Errors
/// `ERR_REQUIRED` on `item_key` when empty; `ERR_FORMAT_INVALID` on `item_key` when it is not a key
/// of the panel; `ServiceError::Db`.
pub fn snooze(ctx: &Context, actor: &Actor, item_key: &str) -> Result<(), ServiceError> {
    let key = item_key.trim();
    if key.is_empty() {
        return Err(ServiceError::reject_field(
            ErrorCode::ErrRequired,
            "item_key",
        ));
    }
    if !is_item_key(key) {
        return Err(ServiceError::reject_field(
            ErrorCode::ErrFormatInvalid,
            "item_key",
        ));
    }
    ctx.db.write(|tx| {
        let mut keys = keys_of(store::pref(tx, actor.user_id, KEY)?);
        keys.retain(|k| k != key);
        keys.push(key.to_owned());
        let extra = keys.len().saturating_sub(MAX_HIDDEN);
        keys.drain(..extra);
        store::set_pref(tx, actor.user_id, KEY, &json!({ "keys": keys }))
    })
}

/// Brings every hidden item back. Called inside the transaction of every بکری save, so it happens
/// with the save or not at all. It writes nothing when nothing was ever hidden.
///
/// # Errors
/// `ServiceError::Db`.
pub(crate) fn clear_snoozes(tx: &Connection, user_id: i64) -> Result<(), ServiceError> {
    tx.execute(
        "UPDATE user_prefs SET value_json = ?3, updated_by = ?1,
             updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), version = version + 1
         WHERE user_id = ?1 AND key = ?2 AND value_json <> ?3",
        params![user_id, KEY, json!({ "keys": [] }).to_string()],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::is_item_key;

    #[test]
    fn only_the_panel_keys_are_taken() {
        for good in [
            "seller:12",
            "buyer:1",
            "payable:7",
            "kind:drafts",
            "kind:open_days",
        ] {
            assert!(is_item_key(good), "{good}");
        }
        for bad in [
            "seller:",
            "seller:-1",
            "x:1",
            "kind:",
            "kind:Drafts",
            "drafts",
            "",
        ] {
            assert!(!is_item_key(bad), "{bad}");
        }
    }
}
