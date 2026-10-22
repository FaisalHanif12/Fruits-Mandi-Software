//! What an edit knows about a saved posting, and the draft the screen edits (plan 16, PRD
//! FR-12.18 to FR-12.20).
//!
//! Everything here is read from the STORED rows of `bikri_get` (never worked out again, Code
//! Standards B20): the draft is built from them in the shape of the book screen, a charge the user
//! typed comes back as a typed charge and a charge a rule decided comes back empty, and the
//! domain gets the same rows as `StoredPosting` so it can tell which lines the user changed.

use std::collections::HashMap;

use mandi_db::bikri::edit::LotState;
use mandi_db::bikri::saved as saved_repo;
use mandi_db::rules as rules_repo;
use mandi_db::rusqlite::Connection;
use mandi_domain::bikri::{
    PostingKind, StoredCarrier, StoredCharge, StoredIdentity, StoredLot, StoredLotCharge,
    StoredPosting, StoredSale,
};
use mandi_domain::core::bikri::{
    BikriDetail, BikriDraft, ChargeInput, FreightPaidBy, HeaderInput, LineChargeInput,
    LineChargeMethod, LotChargeMethod, LotInput, PukhtaInput, SaleInput, SavedCharge, SavedLot,
    SavedLotCharge, SavedSale, StoredMethod,
};
use mandi_domain::core::dates::{BusinessDate, format_ddmmyyyy};
use mandi_domain::core::error::ErrorCode;
use mandi_domain::core::money::{Bps, Paisa};
use mandi_domain::core::rules::{RuleLevel, percent_text};

use super::rows::sum;
use super::shared::data;
use crate::error::{ServiceError, money_error};

/// A saved بکری, or `ERR_NOT_FOUND` on `field`.
///
/// # Errors
/// `ERR_NOT_FOUND` on `field`; `ServiceError::Db`.
pub(super) fn detail_of(
    conn: &Connection,
    bikri_id: i64,
    field: &str,
) -> Result<BikriDetail, ServiceError> {
    saved_repo::get(conn, bikri_id)?
        .ok_or_else(|| ServiceError::reject_field(ErrorCode::ErrNotFound, field))
}

/// The lines of a lot that belong to one posting and are in force.
fn lines_of(lot: &SavedLot, entry_id: i64) -> impl Iterator<Item = &SavedSale> {
    lot.sales
        .iter()
        .filter(move |s| s.journal_entry_id == entry_id && s.is_active)
}

/// The lots a posting touches, each with only the lines of that posting in force. A first posting
/// touches every lot of the بکری (a lot with no line there still has its charges); a later posting
/// touches the lots that have a line in it.
pub(super) fn posting_lots(
    detail: &BikriDetail,
    entry_id: i64,
    kind: PostingKind,
) -> Vec<SavedLot> {
    detail
        .lots
        .iter()
        .filter(|lot| kind == PostingKind::First || lines_of(lot, entry_id).next().is_some())
        .map(|lot| {
            let mut lot = lot.clone();
            lot.sales
                .retain(|s| s.journal_entry_id == entry_id && s.is_active);
            lot
        })
        .collect()
}

/// The level of a rule, read from its row. `None` for a charge no rule decided.
fn level_of_rule(
    conn: &Connection,
    rule_id: Option<i64>,
) -> Result<Option<RuleLevel>, ServiceError> {
    match rule_id {
        None => Ok(None),
        Some(id) => Ok(rules_repo::get(conn, id)?.map(|row| row.level)),
    }
}

fn stored_charge(conn: &Connection, c: &SavedCharge) -> Result<StoredCharge, ServiceError> {
    Ok(StoredCharge {
        method: c.method,
        value_bps: c.value_bps,
        value_paisa: c.value_paisa,
        amount: c.amount,
        rule_id: c.rule_id,
        level: level_of_rule(conn, c.rule_id)?,
    })
}

fn stored_lot_charge(
    conn: &Connection,
    c: &SavedLotCharge,
) -> Result<StoredLotCharge, ServiceError> {
    Ok(StoredLotCharge {
        method: c.method,
        value: c.value_paisa,
        amount: c.amount,
        rule_id: c.rule_id,
        level: level_of_rule(conn, c.rule_id)?,
    })
}

