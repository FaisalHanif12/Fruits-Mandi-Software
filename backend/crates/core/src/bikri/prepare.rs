//! Reading the typed text of a draft: every value that cannot be read is left `None` and its
//! mistake goes to the issues with its field.

use super::input::*;
use super::prepared::*;
use super::{MAX_LOTS, MAX_SALE_LINES};
use crate::dates::BusinessDate;
use crate::error::{ErrorCode, ErrorValue, FieldError};
use crate::money::{Paisa, Qty};
use crate::rules::ChargeValue;
use crate::validate;

fn header_field(name: &str) -> String {
    format!("header.{name}")
}

fn lot_field(lot: usize, name: &str) -> String {
    format!("lots[{lot}].{name}")
}

fn sale_field(lot: usize, sale: usize, name: &str) -> String {
    format!("lots[{lot}].sales[{sale}].{name}")
}

/// Takes a value out of a result. A mistake goes to `issues`.
fn take<T>(issues: &mut Vec<FieldError>, result: Result<T, FieldError>) -> Option<T> {
    match result {
        Ok(value) => Some(value),
        Err(error) => {
            issues.push(error);
            None
        }
    }
}

/// True if the text is empty once spaces are removed.
fn is_blank(text: &str) -> bool {
    text.trim().is_empty()
}

/// Optional text: trimmed, `None` if empty, checked like a note.
fn optional_text(
    issues: &mut Vec<FieldError>,
    field: &str,
    text: Option<&String>,
) -> Option<String> {
    let text = text?;
    if is_blank(text) {
        return None;
    }
    take(issues, validate::free_text(field, text)).filter(|t| !t.is_empty())
}

/// An optional whole-rupee amount. Empty is 0. A bad value is `None` and an error.
fn optional_money(
    issues: &mut Vec<FieldError>,
    field: &str,
    text: Option<&String>,
) -> Option<Paisa> {
    match text {
        None => Some(Paisa::ZERO),
        Some(t) if is_blank(t) => Some(Paisa::ZERO),
        Some(t) => take(issues, validate::money(field, t)),
    }
}

/// Reads an optional note into `slot`. False if the note is not clean.
fn optional_note(
    issues: &mut Vec<FieldError>,
    field: &str,
    text: Option<&String>,
    slot: &mut Option<String>,
) -> bool {
    let Some(text) = text.filter(|t| !is_blank(t)) else {
        return true;
    };
    match take(issues, validate::free_text(field, text)) {
        Some(clean) => {
            *slot = Some(clean).filter(|c| !c.is_empty());
            true
        }
        None => false,
    }
}

fn pukhta_of(input: Option<&PukhtaInput>) -> PreparedPukhta {
    input.map_or_else(PreparedPukhta::default, |p| PreparedPukhta {
        name_ur: p.name_ur.trim().to_owned(),
        name_en: p.name_en.trim().to_owned(),
        phone: p.phone.trim().to_owned(),
    })
}

/// A lot charge typed by hand. Nothing typed (no input, or an empty value) is `Ok(None)`.
fn typed_lot_charge(
    issues: &mut Vec<FieldError>,
    field: &str,
    input: Option<&ChargeInput>,
) -> (Option<TypedLotCharge>, bool) {
    let Some(input) = input else {
        return (None, true);
    };
    if is_blank(&input.value) {
        return (None, true);
    }
    match take(issues, validate::money(field, &input.value)) {
        Some(value) => (
            Some(TypedLotCharge {
                method: input.method,
                value,
            }),
            true,
        ),
        None => (None, false),
    }
}

/// A commission or لاگا typed on a line.
fn typed_line_charge(
    issues: &mut Vec<FieldError>,
    field: &str,
    input: Option<&LineChargeInput>,
) -> (Option<ChargeValue>, bool) {
    let Some(input) = input else {
        return (None, true);
    };
    if is_blank(&input.value) {
        return (None, true);
    }
    let value = match input.method {
        LineChargeMethod::Percent => {
            take(issues, validate::percent(field, &input.value)).map(ChargeValue::Percent)
        }
        LineChargeMethod::PerUnit => {
            take(issues, validate::money(field, &input.value)).map(ChargeValue::PerUnit)
        }
    };
    let ok = value.is_some();
    (value, ok)
}

