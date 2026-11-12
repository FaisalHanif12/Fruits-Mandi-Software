//! R10 مکمل سٹاک / باقی مال (the goods not yet sold) (PRD 19.2, FR-12.14, FR-12.25, C16).
//!
//! One row is a lot of a saved بکری with goods left, a purchase of a market trip with stock
//! left (marked اپنا مال: its seller is the own-goods account and its document is the purchase),
//! or a garden arrival (مال آیا, PRD FR-12.40) with goods left (its seller is the garden's account
//! and its document is the arrival). A trip purchase or an arrival waits from its day to sell.
//! The row says what was received, sold and written off, the balance and how many days the goods
//! have been waiting. The figures are the ones the lots and the purchases keep up to date with
//! every save, the same as the باقی مال screen reads (so the report and the screen agree). The
//! totals row adds up the four quantities over the whole filter.
//!
//! Two more columns split the balance into the goods for our shop and the goods for another shop
//! (PRD FR-12.41): a lot has only our shop's part; a trip purchase or an arrival has both. The two
//! add up to the balance. A column after them shows the name of the other shop, as typed.
//!
//! The balance of a lot is received less sold, written off and taken as fruit (C16, PRD 16.11);
//! the column "taken as fruit" says how much of it left the lot that way (a trip purchase has
//! none: its goods are not offered as fruit).
//!
//! Example (E2E-16 on 10-10-26): one lot, رحیم کیلا, received 50, sold 45, written off 0, taken 0,
//! balance 5, 0 days. After E2E-42 (الف takes 2 at home): taken 2, balance 3.

use mandi_core::accounts::OWN_GOODS_CODE;
use mandi_core::dates::BusinessDate;
use mandi_core::split::StockParts;
use mandi_db::accounts as account_repo;
use mandi_db::bikri::edit::{StockFilter, stock_lots};
use mandi_db::purchases::{ListFilter, StateFilter};
use mandi_db::rusqlite::Connection;

use super::super::PageSpec;
use crate::error::ReportsError;
use crate::reports::common::{col, count_of, load_parties, lot_names, names_cell};
use crate::reports::filters::Typed;
use crate::reports::ids;
use crate::reports::model::{
    Cell, ColumnKind, ReportId, ReportPage, Row, RowKind, RowSource, SourceKind, StockView,
};

/// A row waiting to be sorted: the day, the kind (a lot before a purchase), the document number
/// and the lot number, then the cells.
struct Pending {
    key: (String, u8, i64, i64),
    cells: Vec<Cell>,
    source: RowSource,
}

/// What one lot, purchase or arrival adds to the row of its item (R10 by item): received, sold,
/// written off, taken, balance, our shop's part of the balance and the other shop's part.
struct ItemPart {
    item_id: i64,
    names: (String, String),
    amounts: [i64; 7],
}

/// The name of the other shop as typed (PRD FR-12.41), or nothing when none was given.
fn shop_cell(name: Option<&str>) -> Cell {
    match name {
        Some(text) if !text.is_empty() => Cell::Text {
            text: text.to_owned(),
        },
        _ => Cell::Empty,
    }
}

/// The ids of the items of a group (PRD v1.29): R10 keeps only the goods of these items. The three
/// sources of R10 (lots, trip purchases, garden arrivals) are each cut to them.
fn items_of_group(
    conn: &Connection,
    group: Option<mandi_core::rules::ItemGroup>,
) -> Result<Option<std::collections::HashSet<i64>>, ReportsError> {
    group
        .map(|group| crate::reports::common::item_ids_of_group(conn, group))
        .transpose()
}

