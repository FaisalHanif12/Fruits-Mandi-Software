//! R2 مکمل بکری (full sales register), R3 نقدی (cash sales) and R4 ادھاری (credit sales)
//! (PRD 19.2, FR-19.10, FR-19.11).
//!
//! One row for each sale line in force (`is_active = 1`; a line replaced by an edit or ended by a
//! cancel is not in force). The three reports are one query with one more condition: R3 keeps
//! the cash lines, R4 the credit lines. Every figure is a stored value of the line; nothing is
//! worked out again from a rule (Code Standards B20). The totals are `SUM()` of the stored
//! `INTEGER` columns over the whole filter, from one query (Code Standards B31).
//!
//! The rows are found through the date index of the journal entry the line was posted in, which
//! is the date of the posting the line belongs to (PRD 12.4).
//!
//! Example (the worked day, 10-10-26, R2): three lines, 60 نگ, value 12,000 + 17,400 + 12,000 =
//! 41,400, buyer totals 12,880 + 18,684 + 12,800 = 44,364.

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

/// Which lines a register keeps.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Kind {
    /// R2: every line.
    All,
    /// R3: cash lines (نقدی).
    Cash,
    /// R4: credit lines (ادھاری).
    Credit,
}

impl Kind {
    fn payment_type(self) -> Option<&'static str> {
        match self {
            Kind::All => None,
            Kind::Cash => Some("cash"),
            Kind::Credit => Some("credit"),
        }
    }

    fn id(self) -> ReportId {
        match self {
            Kind::All => ReportId::R2,
            Kind::Cash => ReportId::R3,
            Kind::Credit => ReportId::R4,
        }
    }
}

/// The conditions shared by the page and the totals, so the two queries can never differ. Only
/// the filters that are set are in the clause, so a date range is found through the date index
/// of the journal entry (Code Standards B18).
pub(super) fn conds(typed: &Typed, payment: Option<&'static str>) -> Conds {
    let mut c = Conds::new();
    c.always("s.is_active = 1")
        .always("je.status = 'posted'")
        .text("je.entry_date >= ?", typed.from.as_deref())
        .text("je.entry_date <= ?", typed.to.as_deref())
        .text("s.payment_type = ?", payment)
        .id("s.buyer_account_id = ?", typed.party_id)
        .text("s.pukhta_person_key = ?", typed.person_key.as_deref())
        .id("l.seller_account_id = ?", typed.seller_id)
        .id("b.zamindar_account_id = ?", typed.zamindar_id)
        .id("l.item_id = ?", typed.item_id)
        .id("ba.city_id = ?", typed.city_id)
        .id("b.adda_id = ?", typed.adda_id);
    c
}

/// One page of lines, in the order of the register, and its values (the query plan test reads
/// it through `reports::plans`, Code Standards B18).
pub(crate) fn page_query(
    typed: &Typed,
    payment: Option<&'static str>,
    limit: i64,
    offset: i64,
) -> (String, Vec<Value>) {
    let c = conds(typed, payment);
    let sql = format!(
        "SELECT je.entry_date, je.posting_type, b.id, b.number,
           s.buyer_account_id, s.pukhta_person_key, s.pukhta_name_ur, s.pukhta_name_en,
           ci.name_ur, ci.name_en, i.name_ur, i.name_en,
           s.qty, s.buyer_rate_paisa, s.buyer_value_paisa, s.buyer_total_paisa
         FROM journal_entries je
         JOIN bikri_sales s ON s.journal_entry_id = je.id
         JOIN bikri_lots l ON l.id = s.lot_id
         JOIN bikri_entries b ON b.id = s.bikri_id
         JOIN items i ON i.id = l.item_id
         JOIN accounts ba ON ba.id = s.buyer_account_id
         LEFT JOIN cities ci ON ci.id = ba.city_id
         WHERE {}
         ORDER BY je.entry_date, b.number, s.id
         LIMIT ? OFFSET ?",
        c.sql()
    );
    (sql, c.page_args(limit, offset))
}

/// The count and the totals of the whole filter, from one query.
pub(crate) fn totals_query(typed: &Typed, payment: Option<&'static str>) -> (String, Vec<Value>) {
    let c = conds(typed, payment);
    let sql = format!(
        "SELECT COUNT(*), COALESCE(SUM(s.qty), 0), COALESCE(SUM(s.buyer_value_paisa), 0),
                COALESCE(SUM(s.buyer_total_paisa), 0)
         FROM journal_entries je
         JOIN bikri_sales s ON s.journal_entry_id = je.id
         JOIN bikri_lots l ON l.id = s.lot_id
         JOIN bikri_entries b ON b.id = s.bikri_id
         JOIN accounts ba ON ba.id = s.buyer_account_id
         WHERE {}",
        c.sql()
    );
    (sql, c.args().to_vec())
}

