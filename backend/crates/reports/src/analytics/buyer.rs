//! View C, by buyer, گاہک وار (PRD 18.2, FR-18.6, FR-18.7, FR-13.9).
//!
//! One row for each buyer (and each one-time person inside the پختہ کھاتہ account) with
//! purchases in the filter: نگ, the purchase value, the buyer commission, لاگا, the dues now,
//! and the average days to pay.
//!
//! - **Dues now.** The buyer's balance today, the posted `Σ بنام − Σ جمع` (the same as R20), for
//!   the rows of the page only.
//! - **Average days to pay (PRD 18.2, money agent review).** The average, over his bills that
//!   were fully paid in the period, of the days from the bill date to the date the bill was
//!   fully paid; empty if there is none. A bill is a live debit line from a بکری posting (an
//!   opening, cash lent or a جنرل debit is not a purchase). It is fully paid when its active
//!   matching rows add up to the line, counting only payments that are themselves live; the paid
//!   date is the latest of those payments. Each bill counts once, whatever its amount; the
//!   average is rounded to a whole day, half up. The cell is empty when an item, seller or
//!   اڈا filter is set (a bill cannot be split by item).
//!
//! Example (PRD 18.4): ارشد 30 نگ, 17,400, 1,044, 240, dues 18,684, no average; اکرم 20, 12,000,
//! 720, 160, dues 2,880; the one-time buyer شاہد 10, 12,000, 720, 80, dues 0, average 0 days.

use std::collections::HashMap;

use mandi_core::accounts::Balance;
use mandi_core::money::Paisa;
use mandi_db::rusqlite::types::Value;
use mandi_db::rusqlite::{Connection, params_from_iter};

use super::calc::average_days;
use super::chart;
use super::facts::{self, Sums, compare_range, run_range};
use super::filters::Typed;
use super::ids;
use super::model::{
    AnalyticsChart, AnalyticsChartKind, AnalyticsChartPoint, AnalyticsChartSeries, AnalyticsKey,
    AnalyticsRow,
};
use super::table::{Part, Table, col, compare_cells, compare_columns, count_of, part_cell, row};
use crate::error::ReportsError;
use crate::reports::PageSpec;
use crate::reports::common::{
    IN_CHUNK, LIVE, add, balance_cell, load_parties, paisa, placeholders, pukhta_party,
};
use crate::reports::grouped_sums::pukhta_names;
use crate::reports::{Cell, ColumnKind, PartyRef};

/// The key of a row: the buyer account and, for a پختہ کھاتہ person, his key.
type BuyerKey = (i64, Option<String>);

struct Entry {
    key: BuyerKey,
    sums: Sums,
    compared: Paisa,
}

