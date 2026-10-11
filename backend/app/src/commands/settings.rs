//! Firm, security and language settings, and per-user preferences (PRD Section 21, FR-7.30,
//! FR-7.45, plan 04 Section 4).

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::run;
use crate::error::UiError;
use mandi_services::auth;
use mandi_services::core::auth::{Language, PrefKey};
use mandi_services::core::money::Grouping;
use mandi_services::settings::{self as services, Firm};

/// The firm details (PRD Section 21, Firm group). Text as typed; `address` and `phone` may be empty.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct FirmSettings {
    /// Firm name, Urdu letters.
    pub name_ur: String,
    /// Firm name, English letters.
    pub name_en: String,
    /// Mandi name, Urdu letters.
    pub mandi_ur: String,
    /// Mandi name, English letters.
    pub mandi_en: String,
    /// Address, up to 500 characters.
    pub address: String,
    /// Phone, `03XXXXXXXXX` or `03XX-XXXXXXX`.
    pub phone: String,
}

/// All settings the Settings screen shows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct Settings {
    /// Firm details.
    pub firm: FirmSettings,
    /// Minutes of no use before the screen locks (1 to 120, default 15, PRD FR-8.4).
    pub idle_lock_minutes: u32,
    /// Language for new users and for the backup pack (PRD Section 21).
    pub default_language: Language,
    /// How big numbers are grouped: `1,316,040` or `13,16,040` (PRD FR-19.7).
    pub number_grouping: Grouping,
}

/// Idle lock minutes, as typed.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct SetSecurityInput {
    /// Whole minutes, 1 to 120.
    pub idle_lock_minutes: String,
}

/// The default language for new users.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct SetDefaultLanguageInput {
    /// `ur` or `en`.
    pub language: Language,
}

/// Sets one preference of the signed-in user.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct PrefsSetInput {
    /// Which preference.
    pub key: PrefKey,
    /// The value as text: `ur`, `110`, `/bikri` ...
    pub value: String,
}

impl From<Firm> for FirmSettings {
    fn from(firm: Firm) -> Self {
        Self {
            name_ur: firm.name_ur,
            name_en: firm.name_en,
            mandi_ur: firm.mandi_ur,
            mandi_en: firm.mandi_en,
            address: firm.address,
            phone: firm.phone,
        }
    }
}

impl From<FirmSettings> for Firm {
    fn from(settings: FirmSettings) -> Self {
        Self {
            name_ur: settings.name_ur,
            name_en: settings.name_en,
            mandi_ur: settings.mandi_ur,
            mandi_en: settings.mandi_en,
            address: settings.address,
            phone: settings.phone,
        }
    }
}

/// All settings.
#[tauri::command]
pub async fn settings_get<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<Settings, UiError> {
    run(app, "settings_get", |ctx| {
        auth::require(ctx)?;
        let all = services::get(ctx)?;
        Ok(Settings {
            firm: FirmSettings::from(all.firm),
            idle_lock_minutes: all.idle_lock_minutes,
            default_language: all.default_language,
            number_grouping: all.number_grouping,
        })
    })
    .await
}

/// Saves the firm details, Errors: the name script errors on `name_ur`,
/// `name_en`, `mandi_ur`, `mandi_en`; `ERR_PHONE_INVALID`; `ERR_TEXT_TOO_LONG`.
#[tauri::command]
pub async fn settings_set_firm<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: FirmSettings,
) -> Result<FirmSettings, UiError> {
    run(app, "settings_set_firm", move |ctx| {
        let actor = auth::require(ctx)?;
        services::set_firm(ctx, &actor, &Firm::from(input)).map(FirmSettings::from)
    })
    .await
}

/// Saves the idle lock minutes, Errors: `ERR_NOT_A_NUMBER`, `ERR_OUT_OF_RANGE`
/// (`idle_lock_minutes`, values `min` and `max`).
#[tauri::command]
pub async fn settings_set_security<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: SetSecurityInput,
) -> Result<(), UiError> {
    run(app, "settings_set_security", move |ctx| {
        let actor = auth::require(ctx)?;
        services::set_idle_lock(ctx, &actor, &input.idle_lock_minutes)
    })
    .await
}

/// Saves the default language.
#[tauri::command]
pub async fn settings_set_default_language<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: SetDefaultLanguageInput,
) -> Result<(), UiError> {
    run(app, "settings_set_default_language", move |ctx| {
        let actor = auth::require(ctx)?;
        services::set_default_language(ctx, &actor, input.language)
    })
    .await
}

/// Saves one preference of the signed-in user. Errors: `ERR_OUT_OF_RANGE` (`value`).
#[tauri::command]
pub async fn prefs_set<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: PrefsSetInput,
) -> Result<(), UiError> {
    run(app, "prefs_set", move |ctx| {
        let actor = auth::require(ctx)?;
        services::set_pref(ctx, &actor, input.key, &input.value)
    })
    .await
}
