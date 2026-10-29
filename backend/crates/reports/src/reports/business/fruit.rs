//! R37 پھل رجسٹر (the fruit register) (PRD 19.2 R37, 16.7, FR-16.10 to FR-16.12, FR-19.10,
//! FR-19.11, C16, C27; plan d5-late, after D4R 17d).
//!
//! One row for each saved fruit entry (a cancelled entry is not here): its day and number, for
//! whom it was taken (a partner, or the shop's guests), who is credited (the seller of the lot, the
//! garden, or the other agent), the item and packing, the quantity (نگ), the rate and the value,
//! and where it came from (our shop or another shop). The filters are the day or a range of days,
//! a partner, only the guests, an item, and a garden (the fruit taken from that garden's lots).
//!
//! Every figure is a value stored on the entry when it was saved (Code Standards B20): the
//! quantity, the rate and the value. An edit updates the entry, so a row shows the entry as it
//! stands now and sits on the day it stands on. The totals row is `SUM()` of the stored `qty` and
//! `value_paisa` columns over the whole filter, never a sum of a page (Code Standards B31); the
//! quantity total is a plain count of نگ over different items and packings, as in R4 and R10.
//!
//! Example (E2E-42, 10-10-26): two rows. الف, Banana, 2 @ 580 = 1,160, our shop; the guests,
//! Apple, 3 @ 1,000 = 3,000, another shop. Totals 5 and 4,160.

use mandi_db::rusqlite::{Connection, params_from_iter};

use super::super::PageSpec;
use crate::error::ReportsError;
use mandi_core::money::Paisa;

use crate::print::format::kilos_text;
use crate::reports::common::{
    Conds, col, load_parties, names_cell, paisa, party_of_line, pukhta_party,
};
use crate::reports::filters::Typed;
use crate::reports::ids;
use crate::reports::model::{
    Cell, Column, ColumnKind, Names, ReportId, ReportPage, Row, RowKind, RowSource, SourceKind,
};

/// Which lots the fruit came from, for the analytics goods filter (PRD 18.1).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum LotKind {
    /// Every source, including another agent's shop.
    #[default]
    Any,
    /// Our shop's lots whose seller is not a garden: consigned goods.
    Consigned,
    /// Our shop's lots whose seller is a garden.
    Garden,
}

/// What narrows the saved entries: the days, the partner or only the guests, the item, one
/// garden, and the kind of lot.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct Scope<'a> {
    pub(crate) from: Option<&'a str>,
    pub(crate) to: Option<&'a str>,
    pub(crate) partner_id: Option<i64>,
    pub(crate) guests_only: bool,
    pub(crate) item_id: Option<i64>,
    /// A garden (the id of the venture): the fruit taken from a lot whose seller is its account.
    pub(crate) garden_id: Option<i64>,
    pub(crate) lots: LotKind,
}

/// The WHERE of the register: saved entries inside the scope. `ft` is `fruit_takes`. Every
/// fragment is a constant; values are bound (Code Standards B14).
pub(crate) fn conds(scope: &Scope<'_>) -> Conds {
    let mut c = Conds::new();
    c.always("ft.status = 'saved'");
    c.text("ft.entry_date >= ?", scope.from);
    c.text("ft.entry_date <= ?", scope.to);
    c.id("ft.partner_id = ?", scope.partner_id);
    if scope.guests_only {
        c.always("ft.partner_id IS NULL");
    }
    c.id("ft.item_id = ?", scope.item_id);
    // The fruit of a garden is the fruit taken from a lot whose seller is that garden's account.
    c.id(
        "ft.lot_id IN (SELECT l.id FROM bikri_lots l
                       JOIN ventures v ON v.account_id = l.seller_account_id AND v.kind = 'garden'
                       WHERE v.id = ?)",
        scope.garden_id,
    );
    match scope.lots {
        LotKind::Any => {}
        // Consigned goods are our shop's lots of sellers who are not gardens; another agent's
        // shop has no lot of ours.
        LotKind::Consigned => {
            c.always(
                "ft.lot_id IN (SELECT l.id FROM bikri_lots l WHERE l.purchase_id IS NULL
                     AND l.seller_account_id NOT IN
                         (SELECT account_id FROM ventures WHERE kind = 'garden' AND account_id IS NOT NULL))",
            );
        }
        LotKind::Garden => {
            c.always(
                "ft.lot_id IN (SELECT l.id FROM bikri_lots l WHERE l.seller_account_id IN
                         (SELECT account_id FROM ventures WHERE kind = 'garden' AND account_id IS NOT NULL))",
            );
        }
    }
    c
}

