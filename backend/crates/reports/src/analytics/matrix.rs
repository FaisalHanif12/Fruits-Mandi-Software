//! View D, item by period (قسم مال × مہینہ), and view E, seller by item (بیوپاری × قسم مال)
//! (PRD 18.2, FR-18.1).
//!
//! Both are matrices of one measure (PRD FR-18.10, plan 84): the total commission (seller
//! commission + buyer commission + لاگا, or the side asked for) when none is asked for, or the
//! sales, the buyers' bills, the quantity or one commission. A row has one cell for each column and
//! a row total; the totals row adds each column, and the row total column is named by the measure
//! (`total_comm`, `gross`, `qty` ...). Every cell opens the lines behind it: the key of the row
//! and the `drill` of the column together (FR-18.2).
//!
//! - **D** has one column for each day, month or year that has sales (`group_by`, month when
//!   left out), at most 144 (twelve years by month, plan 80 gate: it was 36, which refused 5 years).
//! - **E** has one column for each item that has sales, at most 24. When there are more, the
//!   items with the most commission keep their column and all the others are added in one
//!   "other items" column.
//!
//! Example (PRD 18.4, October 2026): D has the column 2026-10 with کیلا 3,928 and سیب 1,520, total
//! 5,448; E has رحیم × کیلا 3,928 and کریم × سیب 1,520. By خام بکری D has کیلا 29,400 and سیب
//! 12,000, total 41,400; by نگ 50, 10 and 60.

use std::collections::{BTreeMap, HashMap};

use mandi_core::dates::BusinessDate;
use mandi_core::error::{ErrorCode, FieldError};
use mandi_db::rusqlite::Connection;

use super::facts::{self, Group, compare_range, run_range};
use super::filters::Typed;
use super::ids;
use super::measure::{
    Figure, add, compare_columns_of, compare_figure_cells, figures_by_id, measure_of,
    row_total_column,
};
use super::model::{
    AnalyticsColumn, AnalyticsGroupBy, AnalyticsHeading, AnalyticsKey, AnalyticsMeasure,
    AnalyticsRow,
};
use super::table::{Table, col, count_of, dynamic_col, item_rows, page_window, row};
use crate::error::ReportsError;
use crate::reports::PageSpec;
use crate::reports::common::load_parties;
use crate::reports::{Cell, ColumnKind, Names};

/// Most columns of view D: 144 periods, twelve years by month (PRD 18.2: one column for each month; the
/// cap of 36 refused five years of months, plan 80 gate).
const MAX_PERIOD_COLUMNS: usize = 144;
/// Most columns of view E, the "other items" column included.
const MAX_ITEM_COLUMNS: usize = 24;

/// The SQL of the period of a fact for the grouping chosen.
fn bucket_sql(group_by: AnalyticsGroupBy) -> &'static str {
    match group_by {
        AnalyticsGroupBy::Day => "f.entry_date",
        AnalyticsGroupBy::Month => "substr(f.entry_date, 1, 7)",
        AnalyticsGroupBy::Year => "substr(f.entry_date, 1, 4)",
    }
}

