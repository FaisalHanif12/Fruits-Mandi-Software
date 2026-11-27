//! `gardens_sale_money` (PRD FR-12.53 v1.60, plan 99): where the money of a garden's sales is. Read
//! only.
//!
//! - One row for each active sale line of the garden's lots in our shop: its stored value, the
//!   buyer's bill, what he paid on the spot, what has been matched to its bill since by other
//!   entries (a وصولی, an older credit, a جنرل credit), what is still on his account, and the
//!   garden's credit, the value less the seller commission.
//! - One row for a lot's other charges (freight paid by آڑھتی, مزدوری, منشیانہ, فون, کرایہ سٹور),
//!   taken off the garden's credit, so a lot's rows add up to its seller credit (PRD C8) and no
//!   charge is split over buyers.
//! - One row for each saved sale in another shop: the money that came into روکڑ.
//!
//! The garden's credit over the whole garden is its money in from our shop and other shops, as R36
//! shows it. Every figure is a stored one; the sums use checked maths. A bill line is found by
//! walking the lines of the posting in the order the sales were posted (account, چکتا person and
//! the bill amount).
//!
//! Example: 20 پیٹی آم to مانی at 1,000: value 20,000, bill 21,360, paid 0, still 21,360, garden
//! credit 18,800.

use std::collections::{HashMap, HashSet};

use mandi_db::gardens as garden_repo;
use mandi_db::sale_money as repo;
use mandi_domain::core::accounts::RecordRef;
use mandi_domain::core::dates::BusinessDate;
use mandi_domain::core::error::ErrorCode;
use mandi_domain::core::garden_sale_money::{
    GardensSaleMoney, GardensSaleMoneyInput, SaleMoneyCharges, SaleMoneyKind, SaleMoneyLot,
    SaleMoneyRow, SaleMoneyTotals,
};
use mandi_domain::core::money::Paisa;
use mandi_domain::core::rules::{Working, WorkingArg, WorkingValue};
use mandi_domain::core::text;
use mandi_domain::core::validate::{self, FieldErrors};

use crate::context::Context;
use crate::day;
use crate::error::{ServiceError, money_error};

fn data(message: &str) -> ServiceError {
    ServiceError::Data(message.to_owned())
}

fn paisa(value: i64) -> Result<Paisa, ServiceError> {
    Paisa::try_from_paisa(value).map_err(money_error)
}

fn record(named: &repo::Named) -> RecordRef {
    RecordRef {
        id: named.id,
        code: named.code.clone(),
        name_ur: named.name_ur.clone(),
        name_en: named.name_en.clone(),
        tag: named.tag.clone(),
    }
}

fn money_arg(name: &str, amount: Paisa) -> WorkingArg {
    WorkingArg {
        name: name.to_owned(),
        value: WorkingValue::Money { paisa: amount },
    }
}

fn working(message_id: &str, args: Vec<WorkingArg>) -> Working {
    Working {
        message_id: message_id.to_owned(),
        args,
        rule_id: None,
    }
}

fn lot_view(lot: &repo::LotRef) -> SaleMoneyLot {
    SaleMoneyLot {
        lot_id: lot.lot_id,
        lot_no: lot.lot_no,
        lot_counter: lot.lot_counter,
        item: record(&lot.item),
        packing: record(&lot.packing),
    }
}

/// A row with the words a search looks in and its place in the order of the list: the day, our shop
/// before other shops, the بکری, the lot, the lines before the lot's charges, the line.
struct Built {
    row: SaleMoneyRow,
    order: (String, u8, i64, i64, u8, i64, i64),
    words: Vec<String>,
}

impl Built {
    fn matches(&self, urdu: &str, english: &str) -> bool {
        self.words.iter().any(|w| {
            (!urdu.is_empty() && text::search_ur(w).contains(urdu))
                || (!english.is_empty() && text::search_en(w).contains(english))
        })
    }
}

fn words_of(parts: &[&repo::Named]) -> Vec<String> {
    parts
        .iter()
        .flat_map(|n| [n.code.clone(), n.name_ur.clone(), n.name_en.clone()])
        .collect()
}

