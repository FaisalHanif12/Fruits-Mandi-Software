//! View J, trips, مارکیٹ سفر (PRD 18.2, FR-12.27, FR-12.33, C21, C23, C25; E2E-34, E2E-34b,
//! E2E-41).
//!
//! The rows are the trips (the default), the purchases, the items, or the days the goods were
//! bought (`trip_view`). For each row: what was bought (نگ and cost), what was sold (نگ, the
//! money the sales brought after commission, and the commission on its goods), the cost of what
//! was sold, what was written off (نگ and cost), what is left (نگ and cost), and the result.
//! Then, for each partner of the trips shown, his share (%) and, in the by-trip layout, his part.
//!
//! * **Which purchases.** The filters pick purchases: by the day bought (the period), the item,
//!   the trip, and the partner (only the trips he is in). The figures are those purchases as
//!   they stand today, the same as report R33: nothing is guessed for a day.
//! * **Figures.** The stored values of the purchases and their sales, as R33 reads them
//!   (`figures_of`): sales money is the partners' credits of the own lots (C25) and the money of
//!   the sales in other shops; the cost sold is the cost released less the cost written off
//!   (C21); the result is the sales money less the cost released; نگ sold are the stored
//!   quantities of the purchases, so sales in other shops count too.
//! * **Commission on its goods.** Seller commission, buyer commission and لاگا of the sale lines
//!   of its goods, from the sale facts with their sign (FR-18.6); the side filter narrows it, so
//!   `seller` gives the 540 of E2E-34 and `both` the 1,200 of the PRD note (540 + 540 + 120).
//!   It is not the seller-only commission column of R33.
//! * **Partner parts.** Only in the by-trip layout. A partner's part of a trip is the money that
//!   reached him less the money he put in, from the stored parts, exactly as R33 reads them:
//!   his parts of the seller credits of the own lots (C25) and of the sales in other shops, less
//!   his parts paid from the shop's cash and from his own pocket. It is never a new split of the
//!   result (a split would give other rupees than the books when goods are left or a part was
//!   rounded). So the parts of a trip plus its cost still left are its result. The partner
//!   money is always of the WHOLE trip, so in this layout the filters pick the trips (a trip is
//!   shown when one of its purchases is inside the period and the item), and every figure of the
//!   row is of the whole trip. In the other layouts only the shares (%) are shown.
//!
//! Example (E2E-34b, 11-10-26): trip 1, bought 20 for 10,400, sold 15 for 8,460 after 540
//! commission, cost sold 7,800, written off 5 for 2,600, left 0, result −1,940: parts −776 /
//! −582 / −582 at 40 / 30 / 30.

use std::collections::{BTreeMap, HashMap};

use mandi_core::money::Paisa;
use mandi_db::purchases::{ListFilter, PurchaseRec, SoldFigures, sold_figures};
use mandi_db::rusqlite::Connection;
use mandi_db::ventures::{PartnerShare, lot_credits_by_partner, partner_money, partners_of_many};

use super::facts::{self, conds};
use super::filters::Typed;
use super::ids;
use super::model::{
    AnalyticsColumn, AnalyticsGoods, AnalyticsHeading, AnalyticsKey, AnalyticsTripView,
};
use super::table::{Table, col, count_of, row};
use crate::error::ReportsError;
use crate::reports::business::trips::{Figures, figures_of};
use crate::reports::common::{IN_CHUNK, add, paisa};
use crate::reports::{Cell, ColumnKind, Names, PageSpec};

/// One purchase with its figures.
struct Line {
    purchase: PurchaseRec,
    figures: Figures,
}

/// The ordering key and the label of the row a purchase belongs to.
fn group_of(view: AnalyticsTripView, purchase: &PurchaseRec) -> ((String, i64, i64), Cell) {
    match view {
        AnalyticsTripView::Trip => (
            (String::new(), purchase.trip_number, 0),
            Cell::Names {
                names: Names {
                    ur: purchase.trip_name_ur.clone(),
                    en: purchase.trip_name_en.clone(),
                },
            },
        ),
        AnalyticsTripView::Purchase => (
            (
                purchase.entry_date.clone(),
                purchase.trip_number,
                purchase.number,
            ),
            Cell::Doc {
                type_id: crate::reports::ids::doc_id("purchase"),
                number: Some(purchase.number),
            },
        ),
        AnalyticsTripView::Item => (
            (purchase.item.code.clone(), purchase.item.id, 0),
            Cell::Names {
                names: Names {
                    ur: purchase.item.name_ur.clone(),
                    en: purchase.item.name_en.clone(),
                },
            },
        ),
        AnalyticsTripView::Day => (
            (purchase.entry_date.clone(), 0, 0),
            Cell::Date {
                iso: purchase.entry_date.clone(),
            },
        ),
    }
}

