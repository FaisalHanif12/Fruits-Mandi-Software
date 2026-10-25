//! Firm, security and language settings, and per-user preferences (PRD Section 21, FR-7.30,
//! FR-7.45, plan 04 Section 4).

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::run;
use crate::error::UiError;
use mandi_services::auth;
use mandi_services::core::auth::{Language, PrefKey};
use mandi_services::core::firm::FirmContact;
use mandi_services::core::money::Grouping;
use mandi_services::settings::{self as services, Firm, FirmPrintInfo};

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
    /// The contact persons printed in the header of every print, 0 to 3, in print order (PRD
    /// Section 21, FR-19.13). Stored under the settings key `firm_contacts`.
    pub firm_contacts: Vec<FirmContact>,
    /// The trade mark printed in a box in the header, as typed. Empty if not set. Stored under the
    /// settings key `trade_mark`.
    pub trade_mark: String,
}

/// The firm details to save, with the spelling the Owner confirmed (PRD FR-7.66).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, TS)]
pub struct SetFirmInput {
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
    /// The contact persons, 0 to 3. Each has both names, each in its own script, and a phone.
    /// Missing means none.
    #[serde(default)]
    pub firm_contacts: Vec<FirmContact>,
    /// The trade mark text. Missing or empty means none.
    #[serde(default)]
    pub trade_mark: String,
    /// Words of the four names that only the spelling rules made and that the Owner confirmed.
    /// A word left out is refused with `ERR_SPELLING_UNCONFIRMED`. Missing means none confirmed.
    #[serde(default)]
    pub confirmed_rules_words: Vec<String>,
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

impl From<(Firm, FirmPrintInfo)> for FirmSettings {
    fn from((firm, info): (Firm, FirmPrintInfo)) -> Self {
        Self {
            name_ur: firm.name_ur,
            name_en: firm.name_en,
            mandi_ur: firm.mandi_ur,
            mandi_en: firm.mandi_en,
            address: firm.address,
            phone: firm.phone,
            firm_contacts: info.firm_contacts,
            trade_mark: info.trade_mark,
        }
    }
}

impl From<&SetFirmInput> for FirmPrintInfo {
    fn from(input: &SetFirmInput) -> Self {
        Self {
            firm_contacts: input.firm_contacts.clone(),
            trade_mark: input.trade_mark.clone(),
        }
    }
}

impl From<&SetFirmInput> for Firm {
    fn from(input: &SetFirmInput) -> Self {
        Self {
            name_ur: input.name_ur.clone(),
            name_en: input.name_en.clone(),
            mandi_ur: input.mandi_ur.clone(),
            mandi_en: input.mandi_en.clone(),
            address: input.address.clone(),
            phone: input.phone.clone(),
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
        let print_info = services::get_print_info(ctx)?;
        Ok(Settings {
            firm: FirmSettings::from((all.firm, print_info)),
            idle_lock_minutes: all.idle_lock_minutes,
            default_language: all.default_language,
            number_grouping: all.number_grouping,
        })
    })
    .await
}

/// Saves the firm details with the contact persons and the trade mark of the print header (PRD
/// Section 21, FR-19.13), as one action with one audit entry per change. Errors: the name script
/// errors on `name_ur`, `name_en`, `mandi_ur`, `mandi_en`; `ERR_PHONE_INVALID`;
/// `ERR_TEXT_TOO_LONG`; `ERR_SPELLING_UNCONFIRMED` (value `words`) on a name field for a word the
/// spelling rules made that is not in `confirmed_rules_words` (PRD FR-7.66). Confirmed words are
/// learned (FR-7.26). Until plan 17e T2 a request with a contact or a trade mark answers
/// `ERR_NOT_IMPLEMENTED` on `firm_contacts` or `trade_mark`; from T2 the contact codes are on
/// `firm_contacts[i].name_ur`, `.name_en` and `.phone`, `ERR_OUT_OF_RANGE` (values `min` 0, `max` 3) on `firm_contacts`, and
/// `ERR_TEXT_TOO_LONG` on `trade_mark`.
#[tauri::command]
pub async fn settings_set_firm<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: SetFirmInput,
) -> Result<FirmSettings, UiError> {
    run(app, "settings_set_firm", move |ctx| {
        let actor = auth::require(ctx)?;
        services::set_firm_with_print_info(
            ctx,
            &actor,
            &Firm::from(&input),
            &FirmPrintInfo::from(&input),
            &input.confirmed_rules_words,
        )
        .map(FirmSettings::from)
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

/// The three global defaults of PRD 4.2 and Section 21 "Charges": seller commission, buyer
/// commission and لاگا on fruit.
#[tauri::command]
pub async fn settings_get_charges<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<mandi_services::core::rules::ChargesSettings, UiError> {
    run(app, "settings_get_charges", |ctx| {
        auth::require(ctx)?;
        mandi_services::rules::charges(ctx)
    })
    .await
}

/// Saves the three global defaults. They are rules, so the change is in the audit log with the
/// old and new value, and saved sale lines keep what they stored (PRD FR-11.6). Errors: the
/// value codes of a rule (`seller_commission.value` and so on), `ERR_RULE_NOT_ALLOWED`,
/// `ERR_VERSION_CHANGED`.
#[tauri::command]
pub async fn settings_set_charges<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: mandi_services::core::rules::ChargesInput,
) -> Result<mandi_services::core::rules::ChargesSettings, UiError> {
    run(app, "settings_set_charges", move |ctx| {
        let actor = auth::require(ctx)?;
        mandi_services::rules::set_charges(ctx, &actor, &input)
    })
    .await
}

/// The switches of Section 21 "Overrides" and "Checks": allow a typed commission or لاگا on a
/// sale line, and whether going over a credit limit warns or blocks.
#[tauri::command]
pub async fn settings_get_overrides<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<mandi_services::core::rules::OverridesSettings, UiError> {
    run(app, "settings_get_overrides", |ctx| {
        auth::require(ctx)?;
        mandi_services::rules::overrides(ctx)
    })
    .await
}

/// Saves the two switches (audit row with old and new value).
#[tauri::command]
pub async fn settings_set_overrides<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: mandi_services::core::rules::OverridesSettings,
) -> Result<mandi_services::core::rules::OverridesSettings, UiError> {
    run(app, "settings_set_overrides", move |ctx| {
        let actor = auth::require(ctx)?;
        mandi_services::rules::set_overrides(ctx, &actor, &input)
    })
    .await
}

/// The Printing settings of PRD Section 21: paper per document (starts: seller bill A4, buyer
/// slips A5, پختہ slip A5), printer (empty = the default printer), copies (1), Urdu digits on
/// Urdu prints (off), and the list "print after save" of the signed-in user (PRD FR-7.57).
#[tauri::command]
pub async fn settings_get_printing<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<mandi_services::core::printing::PrintingSettings, UiError> {
    run(app, "settings_get_printing", |ctx| {
        let actor = auth::require(ctx)?;
        mandi_services::printing::get_settings(ctx, &actor)
    })
    .await
}

/// Saves the Printing settings, with an audit row (old and new value). Errors: the copies codes
/// on `copies` (`ERR_REQUIRED`, `ERR_NOT_A_NUMBER`, `ERR_OUT_OF_RANGE` with `min` and `max`),
/// `ERR_TEXT_TOO_LONG` or `ERR_CONTROL_CHARS` on `printer`.
#[tauri::command]
pub async fn settings_set_printing<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: mandi_services::core::printing::SetPrintingInput,
) -> Result<mandi_services::core::printing::PrintingSettings, UiError> {
    run(app, "settings_set_printing", move |ctx| {
        let actor = auth::require(ctx)?;
        mandi_services::printing::set_settings(ctx, &actor, &input)
    })
    .await
}
