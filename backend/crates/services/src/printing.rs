//! Printing: the Printing settings and the record of a print (plan 14, PRD Section 21
//! "Printing", FR-7.57, FR-12.2 print count, FR-21.3 audit).
//!
//! The print data itself is built by `mandi-reports`; the print call is made by the app. This
//! module holds what needs the database and the audit: reading and saving the settings, checking
//! a print request, and raising the print count of a بکری together with its audit row.

use mandi_db::bikri::saved;
use mandi_db::rusqlite::{Connection, OptionalExtension, params};
use mandi_domain::core::auth::Language;
use mandi_domain::core::error::ErrorCode;
use mandi_domain::core::printing::{
    self, CollectionFilterNames, MAX_PRINTER_CHARS, PRINT_AFTER_SAVE_PREF, PrintCollectionInput,
    PrintDoc, PrintInput, PrintSetup, PrintTo, PrintVoucherInput, PrintingSettings, SETTING_KEY,
    SetPrintingInput,
};
use mandi_domain::core::validate::{self, FieldErrors};
use mandi_domain::core::vouchers::{CollectionList, CollectionListInput};
use serde_json::{Value, json};

use crate::audit::Event;
use crate::auth::Actor;
use crate::context::Context;
use crate::error::ServiceError;
use crate::store;

/// The stored per-computer setup, or the starting values (A21). A value that cannot be read
/// never stops a print.
fn load_setup(conn: &Connection) -> Result<PrintSetup, ServiceError> {
    Ok(store::setting(conn, SETTING_KEY)?
        .and_then(|value| serde_json::from_value(value).ok())
        .unwrap_or_default())
}

/// The documents ticked in "print after save" for one user, or the starting list.
fn load_after_save(conn: &Connection, user_id: i64) -> Result<Vec<PrintDoc>, ServiceError> {
    Ok(store::pref(conn, user_id, PRINT_AFTER_SAVE_PREF)?
        .and_then(|value| serde_json::from_value(value).ok())
        .unwrap_or_else(printing::default_print_after_save))
}

fn load(conn: &Connection, user_id: i64) -> Result<PrintingSettings, ServiceError> {
    Ok(PrintingSettings::from_parts(
        load_setup(conn)?,
        load_after_save(conn, user_id)?,
    ))
}

fn settings_json(settings: &PrintingSettings) -> Result<Value, ServiceError> {
    serde_json::to_value(settings.setup())
        .map(|mut value| {
            if let Value::Object(map) = &mut value {
                map.insert(
                    "print_after_save".to_owned(),
                    json!(settings.print_after_save),
                );
            }
            value
        })
        .map_err(|error| ServiceError::Data(format!("printing settings: {error}")))
}

/// The Printing settings of the signed-in user (PRD Section 21).
///
/// # Errors
/// `ServiceError::Db` or `ServiceError::Data`.
pub fn get_settings(ctx: &Context, actor: &Actor) -> Result<PrintingSettings, ServiceError> {
    ctx.db.read(|conn| load(conn, actor.user_id))
}

/// Saves the Printing settings. The paper, printer, copies and Urdu digits belong to this
/// computer; the list "print after save" belongs to the user (PRD FR-7.57). A change writes an
/// audit row with the old and the new value in the same transaction (Code Standards S7); asking
/// for the values it already has writes nothing.
///
/// # Errors
/// `ERR_REQUIRED`, `ERR_NOT_A_NUMBER` or `ERR_OUT_OF_RANGE` (values `min`, `max`) on `copies`;
/// `ERR_TEXT_TOO_LONG` or `ERR_CONTROL_CHARS` on `printer`.
pub fn set_settings(
    ctx: &Context,
    actor: &Actor,
    input: &SetPrintingInput,
) -> Result<PrintingSettings, ServiceError> {
    let mut errors = FieldErrors::default();
    let copies = errors.take(printing::parse_copies("copies", &input.copies));
    let printer = errors.take(printer_name(&input.printer));
    errors.finish()?;
    // `finish` returned an error if either failed, so both values are there.
    let (copies, printer) = (copies.unwrap_or_default(), printer.unwrap_or_default());
    let new = PrintingSettings {
        paper_seller_bill: input.paper_seller_bill,
        paper_buyer_slips: input.paper_buyer_slips,
        paper_pukhta_slip: input.paper_pukhta_slip,
        paper_receipt_slip: input.paper_receipt_slip,
        paper_collection_list: input.paper_collection_list,
        printer,
        copies,
        urdu_digits: input.urdu_digits,
        print_after_save: printing::unique_docs(&input.print_after_save),
    };
    ctx.db.write(|tx| {
        let old = load(tx, actor.user_id)?;
        if old != new {
            let setup = serde_json::to_value(new.setup())
                .map_err(|error| ServiceError::Data(format!("printing settings: {error}")))?;
            store::set_setting(tx, SETTING_KEY, &setup, Some(actor.user_id))?;
            store::set_pref(
                tx,
                actor.user_id,
                PRINT_AFTER_SAVE_PREF,
                &json!(new.print_after_save),
            )?;
            ctx.db.auditor().write(
                tx,
                &Event::new(Some(actor.user_id), "edit", "settings")
                    .old(json!({ "printing": settings_json(&old)? }))
                    .new_value(json!({ "printing": settings_json(&new)? }))
                    .into_entry(),
            )?;
        }
        Ok::<(), ServiceError>(())
    })?;
    Ok(new)
}

