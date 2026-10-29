//! View H, top 10, سرفہرست (PRD 18.2): the ten items, the ten sellers and the ten buyers with the
//! most total commission (seller commission + buyer commission + لاگا of their sale lines, or
//! the side asked for), each list under its own heading row.
//!
//! A one-time buyer of the پختہ کھاتہ account is one row for each person (FR-13.9). A row with
//! no commission left is not listed. The share is the row's part of ALL the commission of the
//! filter, not of the top ten.
//!
//! Example (the worked day): items کیلا 3,928, سیب 1,520; sellers رحیم 3,928, کریم 1,520; buyers
//! ارشد 2,328, اکرم 1,600, شاہد 1,520.

use std::collections::HashMap;

use mandi_core::money::Paisa;
use mandi_db::rusqlite::Connection;

use super::facts::{self, Group, Sums, compare_range, run_range};
use super::filters::Typed;
use super::ids;
use super::model::{AnalyticsKey, AnalyticsRow};
use super::table::{
    Table, col, compare_cells, compare_columns, count_of, item_rows, percent_cell, row,
};
use crate::error::ReportsError;
use crate::reports::PageSpec;
use crate::reports::common::{add, load_parties, pukhta_party};
use crate::reports::grouped_sums::pukhta_names;
use crate::reports::{Cell, ColumnKind, RowKind};

/// Rows in each list.
const TOP: usize = 10;

/// A candidate row of one list: the id (and person key), its sums and its total commission.
struct Candidate {
    id: i64,
    person: Option<String>,
    sums: Sums,
    total: Paisa,
}

/// The candidates of one list, biggest first, with the total of all of them.
fn candidates(
    groups: &[Group],
    typed: &Typed,
    keyed_by_person: bool,
) -> Result<(Vec<Candidate>, Paisa), ReportsError> {
    let mut found = Vec::new();
    let mut grand = Paisa::ZERO;
    for group in groups {
        let Some(id) = group.int(0) else { continue };
        let total = group.sums.total_comm(typed.side)?;
        add(&mut grand, total)?;
        if total > Paisa::ZERO {
            found.push(Candidate {
                id,
                person: if keyed_by_person { group.text(1) } else { None },
                sums: group.sums,
                total,
            });
        }
    }
    Ok((found, grand))
}

fn heading(label: &str, width: usize) -> AnalyticsRow {
    let mut cells = vec![Cell::Label {
        id: label.to_owned(),
    }];
    cells.extend(std::iter::repeat_n(Cell::Empty, width.saturating_sub(1)));
    AnalyticsRow {
        cells,
        key: None,
        kind: RowKind::Group,
    }
}

