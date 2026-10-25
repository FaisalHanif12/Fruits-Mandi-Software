//! The pages of the three documents (plan 14, PRD FR-12.12, R1, R5, FR-13.1 to FR-13.3): the
//! seller bill, the buyer slips and the slip of a one-time buyer. Every number is a stored value
//! or a checked sum of stored values.

use mandi_core::bikri::{BikriDetail, PaymentType, SavedLot, SavedPukhta, SavedSale, SavedTotals};
use mandi_core::money::Paisa;

use super::build::Ctx;
use super::ids::*;
use super::{AmountRole, PrintLine, PrintPage, PrintParty};
use crate::error::ReportsError;

/// A sale line with the lot it belongs to (the lot gives the item and the packing).
type SaleRef<'a> = (&'a SavedLot, &'a SavedSale);

/// The buyer's name on a line: the one-time person's name if he has one, else the account.
fn buyer_name(ctx: &Ctx, sale: &SavedSale) -> String {
    match &sale.pukhta {
        Some(person) => ctx.name(&person.name_ur, &person.name_en),
        None => ctx.name(&sale.buyer.name_ur, &sale.buyer.name_en),
    }
}

/// A phone as typed, or `None` when empty.
fn phone_of(person: Option<&SavedPukhta>) -> Option<String> {
    person
        .map(|p| p.phone.trim())
        .filter(|p| !p.is_empty())
        .map(str::to_owned)
}

/// A line written the client's way for R5 and the buyer slips: `(item qty . rate)`, for example
/// `(کیلا 20 . 600)` (PRD R5, plan 17e). Text only: the quantity and the rate are the formatted
/// stored values, so nothing is worked out here.
fn line_detail(item: &str, qty: &str, rate: &str) -> String {
    format!("({item} {qty} . {rate})")
}

/// One goods line. `seller_side` picks the seller rate and amount (seller bill); otherwise the
/// buyer rate and value (slips).
fn line(
    ctx: &Ctx,
    lot: &SavedLot,
    sale: &SavedSale,
    seller_side: bool,
) -> Result<PrintLine, ReportsError> {
    let (rate, amount) = if seller_side {
        (sale.seller_rate, sale.seller_amount)
    } else {
        (sale.buyer_rate, sale.buyer_value)
    };
    let item = ctx.name(&lot.item.name_ur, &lot.item.name_en);
    let qty = ctx.style.count(sale.qty);
    let rate_text = ctx.money(rate)?;
    Ok(PrintLine {
        date: ctx.style.date(&sale.entry_date)?,
        name: seller_side.then(|| buyer_name(ctx, sale)),
        // A one-time buyer is a person, not an account: he has no tag (PRD 10.2).
        name_tag: if seller_side && sale.pukhta.is_none() {
            sale.buyer.tag.clone()
        } else {
            None
        },
        detail: line_detail(&item, &qty, &rate_text),
        item,
        packing: ctx.name(&lot.packing.name_ur, &lot.packing.name_en),
        qty,
        weight_kg: sale.weight_grams.map(|grams| ctx.style.kilos(grams)),
        rate: rate_text,
        amount: ctx.money(amount)?,
        note: sale
            .note
            .as_deref()
            .map(str::trim)
            .filter(|n| !n.is_empty())
            .map(str::to_owned),
    })
}

/// The seller bill, R1: one page per seller. A seller with two lots (two items on one truck,
/// PRD E2) gets one page with the lines and the sums of both lots.
pub(super) fn seller_pages(
    ctx: &Ctx,
    detail: &BikriDetail,
) -> Result<Vec<PrintPage>, ReportsError> {
    // A seller is the account, and for the پختہ کھاتہ the one-time person too (PRD FR-13.5).
    let key = |lot: &SavedLot| {
        (
            lot.seller.id,
            lot.pukhta.as_ref().map(|p| p.person_key.clone()),
        )
    };
    let mut groups: Vec<(_, Vec<&SavedLot>)> = Vec::new();
    for lot in &detail.lots {
        let lot_key = key(lot);
        match groups.iter_mut().find(|(k, _)| *k == lot_key) {
            Some((_, lots)) => lots.push(lot),
            None => groups.push((lot_key, vec![lot])),
        }
    }
    groups
        .into_iter()
        .enumerate()
        .map(|(index, (_, lots))| seller_page(ctx, serial(ctx, index), &lots))
        .collect()
}