/// A column for a period bucket (`2026-10-10`, `2026-10` or `2026`): its heading and the days it
/// covers.
pub(super) fn period_column(
    group_by: AnalyticsGroupBy,
    bucket: &str,
) -> Result<AnalyticsColumn, ReportsError> {
    let bad = || ReportsError::Data(format!("a period bucket is wrong: {bucket}"));
    let number = |text: &str| text.parse::<i64>().map_err(|_| bad());
    let (heading, from, to) = match group_by {
        AnalyticsGroupBy::Day => (
            AnalyticsHeading::Day {
                iso: bucket.to_owned(),
            },
            bucket.to_owned(),
            bucket.to_owned(),
        ),
        AnalyticsGroupBy::Month => {
            let (year, month) = bucket.split_once('-').ok_or_else(bad)?;
            let (year, month) = (number(year)?, number(month)?);
            let first = BusinessDate::from_ymd(
                i32::try_from(year).map_err(|_| bad())?,
                u32::try_from(month).map_err(|_| bad())?,
                1,
            )
            .ok_or_else(bad)?;
            let next = if month == 12 {
                BusinessDate::from_ymd(
                    i32::try_from(year)
                        .map_err(|_| bad())?
                        .checked_add(1)
                        .ok_or_else(bad)?,
                    1,
                    1,
                )
            } else {
                BusinessDate::from_ymd(
                    i32::try_from(year).map_err(|_| bad())?,
                    u32::try_from(month)
                        .map_err(|_| bad())?
                        .checked_add(1)
                        .ok_or_else(bad)?,
                    1,
                )
            }
            .ok_or_else(bad)?;
            let last = next.previous_day().ok_or_else(bad)?;
            (
                AnalyticsHeading::Month { year, month },
                first.to_iso(),
                last.to_iso(),
            )
        }
        AnalyticsGroupBy::Year => {
            let year = number(bucket)?;
            (
                AnalyticsHeading::Year { year },
                format!("{year:04}-01-01"),
                format!("{year:04}-12-31"),
            )
        }
    };
    Ok(dynamic_col(
        format!("p_{}", bucket.replace('-', "_")),
        ids::COL_PERIOD,
        heading,
        AnalyticsKey {
            from: Some(from),
            to: Some(to),
            ..AnalyticsKey::default()
        },
    ))
}

/// A column of a matrix, of the kind of the measure: money, or a whole number for the quantity.
fn of_kind(mut column: AnalyticsColumn, measure: AnalyticsMeasure) -> AnalyticsColumn {
    column.kind = measure.kind();
    column
}

/// The number of days, months or years a closed range spans, by the grouping of view D. `None` when
/// the range has no first or no last day (then only the facts can say how many periods there are).
fn periods_in_range(typed: &Typed) -> Option<usize> {
    let from = BusinessDate::parse_iso(typed.from.as_deref()?)?;
    let to = BusinessDate::parse_iso(typed.to.as_deref()?)?;
    let count = match typed.group_by {
        AnalyticsGroupBy::Day => to.days_after(from).checked_add(1)?,
        AnalyticsGroupBy::Month => i64::from(to.year())
            .checked_sub(i64::from(from.year()))?
            .checked_mul(12)?
            .checked_add(i64::from(to.month()))?
            .checked_sub(i64::from(from.month()))?
            .checked_add(1)?,
        AnalyticsGroupBy::Year => i64::from(to.year())
            .checked_sub(i64::from(from.year()))?
            .checked_add(1)?,
    };
    usize::try_from(count).ok()
}

