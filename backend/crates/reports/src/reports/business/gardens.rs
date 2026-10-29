//! R36 باغات رپورٹ (the gardens report) (PRD 19.2 R36, 12.11, FR-12.34 to FR-12.38, plan 17c).
//!
//! The rows are the gardens, picked by the day they were made, their main fruit, a partner who is
//! in them, or one garden. Every figure is as of today and over the whole life of the garden,
//! whatever the dates chosen: the dates choose the gardens, not the day a payment or a sale was
//! made. The figures are those of the garden screen, worked out once by
//! [`mandi_db::gardens::Money::of`] and read from the same stored rows ([`crate::gardens`]).
//!
//! **By garden** (the default), one row for each garden: its status, the rent, the installments
//! and the running costs, what was paid from the shop's cash and from partners' pockets, the sales
//! money (in our shop, in other shops, and any other money in, fruit taken from its lots), the
//! commission on its goods (seller commission, buyer commission and لاگا), the money in, the
//! running balance (money in less everything paid; after the closing it is the result) and the
//! rent still to pay. The three kinds of money in add up to the money in, and what was paid from
//! the shop and from pockets add up to what was paid.
//!
//! **By partner**, one row for each partner of each garden (with a partner picked, only his rows): his %, what he paid from the shop and
//! from his pocket, what he got back (his stored part of the closing) and his result, back less
//! paid. The results are never a new split of the garden's result; they add up to the running
//! balance once the garden is closed. While it is open nothing has come back, so a result is
//! minus what the partner paid.
//!
//! The totals row adds up the rows over the whole filter; it never adds up a page (Code Standards
//! B31).
//!
//! Example (E2E-40, after the closing on 13-10-26): شاہ پور باغ, rent 50,000, installments 50,000,
//! costs 5,000, paid from the shop 33,000 and from pockets 22,000, our shop 56,400, other shops
//! 10,000, commission 8,000, money in 66,400, balance 11,400, rent due 0. By partner: الف 60%: 33,000
//! / 0 / back 39,840 / result 6,840; ب 40%: 0 / 22,000 / back 26,560 / result 4,560.

use mandi_core::money::Paisa;
use mandi_db::rusqlite::Connection;

use super::super::PageSpec;
use crate::error::ReportsError;
use crate::gardens::{Garden, Pick, load};
use crate::reports::common::{add, col, count_of, names_cell};
use crate::reports::filters::Typed;
use crate::reports::ids;
use crate::reports::model::{
    Cell, Column, ColumnKind, GardenView, ReportId, ReportPage, Row, RowKind, RowSource, SourceKind,
};

fn money(paisa: Paisa) -> Cell {
    Cell::Money { paisa }
}

fn source(garden: &Garden) -> Option<RowSource> {
    Some(RowSource {
        kind: SourceKind::Venture,
        id: garden.rec.id,
    })
}

fn garden_names(garden: &Garden) -> Cell {
    names_cell(
        Some(garden.rec.name_ur.clone()),
        Some(garden.rec.name_en.clone()),
    )
}

fn garden_columns() -> Vec<Column> {
    vec![
        col("garden", ids::COL_GARDEN, ColumnKind::Names),
        col("date", ids::COL_DATE, ColumnKind::Date),
        col("status", ids::COL_STATUS, ColumnKind::Label),
        col("rent", ids::COL_RENT, ColumnKind::Money),
        col("installments", ids::COL_INSTALLMENTS, ColumnKind::Money),
        col("costs", ids::COL_GARDEN_COSTS, ColumnKind::Money),
        col("paid_shop", ids::COL_MONEY_IN_SHOP, ColumnKind::Money),
        col("paid_pocket", ids::COL_MONEY_IN_POCKET, ColumnKind::Money),
        col("sales_our_shop", ids::COL_SALES_OUR_SHOP, ColumnKind::Money),
        col(
            "sales_other_shops",
            ids::COL_SALES_OTHER_SHOPS,
            ColumnKind::Money,
        ),
        col("other_in", ids::COL_OTHER_IN, ColumnKind::Money),
        col("commission", ids::COL_TOTAL_COMM, ColumnKind::Money),
        col("money_in", ids::COL_MONEY_IN, ColumnKind::Money),
        col("balance", ids::COL_RUNNING_BALANCE, ColumnKind::Money),
        col("rent_due", ids::COL_RENT_DUE, ColumnKind::Money),
    ]
}

/// The money cells of a garden row, in column order from `rent` on.
fn figures(garden: &Garden) -> Result<[Paisa; 12], ReportsError> {
    let m = &garden.money;
    Ok([
        Paisa::try_from_paisa(garden.rec.rent_paisa)?,
        m.installments,
        m.costs,
        m.paid_shop,
        m.paid_pocket,
        garden.our_shop,
        m.other_shop,
        garden.other_in,
        garden.commission.total(true, true)?,
        m.money_in,
        m.balance,
        m.rent_due,
    ])
}

