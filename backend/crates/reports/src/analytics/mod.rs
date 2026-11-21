//! Analytics (plan 21, PRD 18, NFR-SPD-12, NFR-SPD-24): the views A by item, B by seller, C by
//! buyer, D item by period, E seller by item and H top 10, over the sale facts; F by partner,
//! G trend, I gardens, J trips and K fruit taken, over the share rows, the journal and their own
//! tables.
//!
//! - [`catalog`] lists the views with the filters each takes;
//! - [`run`] reads and checks the filters, and returns one page of typed rows with the number
//!   of rows and the totals over the whole filter (never a sum of the page);
//! - [`lines`] returns the lines behind any number, each opening its بکری (FR-18.2);
//! - [`print_data`] returns the same view with every row, formatted for a print language
//!   (FR-18.4);
//! - [`views_get`] and [`views_save`] keep the saved views of the user (FR-18.5).
//!
//! Analytics reads posted records only and writes nothing but the saved views, which are
//! preferences and keep no figure (PRD FR-18.8). Sales figures are the stored values of the sale
//! lines, summed from `analytics_facts` with their sign, never worked out again with today's
//! rules (FR-18.6); a cancelled posting cancels itself out (FR-18.7).
//!
//! Example (the worked day, view A, no filters): کیلا 3,928 (72.10%) and سیب 1,520 (27.90%) of
//! 5,448 total commission, the same as the dashboard.

mod buyer;
mod buyer_matrix;
mod calc;
mod chart;
mod expense;
mod expense_partners;
pub(crate) mod facts;
mod filters;
mod fruit;
mod gardens;
mod grouped;
mod ids;
mod item;
mod lines;
mod matrix;
mod measure;
mod model;
mod partner;
mod partner_lines;
pub(crate) mod print;
mod seller;
mod series;
mod store;
mod table;
mod top;
mod trend;
mod trips;

use mandi_core::printing::Paper;
use mandi_db::Db;
use mandi_db::rusqlite::Connection;

use crate::error::ReportsError;
use crate::print::load::{load_firm, load_grouping};
use crate::reports::common::today;
use crate::reports::print_data::PrintOptions;
use crate::reports::{PageSpec, page_spec};

pub use calc::{average_days, share_hundredths};
pub use filters::compare_window;
pub use model::{
    AnalyticsCatalogEntry, AnalyticsChart, AnalyticsChartKind, AnalyticsChartPoint,
    AnalyticsChartSeries, AnalyticsColumn, AnalyticsCompare, AnalyticsExpensePart,
    AnalyticsFilterKind, AnalyticsFilters, AnalyticsGoods, AnalyticsGroupBy, AnalyticsHeading,
    AnalyticsKey, AnalyticsLinesInput, AnalyticsLinesPage, AnalyticsMeasure, AnalyticsPage,
    AnalyticsPartnerMoney,
    AnalyticsPeriod, AnalyticsPrintColumn, AnalyticsPrintData, AnalyticsPrintHeading,
    AnalyticsPrintInput, AnalyticsPrintRow, AnalyticsResolvedFilters, AnalyticsRow,
    AnalyticsRunInput, AnalyticsSavedView, AnalyticsSide, AnalyticsTripView, AnalyticsView,
    AnalyticsViewsSaveInput,
};
pub use series::{
    BALANCE_WINDOW_MONTHS, BalanceSeries, BalanceSubject, MonthAmount, SalePart, balance_series,
    expense_head_months, garden_sales_months, receipts_months, sale_part_months, trip_sales_months,
};
pub use store::{MAX_NAME_LETTERS, MAX_VIEWS, PREF_KEY, views_get, views_save};

use filters::{Typed, resolve};
use table::Table;