/// One entry as the register reads it.
struct Raw {
    id: i64,
    number: i64,
    date: String,
    partner: Option<Names>,
    source: String,
    credit_account: i64,
    person_key: Option<String>,
    pukhta: Option<(String, String)>,
    item: Names,
    packing: Names,
    weight_grams: Option<i64>,
    qty: i64,
    rate: i64,
    value: i64,
}

/// The entries of one page under a clause, and the sums over the whole clause.
pub(crate) struct Entries {
    pub(crate) rows: Vec<Row>,
    pub(crate) total_count: i64,
    /// Σ of the quantities (a plain count of نگ).
    pub(crate) qty: i64,
    /// Σ of the stored values.
    pub(crate) value: Paisa,
}

/// The columns of the register, which are also those of the lines behind view K.
pub(crate) fn columns() -> Vec<Column> {
    vec![
        col("date", ids::COL_DATE, ColumnKind::Date),
        col("doc", ids::COL_DOC, ColumnKind::Doc),
        col("for", ids::COL_FRUIT_FOR, ColumnKind::Names),
        col("seller", ids::COL_SELLER, ColumnKind::Party),
        col("item", ids::COL_ITEM, ColumnKind::Names),
        col("packing", ids::COL_PACKING, ColumnKind::Names),
        // An item sold by weight: the rate is per kg and the value is the stored one, not
        // qty × rate (C1), so the weight is shown (money agent, round 12).
        col("weight", ids::COL_FRUIT_WEIGHT, ColumnKind::Text),
        col("qty", ids::COL_QTY, ColumnKind::Number),
        col("rate", ids::COL_RATE, ColumnKind::Money),
        col("value", ids::COL_VALUE, ColumnKind::Money),
        col("source", ids::COL_FRUIT_SOURCE, ColumnKind::Label),
    ]
}

/// The totals row under [`columns`].
pub(crate) fn totals_cells(entries: &Entries) -> Vec<Cell> {
    vec![
        Cell::Empty,
        Cell::Empty,
        Cell::Empty,
        Cell::Empty,
        Cell::Empty,
        Cell::Empty,
        Cell::Empty,
        Cell::Number { value: entries.qty },
        Cell::Empty,
        Cell::Money {
            paisa: entries.value,
        },
        Cell::Empty,
    ]
}