/// R10 by item (PRD v1.32): one row for each item with the quantities of all its lots, trip
/// purchases and garden arrivals, the biggest balance first (then the item's id). The totals are
/// the same as the lot layout's, over the whole filter, so the two layouts always agree.
fn by_item(
    typed: &Typed,
    item_parts: Vec<ItemPart>,
    totals: [i64; 7],
    spec: PageSpec,
) -> Result<ReportPage, ReportsError> {
    let mut items: std::collections::BTreeMap<i64, ((String, String), [i64; 7])> =
        std::collections::BTreeMap::new();
    for part in item_parts {
        let slot = items
            .entry(part.item_id)
            .or_insert_with(|| (part.names.clone(), [0; 7]));
        for (sum, amount) in slot.1.iter_mut().zip(part.amounts) {
            *sum = sum
                .checked_add(amount)
                .ok_or_else(|| ReportsError::Data("the stock quantities do not fit".to_owned()))?;
        }
    }
    let mut ordered: Vec<(i64, (String, String), [i64; 7])> = items
        .into_iter()
        .map(|(id, (names, sums))| (id, names, sums))
        .collect();
    // The balance is the fifth amount.
    ordered.sort_by(|a, b| b.2.get(4).cmp(&a.2.get(4)).then_with(|| a.0.cmp(&b.0)));
    let all_rows: Vec<Row> = ordered
        .into_iter()
        .map(|(_, (name_ur, name_en), sums)| {
            let mut cells = vec![names_cell(Some(name_ur), Some(name_en))];
            cells.extend(sums.iter().map(|value| Cell::Number { value: *value }));
            Row {
                cells,
                source: None,
                kind: RowKind::Normal,
            }
        })
        .collect();
    let mut total_cells = vec![Cell::Empty];
    total_cells.extend(totals.iter().map(|value| Cell::Number { value: *value }));
    Ok(ReportPage {
        group_above: None,
        tables: Vec::new(),
        id: ReportId::R10,
        title_id: ids::title_id("R10"),
        filters: typed.resolved.clone(),
        columns: vec![
            col("item", ids::COL_ITEM, ColumnKind::Names),
            col("received", ids::COL_RECEIVED, ColumnKind::Number),
            col("sold", ids::COL_SOLD, ColumnKind::Number),
            col("written_off", ids::COL_WRITTEN_OFF, ColumnKind::Number),
            col("fruit_taken", ids::COL_FRUIT_TAKEN, ColumnKind::Number),
            col("balance", ids::COL_BALANCE_QTY, ColumnKind::Number),
            col("our_left", ids::COL_OUR_LEFT, ColumnKind::Number),
            col("other_left", ids::COL_OTHER_LEFT, ColumnKind::Number),
        ],
        rows: spec.slice(&all_rows),
        offset: spec.offset,
        limit: spec.reported_limit(),
        total_count: count_of(all_rows.len()),
        totals_label_id: ids::TOTAL.to_owned(),
        totals: total_cells,
        summary: Vec::new(),
    })
}