/// A printer name as typed: trimmed, at most 259 characters, no control characters. Empty is
/// allowed and means the default printer.
fn printer_name(text: &str) -> Result<String, mandi_domain::core::error::FieldError> {
    let name = validate::free_text("printer", text)?;
    if name.chars().count() > MAX_PRINTER_CHARS {
        return Err(mandi_domain::core::error::FieldError::new(
            ErrorCode::ErrTextTooLong,
            "printer",
        ));
    }
    Ok(name)
}

/// A print request that passed its checks: what the app needs to print.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrintPlan {
    /// The بکری.
    pub bikri_id: i64,
    /// Its number, for the PDF file names.
    pub number: i64,
    /// The print language.
    pub language: Language,
    /// The documents, each once, in the order asked.
    pub docs: Vec<PrintDoc>,
    /// Printer or PDF.
    pub to: PrintTo,
    /// Copies of this print.
    pub copies: u32,
    /// The Printing settings: printer name and paper per document.
    pub setup: PrintSetup,
    /// Only this buyer's slip.
    pub only_buyer_id: Option<i64>,
}

/// Checks a print request and reads what it needs (the number of the بکری and the settings).
/// Nothing is written.
///
/// # Errors
/// `ERR_NOT_FOUND` on `bikri_id`; `ERR_REQUIRED` on `docs`; `ERR_FORMAT_INVALID` on `docs` for
/// `receipt_slip` or `collection_list`; the copies codes on `copies`.
pub fn plan(ctx: &Context, input: &PrintInput) -> Result<PrintPlan, ServiceError> {
    let mut errors = FieldErrors::default();
    let docs = printing::unique_docs(&input.docs);
    if docs.is_empty() {
        errors.push(mandi_domain::core::error::FieldError::new(
            ErrorCode::ErrRequired,
            "docs",
        ));
    }
    // The voucher slip and the collection list are not documents of a بکری: they have their own
    // commands (plan 17e). Asking for one here is refused, not turned into an internal error.
    if docs
        .iter()
        .any(|doc| matches!(doc, PrintDoc::ReceiptSlip | PrintDoc::CollectionList))
    {
        errors.push(mandi_domain::core::error::FieldError::new(
            ErrorCode::ErrFormatInvalid,
            "docs",
        ));
    }
    let typed_copies = input
        .copies
        .as_deref()
        .map(str::trim)
        .filter(|text| !text.is_empty());
    let copies = typed_copies.and_then(|text| errors.take(printing::parse_copies("copies", text)));
    errors.finish()?;
    ctx.db.read(|conn| {
        let number = saved::number_of(conn, input.bikri_id)?
            .ok_or_else(|| ServiceError::reject_field(ErrorCode::ErrNotFound, "bikri_id"))?;
        let setup = load_setup(conn)?;
        Ok(PrintPlan {
            bikri_id: input.bikri_id,
            number,
            language: input.language,
            docs,
            to: input.to,
            copies: copies.unwrap_or(setup.copies),
            setup,
            only_buyer_id: input.only_buyer_id,
        })
    })
}