/// Builds view D.
///
/// # Errors
/// `ReportsError::Rejected` (`ERR_OUT_OF_RANGE`, `filters.group_by`) when there would be more
/// than 144 columns; a database error; `ReportsError::Data` for a fact net below 0.
pub(crate) fn run_d(
    conn: &Connection,
    typed: &Typed,
    spec: PageSpec,
) -> Result<Table, ReportsError> {
    let measure = measure_of(typed);
    // A closed range of more than 144 periods is refused from the dates, before any fact is read
    // (ten years by month used to be read in full and refused after, two seconds later).
    if periods_in_range(typed).is_some_and(|n| n > MAX_PERIOD_COLUMNS) {
        return Err(ReportsError::Rejected(vec![FieldError::new(
            ErrorCode::ErrOutOfRange,
            "filters.group_by",
        )]));
    }
    let none = AnalyticsKey::default();
    let current = facts::groups(
        conn,
        &facts::conds(typed, run_range(typed), &none),
        &["f.item_id", bucket_sql(typed.group_by)],
    )?;
    let compared = if typed.compare.is_some() {
        figures_by_id(
            &facts::groups(
                conn,
                &facts::conds(typed, compare_range(typed), &none),
                &["f.item_id"],
            )?,
            measure,
            typed.side,
        )?
    } else {
        HashMap::new()
    };

    // Buckets in date order; figures by (item, bucket).
    let mut buckets: BTreeMap<String, ()> = BTreeMap::new();
    let mut cells: HashMap<(i64, String), Figure> = HashMap::new();
    for group in &current {
        let (Some(item), Some(bucket)) = (group.int(0), group.text(1)) else {
            continue;
        };
        buckets.insert(bucket.clone(), ());
        cells.insert((item, bucket), group.sums.figure(measure, typed.side)?);
    }
    if buckets.len() > MAX_PERIOD_COLUMNS {
        return Err(ReportsError::Rejected(vec![FieldError::new(
            ErrorCode::ErrOutOfRange,
            "filters.group_by",
        )]));
    }
    let bucket_list: Vec<String> = buckets.into_keys().collect();

    // Items with sales, and items only in the compared period.
    let mut item_ids: Vec<i64> = current.iter().filter_map(|g| g.int(0)).collect();
    item_ids.sort_unstable();
    item_ids.dedup();
    let mut all_ids = item_ids.clone();
    all_ids.extend(compared.keys().copied());
    let items = item_rows(conn, &all_ids)?;

    let zero = Figure::zero(measure);
    struct Entry {
        id: i64,
        total: Figure,
        compared: Figure,
    }
    let mut entries: Vec<Entry> = Vec::new();
    for id in &item_ids {
        let mut total = zero;
        for bucket in &bucket_list {
            if let Some(amount) = cells.get(&(*id, bucket.clone())) {
                add(&mut total, *amount)?;
            }
        }
        entries.push(Entry {
            id: *id,
            total,
            compared: compared.get(id).copied().unwrap_or(zero),
        });
    }
    for (id, amount) in &compared {
        if !item_ids.contains(id) {
            entries.push(Entry {
                id: *id,
                total: zero,
                compared: *amount,
            });
        }
    }
    let code_of = |id: i64| items.get(&id).map(|i| i.code.clone()).unwrap_or_default();
    entries.sort_by(|a, b| {
        b.total
            .cmp(&a.total)
            .then_with(|| code_of(a.id).cmp(&code_of(b.id)))
            .then_with(|| a.id.cmp(&b.id))
    });

    let mut columns = vec![col("item", ids::COL_ITEM, ColumnKind::Names)];
    for bucket in &bucket_list {
        columns.push(of_kind(period_column(typed.group_by, bucket)?, measure));
    }
    columns.push(row_total_column(measure));
    let with_compare = typed.compare.is_some();
    if with_compare {
        columns.extend(compare_columns_of(measure));
    }

    let mut all_rows: Vec<AnalyticsRow> = Vec::with_capacity(entries.len());
    let mut column_totals = vec![zero; bucket_list.len()];
    let mut grand = zero;
    let mut grand_compared = zero;
    for entry in &entries {
        let mut out = vec![items.get(&entry.id).map_or(Cell::Empty, |i| Cell::Names {
            names: i.names.clone(),
        })];
        for (index, bucket) in bucket_list.iter().enumerate() {
            let amount = cells
                .get(&(entry.id, bucket.clone()))
                .copied()
                .unwrap_or(zero);
            if let Some(slot) = column_totals.get_mut(index) {
                add(slot, amount)?;
            }
            out.push(amount.cell_or_empty());
        }
        out.push(entry.total.cell());
        if with_compare {
            out.extend(compare_figure_cells(entry.total, entry.compared)?);
        }
        add(&mut grand, entry.total)?;
        add(&mut grand_compared, entry.compared)?;
        all_rows.push(row(
            out,
            AnalyticsKey {
                item_id: Some(entry.id),
                ..AnalyticsKey::default()
            },
        ));
    }
    let mut totals = vec![Cell::Empty];
    totals.extend(column_totals.into_iter().map(Figure::cell_or_empty));
    totals.push(grand.cell());
    if with_compare {
        totals.extend(compare_figure_cells(grand, grand_compared)?);
    }

    Ok(Table {
        charts: Vec::new(),
        columns,
        total_count: count_of(all_rows.len()),
        rows: spec.slice(&all_rows),
        totals,
    })
}