/// Reads one page of entries under a clause, with the count and the sums over all of them, each
/// from one indexed query on INTEGER columns (Code Standards B31).
///
/// # Errors
/// A database error; `ReportsError::Money` if a stored amount is not whole rupees.
pub(crate) fn read(conn: &Connection, c: &Conds, spec: PageSpec) -> Result<Entries, ReportsError> {
    let (total_count, qty_total, value_total): (i64, i64, i64) = conn.query_row(
        &format!(
            "SELECT COUNT(*), COALESCE(SUM(ft.qty), 0), COALESCE(SUM(ft.value_paisa), 0)
             FROM fruit_takes ft WHERE {}",
            c.sql()
        ),
        params_from_iter(c.args().iter()),
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    )?;

    let sql = format!(
        "SELECT ft.id, ft.number, ft.entry_date, p.name_ur, p.name_en, ft.source,
                ft.credit_account_id, ft.pukhta_person_key, l.pukhta_name_ur, l.pukhta_name_en,
                i.name_ur, i.name_en, pk.name_ur, pk.name_en, ft.weight_grams, ft.qty, ft.rate_paisa,
                ft.value_paisa
         FROM fruit_takes ft
         JOIN items i ON i.id = ft.item_id
         JOIN packings pk ON pk.id = ft.packing_id
         LEFT JOIN partners p ON p.id = ft.partner_id
         LEFT JOIN bikri_lots l ON l.id = ft.lot_id
         WHERE {} ORDER BY ft.entry_date, ft.number LIMIT ? OFFSET ?",
        c.sql()
    );
    let args = c.page_args(spec.sql_limit(), spec.offset);
    let raws = conn
        .prepare(&sql)?
        .query_map(params_from_iter(args.iter()), |r| {
            let partner_ur: Option<String> = r.get(3)?;
            let partner_en: Option<String> = r.get(4)?;
            let pukhta_ur: Option<String> = r.get(8)?;
            let pukhta_en: Option<String> = r.get(9)?;
            Ok(Raw {
                id: r.get(0)?,
                number: r.get(1)?,
                date: r.get(2)?,
                partner: partner_ur.zip(partner_en).map(|(ur, en)| Names { ur, en }),
                source: r.get(5)?,
                credit_account: r.get(6)?,
                person_key: r.get(7)?,
                pukhta: pukhta_ur.zip(pukhta_en),
                item: Names {
                    ur: r.get(10)?,
                    en: r.get(11)?,
                },
                packing: Names {
                    ur: r.get(12)?,
                    en: r.get(13)?,
                },
                weight_grams: r.get(14)?,
                qty: r.get(15)?,
                rate: r.get(16)?,
                value: r.get(17)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let accounts: Vec<i64> = raws.iter().map(|r| r.credit_account).collect();
    let parties = load_parties(conn, &accounts)?;
    let mut rows = Vec::with_capacity(raws.len());
    for raw in raws {
        let mut credited = party_of_line(&parties, raw.credit_account, None)?;
        if let Some(key) = raw.person_key {
            // A one-time seller of the lot keeps his own names (PRD FR-13.5).
            let (ur, en) = raw.pukhta.clone().unwrap_or_default();
            credited = pukhta_party(credited, Some(key), Some(ur.as_str()), Some(en.as_str()));
        }
        let for_whom = match raw.partner {
            Some(names) => Cell::Names { names },
            None => Cell::Label {
                id: ids::FRUIT_FOR_GUESTS.to_owned(),
            },
        };
        let source = if raw.source == "shop" {
            ids::FRUIT_SOURCE_OWN
        } else {
            ids::FRUIT_SOURCE_OTHER
        };
        rows.push(Row {
            cells: vec![
                Cell::Date { iso: raw.date },
                Cell::Doc {
                    type_id: ids::doc_id("fruit_taken"),
                    number: Some(raw.number),
                },
                for_whom,
                Cell::Party { party: credited },
                names_cell(Some(raw.item.ur), Some(raw.item.en)),
                names_cell(Some(raw.packing.ur), Some(raw.packing.en)),
                raw.weight_grams.map_or(Cell::Empty, |grams| Cell::Text {
                    text: kilos_text(grams),
                }),
                Cell::Number { value: raw.qty },
                Cell::Money {
                    paisa: paisa(raw.rate)?,
                },
                Cell::Money {
                    paisa: paisa(raw.value)?,
                },
                Cell::Label {
                    id: source.to_owned(),
                },
            ],
            source: Some(RowSource {
                kind: SourceKind::Fruit,
                id: raw.id,
            }),
            kind: RowKind::Normal,
        });
    }
    Ok(Entries {
        rows,
        total_count,
        qty: qty_total,
        value: paisa(value_total)?,
    })
}

/// Builds R37.
///
/// # Errors
/// A database error; `ReportsError::Money` if a stored amount is not whole rupees.
pub(crate) fn run(
    conn: &Connection,
    typed: &Typed,
    spec: PageSpec,
) -> Result<ReportPage, ReportsError> {
    let c = conds(&Scope {
        from: typed.from.as_deref(),
        to: typed.to.as_deref(),
        partner_id: typed.partner_id,
        guests_only: typed.guests_only,
        item_id: typed.item_id,
        garden_id: typed.garden_id,
        lots: LotKind::Any,
    });
    let entries = read(conn, &c, spec)?;
    Ok(ReportPage {
        group_above: None,
        id: ReportId::R37,
        title_id: ids::title_id("R37"),
        filters: typed.resolved.clone(),
        columns: columns(),
        totals: totals_cells(&entries),
        rows: entries.rows,
        offset: spec.offset,
        limit: spec.reported_limit(),
        total_count: entries.total_count,
        totals_label_id: ids::TOTAL.to_owned(),
        summary: Vec::new(),
    })
}
