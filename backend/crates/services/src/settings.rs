//! Firm details, security and language settings, and per-user preferences
//! (PRD Section 21, FR-7.30, FR-8.4, plan 04 Section 4).
//!
//! Settings live in the `settings` table as JSON by key; preferences in `user_prefs`.
//! Every change by the Owner writes an audit row in the same transaction (S7).

use mandi_db::rusqlite::Connection;
use mandi_domain::auth::validate as rules;
use mandi_domain::core::auth::{Language, PrefKey};
use mandi_domain::core::error::FieldError;
use mandi_domain::core::money::Grouping;
use mandi_domain::core::validate::{self, FieldErrors};
use serde_json::{Value, json};

use crate::audit::Event;
use crate::auth::Actor;
use crate::context::Context;
use crate::error::ServiceError;
use crate::store;

/// Setting keys in the `settings` table.
pub(crate) const KEY_FIRM: &str = "firm";
pub(crate) const KEY_IDLE_LOCK: &str = "idle_lock_minutes";
pub(crate) const KEY_DEFAULT_LANGUAGE: &str = "default_language";
pub(crate) const KEY_NUMBER_GROUPING: &str = "number_grouping";
/// The language last used on this computer, for the sign-in screen (PRD FR-7.30).
pub(crate) const KEY_LOGIN_LANGUAGE: &str = "login_language";

/// The firm details (PRD Section 21, Firm group).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Firm {
    /// Firm name, Urdu letters.
    pub name_ur: String,
    /// Firm name, English letters.
    pub name_en: String,
    /// Mandi name, Urdu letters.
    pub mandi_ur: String,
    /// Mandi name, English letters.
    pub mandi_en: String,
    /// Address, up to 500 characters. May be empty.
    pub address: String,
    /// Phone as 11 digits. May be empty.
    pub phone: String,
}

/// The names of the form fields an error points at. The first-run wizard and the Settings
/// screen call the same fields by different names.
#[derive(Debug, Clone, Copy)]
pub struct FirmFields {
    /// Field of the Urdu firm name.
    pub name_ur: &'static str,
    /// Field of the English firm name.
    pub name_en: &'static str,
    /// Field of the Urdu mandi name.
    pub mandi_ur: &'static str,
    /// Field of the English mandi name.
    pub mandi_en: &'static str,
    /// Field of the address.
    pub address: &'static str,
    /// Field of the phone.
    pub phone: &'static str,
}

impl FirmFields {
    /// The Settings screen.
    pub const SETTINGS: FirmFields = FirmFields {
        name_ur: "name_ur",
        name_en: "name_en",
        mandi_ur: "mandi_ur",
        mandi_en: "mandi_en",
        address: "address",
        phone: "phone",
    };
    /// The first-run wizard.
    pub const FIRST_RUN: FirmFields = FirmFields {
        name_ur: "firm_name_ur",
        name_en: "firm_name_en",
        mandi_ur: "mandi_ur",
        mandi_en: "mandi_en",
        address: "address",
        phone: "phone",
    };
}

impl Firm {
    /// Checks every field and returns the cleaned firm, or every problem found. The names must
    /// be in their own script (PRD FR-7.28); address and phone may be empty.
    ///
    /// # Errors
    /// The field errors, all together.
    pub fn validated(&self, fields: &FirmFields) -> Result<Firm, Vec<FieldError>> {
        let mut errors = FieldErrors::default();
        let name_ur = errors.take(validate::name_ur(fields.name_ur, &self.name_ur));
        let name_en = errors.take(validate::name_en(fields.name_en, &self.name_en));
        let mandi_ur = errors.take(validate::name_ur(fields.mandi_ur, &self.mandi_ur));
        let mandi_en = errors.take(validate::name_en(fields.mandi_en, &self.mandi_en));
        let address = errors.take(validate::free_text(fields.address, &self.address));
        let phone = errors.take(validate::phone_optional(fields.phone, &self.phone));
        errors.finish()?;
        Ok(Firm {
            name_ur: name_ur.unwrap_or_default(),
            name_en: name_en.unwrap_or_default(),
            mandi_ur: mandi_ur.unwrap_or_default(),
            mandi_en: mandi_en.unwrap_or_default(),
            address: address.unwrap_or_default(),
            phone: phone.flatten().unwrap_or_default(),
        })
    }

    pub(crate) fn to_json(&self) -> Value {
        json!({
            "name_ur": self.name_ur,
            "name_en": self.name_en,
            "mandi_ur": self.mandi_ur,
            "mandi_en": self.mandi_en,
            "address": self.address,
            "phone": self.phone,
        })
    }

    fn from_json(value: &Value) -> Firm {
        let text = |key: &str| {
            value
                .get(key)
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned()
        };
        Firm {
            name_ur: text("name_ur"),
            name_en: text("name_en"),
            mandi_ur: text("mandi_ur"),
            mandi_en: text("mandi_en"),
            address: text("address"),
            phone: text("phone"),
        }
    }
}

/// All the settings the Settings screen shows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Settings {
    /// Firm details.
    pub firm: Firm,
    /// Minutes of no use before the screen locks.
    pub idle_lock_minutes: u32,
    /// Language for new users and for the backup pack.
    pub default_language: Language,
    /// How big numbers are grouped.
    pub number_grouping: Grouping,
}

/// The firm as saved. Empty before the first run.
pub(crate) fn load_firm(conn: &Connection) -> Result<Firm, ServiceError> {
    Ok(store::setting(conn, KEY_FIRM)?
        .map(|v| Firm::from_json(&v))
        .unwrap_or_default())
}

fn load_language(conn: &Connection, key: &str) -> Result<Option<Language>, ServiceError> {
    Ok(store::setting(conn, key)?
        .as_ref()
        .and_then(Value::as_str)
        .and_then(Language::parse))
}