/// What the domain needs to know about a posting being edited: its lines and, for each lot, what
/// is sold elsewhere, the lot charges and the line that carries the per-lot commission.
///
/// # Errors
/// `ServiceError::Db`; `ServiceError::Data` for a sum that does not fit.
pub(super) fn stored_posting(
    conn: &Connection,
    detail: &BikriDetail,
    entry_id: i64,
    kind: PostingKind,
) -> Result<StoredPosting, ServiceError> {
    let mut lots = HashMap::new();
    let mut sales = HashMap::new();
    for lot in &detail.lots {
        let elsewhere = lot
            .sales
            .iter()
            .filter(|s| s.is_active && s.journal_entry_id != entry_id)
            .try_fold(0_i64, |total, s| total.checked_add(s.qty))
            .and_then(|total| total.checked_add(lot.qty_written_off))
            .ok_or_else(|| data("the quantity of a lot does not fit"))?;
        let mut carrier = None;
        for sale in lines_of(lot, entry_id) {
            let seller_comm = stored_charge(conn, &sale.seller_comm)?;
            if kind == PostingKind::First && carrier.is_none() && seller_comm.carries_per_lot() {
                carrier = Some(StoredCarrier {
                    sale_id: sale.id,
                    charge: seller_comm.clone(),
                });
            }
            sales.insert(
                sale.id,
                StoredSale {
                    lot_id: lot.id,
                    buyer_account_id: sale.buyer.id,
                    person_key: sale.pukhta.as_ref().map(|p| p.person_key.clone()),
                    qty: sale.qty,
                    weight_grams: sale.weight_grams,
                    seller_rate: sale.seller_rate,
                    buyer_rate: sale.buyer_rate,
                    payment_type: sale.payment_type,
                    seller_comm,
                    buyer_comm: stored_charge(conn, &sale.buyer_comm)?,
                    laga: stored_charge(conn, &sale.laga)?,
                    cost: sale.cost_paisa,
                },
            );
        }
        let charges = kind == PostingKind::First;
        lots.insert(
            lot.id,
            StoredLot {
                identity: Some(StoredIdentity {
                    seller_account_id: lot.seller.id,
                    item_id: lot.item.id,
                    person_key: lot.pukhta.as_ref().map(|p| p.person_key.clone()),
                    purchase_id: lot.purchase.as_ref().map(|p| p.id),
                }),
                carrier,
                elsewhere_qty: elsewhere,
                labour: charges
                    .then(|| stored_lot_charge(conn, &lot.labour))
                    .transpose()?,
                munshiana: charges
                    .then(|| stored_lot_charge(conn, &lot.munshiana))
                    .transpose()?,
                phone: charges
                    .then(|| stored_lot_charge(conn, &lot.phone))
                    .transpose()?,
            },
        );
    }
    Ok(StoredPosting { kind, lots, sales })
}

// ---- The draft ----

fn rupees(amount: Paisa) -> String {
    amount.rupees().to_string()
}

/// Kilos from grams, up to 3 decimals, with no trailing zeros: 12,345 g is "12.345", 12,500 g
/// "12.5", 12,000 g "12".
fn kilos(grams: i64) -> String {
    let whole = grams.div_euclid(1000);
    let fraction = grams.rem_euclid(1000);
    if fraction == 0 {
        return whole.to_string();
    }
    let text = format!("{whole}.{fraction:03}");
    text.trim_end_matches('0').to_owned()
}

/// A commission or لاگا as the user typed it, or `None` if a rule decided it.
fn typed_line_charge(charge: &SavedCharge) -> Option<LineChargeInput> {
    if charge.method != StoredMethod::Manual {
        return None;
    }
    match (charge.value_bps, charge.value_paisa) {
        (Some(bps), None) => Bps::new(bps).ok().map(|bps| LineChargeInput {
            method: LineChargeMethod::Percent,
            value: percent_text(bps),
        }),
        (None, Some(paisa)) => Some(LineChargeInput {
            method: LineChargeMethod::PerUnit,
            value: rupees(paisa),
        }),
        _ => None,
    }
}

