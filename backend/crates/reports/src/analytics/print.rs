//! The print data of a view (PRD FR-18.4, FR-19.3, FR-19.13): the same page the screen shows, with
//! every row, formatted for the print language, on the shared print layout of the reports.
//!
//! - every number is formatted here: grouping by the number grouping setting, English digits
//!   unless the Urdu digits setting is on and the print language is Urdu;
//! - titles, column headings and labels are Fluent message IDs; names are already in the print
//!   language; the month of a column is named by the template from Fluent;
//! - the header is the shared one of the other prints, and the filters of the run are listed in
//!   it: the dates, the picked records, the side, the grouping and the compared period.
//!
//! Example (view A, October 2026, English): the title row reads `analytics-view-a-title`, the
//! total row `60`, `41,400`, `2,484`, `2,484`, `480`, `5,448` and `100.00`.

use mandi_core::auth::Language;

use super::ids;
use super::model::{
    AnalyticsCompare, AnalyticsGoods, AnalyticsGroupBy, AnalyticsHeading, AnalyticsPage,
    AnalyticsPrintColumn, AnalyticsPrintData, AnalyticsPrintHeading, AnalyticsPrintRow,
    AnalyticsResolvedFilters, AnalyticsSide, AnalyticsTripView,
};
use crate::error::ReportsError;
use crate::print::{PrintFact, PrintFirm, TextDir};
use crate::reports::print_data::{Fmt, PrintOptions, fact};

fn heading(fmt: &Fmt, heading: &AnalyticsHeading) -> Result<AnalyticsPrintHeading, ReportsError> {
    Ok(match heading {
        AnalyticsHeading::Day { iso } => AnalyticsPrintHeading::Day {
            text: fmt.style.date(iso)?,
        },
        AnalyticsHeading::Month { year, month } => AnalyticsPrintHeading::Month {
            year: fmt.style.plain(*year),
            month: *month,
        },
        AnalyticsHeading::Year { year } => AnalyticsPrintHeading::Year {
            text: fmt.style.plain(*year),
        },
        AnalyticsHeading::Item { names } => AnalyticsPrintHeading::Item {
            text: fmt.names(names),
        },
        AnalyticsHeading::OtherItems => AnalyticsPrintHeading::OtherItems,
        AnalyticsHeading::Partner { names } => AnalyticsPrintHeading::Partner {
            text: fmt.names(names),
        },
    })
}

