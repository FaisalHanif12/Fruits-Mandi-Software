//! The tool `analytics`: a view A to K over a period, with the filters of the analytics screen
//! (PRD FR-7.87, FR-18, Code Standards Q3).
//!
//! It calls `mandi_reports::analytics::run`, the function of the screen. A filter the view does not
//! have is an error. With `compare` each row also carries `earlier` (the figure of the compared
//! period) and `difference` (its size and direction, so the app writes زیادہ or کم), and the result
//! has `compare` with the compared dates and the totals. A monthly series is also given as
//! `series` (the first measure, one value for each month), so the model may show it as a chart.
//!
//! The first column of a table, which names the row, is called `name`; the quantity is `nag`. Those
//! are the words of the prompt (D6-system-prompt Section 5.2); the engine's own keys are used for
//! every other column.
//!
//! Example (worked day): view A for October, rows `i40` (کیلا) and `i41` (سیب), `totals.total_comm`
//! 5,448.

use chrono::{Datelike, NaiveDate};
use mandi_reports::analytics::{
    AnalyticsChartKind, AnalyticsColumn, AnalyticsCompare, AnalyticsFilterKind, AnalyticsFilters,
    AnalyticsGoods, AnalyticsGroupBy, AnalyticsKey, AnalyticsPage, AnalyticsRunInput,
    AnalyticsSide, AnalyticsView, catalog, compare_window, run as engine_run,
};
use mandi_reports::reports::{Cell, Column, PartyFilter, Row};
use mandi_services::core::paging::{SortDirection, SortInput};
use serde::Deserialize;
use serde_json::Value as Json;

use super::cells::{cell_node, compare_cells, is_left_out};
use super::records::named;
use super::report::{DEFAULT_ROWS, MAX_ROWS, OrderWord, SortKey, reports_stop};
use super::teach::{column_words, views_with};
use super::{Draft, Env, PartnerRule, State, Stop, ToolError, date_arg, input, view_rule};
use crate::answer::{
    AnswerChart, AnswerMonth, AnswerRow, AnswerSource, AnswerTable, ChartSeries, ChartSeriesKind,
};
use crate::results::{Node, Shown, Val};
use crate::source::{AnswerRecord, SourceAnalyticsFilters, SourceParty};

/// The most rows read to sort by two keys in memory (the engine's largest page).
const SORT_PAGE: i64 = 1_000;
/// The measure of a comparison: the figure each view compares.
const COMPARED_FIGURE: &str = "total_comm";
/// The compared figure of view C (by buyer).
const COMPARED_FIGURE_BUYER: &str = "purchase_value";

/// The input of `analytics` (D6-system-prompt Section 3).
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Input {
    /// `A` to `K`.
    pub view: String,
    /// A period result (`r2.from`) or `YYYY-MM-DD`.
    pub from: Option<String>,
    /// The last day.
    pub to: Option<String>,
    /// `day`, `month` or `year`.
    pub group_by: Option<String>,
    /// An item.
    pub item: Option<String>,
    /// A seller.
    pub seller: Option<String>,
    /// A buyer.
    pub buyer: Option<String>,
    /// A city.
    pub city: Option<String>,
    /// An اڈا.
    pub adda: Option<String>,
    /// A garden.
    pub garden: Option<String>,
    /// A trip.
    pub trip: Option<String>,
    /// `all`, `consigned`, `trip` or `garden`.
    pub goods: Option<String>,
    /// A partner.
    pub partner: Option<String>,
    /// `both`, `seller` or `buyer`.
    pub side: Option<String>,
    /// `previous` or `last_year`.
    pub compare: Option<String>,
    /// Up to 2 sort keys.
    pub sort: Option<Vec<SortKey>>,
    /// How many rows.
    pub rows: Option<i64>,
}

fn bad(code: &str, field: &str, detail: &str) -> ToolError {
    ToolError::new(code, field, detail)
}

fn takes_or(
    takes: &[AnalyticsFilterKind],
    kind: AnalyticsFilterKind,
    field: &str,
) -> Result<(), ToolError> {
    if takes.contains(&kind) {
        Ok(())
    } else {
        Err(bad(
            "ERR_OUT_OF_RANGE",
            field,
            "this view does not have that filter; the views that do are listed",
        )
        .allowed(views_with(kind)))
    }
}