/// The language for new users: the saved default, else Urdu (PRD FR-7.30).
pub(crate) fn load_default_language(conn: &Connection) -> Result<Language, ServiceError> {
    Ok(load_language(conn, KEY_DEFAULT_LANGUAGE)?.unwrap_or(Language::Ur))
}

/// The language last used on this computer: the saved one, else the default language.
pub(crate) fn load_login_language(conn: &Connection) -> Result<Language, ServiceError> {
    match load_language(conn, KEY_LOGIN_LANGUAGE)? {
        Some(language) => Ok(language),
        None => load_default_language(conn),
    }
}

fn load_idle_lock(conn: &Connection) -> Result<u32, ServiceError> {
    Ok(store::setting(conn, KEY_IDLE_LOCK)?
        .and_then(|v| v.as_u64())
        .and_then(|n| u32::try_from(n).ok())
        .unwrap_or(rules::DEFAULT_IDLE_LOCK_MINUTES))
}

fn load_grouping(conn: &Connection) -> Result<Grouping, ServiceError> {
    Ok(store::setting(conn, KEY_NUMBER_GROUPING)?
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default())
}

/// All settings. Any signed-in user may read them (the caller has checked the session).
///
/// # Errors
/// `ServiceError::Db` or `ServiceError::Data`.
pub fn get(ctx: &Context) -> Result<Settings, ServiceError> {
    ctx.db.read(|conn| {
        Ok(Settings {
            firm: load_firm(conn)?,
            idle_lock_minutes: load_idle_lock(conn)?,
            default_language: load_default_language(conn)?,
            number_grouping: load_grouping(conn)?,
        })
    })
}

/// Saves the firm details (Owner). Both scripts of every name are checked.
///
/// # Errors
/// The field errors on `name_ur`, `name_en`, `mandi_ur`, `mandi_en`, `address`, `phone`.
pub fn set_firm(ctx: &Context, actor: &Actor, input: &Firm) -> Result<Firm, ServiceError> {
    let firm = input.validated(&FirmFields::SETTINGS)?;
    ctx.db.write(|tx| {
        let old = load_firm(tx)?;
        store::set_setting(tx, KEY_FIRM, &firm.to_json(), Some(actor.user_id))?;
        ctx.db.auditor().write(
            tx,
            &Event::new(Some(actor.user_id), "edit", "settings")
                .old(json!({ "firm": old.to_json() }))
                .new_value(json!({ "firm": firm.to_json() }))
                .into_entry(),
        )?;
        Ok(firm.clone())
    })
}

/// Saves the idle lock minutes (Owner), typed as text: 1 to 120.
///
/// # Errors
/// `ERR_REQUIRED`, `ERR_NOT_A_NUMBER` or `ERR_OUT_OF_RANGE` on `idle_lock_minutes`.
pub fn set_idle_lock(ctx: &Context, actor: &Actor, minutes_text: &str) -> Result<(), ServiceError> {
    let minutes = rules::idle_lock_minutes("idle_lock_minutes", minutes_text)?;
    ctx.db.write(|tx| {
        let old = load_idle_lock(tx)?;
        store::set_setting(tx, KEY_IDLE_LOCK, &json!(minutes), Some(actor.user_id))?;
        ctx.db.auditor().write(
            tx,
            &Event::new(Some(actor.user_id), "edit", "settings")
                .old(json!({ "idle_lock_minutes": old }))
                .new_value(json!({ "idle_lock_minutes": minutes }))
                .into_entry(),
        )?;
        Ok(())
    })
}

/// Saves the default language for new users (Owner).
///
/// # Errors
/// `ServiceError::Db`.
pub fn set_default_language(
    ctx: &Context,
    actor: &Actor,
    language: Language,
) -> Result<(), ServiceError> {
    ctx.db.write(|tx| {
        let old = load_default_language(tx)?;
        store::set_setting(
            tx,
            KEY_DEFAULT_LANGUAGE,
            &json!(language.as_str()),
            Some(actor.user_id),
        )?;
        ctx.db.auditor().write(
            tx,
            &Event::new(Some(actor.user_id), "edit", "settings")
                .old(json!({ "default_language": old.as_str() }))
                .new_value(json!({ "default_language": language.as_str() }))
                .into_entry(),
        )?;
        Ok(())
    })
}

/// Saves one preference of the signed-in user (PRD FR-7.30, FR-7.45). The value is text and is
/// checked by key. The language also becomes the sign-in language of this computer.
///
/// # Errors
/// `ERR_OUT_OF_RANGE` (`value`) for a language that is not `ur` or `en`, a size outside
/// 80 to 150 in tens, or a route that does not start with `/`; `ERR_NOT_A_NUMBER`.
pub fn set_pref(
    ctx: &Context,
    actor: &Actor,
    key: PrefKey,
    value_text: &str,
) -> Result<(), ServiceError> {
    let value: Value = match key {
        PrefKey::Language => {
            let language = Language::parse(value_text.trim()).ok_or_else(|| {
                ServiceError::reject_field(
                    mandi_domain::core::error::ErrorCode::ErrOutOfRange,
                    "value",
                )
            })?;
            json!(language.as_str())
        }
        PrefKey::TextSize => json!(rules::text_size("value", value_text)?),
        PrefKey::LastScreen => json!(rules::last_screen("value", value_text)?),
    };
    ctx.db.write(|tx| {
        store::set_pref(tx, actor.user_id, key.as_str(), &value)?;
        if key == PrefKey::Language {
            store::set_setting(tx, KEY_LOGIN_LANGUAGE, &value, Some(actor.user_id))?;
        }
        Ok(())
    })
}
