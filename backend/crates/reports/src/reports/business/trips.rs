//! R33 تجارتی منافع / نقصان (the profit or loss of market trips) (PRD 19.2, FR-12.25, FR-12.27,
//! C21, C25).
//!
//! The rows are the saved purchases of the trips, filtered by the day of the purchase and by item.
//! For each purchase the report shows what was bought (نگ and cost), what was sold (نگ, the sales
//! money after commission, and the commission), the cost of the goods sold, what was written off,
//! what is left (نگ and cost) and the result:
//!
//! * sales money = the partners' parts of the seller credits of the purchase's own lots (C25: the
//!   sale after commission and any lot charge) plus the money of its sales in other shops. The
//!   commission column is the seller commission only (E2E-34: 540); next to it the total
//!   commission column (plan 37) is seller commission + buyer commission + لاگا of the sales in our
//!   shop, as the trip screens and analytics view J have it (E2E-34: 1,200). No result changes:
//!   the result stays after the seller commission, as before;
//! * cost released = what sales and write-offs took out of the purchase's cost (C21), kept on the
//!   purchase; cost sold = cost released less the cost written off;
//! * result = sales money less cost released (FR-12.27). It can be below 0 (a loss).
//!
//! These are the figures of the trip screen (`mandi_services::trips`), read from the same stored
//! rows, so the report and the screen cannot disagree. A purchase that is only part sold has its
//! cost left still out of the result; the result of a trip is final when nothing is left.
//!
//! Layouts (the `trip_view` filter): by trip (a heading row for each trip with its totals, then
//! its purchases), by item, by day, by month (all four group the same purchase rows), and by
//! partner: one row for each partner of each trip with his %, the money he paid in from the
//! shop's cash and from his pocket, the money he got back and his part of the result so far (money
//! back less money in, E2E-34b: 3,384 − 4,160 = −776). A part is final only when nothing is left:
//! the parts of a trip add up to its result less its cost left, so the layout shows the cost left
//! of each trip. The partner money is that of the whole trip, whatever the dates and the item
//! chosen. Every figure is as of today, whatever the dates chosen: the dates and the item choose
//! the purchases, not the day the sales were made.
//!
//! The totals row adds up the purchase rows (or the partner rows) over the whole filter; it never
//! adds up a page (Code Standards B31).
//!
//! Example (E2E-34b, 11-10-26): purchase خ-1, مالٹا, bought 20 for 10,400, sold 15 for 8,460 after
//! 540 commission, cost sold 7,800, written off 5 for 2,600, left 0, result −1,940. Partners
//! 40 / 30 / 30: money in 4,160 / 3,120 / 3,120, back 3,384 / 2,538 / 2,538, parts −776 / −582 /
//! −582.

use std::collections::{BTreeMap, HashMap};

use mandi_core::money::Paisa;
use mandi_db::purchases::{ListFilter, PurchaseRec, SoldFigures, sold_figures};
use mandi_db::rusqlite::{Connection, params_from_iter};
use mandi_db::ventures::{lot_credits_by_partner, partner_money, partners_of};

use super::super::PageSpec;
use crate::error::ReportsError;
use crate::reports::common::{IN_CHUNK, add, col, count_of, names_cell, paisa, placeholders};
use crate::reports::filters::Typed;
use crate::reports::ids;
use crate::reports::model::{
    Cell, Column, ColumnKind, ReportId, ReportPage, Row, RowKind, RowSource, SourceKind, TripView,
};

/// The figures of one purchase, or the sum of several.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct Figures {
    pub(crate) bought_qty: i64,
    pub(crate) bought_cost: Paisa,
    pub(crate) sold_qty: i64,
    pub(crate) sales_money: Paisa,
    pub(crate) commission: Paisa,
    /// Seller commission + buyer commission + لاگا (plan 37). Only R33 shows it; analytics view J
    /// puts this number in `commission` and does not read this field.
    pub(crate) total_commission: Paisa,
    pub(crate) cost_sold: Paisa,
    pub(crate) written_off_qty: i64,
    pub(crate) written_off_cost: Paisa,
    pub(crate) left_qty: i64,
    pub(crate) left_cost: Paisa,
    pub(crate) result: Paisa,
}

