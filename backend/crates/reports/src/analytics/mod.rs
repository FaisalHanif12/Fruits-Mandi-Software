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
mod calc;
mod facts;
mod filters;
mod fruit;
mod gardens;
mod ids;
mod item;
mod lines;
mod matrix;
mod model;
mod partner;
mod partner_lines;
pub(crate) mod print;
mod seller;
mod store;
mod table;
mod top;
mod trend;
mod trips;

use mandi_core::printing::Paper;
use mandi_db::Db;
use mandi_db::rusqlite::Connection;

use crate::error::ReportsError;
use crate::print::load::{load_firm, load_grouping, load_setup};
use crate::reports::common::today;
use crate::reports::print_data::PrintOptions;
use crate::reports::{PageSpec, page_spec};

pub use calc::{average_days, share_hundredths};
pub use model::{
    AnalyticsCatalogEntry, AnalyticsColumn, AnalyticsCompare, AnalyticsFilterKind,
    AnalyticsFilters, AnalyticsGoods, AnalyticsGroupBy, AnalyticsHeading, AnalyticsKey,
    AnalyticsLinesInput, AnalyticsLinesPage, AnalyticsPage, AnalyticsPartnerMoney, AnalyticsPeriod,
    AnalyticsPrintColumn, AnalyticsPrintData, AnalyticsPrintHeading, AnalyticsPrintInput,
    AnalyticsPrintRow, AnalyticsResolvedFilters, AnalyticsRow, AnalyticsRunInput,
    AnalyticsSavedView, AnalyticsSide, AnalyticsTripView, AnalyticsView, AnalyticsViewsSaveInput,
};
pub use store::{MAX_NAME_LETTERS, MAX_VIEWS, PREF_KEY, views_get, views_save};

use filters::{Typed, resolve};
use table::Table;

/// The views of the catalogue, in order.
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

/// The filters a view takes (PRD 18.1). Views D and G take the grouping, view J the layout of its
/// rows. Partners work on periods: F takes the period and the partner only; G and J take the sale
/// filters, the partner, the trip and the goods, and no comparison.
fn filters_of(view: AnalyticsView) -> Vec<AnalyticsFilterKind> {
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
    kinds
}

/// The catalogue: every view with its title and the filters it takes (PRD 18.2).
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
    let table = match view {
        AnalyticsView::A => item::run(conn, &typed, spec)?,
        AnalyticsView::B => seller::run(conn, &typed, spec)?,
        AnalyticsView::C => buyer::run(conn, &typed, spec)?,
        AnalyticsView::D => matrix::run_d(conn, &typed, spec)?,
        AnalyticsView::E => matrix::run_e(conn, &typed, spec)?,
        AnalyticsView::F => partner::run(conn, &typed, spec)?,
        AnalyticsView::G => trend::run(conn, &typed, spec)?,
        AnalyticsView::H => top::run(conn, &typed, spec)?,
        AnalyticsView::I => gardens::run(conn, &typed, spec)?,
        AnalyticsView::J => trips::run(conn, &typed, spec)?,
        AnalyticsView::K => fruit::run(conn, &typed, spec)?,
    };
    Ok((typed, table))
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
        let (typed, table) = build(conn, db, input.view, &input.filters, spec)?;
        Ok(page_of(input.view, &typed, table, spec))
    })
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
        let (typed, table) = build(conn, db, view, filters, all)?;
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
    let (page, grouping, setup) = db.read(|conn| {
        let (typed, table) = build(conn, db, input.view, &input.filters, all)?;
        Ok::<_, ReportsError>((
            page_of(input.view, &typed, table, all),
            load_grouping(conn)?,
            load_setup(conn)?,
        ))
    })?;
    print::build(
        &page,
        &PrintOptions {
            language: input.language,
            grouping,
            urdu_digits_setting: setup.urdu_digits,
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

    /// The facts query of view B (by seller and بکری) for a date range.
    pub fn view_b(from: Option<&str>, to: Option<&str>) -> (String, Vec<Value>) {
        let c = facts::dated((from, to));
        (
            facts::groups_sql(&c, &["f.seller_account_id", "f.bikri_id"]),
            c.args().to_vec(),
        )
    }
}