/// The bill line of each sale line: the debit lines of its posting are walked in the order the sale
/// lines were posted, each matched on the account, the چکتا person and the bill amount.
fn bill_lines(
    sales: &HashMap<i64, Vec<repo::PostedSale>>,
    lines: &HashMap<i64, Vec<repo::DebitLine>>,
) -> Result<HashMap<i64, i64>, ServiceError> {
    let mut found = HashMap::new();
    for (entry, posted) in sales {
        let debits = lines
            .get(entry)
            .ok_or_else(|| data("a posting of a sale has no debit line"))?;
        let mut used: HashSet<i64> = HashSet::new();
        let mut from = 0_usize;
        for sale in posted {
            let fits = |l: &repo::DebitLine, used: &HashSet<i64>| {
                !used.contains(&l.line_id)
                    && l.account_id == sale.buyer_account_id
                    && l.person_key == sale.person_key
                    && l.dr_paisa == sale.buyer_total_paisa
            };
            // The next one after the last found; if the lots were posted in another order, the
            // first unused one.
            let at = debits
                .iter()
                .enumerate()
                .skip(from)
                .find(|(_, l)| fits(l, &used))
                .or_else(|| debits.iter().enumerate().find(|(_, l)| fits(l, &used)));
            let (index, line) = at.ok_or_else(|| data("the bill line of a sale was not found"))?;
            used.insert(line.line_id);
            from = index.saturating_add(1);
            found.insert(sale.sale_id, line.line_id);
        }
    }
    Ok(found)
}

fn sale_row(line: &repo::Line, collected: i64) -> Result<Built, ServiceError> {
    let value = paisa(line.value_paisa)?;
    let commission = paisa(line.seller_comm_paisa)?;
    let credit = value.checked_sub(commission).map_err(money_error)?;
    let bill = paisa(line.buyer_total_paisa)?;
    let paid = paisa(line.paid_paisa)?;
    let collected = paisa(collected)?;
    let still = bill
        .checked_sub(paid)
        .and_then(|v| v.checked_sub(collected))
        .map_err(money_error)?;
    if still.is_negative() {
        return Err(data("more is matched on a bill than the bill"));
    }
    let mut words = words_of(&[&line.buyer, &line.lot.item]);
    words.push(line.lot.bikri_number.to_string());
    Ok(Built {
        order: (
            line.date.clone(),
            0,
            line.lot.bikri_number,
            line.lot.lot_no,
            0,
            line.line_no,
            line.sale_id,
        ),
        words,
        row: SaleMoneyRow {
            kind: SaleMoneyKind::Bikri,
            date: line.date.clone(),
            bikri_id: Some(line.lot.bikri_id),
            bikri_number: Some(line.lot.bikri_number),
            lot: Some(lot_view(&line.lot)),
            item: None,
            buyer: Some(record(&line.buyer)),
            shop_name: None,
            entry_id: None,
            qty: Some(line.qty),
            weight_grams: line.weight_grams,
            rate_paisa: Some(paisa(line.rate_paisa)?),
            value_paisa: Some(value),
            buyer_total_paisa: Some(bill),
            paid_paisa: Some(paid),
            collected_paisa: Some(collected),
            still_due_paisa: Some(still),
            garden_credit_paisa: credit,
            charges: None,
            working: working(
                "working-sale-money-credit",
                vec![
                    money_arg("value", value),
                    money_arg("commission", commission),
                    money_arg("result", credit),
                ],
            ),
        },
    })
}