/// Builds view H.
///
/// # Errors
/// A database error; `ReportsError::Data` for a fact net below 0; `ReportsError::Money`.
pub(crate) fn run(conn: &Connection, typed: &Typed, spec: PageSpec) -> Result<Table, ReportsError> {
    let none = AnalyticsKey::default();
    let by_item = facts::groups(
        conn,
        &facts::conds(typed, run_range(typed), &none),
        &["f.item_id"],
    )?;
    let by_seller = facts::groups(
        conn,
        &facts::conds(typed, run_range(typed), &none),
        &["f.seller_account_id"],
    )?;
    let by_buyer = facts::groups(
        conn,
        &facts::conds(typed, run_range(typed), &none),
        &["f.buyer_account_id", "f.pukhta_person_key"],
    )?;
    let (mut items, grand) = candidates(&by_item, typed, false)?;
    let (mut sellers, _) = candidates(&by_seller, typed, false)?;
    let (mut buyers, _) = candidates(&by_buyer, typed, true)?;

    // Names for the candidates, and the order (biggest first, then code, then id).
    let item_info = item_rows(conn, &items.iter().map(|c| c.id).collect::<Vec<_>>())?;
    let mut account_ids: Vec<i64> = sellers.iter().map(|c| c.id).collect();
    account_ids.extend(buyers.iter().map(|c| c.id));
    let parties = load_parties(conn, &account_ids)?;
    let account_code = |id: i64| parties.get(&id).map(|p| p.code.clone()).unwrap_or_default();
    items.sort_by(|a, b| {
        let code = |id: i64| {
            item_info
                .get(&id)
                .map(|i| i.code.clone())
                .unwrap_or_default()
        };
        b.total
            .cmp(&a.total)
            .then_with(|| code(a.id).cmp(&code(b.id)))
            .then_with(|| a.id.cmp(&b.id))
    });
    for list in [&mut sellers, &mut buyers] {
        list.sort_by(|a, b| {
            b.total
                .cmp(&a.total)
                .then_with(|| account_code(a.id).cmp(&account_code(b.id)))
                .then_with(|| a.id.cmp(&b.id))
                .then_with(|| a.person.cmp(&b.person))
        });
    }
    items.truncate(TOP);
    sellers.truncate(TOP);
    buyers.truncate(TOP);

    // The compared period: the total commission of the same entities.
    let with_compare = typed.compare.is_some();
    let mut compare_item: HashMap<i64, Paisa> = HashMap::new();
    let mut compare_seller: HashMap<i64, Paisa> = HashMap::new();
    let mut compare_buyer: HashMap<(i64, Option<String>), Paisa> = HashMap::new();
    let mut grand_compared = Paisa::ZERO;
    if with_compare {
        let range = compare_range(typed);
        for group in facts::groups(conn, &facts::conds(typed, range, &none), &["f.item_id"])? {
            if let Some(id) = group.int(0) {
                let total = group.sums.total_comm(typed.side)?;
                add(&mut grand_compared, total)?;
                compare_item.insert(id, total);
            }
        }
        for group in facts::groups(
            conn,
            &facts::conds(typed, range, &none),
            &["f.seller_account_id"],
        )? {
            if let Some(id) = group.int(0) {
                compare_seller.insert(id, group.sums.total_comm(typed.side)?);
            }
        }
        for group in facts::groups(
            conn,
            &facts::conds(typed, range, &none),
            &["f.buyer_account_id", "f.pukhta_person_key"],
        )? {
            if let Some(id) = group.int(0) {
                compare_buyer.insert((id, group.text(1)), group.sums.total_comm(typed.side)?);
            }
        }
    }

    let pukhta_ids: Vec<i64> = {
        let mut ids: Vec<i64> = buyers
            .iter()
            .filter(|c| c.person.is_some())
            .map(|c| c.id)
            .collect();
        ids.sort_unstable();
        ids.dedup();
        ids
    };
    let names = pukhta_names(conn, &pukhta_ids)?;

    let mut columns = vec![
        col("rank", ids::COL_RANK, ColumnKind::Number),
        col("name", ids::COL_NAME, ColumnKind::Party),
        col("qty", ids::COL_QTY, ColumnKind::Number),
        col("total_comm", ids::COL_TOTAL_COMM, ColumnKind::Money),
        col("share", ids::COL_SHARE, ColumnKind::Percent),
    ];
    if with_compare {
        columns.extend(compare_columns());
    }
    let width = columns.len();

    let one = |rank: usize,
               name: Cell,
               candidate: &Candidate,
               compared: Paisa,
               key: AnalyticsKey|
     -> Result<AnalyticsRow, ReportsError> {
        let mut cells = vec![
            Cell::Number {
                value: i64::try_from(rank).unwrap_or(i64::MAX),
            },
            name,
            Cell::Number {
                value: candidate.sums.qty,
            },
            Cell::Money {
                paisa: candidate.total,
            },
            percent_cell(candidate.total, grand)?,
        ];
        if with_compare {
            cells.extend(compare_cells(candidate.total, compared)?);
        }
        Ok(row(cells, key))
    };

    let mut item_rows_out: Vec<AnalyticsRow> = Vec::new();
    for (index, candidate) in items.iter().enumerate() {
        let name = item_info
            .get(&candidate.id)
            .map_or(Cell::Empty, |i| Cell::Names {
                names: i.names.clone(),
            });
        item_rows_out.push(one(
            index.saturating_add(1),
            name,
            candidate,
            compare_item
                .get(&candidate.id)
                .copied()
                .unwrap_or(Paisa::ZERO),
            AnalyticsKey {
                item_id: Some(candidate.id),
                ..AnalyticsKey::default()
            },
        )?);
    }
    let mut seller_rows_out: Vec<AnalyticsRow> = Vec::new();
    for (index, candidate) in sellers.iter().enumerate() {
        let name = parties
            .get(&candidate.id)
            .map_or(Cell::Empty, |p| Cell::Party { party: p.clone() });
        seller_rows_out.push(one(
            index.saturating_add(1),
            name,
            candidate,
            compare_seller
                .get(&candidate.id)
                .copied()
                .unwrap_or(Paisa::ZERO),
            AnalyticsKey {
                seller_id: Some(candidate.id),
                ..AnalyticsKey::default()
            },
        )?);
    }
    let mut buyer_rows_out: Vec<AnalyticsRow> = Vec::new();
    for (index, candidate) in buyers.iter().enumerate() {
        let name = parties.get(&candidate.id).map_or(Cell::Empty, |base| {
            let party = match &candidate.person {
                Some(person) => {
                    let saved = names.get(person);
                    pukhta_party(
                        base.clone(),
                        Some(person.clone()),
                        saved.map(|p| p.name_ur.as_str()),
                        saved.map(|p| p.name_en.as_str()),
                    )
                }
                None => base.clone(),
            };
            Cell::Party { party }
        });
        buyer_rows_out.push(one(
            index.saturating_add(1),
            name,
            candidate,
            compare_buyer
                .get(&(candidate.id, candidate.person.clone()))
                .copied()
                .unwrap_or(Paisa::ZERO),
            AnalyticsKey {
                buyer_id: Some(candidate.id),
                person_key: candidate.person.clone(),
                ..AnalyticsKey::default()
            },
        )?);
    }
    // Each list that has rows goes under its heading.
    let mut all_rows: Vec<AnalyticsRow> = Vec::new();
    for (label, list) in [
        (ids::TOP_ITEMS, item_rows_out),
        (ids::TOP_SELLERS, seller_rows_out),
        (ids::TOP_BUYERS, buyer_rows_out),
    ] {
        if !list.is_empty() {
            all_rows.push(heading(label, width));
            all_rows.extend(list);
        }
    }

    // The totals row: the quantity and the commission of the whole filter.
    let mut qty = 0_i64;
    for group in &by_item {
        qty = qty
            .checked_add(group.sums.qty)
            .ok_or_else(|| ReportsError::Data("the quantity total does not fit".to_owned()))?;
    }
    let mut totals = vec![
        Cell::Empty,
        Cell::Empty,
        Cell::Number { value: qty },
        Cell::Money { paisa: grand },
        percent_cell(grand, grand)?,
    ];
    if with_compare {
        totals.extend(compare_cells(grand, grand_compared)?);
    }

    Ok(Table {
        charts: Vec::new(),
        columns,
        total_count: count_of(all_rows.len()),
        rows: spec.slice(&all_rows),
        totals,
    })
}