/// Records a print that happened: raises the print count of the بکری by one and writes the audit
/// row, in one transaction (PRD 12.2, Code Standards B3, S7). Returns the new count. One print
/// action raises the count once, however many documents it printed. The row's `version` is not
/// bumped: a print changes no business value, and a screen that is editing the بکری must not be
/// told it changed.
///
/// # Errors
/// `ERR_NOT_FOUND` on `bikri_id`; `ServiceError::Db`.
pub fn record(
    ctx: &Context,
    actor: &Actor,
    plan: &PrintPlan,
    printed: &[PrintDoc],
) -> Result<i64, ServiceError> {
    ctx.db.write(|tx| {
        let changed = tx.execute(
            "UPDATE bikri_entries SET print_count = print_count + 1,
                 updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), updated_by = ?2
             WHERE id = ?1",
            params![plan.bikri_id, actor.user_id],
        )?;
        if changed == 0 {
            return Err(ServiceError::reject_field(
                ErrorCode::ErrNotFound,
                "bikri_id",
            ));
        }
        let count: i64 = tx.query_row(
            "SELECT print_count FROM bikri_entries WHERE id = ?1",
            [plan.bikri_id],
            |row| row.get(0),
        )?;
        ctx.db.auditor().write(
            tx,
            &Event::new(Some(actor.user_id), "print", "bikri")
                .entity_id(plan.bikri_id)
                .new_value(json!({
                    "number": plan.number,
                    "docs": printed,
                    "language": plan.language.as_str(),
                    "to": plan.to,
                    "copies": plan.copies,
                    "print_count": count,
                }))
                .into_entry(),
        )?;
        Ok(count)
    })
}

/// A voucher slip print request that passed its checks (plan 15, PRD FR-15.2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VoucherPrintPlan {
    /// The voucher.
    pub voucher_id: i64,
    /// Its number in its series, for the audit row.
    pub number: i64,
    /// Its kind, as stored, for the audit row.
    pub voucher_type: String,
    /// The print language.
    pub language: Language,
    /// Printer or PDF.
    pub to: PrintTo,
    /// Copies of this print.
    pub copies: u32,
    /// The Printing settings: printer name and paper.
    pub setup: PrintSetup,
}