fn add_qty(total: &mut i64, amount: i64) -> Result<(), ReportsError> {
    *total = total
        .checked_add(amount)
        .ok_or_else(|| ReportsError::Data("the quantities of the trips do not fit".to_owned()))?;
    Ok(())
}

impl Figures {
    /// Adds another purchase's figures with checked maths.
    pub(crate) fn add(&mut self, other: &Figures) -> Result<(), ReportsError> {
        add_qty(&mut self.bought_qty, other.bought_qty)?;
        add(&mut self.bought_cost, other.bought_cost)?;
        add_qty(&mut self.sold_qty, other.sold_qty)?;
        add(&mut self.sales_money, other.sales_money)?;
        add(&mut self.commission, other.commission)?;
        add(&mut self.total_commission, other.total_commission)?;
        add(&mut self.cost_sold, other.cost_sold)?;
        add_qty(&mut self.written_off_qty, other.written_off_qty)?;
        add(&mut self.written_off_cost, other.written_off_cost)?;
        add_qty(&mut self.left_qty, other.left_qty)?;
        add(&mut self.left_cost, other.left_cost)?;
        add(&mut self.result, other.result)?;
        Ok(())
    }

    /// The twelve cells of R33: the eleven figures with the total commission after the seller
    /// commission, in the order of its columns.
    pub(crate) fn r33_cells(&self) -> Vec<Cell> {
        let mut cells = self.cells();
        cells.insert(
            5,
            Cell::Money {
                paisa: self.total_commission,
            },
        );
        cells
    }

    /// The eleven cells of the figures, in the order of the columns (analytics view J).
    pub(crate) fn cells(&self) -> Vec<Cell> {
        let money = |paisa| Cell::Money { paisa };
        let qty = |value| Cell::Number { value };
        vec![
            qty(self.bought_qty),
            money(self.bought_cost),
            qty(self.sold_qty),
            money(self.sales_money),
            money(self.commission),
            money(self.cost_sold),
            qty(self.written_off_qty),
            money(self.written_off_cost),
            qty(self.left_qty),
            money(self.left_cost),
            money(self.result),
        ]
    }
}

/// The figures of one saved purchase: sales money, cost sold, written off, left and the result.
pub(crate) fn figures_of(
    purchase: &PurchaseRec,
    sold: SoldFigures,
    commission: Paisa,
) -> Result<Figures, ReportsError> {
    let sales_money = paisa(sold.sales_paisa)?.checked_add(paisa(sold.other_shop_paisa)?)?;
    let released = paisa(purchase.cost_released_paisa)?;
    let written_off_cost = paisa(sold.cost_written_off_paisa)?;
    let cost_sold = released.checked_sub(written_off_cost)?;
    let total_cost = paisa(purchase.total_cost_paisa)?;
    let left_cost = total_cost.checked_sub(released)?;
    let left_qty = purchase
        .qty
        .checked_sub(purchase.qty_sold)
        .and_then(|v| v.checked_sub(purchase.qty_written_off))
        .ok_or_else(|| ReportsError::Data("the quantity of a purchase does not fit".to_owned()))?;
    Ok(Figures {
        bought_qty: purchase.qty,
        bought_cost: total_cost,
        sold_qty: purchase.qty_sold,
        sales_money,
        commission,
        total_commission: Paisa::ZERO,
        cost_sold,
        written_off_qty: purchase.qty_written_off,
        written_off_cost,
        left_qty,
        left_cost,
        result: sales_money.checked_sub(released)?,
    })
}