/// The views of the assistant's catalogue, in order: A to K, the views its fixed instructions name
/// (PRD FR-7.87, `D6-system-prompt.md`). Views L and M are on the screens only (PRD v1.42, plan 77).
const VIEWS: [AnalyticsView; 11] = [
    AnalyticsView::A,
    AnalyticsView::B,
    AnalyticsView::C,
    AnalyticsView::D,
    AnalyticsView::E,
    AnalyticsView::F,
    AnalyticsView::G,
    AnalyticsView::H,
    AnalyticsView::I,
    AnalyticsView::J,
    AnalyticsView::K,
];

/// Every view of the screens, in order: A to K and the two of PRD v1.42 (plan 77).
const SCREEN_VIEWS: [AnalyticsView; 13] = [
    AnalyticsView::A,
    AnalyticsView::B,
    AnalyticsView::C,
    AnalyticsView::D,
    AnalyticsView::E,
    AnalyticsView::F,
    AnalyticsView::G,
    AnalyticsView::H,
    AnalyticsView::I,
    AnalyticsView::J,
    AnalyticsView::K,
    AnalyticsView::L,
    AnalyticsView::M,
];

/// The filters a view takes (PRD 18.1). Views D and G take the grouping, view J the layout of its
/// rows. Partners work on periods: F takes the period and the partner only; G and J take the sale
/// filters, the partner, the trip and the goods, and no comparison.
fn filters_of(view: AnalyticsView) -> Vec<AnalyticsFilterKind> {
    // Expenses by head: the period and the columns. It counts no sale, so no sale filter applies.
    if view == AnalyticsView::M {
        return vec![
            AnalyticsFilterKind::Period,
            AnalyticsFilterKind::GroupBy,
            AnalyticsFilterKind::Partner,
            AnalyticsFilterKind::ExpensePart,
        ];
    }
    if view == AnalyticsView::F {
        return vec![AnalyticsFilterKind::Period, AnalyticsFilterKind::Partner];
    }
    // Gardens are picked by the day made, the main fruit, a partner and a garden; the side narrows
    // the commission on their goods.
    if view == AnalyticsView::I {
        return vec![
            AnalyticsFilterKind::Period,
            AnalyticsFilterKind::Item,
            AnalyticsFilterKind::Side,
            AnalyticsFilterKind::Partner,
            AnalyticsFilterKind::Garden,
        ];
    }
    // Fruit taken is picked by the day, the item, a partner (or the guests row), a garden whose lot
    // it came from, and the kind of goods.
    if view == AnalyticsView::K {
        return vec![
            AnalyticsFilterKind::Period,
            AnalyticsFilterKind::Item,
            AnalyticsFilterKind::Partner,
            AnalyticsFilterKind::Garden,
            AnalyticsFilterKind::Goods,
        ];
    }
    let mut kinds = vec![
        AnalyticsFilterKind::Period,
        AnalyticsFilterKind::Item,
        AnalyticsFilterKind::Seller,
        AnalyticsFilterKind::Buyer,
        AnalyticsFilterKind::City,
        AnalyticsFilterKind::Adda,
        AnalyticsFilterKind::Side,
        AnalyticsFilterKind::Trip,
        AnalyticsFilterKind::Goods,
    ];
    // View L is each buyer's own commission (buyer commission and لاگا): the side is not his.
    if view == AnalyticsView::L {
        kinds.retain(|k| *k != AnalyticsFilterKind::Side);
    }
    if view != AnalyticsView::J {
        kinds.push(AnalyticsFilterKind::Garden);
    }
    if !matches!(view, AnalyticsView::G | AnalyticsView::J) {
        kinds.push(AnalyticsFilterKind::Compare);
    }
    if matches!(view, AnalyticsView::D | AnalyticsView::G) {
        kinds.push(AnalyticsFilterKind::GroupBy);
    }
    if matches!(view, AnalyticsView::G | AnalyticsView::J) {
        kinds.push(AnalyticsFilterKind::Partner);
    }
    if view == AnalyticsView::J {
        kinds.push(AnalyticsFilterKind::TripView);
    }
    // All the items of a group, پھل or سبزی: views A, B, C, D, G and H (PRD v1.29). It goes right
    // after the item in the list.
    if filters::takes_item_group(view) {
        let at = kinds
            .iter()
            .position(|k| *k == AnalyticsFilterKind::Item)
            .map_or(kinds.len(), |i| i.saturating_add(1));
        kinds.insert(at, AnalyticsFilterKind::ItemGroup);
    }
    // The quality of the sale lines: view A only (PRD FR-12.42).
    if view == AnalyticsView::A {
        kinds.push(AnalyticsFilterKind::Grade);
    }
    kinds
}

