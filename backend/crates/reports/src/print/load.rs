//! Reads a saved بکری and the settings that decide how it is printed, then builds the print data
//! (plan 14). The only part of the print data that touches the database.

use mandi_core::money::Grouping;
use mandi_core::printing::PrintSetup;
use mandi_db::Db;
use mandi_db::bikri::saved;
use mandi_db::rusqlite::{Connection, OptionalExtension};
use serde_json::Value;

use super::build::{BuildOptions, build};
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
}

/// Reads the saved بکری and the settings, and builds the print data of one document.
///
/// # Errors
/// `ReportsError::NotFound` if the بکری does not exist; a database or data error otherwise.
pub fn bikri_print_data(db: &Db, input: &BikriPrintDataInput) -> Result<PrintData, ReportsError> {
    let (detail, firm, grouping, setup) = db.read(|conn| {
        let detail = saved::get(conn, input.bikri_id)?.ok_or(ReportsError::NotFound)?;
        Ok::<_, ReportsError>((
            detail,
            load_firm(conn)?,
            load_grouping(conn)?,
            load_setup(conn)?,
        ))
    })?;
    let options = BuildOptions {
        doc: input.doc,
        language: input.language,
        grouping,
        setup,
        firm,
        now_utc: db.clock().now_utc(),
        only_buyer_id: input.only_buyer_id,
    };
    build(&detail, &options)
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

pub(super) fn load_firm(conn: &Connection) -> Result<FirmNames, ReportsError> {
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
    })
}

pub(super) fn load_grouping(conn: &Connection) -> Result<Grouping, ReportsError> {
    Ok(setting(conn, KEY_NUMBER_GROUPING)?
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default())
}

/// The saved Printing setup, or the starting values (A21). A damaged value never stops a print.
pub(super) fn load_setup(conn: &Connection) -> Result<PrintSetup, ReportsError> {
    Ok(setting(conn, mandi_core::printing::SETTING_KEY)?
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default())
}