struct Line {
    entry_date: String,
    posting_type: String,
    bikri_id: i64,
    bikri_number: i64,
    buyer_id: i64,
    person_key: Option<String>,
    person_ur: Option<String>,
    person_en: Option<String>,
    city_ur: Option<String>,
    city_en: Option<String>,
    item_ur: String,
    item_en: String,
    qty: i64,
    rate: i64,
    value: i64,
    total: i64,
}

/// Builds R2, R3 or R4.
///
/// # Errors
/// A database error; `ReportsError::Money` if a stored amount is not whole rupees or a total
/// does not fit.
pub(crate) fn run(
    conn: &Connection,
    typed: &Typed,
    spec: PageSpec,
    kind: Kind,
) -> Result<ReportPage, ReportsError> {
    let payment = kind.payment_type();
    let (totals_sql, totals_args) = totals_query(typed, payment);
    let (count, qty, value, total): (i64, i64, i64, i64) =
        conn.query_row(&totals_sql, params_from_iter(totals_args.iter()), |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
        })?;

    let (page_sql, page_args) = page_query(typed, payment, spec.sql_limit(), spec.offset);
    let mut statement = conn.prepare(&page_sql)?;
    let lines = statement
        .query_map(params_from_iter(page_args.iter()), |r| {
            Ok(Line {
                entry_date: r.get(0)?,
                posting_type: r.get(1)?,
                bikri_id: r.get(2)?,
                bikri_number: r.get(3)?,
                buyer_id: r.get(4)?,
                person_key: r.get(5)?,
                person_ur: r.get(6)?,
                person_en: r.get(7)?,
                city_ur: r.get(8)?,
                city_en: r.get(9)?,
                item_ur: r.get(10)?,
                item_en: r.get(11)?,
                qty: r.get(12)?,
                rate: r.get(13)?,
                value: r.get(14)?,
                total: r.get(15)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let buyer_ids: Vec<i64> = lines.iter().map(|l| l.buyer_id).collect();
    let parties = load_parties(conn, &buyer_ids)?;
    let mut rows = Vec::with_capacity(lines.len());
    for line in lines {
        let base = party_of_line(&parties, line.buyer_id, None)?;
        let party = pukhta_party(
            base,
            line.person_key.clone(),
            line.person_ur.as_deref(),
            line.person_en.as_deref(),
        );
        rows.push(Row {
            cells: vec![
                Cell::Date {
                    iso: line.entry_date.clone(),
                },
                Cell::Doc {
                    type_id: ids::doc_id(&line.posting_type),
                    number: Some(line.bikri_number),
                },
                Cell::Party { party },
                names_cell(line.city_ur, line.city_en),
                names_cell(Some(line.item_ur), Some(line.item_en)),
                Cell::Number { value: line.qty },
                Cell::Money {
                    paisa: paisa(line.rate)?,
                },
                Cell::Money {
                    paisa: paisa(line.value)?,
                },
                Cell::Money {
                    paisa: paisa(line.total)?,
                },
            ],
            source: Some(RowSource {
                kind: SourceKind::Bikri,
                id: line.bikri_id,
            }),
            kind: RowKind::Normal,
        });
    }

    let id = kind.id();
    Ok(ReportPage {
        group_above: None,
        id,
        title_id: ids::title_id(id.code()),
        filters: typed.resolved.clone(),
        columns: vec![
            col("date", ids::COL_DATE, ColumnKind::Date),
            col("doc", ids::COL_DOC, ColumnKind::Doc),
            col("buyer", ids::COL_BUYER, ColumnKind::Party),
            col("city", ids::COL_CITY, ColumnKind::Names),
            col("item", ids::COL_ITEM, ColumnKind::Names),
            col("qty", ids::COL_QTY, ColumnKind::Number),
            col("rate", ids::COL_RATE, ColumnKind::Money),
            col("value", ids::COL_VALUE, ColumnKind::Money),
            col("buyer_total", ids::COL_BUYER_TOTAL, ColumnKind::Money),
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
            Cell::Empty,
            Cell::Number { value: qty },
            Cell::Empty,
            Cell::Money {
                paisa: paisa(value)?,
            },
            Cell::Money {
                paisa: paisa(total)?,
            },
        ],
        summary: Vec::new(),
    })
}