/// The row number of a page in its document: 1, 2, 3 ... formatted for print.
fn serial(ctx: &Ctx, index: usize) -> String {
    ctx.style
        .count(i64::try_from(index.saturating_add(1)).unwrap_or(i64::MAX))
}

fn seller_page(ctx: &Ctx, serial: String, lots: &[&SavedLot]) -> Result<PrintPage, ReportsError> {
    let first = lots
        .first()
        .ok_or_else(|| ReportsError::Data("a seller without a lot".to_owned()))?;
    let mut lines = Vec::new();
    let mut totals = SavedTotals::default();
    for lot in lots {
        // Each lot on its own, so two errors in different lots cannot cancel out.
        check_seller_totals(&lot.totals)?;
        totals = totals.plus(&lot.totals)?;
        for sale in lot.sales.iter().filter(|s| s.is_active) {
            lines.push(line(ctx, lot, sale, true)?);
        }
    }
    let party = match &first.pukhta {
        Some(person) => PrintParty {
            role_id: PARTY_SELLER.to_owned(),
            code: None,
            name: ctx.name(&person.name_ur, &person.name_en),
            tag: None,
            phone: phone_of(Some(person)),
        },
        None => PrintParty {
            role_id: PARTY_SELLER.to_owned(),
            code: Some(first.seller.code.clone()),
            name: ctx.name(&first.seller.name_ur, &first.seller.name_en),
            tag: first.seller.tag.clone(),
            phone: None,
        },
    };
    // PRD 4.4: gross, the seven deductions, their total, the net sale. All seven are always
    // printed, also when 0 (store rent 0 is on the worked bill).
    let line_role = AmountRole::Line;
    let mut amounts = vec![
        ctx.amount(GROSS_SALE, totals.gross, AmountRole::Subtotal)?,
        ctx.amount(FREIGHT, totals.freight, line_role)?,
        ctx.amount(COMMISSION, totals.seller_comm, line_role)?,
        ctx.amount(LABOUR, totals.labour, line_role)?,
        ctx.amount(MUNSHIANA, totals.munshiana, line_role)?,
        ctx.amount(CASH_PAID, totals.cash_paid, line_role)?,
        ctx.amount(STORE_RENT, totals.store_rent, line_role)?,
        ctx.amount(PHONE_CHARGE, totals.phone, line_role)?,
        ctx.amount(TOTAL_DEDUCTIONS, totals.deductions, AmountRole::Subtotal)?,
    ];
    // A net below 0 means the seller owes the firm (PRD FR-12.7, E28). It is printed as a
    // positive amount under its own label, never with a minus sign, and its words follow.
    let (net_label, net_shown) = if totals.net.is_negative() {
        (SELLER_OWES, totals.net.checked_neg()?)
    } else {
        (NET_SALE, totals.net)
    };
    amounts.push(ctx.amount(net_label, net_shown, AmountRole::Total)?);
    Ok(PrintPage {
        serial,
        party,
        lines,
        amounts,
        payment_id: None,
        words: Some(ctx.words(net_shown)?),
    })
}

/// A check, not a calculation (Code Standards B20): the stored lot totals must agree with each
/// other, so a bill never prints numbers that do not add up. The seven deductions make the total
/// deductions (C6) and gross less deductions is the net (C7). If stored data ever disagrees, the
/// print stops with an error instead of showing a wrong bill.
fn check_seller_totals(t: &SavedTotals) -> Result<(), ReportsError> {
    let deductions = [
        t.freight,
        t.seller_comm,
        t.labour,
        t.munshiana,
        t.cash_paid,
        t.store_rent,
        t.phone,
    ]
    .into_iter()
    .try_fold(Paisa::ZERO, Paisa::checked_add)?;
    if deductions != t.deductions || t.gross.checked_sub(t.deductions)? != t.net {
        return Err(ReportsError::Data(
            "the stored totals of a seller do not add up".to_owned(),
        ));
    }
    Ok(())
}

