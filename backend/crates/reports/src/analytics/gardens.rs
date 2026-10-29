//! View I, gardens, باغات (PRD 18.2, FR-12.34 to FR-12.38, FR-12.37, E2E-40, E2E-40b).
//!
//! One row for each garden: what was paid (from the shop's cash and from partners' pockets), the
//! sales money (in our shop, in other shops, and any other money in), the commission on its goods
//! and its result: the running balance while it is open (money in less everything paid), the
//! result once it is closed (the closing and its undo are not money in, so it is the same number).
//! Then, for each partner of the gardens shown, his % and his result: what he got back less what
//! he paid, from the stored parts (never a new split of the garden's result), so once the garden is
//! closed the results of its partners add up to its result.
//!
//! The figures are those of report R36 and of the garden screen ([`crate::gardens`]): the dates
//! pick the gardens by the day they were made, the item by their main fruit, and the figures are
//! over the whole life of the garden as of today. The commission follows the side filter, as in
//! view J; it is not narrowed by the period.
//!
//! Example (E2E-40, closed on 13-10-26): paid 33,000 / 22,000, our shop 56,400, other shops
//! 10,000, commission 8,000, result 11,400; الف 60% +6,840, ب 40% +4,560.

use std::collections::BTreeMap;

use mandi_core::money::Paisa;
use mandi_db::rusqlite::Connection;

use super::filters::Typed;
use super::ids;
use super::model::{
    AnalyticsColumn, AnalyticsGoods, AnalyticsHeading, AnalyticsKey, AnalyticsSide,
};
use super::table::{Table, col, count_of, row};
use crate::error::ReportsError;
use crate::gardens::{Garden, Pick, load};
use crate::reports::common::add;
use crate::reports::{Cell, ColumnKind, Names, PageSpec};

fn money(paisa: Paisa) -> Cell {
    Cell::Money { paisa }
}

/// A partner's two columns: his share and his result.
fn partner_columns(partner_id: i64, names: &Names) -> [AnalyticsColumn; 2] {
    let heading = AnalyticsHeading::Partner {
        names: names.clone(),
    };
    let make = |key: String, title: &str, kind| AnalyticsColumn {
        key,
        title_id: title.to_owned(),
        kind,
        heading: Some(heading.clone()),
        drill: None,
    };
    [
        make(
            format!("share_{partner_id}"),
            ids::COL_PARTNER_SHARE,
            ColumnKind::Percent,
        ),
        make(
            format!("result_{partner_id}"),
            ids::COL_PARTNER_RESULT,
            ColumnKind::Money,
        ),
    ]
}

fn fixed_columns() -> Vec<AnalyticsColumn> {
    vec![
        col("group", ids::COL_GARDEN, ColumnKind::Names),
        col("status", ids::COL_GARDEN_STATUS, ColumnKind::Label),
        col("paid_shop", ids::COL_PAID_SHOP, ColumnKind::Money),
        col("paid_pocket", ids::COL_PAID_POCKET, ColumnKind::Money),
        col("sales_our_shop", ids::COL_SALES_OUR_SHOP, ColumnKind::Money),
        col(
            "sales_other_shops",
            ids::COL_SALES_OTHER_SHOPS,
            ColumnKind::Money,
        ),
        col("other_in", ids::COL_OTHER_IN, ColumnKind::Money),
        col("commission", ids::COL_TOTAL_COMM, ColumnKind::Money),
        col("result", ids::COL_GARDEN_RESULT, ColumnKind::Money),
    ]
}

/// The seven money figures of a garden row, in column order.
fn figures(garden: &Garden, side: AnalyticsSide) -> Result<[Paisa; 7], ReportsError> {
    let (seller, buyer) = match side {
        AnalyticsSide::Both => (true, true),
        AnalyticsSide::Seller => (true, false),
        AnalyticsSide::Buyer => (false, true),
    };
    Ok([
        garden.money.paid_shop,
        garden.money.paid_pocket,
        garden.our_shop,
        garden.money.other_shop,
        garden.other_in,
        garden.commission.total(seller, buyer)?,
        garden.money.balance,
    ])
}

/// Builds view I.
///
/// # Errors
/// A database error; `ReportsError::Money`; `ReportsError::Data` if the figures of a garden do not
/// add up.
pub(crate) fn run(conn: &Connection, typed: &Typed, spec: PageSpec) -> Result<Table, ReportsError> {
    // Garden goods only: consigned goods and trip goods have no gardens.
    let gardens = if matches!(typed.goods, AnalyticsGoods::All | AnalyticsGoods::Garden) {
        load(
            conn,
            &Pick {
                from: typed.from.clone(),
                to: typed.to.clone(),
                item_id: typed.item_id,
                partner_id: typed.partner_id,
                garden_id: typed.garden_id,
                status: None,
            },
        )?
    } else {
        Vec::new()
    };

    // The partners of the gardens shown, in the order they were created.
    let mut partners: BTreeMap<i64, Names> = BTreeMap::new();
    for garden in &gardens {
        for p in &garden.partners {
            partners.entry(p.partner_id).or_insert_with(|| Names {
                ur: p.name_ur.clone(),
                en: p.name_en.clone(),
            });
        }
    }
    let mut columns = fixed_columns();
    for (id, names) in &partners {
        columns.extend(partner_columns(*id, names));
    }

    let mut totals = [Paisa::ZERO; 7];
    let mut partner_totals: BTreeMap<i64, Paisa> =
        partners.keys().map(|id| (*id, Paisa::ZERO)).collect();
    let mut rows = Vec::with_capacity(gardens.len());
    for garden in &gardens {
        let values = figures(garden, typed.side)?;
        for (sum, value) in totals.iter_mut().zip(values) {
            add(sum, value)?;
        }
        let status = if garden.rec.status == "closed" {
            crate::reports::ids::GARDEN_CLOSED
        } else {
            crate::reports::ids::GARDEN_OPEN
        };
        let mut cells = vec![
            Cell::Names {
                names: Names {
                    ur: garden.rec.name_ur.clone(),
                    en: garden.rec.name_en.clone(),
                },
            },
            Cell::Label {
                id: status.to_owned(),
            },
        ];
        cells.extend(values.into_iter().map(money));
        for id in partners.keys() {
            match garden.partners.iter().find(|p| p.partner_id == *id) {
                Some(p) => {
                    cells.push(Cell::Percent {
                        hundredths: p.share_bps,
                    });
                    cells.push(money(p.net));
                    if let Some(sum) = partner_totals.get_mut(id) {
                        add(sum, p.net)?;
                    }
                }
                None => {
                    cells.push(Cell::Empty);
                    cells.push(Cell::Empty);
                }
            }
        }
        rows.push(row(
            cells,
            // The garden's goods are the lots whose seller is its account.
            AnalyticsKey {
                seller_id: Some(garden.rec.account.id),
                ..AnalyticsKey::default()
            },
        ));
    }

    let mut total_cells = vec![Cell::Empty, Cell::Empty];
    total_cells.extend(totals.into_iter().map(money));
    for id in partners.keys() {
        total_cells.push(Cell::Empty);
        total_cells.push(money(partner_totals.get(id).copied().unwrap_or_default()));
    }
    Ok(Table {
        charts: Vec::new(),
        columns,
        total_count: count_of(rows.len()),
        rows: spec.slice(&rows),
        totals: total_cells,
    })
}