/// The first column: what a row stands for.
fn group_column(view: AnalyticsTripView) -> AnalyticsColumn {
    match view {
        AnalyticsTripView::Trip => col("group", ids::COL_TRIP, ColumnKind::Names),
        AnalyticsTripView::Purchase => col("group", ids::COL_PURCHASE, ColumnKind::Doc),
        AnalyticsTripView::Item => col("group", ids::COL_ITEM, ColumnKind::Names),
        AnalyticsTripView::Day => col("group", ids::COL_DATE, ColumnKind::Date),
    }
}

/// The eleven figure columns of R33, in its order.
fn figure_columns() -> Vec<AnalyticsColumn> {
    vec![
        col("bought_qty", ids::COL_BOUGHT_QTY, ColumnKind::Number),
        col("bought_cost", ids::COL_BOUGHT_COST, ColumnKind::Money),
        col("sold_qty", ids::COL_SOLD_QTY, ColumnKind::Number),
        col("sales_money", ids::COL_SALES_MONEY, ColumnKind::Money),
        col("commission", ids::COL_TOTAL_COMM, ColumnKind::Money),
        col("cost_sold", ids::COL_COST_SOLD, ColumnKind::Money),
        col("written_off_qty", ids::COL_WRITTEN_OFF, ColumnKind::Number),
        col(
            "written_off_cost",
            ids::COL_WRITTEN_OFF_COST,
            ColumnKind::Money,
        ),
        col("left_qty", ids::COL_LEFT_QTY, ColumnKind::Number),
        col("left_cost", ids::COL_LEFT_COST, ColumnKind::Money),
        col("result", ids::COL_RESULT, ColumnKind::Money),
    ]
}

/// A partner's two columns: his share and his part.
fn partner_columns(partner: &PartnerShare) -> [AnalyticsColumn; 2] {
    let heading = AnalyticsHeading::Partner {
        names: Names {
            ur: partner.name_ur.clone(),
            en: partner.name_en.clone(),
        },
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
            format!("share_{}", partner.partner_id),
            ids::COL_PARTNER_SHARE,
            ColumnKind::Percent,
        ),
        make(
            format!("part_{}", partner.partner_id),
            ids::COL_PARTNER_PART,
            ColumnKind::Money,
        ),
    ]
}

/// The commission on the goods of these purchases, from the sale facts under the run's filters,
/// summed with their sign (FR-18.6).
fn commissions(
    conn: &Connection,
    typed: &Typed,
    purchase_ids: &[i64],
) -> Result<HashMap<i64, Paisa>, ReportsError> {
    let mut out = HashMap::new();
    // The item picks the purchases; it must not narrow the commission again, because the by-trip
    // layout reads every purchase of the trip and its commission is of all of them (money agent
    // M1). The same settings are used to open the lines behind the row (lines.rs).
    let typed = &facts::for_trips(typed);
    for chunk in purchase_ids.chunks(IN_CHUNK) {
        let key = AnalyticsKey {
            purchase_ids: chunk.to_vec(),
            ..AnalyticsKey::default()
        };
        let c = conds(typed, facts::TRIPS_RANGE, &key);
        for group in facts::groups(conn, &c, &["f.purchase_id"])? {
            if let Some(id) = group.int(0) {
                out.insert(id, group.sums.total_comm(typed.side)?);
            }
        }
    }
    Ok(out)
}

/// One row being built.
struct Group {
    label: Cell,
    purchase_ids: Vec<i64>,
    trip_ids: Vec<i64>,
    figures: Figures,
}