/// The seller commission of the active sale lines of the own lots of these purchases.
fn commissions(conn: &Connection, ids_of: &[i64]) -> Result<HashMap<i64, Paisa>, ReportsError> {
    let mut out = HashMap::new();
    for chunk in ids_of.chunks(IN_CHUNK) {
        let sql = format!(
            "SELECT l.purchase_id, COALESCE(SUM(s.seller_comm_paisa), 0)
             FROM bikri_sales s JOIN bikri_lots l ON l.id = s.lot_id
             WHERE s.is_active = 1 AND l.purchase_id IN ({}) GROUP BY l.purchase_id",
            placeholders(chunk.len())
        );
        let mut statement = conn.prepare(&sql)?;
        let rows = statement.query_map(params_from_iter(chunk.iter()), |r| {
            Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?))
        })?;
        for row in rows {
            let (id, sum) = row?;
            out.insert(id, paisa(sum)?);
        }
    }
    Ok(out)
}

/// A saved purchase with its figures.
struct Line {
    purchase: PurchaseRec,
    figures: Figures,
}

fn load(conn: &Connection, typed: &Typed) -> Result<Vec<Line>, ReportsError> {
    let filter = ListFilter {
        hide_cancelled: false,
        order_by: None,
        search: None,
        trip_id: None,
        from: typed.from.clone(),
        to: typed.to.clone(),
        item_id: typed.item_id,
        state: None,
    };
    // Every purchase of the filter; a cancelled one is left out here (it has no goods).
    let mut purchases: Vec<PurchaseRec> = mandi_db::purchases::list(conn, &filter, 0, -1)?
        .into_iter()
        .filter(|p| p.status == "saved")
        .collect();
    purchases.sort_by_key(|p| (p.trip_number, p.entry_date.clone(), p.number));
    let ids_of: Vec<i64> = purchases.iter().map(|p| p.id).collect();
    let mut sold: HashMap<i64, SoldFigures> = HashMap::new();
    for chunk in ids_of.chunks(IN_CHUNK) {
        sold.extend(sold_figures(conn, chunk)?);
    }
    let commission = commissions(conn, &ids_of)?;
    // The total commission (plan 37): the same grouped read as the trip screens and analytics J.
    let total_commission = mandi_db::commission::by_purchases(conn, &ids_of)?;
    purchases
        .into_iter()
        .map(|purchase| {
            let mut figures = figures_of(
                &purchase,
                sold.get(&purchase.id).copied().unwrap_or_default(),
                commission.get(&purchase.id).copied().unwrap_or(Paisa::ZERO),
            )?;
            if let Some(c) = total_commission.get(&purchase.id) {
                let total = c.total_paisa().filter(|t| *t >= 0).ok_or_else(|| {
                    ReportsError::Data(
                        "the commission on a purchase's goods is not valid".to_owned(),
                    )
                })?;
                figures.total_commission = paisa(total)?;
            }
            Ok(Line { purchase, figures })
        })
        .collect()
}