struct Built {
    filters: AnalyticsFilters,
    source: SourceAnalyticsFilters,
    compare: AnalyticsCompare,
    /// What the tool did that the model did not ask for, in words (plan 52 item 2). It is given
    /// back as `note` so the model can say so.
    notes: Vec<String>,
}

fn build_filters(
    env: &Env<'_>,
    state: &State,
    parsed: &Input,
    takes: &[AnalyticsFilterKind],
) -> Result<Built, ToolError> {
    use AnalyticsFilterKind as K;
    let mut filters = AnalyticsFilters::default();
    let mut source = SourceAnalyticsFilters::default();
    let date = |field: &str, text: &Option<String>| -> Result<Option<String>, ToolError> {
        // A period field with no date (the `from` of `up_to_now`) leaves the date open.
        Ok(text
            .as_deref()
            .map(|t| date_arg(env, state, t, field))
            .transpose()?
            .flatten())
    };
    if let Some(from) = date("from", &parsed.from)? {
        takes_or(takes, K::Period, "from")?;
        filters.from = from.clone();
        source.from = from;
    }
    if let Some(to) = date("to", &parsed.to)? {
        takes_or(takes, K::Period, "to")?;
        filters.to = to.clone();
        source.to = to;
    }
    if let Some(word) = &parsed.group_by {
        takes_or(takes, K::GroupBy, "group_by")?;
        let group = match word.as_str() {
            "day" => AnalyticsGroupBy::Day,
            "month" => AnalyticsGroupBy::Month,
            "year" => AnalyticsGroupBy::Year,
            _ => return Err(bad("ERR_OUT_OF_RANGE", "group_by", "day, month or year")),
        };
        filters.group_by = Some(group);
        source.group_by = Some(group);
    }
    let one = |field: &str,
               text: &Option<String>,
               prefix: &str|
     -> Result<Option<(i64, AnswerRecord)>, ToolError> {
        text.as_deref()
            .map(|t| named(env, state, t, field, &[prefix]))
            .transpose()
    };
    if let Some((id, record)) = one("item", &parsed.item, "i")? {
        takes_or(takes, K::Item, "item")?;
        filters.item_id = Some(id);
        source.item = Some(record);
    }
    if let Some((id, record)) = one("seller", &parsed.seller, "a")? {
        takes_or(takes, K::Seller, "seller")?;
        filters.seller_id = Some(id);
        source.seller = Some(record);
    }
    if let Some((id, record)) = one("buyer", &parsed.buyer, "a")? {
        takes_or(takes, K::Buyer, "buyer")?;
        filters.buyer = Some(PartyFilter {
            account_id: id,
            person_key: None,
        });
        source.buyer = Some(SourceParty {
            record,
            person_key: None,
        });
    }
    if let Some((id, record)) = one("city", &parsed.city, "ct")? {
        takes_or(takes, K::City, "city")?;
        filters.city_id = Some(id);
        source.city = Some(record);
    }
    if let Some((id, record)) = one("adda", &parsed.adda, "ad")? {
        takes_or(takes, K::Adda, "adda")?;
        filters.adda_id = Some(id);
        source.adda = Some(record);
    }
    if let Some((id, record)) = one("garden", &parsed.garden, "g")? {
        takes_or(takes, K::Garden, "garden")?;
        filters.garden_id = Some(id);
        source.garden = Some(record);
    }
    if let Some((id, record)) = one("trip", &parsed.trip, "t")? {
        takes_or(takes, K::Trip, "trip")?;
        filters.trip_id = Some(id);
        source.trip = Some(record);
    }
    if let Some((id, record)) = one("partner", &parsed.partner, "s")? {
        takes_or(takes, K::Partner, "partner")?;
        filters.partner_id = Some(id);
        source.partner = Some(record);
    }
    if let Some(word) = &parsed.goods {
        let goods = match word.as_str() {
            "all" => AnalyticsGoods::All,
            "consigned" => AnalyticsGoods::Consigned,
            "trip" => AnalyticsGoods::Trip,
            "garden" => AnalyticsGoods::Garden,
            _ => {
                return Err(bad(
                    "ERR_OUT_OF_RANGE",
                    "goods",
                    "all, consigned, trip or garden",
                ));
            }
        };
        // "all" is the default every view accepts.
        if goods != AnalyticsGoods::All {
            takes_or(takes, K::Goods, "goods")?;
        }
        filters.goods = goods;
        source.goods = goods;
    }
    if let Some(word) = &parsed.side {
        let side = match word.as_str() {
            "both" => AnalyticsSide::Both,
            "seller" => AnalyticsSide::Seller,
            "buyer" => AnalyticsSide::Buyer,
            _ => return Err(bad("ERR_OUT_OF_RANGE", "side", "both, seller or buyer")),
        };
        if side != AnalyticsSide::Both {
            takes_or(takes, K::Side, "side")?;
        }
        filters.side = side;
        source.side = side;
    }
    let mut compare = AnalyticsCompare::Off;
    let mut notes: Vec<String> = Vec::new();
    if let Some(word) = &parsed.compare {
        compare = match word.as_str() {
            "previous" => AnalyticsCompare::Previous,
            "last_year" => AnalyticsCompare::LastYear,
            _ => return Err(bad("ERR_OUT_OF_RANGE", "compare", "previous or last_year")),
        };
        takes_or(takes, K::Compare, "compare")?;
        // A comparison with no dates at all means this month so far against the same number of
        // days before it, or this year so far against the same dates a year earlier when the
        // question is about years (plan 52 item 2). The dates used are in the note.
        if parsed.from.is_none() && parsed.to.is_none() {
            takes_or(takes, K::Period, "from")?;
            let by_year =
                compare == AnalyticsCompare::LastYear || parsed.group_by.as_deref() == Some("year");
            if let Some((from, to)) = default_window(env, by_year) {
                notes.push(if by_year {
                    format!(
                        "no dates were given, so this year so far ({from} to {to}) is compared \
                         with the same dates one year earlier"
                    )
                } else {
                    format!(
                        "no dates were given, so this month so far ({from} to {to}) is compared \
                         with the same number of days before it"
                    )
                });
                compare = if by_year {
                    AnalyticsCompare::LastYear
                } else {
                    AnalyticsCompare::Previous
                };
                filters.from = from.clone();
                filters.to = to.clone();
                source.from = from;
                source.to = to;
            }
        }
        // A comparison needs a closed period (PRD 18.1): the earlier period is as long as this one.
        if filters.from.is_empty() || filters.to.is_empty() {
            let missing = if filters.from.is_empty() {
                "from"
            } else {
                "to"
            };
            return Err(bad(
                "ERR_REQUIRED",
                missing,
                "compare needs a period with both dates, such as this_year, last_month or \
                 last_n_months. Use a closed period, or leave compare out",
            ));
        }
        // An earlier period that lies wholly before the books has nothing in it: the comparison
        // is left out and the note says so, rather than showing every figure as new. One that
        // starts before the books is only partly covered, so the note says that too (money review,
        // plan 52 item 2).
        let first_day = super::period::books_start(env).map(|d| d.format("%Y-%m-%d").to_string());
        let window = compare_window(compare, &filters.from, &filters.to);
        match (window, first_day) {
            (Some((_, earlier_to)), Some(first)) if earlier_to < first => {
                notes.push(format!(
                    "the books start on {first}, so there is no earlier period to compare with; \
                     the comparison was left out"
                ));
                compare = AnalyticsCompare::Off;
            }
            (Some((earlier_from, earlier_to)), Some(first)) if earlier_from < first => {
                notes.push(format!(
                    "the books start on {first}, so the earlier period ({earlier_from} to \
                     {earlier_to}) is only partly covered and is not like for like"
                ));
                filters.compare = compare;
                source.compare = compare;
            }
            _ => {
                filters.compare = compare;
                source.compare = compare;
            }
        }
    }
    Ok(Built {
        filters,
        source,
        compare,
        notes,
    })
}