/// An empty table with the columns of the layout (no purchase matches).
fn empty(view: AnalyticsTripView) -> Table {
    let mut columns = vec![group_column(view)];
    columns.extend(figure_columns());
    let mut totals = vec![Cell::Empty];
    totals.extend(Figures::default().cells());
    Table {
        columns,
        rows: Vec::new(),
        total_count: 0,
        totals,
    }
}

/// The saved purchases the filters pick.
fn picked(conn: &Connection, typed: &Typed) -> Result<Vec<PurchaseRec>, ReportsError> {
    Ok(mandi_db::purchases::list(
        conn,
        &ListFilter {
            trip_id: typed.trip_id,
            from: typed.from.clone(),
            to: typed.to.clone(),
            item_id: typed.item_id,
            state: None,
        },
        0,
        -1,
    )?
    .into_iter()
    .filter(|p| p.status == "saved")
    .collect())
}

/// Every saved purchase of these trips.
fn whole_trips(conn: &Connection, trip_ids: &[i64]) -> Result<Vec<PurchaseRec>, ReportsError> {
    let mut whole = Vec::new();
    for trip_id in trip_ids {
        whole.extend(
            mandi_db::purchases::list(
                conn,
                &ListFilter {
                    trip_id: Some(*trip_id),
                    from: None,
                    to: None,
                    item_id: None,
                    state: None,
                },
                0,
                -1,
            )?
            .into_iter()
            .filter(|p| p.status == "saved"),
        );
    }
    Ok(whole)
}

/// The stored part of each partner of a trip: the money back less the money in, as R33 reads it.
fn stored_parts(
    conn: &Connection,
    trip_id: i64,
    shares: &[PartnerShare],
) -> Result<BTreeMap<i64, Paisa>, ReportsError> {
    let money = partner_money(conn, trip_id)?;
    let lot_back = lot_credits_by_partner(conn, trip_id)?;
    let mut parts = BTreeMap::new();
    for share in shares {
        let m = money.get(&share.partner_id).copied().unwrap_or_default();
        let back = paisa(m.back_other_paisa)?.checked_add(paisa(
            lot_back.get(&share.partner_id).copied().unwrap_or(0),
        )?)?;
        let part = back
            .checked_sub(paisa(m.in_shop_paisa)?)?
            .checked_sub(paisa(m.in_pocket_paisa)?)?;
        parts.insert(share.partner_id, part);
    }
    Ok(parts)
}

