//! Firm details, security and language settings, and per-user preferences
//! (PRD Section 21, FR-7.30, FR-8.4, plan 04 Section 4).
//!
//! Settings live in the `settings` table as JSON by key; preferences in `user_prefs`.
//! Every change by the Owner writes an audit row in the same transaction (S7).

use mandi_db::rusqlite::Connection;
use mandi_domain::auth::validate as rules;
use mandi_domain::core::auth::{Language, PrefKey};
use mandi_domain::core::error::FieldError;
use mandi_domain::core::firm::{
    FirmContact, MAX_CONTACTS, SETTING_KEY_CONTACTS, SETTING_KEY_TRADE_MARK,
};
use mandi_domain::core::money::Grouping;
use mandi_domain::core::translit::Kind;
use mandi_domain::core::validate::{self, FieldErrors};
use serde_json::{Value, json};

use crate::audit::Event;
use crate::auth::Actor;
use crate::context::Context;
use crate::error::ServiceError;
use crate::spelling::{self, NamePair, SpellingCheck};
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

/// Checks the spelling of the firm name and the mandi name (PRD FR-7.24, FR-7.66). A word that
/// only the spelling rules could have made is refused until the screen sends it in
/// `confirmed_rules_words`. Both names are checked together, so every open word comes back in
/// one answer. The firm is a person-like name, the mandi a place.
///
/// # Errors
/// `ERR_SPELLING_UNCONFIRMED` on the fields in `fields`, with `values.words`.
pub(crate) fn spelling_checks(
    ctx: &Context,
    firm: &Firm,
    fields: &FirmFields,
    confirmed_rules_words: &[String],
) -> Result<Vec<SpellingCheck>, ServiceError> {
    spelling::check_all(
        ctx,
        &[
            NamePair {
                field_ur: fields.name_ur,
                name_ur: &firm.name_ur,
                field_en: fields.name_en,
                name_en: &firm.name_en,
                kind: Kind::Name,
            },
            NamePair {
                field_ur: fields.mandi_ur,
                name_ur: &firm.mandi_ur,
                field_en: fields.mandi_en,
                name_en: &firm.mandi_en,
                kind: Kind::Place,
            },
        ],
        confirmed_rules_words,
    )
}

/// Saves the firm details (Owner) with no spelling confirmed: both scripts of every name are
/// checked, and a name with a rules-only word is refused. See [`set_firm_confirmed`].
///
/// # Errors
/// The field errors on `name_ur`, `name_en`, `mandi_ur`, `mandi_en`, `address`, `phone`;
/// `ERR_SPELLING_UNCONFIRMED` on a name field.
pub fn set_firm(ctx: &Context, actor: &Actor, input: &Firm) -> Result<Firm, ServiceError> {
    set_firm_confirmed(ctx, actor, input, &[])
}

/// Saves the firm details (Owner). Both scripts of every name are checked, and so is the
/// spelling (PRD FR-7.66): the words in `confirmed_rules_words` are accepted and learned as
/// pairs inside the same transaction (PRD FR-7.26).
///
/// # Errors
/// The field errors on `name_ur`, `name_en`, `mandi_ur`, `mandi_en`, `address`, `phone`;
/// `ERR_SPELLING_UNCONFIRMED` (value `words`) on a name field that has an open word.
pub fn set_firm_confirmed(
    ctx: &Context,
    actor: &Actor,
    input: &Firm,
    confirmed_rules_words: &[String],
) -> Result<Firm, ServiceError> {
    let firm = input.validated(&FirmFields::SETTINGS)?;
    let checks = spelling_checks(ctx, &firm, &FirmFields::SETTINGS, confirmed_rules_words)?;
    let (saved, learned) = ctx.db.write(|tx| {
        let old = load_firm(tx)?;
        store::set_setting(tx, KEY_FIRM, &firm.to_json(), Some(actor.user_id))?;
        ctx.db.auditor().write(
            tx,
            &Event::new(Some(actor.user_id), "edit", "settings")
                .old(json!({ "firm": old.to_json() }))
                .new_value(json!({ "firm": firm.to_json() }))
                .into_entry(),
        )?;
        let mut learned = Vec::new();
        for check in &checks {
            learned.push(check.learn(ctx, actor, tx)?);
        }
        Ok::<_, ServiceError>((firm.clone(), learned))
    })?;
    for learned in learned {
        learned.after_commit(ctx);
    }
    Ok(saved)
}