/// This month so far, or this year so far (the first day to today), as ISO dates.
fn default_window(env: &Env<'_>, by_year: bool) -> Option<(String, String)> {
    let today = NaiveDate::parse_from_str(&env.today.to_iso(), "%Y-%m-%d").ok()?;
    let from = if by_year {
        NaiveDate::from_ymd_opt(today.year(), 1, 1)?
    } else {
        today.with_day(1)?
    };
    Some((
        from.format("%Y-%m-%d").to_string(),
        today.format("%Y-%m-%d").to_string(),
    ))
}

/// The columns of a view as the plain columns of a table (the heading and the drill are not
/// needed here).
fn plain_columns(columns: &[AnalyticsColumn]) -> Vec<Column> {
    columns
        .iter()
        .map(|c| Column {
            key: c.key.clone(),
            title_id: c.title_id.clone(),
            kind: c.kind,
        })
        .collect()
}

fn shown_key(index: usize, column: &Column) -> String {
    super::cells::shown_key(index, column, true)
}

/// The figure the engine compares in a view: the buyer's purchase value in view C, the total
/// commission in every other.
fn compared_figure(view: AnalyticsView) -> &'static str {
    if view == AnalyticsView::C {
        COMPARED_FIGURE_BUYER
    } else {
        COMPARED_FIGURE
    }
}

