//! R6 بکری تفصیل / روزنامچہ بکری (the seller summary, one row for each lot) and R27 بیجک سیریز
//! (the bill series, one row for each seller bill) (PRD 19.2, FR-19.10, FR-19.11).
//!
//! Both read the values stored on the lot at save time: خام بکری, the seven deductions, کل
//! اخراجات and صافی بکری (Code Standards B20). A lot that sold in more than one posting shows
//! the totals of all its postings, as the lot does (PRD 12.9, E2E-16b). Nothing is worked out
//! again. The deductions in the columns add up to کل اخراجات, because the database checks it
//! (`CHECK` on `bikri_lots`): freight only when the firm paid the driver, commission, labour,
//! munshiana, نقد, store rent and phone.
//!
//! A cancelled بکری is not listed. The date range is the date of the بکری.
//!
//! Example (E2E-1): رحیم's lot of بکری 1 reads خام 29,400, کرایہ 3,000, کمیشن 1,764, مزدوری 250,
//! منشیانہ 50, نقد 5,000, فون 20, کل اخراجات 10,084, صافی 19,316.

use mandi_db::rusqlite::types::Value;
use mandi_db::rusqlite::{Connection, params_from_iter};

use super::super::PageSpec;
use crate::error::ReportsError;
use crate::reports::common::{
    Conds, col, load_parties, names_cell, paisa, party_of_line, pukhta_party,
};
use crate::reports::filters::Typed;
use crate::reports::ids;
use crate::reports::model::{
    Cell, ColumnKind, ReportId, ReportPage, Row, RowKind, RowSource, SourceKind,
};

/// The conditions of R6 and R27, with only the filters that are set (Code Standards B18).
fn conds(typed: &Typed) -> Conds {
    let mut c = Conds::new();
    c.always("b.status = 'saved'")
        .text("b.entry_date >= ?", typed.from.as_deref())
        .text("b.entry_date <= ?", typed.to.as_deref())
        .id("l.seller_account_id = ?", typed.seller_id)
        .id("b.zamindar_account_id = ?", typed.zamindar_id)
        .id("l.item_id = ?", typed.item_id)
        .id("b.adda_id = ?", typed.adda_id);
    c
}

/// One page of lots of R6.
pub(crate) fn r6_page_query(typed: &Typed, limit: i64, offset: i64) -> (String, Vec<Value>) {
    let c = conds(typed);
    let sql = format!(
        "SELECT b.id, b.number, b.entry_date, l.seller_account_id, l.pukhta_person_key,
            l.pukhta_name_ur, l.pukhta_name_en, i.name_ur, i.name_en, l.qty_sold, l.gross_paisa,
            CASE WHEN l.freight_paid_by = 'firm' THEN l.freight_paisa ELSE 0 END,
            l.seller_comm_paisa, l.labour_paisa, l.munshiana_paisa, l.cash_paid_paisa,
            l.store_rent_paisa, l.phone_paisa, l.deductions_paisa, l.net_paisa
         FROM bikri_entries b
         JOIN bikri_lots l ON l.bikri_id = b.id
         JOIN items i ON i.id = l.item_id
         WHERE {}
         ORDER BY b.entry_date, b.number, l.lot_no
         LIMIT ? OFFSET ?",
        c.sql()
    );
    (sql, c.page_args(limit, offset))
}

/// The count and the totals of R6 over the whole filter.
fn r6_totals_query(typed: &Typed) -> (String, Vec<Value>) {
    let c = conds(typed);
    let sql = format!(
        "SELECT COUNT(*), COALESCE(SUM(l.qty_sold), 0), COALESCE(SUM(l.gross_paisa), 0),
            COALESCE(SUM(CASE WHEN l.freight_paid_by = 'firm' THEN l.freight_paisa ELSE 0 END), 0),
            COALESCE(SUM(l.seller_comm_paisa), 0), COALESCE(SUM(l.labour_paisa), 0),
            COALESCE(SUM(l.munshiana_paisa), 0), COALESCE(SUM(l.cash_paid_paisa), 0),
            COALESCE(SUM(l.store_rent_paisa), 0), COALESCE(SUM(l.phone_paisa), 0),
            COALESCE(SUM(l.deductions_paisa), 0), COALESCE(SUM(l.net_paisa), 0)
         FROM bikri_entries b
         JOIN bikri_lots l ON l.bikri_id = b.id
         WHERE {}",
        c.sql()
    );
    (sql, c.args().to_vec())
}

/// One page of seller bills of R27: the lots of one بکری and one seller together.
fn r27_page_query(typed: &Typed, limit: i64, offset: i64) -> (String, Vec<Value>) {
    let c = conds(typed);
    let sql = format!(
        "SELECT b.id, b.number, b.entry_date, b.bill_date, l.seller_account_id,
            l.pukhta_person_key, MAX(l.pukhta_name_ur), MAX(l.pukhta_name_en),
            SUM(l.gross_paisa), SUM(l.deductions_paisa), SUM(l.net_paisa)
         FROM bikri_entries b
         JOIN bikri_lots l ON l.bikri_id = b.id
         WHERE {}
         GROUP BY b.id, l.seller_account_id, l.pukhta_person_key
         ORDER BY b.number, l.seller_account_id, l.pukhta_person_key
         LIMIT ? OFFSET ?",
        c.sql()
    );
    (sql, c.page_args(limit, offset))
}