/// Builds view J.
///
/// # Errors
/// A database error; `ReportsError::Money`.
pub(crate) fn run(conn: &Connection, typed: &Typed, spec: PageSpec) -> Result<Table, ReportsError> {
    let view = typed.trip_view;
    // Only the goods of trips are here; consigned goods and garden goods are not.
    if matches!(
        typed.goods,
        AnalyticsGoods::Consigned | AnalyticsGoods::Garden
    ) {
        return Ok(empty(view));
    }
    let mut purchases = picked(conn, typed)?;
    let mut trip_ids: Vec<i64> = purchases.iter().map(|p| p.trip_id).collect();
    trip_ids.sort_unstable();
    trip_ids.dedup();
    let mut shares = partners_of_many(conn, &trip_ids)?;
    // A partner picked: only the trips he is in.
    if let Some(id) = typed.partner_id {
        trip_ids.retain(|trip| {
            shares
                .get(trip)
                .is_some_and(|list| list.iter().any(|s| s.partner_id == id && s.share_bps > 0))
        });
        purchases.retain(|p| trip_ids.contains(&p.trip_id));
    }
    if purchases.is_empty() {
        return Ok(empty(view));
    }
    // The by-trip layout is of whole trips: the partner money is of the whole trip, so the
    // purchases outside the period or the item are read too.
    if view == AnalyticsTripView::Trip {
        purchases = whole_trips(conn, &trip_ids)?;
        shares = partners_of_many(conn, &trip_ids)?;
    }

    let ids_of: Vec<i64> = purchases.iter().map(|p| p.id).collect();
    let mut sold: HashMap<i64, SoldFigures> = HashMap::new();
    for chunk in ids_of.chunks(IN_CHUNK) {
        sold.extend(sold_figures(conn, chunk)?);
    }
    let commission = commissions(conn, typed, &ids_of)?;
    let mut lines = Vec::with_capacity(purchases.len());
    for purchase in purchases {
        let figures = figures_of(
            &purchase,
            sold.get(&purchase.id).copied().unwrap_or_default(),
            commission.get(&purchase.id).copied().unwrap_or(Paisa::ZERO),
        )?;
        lines.push(Line { purchase, figures });
    }

    // The partners of the trips shown, in the order they were created.
    let mut partner_list: BTreeMap<i64, PartnerShare> = BTreeMap::new();
    for line in &lines {
        for share in shares.get(&line.purchase.trip_id).into_iter().flatten() {
            partner_list
                .entry(share.partner_id)
                .or_insert_with(|| share.clone());
        }
    }

    // The rows.
    let mut built: BTreeMap<(String, i64, i64), Group> = BTreeMap::new();
    for line in &lines {
        let (key, label) = group_of(view, &line.purchase);
        let slot = built.entry(key).or_insert_with(|| Group {
            label,
            purchase_ids: Vec::new(),
            trip_ids: Vec::new(),
            figures: Figures::default(),
        });
        slot.purchase_ids.push(line.purchase.id);
        if !slot.trip_ids.contains(&line.purchase.trip_id) {
            slot.trip_ids.push(line.purchase.trip_id);
        }
        slot.figures.add(&line.figures)?;
    }

    let mut stored: BTreeMap<i64, BTreeMap<i64, Paisa>> = BTreeMap::new();
    if view == AnalyticsTripView::Trip {
        for trip_id in &trip_ids {
            let list = shares.get(trip_id).map(Vec::as_slice).unwrap_or_default();
            stored.insert(*trip_id, stored_parts(conn, *trip_id, list)?);
        }
    }

    let mut columns = vec![group_column(view)];
    columns.extend(figure_columns());
    for partner in partner_list.values() {
        columns.extend(partner_columns(partner));
    }
    let mut grand = Figures::default();
    let mut grand_parts: BTreeMap<i64, Paisa> =
        partner_list.keys().map(|id| (*id, Paisa::ZERO)).collect();
    let mut all_rows = Vec::with_capacity(built.len());
    for (_, mut item) in built {
        // The share of a partner in the row, if every trip of the row gives him the same one.
        let mut common: BTreeMap<i64, Option<i64>> = BTreeMap::new();
        let mut parts: BTreeMap<i64, Paisa> = BTreeMap::new();
        for trip_id in &item.trip_ids {
            for s in shares.get(trip_id).into_iter().flatten() {
                common
                    .entry(s.partner_id)
                    .and_modify(|seen| {
                        if *seen != Some(s.share_bps) {
                            *seen = None;
                        }
                    })
                    .or_insert(Some(s.share_bps));
            }
            for (partner, part) in stored.get(trip_id).into_iter().flatten() {
                add(parts.entry(*partner).or_insert(Paisa::ZERO), *part)?;
            }
        }
        let mut cells = vec![item.label.clone()];
        cells.extend(item.figures.cells());
        for partner in partner_list.keys() {
            let share = common.get(partner).copied().flatten();
            cells.push(match share {
                Some(bps) => Cell::Percent { hundredths: bps },
                None => Cell::Empty,
            });
            let part = parts.get(partner).copied();
            cells.push(part.map_or(Cell::Empty, |paisa| Cell::Money { paisa }));
            if let (Some(part), Some(sum)) = (part, grand_parts.get_mut(partner)) {
                add(sum, part)?;
            }
        }
        grand.add(&item.figures)?;
        item.purchase_ids.sort_unstable();
        all_rows.push(row(
            cells,
            AnalyticsKey {
                purchase_ids: item.purchase_ids.clone(),
                ..AnalyticsKey::default()
            },
        ));
    }

    let mut totals = vec![Cell::Empty];
    totals.extend(grand.cells());
    for partner in partner_list.keys() {
        totals.push(Cell::Empty);
        totals.push(if view == AnalyticsTripView::Trip {
            grand_parts
                .get(partner)
                .copied()
                .map_or(Cell::Empty, |paisa| Cell::Money { paisa })
        } else {
            Cell::Empty
        });
    }
    Ok(Table {
        columns,
        total_count: count_of(all_rows.len()),
        rows: spec.slice(&all_rows),
        totals,
    })
}