/// The engine's key for a key the model uses.
fn engine_key(
    view: AnalyticsView,
    columns: &[Column],
    shown: &str,
    field: &str,
) -> Result<String, ToolError> {
    // The comparison columns are read as `difference.<figure>` and `earlier.<figure>`, and only of
    // the figure the engine compares: `difference.gross` is not the commission change (money
    // review, analytics note b).
    let compared = |figure: &str| figure == compared_figure(view);
    let wanted = shown
        .strip_prefix("difference.")
        .filter(|figure| compared(figure))
        .map(|_| "change")
        .or_else(|| {
            shown
                .strip_prefix("earlier.")
                .filter(|figure| compared(figure))
                .map(|_| "compare")
        });
    if let Some(wanted) = wanted
        && columns.iter().any(|c| c.key == wanted)
    {
        return Ok(wanted.to_owned());
    }
    columns
        .iter()
        .enumerate()
        .find(|(i, c)| shown_key(*i, c) == shown && !is_left_out(c))
        .map(|(_, c)| c.key.clone())
        .ok_or_else(|| {
            bad(
                "ERR_OUT_OF_RANGE",
                field,
                "no such column in this view; use one of the listed words",
            )
            .allowed(column_words(columns, true, Some(compared_figure(view))))
        })
}

/// The key of a row: its record, else its month or day, else its place.
fn key_of(key: &Option<AnalyticsKey>, row: &Row, place: usize, used: &mut Vec<String>) -> String {
    let from_record = key.as_ref().and_then(|k| {
        k.item_id
            .map(|id| format!("i{id}"))
            .or_else(|| k.seller_id.map(|id| format!("a{id}")))
            .or_else(|| k.buyer_id.map(|id| format!("a{id}")))
            .or_else(|| k.partner_id.map(|id| format!("s{id}")))
            .or_else(|| k.guests.filter(|g| *g).map(|_| "guests".to_owned()))
    });
    let from_cells = || {
        row.cells.iter().find_map(|cell| match cell {
            Cell::Month { year, month } => Some(format!("m{year}{month:02}")),
            Cell::Party { party } => Some(format!("a{}", party.account_id)),
            Cell::Date { iso } => Some(format!("d{}", iso.replace('-', ""))),
            _ => None,
        })
    };
    let base = from_record
        .or_else(from_cells)
        .unwrap_or_else(|| format!("x{}", place.saturating_add(1)));
    let base = if row.kind == mandi_reports::reports::RowKind::Normal {
        base
    } else {
        format!("h{}", place.saturating_add(1))
    };
    let mut candidate = base.clone();
    let mut copy: usize = 1;
    while used.contains(&candidate) {
        copy = copy.saturating_add(1);
        candidate = format!("{base}_{copy}");
    }
    used.push(candidate.clone());
    candidate
}