/// The assistant's catalogue: the views A to K with their titles and the filters they take (PRD
/// 18.2). Its fixed instructions name exactly these (plan 77 leaves them unchanged).
pub fn catalog() -> Vec<AnalyticsCatalogEntry> {
    VIEWS
        .iter()
        .map(|view| AnalyticsCatalogEntry {
            id: *view,
            title_id: ids::title_id(*view),
            filters: filters_of(*view),
        })
        .collect()
}

/// The catalogue of the screens: every view, A to M, with its title and the filters it takes
/// (PRD 18.2, plan 77). `analytics_catalog` sends this one.
pub fn screen_catalog() -> Vec<AnalyticsCatalogEntry> {
    SCREEN_VIEWS
        .iter()
        .map(|view| {
            let mut filters = filters_of(*view);
            // The rows of F, I, J and K can be grouped by day, month or year on the screens (plan 77
            // coverage audit); the assistant's catalogue does not list it.
            if grouped::takes_row_grouping(*view, AnalyticsExpensePart::Shop)
                && !filters.contains(&AnalyticsFilterKind::GroupBy)
            {
                filters.push(AnalyticsFilterKind::GroupBy);
            }
            // What the view counts (PRD FR-18.10, plan 84): the screens only; the assistant's
            // catalogue does not list it.
            if filters::takes_measure(*view) {
                filters.push(AnalyticsFilterKind::Measure);
            }
            AnalyticsCatalogEntry {
                id: *view,
                title_id: ids::title_id(*view),
                filters,
            }
        })
        .collect()
}

/// Reads and checks the filters, then builds the view, all from one snapshot of the books.
fn build(
    conn: &Connection,
    db: &Db,
    view: AnalyticsView,
    input: &AnalyticsFilters,
    spec: PageSpec,
) -> Result<(Typed, Table), ReportsError> {
    crate::reports::common::in_snapshot(conn, |conn| build_in(conn, db, view, input, spec))
}

fn build_in(
    conn: &Connection,
    db: &Db,
    view: AnalyticsView,
    input: &AnalyticsFilters,
    spec: PageSpec,
) -> Result<(Typed, Table), ReportsError> {
    let now = today(db.clock().now_utc())?;
    let typed = resolve(conn, now, view, input)?;
    let grouped = typed.rows_by.filter(|_| grouped::applies(view, &typed));
    let table = match (view, grouped) {
        (_, Some(group)) => grouped::run(conn, &typed, view, group, spec)?,
        (AnalyticsView::A, _) => item::run(conn, &typed, spec)?,
        (AnalyticsView::B, _) => seller::run(conn, &typed, spec)?,
        (AnalyticsView::C, _) => buyer::run(conn, &typed, spec)?,
        (AnalyticsView::D, _) => matrix::run_d(conn, &typed, spec)?,
        (AnalyticsView::E, _) => matrix::run_e(conn, &typed, spec)?,
        (AnalyticsView::F, _) => partner::run(conn, &typed, spec)?,
        (AnalyticsView::G, _) => trend::run(conn, &typed, spec)?,
        (AnalyticsView::H, _) => top::run(conn, &typed, spec)?,
        (AnalyticsView::I, _) => gardens::run(conn, &typed, spec)?,
        (AnalyticsView::J, _) => trips::run(conn, &typed, spec)?,
        (AnalyticsView::K, _) => fruit::run(conn, &typed, spec)?,
        (AnalyticsView::L, _) => buyer_matrix::run_l(conn, &typed, spec)?,
        (AnalyticsView::M, _) => expense::run_m(conn, &typed, spec)?,
    };
    let mut table = table;
    if typed.hide_partners {
        drop_partner_columns(&mut table);
    }
    Ok((typed, table))
}

