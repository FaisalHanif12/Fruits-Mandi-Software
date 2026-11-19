//! View L, buyer by item, گاہک × قسم مال (PRD 18.2, FR-18.1, FR-18.9, FR-13.7, plan 77).
//!
//! The mirror of view E: one row for each buyer (and each چکتا buyer inside the چکتا کھاتہ
//! account) with purchases in the filter, one column for each item, and in each cell the buyer's
//! total commission: his buyer commission plus لاگا, the two charges he pays. The seller side is
//! not his, so the commission side filter is not taken. A row has its total, the totals row adds
//! each column, and every cell opens the lines behind it (FR-18.2): the key of the row and the
//! `drill` of the column together.
//!
//! There is one column for each item that has sales, at most 24. When there are more, the items
//! with the most commission keep their column and all the others are added in one "other items"
//! column.
//!
//! Example (PRD 18.4, October 2026): اکرم ٹول ٹیکس × کیلا 720 + 160 = 880; ارشد ملتانی × کیلا
//! 1,044 + 240 = 1,284; the چکتا buyer شاہد × سیب 720 + 80 = 800. The totals row: کیلا 2,164, سیب 800,
//! total 2,964 (the day's buyer commission 2,484 + لاگا 480).

use std::collections::{HashMap, HashSet};

use mandi_core::money::Paisa;
use mandi_db::rusqlite::Connection;

use super::facts::{self, Group, compare_range, run_range};
use super::filters::Typed;
use super::ids;
use super::model::{AnalyticsHeading, AnalyticsKey, AnalyticsRow, AnalyticsSide};
use super::table::{
    Table, col, compare_cells, compare_columns, count_of, dynamic_col, item_rows, page_window, row,
};
use crate::error::ReportsError;
use crate::reports::PageSpec;
use crate::reports::common::{add, amount_or_empty, chukta_party, load_parties};
use crate::reports::grouped_sums::chukta_names;
use crate::reports::{Cell, ColumnKind, Names, PartyRef};

/// Most columns of view L, the "other items" column included.
const MAX_ITEM_COLUMNS: usize = 24;

/// The key of a row: the buyer account and, for a چکتا کھاتہ person, his key.
type BuyerKey = (i64, Option<String>);

/// A total commission as a cell: money, or empty when it is 0.
fn cell(amount: Paisa) -> Cell {
    amount_or_empty(amount)
}