/// The compared period as the model reads it: its dates and, when it is exactly one month, the
/// month.
fn compare_node(
    page: &AnalyticsPage,
    totals_earlier: Option<Node>,
    totals_diff: Option<Node>,
    figure: &str,
) -> Node {
    // The compared dates are the engine's own, the ones its `earlier` figures were worked out
    // for: never a second calculation of the same window (money review, plan 51 B2).
    let (Some(from), Some(to)) = (&page.filters.compare_from, &page.filters.compare_to) else {
        return Node::Null;
    };
    let parse = |text: &str| NaiveDate::parse_from_str(text, "%Y-%m-%d").ok();
    let (Some(cf), Some(ct)) = (parse(from), parse(to)) else {
        return Node::Null;
    };
    let mut fields = vec![
        (
            "from",
            Node::Val(Val::Date(cf.format("%Y-%m-%d").to_string())),
        ),
        (
            "to",
            Node::Val(Val::Date(ct.format("%Y-%m-%d").to_string())),
        ),
    ];
    let one_month = cf.day() == 1
        && ct.month() == cf.month()
        && ct.year() == cf.year()
        && ct.succ_opt().is_some_and(|next| next.day() == 1);
    if one_month {
        fields.push((
            "month",
            Node::Val(Val::Month {
                year: cf.year(),
                month: cf.month(),
            }),
        ));
    }
    if let Some(node) = totals_earlier {
        fields.push(("totals", Node::obj(vec![(figure, node)])));
    }
    if let Some(node) = totals_diff {
        fields.push(("difference", Node::obj(vec![(figure, node)])));
    }
    Node::obj(fields)
}

/// The first measure of a monthly trend chart, as a series of months.
fn series_of(page: &AnalyticsPage) -> Option<(Node, AnswerChart)> {
    let chart = page
        .charts
        .iter()
        .find(|c| c.kind == AnalyticsChartKind::Trend)?;
    let series = chart.series.first()?;
    let mut months = Vec::new();
    let mut values = Vec::new();
    let mut nodes = Vec::new();
    for point in &chart.points {
        let Cell::Month { year, month } = &point.label else {
            return None;
        };
        let value = point.values.first()?.clone();
        nodes.push(Node::obj(vec![
            ("key", Node::Str(format!("m{year}{month:02}"))),
            (
                "month",
                Node::Val(Val::Month {
                    year: i32::try_from(*year).ok()?,
                    month: u32::try_from(*month).ok()?,
                }),
            ),
        ]));
        months.push(AnswerMonth {
            year: *year,
            month: *month,
        });
        values.push(value);
    }
    if months.is_empty() {
        return None;
    }
    let measure_id = page
        .columns
        .iter()
        .find(|c| c.key == series.key)
        .map(|c| c.title_id.clone())?;
    Some((
        Node::List(nodes),
        AnswerChart {
            result_id: String::new(),
            measure_id,
            months,
            series: vec![ChartSeries {
                kind: ChartSeriesKind::Actual,
                values,
            }],
            is_estimate: false,
        },
    ))
}

/// The sort asked of the engine: one key, in the engine's own column name.
fn engine_sort(
    env: &Env<'_>,
    view: AnalyticsView,
    columns: &[Column],
    key: &SortKey,
) -> Result<SortInput, ToolError> {
    Ok(SortInput {
        column: engine_key(view, columns, &key.column, "sort.column")?,
        direction: match key.order {
            OrderWord::Asc => SortDirection::Asc,
            OrderWord::Desc => SortDirection::Desc,
        },
        language: Some(env.language),
    })
}

fn read(
    env: &Env<'_>,
    view: AnalyticsView,
    filters: &AnalyticsFilters,
    limit: i64,
    sort: Option<SortInput>,
) -> Result<AnalyticsPage, Stop> {
    engine_run(
        env.ctx.db(),
        &AnalyticsRunInput {
            view,
            filters: filters.clone(),
            offset: 0,
            limit,
            sort,
        },
    )
    .map_err(reports_stop)
}