fn prepare_sale(
    issues: &mut Vec<FieldError>,
    lot: usize,
    index: usize,
    input: &SaleInput,
) -> PreparedSale {
    let pukhta = pukhta_of(input.pukhta.as_ref());
    // A row with no buyer and no seller rate is a blank row (PRD FR-7.52). Its quantity may be
    // the quantity the screen filled in, so nothing else in it is read.
    let is_blank_row =
        input.buyer_account_id.is_none() && pukhta.is_empty() && is_blank(&input.seller_rate);
    let mut sale = PreparedSale {
        key: input.key.clone(),
        sale_id: input.sale_id,
        is_blank: is_blank_row,
        has_errors: false,
        buyer_account_id: input.buyer_account_id,
        pukhta,
        qty: None,
        weight: None,
        seller_rate: None,
        buyer_rate: None,
        rate_differs: input.rate_differs,
        payment_type: input.payment_type,
        seller_comm: None,
        buyer_comm: None,
        laga: None,
        note: None,
    };
    if is_blank_row {
        return sale;
    }
    let mut ok = true;
    sale.qty = take(
        issues,
        validate::qty(&sale_field(lot, index, "qty"), &input.qty),
    );
    ok &= sale.qty.is_some();
    sale.seller_rate = take(
        issues,
        validate::rate(&sale_field(lot, index, "seller_rate"), &input.seller_rate),
    );
    ok &= sale.seller_rate.is_some();
    if let Some(weight) = input.weight.as_ref().filter(|w| !is_blank(w)) {
        sale.weight = take(
            issues,
            validate::weight(&sale_field(lot, index, "weight"), weight),
        );
        ok &= sale.weight.is_some();
    }
    if input.rate_differs {
        let field = sale_field(lot, index, "buyer_rate");
        let text = input.buyer_rate.as_deref().unwrap_or("");
        sale.buyer_rate = take(issues, validate::rate(&field, text));
        ok &= sale.buyer_rate.is_some();
    }
    let (seller_comm, ok_a) = typed_line_charge(
        issues,
        &sale_field(lot, index, "seller_comm"),
        input.seller_comm.as_ref(),
    );
    let (buyer_comm, ok_b) = typed_line_charge(
        issues,
        &sale_field(lot, index, "buyer_comm"),
        input.buyer_comm.as_ref(),
    );
    let (laga, ok_c) =
        typed_line_charge(issues, &sale_field(lot, index, "laga"), input.laga.as_ref());
    sale.seller_comm = seller_comm;
    sale.buyer_comm = buyer_comm;
    sale.laga = laga;
    ok &= ok_a && ok_b && ok_c;
    // A note is not used by the maths, but it must be a clean note (PRD Section 4.2). The save
    // keeps it.
    ok &= optional_note(
        issues,
        &sale_field(lot, index, "note"),
        input.note.as_ref(),
        &mut sale.note,
    );
    sale.has_errors = !ok;
    sale
}

fn prepare_lot(
    issues: &mut Vec<FieldError>,
    index: usize,
    input: &LotInput,
    lines_left: &mut usize,
) -> PreparedLot {
    let mut ok = true;
    let qty_received = take(
        issues,
        validate::qty(&lot_field(index, "qty_received"), &input.qty_received),
    );
    ok &= qty_received.is_some();
    let weight_received = match input.weight_received.as_ref().filter(|w| !is_blank(w)) {
        Some(w) => {
            let weight = take(
                issues,
                validate::weight(&lot_field(index, "weight_received"), w),
            );
            ok &= weight.is_some();
            weight
        }
        None => None,
    };
    let freight = optional_money(issues, &lot_field(index, "freight"), input.freight.as_ref());
    // Empty نقد is "not typed" (None), not 0: a one-time seller's نقد then starts as his net
    // (PRD FR-13.10).
    let cash_paid = match input.cash_paid.as_ref().filter(|t| !is_blank(t)) {
        Some(text) => {
            let value = take(
                issues,
                validate::money(&lot_field(index, "cash_paid"), text),
            );
            ok &= value.is_some();
            value
        }
        None => None,
    };
    let store_rent = optional_money(
        issues,
        &lot_field(index, "store_rent"),
        input.store_rent.as_ref(),
    );
    ok &= freight.is_some() && store_rent.is_some();
    let (labour, ok_l) =
        typed_lot_charge(issues, &lot_field(index, "labour"), input.labour.as_ref());
    let (munshiana, ok_m) = typed_lot_charge(
        issues,
        &lot_field(index, "munshiana"),
        input.munshiana.as_ref(),
    );
    let (phone, ok_p) = typed_lot_charge(issues, &lot_field(index, "phone"), input.phone.as_ref());
    ok &= ok_l && ok_m && ok_p;

    let mut sales = Vec::new();
    for (sale_index, sale) in input.sales.iter().enumerate() {
        if *lines_left == 0 {
            issues.push(
                FieldError::new(ErrorCode::ErrOutOfRange, &lot_field(index, "sales")).with(
                    "max",
                    ErrorValue::Number(i64::try_from(MAX_SALE_LINES).unwrap_or(i64::MAX)),
                ),
            );
            break;
        }
        *lines_left = lines_left.saturating_sub(1);
        sales.push(prepare_sale(issues, index, sale_index, sale));
    }
    PreparedLot {
        key: input.key.clone(),
        lot_id: input.lot_id,
        has_errors: !ok,
        seller_account_id: input.seller_account_id,
        pukhta: pukhta_of(input.pukhta.as_ref()),
        item_id: input.item_id,
        packing_id: input.packing_id,
        marka_id: input.marka_id,
        store_id: input.store_id,
        qty_received,
        weight_received,
        freight: freight.unwrap_or(Paisa::ZERO),
        freight_paid_by: input.freight_paid_by,
        labour,
        munshiana,
        phone,
        cash_paid,
        store_rent: store_rent.unwrap_or(Paisa::ZERO),
        adjust_advance: input.adjust_advance,
        sales,
    }
}