/// Builds view L.
///
/// # Errors
/// A database error; `ReportsError::Data` for a fact net below 0; `ReportsError::Money`.
pub(crate) fn run_l(
    conn: &Connection,
    typed: &Typed,
    spec: PageSpec,
) -> Result<Table, ReportsError> {
    let none = AnalyticsKey::default();
    let current: Vec<Group> = facts::groups(
        conn,
        &facts::conds(typed, run_range(typed), &none),
        &["f.buyer_account_id", "f.chukta_person_key", "f.item_id"],
    )?;
    let mut compared: HashMap<BuyerKey, Paisa> = HashMap::new();
    if typed.compare.is_some() {
        for group in facts::groups(
            conn,
            &facts::conds(typed, compare_range(typed), &none),
            &["f.buyer_account_id", "f.chukta_person_key"],
        )? {
            if let Some(id) = group.int(0) {
                compared.insert(
                    (id, group.text(1)),
                    group.sums.total_comm(AnalyticsSide::Buyer)?,
                );
            }
        }
    }

    let mut amounts: HashMap<(BuyerKey, i64), Paisa> = HashMap::new();
    let mut by_item: HashMap<i64, Paisa> = HashMap::new();
    let mut by_buyer: HashMap<BuyerKey, Paisa> = HashMap::new();
    for group in &current {
        let (Some(buyer), Some(item)) = (group.int(0), group.int(2)) else {
            continue;
        };
        let key: BuyerKey = (buyer, group.text(1));
        let amount = group.sums.total_comm(AnalyticsSide::Buyer)?;
        amounts.insert((key.clone(), item), amount);
        add(by_item.entry(item).or_insert(Paisa::ZERO), amount)?;
        add(by_buyer.entry(key).or_insert(Paisa::ZERO), amount)?;
    }

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
    let mut shown_sorted = shown;
    shown_sorted.sort_by(|a, b| item_code(*a).cmp(&item_code(*b)).then_with(|| a.cmp(b)));

    // The rows: buyers with purchases, and buyers only in the compared period.
    let mut keys: Vec<BuyerKey> = by_buyer.keys().cloned().collect();
    for key in compared.keys() {
        if !by_buyer.contains_key(key) {
            keys.push(key.clone());
        }
    }
    let account_ids: Vec<i64> = keys.iter().map(|(id, _)| *id).collect();
    let parties = load_parties(conn, &account_ids)?;
    let chukta_ids: Vec<i64> = {
        let mut ids: Vec<i64> = keys
            .iter()
            .filter(|(_, person)| person.is_some())
            .map(|(id, _)| *id)
            .collect();
        ids.sort_unstable();
        ids.dedup();
        ids
    };
    let saved = chukta_names(conn, &chukta_ids)?;
    let party_of = |key: &BuyerKey| -> Option<PartyRef> {
        let base = parties.get(&key.0)?.clone();
        Some(match &key.1 {
            Some(person) => {
                let names = saved.get(person);
                chukta_party(
                    base,
                    Some(person.clone()),
                    names.map(|p| p.name_ur.as_str()),
                    names.map(|p| p.name_en.as_str()),
                )
            }
            None => base,
        })
    };

    struct Entry {
        key: BuyerKey,
        total: Paisa,
        compared: Paisa,
    }
    let mut entries: Vec<Entry> = keys
        .into_iter()
        .map(|key| Entry {
            total: by_buyer.get(&key).copied().unwrap_or(Paisa::ZERO),
            compared: compared.get(&key).copied().unwrap_or(Paisa::ZERO),
            key,
        })
        .collect();
    let code_of = |key: &BuyerKey| {
        parties
            .get(&key.0)
            .map(|p| p.code.clone())
            .unwrap_or_default()
    };
    // Biggest buyers first by what they were charged, then by code.
    entries.sort_by(|a, b| {
        b.total
            .cmp(&a.total)
            .then_with(|| code_of(&a.key).cmp(&code_of(&b.key)))
            .then_with(|| a.key.1.cmp(&b.key.1))
            .then_with(|| a.key.0.cmp(&b.key.0))
    });

    let mut columns = vec![col("buyer", ids::COL_BUYER, ColumnKind::Party)];
    for item in &shown_sorted {
        let names = item_info.get(item).map_or_else(
            || Names {
                ur: String::new(),
                en: String::new(),
            },
            |i| i.names.clone(),
        );
        columns.push(dynamic_col(
            format!("i_{item}"),
            ids::COL_ITEM_COLUMN,
            AnalyticsHeading::Item { names },
            AnalyticsKey {
                item_id: Some(*item),
                ..AnalyticsKey::default()
            },
        ));
    }
    if overflow {
        columns.push(dynamic_col(
            "i_other".to_owned(),
            ids::COL_ITEM_COLUMN,
            AnalyticsHeading::OtherItems,
            AnalyticsKey {
                not_item_ids: shown_sorted.clone(),
                ..AnalyticsKey::default()
            },
        ));
    }
    columns.push(col("total_comm", ids::COL_ROW_TOTAL, ColumnKind::Money));
    let with_compare = typed.compare.is_some();
    if with_compare {
        columns.extend(compare_columns());
    }

    // What each buyer has in the items without a column of their own.
    let shown_set: HashSet<i64> = shown_sorted.iter().copied().collect();
    let mut rest_by_buyer: HashMap<BuyerKey, Paisa> = HashMap::new();
    if overflow {
        for ((buyer, item), amount) in &amounts {
            if !shown_set.contains(item) {
                add(
                    rest_by_buyer.entry(buyer.clone()).or_insert(Paisa::ZERO),
                    *amount,
                )?;
            }
        }
    }
    let width = shown_sorted.len() + usize::from(overflow);
    // The totals row is over every buyer, read from the sums themselves: one pass over them, not one
    // lookup for each buyer and item. The rows are built for the page only (a ten-year view has
    // thousands of buyers and a page has a hundred).
    let mut column_totals = vec![Paisa::ZERO; width];
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
    let mut grand = Paisa::ZERO;
    let mut grand_compared = Paisa::ZERO;
    for entry in &entries {
        add(&mut grand, entry.total)?;
        add(&mut grand_compared, entry.compared)?;
    }
    let (skip, take) = page_window(spec);
    let mut all_rows: Vec<AnalyticsRow> = Vec::new();
    for entry in entries.iter().skip(skip).take(take) {
        let mut out = vec![party_of(&entry.key).map_or(Cell::Empty, |party| Cell::Party { party })];
        for item in &shown_sorted {
            let amount = amounts
                .get(&(entry.key.clone(), *item))
                .copied()
                .unwrap_or(Paisa::ZERO);
            out.push(cell(amount));
        }
        if overflow {
            let rest = rest_by_buyer
                .get(&entry.key)
                .copied()
                .unwrap_or(Paisa::ZERO);
            out.push(cell(rest));
        }
        out.push(Cell::Money { paisa: entry.total });
        if with_compare {
            out.extend(compare_cells(entry.total, entry.compared)?);
        }
        all_rows.push(row(
            out,
            AnalyticsKey {
                buyer_id: Some(entry.key.0),
                person_key: entry.key.1.clone(),
                ..AnalyticsKey::default()
            },
        ));
    }
    let mut totals = vec![Cell::Empty];
    totals.extend(column_totals.into_iter().map(cell));
    totals.push(Cell::Money { paisa: grand });
    if with_compare {
        totals.extend(compare_cells(grand, grand_compared)?);
    }

    Ok(Table {
        charts: Vec::new(),
        columns,
        total_count: count_of(entries.len()),
        rows: all_rows,
        totals,
    })
}