/// Sorts the rows of one page by up to two keys: the second key first, then the first, each a
/// stable sort (a difference sorts by its signed value, so "down" is below zero).
fn sort_in_memory(
    env: &Env<'_>,
    view: AnalyticsView,
    page: &AnalyticsPage,
    keys: &[SortKey],
) -> Result<Vec<mandi_reports::analytics::AnalyticsRow>, ToolError> {
    let ur = matches!(env.language, mandi_services::core::auth::Language::Ur);
    let mut spec = Vec::new();
    for key in keys {
        let real = engine_key(
            view,
            &plain_columns(&page.columns),
            &key.column,
            "sort.column",
        )?;
        let index = page
            .columns
            .iter()
            .position(|c| c.key == real)
            .ok_or_else(|| bad("ERR_OUT_OF_RANGE", "sort.column", "no such column"))?;
        // A balance column that holds both sides (a buyer who owes and a buyer who is owed) is
        // ranked by size here, which would put a credit among the biggest dues (money review).
        if super::cells::mixed_sides(page.rows.iter().filter_map(|r| r.cells.get(index))) {
            return Err(bad(
                "ERR_OUT_OF_RANGE",
                "sort.column",
                "this column holds both sides; sort on another column",
            ));
        }
        spec.push((index, key.order));
    }
    let mut rows = page.rows.clone();
    for (index, order) in spec.iter().rev() {
        rows.sort_by(|a, b| {
            let ordering = match (a.cells.get(*index), b.cells.get(*index)) {
                (Some(l), Some(r)) => compare_cells(l, r, ur),
                _ => std::cmp::Ordering::Equal,
            };
            match order {
                OrderWord::Asc => ordering,
                OrderWord::Desc => ordering.reverse(),
            }
        });
    }
    Ok(rows)
}

/// The tool.
pub fn run(env: &Env<'_>, state: &State, args: &Json) -> Result<Draft, Stop> {
    let parsed: Input = input(args)?;
    let view: AnalyticsView = serde_json::from_value(Json::String(parsed.view.clone()))
        .map_err(|_| bad("ERR_OUT_OF_RANGE", "view", "A to K"))?;
    let entry = catalog()
        .into_iter()
        .find(|e| e.id == view)
        .ok_or_else(|| bad("ERR_OUT_OF_RANGE", "view", "A to K"))?;
    let rows_wanted = parsed.rows.unwrap_or(DEFAULT_ROWS);
    if !(1..=MAX_ROWS).contains(&rows_wanted) {
        return Err(bad("ERR_OUT_OF_RANGE", "rows", "1 to 100").into());
    }
    let keys = parsed.sort.clone().unwrap_or_default();
    if keys.len() > 2 {
        return Err(bad("ERR_OUT_OF_RANGE", "sort", "up to 2 keys").into());
    }
    let mut built = build_filters(env, state, &parsed, &entry.filters)?;
    let mut hidden_parts = false;
    let rule = view_rule(view, &built.filters);
    let partner_data = rule != PartnerRule::Free && !env.partner_locked;
    match rule {
        PartnerRule::Pause if env.partner_locked => return Err(Stop::Locked),
        PartnerRule::HideParts if env.partner_locked => {
            built.filters.hide_partners = true;
            hidden_parts = true;
        }
        _ => {}
    }
    let (page, rows) = if keys.is_empty() {
        let page = read(env, view, &built.filters, rows_wanted, None)?;
        let rows = page.rows.clone();
        (page, rows)
    } else {
        // The columns are known from a first read of up to one page of rows.
        let first = read(env, view, &built.filters, SORT_PAGE, None)?;
        // A sort word the view has no column for is left out, and the note says so (plan 52 item
        // 2). The rows then come in the view's own order, so the tool gives every row (up to 100)
        // and never a cut that looks like a top list; a longer table still needs a real column.
        let columns = plain_columns(&first.columns);
        let (known, dropped): (Vec<SortKey>, Vec<SortKey>) = keys
            .iter()
            .cloned()
            .partition(|k| engine_key(view, &columns, &k.column, "sort.column").is_ok());
        if let Some(first_dropped) = dropped.first() {
            if known.is_empty() && first.total_count > MAX_ROWS {
                return Err(
                    engine_key(view, &columns, &first_dropped.column, "sort.column")
                        .err()
                        .unwrap_or_else(|| bad("ERR_OUT_OF_RANGE", "sort.column", "no such column"))
                        .into(),
                );
            }
            let words: Vec<&str> = dropped.iter().map(|k| k.column.as_str()).collect();
            built.notes.push(format!(
                "the sort word {} is not a column of this view, so it was left out{}; the sort \
                 words of this view are {}",
                words.join(", "),
                if known.is_empty() {
                    ", every row is given in the view's own order"
                } else {
                    ""
                },
                column_words(&columns, true, Some(compared_figure(view))).join(", ")
            ));
        }
        if known.is_empty() && !dropped.is_empty() {
            let take = usize::try_from(MAX_ROWS).unwrap_or(0);
            let rows = first.rows.iter().take(take).cloned().collect();
            (first, rows)
        } else {
            let keys = known;
            let flat = first
                .rows
                .iter()
                .all(|r| r.kind == mandi_reports::reports::RowKind::Normal);
            if first.total_count <= SORT_PAGE && flat {
                let sorted = sort_in_memory(env, view, &first, &keys)?;
                let take = usize::try_from(rows_wanted).unwrap_or(0);
                let rows = sorted.into_iter().take(take).collect();
                (first, rows)
            } else if let [key] = keys.as_slice() {
                let sort = engine_sort(env, view, &plain_columns(&first.columns), key)?;
                let page = read(env, view, &built.filters, rows_wanted, Some(sort))?;
                let rows = page.rows.clone();
                (page, rows)
            } else {
                return Err(bad(
                    "ERR_OUT_OF_RANGE",
                    "sort",
                    "two sort keys need a table of at most 1,000 rows",
                )
                .into());
            }
        }
    };
    let mut draft = assemble(env, args, view, &built, &page, &rows, hidden_parts);
    draft.partner_data = partner_data;
    Ok(draft)
}