/// The number of seller bills of R27 over the whole filter.
fn r27_count_query(typed: &Typed) -> (String, Vec<Value>) {
    let c = conds(typed);
    let sql = format!(
        "SELECT COUNT(*) FROM (SELECT 1 FROM bikri_entries b
         JOIN bikri_lots l ON l.bikri_id = b.id
         WHERE {}
         GROUP BY b.id, l.seller_account_id, l.pukhta_person_key)",
        c.sql()
    );
    (sql, c.args().to_vec())
}

/// The totals of R27 over the whole filter.
fn r27_totals_query(typed: &Typed) -> (String, Vec<Value>) {
    let c = conds(typed);
    let sql = format!(
        "SELECT COALESCE(SUM(l.gross_paisa), 0), COALESCE(SUM(l.deductions_paisa), 0),
            COALESCE(SUM(l.net_paisa), 0)
         FROM bikri_entries b
         JOIN bikri_lots l ON l.bikri_id = b.id
         WHERE {}",
        c.sql()
    );
    (sql, c.args().to_vec())
}

struct LotRow {
    bikri_id: i64,
    number: i64,
    entry_date: String,
    seller_id: i64,
    person_key: Option<String>,
    person_ur: Option<String>,
    person_en: Option<String>,
    item_ur: String,
    item_en: String,
    qty_sold: i64,
    amounts: [i64; 10],
}