/// The buyer slips, R5: one page per buyer account, in the order the buyers first appear.
/// One-time buyers have their own slip (`pukhta_pages`).
pub(super) fn buyer_pages(
    ctx: &Ctx,
    detail: &BikriDetail,
    only_buyer_id: Option<i64>,
) -> Result<Vec<PrintPage>, ReportsError> {
    let mut groups: Vec<(i64, Vec<SaleRef<'_>>)> = Vec::new();
    for lot in &detail.lots {
        for sale in lot
            .sales
            .iter()
            .filter(|s| s.is_active && s.pukhta.is_none())
        {
            if only_buyer_id.is_some_and(|id| id != sale.buyer.id) {
                continue;
            }
            match groups.iter_mut().find(|(id, _)| *id == sale.buyer.id) {
                Some((_, sales)) => sales.push((lot, sale)),
                None => groups.push((sale.buyer.id, vec![(lot, sale)])),
            }
        }
    }
    groups
        .into_iter()
        .enumerate()
        .map(|(index, (_, sales))| {
            let buyer = sales
                .first()
                .map(|(_, sale)| &sale.buyer)
                .ok_or_else(|| ReportsError::Data("a buyer without a line".to_owned()))?;
            let party = PrintParty {
                role_id: PARTY_BUYER.to_owned(),
                code: Some(buyer.code.clone()),
                name: ctx.name(&buyer.name_ur, &buyer.name_en),
                tag: buyer.tag.clone(),
                phone: None,
            };
            slip_page(ctx, serial(ctx, index), party, &sales)
        })
        .collect()
}

/// The slip of each one-time buyer of the پختہ کھاتہ: one page per person, by his person key
/// (PRD FR-13.5, so شاہد's lines never join ناصر's).
pub(super) fn pukhta_pages(
    ctx: &Ctx,
    detail: &BikriDetail,
) -> Result<Vec<PrintPage>, ReportsError> {
    let mut groups: Vec<(&str, &SavedPukhta, Vec<SaleRef<'_>>)> = Vec::new();
    for lot in &detail.lots {
        for sale in lot.sales.iter().filter(|s| s.is_active) {
            let Some(person) = sale.pukhta.as_ref() else {
                continue;
            };
            let key = person.person_key.as_str();
            match groups.iter_mut().find(|(k, _, _)| *k == key) {
                Some((_, _, sales)) => sales.push((lot, sale)),
                None => groups.push((key, person, vec![(lot, sale)])),
            }
        }
    }
    groups
        .into_iter()
        .enumerate()
        .map(|(index, (_, person, sales))| {
            let party = PrintParty {
                role_id: PARTY_ONETIME_BUYER.to_owned(),
                code: None,
                name: ctx.name(&person.name_ur, &person.name_en),
                tag: None,
                phone: phone_of(Some(person)),
            };
            slip_page(ctx, serial(ctx, index), party, &sales)
        })
        .collect()
}

/// One slip page: the lines of one buyer, the value of the goods, his commission, لاگا and
/// what he pays (PRD C5), all stored amounts and their sums.
fn slip_page(
    ctx: &Ctx,
    serial: String,
    party: PrintParty,
    sales: &[SaleRef<'_>],
) -> Result<PrintPage, ReportsError> {
    let (mut value, mut comm, mut laga, mut total) =
        (Paisa::ZERO, Paisa::ZERO, Paisa::ZERO, Paisa::ZERO);
    let mut lines = Vec::with_capacity(sales.len());
    let (mut any_cash, mut any_credit) = (false, false);
    for (lot, sale) in sales {
        // C5, as a check: what the buyer pays is his value, commission and لاگا.
        if sale
            .buyer_value
            .checked_add(sale.buyer_comm.amount)?
            .checked_add(sale.laga.amount)?
            != sale.buyer_total
        {
            return Err(ReportsError::Data(
                "the stored total of a sale line does not add up".to_owned(),
            ));
        }
        value = value.checked_add(sale.buyer_value)?;
        comm = comm.checked_add(sale.buyer_comm.amount)?;
        laga = laga.checked_add(sale.laga.amount)?;
        total = total.checked_add(sale.buyer_total)?;
        match sale.payment_type {
            PaymentType::Cash => any_cash = true,
            PaymentType::Credit => any_credit = true,
        }
        lines.push(line(ctx, lot, sale, false)?);
    }
    // One mark only when every line is paid the same way; a mixed slip has none.
    let payment_id = match (any_cash, any_credit) {
        (true, false) => Some(PAYMENT_CASH.to_owned()),
        (false, true) => Some(PAYMENT_CREDIT.to_owned()),
        _ => None,
    };
    let amounts = vec![
        ctx.amount(ITEMS_VALUE, value, AmountRole::Subtotal)?,
        ctx.amount(BUYER_COMMISSION, comm, AmountRole::Line)?,
        ctx.amount(LAGA, laga, AmountRole::Line)?,
        ctx.amount(BUYER_PAYS, total, AmountRole::Total)?,
    ];
    Ok(PrintPage {
        serial,
        party,
        lines,
        amounts,
        payment_id,
        words: Some(ctx.words(total)?),
    })
}