/// Builds view E.
///
/// # Errors
/// A database error; `ReportsError::Data` for a fact net below 0; `ReportsError::Money`.
pub(crate) fn run_e(
    conn: &Connection,
    typed: &Typed,
    spec: PageSpec,
) -> Result<Table, ReportsError> {
    let measure = measure_of(typed);
    let zero = Figure::zero(measure);
    let none = AnalyticsKey::default();
    let current: Vec<Group> = facts::groups(
        conn,
        &facts::conds(typed, run_range(typed), &none),
        &["f.seller_account_id", "f.item_id"],
    )?;
    let compared = if typed.compare.is_some() {
        figures_by_id(
            &facts::groups(
                conn,
                &facts::conds(typed, compare_range(typed), &none),
                &["f.seller_account_id"],
            )?,
            measure,
            typed.side,
        )?
    } else {
        HashMap::new()
    };

    let mut amounts: HashMap<(i64, i64), Figure> = HashMap::new();
    let mut by_item: HashMap<i64, Figure> = HashMap::new();
    let mut by_seller: HashMap<i64, Figure> = HashMap::new();
    for group in &current {
        let (Some(seller), Some(item)) = (group.int(0), group.int(1)) else {
            continue;
        };
        let amount = group.sums.figure(measure, typed.side)?;
        amounts.insert((seller, item), amount);
        add(by_item.entry(item).or_insert(zero), amount)?;
        add(by_seller.entry(seller).or_insert(zero), amount)?;
    }
    let mut sellers: Vec<i64> = by_seller.keys().copied().collect();
    sellers.sort_unstable();

    // The items that get a column: all, or the biggest ones and one column for the rest.
    let mut item_order: Vec<i64> = by_item.keys().copied().collect();
    let item_info = item_rows(conn, &item_order)?;
    let item_code = |id: i64| {
        item_info
            .get(&id)
            .map(|i| i.code.clone())
            .unwrap_or_default()
    };
    item_order.sort_by(|a, b| {
        by_item
            .get(b)
            .cmp(&by_item.get(a))
            .then_with(|| item_code(*a).cmp(&item_code(*b)))
            .then_with(|| a.cmp(b))
    });
    let overflow = item_order.len() > MAX_ITEM_COLUMNS && !typed.every_item_column;
    let shown: Vec<i64> = if overflow {
        item_order
            .iter()
            .take(MAX_ITEM_COLUMNS.saturating_sub(1))
            .copied()
            .collect()
    } else {
        item_order.clone()
    };
    let mut shown_sorted = shown.clone();
    shown_sorted.sort_by(|a, b| item_code(*a).cmp(&item_code(*b)).then_with(|| a.cmp(b)));

    let mut all_ids = sellers.clone();
    all_ids.extend(compared.keys().copied());
    let parties = load_parties(conn, &all_ids)?;

    struct Entry {
        id: i64,
        total: Figure,
        compared: Figure,
    }
    let mut entries: Vec<Entry> = Vec::new();
    for seller in &sellers {
        entries.push(Entry {
            id: *seller,
            total: by_seller.get(seller).copied().unwrap_or(zero),
            compared: compared.get(seller).copied().unwrap_or(zero),
        });
    }
    for (id, amount) in &compared {
        if !sellers.contains(id) {
            entries.push(Entry {
                id: *id,
                total: zero,
                compared: *amount,
            });
        }
    }
    let code_of = |id: i64| parties.get(&id).map(|p| p.code.clone()).unwrap_or_default();
    entries.sort_by(|a, b| {
        b.total
            .cmp(&a.total)
            .then_with(|| code_of(a.id).cmp(&code_of(b.id)))
            .then_with(|| a.id.cmp(&b.id))
    });

    let mut columns = vec![col("seller", ids::COL_SELLER, ColumnKind::Party)];
    for item in &shown_sorted {
        let names = item_info.get(item).map_or_else(
            || Names {
                ur: String::new(),
                en: String::new(),
            },
            |i| i.names.clone(),
        );
        columns.push(of_kind(
            dynamic_col(
                format!("i_{item}"),
                ids::COL_ITEM_COLUMN,
                AnalyticsHeading::Item { names },
                AnalyticsKey {
                    item_id: Some(*item),
                    ..AnalyticsKey::default()
                },
            ),
            measure,
        ));
    }
    if overflow {
        columns.push(of_kind(
            dynamic_col(
                "i_other".to_owned(),
                ids::COL_ITEM_COLUMN,
                AnalyticsHeading::OtherItems,
                AnalyticsKey {
                    not_item_ids: shown_sorted.clone(),
                    ..AnalyticsKey::default()
                },
            ),
            measure,
        ));
    }
    columns.push(row_total_column(measure));
    let with_compare = typed.compare.is_some();
    if with_compare {
        columns.extend(compare_columns_of(measure));
    }

    // What each seller has in the items without a column of their own.
    let mut rest_by_seller: HashMap<i64, Figure> = HashMap::new();
    if overflow {
        for ((seller, item), amount) in &amounts {
            if !shown_sorted.contains(item) {
                add(rest_by_seller.entry(*seller).or_insert(zero), *amount)?;
            }
        }
    }
    let width = shown_sorted.len() + usize::from(overflow);
    // The totals row is over every seller, read from the sums themselves: one pass over them, not one
    // lookup for each seller and item. The rows are built for the page only (a ten-year view has
    // thousands of sellers and a page has a hundred).
    let mut column_totals = vec![zero; width];
    let position: HashMap<i64, usize> = shown_sorted
        .iter()
        .enumerate()
        .map(|(index, item)| (*item, index))
        .collect();
    for ((_, item), amount) in &amounts {
        let at = position
            .get(item)
            .copied()
            .or(overflow.then_some(shown_sorted.len()));
        if let Some(slot) = at.and_then(|i| column_totals.get_mut(i)) {
            add(slot, *amount)?;
        }
    }
    let mut grand = zero;
    let mut grand_compared = zero;
    for entry in &entries {
        add(&mut grand, entry.total)?;
        add(&mut grand_compared, entry.compared)?;
    }
    let (skip, take) = page_window(spec);
    let mut all_rows: Vec<AnalyticsRow> = Vec::new();
    for entry in entries.iter().skip(skip).take(take) {
        let mut out = vec![
            parties
                .get(&entry.id)
                .map_or(Cell::Empty, |p| Cell::Party { party: p.clone() }),
        ];
        for item in &shown_sorted {
            let amount = amounts.get(&(entry.id, *item)).copied().unwrap_or(zero);
            out.push(amount.cell_or_empty());
        }
        if overflow {
            let rest = rest_by_seller.get(&entry.id).copied().unwrap_or(zero);
            out.push(rest.cell_or_empty());
        }
        out.push(entry.total.cell());
        if with_compare {
            out.extend(compare_figure_cells(entry.total, entry.compared)?);
        }
        all_rows.push(row(
            out,
            AnalyticsKey {
                seller_id: Some(entry.id),
                ..AnalyticsKey::default()
            },
        ));
    }
    let mut totals = vec![Cell::Empty];
    totals.extend(column_totals.into_iter().map(Figure::cell_or_empty));
    totals.push(grand.cell());
    if with_compare {
        totals.extend(compare_figure_cells(grand, grand_compared)?);
    }

    Ok(Table {
        charts: Vec::new(),
        columns,
        total_count: count_of(entries.len()),
        rows: all_rows,
        totals,
    })
}