/// Checks a voucher slip print request and reads what it needs. Nothing is written.
///
/// # Errors
/// `ERR_NOT_FOUND` on `voucher_id`; the copies codes on `copies`.
pub fn plan_voucher(
    ctx: &Context,
    input: &PrintVoucherInput,
) -> Result<VoucherPrintPlan, ServiceError> {
    let mut errors = FieldErrors::default();
    let typed_copies = input
        .copies
        .as_deref()
        .map(str::trim)
        .filter(|text| !text.is_empty());
    let copies = typed_copies.and_then(|text| errors.take(printing::parse_copies("copies", text)));
    errors.finish()?;
    ctx.db.read(|conn| {
        let (number, voucher_type): (i64, String) = conn
            .query_row(
                "SELECT number, type FROM vouchers WHERE id = ?1",
                [input.voucher_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .map_err(|error| match error {
                mandi_db::rusqlite::Error::QueryReturnedNoRows => {
                    ServiceError::reject_field(ErrorCode::ErrNotFound, "voucher_id")
                }
                other => ServiceError::from(other),
            })?;
        let setup = load_setup(conn)?;
        Ok(VoucherPrintPlan {
            voucher_id: input.voucher_id,
            number,
            voucher_type,
            language: input.language,
            to: input.to,
            copies: copies.unwrap_or(setup.copies),
            setup,
        })
    })
}

/// Records a voucher slip that was printed: an audit row (Code Standards S7). A voucher has no
/// print count, so the audit row is the record of the print (plan 15).
///
/// # Errors
/// `ServiceError::Db`.
pub fn record_voucher(
    ctx: &Context,
    actor: &Actor,
    plan: &VoucherPrintPlan,
) -> Result<(), ServiceError> {
    ctx.db.write(|tx| {
        ctx.db.auditor().write(
            tx,
            &Event::new(Some(actor.user_id), "print", "voucher")
                .entity_id(plan.voucher_id)
                .new_value(json!({
                    "type": plan.voucher_type,
                    "number": plan.number,
                    "doc": PrintDoc::ReceiptSlip,
                    "language": plan.language.as_str(),
                    "to": plan.to,
                    "copies": plan.copies,
                }))
                .into_entry(),
        )?;
        Ok(())
    })
}

/// The collection list of a day with the names of its filters, read for a print (plan 17e, PRD
/// FR-15.6). The list is the one the screen reads, `vouchers::collection_list`; the print builder
/// takes it as it is and checks that it adds up (two reads of the same value, accepted deviation
/// from Code Standards B2). Only reads.
///
/// # Errors
/// The date codes on `date`; `ERR_NOT_FOUND` on `collector_staff_id` or `city_id` for an id that
/// is not a staff member or a city.
pub fn collection_for_print(
    ctx: &Context,
    date: &str,
    collector_staff_id: Option<i64>,
    city_id: Option<i64>,
) -> Result<(CollectionList, CollectionFilterNames), ServiceError> {
    let names = |conn: &Connection, sql: &str, id: Option<i64>, field: &str| {
        let Some(id) = id else {
            return Ok(None);
        };
        conn.query_row(sql, [id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .optional()?
        .map(Some)
        .ok_or_else(|| ServiceError::reject_field(ErrorCode::ErrNotFound, field))
    };
    let filters = ctx.db.read(|conn| {
        Ok::<_, ServiceError>(CollectionFilterNames {
            collector: names(
                conn,
                "SELECT name_ur, name_en FROM staff WHERE id = ?1",
                collector_staff_id,
                "collector_staff_id",
            )?,
            city: names(
                conn,
                "SELECT name_ur, name_en FROM cities WHERE id = ?1",
                city_id,
                "city_id",
            )?,
        })
    })?;
    let list = crate::vouchers::collection_list(
        ctx,
        &CollectionListInput {
            date: date.to_owned(),
            collector_staff_id,
            city_id,
        },
    )?;
    Ok((list, filters))
}

/// A collection list print request that passed its checks (plan 17e).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollectionPrintPlan {
    /// The list, as the screen reads it.
    pub list: CollectionList,
    /// The names of the collector and the city it is filtered by.
    pub filters: CollectionFilterNames,
    /// The day as the number `YYYYMMDD`, for the PDF file name.
    pub file_number: i64,
    /// The collector filter, as the id the print route carries.
    pub collector_staff_id: Option<i64>,
    /// The city filter, as the id the print route carries.
    pub city_id: Option<i64>,
    /// The print language.
    pub language: Language,
    /// Printer or PDF.
    pub to: PrintTo,
    /// Copies of this print.
    pub copies: u32,
    /// The Printing settings: printer name and paper.
    pub setup: PrintSetup,
}

/// Checks a collection list print request and reads what it needs. Nothing is written.
///
/// # Errors
/// As [`collection_for_print`], and the copies codes on `copies`.
pub fn plan_collection(
    ctx: &Context,
    input: &PrintCollectionInput,
) -> Result<CollectionPrintPlan, ServiceError> {
    let mut errors = FieldErrors::default();
    let typed_copies = input
        .copies
        .as_deref()
        .map(str::trim)
        .filter(|text| !text.is_empty());
    let copies = typed_copies.and_then(|text| errors.take(printing::parse_copies("copies", text)));
    errors.finish()?;
    let (list, filters) =
        collection_for_print(ctx, &input.date, input.collector_staff_id, input.city_id)?;
    let file_number = list
        .date
        .replace('-', "")
        .parse::<i64>()
        .map_err(|_| ServiceError::Data(format!("collection list date {}", list.date)))?;
    let setup = ctx.db.read(load_setup)?;
    Ok(CollectionPrintPlan {
        file_number,
        collector_staff_id: input.collector_staff_id,
        city_id: input.city_id,
        language: input.language,
        to: input.to,
        copies: copies.unwrap_or(setup.copies),
        setup,
        list,
        filters,
    })
}

/// Records a collection list that was printed: an audit row (Code Standards S7). The list has no
/// record of its own, so the audit row is the record of the print.
///
/// # Errors
/// `ServiceError::Db`.
pub fn record_collection(
    ctx: &Context,
    actor: &Actor,
    plan: &CollectionPrintPlan,
) -> Result<(), ServiceError> {
    let buyers: usize = plan.list.groups.iter().map(|g| g.buyers.len()).sum();
    ctx.db.write(|tx| {
        ctx.db.auditor().write(
            tx,
            &Event::new(Some(actor.user_id), "print", "collection_list")
                .new_value(json!({
                    "date": plan.list.date,
                    "buyers": buyers,
                    "total_paisa": plan.list.total_paisa.as_paisa(),
                    "doc": PrintDoc::CollectionList,
                    "language": plan.language.as_str(),
                    "to": plan.to,
                    "copies": plan.copies,
                }))
                .into_entry(),
        )?;
        Ok(())
    })
}