/// A lot charge as the user typed it, or `None` if a rule decided it or nothing was typed.
fn typed_lot_charge(charge: &SavedLotCharge) -> Option<ChargeInput> {
    if charge.rule_id.is_some() {
        return None;
    }
    match charge.method {
        LotChargeMethod::Manual => (charge.amount > Paisa::ZERO).then(|| ChargeInput {
            method: LotChargeMethod::Manual,
            value: rupees(charge.amount),
        }),
        method => Some(ChargeInput {
            method,
            value: rupees(charge.value_paisa),
        }),
    }
}

fn pukhta_input(person: &mandi_domain::core::bikri::SavedPukhta) -> PukhtaInput {
    PukhtaInput {
        name_ur: person.name_ur.clone(),
        name_en: person.name_en.clone(),
        phone: person.phone.clone(),
    }
}

fn sale_input(sale: &SavedSale, keep_ids: bool) -> SaleInput {
    let differs = sale.buyer_rate != sale.seller_rate;
    SaleInput {
        key: format!("sale:{}", sale.id),
        sale_id: keep_ids.then_some(sale.id),
        buyer_account_id: Some(sale.buyer.id),
        pukhta: sale.pukhta.as_ref().map(pukhta_input),
        qty: sale.qty.to_string(),
        weight: sale.weight_grams.map(kilos),
        seller_rate: rupees(sale.seller_rate),
        rate_differs: differs,
        buyer_rate: differs.then(|| rupees(sale.buyer_rate)),
        payment_type: Some(sale.payment_type),
        seller_comm: typed_line_charge(&sale.seller_comm),
        buyer_comm: typed_line_charge(&sale.buyer_comm),
        laga: typed_line_charge(&sale.laga),
        note: sale.note.clone(),
    }
}

/// What the lot credits its seller before نقد in the lines of this posting (C8): gross less the
/// commissions and less the lot charges. Used to tell a one-time seller who was paid his whole
/// net on the spot.
fn credit_of(lot: &SavedLot, entry_id: i64) -> Result<Paisa, ServiceError> {
    let gross = sum(lines_of(lot, entry_id).map(|s| s.seller_amount))?;
    let comm = sum(lines_of(lot, entry_id).map(|s| s.seller_comm.amount))?;
    let freight = if lot.freight_paid_by == FreightPaidBy::Firm {
        lot.freight
    } else {
        Paisa::ZERO
    };
    let charges = sum([
        freight,
        lot.labour.amount,
        lot.munshiana.amount,
        lot.store_rent,
        lot.phone.amount,
    ]
    .into_iter())?;
    gross
        .checked_sub(comm)
        .and_then(|v| v.checked_sub(charges))
        .map_err(money_error)
}

fn lot_input(lot: &SavedLot, entry_id: i64, keep_ids: bool) -> Result<LotInput, ServiceError> {
    // A one-time seller paid his whole net on the spot (PRD FR-13.10) stays so after an edit: the
    // field comes back empty and the default applies to the new net. A partial payment is kept.
    let pays_on_the_spot = lot.pukhta.is_some()
        && lot.cash_paid > Paisa::ZERO
        && lot.cash_paid == credit_of(lot, entry_id)?;
    Ok(LotInput {
        key: format!("lot:{}", lot.id),
        lot_id: keep_ids.then_some(lot.id),
        seller_account_id: Some(lot.seller.id),
        purchase_id: lot.purchase.as_ref().map(|p| p.id),
        pukhta: lot.pukhta.as_ref().map(pukhta_input),
        item_id: Some(lot.item.id),
        packing_id: Some(lot.packing.id),
        marka_id: lot.marka_id,
        store_id: lot.store_id,
        qty_received: lot.qty_received.to_string(),
        weight_received: lot.weight_received_grams.map(kilos),
        freight: (lot.freight > Paisa::ZERO).then(|| rupees(lot.freight)),
        freight_paid_by: lot.freight_paid_by,
        labour: typed_lot_charge(&lot.labour),
        munshiana: typed_lot_charge(&lot.munshiana),
        phone: typed_lot_charge(&lot.phone),
        cash_paid: (!pays_on_the_spot).then(|| rupees(lot.cash_paid)),
        store_rent: (lot.store_rent > Paisa::ZERO).then(|| rupees(lot.store_rent)),
        adjust_advance: lot.adjust_advance,
        sales: lines_of(lot, entry_id)
            .map(|s| sale_input(s, keep_ids))
            .collect(),
    })
}