fn charges_row(lot: &repo::LotCharges) -> Result<Built, ServiceError> {
    let charges = SaleMoneyCharges {
        freight_paisa: paisa(lot.freight_paisa)?,
        labour_paisa: paisa(lot.labour_paisa)?,
        munshiana_paisa: paisa(lot.munshiana_paisa)?,
        phone_paisa: paisa(lot.phone_paisa)?,
        store_rent_paisa: paisa(lot.store_rent_paisa)?,
    };
    let mut total = Paisa::ZERO;
    for part in [
        charges.freight_paisa,
        charges.labour_paisa,
        charges.munshiana_paisa,
        charges.phone_paisa,
        charges.store_rent_paisa,
    ] {
        total = total.checked_add(part).map_err(money_error)?;
    }
    let mut words = words_of(&[&lot.lot.item]);
    words.push(lot.lot.bikri_number.to_string());
    Ok(Built {
        order: (
            lot.lot.bikri_date.clone(),
            0,
            lot.lot.bikri_number,
            lot.lot.lot_no,
            1,
            0,
            lot.lot.lot_id,
        ),
        words,
        row: SaleMoneyRow {
            kind: SaleMoneyKind::LotCharges,
            date: lot.lot.bikri_date.clone(),
            bikri_id: Some(lot.lot.bikri_id),
            bikri_number: Some(lot.lot.bikri_number),
            lot: Some(lot_view(&lot.lot)),
            item: None,
            buyer: None,
            shop_name: None,
            entry_id: None,
            qty: None,
            weight_grams: None,
            rate_paisa: None,
            value_paisa: None,
            buyer_total_paisa: None,
            paid_paisa: None,
            collected_paisa: None,
            still_due_paisa: None,
            garden_credit_paisa: total.checked_neg().map_err(money_error)?,
            charges: Some(charges),
            working: working(
                "working-sale-money-lot-charges",
                vec![
                    money_arg("freight", charges.freight_paisa),
                    money_arg("labour", charges.labour_paisa),
                    money_arg("munshiana", charges.munshiana_paisa),
                    money_arg("phone", charges.phone_paisa),
                    money_arg("store_rent", charges.store_rent_paisa),
                    money_arg("result", total),
                ],
            ),
        },
    })
}

fn other_row(sale: &repo::OtherSale) -> Result<Built, ServiceError> {
    let amount = paisa(sale.amount_paisa)?;
    let mut words = sale
        .item
        .as_ref()
        .map(|i| words_of(&[i]))
        .unwrap_or_default();
    words.extend(sale.shop_name.clone());
    Ok(Built {
        order: (sale.date.clone(), 1, 0, 0, 0, 0, sale.entry_id),
        words,
        row: SaleMoneyRow {
            kind: SaleMoneyKind::OtherShop,
            date: sale.date.clone(),
            bikri_id: None,
            bikri_number: None,
            lot: None,
            item: sale.item.as_ref().map(record),
            buyer: None,
            shop_name: Some(sale.shop_name.clone().unwrap_or_default()),
            entry_id: Some(sale.entry_id),
            qty: Some(sale.qty),
            weight_grams: None,
            rate_paisa: None,
            value_paisa: Some(amount),
            buyer_total_paisa: None,
            paid_paisa: None,
            collected_paisa: None,
            still_due_paisa: None,
            garden_credit_paisa: amount,
            charges: None,
            working: working(
                "working-sale-money-other-shop",
                vec![money_arg("result", amount)],
            ),
        },
    })
}

fn add(total: &mut Paisa, part: Option<Paisa>) -> Result<(), ServiceError> {
    if let Some(part) = part {
        *total = total.checked_add(part).map_err(money_error)?;
    }
    Ok(())
}

fn totals_of(rows: &[Built]) -> Result<SaleMoneyTotals, ServiceError> {
    let mut totals = SaleMoneyTotals {
        qty: 0,
        value_paisa: Paisa::ZERO,
        buyer_total_paisa: Paisa::ZERO,
        paid_paisa: Paisa::ZERO,
        collected_paisa: Paisa::ZERO,
        still_due_paisa: Paisa::ZERO,
        lot_charges_paisa: Paisa::ZERO,
        garden_credit_paisa: Paisa::ZERO,
    };
    for built in rows {
        let row = &built.row;
        totals.qty = totals
            .qty
            .checked_add(row.qty.unwrap_or(0))
            .ok_or_else(|| data("the quantities do not fit"))?;
        add(&mut totals.value_paisa, row.value_paisa)?;
        add(&mut totals.buyer_total_paisa, row.buyer_total_paisa)?;
        add(&mut totals.paid_paisa, row.paid_paisa)?;
        add(&mut totals.collected_paisa, row.collected_paisa)?;
        add(&mut totals.still_due_paisa, row.still_due_paisa)?;
        if row.kind == SaleMoneyKind::LotCharges {
            add(
                &mut totals.lot_charges_paisa,
                Some(row.garden_credit_paisa.checked_neg().map_err(money_error)?),
            )?;
        }
        add(
            &mut totals.garden_credit_paisa,
            Some(row.garden_credit_paisa),
        )?;
    }
    Ok(totals)
}