/// The contact persons and the trade mark that print in the header of every print (PRD
/// Section 21 "Firm", FR-19.13, plan 17e). They live under their own settings keys, apart from
/// the firm details.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FirmPrintInfo {
    /// The contact persons, 0 to 3 (`firm_contacts`).
    pub firm_contacts: Vec<FirmContact>,
    /// The trade mark text as typed, or empty (`trade_mark`).
    pub trade_mark: String,
}

impl FirmPrintInfo {
    /// True when no contact and no trade mark is set.
    pub fn is_empty(&self) -> bool {
        self.firm_contacts.is_empty() && self.trade_mark.trim().is_empty()
    }
}

/// The saved contact persons and trade mark. Empty before the Owner sets them. A damaged value
/// gives none, so a screen or a print is never stopped by it.
fn load_print_info(conn: &Connection) -> Result<FirmPrintInfo, ServiceError> {
    let mut firm_contacts: Vec<FirmContact> = store::setting(conn, SETTING_KEY_CONTACTS)?
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default();
    firm_contacts.truncate(MAX_CONTACTS);
    let trade_mark = store::setting(conn, SETTING_KEY_TRADE_MARK)?
        .as_ref()
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned();
    Ok(FirmPrintInfo {
        firm_contacts,
        trade_mark,
    })
}

/// The contact persons and the trade mark for the Settings screen. Read only.
///
/// # Errors
/// `ServiceError::Db` or `ServiceError::Data`.
pub fn get_print_info(ctx: &Context) -> Result<FirmPrintInfo, ServiceError> {
    ctx.db.read(load_print_info)
}

/// Saves the firm details together with the contact persons and the trade mark (Owner): one user
/// action, one transaction, one audit entry per change (Code Standards B3, S7).
///
/// Plan 17e T1: the contacts and the trade mark are in the contract but are not stored yet. A
/// request that carries either is refused, so a save never drops them silently; one that carries
/// neither saves the firm exactly as `set_firm_confirmed` does. T2 stores them, checks each
/// contact (names in their own script, phone, at most 3) and writes the audit entry.
///
/// # Errors
/// As `set_firm_confirmed`; `ERR_NOT_IMPLEMENTED` on `firm_contacts` or `trade_mark` until T2.
pub fn set_firm_with_print_info(
    ctx: &Context,
    actor: &Actor,
    firm: &Firm,
    info: &FirmPrintInfo,
    confirmed_rules_words: &[String],
) -> Result<(Firm, FirmPrintInfo), ServiceError> {
    if !info.firm_contacts.is_empty() {
        return Err(ServiceError::reject_field(
            mandi_domain::core::error::ErrorCode::ErrNotImplemented,
            "firm_contacts",
        ));
    }
    if !info.trade_mark.trim().is_empty() {
        return Err(ServiceError::reject_field(
            mandi_domain::core::error::ErrorCode::ErrNotImplemented,
            "trade_mark",
        ));
    }
    let saved = set_firm_confirmed(ctx, actor, firm, confirmed_rules_words)?;
    Ok((saved, get_print_info(ctx)?))
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
            // The sign-in language is a setting of this computer, so a change of it is audited
            // in the same transaction (Code Standards S7). A choice that changes nothing writes
            // no row.
            let old = load_login_language(tx)?;
            store::set_setting(tx, KEY_LOGIN_LANGUAGE, &value, Some(actor.user_id))?;
            if value.as_str() != Some(old.as_str()) {
                ctx.db.auditor().write(
                    tx,
                    &Event::new(Some(actor.user_id), "edit", "settings")
                        .old(json!({ "login_language": old.as_str() }))
                        .new_value(json!({ "login_language": value }))
                        .into_entry(),
                )?;
            }
        }
        Ok::<(), ServiceError>(())
    })
}