/// The header of the draft of a posting.
fn header_input(detail: &BikriDetail, date: BusinessDate, first: bool) -> HeaderInput {
    HeaderInput {
        entry_date: format_ddmmyyyy(date),
        bill_date: if first {
            BusinessDate::parse_iso(&detail.bill_date)
                .map(format_ddmmyyyy)
                .unwrap_or_default()
        } else {
            String::new()
        },
        book: detail.book.clone(),
        truck_no: detail.truck_no.clone(),
        zamindar_account_id: detail.zamindar.as_ref().map(|z| z.id),
        adda_id: detail.adda_id,
        driver_name: detail.driver_name.clone(),
        driver_phone: detail.driver_phone.clone(),
        total_freight: detail.total_freight.map(rupees),
        store_id: detail.store_id,
        note: detail.note.clone(),
    }
}

/// The draft of one posting of a saved بکری, built from its stored values (PRD FR-12.18). With
/// `keep_ids` the lots and lines carry their ids (an edit); without them it is a new unsaved
/// بکری with the same values (the undo, PRD FR-7.47).
///
/// # Errors
/// `ServiceError::Data` for a sum that does not fit.
pub(super) fn draft_of(
    detail: &BikriDetail,
    entry_id: i64,
    date: BusinessDate,
    kind: PostingKind,
    keep_ids: bool,
) -> Result<BikriDraft, ServiceError> {
    let lots = posting_lots(detail, entry_id, kind)
        .iter()
        .map(|lot| lot_input(lot, entry_id, keep_ids))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(BikriDraft {
        header: header_input(detail, date, kind == PostingKind::First),
        lots,
    })
}

/// The lot of a later sale, or of an edit of a later posting, as a lot of a draft (PRD FR-12.17):
/// its seller, item and packing from the database, the quantity received, no lot charge, no
/// freight, no store rent, and نقد 0 (a later sale gives no نقد; a one-time seller is paid later
/// by an ادائیگی, PRD FR-13.7). Only the lines are the user's.
pub(super) fn later_lot_input(state: &LotState, sales: Vec<SaleInput>) -> LotInput {
    LotInput {
        key: format!("lot:{}", state.id),
        lot_id: Some(state.id),
        seller_account_id: Some(state.seller_account_id),
        // A lot of own goods never has a later sale (PRD FR-12.26).
        purchase_id: None,
        pukhta: state
            .pukhta
            .as_ref()
            .map(|(name_ur, name_en, phone, _)| PukhtaInput {
                name_ur: name_ur.clone(),
                name_en: name_en.clone(),
                phone: phone.clone(),
            }),
        item_id: Some(state.item_id),
        packing_id: Some(state.packing_id),
        marka_id: state.marka_id,
        store_id: state.store_id,
        qty_received: state.qty_received.to_string(),
        weight_received: state.weight_received_grams.map(kilos),
        freight: None,
        freight_paid_by: if state.freight_by_firm {
            FreightPaidBy::Firm
        } else {
            FreightPaidBy::Seller
        },
        labour: None,
        munshiana: None,
        phone: None,
        cash_paid: Some("0".to_owned()),
        store_rent: None,
        adjust_advance: state.adjust_advance,
        sales,
    }
}

/// The header of a later sale (or of an edit of a later posting): only its day matters; the rest
/// belongs to the بکری and is not checked again.
pub(super) fn later_header(date: BusinessDate) -> HeaderInput {
    HeaderInput {
        entry_date: format_ddmmyyyy(date),
        bill_date: String::new(),
        book: None,
        truck_no: None,
        zamindar_account_id: None,
        adda_id: None,
        driver_name: None,
        driver_phone: None,
        total_freight: None,
        store_id: None,
        note: None,
    }
}
