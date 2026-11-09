//! The pages of the two documents of a بکری (plan 14, PRD FR-12.12, R1, R5, FR-7.57, FR-13.5): the
//! seller bill and the buyer slips, with one slip for each چکتا buyer (plan 60 C9a). Every number is
//! a stored value or a checked sum of stored values.

use mandi_core::accounts::short_code_of;
use mandi_core::bikri::{BikriDetail, SavedChukta, SavedLot, SavedSale, SavedTotals};
use mandi_core::money::Paisa;
use mandi_core::text;

use super::build::Ctx;
use super::ids::*;
use super::{AmountRole, PrintLine, PrintPage, PrintParty};
use crate::error::ReportsError;

/// A sale line with the lot it belongs to (the lot gives the item and the packing).
type SaleRef<'a> = (&'a SavedLot, &'a SavedSale);

/// The buyer's name on a line: the چکتا buyer's name if he has one, else the account.
fn buyer_name(ctx: &Ctx, sale: &SavedSale) -> String {
    match &sale.chukta {
        Some(person) => ctx.name(&person.name_ur, &person.name_en),
        None => ctx.name(&sale.buyer.name_ur, &sale.buyer.name_en),
    }
}

/// A چکتا buyer's phone as the print shows it (`03XX-XXXXXXX`), or `None` when empty.
fn phone_of(person: Option<&SavedChukta>) -> Option<String> {
    person
        .map(|p| p.phone.trim())
        .filter(|p| !p.is_empty())
        .map(text::phone_display)
}

/// A line written the client's way for R5 and the buyer slips: `(item qty . rate)`, for example
/// `(کیلا 20 . 600)` (PRD R5, plan 17e). With a quality its word follows the item: `(سیب اول 10 .
/// 1200)` (PRD FR-12.42). Text only: the quantity and the rate are the formatted stored values,
/// so nothing is worked out here.
fn line_detail(item: &str, grade_word: Option<&str>, qty: &str, rate: &str) -> String {
    match grade_word {
        Some(word) => format!("({item} {word} {qty} . {rate})"),
        None => format!("({item} {qty} . {rate})"),
    }
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
        // A چکتا buyer is a person, not an account: he has no tag (PRD 10.2).
        name_tag: if seller_side && sale.chukta.is_none() {
            sale.buyer.tag.clone()
        } else {
            None
        },
        detail: line_detail(
            &item,
            sale.grade.and_then(|grade| ctx.grade_word(grade)),
            &qty,
            &rate_text,
        ),
        item,
        packing: ctx.name(&lot.packing.name_ur, &lot.packing.name_en),
        grade: sale.grade,
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
    // A seller is the account.
    let mut groups: Vec<(i64, Vec<&SavedLot>)> = Vec::new();
    for lot in &detail.lots {
        match groups.iter_mut().find(|(k, _)| *k == lot.seller.id) {
            Some((_, lots)) => lots.push(lot),
            None => groups.push((lot.seller.id, vec![lot])),
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
    let party = PrintParty {
        role_id: PARTY_SELLER.to_owned(),
        code: Some(first.seller.code.clone()),
        short_code: short_code_of(&first.seller.code),
        name: ctx.name(&first.seller.name_ur, &first.seller.name_en),
        tag: first.seller.tag.clone(),
        phone: None,
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

/// The buyer slips, R5: one page per buyer, in the order the buyers first appear. A چکتا buyer has a
/// slip of his own, told by his person key (PRD FR-13.5), so شاہد's lines never join ناصر's.
pub(super) fn buyer_pages(
    ctx: &Ctx,
    detail: &BikriDetail,
    only_buyer_id: Option<i64>,
) -> Result<Vec<PrintPage>, ReportsError> {
    type Group<'a> = ((i64, Option<&'a str>), Vec<SaleRef<'a>>);
    let mut groups: Vec<Group<'_>> = Vec::new();
    for lot in &detail.lots {
        for sale in lot.sales.iter().filter(|s| s.is_active) {
            if only_buyer_id.is_some_and(|id| id != sale.buyer.id) {
                continue;
            }
            let key = (
                sale.buyer.id,
                sale.chukta.as_ref().map(|p| p.person_key.as_str()),
            );
            match groups.iter_mut().find(|(k, _)| *k == key) {
                Some((_, sales)) => sales.push((lot, sale)),
                None => groups.push((key, vec![(lot, sale)])),
            }
        }
    }
    groups
        .into_iter()
        .enumerate()
        .map(|(index, (_, sales))| {
            let first = sales
                .first()
                .map(|(_, sale)| *sale)
                .ok_or_else(|| ReportsError::Data("a buyer without a line".to_owned()))?;
            let buyer = &first.buyer;
            let party = match &first.chukta {
                // A چکتا buyer is a person, not an account: his name and phone, no code and no tag.
                Some(person) => PrintParty {
                    role_id: PARTY_BUYER.to_owned(),
                    code: None,
                    short_code: None,
                    name: ctx.name(&person.name_ur, &person.name_en),
                    tag: None,
                    phone: phone_of(Some(person)),
                },
                None => PrintParty {
                    role_id: PARTY_BUYER.to_owned(),
                    code: Some(buyer.code.clone()),
                    short_code: short_code_of(&buyer.code),
                    name: ctx.name(&buyer.name_ur, &buyer.name_en),
                    tag: buyer.tag.clone(),
                    phone: None,
                },
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
    // What the buyer paid on the spot and what stays on his account, over the lines (PRD FR-12.44).
    let (mut paid, mut left) = (Paisa::ZERO, Paisa::ZERO);
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
        paid = paid.checked_add(sale.paid_paisa)?;
        left = left.checked_add(sale.left_paisa)?;
        lines.push(line(ctx, lot, sale, false)?);
    }
    // One mark only when the whole slip is paid the same way: all of it paid on the spot, or none of
    // it. A slip paid in part has no mark; it shows the two amounts instead (PRD FR-12.44).
    let part_paid = paid > Paisa::ZERO && left > Paisa::ZERO;
    let payment_id = match (paid > Paisa::ZERO, left > Paisa::ZERO) {
        (true, false) => Some(PAYMENT_CASH.to_owned()),
        (false, true) => Some(PAYMENT_CREDIT.to_owned()),
        _ => None,
    };
    let mut amounts = vec![
        ctx.amount(ITEMS_VALUE, value, AmountRole::Subtotal)?,
        ctx.amount(BUYER_COMMISSION, comm, AmountRole::Line)?,
        ctx.amount(LAGA, laga, AmountRole::Line)?,
        ctx.amount(BUYER_PAYS, total, AmountRole::Total)?,
    ];
    if part_paid {
        amounts.push(ctx.amount(PAID_ON_SPOT, paid, AmountRole::Line)?);
        amounts.push(ctx.amount(LEFT_ON_ACCOUNT, left, AmountRole::Line)?);
    }
    Ok(PrintPage {
        serial,
        party,
        lines,
        amounts,
        payment_id,
        words: Some(ctx.words(total)?),
    })
}
