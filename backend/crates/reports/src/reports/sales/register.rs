//! R2 مکمل بکری (full sales register), R3 نقدی (cash sales) and R4 ادھاری (credit sales)
//! (PRD 19.2, FR-19.10, FR-19.11, FR-12.44).
//!
//! One row for each sale line in force (`is_active = 1`; a line replaced by an edit or ended by a
//! cancel is not in force). The three reports are one query with one more condition: R3 keeps
//! the lines with money paid on the spot (paid above 0), R4 the lines with something left on the
//! account (paid below the buyer total). A part-paid line is on both (plan 60 P12). R3 and R4 have
//! two more columns at the end, `paid` and `left`; R2 is as it was. Every figure is a stored value of the line; nothing is
//! worked out again from a rule (Code Standards B20). The totals are `SUM()` of the stored
//! `INTEGER` columns over the whole filter, from one query (Code Standards B31).
//!
//! The rows are found through the date index of the journal entry the line was posted in, which
//! is the date of the posting the line belongs to (PRD 12.4).
//!
//! Example (the worked day, 10-10-26, R2): three lines, 60 نگ, value 12,000 + 17,400 + 12,000 =
//! 41,400, buyer totals 12,880 + 18,684 + 12,800 = 44,364. R3 has شاہد's line only (paid 12,800,
//! left 0). R4 has اکرم's and ارشد's lines (paid 0, left 12,880 and 18,684).

use mandi_db::rusqlite::types::Value;
use mandi_db::rusqlite::{Connection, params_from_iter};

use super::super::PageSpec;
use super::buyer_bills::left_of;
use crate::error::ReportsError;
use crate::reports::common::{
    Conds, chukta_party, col, grade_of, load_parties, lot_names, names_cell, paisa, party_of_line,
};
use crate::reports::filters::{LineSort, Typed};
use crate::reports::ids;
use crate::reports::model::{
    Cell, ColumnKind, ReportId, ReportPage, Row, RowKind, RowSource, SourceKind,
};
use mandi_core::money::Paisa;

/// Which lines a register keeps.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Kind {
    /// R2: every line.
    All,
    /// R3: lines with money paid on the spot (نقدی).
    Cash,
    /// R4: lines with something left on the account (ادھاری).
    Credit,
}