/// True for the key of a column that belongs to one partner: `share_3`, `result_3`, `part_3`
/// (views I and J).
fn is_partner_column(key: &str) -> bool {
    ["share_", "result_", "part_"].iter().any(|prefix| {
        key.strip_prefix(prefix)
            .is_some_and(|rest| !rest.is_empty() && rest.bytes().all(|b| b.is_ascii_digit()))
    })
}

/// Takes the columns of each partner out of a table, with their cells in every row and in the
/// totals row (PRD FR-17.11: the partner columns are behind the partner code).
fn drop_partner_columns(table: &mut Table) {
    let keep: Vec<bool> = table
        .columns
        .iter()
        .map(|column| !is_partner_column(&column.key))
        .collect();
    if keep.iter().all(|&k| k) {
        return;
    }
    let filter = |cells: &mut Vec<crate::reports::Cell>| {
        let mut index = 0;
        cells.retain(|_| {
            let kept = keep.get(index).copied().unwrap_or(true);
            index += 1;
            kept
        });
    };
    let mut index = 0;
    table.columns.retain(|_| {
        let kept = keep.get(index).copied().unwrap_or(true);
        index += 1;
        kept
    });
    for row in &mut table.rows {
        filter(&mut row.cells);
    }
    filter(&mut table.totals);
}

fn page_of(view: AnalyticsView, typed: &Typed, table: Table, spec: PageSpec) -> AnalyticsPage {
    AnalyticsPage {
        view,
        title_id: ids::title_id(view),
        filters: typed.resolved.clone(),
        columns: table.columns,
        rows: table.rows,
        offset: spec.offset,
        limit: spec.reported_limit(),
        total_count: table.total_count,
        totals_label_id: ids::TOTAL.to_owned(),
        totals: table.totals,
        charts: table.charts,
    }
}

/// One page of a view with the totals over the whole filter.
///
/// # Errors
/// `ReportsError::Rejected` with the problems of the filters or the page, each with its field;
/// a database or money error.
pub fn run(db: &Db, input: &AnalyticsRunInput) -> Result<AnalyticsPage, ReportsError> {
    let spec = page_spec(input.offset, input.limit)?;
    db.read(|conn| {
        let (typed, table) = table_for(
            conn,
            db,
            input.view,
            &input.filters,
            spec,
            input.sort.as_ref(),
        )?;
        Ok(page_of(input.view, &typed, table, spec))
    })
}

