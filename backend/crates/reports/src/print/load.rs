//! Reads a saved بکری and the settings that decide how it is printed, then builds the print data
//! (plan 14). The only part of the print data that touches the database.

use mandi_core::edit::EditStamp;
use mandi_core::firm::{
    FirmContact, FirmRole, MAX_CONTACTS, SETTING_KEY_CONTACTS, SETTING_KEY_TRADE_MARK,
};
use mandi_core::money::Grouping;
use mandi_core::printing::PrintSetup;
use mandi_db::Db;
use mandi_db::bikri::saved;
use mandi_db::rusqlite::{Connection, OptionalExtension};
use serde_json::Value;

use super::build::{BuildOptions, build};
use super::format::NumberStyle;
use super::{BikriPrintDataInput, PrintData};
use crate::error::ReportsError;

// The keys other services write. `firm` and `number_grouping` belong to `services::settings`;
// the Printing setup key is shared through `mandi_core::printing::SETTING_KEY`.
const KEY_FIRM: &str = "firm";
const KEY_NUMBER_GROUPING: &str = "number_grouping";

/// The firm names and details as saved in Settings "Firm" (both scripts).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FirmNames {
    /// Firm name, Urdu letters.
    pub name_ur: String,
    /// Firm name, English letters.
    pub name_en: String,
    /// Mandi name, Urdu letters.
    pub mandi_ur: String,
    /// Mandi name, English letters.
    pub mandi_en: String,
    /// Address as typed.
    pub address: String,
    /// Phone as typed.
    pub phone: String,
    /// The contact persons for the print header, at most 3 (PRD Section 21, plan 17e). Empty if
    /// none are set.
    pub contacts: Vec<FirmContact>,
    /// The trade mark text as typed. Empty if not set.
    pub trade_mark: String,
}

/// Reads the saved بکری and the settings, and builds the print data of one document.
///
/// # Errors
/// `ReportsError::NotFound` if the بکری does not exist; a database or data error otherwise.
pub fn bikri_print_data(db: &Db, input: &BikriPrintDataInput) -> Result<PrintData, ReportsError> {
    let (detail, firm, grouping, setup, revised) = db.read(|conn| {
        let detail = saved::get(conn, input.bikri_id)?.ok_or(ReportsError::NotFound)?;
        Ok::<_, ReportsError>((
            detail,
            load_firm(conn)?,
            load_grouping(conn)?,
            load_setup(conn)?,
            last_edit_day(conn, "bikri", input.bikri_id)?,
        ))
    })?;
    let style = NumberStyle::new(grouping);
    let options = BuildOptions {
        doc: input.doc,
        language: input.language,
        grouping,
        setup,
        firm,
        now_utc: db.clock().now_utc(),
        only_buyer_id: input.only_buyer_id,
    };
    let mut data = build(&detail, &options)?;
    data.revised_on = revised.map(|day| style.date(&day)).transpose()?;
    Ok(data)
}

/// The Pakistan day (`YYYY-MM-DD`) of the last edit of an entry, from the audit log. `None` if the
/// entry was never edited (PRD FR-7.71).
pub(super) fn last_edit_day(
    conn: &Connection,
    entity: &str,
    id: i64,
) -> Result<Option<String>, ReportsError> {
    let last = mandi_db::edit_stamps::last_edits(conn, entity, &[id])?;
    last.get(&id)
        .map(|last| {
            EditStamp::from_utc(&last.at, String::new(), last.count, None)
                .map(|stamp| stamp.local_date)
                .ok_or_else(|| {
                    ReportsError::Data(format!("an audit time is not a time: {}", last.at))
                })
        })
        .transpose()
}

/// A setting as JSON, if it was ever saved.
fn setting(conn: &Connection, key: &str) -> Result<Option<Value>, ReportsError> {
    let text: Option<String> = conn
        .query_row(
            "SELECT value_json FROM settings WHERE key = ?1",
            [key],
            |row| row.get(0),
        )
        .optional()?;
    text.map(|text| {
        serde_json::from_str(&text)
            .map_err(|error| ReportsError::Data(format!("setting {key}: {error}")))
    })
    .transpose()
}