/// The row as a node: its cells by column key, and with a comparison its `earlier` and
/// `difference`.
fn row_fields(
    env: &Env<'_>,
    columns: &[Column],
    cells: &[Cell],
    compare_at: Option<(usize, usize)>,
    figure: &str,
) -> Vec<(String, Node)> {
    let mut fields: Vec<(String, Node)> = Vec::new();
    for (i, column) in columns.iter().enumerate() {
        let is_compare = compare_at.is_some_and(|(a, b)| i == a || i == b);
        if is_left_out(column) || is_compare {
            continue;
        }
        if let Some(cell) = cells.get(i) {
            fields.push((shown_key(i, column), cell_node(env, cell, false)));
        }
    }
    if let Some((earlier, change)) = compare_at {
        if let Some(cell) = cells.get(earlier) {
            fields.push((
                "earlier".to_owned(),
                Node::obj(vec![(figure, cell_node(env, cell, false))]),
            ));
        }
        if let Some(cell) = cells.get(change) {
            fields.push((
                "difference".to_owned(),
                Node::obj(vec![(figure, cell_node(env, cell, true))]),
            ));
        }
    }
    fields
}

fn assemble(
    env: &Env<'_>,
    args: &Json,
    view: AnalyticsView,
    built: &Built,
    page: &AnalyticsPage,
    rows: &[mandi_reports::analytics::AnalyticsRow],
    hidden_parts: bool,
) -> Draft {
    let compare_at = if built.compare == AnalyticsCompare::Off {
        None
    } else {
        let at = |key: &str| page.columns.iter().position(|c| c.key == key);
        at("compare").zip(at("change"))
    };
    let figure = compared_figure(view);
    let columns = plain_columns(&page.columns);
    let column_keys: Vec<Node> = columns
        .iter()
        .enumerate()
        .filter(|(i, c)| !is_left_out(c) && !compare_at.is_some_and(|(a, b)| *i == a || *i == b))
        .map(|(i, c)| Node::Str(shown_key(i, c)))
        .collect();
    let mut used = Vec::new();
    let keyed: Vec<(String, Node)> = rows
        .iter()
        .enumerate()
        .map(|(place, row)| {
            let key = key_of(
                &row.key,
                &Row {
                    cells: row.cells.clone(),
                    source: None,
                    kind: row.kind,
                },
                place,
                &mut used,
            );
            (
                key,
                Node::Obj(row_fields(env, &columns, &row.cells, compare_at, figure)),
            )
        })
        .collect();
    let totals_fields = row_fields(env, &columns, &page.totals, None, figure);
    let totals: Vec<(String, Node)> = totals_fields
        .into_iter()
        .filter(|(_, node)| !matches!(node, Node::Null))
        .collect();
    let compare = compare_at.map_or(Node::Null, |(earlier, change)| {
        compare_node(
            page,
            page.totals.get(earlier).map(|c| cell_node(env, c, false)),
            page.totals.get(change).map(|c| cell_node(env, c, true)),
            figure,
        )
    });
    let chart = series_of(page);
    let mut echo = Vec::new();
    for (name, value) in [
        ("from", page.filters.from.clone()),
        ("to", page.filters.to.clone()),
    ] {
        if let Some(value) = value {
            echo.push((name, Node::Str(value)));
        }
    }
    let series = chart
        .as_ref()
        .map_or(Node::Null, |(_, c)| series_node(page, c));
    let mut fields = vec![
        ("source", Node::Str(view.code().to_owned())),
        (
            "title",
            Node::Str(env.words.text(env.language, &page.title_id)),
        ),
        ("filters", Node::obj(echo)),
        ("columns", Node::List(column_keys)),
        ("rows", Node::Keyed(keyed)),
        ("count", Node::Val(Val::Count(page.total_count))),
        ("rows_cut", Node::Bool(false)),
        ("totals", Node::Obj(totals)),
        ("compare", compare),
        ("series", series),
    ];
    // What the tool did that was not asked for (plan 52 item 2).
    if !built.notes.is_empty() {
        fields.push(("note", Node::Str(built.notes.join("; "))));
    }
    let node = Node::obj(fields);
    let mut draft = Draft::new("analytics", args.clone(), node);
    let table = AnswerTable {
        result_id: String::new(),
        title_id: page.title_id.clone(),
        columns: columns.clone(),
        rows: rows
            .iter()
            .map(|row| AnswerRow {
                row: Row {
                    cells: row.cells.clone(),
                    source: None,
                    kind: row.kind,
                },
                analytics_key: row.key.clone(),
            })
            .collect(),
        count: page.total_count,
        totals_label_id: page.totals_label_id.clone(),
        totals: page.totals.clone(),
    };
    draft.shown = Some(match chart {
        Some((_, chart)) => Shown::TableChart(table, Box::new(chart)),
        None => Shown::Table(table),
    });
    draft.source = Some(AnswerSource::Analytics {
        result_id: String::new(),
        label_id: page.title_id.clone(),
        view,
        filters: built.source.clone(),
    });
    draft.partner_parts_locked = hidden_parts;
    draft
}

/// The series node: one value for each month, from the same trend chart the table draws.
fn series_node(page: &AnalyticsPage, chart: &AnswerChart) -> Node {
    let Some(trend) = page
        .charts
        .iter()
        .find(|c| c.kind == AnalyticsChartKind::Trend)
    else {
        return Node::Null;
    };
    let _ = chart;
    let items = trend
        .points
        .iter()
        .filter_map(|point| {
            let Cell::Month { year, month } = &point.label else {
                return None;
            };
            let value = point.values.first()?;
            let money = match value {
                Cell::Money { paisa } => {
                    Node::Val(Val::Money(crate::results::Money::plain(*paisa)))
                }
                Cell::Number { value } => Node::Val(Val::Count(*value)),
                _ => Node::Null,
            };
            Some(Node::obj(vec![
                ("key", Node::Str(format!("m{year}{month:02}"))),
                (
                    "month",
                    Node::Val(Val::Month {
                        year: i32::try_from(*year).ok()?,
                        month: u32::try_from(*month).ok()?,
                    }),
                ),
                ("value", money),
            ]))
        })
        .collect();
    Node::List(items)
}