/// Builds a view as the screen asked: sorted, with its charts and the page of `spec`. The run, the
/// print and the Excel export use it, so what is printed is what is on the screen (plan 38 R4).
///
/// View C cuts its page early, because the dues and the days to pay are worked out for the page
/// only: it sorts and makes its chart itself (see `buyer`). Every other view builds its whole
/// table, which costs no more reading than before (they all work out every row), so the chart and
/// a sort come from the whole filter (decision D1).
fn table_for(
    conn: &Connection,
    db: &Db,
    view: AnalyticsView,
    filters: &AnalyticsFilters,
    spec: PageSpec,
    sort: Option<&mandi_core::paging::SortInput>,
) -> Result<(Typed, Table), ReportsError> {
    // A sale view with its rows grouped by period reads its own page: it counts the rows of each
    // bucket and reads only the buckets of the page, so no screen waits for every row of ten years
    // (robustness finding 2). A sort by a column needs every row, so it is built the long way.
    if sort.is_none() && grouped::pages_itself(view, filters.group_by) {
        return build(conn, db, view, filters, spec);
    }
    // View C cuts its page early; with its rows grouped by period it is built like the others.
    if view == AnalyticsView::C && filters.group_by.is_none() {
        return crate::reports::common::in_snapshot(conn, |conn| {
            build_with_sort(conn, db, view, filters, spec, sort)
        });
    }
    // The whole table of one filter, with its sort and its charts, is worked out once and kept in
    // memory: the next page of the same filter cuts its rows from it, and does not group ten years
    // of facts again. The question includes the day of today, because the shortcuts of a period
    // (this month, last year) are worked out from it (`result_cache`).
    let now = today(db.clock().now_utc())?;
    let what = format!(
        "analytics view|{view:?}|{}|{filters:?}|{sort:?}",
        now.to_iso()
    );
    let built = crate::result_cache::remembered(conn, &what, || {
        let all = PageSpec {
            offset: 0,
            limit: None,
        };
        let (typed, mut table) = build(conn, db, view, filters, all)?;
        // A view with its rows grouped by period has no ranked chart: its first cell is the period.
        let charts = if grouped::applies(view, &typed) {
            Vec::new()
        } else if view == AnalyticsView::M && typed.expense_part == AnalyticsExpensePart::Partner {
            // The kharch each partner spent himself, ranked (plan 77).
            chart::ranked_by(&table.columns, &table.rows, "own")?
        } else if chart::makes_own_chart(view, typed.measure) {
            // A, B, C and G keep every column of their tables, so with a measure they made their
            // chart from their sums (PRD FR-18.10).
            std::mem::take(&mut table.charts)
        } else {
            chart::build(
                view,
                Some(typed.trip_view),
                typed.measure,
                &table.columns,
                &table.rows,
                &table.totals,
            )?
        };
        if let Some(sort) = sort {
            table.rows = chart::sort_rows(&table.columns, std::mem::take(&mut table.rows), sort)?;
        }
        table.charts = charts;
        table.total_count = i64::try_from(table.rows.len()).unwrap_or(i64::MAX);
        Ok((typed, table))
    })?;
    let (typed, full) = &*built;
    Ok((
        typed.clone(),
        Table {
            columns: full.columns.clone(),
            rows: spec.slice(&full.rows),
            total_count: full.total_count,
            totals: full.totals.clone(),
            charts: full.charts.clone(),
        },
    ))
}

fn build_with_sort(
    conn: &Connection,
    db: &Db,
    view: AnalyticsView,
    input: &AnalyticsFilters,
    spec: PageSpec,
    sort: Option<&mandi_core::paging::SortInput>,
) -> Result<(Typed, Table), ReportsError> {
    let now = today(db.clock().now_utc())?;
    let mut typed = resolve(conn, now, view, input)?;
    typed.sort = sort.cloned();
    let table = buyer::run(conn, &typed, spec)?;
    Ok((typed, table))
}

/// The lines behind a row or a cell of a view (PRD FR-18.2), one page of them, with the totals
/// over all of them.
///
/// # Errors
/// As [`run`]; `ERR_OUT_OF_RANGE` on `column_key` when the keys name different records.
pub fn lines(db: &Db, input: &AnalyticsLinesInput) -> Result<AnalyticsLinesPage, ReportsError> {
    let spec = page_spec(input.offset, input.limit)?;
    db.read(|conn| {
        let now = today(db.clock().now_utc())?;
        let typed = resolve(conn, now, input.view, &input.filters)?;
        lines::run(conn, &typed, input, spec)
    })
}

/// Every row of a view with the firm header names and the number grouping, for the Excel
/// export. Reads only; one database read.
///
/// # Errors
/// As [`run`].
pub(crate) fn export_page(
    db: &Db,
    view: AnalyticsView,
    filters: &AnalyticsFilters,
    sort: Option<&mandi_core::paging::SortInput>,
) -> Result<
    (
        AnalyticsPage,
        crate::print::load::FirmNames,
        mandi_core::money::Grouping,
    ),
    ReportsError,
> {
    let all = PageSpec {
        offset: 0,
        limit: None,
    };
    db.read(|conn| {
        let (typed, table) = table_for(conn, db, view, filters, all, sort)?;
        Ok::<_, ReportsError>((
            page_of(view, &typed, table, all),
            load_firm(conn)?,
            load_grouping(conn)?,
        ))
    })
}