/// Builds R10.
///
/// # Errors
/// A database error; `ReportsError::Data` if the own-goods account is missing or the clock gives
/// a day outside the calendar.
pub(crate) fn run(
    conn: &Connection,
    typed: &Typed,
    spec: PageSpec,
) -> Result<ReportPage, ReportsError> {
    let today = typed
        .today
        .ok_or_else(|| ReportsError::Data("the report has no today".to_owned()))?;
    let days_waiting = |received: &str| -> i64 {
        BusinessDate::parse_iso(received).map_or(0, |day| today.days_after(day).max(0))
    };
    let own_account = account_repo::id_by_code(conn, OWN_GOODS_CODE)?
        .ok_or_else(|| ReportsError::Data("the own goods account is missing".to_owned()))?;

    let group_items = items_of_group(conn, typed.item_group)?;
    let in_group = |item_id: i64| {
        group_items
            .as_ref()
            .is_none_or(|ids| ids.contains(&item_id))
    };
    let mut lots = stock_lots(
        conn,
        &StockFilter {
            from: typed.from.clone(),
            to: typed.to.clone(),
            seller_id: typed.seller_id,
            item_id: typed.item_id,
            lot_id: typed.lot_id,
            include_written_off: false,
        },
    )?;
    lots.retain(|lot| in_group(lot.item.id));
    // The purchases of trips with goods left. A seller filter keeps them only if it picks the
    // own-goods account itself.
    // One lot (PRD FR-12.45) is a lot of بکری: no purchase of a trip and no garden arrival is it.
    let mut purchases =
        if typed.lot_id.is_none() && typed.seller_id.is_none_or(|id| id == own_account) {
            mandi_db::purchases::list(
                conn,
                &ListFilter {
                    hide_cancelled: false,
                    order_by: None,
                    search: None,
                    trip_id: None,
                    from: typed.from.clone(),
                    to: typed.to.clone(),
                    item_id: typed.item_id,
                    state: Some(StateFilter::InStock),
                },
                0,
                -1,
            )?
        } else {
            Vec::new()
        };

    purchases.retain(|purchase| in_group(purchase.item.id));

    // The garden arrivals with goods left. A seller filter keeps the arrivals whose garden account
    // is that seller.
    let mut arrivals = if typed.lot_id.is_none() {
        mandi_db::arrivals::with_stock(
            conn,
            typed.from.as_deref(),
            typed.to.as_deref(),
            typed.item_id,
            typed.seller_id,
        )?
    } else {
        Vec::new()
    };
    arrivals.retain(|arrival| in_group(arrival.item.id));

    let mut ids_to_load: Vec<i64> = lots.iter().map(|l| l.seller.id).collect();
    ids_to_load.push(own_account);
    ids_to_load.extend(arrivals.iter().map(|a| a.account.id));
    let parties = load_parties(conn, &ids_to_load)?;

    let mut item_parts: Vec<ItemPart> = Vec::new();
    let mut pending: Vec<Pending> = Vec::with_capacity(
        lots.len()
            .saturating_add(purchases.len())
            .saturating_add(arrivals.len()),
    );
    let mut totals = [0_i64; 7];
    let mut add_totals = |received: i64,
                          sold: i64,
                          written_off: i64,
                          taken: i64,
                          balance: i64,
                          (our, other): (i64, i64)|
     -> Result<(), ReportsError> {
        for (slot, amount) in
            totals
                .iter_mut()
                .zip([received, sold, written_off, taken, balance, our, other])
        {
            *slot = slot
                .checked_add(amount)
                .ok_or_else(|| ReportsError::Data("the stock quantities do not fit".to_owned()))?;
        }
        Ok(())
    };
    for lot in lots {
        let balance = lot
            .qty_received
            .checked_sub(lot.qty_sold)
            .and_then(|v| v.checked_sub(lot.qty_written_off))
            .and_then(|v| v.checked_sub(lot.qty_taken))
            .ok_or_else(|| ReportsError::Data("the quantity of a lot does not fit".to_owned()))?;
        add_totals(
            lot.qty_received,
            lot.qty_sold,
            lot.qty_written_off,
            lot.qty_taken,
            balance,
            (balance, 0),
        )?;
        item_parts.push(ItemPart {
            item_id: lot.item.id,
            names: (lot.item.name_ur.clone(), lot.item.name_en.clone()),
            amounts: [
                lot.qty_received,
                lot.qty_sold,
                lot.qty_written_off,
                lot.qty_taken,
                balance,
                balance,
                0,
            ],
        });
        let seller = parties
            .get(&lot.seller.id)
            .cloned()
            .ok_or_else(|| ReportsError::Data("a lot has an unknown seller".to_owned()))?;
        pending.push(Pending {
            key: (lot.received_date.clone(), 0, lot.bikri_number, lot.lot_no),
            cells: vec![
                Cell::Date {
                    iso: lot.received_date.clone(),
                },
                Cell::Doc {
                    type_id: ids::doc_id("bikri_first"),
                    number: Some(lot.bikri_number),
                },
                Cell::Party { party: seller },
                // The item with the lot's counter, next to the seller: the lot's name (PRD FR-12.45).
                Cell::Names {
                    names: lot_names(&lot.item.name_ur, &lot.item.name_en, lot.lot_counter),
                },
                names_cell(Some(lot.packing.name_ur), Some(lot.packing.name_en)),
                Cell::Number {
                    value: lot.qty_received,
                },
                Cell::Number {
                    value: lot.qty_sold,
                },
                Cell::Number {
                    value: lot.qty_written_off,
                },
                Cell::Number {
                    value: lot.qty_taken,
                },
                Cell::Number { value: balance },
                Cell::Number { value: balance },
                Cell::Number { value: 0 },
                Cell::Empty,
                Cell::Number {
                    value: days_waiting(&lot.received_date),
                },
            ],
            source: RowSource {
                kind: SourceKind::Bikri,
                id: lot.bikri_id,
            },
        });
    }
    let own_party = parties
        .get(&own_account)
        .cloned()
        .ok_or_else(|| ReportsError::Data("the own goods account is missing".to_owned()))?;
    for purchase in purchases {
        let balance = purchase
            .qty
            .checked_sub(purchase.qty_sold)
            .and_then(|v| v.checked_sub(purchase.qty_written_off))
            .ok_or_else(|| {
                ReportsError::Data("the quantity of a purchase does not fit".to_owned())
            })?;
        let parts = StockParts::of(
            purchase.qty,
            purchase.other_qty,
            purchase.qty_sold,
            purchase.qty_sold_other,
            purchase.qty_written_off,
        )
        .ok_or_else(|| ReportsError::Data("the split of a purchase does not fit".to_owned()))?;
        add_totals(
            purchase.qty,
            purchase.qty_sold,
            purchase.qty_written_off,
            0,
            balance,
            (parts.our_left, parts.other_left),
        )?;
        item_parts.push(ItemPart {
            item_id: purchase.item.id,
            names: (purchase.item.name_ur.clone(), purchase.item.name_en.clone()),
            amounts: [
                purchase.qty,
                purchase.qty_sold,
                purchase.qty_written_off,
                0,
                balance,
                parts.our_left,
                parts.other_left,
            ],
        });
        pending.push(Pending {
            key: (purchase.entry_date.clone(), 1, purchase.number, 0),
            cells: vec![
                Cell::Date {
                    iso: purchase.entry_date.clone(),
                },
                Cell::Doc {
                    type_id: ids::doc_id("purchase"),
                    number: Some(purchase.number),
                },
                Cell::Party {
                    party: own_party.clone(),
                },
                names_cell(Some(purchase.item.name_ur), Some(purchase.item.name_en)),
                names_cell(
                    Some(purchase.packing.name_ur),
                    Some(purchase.packing.name_en),
                ),
                Cell::Number {
                    value: purchase.qty,
                },
                Cell::Number {
                    value: purchase.qty_sold,
                },
                Cell::Number {
                    value: purchase.qty_written_off,
                },
                // Trip goods are not offered as fruit: none was taken.
                Cell::Number { value: 0 },
                Cell::Number { value: balance },
                Cell::Number {
                    value: parts.our_left,
                },
                Cell::Number {
                    value: parts.other_left,
                },
                shop_cell(purchase.shop_name.as_deref()),
                Cell::Number {
                    value: days_waiting(&purchase.sell_date),
                },
            ],
            source: RowSource {
                kind: SourceKind::Purchase,
                id: purchase.id,
            },
        });
    }
    for arrival in arrivals {
        let balance = arrival
            .qty
            .checked_sub(arrival.qty_sold)
            .and_then(|v| v.checked_sub(arrival.qty_written_off))
            .ok_or_else(|| {
                ReportsError::Data("the quantity of an arrival does not fit".to_owned())
            })?;
        let parts = StockParts::of(
            arrival.qty,
            arrival.other_qty,
            arrival.qty_sold,
            arrival.qty_sold_other,
            arrival.qty_written_off,
        )
        .ok_or_else(|| ReportsError::Data("the split of an arrival does not fit".to_owned()))?;
        add_totals(
            arrival.qty,
            arrival.qty_sold,
            arrival.qty_written_off,
            0,
            balance,
            (parts.our_left, parts.other_left),
        )?;
        item_parts.push(ItemPart {
            item_id: arrival.item.id,
            names: (arrival.item.name_ur.clone(), arrival.item.name_en.clone()),
            amounts: [
                arrival.qty,
                arrival.qty_sold,
                arrival.qty_written_off,
                0,
                balance,
                parts.our_left,
                parts.other_left,
            ],
        });
        let garden = parties.get(&arrival.account.id).cloned().ok_or_else(|| {
            ReportsError::Data("an arrival has an unknown garden account".to_owned())
        })?;
        pending.push(Pending {
            key: (arrival.entry_date.clone(), 2, arrival.number, 0),
            cells: vec![
                Cell::Date {
                    iso: arrival.entry_date.clone(),
                },
                Cell::Doc {
                    type_id: ids::doc_id("garden_arrival"),
                    number: Some(arrival.number),
                },
                Cell::Party { party: garden },
                names_cell(
                    Some(arrival.item.name_ur.clone()),
                    Some(arrival.item.name_en.clone()),
                ),
                names_cell(
                    Some(arrival.packing.name_ur.clone()),
                    Some(arrival.packing.name_en.clone()),
                ),
                Cell::Number { value: arrival.qty },
                Cell::Number {
                    value: arrival.qty_sold,
                },
                Cell::Number {
                    value: arrival.qty_written_off,
                },
                // Garden goods are taken as fruit through a lot, never from an arrival.
                Cell::Number { value: 0 },
                Cell::Number { value: balance },
                Cell::Number {
                    value: parts.our_left,
                },
                Cell::Number {
                    value: parts.other_left,
                },
                shop_cell(arrival.shop_name.as_deref()),
                Cell::Number {
                    value: days_waiting(&arrival.sell_date),
                },
            ],
            source: RowSource {
                kind: SourceKind::Arrival,
                id: arrival.id,
            },
        });
    }
    if typed.stock_view == StockView::Item {
        return by_item(typed, item_parts, totals, spec);
    }
    pending.sort_by(|a, b| a.key.cmp(&b.key));
    let all_rows: Vec<Row> = pending
        .into_iter()
        .map(|p| Row {
            cells: p.cells,
            source: Some(p.source),
            kind: RowKind::Normal,
        })
        .collect();

    let [
        received,
        sold,
        written_off,
        taken,
        balance,
        our_left,
        other_left,
    ] = totals;
    Ok(ReportPage {
        group_above: None,
        tables: Vec::new(),
        id: ReportId::R10,
        title_id: ids::title_id("R10"),
        filters: typed.resolved.clone(),
        columns: vec![
            col("date", ids::COL_DATE, ColumnKind::Date),
            col("doc", ids::COL_DOC, ColumnKind::Doc),
            col("seller", ids::COL_SELLER, ColumnKind::Party),
            col("item", ids::COL_ITEM, ColumnKind::Names),
            col("packing", ids::COL_PACKING, ColumnKind::Names),
            col("received", ids::COL_RECEIVED, ColumnKind::Number),
            col("sold", ids::COL_SOLD, ColumnKind::Number),
            col("written_off", ids::COL_WRITTEN_OFF, ColumnKind::Number),
            col("fruit_taken", ids::COL_FRUIT_TAKEN, ColumnKind::Number),
            col("balance", ids::COL_BALANCE_QTY, ColumnKind::Number),
            col("our_left", ids::COL_OUR_LEFT, ColumnKind::Number),
            col("other_left", ids::COL_OTHER_LEFT, ColumnKind::Number),
            col("shop_name", ids::COL_SHOP_NAME, ColumnKind::Text),
            col("days", ids::COL_DAYS_WAITING, ColumnKind::Number),
        ],
        rows: spec.slice(&all_rows),
        offset: spec.offset,
        limit: spec.reported_limit(),
        total_count: count_of(all_rows.len()),
        totals_label_id: ids::TOTAL.to_owned(),
        totals: vec![
            Cell::Empty,
            Cell::Empty,
            Cell::Empty,
            Cell::Empty,
            Cell::Empty,
            Cell::Number { value: received },
            Cell::Number { value: sold },
            Cell::Number { value: written_off },
            Cell::Number { value: taken },
            Cell::Number { value: balance },
            Cell::Number { value: our_left },
            Cell::Number { value: other_left },
            Cell::Empty,
            Cell::Empty,
        ],
        summary: Vec::new(),
    })
}