/// Builds R6, one row for each lot.
///
/// # Errors
/// A database error; `ReportsError::Money` if a stored amount is not whole rupees.
pub(crate) fn run_r6(
    conn: &Connection,
    typed: &Typed,
    spec: PageSpec,
) -> Result<ReportPage, ReportsError> {
    let (totals_sql, totals_args) = r6_totals_query(typed);
    let (count, qty, totals): (i64, i64, [i64; 10]) =
        conn.query_row(&totals_sql, params_from_iter(totals_args.iter()), |r| {
            let mut sums = [0_i64; 10];
            for (slot, index) in sums.iter_mut().zip(2_usize..) {
                *slot = r.get(index)?;
            }
            Ok((r.get(0)?, r.get(1)?, sums))
        })?;

    let (page_sql, page_args) = r6_page_query(typed, spec.sql_limit(), spec.offset);
    let mut statement = conn.prepare(&page_sql)?;
    let lots = statement
        .query_map(params_from_iter(page_args.iter()), |r| {
            let mut amounts = [0_i64; 10];
            for (slot, index) in amounts.iter_mut().zip(10_usize..) {
                *slot = r.get(index)?;
            }
            Ok(LotRow {
                bikri_id: r.get(0)?,
                number: r.get(1)?,
                entry_date: r.get(2)?,
                seller_id: r.get(3)?,
                person_key: r.get(4)?,
                person_ur: r.get(5)?,
                person_en: r.get(6)?,
                item_ur: r.get(7)?,
                item_en: r.get(8)?,
                qty_sold: r.get(9)?,
                amounts,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let seller_ids: Vec<i64> = lots.iter().map(|l| l.seller_id).collect();
    let parties = load_parties(conn, &seller_ids)?;
    let mut rows = Vec::with_capacity(lots.len());
    for lot in lots {
        let party = pukhta_party(
            party_of_line(&parties, lot.seller_id, None)?,
            lot.person_key.clone(),
            lot.person_ur.as_deref(),
            lot.person_en.as_deref(),
        );
        let mut cells = vec![
            Cell::Date {
                iso: lot.entry_date.clone(),
            },
            Cell::Doc {
                type_id: ids::doc_id("bikri_first"),
                number: Some(lot.number),
            },
            Cell::Party { party },
            names_cell(Some(lot.item_ur), Some(lot.item_en)),
            Cell::Number {
                value: lot.qty_sold,
            },
        ];
        for amount in lot.amounts {
            cells.push(Cell::Money {
                paisa: paisa(amount)?,
            });
        }
        rows.push(Row {
            cells,
            source: Some(RowSource {
                kind: SourceKind::Bikri,
                id: lot.bikri_id,
            }),
            kind: RowKind::Normal,
        });
    }

    let mut total_cells = vec![
        Cell::Empty,
        Cell::Empty,
        Cell::Empty,
        Cell::Empty,
        Cell::Number { value: qty },
    ];
    for amount in totals {
        total_cells.push(Cell::Money {
            paisa: paisa(amount)?,
        });
    }
    Ok(ReportPage {
        id: ReportId::R6,
        title_id: ids::title_id("R6"),
        filters: typed.resolved.clone(),
        columns: vec![
            col("date", ids::COL_DATE, ColumnKind::Date),
            col("doc", ids::COL_DOC, ColumnKind::Doc),
            col("seller", ids::COL_SELLER, ColumnKind::Party),
            col("item", ids::COL_ITEM, ColumnKind::Names),
            col("qty", ids::COL_QTY, ColumnKind::Number),
            col("gross", ids::COL_GROSS, ColumnKind::Money),
            col("freight", ids::COL_FREIGHT, ColumnKind::Money),
            col("commission", ids::COL_COMMISSION, ColumnKind::Money),
            col("labour", ids::COL_LABOUR, ColumnKind::Money),
            col("munshiana", ids::COL_MUNSHIANA, ColumnKind::Money),
            col("cash_paid", ids::COL_CASH_PAID, ColumnKind::Money),
            col("store_rent", ids::COL_STORE_RENT, ColumnKind::Money),
            col("phone_charge", ids::COL_PHONE_CHARGE, ColumnKind::Money),
            col("deductions", ids::COL_DEDUCTIONS, ColumnKind::Money),
            col("net", ids::COL_NET, ColumnKind::Money),
        ],
        rows,
        offset: spec.offset,
        limit: spec.reported_limit(),
        total_count: count,
        totals_label_id: ids::TOTAL.to_owned(),
        totals: total_cells,
        summary: Vec::new(),
    })
}

struct BillRow {
    bikri_id: i64,
    number: i64,
    entry_date: String,
    bill_date: String,
    seller_id: i64,
    person_key: Option<String>,
    person_ur: Option<String>,
    person_en: Option<String>,
    gross: i64,
    deductions: i64,
    net: i64,
}

/// Builds R27, one row for each seller bill in number order.
///
/// # Errors
/// A database error; `ReportsError::Money` if a stored amount is not whole rupees.
pub(crate) fn run_r27(
    conn: &Connection,
    typed: &Typed,
    spec: PageSpec,
) -> Result<ReportPage, ReportsError> {
    let (count_sql, count_args) = r27_count_query(typed);
    let count: i64 = conn.query_row(&count_sql, params_from_iter(count_args.iter()), |r| {
        r.get(0)
    })?;
    let (totals_sql, totals_args) = r27_totals_query(typed);
    let (gross, deductions, net): (i64, i64, i64) =
        conn.query_row(&totals_sql, params_from_iter(totals_args.iter()), |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?))
        })?;
    let (page_sql, page_args) = r27_page_query(typed, spec.sql_limit(), spec.offset);
    let mut statement = conn.prepare(&page_sql)?;
    let bills = statement
        .query_map(params_from_iter(page_args.iter()), |r| {
            Ok(BillRow {
                bikri_id: r.get(0)?,
                number: r.get(1)?,
                entry_date: r.get(2)?,
                bill_date: r.get(3)?,
                seller_id: r.get(4)?,
                person_key: r.get(5)?,
                person_ur: r.get(6)?,
                person_en: r.get(7)?,
                gross: r.get(8)?,
                deductions: r.get(9)?,
                net: r.get(10)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    let seller_ids: Vec<i64> = bills.iter().map(|b| b.seller_id).collect();
    let parties = load_parties(conn, &seller_ids)?;
    let mut rows = Vec::with_capacity(bills.len());
    for bill in bills {
        let party = pukhta_party(
            party_of_line(&parties, bill.seller_id, None)?,
            bill.person_key.clone(),
            bill.person_ur.as_deref(),
            bill.person_en.as_deref(),
        );
        rows.push(Row {
            cells: vec![
                Cell::Doc {
                    type_id: ids::doc_id("bikri_first"),
                    number: Some(bill.number),
                },
                Cell::Date {
                    iso: bill.entry_date,
                },
                Cell::Date {
                    iso: bill.bill_date,
                },
                Cell::Party { party },
                Cell::Money {
                    paisa: paisa(bill.gross)?,
                },
                Cell::Money {
                    paisa: paisa(bill.deductions)?,
                },
                Cell::Money {
                    paisa: paisa(bill.net)?,
                },
            ],
            source: Some(RowSource {
                kind: SourceKind::Bikri,
                id: bill.bikri_id,
            }),
            kind: RowKind::Normal,
        });
    }
    Ok(ReportPage {
        id: ReportId::R27,
        title_id: ids::title_id("R27"),
        filters: typed.resolved.clone(),
        columns: vec![
            col("doc", ids::COL_DOC, ColumnKind::Doc),
            col("date", ids::COL_DATE, ColumnKind::Date),
            col("bill_date", ids::COL_BILL_DATE, ColumnKind::Date),
            col("seller", ids::COL_SELLER, ColumnKind::Party),
            col("gross", ids::COL_GROSS, ColumnKind::Money),
            col("deductions", ids::COL_DEDUCTIONS, ColumnKind::Money),
            col("net", ids::COL_NET, ColumnKind::Money),
        ],
        rows,
        offset: spec.offset,
        limit: spec.reported_limit(),
        total_count: count,
        totals_label_id: ids::TOTAL.to_owned(),
        totals: vec![
            Cell::Empty,
            Cell::Empty,
            Cell::Empty,
            Cell::Empty,
            Cell::Money {
                paisa: paisa(gross)?,
            },
            Cell::Money {
                paisa: paisa(deductions)?,
            },
            Cell::Money { paisa: paisa(net)? },
        ],
        summary: Vec::new(),
    })
}
