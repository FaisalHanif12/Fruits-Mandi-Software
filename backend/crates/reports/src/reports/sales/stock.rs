//! R10 مکمل سٹاک / باقی مال (the goods not yet sold) (PRD 19.2, FR-12.14, FR-12.25, C16).
//!
//! One row is a lot of a saved بکری with goods left, or a purchase of a market trip with stock
//! left (marked اپنا مال: its seller is the own-goods account and its document is the purchase).
//! The row says what was received, sold and written off, the balance and how many days the goods
//! have been waiting. The figures are the ones the lots and the purchases keep up to date with
//! every save, the same as the باقی مال screen reads (so the report and the screen agree). The
//! totals row adds up the four quantities over the whole filter.
//!
//! The "taken as fruit" column of the PRD line comes with the fruit screen (D4R); this report has
//! no such column yet.
//!
//! Example (E2E-16 on 10-10-26): one lot, رحیم کیلا, received 50, sold 45, written off 0,
//! balance 5, 0 days.

use mandi_core::accounts::OWN_GOODS_CODE;
use mandi_core::dates::BusinessDate;
use mandi_db::accounts as account_repo;
use mandi_db::bikri::edit::{StockFilter, stock_lots};
use mandi_db::purchases::{ListFilter, StateFilter};
use mandi_db::rusqlite::Connection;

use super::super::PageSpec;
use crate::error::ReportsError;
use crate::reports::common::{col, count_of, load_parties, names_cell, pukhta_party};
use crate::reports::filters::Typed;
use crate::reports::ids;
use crate::reports::model::{
    Cell, ColumnKind, ReportId, ReportPage, Row, RowKind, RowSource, SourceKind,
};

/// A row waiting to be sorted: the day, the kind (a lot before a purchase), the document number
/// and the lot number, then the cells.
struct Pending {
    key: (String, u8, i64, i64),
    cells: Vec<Cell>,
    source: RowSource,
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

    let lots = stock_lots(
        conn,
        &StockFilter {
            from: typed.from.clone(),
            to: typed.to.clone(),
            seller_id: typed.seller_id,
            item_id: typed.item_id,
            include_written_off: false,
        },
    )?;
    // The purchases of trips with goods left. A seller filter keeps them only if it picks the
    // own-goods account itself.
    let purchases = if typed.seller_id.is_none_or(|id| id == own_account) {
        mandi_db::purchases::list(
            conn,
            &ListFilter {
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

    let mut ids_to_load: Vec<i64> = lots.iter().map(|l| l.seller.id).collect();
    ids_to_load.push(own_account);
    let parties = load_parties(conn, &ids_to_load)?;

    let mut pending: Vec<Pending> = Vec::with_capacity(lots.len().saturating_add(purchases.len()));
    let mut totals = [0_i64; 4];
    let mut add_totals =
        |received: i64, sold: i64, written_off: i64, balance: i64| -> Result<(), ReportsError> {
            for (slot, amount) in totals
                .iter_mut()
                .zip([received, sold, written_off, balance])
            {
                *slot = slot.checked_add(amount).ok_or_else(|| {
                    ReportsError::Data("the stock quantities do not fit".to_owned())
                })?;
            }
            Ok(())
        };
    for lot in lots {
        let balance = lot
            .qty_received
            .checked_sub(lot.qty_sold)
            .and_then(|v| v.checked_sub(lot.qty_written_off))
            .ok_or_else(|| ReportsError::Data("the quantity of a lot does not fit".to_owned()))?;
        add_totals(lot.qty_received, lot.qty_sold, lot.qty_written_off, balance)?;
        let mut seller = parties
            .get(&lot.seller.id)
            .cloned()
            .ok_or_else(|| ReportsError::Data("a lot has an unknown seller".to_owned()))?;
        if let Some((name_ur, name_en, _phone, key)) = &lot.pukhta {
            seller = pukhta_party(seller, Some(key.clone()), Some(name_ur), Some(name_en));
        }
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
                names_cell(Some(lot.item.name_ur), Some(lot.item.name_en)),
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
                Cell::Number { value: balance },
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
        add_totals(
            purchase.qty,
            purchase.qty_sold,
            purchase.qty_written_off,
            balance,
        )?;
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
                Cell::Number { value: balance },
                Cell::Number {
                    value: days_waiting(&purchase.entry_date),
                },
            ],
            source: RowSource {
                kind: SourceKind::Purchase,
                id: purchase.id,
            },
        });
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

    let [received, sold, written_off, balance] = totals;
    Ok(ReportPage {
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
            col("balance", ids::COL_BALANCE_QTY, ColumnKind::Number),
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
            Cell::Number { value: balance },
            Cell::Empty,
        ],
        summary: Vec::new(),
    })
}