fn prepare_header(
    issues: &mut Vec<FieldError>,
    input: &HeaderInput,
    today: BusinessDate,
    closed_up_to: Option<BusinessDate>,
) -> PreparedHeader {
    let entry_date = take(
        issues,
        validate::posting_date(
            &header_field("entry_date"),
            &input.entry_date,
            closed_up_to,
            today,
        ),
    );
    let bill_date = if is_blank(&input.bill_date) {
        entry_date
    } else {
        take(
            issues,
            validate::date(&header_field("bill_date"), &input.bill_date, today),
        )
    };
    // These fields are checked now and kept by the save (plan 13).
    let book = optional_text(issues, &header_field("book"), input.book.as_ref());
    let truck_no = optional_text(issues, &header_field("truck_no"), input.truck_no.as_ref());
    let driver_name = optional_text(
        issues,
        &header_field("driver_name"),
        input.driver_name.as_ref(),
    );
    let driver_phone = match input.driver_phone.as_ref().filter(|p| !is_blank(p)) {
        Some(phone) => take(
            issues,
            validate::phone(&header_field("driver_phone"), phone),
        ),
        None => None,
    };
    let note = optional_text(issues, &header_field("note"), input.note.as_ref());
    let total_freight = match input.total_freight.as_ref().filter(|t| !is_blank(t)) {
        Some(t) => take(issues, validate::money(&header_field("total_freight"), t)),
        None => None,
    };
    PreparedHeader {
        entry_date,
        bill_date,
        book,
        truck_no,
        driver_name,
        driver_phone,
        note,
        zamindar_account_id: input.zamindar_account_id,
        adda_id: input.adda_id,
        store_id: input.store_id,
        total_freight,
    }
}

/// Reads a typed draft (Code Standards 4.1 layer 2). Every value that cannot be read is left
/// `None` and its mistake goes to `issues` with its field, so the user fixes them all at once.
/// More than 50 lots or 500 lines is cut off with `ERR_OUT_OF_RANGE` (NFR-SCL-3).
///
/// Example: a line with qty `20+5` and rate `600` is read as qty 25 and rate 600 rupees; a line
/// with qty `abc` has no qty and an `ERR_NOT_A_NUMBER` on `lots[0].sales[0].qty`.
pub fn prepare_draft(
    draft: &BikriDraft,
    today: BusinessDate,
    closed_up_to: Option<BusinessDate>,
) -> PreparedDraft {
    let mut issues = Vec::new();
    let header = prepare_header(&mut issues, &draft.header, today, closed_up_to);
    if draft.lots.len() > MAX_LOTS {
        issues.push(FieldError::new(ErrorCode::ErrOutOfRange, "lots").with(
            "max",
            ErrorValue::Number(i64::try_from(MAX_LOTS).unwrap_or(i64::MAX)),
        ));
    }
    let mut lines_left = MAX_SALE_LINES;
    let lots = draft
        .lots
        .iter()
        .take(MAX_LOTS)
        .enumerate()
        .map(|(index, lot)| prepare_lot(&mut issues, index, lot, &mut lines_left))
        .collect();
    PreparedDraft {
        header,
        lots,
        issues,
    }
}

/// Splits a header freight total over lots (PRD C17), reading the text first.
///
/// Example (C17): 1,000 over lots of 30 and 20 نگ gives 600 and 400.
///
/// # Errors
/// The money and quantity codes on `total_freight` and `lots[i].qty_received`, all together.
pub fn split_freight(input: &SplitFreightInput) -> Result<Vec<FreightPart>, Vec<FieldError>> {
    let mut issues = Vec::new();
    let total = take(
        &mut issues,
        validate::money("total_freight", &input.total_freight),
    );
    let mut parts: Vec<(usize, Qty)> = Vec::with_capacity(input.lots.len());
    if input.lots.len() > MAX_LOTS {
        issues.push(FieldError::new(ErrorCode::ErrOutOfRange, "lots").with(
            "max",
            ErrorValue::Number(i64::try_from(MAX_LOTS).unwrap_or(i64::MAX)),
        ));
    }
    if input.lots.is_empty() {
        issues.push(FieldError::new(ErrorCode::ErrNoLots, "lots"));
    }
    for (index, lot) in input.lots.iter().enumerate() {
        let field = format!("lots[{index}].qty_received");
        if let Some(qty) = take(&mut issues, validate::qty(&field, &lot.qty_received)) {
            parts.push((index, qty));
        }
    }
    let Some(total) = total else {
        return Err(issues);
    };
    if !issues.is_empty() {
        return Err(issues);
    }
    // The lot number is the position, so on a tie in نگ the first lot takes the leftover.
    let split = crate::money::split_by_qty(total, &parts)
        .map_err(|e| vec![FieldError::new(e.code(), "total_freight")])?;
    let mut out = Vec::with_capacity(split.len());
    for (index, freight_paisa) in split {
        let key = input
            .lots
            .get(index)
            .map(|lot| lot.key.clone())
            .unwrap_or_default();
        out.push(FreightPart { key, freight_paisa });
    }
    Ok(out)
}