/// The twelve figure columns of R33.
fn figure_columns() -> Vec<Column> {
    vec![
        col("bought_qty", ids::COL_BOUGHT_QTY, ColumnKind::Number),
        col("bought_cost", ids::COL_BOUGHT_COST, ColumnKind::Money),
        col("sold_qty", ids::COL_SOLD, ColumnKind::Number),
        col("sales_money", ids::COL_SALES_MONEY, ColumnKind::Money),
        col("commission", ids::COL_SELLER_COMM, ColumnKind::Money),
        col("total_commission", ids::COL_TOTAL_COMM, ColumnKind::Money),
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

/// A row: its identifying cells, then the twelve figures.
fn figure_row(lead: Vec<Cell>, figures: &Figures, kind: RowKind, source: Option<RowSource>) -> Row {
    let mut cells = lead;
    cells.extend(figures.r33_cells());
    Row {
        cells,
        source,
        kind,
    }
}

/// One group of rows: the key that sorts it, its heading cell and the purchases in it.
struct Group {
    heading: Cell,
    figures: Figures,
    members: Vec<usize>,
}

/// Groups the lines by a key. `BTreeMap` keeps the groups in the order of their keys.
fn group_by<K: Ord>(
    lines: &[Line],
    key_of: impl Fn(&Line) -> K,
    heading_of: impl Fn(&Line) -> Cell,
) -> Result<Vec<Group>, ReportsError> {
    let mut groups: BTreeMap<K, Group> = BTreeMap::new();
    for (i, line) in lines.iter().enumerate() {
        let group = groups.entry(key_of(line)).or_insert_with(|| Group {
            heading: heading_of(line),
            figures: Figures::default(),
            members: Vec::new(),
        });
        group.figures.add(&line.figures)?;
        group.members.push(i);
    }
    Ok(groups.into_values().collect())
}

fn trip_cell(p: &PurchaseRec) -> Cell {
    names_cell(Some(p.trip_name_ur.clone()), Some(p.trip_name_en.clone()))
}

/// Builds R33.
///
/// # Errors
/// A database error; `ReportsError::Money` if a stored amount is not whole rupees or a total does
/// not fit; `ReportsError::Data` if the stored figures of a purchase disagree.
pub(crate) fn run(
    conn: &Connection,
    typed: &Typed,
    spec: PageSpec,
) -> Result<ReportPage, ReportsError> {
    let lines = load(conn, typed)?;
    if typed.trip_view == TripView::Partner {
        return partners(conn, typed, spec, &lines);
    }
    let mut total = Figures::default();
    for line in &lines {
        total.add(&line.figures)?;
    }

    let (lead_columns, rows): (Vec<Column>, Vec<Row>) = match typed.trip_view {
        TripView::Trip | TripView::Partner => {
            let groups = group_by(
                &lines,
                |l| l.purchase.trip_number,
                |l| trip_cell(&l.purchase),
            )?;
            let mut rows = Vec::new();
            for group in groups {
                rows.push(figure_row(
                    vec![group.heading.clone(), Cell::Empty, Cell::Empty, Cell::Empty],
                    &group.figures,
                    RowKind::Group,
                    None,
                ));
                for &i in &group.members {
                    let Some(line) = lines.get(i) else { continue };
                    let p = &line.purchase;
                    rows.push(figure_row(
                        vec![
                            Cell::Empty,
                            Cell::Doc {
                                type_id: ids::doc_id("purchase"),
                                number: Some(p.number),
                            },
                            Cell::Date {
                                iso: p.entry_date.clone(),
                            },
                            names_cell(Some(p.item.name_ur.clone()), Some(p.item.name_en.clone())),
                        ],
                        &line.figures,
                        RowKind::Normal,
                        Some(RowSource {
                            kind: SourceKind::Purchase,
                            id: p.id,
                        }),
                    ));
                }
            }
            (
                vec![
                    col("trip", ids::COL_TRIP, ColumnKind::Names),
                    col("purchase", ids::COL_PURCHASE, ColumnKind::Doc),
                    col("date", ids::COL_DATE, ColumnKind::Date),
                    col("item", ids::COL_ITEM, ColumnKind::Names),
                ],
                rows,
            )
        }
        TripView::Item => {
            let groups = group_by(
                &lines,
                |l| (l.purchase.item.name_en.clone(), l.purchase.item.id),
                |l| {
                    names_cell(
                        Some(l.purchase.item.name_ur.clone()),
                        Some(l.purchase.item.name_en.clone()),
                    )
                },
            )?;
            (
                vec![col("item", ids::COL_ITEM, ColumnKind::Names)],
                groups
                    .into_iter()
                    .map(|g| figure_row(vec![g.heading], &g.figures, RowKind::Normal, None))
                    .collect(),
            )
        }
        TripView::Day => {
            let groups = group_by(
                &lines,
                |l| l.purchase.entry_date.clone(),
                |l| Cell::Date {
                    iso: l.purchase.entry_date.clone(),
                },
            )?;
            (
                vec![col("period", ids::COL_PERIOD, ColumnKind::Date)],
                groups
                    .into_iter()
                    .map(|g| figure_row(vec![g.heading], &g.figures, RowKind::Normal, None))
                    .collect(),
            )
        }
        TripView::Month => {
            let groups = group_by(
                &lines,
                |l| {
                    l.purchase
                        .entry_date
                        .get(..7)
                        .unwrap_or_default()
                        .to_owned()
                },
                |l| month_cell(&l.purchase.entry_date),
            )?;
            (
                vec![col("period", ids::COL_PERIOD, ColumnKind::Month)],
                groups
                    .into_iter()
                    .map(|g| figure_row(vec![g.heading], &g.figures, RowKind::Normal, None))
                    .collect(),
            )
        }
    };

    let mut columns = lead_columns;
    let lead_len = columns.len();
    columns.extend(figure_columns());
    let mut totals = vec![Cell::Empty; lead_len];
    totals.extend(total.r33_cells());
    Ok(ReportPage {
        group_above: None,
        id: ReportId::R33,
        title_id: ids::title_id("R33"),
        filters: typed.resolved.clone(),
        columns,
        total_count: count_of(rows.len()),
        rows: spec.slice(&rows),
        offset: spec.offset,
        limit: spec.reported_limit(),
        totals_label_id: ids::TOTAL.to_owned(),
        totals,
        summary: Vec::new(),
    })
}

/// A `YYYY-MM-DD` day as the month cell of its month.
fn month_cell(iso: &str) -> Cell {
    let year = iso.get(..4).and_then(|y| y.parse::<i64>().ok());
    let month = iso.get(5..7).and_then(|m| m.parse::<i64>().ok());
    match (year, month) {
        (Some(year), Some(month)) => Cell::Month { year, month },
        _ => Cell::Empty,
    }
}

/// The partner layout: each partner of each trip with his money in, money back and part so far.
fn partners(
    conn: &Connection,
    typed: &Typed,
    spec: PageSpec,
    lines: &[Line],
) -> Result<ReportPage, ReportsError> {
    let mut seen: BTreeMap<i64, &PurchaseRec> = BTreeMap::new();
    for line in lines {
        seen.entry(line.purchase.trip_id).or_insert(&line.purchase);
    }
    let mut trips: Vec<&PurchaseRec> = seen.into_values().collect();
    trips.sort_by_key(|p| p.trip_number);
    // The cost still left in every saved purchase of each trip. The filters pick the trips only:
    // a partner's money is for the whole trip, so his part so far plus the cost left of the whole
    // trip is the result of the trip, whatever dates or item the filter names (money agent,
    // round 8).
    let mut cost_left: HashMap<i64, Paisa> = HashMap::new();
    for trip in &trips {
        let all = mandi_db::purchases::list(
            conn,
            &ListFilter {
                hide_cancelled: false,
                order_by: None,
                search: None,
                trip_id: Some(trip.trip_id),
                from: None,
                to: None,
                item_id: None,
                state: None,
            },
            0,
            -1,
        )?;
        let mut left = Paisa::ZERO;
        for purchase in all.iter().filter(|p| p.status == "saved") {
            let remaining = paisa(purchase.total_cost_paisa)?
                .checked_sub(paisa(purchase.cost_released_paisa)?)?;
            add(&mut left, remaining)?;
        }
        cost_left.insert(trip.trip_id, left);
    }
    let mut sum_left = Paisa::ZERO;

    let (mut sum_shop, mut sum_pocket, mut sum_back, mut sum_net) =
        (Paisa::ZERO, Paisa::ZERO, Paisa::ZERO, Paisa::ZERO);
    let mut rows = Vec::new();
    for trip in trips {
        let shares = partners_of(conn, trip.trip_id)?;
        let money = partner_money(conn, trip.trip_id)?;
        let lot_back = lot_credits_by_partner(conn, trip.trip_id)?;
        let mut trip_totals = [Paisa::ZERO; 4];
        let mut partner_rows = Vec::with_capacity(shares.len());
        for share in shares {
            let m = money.get(&share.partner_id).copied().unwrap_or_default();
            let in_shop = paisa(m.in_shop_paisa)?;
            let in_pocket = paisa(m.in_pocket_paisa)?;
            let back = paisa(m.back_other_paisa)?.checked_add(paisa(
                lot_back.get(&share.partner_id).copied().unwrap_or(0),
            )?)?;
            let net = back.checked_sub(in_shop)?.checked_sub(in_pocket)?;
            for (slot, amount) in trip_totals.iter_mut().zip([in_shop, in_pocket, back, net]) {
                add(slot, amount)?;
            }
            partner_rows.push(Row {
                cells: vec![
                    Cell::Empty,
                    names_cell(Some(share.name_ur), Some(share.name_en)),
                    Cell::Percent {
                        hundredths: share.share_bps,
                    },
                    Cell::Money { paisa: in_shop },
                    Cell::Money { paisa: in_pocket },
                    Cell::Money { paisa: back },
                    Cell::Money { paisa: net },
                    Cell::Empty,
                ],
                source: None,
                kind: RowKind::Normal,
            });
        }
        let [t_shop, t_pocket, t_back, t_net] = trip_totals;
        let t_left = cost_left.get(&trip.trip_id).copied().unwrap_or(Paisa::ZERO);
        add(&mut sum_left, t_left)?;
        for (slot, amount) in [&mut sum_shop, &mut sum_pocket, &mut sum_back, &mut sum_net]
            .into_iter()
            .zip(trip_totals)
        {
            add(slot, amount)?;
        }
        rows.push(Row {
            cells: vec![
                trip_cell(trip),
                Cell::Empty,
                Cell::Empty,
                Cell::Money { paisa: t_shop },
                Cell::Money { paisa: t_pocket },
                Cell::Money { paisa: t_back },
                Cell::Money { paisa: t_net },
                Cell::Money { paisa: t_left },
            ],
            source: None,
            kind: RowKind::Group,
        });
        rows.extend(partner_rows);
    }

    Ok(ReportPage {
        group_above: None,
        id: ReportId::R33,
        title_id: ids::title_id("R33"),
        filters: typed.resolved.clone(),
        columns: vec![
            col("trip", ids::COL_TRIP, ColumnKind::Names),
            col("partner", ids::COL_PARTNER, ColumnKind::Names),
            col("share", ids::COL_SHARE, ColumnKind::Percent),
            col("money_in_shop", ids::COL_MONEY_IN_SHOP, ColumnKind::Money),
            col(
                "money_in_pocket",
                ids::COL_MONEY_IN_POCKET,
                ColumnKind::Money,
            ),
            col("money_back", ids::COL_MONEY_BACK, ColumnKind::Money),
            col("net", ids::COL_PART_OF_RESULT, ColumnKind::Money),
            col("cost_left", ids::COL_LEFT_COST, ColumnKind::Money),
        ],
        total_count: count_of(rows.len()),
        rows: spec.slice(&rows),
        offset: spec.offset,
        limit: spec.reported_limit(),
        totals_label_id: ids::TOTAL.to_owned(),
        totals: vec![
            Cell::Empty,
            Cell::Empty,
            Cell::Empty,
            Cell::Money { paisa: sum_shop },
            Cell::Money { paisa: sum_pocket },
            Cell::Money { paisa: sum_back },
            Cell::Money { paisa: sum_net },
            Cell::Money { paisa: sum_left },
        ],
        summary: Vec::new(),
    })
}