/// One page of a garden's sale money with the totals over the whole filter (PRD FR-12.53,
/// FR-19.11). Read only.
///
/// # Errors
/// `ERR_NOT_FOUND` on `garden_id`; the date codes on `from` and `to`; `ServiceError::Db` or
/// `ServiceError::Data`.
pub fn sale_money(
    ctx: &Context,
    input: &GardensSaleMoneyInput,
) -> Result<GardensSaleMoney, ServiceError> {
    let (_, today) = day::window(ctx)?;
    let mut errors = FieldErrors::default();
    let mut day_of = |field: &str, text: Option<&String>| {
        let text = text.map(|t| t.trim()).filter(|t| !t.is_empty())?;
        errors
            .take(validate::date(field, text, today))
            .map(BusinessDate::to_iso)
    };
    let from = day_of("from", input.from.as_ref());
    let to = day_of("to", input.to.as_ref());
    errors.finish()?;
    ctx.db.read_snapshot(|conn| {
        let garden = garden_repo::get(conn, input.garden_id)?
            .ok_or_else(|| ServiceError::reject_field(ErrorCode::ErrNotFound, "garden_id"))?;
        let lines = repo::lines(conn, garden.account.id, input.arrival_id)?;
        let mut entries: Vec<i64> = lines.iter().map(|l| l.entry_id).collect();
        entries.sort_unstable();
        entries.dedup();
        let bills = bill_lines(
            &repo::posted_sales(conn, &entries)?,
            &repo::debit_lines(conn, &entries)?,
        )?;
        let wanted: Vec<i64> = lines
            .iter()
            .map(|l| {
                bills
                    .get(&l.sale_id)
                    .copied()
                    .ok_or_else(|| data("a sale line has no bill line"))
            })
            .collect::<Result<_, _>>()?;
        let collected = repo::collected_since(conn, &wanted)?;
        let mut rows = Vec::with_capacity(lines.len());
        for (line, bill) in lines.iter().zip(&wanted) {
            rows.push(sale_row(line, collected.get(bill).copied().unwrap_or(0))?);
        }
        for lot in repo::lot_charges(conn, garden.account.id, input.arrival_id)? {
            rows.push(charges_row(&lot)?);
        }
        for sale in repo::other_sales(conn, garden.id, input.arrival_id)? {
            rows.push(other_row(&sale)?);
        }
        rows.retain(|b| {
            from.as_ref().is_none_or(|f| b.row.date >= *f)
                && to.as_ref().is_none_or(|t| b.row.date <= *t)
        });
        if let Some(typed) = input
            .search
            .as_deref()
            .map(str::trim)
            .filter(|t| !t.is_empty())
        {
            let (urdu, english) = (text::search_ur(typed), text::search_en(typed));
            rows.retain(|b| b.matches(&urdu, &english));
        }
        rows.sort_by(|a, b| a.order.cmp(&b.order));
        let totals = totals_of(&rows)?;
        let total_count = i64::try_from(rows.len()).unwrap_or(i64::MAX);
        let limit = usize::try_from(crate::page::limit_of(input.limit)).unwrap_or(usize::MAX);
        let offset = usize::try_from(input.offset).unwrap_or(usize::MAX);
        Ok(GardensSaleMoney {
            rows: rows
                .into_iter()
                .skip(offset)
                .take(limit)
                .map(|b| b.row)
                .collect(),
            total_count,
            totals,
        })
    })
}