impl Kind {
    /// The condition of the kind, a fixed fragment (PRD FR-12.44). Nothing is worked out in SQL:
    /// both sides are stored columns.
    fn condition(self) -> Option<&'static str> {
        match self {
            Kind::All => None,
            Kind::Cash => Some("s.paid_paisa > 0"),
            Kind::Credit => Some("s.paid_paisa < s.buyer_total_paisa"),
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
pub(super) fn conds(typed: &Typed, condition: Option<&'static str>) -> Conds {
    let mut c = Conds::new();
    c.always("s.is_active = 1")
        .always("je.status = 'posted'")
        .text("je.entry_date >= ?", typed.from.as_deref())
        .text("je.entry_date <= ?", typed.to.as_deref());
    if let Some(condition) = condition {
        c.always(condition);
    }
    // The quality of the line (PRD FR-12.42): only the lines of one quality.
    c.text("s.grade = ?", typed.grade.map(|g| g.as_str()))
        .id("s.buyer_account_id = ?", typed.party_id)
        .text("s.chukta_person_key = ?", typed.person_key.as_deref())
        .id("l.seller_account_id = ?", typed.seller_id)
        .id("b.zamindar_account_id = ?", typed.zamindar_id)
        // One بکری (R5 only: no other register takes the filter, so it is never set for them).
        .id("b.id = ?", typed.bikri_id)
        .id("l.item_id = ?", typed.item_id)
        // One lot (PRD FR-12.45): the lot the Owner picked in the assistant.
        .id("l.id = ?", typed.lot_id)
        // All the items of a group (PRD v1.29); never together with one item.
        .item_group("l.item_id", typed.item_group)
        .id("ba.city_id = ?", typed.city_id)
        .id("b.adda_id = ?", typed.adda_id);
    c
}

/// One page of lines, in the order of the register, and its values (the query plan test reads
/// it through `reports::plans`, Code Standards B18).
pub(crate) fn page_query(
    typed: &Typed,
    condition: Option<&'static str>,
    limit: i64,
    offset: i64,
) -> (String, Vec<Value>) {
    let c = conds(typed, condition);
    // The order of the register, and its exact reverse for "newest first": the line's own posting date,
    // then the بکری number, then the lot and the line number. So the first row of the newest-first page is
    // the LAST line of the newest بکری of the newest day, and a later sale counts on its own date. The
    // line number, not the row id, is the last key, so an edit (which writes its lines again with new
    // ids, in the same order after the lot's last line) does not move a line over the others of its
    // posting. Open: an edit of a first posting after a later sale of the same lot on the same day puts
    // the first posting's lines after the later sale (a known edge, plan 59).
    // PRD FR-7.90 (D7, plan 59 F1, Q2436, S-B1); one SQL page, never the whole register.
    let order = if matches!(typed.line_sort, Some(LineSort::Newest)) {
        "je.entry_date DESC, b.number DESC, s.lot_id DESC, s.line_no DESC, s.id DESC"
    } else {
        "je.entry_date, b.number, s.lot_id, s.line_no, s.id"
    };
    let sql = format!(
        "SELECT je.entry_date, je.posting_type, b.id, b.number,
           s.buyer_account_id, s.chukta_person_key, s.chukta_name_ur, s.chukta_name_en,
           ci.name_ur, ci.name_en, i.name_ur, i.name_en,
           s.qty, s.buyer_rate_paisa, s.buyer_value_paisa, s.buyer_total_paisa, s.grade,
           s.seller_rate_paisa, pk.name_ur, pk.name_en, s.paid_paisa, l.lot_counter
         FROM journal_entries je
         JOIN bikri_sales s ON s.journal_entry_id = je.id
         JOIN bikri_lots l ON l.id = s.lot_id
         JOIN bikri_entries b ON b.id = s.bikri_id
         JOIN items i ON i.id = l.item_id
         JOIN packings pk ON pk.id = l.packing_id
         JOIN accounts ba ON ba.id = s.buyer_account_id
         LEFT JOIN cities ci ON ci.id = ba.city_id
         WHERE {}
         ORDER BY {order}
         LIMIT ? OFFSET ?",
        c.sql()
    );
    (sql, c.page_args(limit, offset))
}

/// The count and the totals of the whole filter, from one query.
pub(crate) fn totals_query(typed: &Typed, condition: Option<&'static str>) -> (String, Vec<Value>) {
    let c = conds(typed, condition);
    let sql = format!(
        "SELECT COUNT(*), COALESCE(SUM(s.qty), 0), COALESCE(SUM(s.buyer_value_paisa), 0),
                COALESCE(SUM(s.buyer_total_paisa), 0), COALESCE(MAX(s.grade IS NOT NULL), 0),
                COALESCE(MAX(s.seller_rate_paisa <> s.buyer_rate_paisa), 0),
                COUNT(DISTINCT l.packing_id) > 1, COALESCE(SUM(s.paid_paisa), 0)
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

/// What was paid on the spot and what is left of a buyer total (PRD FR-12.44), from the stored
/// columns: `left = total - paid` by one checked subtraction, shared with R5.
fn paid_and_left(paid: i64, total: i64) -> Result<(Paisa, Paisa), ReportsError> {
    let (paid, total) = (paisa(paid)?, paisa(total)?);
    Ok((paid, left_of(total, paid)?))
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
    /// The counter of the lot's name (PRD FR-12.45).
    lot_counter: i64,
    qty: i64,
    rate: i64,
    value: i64,
    total: i64,
    /// The stored quality of the line, as text (PRD FR-12.42).
    grade: Option<String>,
    /// The seller's rate of the line (ریٹ بیوپاری), which is the buyer's `rate` unless the line has a
    /// rate difference (PRD FR-12.3).
    seller_rate: i64,
    packing_ur: String,
    packing_en: String,
    /// Money paid on the spot (PRD FR-12.44).
    paid: i64,
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
    let condition = kind.condition();
    let (totals_sql, totals_args) = totals_query(typed, condition);
    // `with_grade` is true when some line of the whole filter has a quality (PRD FR-12.42): then
    // every page of the register has the quality column after the item, so the columns do not
    // change from page to page. With no quality anywhere the register is exactly as it was.
    // `with_rate_diff` is true when some line of the whole filter was sold at a seller rate that is not
    // the buyer's (PRD FR-12.3): then every page has the seller's rate after the rate, so the last rate
    // of the sellers (the auction rate) can be read from the row. With no such line the register is
    // exactly as it was and the rate is both.
    // `with_packing` is true when the lines of the whole filter were sold in more than one packing: then
    // every page names the packing after the item, because the rate of an item in a peti is not the rate
    // of the same item in a crate (PRD FR-7.53, which keeps the last rate per item and packing).
    let (count, qty, value, total, with_grade, with_rate_diff, with_packing, paid_total): (
        i64,
        i64,
        i64,
        i64,
        bool,
        bool,
        bool,
        i64,
    ) = conn.query_row(&totals_sql, params_from_iter(totals_args.iter()), |r| {
        Ok((
            r.get(0)?,
            r.get(1)?,
            r.get(2)?,
            r.get(3)?,
            r.get(4)?,
            r.get(5)?,
            r.get(6)?,
            r.get(7)?,
        ))
    })?;

    let (page_sql, page_args) = page_query(typed, condition, spec.sql_limit(), spec.offset);
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
                grade: r.get(16)?,
                seller_rate: r.get(17)?,
                packing_ur: r.get(18)?,
                packing_en: r.get(19)?,
                paid: r.get(20)?,
                lot_counter: r.get(21)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let with_paid = kind != Kind::All;
    let buyer_ids: Vec<i64> = lines.iter().map(|l| l.buyer_id).collect();
    let parties = load_parties(conn, &buyer_ids)?;
    let mut rows = Vec::with_capacity(lines.len());
    for line in lines {
        let base = party_of_line(&parties, line.buyer_id, None)?;
        let party = chukta_party(
            base,
            line.person_key.clone(),
            line.person_ur.as_deref(),
            line.person_en.as_deref(),
        );
        let grade_cell = match grade_of(line.grade.as_deref())? {
            Some(grade) => Cell::Label {
                id: ids::grade_id(grade).to_owned(),
            },
            None => Cell::Empty,
        };
        let mut cells = vec![
            Cell::Date {
                iso: line.entry_date.clone(),
            },
            Cell::Doc {
                type_id: ids::doc_id(&line.posting_type),
                number: Some(line.bikri_number),
            },
            Cell::Party { party },
            names_cell(line.city_ur, line.city_en),
            // The item with the lot's counter: the lot the line sold from (PRD FR-12.45).
            Cell::Names {
                names: lot_names(&line.item_ur, &line.item_en, line.lot_counter),
            },
        ];
        if with_packing {
            cells.push(names_cell(Some(line.packing_ur), Some(line.packing_en)));
        }
        if with_grade {
            cells.push(grade_cell);
        }
        cells.extend([
            Cell::Number { value: line.qty },
            Cell::Money {
                paisa: paisa(line.rate)?,
            },
        ]);
        if with_rate_diff {
            cells.push(Cell::Money {
                paisa: paisa(line.seller_rate)?,
            });
        }
        cells.extend([
            Cell::Money {
                paisa: paisa(line.value)?,
            },
            Cell::Money {
                paisa: paisa(line.total)?,
            },
        ]);
        // R3 and R4: what was paid on the spot and what is left on the account (PRD FR-12.44).
        if with_paid {
            let (paid, left) = paid_and_left(line.paid, line.total)?;
            cells.extend([Cell::Money { paisa: paid }, Cell::Money { paisa: left }]);
        }
        rows.push(Row {
            cells,
            source: Some(RowSource {
                kind: SourceKind::Bikri,
                id: line.bikri_id,
            }),
            kind: RowKind::Normal,
        });
    }

    let mut columns = vec![
        col("date", ids::COL_DATE, ColumnKind::Date),
        col("doc", ids::COL_DOC, ColumnKind::Doc),
        col("buyer", ids::COL_BUYER, ColumnKind::Party),
        col("city", ids::COL_CITY, ColumnKind::Names),
        col("item", ids::COL_ITEM, ColumnKind::Names),
    ];
    let mut totals_cells = vec![Cell::Empty; 5];
    if with_packing {
        columns.push(col("packing", ids::COL_PACKING, ColumnKind::Names));
        totals_cells.push(Cell::Empty);
    }
    if with_grade {
        columns.push(col("grade", ids::COL_GRADE, ColumnKind::Label));
        totals_cells.push(Cell::Empty);
    }
    columns.extend([
        col("qty", ids::COL_QTY, ColumnKind::Number),
        // With the seller's rate beside it, `rate` is told apart as the buyer's.
        col(
            "rate",
            if with_rate_diff {
                ids::COL_BUYER_RATE
            } else {
                ids::COL_RATE
            },
            ColumnKind::Money,
        ),
    ]);
    if with_rate_diff {
        columns.push(col("seller_rate", ids::COL_SELLER_RATE, ColumnKind::Money));
    }
    columns.extend([
        col("value", ids::COL_VALUE, ColumnKind::Money),
        col("buyer_total", ids::COL_BUYER_TOTAL, ColumnKind::Money),
    ]);
    totals_cells.extend([Cell::Number { value: qty }, Cell::Empty]);
    if with_rate_diff {
        totals_cells.push(Cell::Empty);
    }
    totals_cells.extend([
        Cell::Money {
            paisa: paisa(value)?,
        },
        Cell::Money {
            paisa: paisa(total)?,
        },
    ]);
    if with_paid {
        columns.extend([
            col("paid", ids::COL_PAID_ON_SPOT, ColumnKind::Money),
            col("left", ids::COL_LEFT_ON_ACCOUNT, ColumnKind::Money),
        ]);
        // The sums of the whole filter: the stored columns added in SQL, the difference made once,
        // here, by a checked subtraction (Code Standards M7).
        let (paid_sum, left_sum) = paid_and_left(paid_total, total)?;
        totals_cells.extend([
            Cell::Money { paisa: paid_sum },
            Cell::Money { paisa: left_sum },
        ]);
    }

    let id = kind.id();
    Ok(ReportPage {
        group_above: None,
        tables: Vec::new(),
        id,
        title_id: ids::title_id(id.code()),
        filters: typed.resolved.clone(),
        columns,
        rows,
        offset: spec.offset,
        limit: spec.reported_limit(),
        total_count: count,
        totals_label_id: ids::TOTAL.to_owned(),
        totals: totals_cells,
        summary: Vec::new(),
    })
}