pub(crate) fn load_firm(conn: &Connection) -> Result<FirmNames, ReportsError> {
    let value = setting(conn, KEY_FIRM)?;
    let text = |key: &str| {
        value
            .as_ref()
            .and_then(|v| v.get(key))
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned()
    };
    Ok(FirmNames {
        name_ur: text("name_ur"),
        name_en: text("name_en"),
        mandi_ur: text("mandi_ur"),
        mandi_en: text("mandi_en"),
        address: text("address"),
        phone: text("phone"),
        contacts: load_contacts(conn)?,
        trade_mark: setting(conn, SETTING_KEY_TRADE_MARK)?
            .as_ref()
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned(),
    })
}

/// The language of the readable backup pack: the saved default language, else Urdu (PRD
/// Section 21, FR-7.30).
pub(crate) fn load_default_language(
    conn: &Connection,
) -> Result<mandi_core::auth::Language, ReportsError> {
    Ok(setting(conn, "default_language")?
        .as_ref()
        .and_then(Value::as_str)
        .and_then(mandi_core::auth::Language::parse)
        .unwrap_or(mandi_core::auth::Language::Ur))
}

/// The most active munshis of the staff list that the header adds (plan 53 T4).
const MAX_STAFF_MUNSHIS: usize = 3;

/// The people of every print header (PRD Section 21, v1.23, plan 53 T4): the people typed in
/// Settings, in their order, then the active staff whose job is منشی, by staff id. A staff munshi who
/// is in the Settings list too (the same Urdu name and phone) prints once. At most
/// [`MAX_STAFF_MUNSHIS`] munshis come from the staff list and at most [`MAX_CONTACTS`] people in
/// all. A damaged Settings value gives no typed people, so a print is never stopped by it.
///
/// Example: Settings has Maher Adnan (owner); staff has آصف (munshi, active) and اسلم (collector):
/// the header has Maher Adnan, then آصف. Make آصف inactive and add بلال: Maher Adnan, then بلال.
fn load_contacts(conn: &Connection) -> Result<Vec<FirmContact>, ReportsError> {
    let mut contacts: Vec<FirmContact> = setting(conn, SETTING_KEY_CONTACTS)?
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default();
    contacts.truncate(MAX_CONTACTS);
    // The same person: the same Urdu name (after the search normalisation) and the same phone digits.
    let key = |name_ur: &str, phone: &str| {
        (
            mandi_core::text::search_ur(name_ur),
            phone
                .chars()
                .filter(char::is_ascii_digit)
                .collect::<String>(),
        )
    };
    let mut statement = conn.prepare(
        "SELECT name_ur, name_en, COALESCE(phone, '') FROM staff \
         WHERE role = 'munshi' AND is_active = 1 ORDER BY id",
    )?;
    let munshis = statement.query_map([], |row| {
        Ok(FirmContact {
            name_ur: row.get(0)?,
            name_en: row.get(1)?,
            role: FirmRole::Munshi,
            phone: row.get(2)?,
        })
    })?;
    let mut added = 0usize;
    for munshi in munshis {
        let munshi = munshi?;
        if added >= MAX_STAFF_MUNSHIS || contacts.len() >= MAX_CONTACTS {
            break;
        }
        let same = key(&munshi.name_ur, &munshi.phone);
        if contacts
            .iter()
            .any(|have| key(&have.name_ur, &have.phone) == same)
        {
            continue;
        }
        contacts.push(munshi);
        added = added.saturating_add(1);
    }
    Ok(contacts)
}

pub(crate) fn load_grouping(conn: &Connection) -> Result<Grouping, ReportsError> {
    Ok(setting(conn, KEY_NUMBER_GROUPING)?
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default())
}

/// The saved Printing setup, or the starting values (A21). A damaged value never stops a print.
pub(crate) fn load_setup(conn: &Connection) -> Result<PrintSetup, ReportsError> {
    Ok(setting(conn, mandi_core::printing::SETTING_KEY)?
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default())
}