/// The filters of the run as header facts (PRD FR-18.4, FR-19.10): dates and picked records
/// first, then the side, the grouping and the compared period.
pub(crate) fn filter_facts(
    fmt: &Fmt,
    filters: &AnalyticsResolvedFilters,
) -> Result<Vec<PrintFact>, ReportsError> {
    let mut facts = Vec::new();
    if let Some(from) = &filters.from {
        facts.push(fact(ids::FILTER_FROM, fmt.style.date(from)?));
    }
    if let Some(to) = &filters.to {
        facts.push(fact(ids::FILTER_TO, fmt.style.date(to)?));
    }
    if let Some(buyer) = &filters.buyer {
        facts.push(fact(ids::FILTER_BUYER, fmt.party_text(buyer)));
    }
    if let Some(seller) = &filters.seller {
        facts.push(fact(ids::FILTER_SELLER, fmt.party_text(seller)));
    }
    if let Some(item) = &filters.item {
        facts.push(fact(ids::FILTER_ITEM, fmt.names(item)));
    }
    if let Some(city) = &filters.city {
        facts.push(fact(ids::FILTER_CITY, fmt.names(city)));
    }
    if let Some(adda) = &filters.adda {
        facts.push(fact(ids::FILTER_ADDA, fmt.names(adda)));
    }
    if let Some(partner) = &filters.partner {
        facts.push(fact(ids::FILTER_PARTNER, fmt.names(partner)));
    }
    if let Some(trip) = &filters.trip {
        facts.push(fact(ids::FILTER_TRIP, fmt.names(trip)));
    }
    if let Some(garden) = &filters.garden {
        facts.push(fact(ids::FILTER_GARDEN, fmt.names(garden)));
    }
    // The goods and the layout say it all in their label, so their value is empty.
    match filters.goods {
        AnalyticsGoods::All => {}
        AnalyticsGoods::Consigned => facts.push(fact(ids::FILTER_GOODS_CONSIGNED, String::new())),
        AnalyticsGoods::Trip => facts.push(fact(ids::FILTER_GOODS_TRIP, String::new())),
        AnalyticsGoods::Garden => facts.push(fact(ids::FILTER_GOODS_GARDEN, String::new())),
    }
    if let Some(view) = filters.trip_view {
        let label = match view {
            AnalyticsTripView::Trip => ids::FILTER_TRIP_BY_TRIP,
            AnalyticsTripView::Purchase => ids::FILTER_TRIP_BY_PURCHASE,
            AnalyticsTripView::Item => ids::FILTER_TRIP_BY_ITEM,
            AnalyticsTripView::Day => ids::FILTER_TRIP_BY_DAY,
        };
        facts.push(fact(label, String::new()));
    }
    // The side and the grouping say it all in their label, so their value is empty.
    match filters.side {
        AnalyticsSide::Both => {}
        AnalyticsSide::Seller => facts.push(fact(ids::FILTER_SIDE_SELLER, String::new())),
        AnalyticsSide::Buyer => facts.push(fact(ids::FILTER_SIDE_BUYER, String::new())),
    }
    if let Some(group_by) = filters.group_by {
        let label = match group_by {
            AnalyticsGroupBy::Day => ids::FILTER_BY_DAY,
            AnalyticsGroupBy::Month => ids::FILTER_BY_MONTH,
            AnalyticsGroupBy::Year => ids::FILTER_BY_YEAR,
        };
        facts.push(fact(label, String::new()));
    }
    if filters.compare != AnalyticsCompare::Off {
        if let Some(from) = &filters.compare_from {
            facts.push(fact(ids::FILTER_COMPARE_FROM, fmt.style.date(from)?));
        }
        if let Some(to) = &filters.compare_to {
            facts.push(fact(ids::FILTER_COMPARE_TO, fmt.style.date(to)?));
        }
    }
    Ok(facts)
}

/// Builds the print data from a page that holds every row. Pure: it reads nothing else.
///
/// # Errors
/// `ReportsError::Money` for an amount that does not fit; `ReportsError::Data` for a stored date
/// that is not a date.
pub(crate) fn build(
    page: &AnalyticsPage,
    options: &PrintOptions,
) -> Result<AnalyticsPrintData, ReportsError> {
    let fmt = Fmt::new(options);
    let language: Language = options.language;
    let mut columns = Vec::with_capacity(page.columns.len());
    for column in &page.columns {
        columns.push(AnalyticsPrintColumn {
            key: column.key.clone(),
            title_id: column.title_id.clone(),
            kind: column.kind,
            heading: column
                .heading
                .as_ref()
                .map(|h| heading(&fmt, h))
                .transpose()?,
        });
    }
    let mut rows = Vec::with_capacity(page.rows.len());
    for row in &page.rows {
        rows.push(AnalyticsPrintRow {
            cells: fmt.cells(&row.cells)?,
            kind: row.kind,
        });
    }
    Ok(AnalyticsPrintData {
        view: page.view,
        language,
        dir: TextDir::of(language),
        paper: options.paper,
        urdu_digits: fmt.style.urdu_digits(),
        title_id: page.title_id.clone(),
        firm: PrintFirm {
            name: fmt.name(&options.firm.name_ur, &options.firm.name_en),
            mandi: fmt.name(&options.firm.mandi_ur, &options.firm.mandi_en),
            address: options.firm.address.clone(),
            phone: options.firm.phone.clone(),
        },
        filters: filter_facts(&fmt, &page.filters)?,
        printed_at: fmt.style.printed_at(options.now_utc),
        columns,
        rows,
        row_count: fmt.style.count(page.total_count),
        totals_label_id: page.totals_label_id.clone(),
        totals: fmt.cells(&page.totals)?,
        footer_id: ids::PAGE_FOOTER.to_owned(),
    })
}