/// The balances of some buyer keys today: `Σ بنام − Σ جمع` of the posted lines, one query for
/// each chunk of accounts (Code Standards B22).
fn balances(
    conn: &Connection,
    keys: &[BuyerKey],
) -> Result<HashMap<BuyerKey, Balance>, ReportsError> {
    let mut accounts: Vec<i64> = keys.iter().map(|(id, _)| *id).collect();
    accounts.sort_unstable();
    accounts.dedup();
    let mut found = HashMap::new();
    for chunk in accounts.chunks(IN_CHUNK) {
        let sql = format!(
            "SELECT l.account_id, l.pukhta_person_key, COALESCE(SUM(l.dr_paisa), 0),
                    COALESCE(SUM(l.cr_paisa), 0)
             FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
             WHERE e.status = 'posted' AND l.account_id IN ({})
             GROUP BY l.account_id, l.pukhta_person_key",
            placeholders(chunk.len())
        );
        let mut statement = conn.prepare(&sql)?;
        let rows = statement
            .query_map(params_from_iter(chunk.iter()), |r| {
                Ok((
                    (r.get::<_, i64>(0)?, r.get::<_, Option<String>>(1)?),
                    r.get::<_, i64>(2)?,
                    r.get::<_, i64>(3)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        for (key, dr, cr) in rows {
            found.insert(key, Balance::from_totals(paisa(dr)?, paisa(cr)?)?);
        }
    }
    Ok(found)
}

/// For some buyer keys: how many of their bills were fully paid in the period, and the sum of
/// the days it took. One query for each chunk of accounts.
fn days_to_pay(
    conn: &Connection,
    typed: &Typed,
    keys: &[BuyerKey],
) -> Result<HashMap<BuyerKey, (i64, i64)>, ReportsError> {
    // A matching row counts only when the payment it points at is live: a posted entry that is
    // not a reversal and not reversed.
    let live_payment = "pe.status = 'posted' AND pe.reverses_entry_id IS NULL
        AND NOT EXISTS (SELECT 1 FROM journal_entries r
                        WHERE r.reverses_entry_id = pe.id AND r.status = 'posted')";
    let mut accounts: Vec<i64> = keys.iter().map(|(id, _)| *id).collect();
    accounts.sort_unstable();
    accounts.dedup();
    let mut found = HashMap::new();
    for chunk in accounts.chunks(IN_CHUNK) {
        let mut args: Vec<Value> = chunk.iter().map(|id| Value::Integer(*id)).collect();
        let mut bounds = String::new();
        if let Some(from) = &typed.from {
            bounds.push_str(" AND t.paid >= ?");
            args.push(Value::Text(from.clone()));
        }
        if let Some(to) = &typed.to {
            bounds.push_str(" AND t.paid <= ?");
            args.push(Value::Text(to.clone()));
        }
        let sql = format!(
            "SELECT t.account_id, t.person_key, COUNT(*),
                    COALESCE(SUM(MAX(0, CAST(julianday(t.paid) - julianday(t.bill) AS INTEGER))), 0)
             FROM (
                 SELECT l.account_id AS account_id, l.pukhta_person_key AS person_key,
                        e.entry_date AS bill, l.dr_paisa AS amount,
                        (SELECT SUM(a.amount_paisa) FROM bill_allocations a
                           JOIN journal_lines pl ON pl.id = a.payment_line_id
                           JOIN journal_entries pe ON pe.id = pl.entry_id
                          WHERE a.bill_line_id = l.id AND a.is_active = 1 AND {live_payment}) AS matched,
                        (SELECT MAX(pe.entry_date) FROM bill_allocations a
                           JOIN journal_lines pl ON pl.id = a.payment_line_id
                           JOIN journal_entries pe ON pe.id = pl.entry_id
                          WHERE a.bill_line_id = l.id AND a.is_active = 1 AND {live_payment}) AS paid
                 FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
                 WHERE l.account_id IN ({marks}) AND l.dr_paisa > 0
                   AND e.source_type = 'bikri' AND e.posting_type IN ('bikri_first', 'bikri_later')
                   AND {LIVE}
             ) t
             WHERE t.matched = t.amount{bounds}
             GROUP BY t.account_id, t.person_key",
            marks = placeholders(chunk.len())
        );
        let mut statement = conn.prepare(&sql)?;
        let rows = statement
            .query_map(params_from_iter(args.iter()), |r| {
                Ok((
                    (r.get::<_, i64>(0)?, r.get::<_, Option<String>>(1)?),
                    r.get::<_, i64>(2)?,
                    r.get::<_, i64>(3)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        for (key, count, days) in rows {
            found.insert(key, (count, days));
        }
    }
    Ok(found)
}

/// Builds view C.
///
/// # Errors
/// A database error; `ReportsError::Data` for a fact net below 0; `ReportsError::Money`.
pub(crate) fn run(conn: &Connection, typed: &Typed, spec: PageSpec) -> Result<Table, ReportsError> {
    let none = AnalyticsKey::default();
    let dims = ["f.buyer_account_id", "f.pukhta_person_key"];
    let current = facts::groups(conn, &facts::conds(typed, run_range(typed), &none), &dims)?;
    let mut compared: HashMap<BuyerKey, Paisa> = HashMap::new();
    if typed.compare.is_some() {
        for group in facts::groups(
            conn,
            &facts::conds(typed, compare_range(typed), &none),
            &dims,
        )? {
            if let Some(id) = group.int(0) {
                compared.insert((id, group.text(1)), group.sums.buyer_value);
            }
        }
    }

    let mut entries: Vec<Entry> = Vec::new();
    for group in &current {
        let Some(id) = group.int(0) else { continue };
        let key = (id, group.text(1));
        entries.push(Entry {
            compared: compared.get(&key).copied().unwrap_or(Paisa::ZERO),
            key,
            sums: group.sums,
        });
    }
    for (key, amount) in &compared {
        if !entries.iter().any(|e| &e.key == key) {
            entries.push(Entry {
                key: key.clone(),
                sums: Sums::default(),
                compared: *amount,
            });
        }
    }

    let account_ids: Vec<i64> = entries.iter().map(|e| e.key.0).collect();
    let parties = load_parties(conn, &account_ids)?;
    let pukhta_ids: Vec<i64> = {
        let mut ids: Vec<i64> = entries
            .iter()
            .filter(|e| e.key.1.is_some())
            .map(|e| e.key.0)
            .collect();
        ids.sort_unstable();
        ids.dedup();
        ids
    };
    let names = pukhta_names(conn, &pukhta_ids)?;
    let party_of = |key: &BuyerKey| -> Option<PartyRef> {
        let base = parties.get(&key.0)?.clone();
        Some(match &key.1 {
            Some(person) => {
                let saved = names.get(person);
                pukhta_party(
                    base,
                    Some(person.clone()),
                    saved.map(|p| p.name_ur.as_str()),
                    saved.map(|p| p.name_en.as_str()),
                )
            }
            None => base,
        })
    };
    // Biggest buyers first by the value they bought, then by code (PRD 18.4 lists ارشد, اکرم,
    // شاہد).
    entries.sort_by(|a, b| {
        let code = |e: &Entry| {
            parties
                .get(&e.key.0)
                .map(|p| p.code.clone())
                .unwrap_or_default()
        };
        b.sums
            .buyer_value
            .cmp(&a.sums.buyer_value)
            .then_with(|| code(a).cmp(&code(b)))
            .then_with(|| a.key.1.cmp(&b.key.1))
    });

    let mut sum = Sums::default();
    let mut grand_compared = Paisa::ZERO;
    for entry in &entries {
        sum.add(&entry.sums)?;
        add(&mut grand_compared, entry.compared)?;
    }

    // The chart (plan 38 R4, D1): the ten buyers who bought most, and one point for the rest,
    // from every buyer of the filter. The entries are in that order now; a sort changes the
    // table, never the chart.
    let charts = {
        let mut points = Vec::new();
        let (mut rest, mut rest_count) = (Paisa::ZERO, 0_i64);
        // A buyer who only bought in the compared period has nothing in this one.
        for (rank, entry) in entries
            .iter()
            .filter(|e| e.sums.buyer_value > Paisa::ZERO)
            .enumerate()
        {
            if rank < chart::TOP {
                points.push(AnalyticsChartPoint {
                    label: party_of(&entry.key)
                        .map(|party| Cell::Party { party })
                        .unwrap_or(Cell::Empty),
                    key: Some(AnalyticsKey {
                        buyer_id: Some(entry.key.0),
                        person_key: entry.key.1.clone(),
                        ..AnalyticsKey::default()
                    }),
                    values: vec![Cell::Money {
                        paisa: entry.sums.buyer_value,
                    }],
                    is_others: false,
                });
            } else {
                add(&mut rest, entry.sums.buyer_value)?;
                rest_count = rest_count
                    .checked_add(1)
                    .ok_or(mandi_core::money::MoneyError::Overflow)?;
            }
        }
        if rest_count > 0 {
            points.push(AnalyticsChartPoint {
                label: Cell::Empty,
                key: None,
                values: vec![Cell::Money { paisa: rest }],
                is_others: true,
            });
        }
        if points.is_empty() {
            Vec::new()
        } else {
            vec![AnalyticsChart {
                kind: AnalyticsChartKind::Ranked,
                title: None,
                series: vec![AnalyticsChartSeries {
                    key: "purchase_value".to_owned(),
                    kind: ColumnKind::Money,
                }],
                points,
                others_count: rest_count,
                by_month: false,
            }]
        }
    };

    // A sort by the columns known before the page is cut. The dues and the days to pay are worked
    // out for the page only, so they cannot be sorted (plan 38 R4).
    if let Some(sort) = &typed.sort {
        const SORTABLE: [&str; 5] = ["buyer", "qty", "purchase_value", "buyer_comm", "laga"];
        sort.check(&SORTABLE)
            .map_err(|e| ReportsError::Rejected(vec![e]))?;
        let code = |e: &Entry| {
            parties
                .get(&e.key.0)
                .map(|p| p.code.clone())
                .unwrap_or_default()
        };
        entries.sort_by(|a, b| {
            let ordering = match sort.column.as_str() {
                "buyer" => code(a).cmp(&code(b)).then_with(|| a.key.1.cmp(&b.key.1)),
                "qty" => a.sums.qty.cmp(&b.sums.qty),
                "purchase_value" => a.sums.buyer_value.cmp(&b.sums.buyer_value),
                "buyer_comm" => a.sums.buyer_comm.cmp(&b.sums.buyer_comm),
                _ => a.sums.laga.cmp(&b.sums.laga),
            };
            sort.direction.apply(ordering)
        });
    }

    // Dues and days to pay for the rows of this page only.
    let page_entries: Vec<&Entry> = {
        let ordered: Vec<&Entry> = entries.iter().collect();
        spec.slice(&ordered)
    };
    let page_keys: Vec<BuyerKey> = page_entries.iter().map(|e| e.key.clone()).collect();
    let dues = balances(conn, &page_keys)?;
    let paid_in_period =
        if typed.item_id.is_none() && typed.seller_id.is_none() && typed.adda_id.is_none() {
            Some(days_to_pay(conn, typed, &page_keys)?)
        } else {
            None
        };

    let with_compare = typed.compare.is_some();
    let mut columns = vec![
        col("buyer", ids::COL_BUYER, ColumnKind::Party),
        col("qty", ids::COL_QTY, ColumnKind::Number),
        col("purchase_value", ids::COL_PURCHASE_VALUE, ColumnKind::Money),
        col("buyer_comm", ids::COL_BUYER_COMM, ColumnKind::Money),
        col("laga", ids::COL_LAGA, ColumnKind::Money),
        col("dues_now", ids::COL_DUES_NOW, ColumnKind::Balance),
        col("avg_days", ids::COL_AVG_DAYS, ColumnKind::Number),
    ];
    if with_compare {
        columns.extend(compare_columns());
    }

    // A buyer in the dead amount keeps his balance in "dues now" and is marked ڈوبی (PRD FR-10.10,
    // view C).
    let marks = mandi_db::dead::marks_on(conn, None)?;
    let mut rows: Vec<AnalyticsRow> = Vec::with_capacity(page_entries.len());
    for entry in page_entries {
        let buyer = party_of(&entry.key)
            .map(|mut party| {
                party.dead = marks
                    .iter()
                    .find(|m| m.account_id == entry.key.0 && m.person_key == entry.key.1)
                    .map(|m| m.info());
                Cell::Party { party }
            })
            .unwrap_or(Cell::Empty);
        let dues_cell = dues
            .get(&entry.key)
            .copied()
            .map_or(Cell::Empty, balance_cell);
        let days_cell = match paid_in_period
            .as_ref()
            .and_then(|m| m.get(&entry.key))
            .map(|(count, days)| average_days(*days, *count))
            .transpose()?
            .flatten()
        {
            Some(days) => Cell::Number { value: days },
            None => Cell::Empty,
        };
        let mut cells = vec![
            buyer,
            Cell::Number {
                value: entry.sums.qty,
            },
            Cell::Money {
                paisa: entry.sums.buyer_value,
            },
            part_cell(typed.side, Part::Buyer, entry.sums.buyer_comm),
            part_cell(typed.side, Part::Buyer, entry.sums.laga),
            dues_cell,
            days_cell,
        ];
        if with_compare {
            cells.extend(compare_cells(entry.sums.buyer_value, entry.compared)?);
        }
        rows.push(row(
            cells,
            AnalyticsKey {
                buyer_id: Some(entry.key.0),
                person_key: entry.key.1.clone(),
                ..AnalyticsKey::default()
            },
        ));
    }

    let mut totals = vec![
        Cell::Empty,
        Cell::Number { value: sum.qty },
        Cell::Money {
            paisa: sum.buyer_value,
        },
        part_cell(typed.side, Part::Buyer, sum.buyer_comm),
        part_cell(typed.side, Part::Buyer, sum.laga),
        Cell::Empty,
        Cell::Empty,
    ];
    if with_compare {
        totals.extend(compare_cells(sum.buyer_value, grand_compared)?);
    }

    Ok(Table {
        columns,
        total_count: count_of(entries.len()),
        rows,
        totals,
        charts,
    })
}