/// The print data of a view: every row of the filter, formatted for the print language.
///
/// # Errors
/// As [`run`].
pub fn print_data(
    db: &Db,
    input: &AnalyticsPrintInput,
) -> Result<AnalyticsPrintData, ReportsError> {
    let all = PageSpec {
        offset: 0,
        limit: None,
    };
    let (page, grouping) = db.read(|conn| {
        let (typed, table) = table_for(
            conn,
            db,
            input.view,
            &input.filters,
            all,
            input.sort.as_ref(),
        )?;
        Ok::<_, ReportsError>((
            page_of(input.view, &typed, table, all),
            load_grouping(conn)?,
        ))
    })?;
    print::build(
        &page,
        &PrintOptions {
            language: input.language,
            grouping,
            paper: input.paper.unwrap_or(Paper::A4),
        },
    )
}

/// The SQL of the analytics views for the query plan test (Code Standards B18): the grouped
/// facts query for a date range, with the values to bind.
#[doc(hidden)]
pub mod plans {
    use mandi_db::rusqlite::types::Value;

    use super::facts;

    /// The facts query of view A (by item) for a date range.
    pub fn view_a(from: Option<&str>, to: Option<&str>) -> (String, Vec<Value>) {
        let c = facts::dated((from, to));
        (facts::groups_sql(&c, &["f.item_id"]), c.args().to_vec())
    }

    /// The query every view reads the facts with since plan 80 (no `GROUP BY`, the sums are added up in
    /// memory): the dimensions of view H's buyer list and the six summed columns for a date range.
    pub fn facts_rows(from: Option<&str>, to: Option<&str>) -> (String, Vec<Value>) {
        let c = facts::dated((from, to));
        (
            facts::rows_sql(&c, &["f.buyer_account_id", "f.chukta_person_key"]),
            c.args().to_vec(),
        )
    }

    /// The facts query of view B (by seller and بکری) for a date range.
    pub fn view_b(from: Option<&str>, to: Option<&str>) -> (String, Vec<Value>) {
        let c = facts::dated((from, to));
        (
            facts::groups_sql(&c, &["f.seller_account_id", "f.bikri_id"]),
            c.args().to_vec(),
        )
    }
}

#[cfg(test)]
mod partner_column_tests {
    use super::*;
    use crate::reports::{Cell, ColumnKind};

    #[test]
    fn only_the_columns_of_a_partner_are_dropped_with_their_cells() {
        let keys = [
            "garden", "share_1", "result_1", "result", "share_22", "part_3", "shares",
        ];
        let mut table = Table {
            columns: keys
                .iter()
                .map(|key| table::col(key, "title", ColumnKind::Label))
                .collect(),
            rows: vec![model::AnalyticsRow {
                cells: (0..keys.len())
                    .map(|i| Cell::Number {
                        value: i64::try_from(i).unwrap_or(0),
                    })
                    .collect(),
                key: None,
                kind: crate::reports::RowKind::Normal,
            }],
            total_count: 1,
            totals: (0..keys.len())
                .map(|i| Cell::Number {
                    value: i64::try_from(i).unwrap_or(0),
                })
                .collect(),
            charts: Vec::new(),
        };
        drop_partner_columns(&mut table);
        let left: Vec<&str> = table.columns.iter().map(|c| c.key.as_str()).collect();
        assert_eq!(left, ["garden", "result", "shares"]);
        let cells = |cells: &[Cell]| -> Vec<i64> {
            cells
                .iter()
                .map(|c| match c {
                    Cell::Number { value } => *value,
                    _ => -1,
                })
                .collect()
        };
        // The cells of columns 0, 3 and 6 stay, in both the row and the totals.
        assert_eq!(cells(&table.rows[0].cells), [0, 3, 6]);
        assert_eq!(cells(&table.totals), [0, 3, 6]);
    }
}