fn by_garden(gardens: &[Garden], spec: PageSpec) -> Result<ReportPage, ReportsError> {
    let mut rows = Vec::with_capacity(gardens.len());
    let mut totals = [Paisa::ZERO; 12];
    for garden in gardens {
        let values = figures(garden)?;
        for (sum, value) in totals.iter_mut().zip(values) {
            add(sum, value)?;
        }
        let status = if garden.rec.status == "closed" {
            ids::GARDEN_CLOSED
        } else {
            ids::GARDEN_OPEN
        };
        let mut cells = vec![
            garden_names(garden),
            Cell::Date {
                iso: garden.rec.entry_date.clone(),
            },
            Cell::Label {
                id: status.to_owned(),
            },
        ];
        cells.extend(values.into_iter().map(money));
        rows.push(Row {
            cells,
            source: source(garden),
            kind: RowKind::Normal,
        });
    }
    let mut total_cells = vec![Cell::Empty; 3];
    total_cells.extend(totals.into_iter().map(money));
    Ok(page(garden_columns(), rows, total_cells, spec))
}

fn partner_columns() -> Vec<Column> {
    vec![
        col("garden", ids::COL_GARDEN, ColumnKind::Names),
        col("partner", ids::COL_PARTNER, ColumnKind::Names),
        col("share", ids::COL_SHARE, ColumnKind::Percent),
        col("paid_shop", ids::COL_MONEY_IN_SHOP, ColumnKind::Money),
        col("paid_pocket", ids::COL_MONEY_IN_POCKET, ColumnKind::Money),
        col("money_back", ids::COL_MONEY_BACK, ColumnKind::Money),
        col("result", ids::COL_PART_OF_RESULT, ColumnKind::Money),
    ]
}

fn by_partner(
    gardens: &[Garden],
    only: Option<i64>,
    spec: PageSpec,
) -> Result<ReportPage, ReportsError> {
    let mut rows = Vec::new();
    let (mut shop, mut pocket, mut back, mut net) =
        (Paisa::ZERO, Paisa::ZERO, Paisa::ZERO, Paisa::ZERO);
    for garden in gardens {
        for p in garden
            .partners
            .iter()
            .filter(|p| only.is_none_or(|id| id == p.partner_id))
        {
            add(&mut shop, p.paid_shop)?;
            add(&mut pocket, p.paid_pocket)?;
            add(&mut back, p.back)?;
            add(&mut net, p.net)?;
            rows.push(Row {
                cells: vec![
                    garden_names(garden),
                    names_cell(Some(p.name_ur.clone()), Some(p.name_en.clone())),
                    Cell::Percent {
                        hundredths: p.share_bps,
                    },
                    money(p.paid_shop),
                    money(p.paid_pocket),
                    money(p.back),
                    money(p.net),
                ],
                source: source(garden),
                kind: RowKind::Normal,
            });
        }
    }
    let total_cells = vec![
        Cell::Empty,
        Cell::Empty,
        Cell::Empty,
        money(shop),
        money(pocket),
        money(back),
        money(net),
    ];
    Ok(page(partner_columns(), rows, total_cells, spec))
}

fn page(columns: Vec<Column>, rows: Vec<Row>, totals: Vec<Cell>, spec: PageSpec) -> ReportPage {
    ReportPage {
        group_above: None,
        id: ReportId::R36,
        title_id: ids::title_id("R36"),
        filters: Default::default(),
        columns,
        total_count: count_of(rows.len()),
        rows: spec.slice(&rows),
        offset: spec.offset,
        limit: spec.reported_limit(),
        totals_label_id: ids::TOTAL.to_owned(),
        totals,
        summary: Vec::new(),
    }
}

/// Builds R36.
///
/// # Errors
/// A database error; `ReportsError::Money`; `ReportsError::Data` if the figures of a garden do not
/// add up (the parts of its payments, or its money in from its sources).
pub(crate) fn run(
    conn: &Connection,
    typed: &Typed,
    spec: PageSpec,
) -> Result<ReportPage, ReportsError> {
    let pick = Pick {
        from: typed.from.clone(),
        to: typed.to.clone(),
        item_id: typed.item_id,
        partner_id: typed.partner_id,
        garden_id: typed.garden_id,
        status: None,
    };
    let gardens = load(conn, &pick)?;
    let mut page = match typed.garden_view {
        GardenView::Garden => by_garden(&gardens, spec)?,
        GardenView::Partner => by_partner(&gardens, typed.partner_id, spec)?,
    };
    page.filters = typed.resolved.clone();
    Ok(page)
}
