//! R13 کھاتہ, the party ledger, and R14 فرد حساب, the party statement (PRD 19.2, FR-19.8,
//! FR-19.10, FR-19.11, C13).
//!
//! Both read the posted lines of one account (or one person of a پختہ کھاتہ account). The
//! opening balance is everything dated before the first day; each line carries the balance after
//! it. A balance is `Σ debit − Σ credit` of stored paisa, worked out in Rust with checked maths.
//! The balance of a page's first row is the sum of the lines before it, read from the nearer end of
//! the account (`lines::page_with_running`), and each next row adds its own line, so a page that
//! starts at line 5,000 has the right balance without reading the rows before it into memory.
//!
//! R13 lists every line with its items (FR-19.8). R14 is the same data as a statement: opening,
//! total bought or sold, paid, closing (balance only), and with detail the lines of R13.
//!
//! Example (E2E-33, اکرم on 10-10-26): opening 0; the sale line "کیلا 20 پیٹی @ 600 = 12,000،
//! کمیشن 720، لاگا 160" with 12,880 بنام, balance 12,880; the receipt with 10,000 جمع, balance
//! 2,880; closing 2,880 بنام.

use mandi_core::accounts::Balance;
use mandi_core::money::Paisa;
use mandi_db::memo;
use mandi_db::rusqlite::{Connection, OptionalExtension, params};

use super::PageSpec;
use super::ledger_items::{DetailQuery, load};
use super::lines::{self, LineFilter, RawLine};
use crate::error::ReportsError;
use crate::reports::common::{LIVE, amount_or_empty, balance_cell, paisa, source_of};
use crate::reports::filters::{LineSort, Typed};
use crate::reports::ids;
use crate::reports::model::{
    Cell, Column, ColumnKind, DetailPart, ReportId, ReportPage, Row, RowKind, SummaryLine,
};

fn columns() -> Vec<Column> {
    let col = |key: &str, title: &str, kind| Column {
        key: key.to_owned(),
        title_id: title.to_owned(),
        kind,
    };
    vec![
        col("date", ids::COL_DATE, ColumnKind::Date),
        col("doc", ids::COL_DOC, ColumnKind::Doc),
        col("detail", ids::COL_DETAIL, ColumnKind::Detail),
        col("narration", ids::COL_NARRATION, ColumnKind::Text),
        col("debit", ids::COL_DEBIT, ColumnKind::Money),
        col("credit", ids::COL_CREDIT, ColumnKind::Money),
        col("balance", ids::COL_BALANCE, ColumnKind::Balance),
    ]
}

/// R13.
///
/// # Errors
/// A database error; `ReportsError::Money`.
pub(crate) fn run_r13(
    conn: &Connection,
    typed: &Typed,
    spec: PageSpec,
) -> Result<ReportPage, ReportsError> {
    run(conn, ReportId::R13, typed, spec, true)
}

/// R14. With `detail` off it returns the balances only; with `detail` on, the lines of R13.
///
/// # Errors
/// A database error; `ReportsError::Money`.
pub(crate) fn run_r14(
    conn: &Connection,
    typed: &Typed,
    spec: PageSpec,
) -> Result<ReportPage, ReportsError> {
    run(conn, ReportId::R14, typed, spec, typed.detail)
}

/// The sums of the live lines only, for the "bought or sold" and "paid" lines of R14. An entry
/// that was reversed and the reversal itself cancel each other, so neither counts as bought, sold
/// or paid (PRD FR-18.7 in spirit, C13). The closing balance still uses [`TOTALS_SQL`]: the pair
/// has the same date and nets to zero there.
fn live_totals_sql() -> String {
    format!(
        "SELECT COALESCE(SUM(l.dr_paisa), 0), COALESCE(SUM(l.cr_paisa), 0)
         FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
         WHERE {LIVE} AND l.account_id = ?1
           AND (?2 IS NULL OR l.pukhta_person_key = ?2)
           AND (?3 IS NULL OR e.entry_date >= ?3) AND (?4 IS NULL OR e.entry_date <= ?4)"
    )
}

fn run(
    conn: &Connection,
    id: ReportId,
    typed: &Typed,
    spec: PageSpec,
    with_lines: bool,
) -> Result<ReportPage, ReportsError> {
    let account_id = typed
        .party_id
        .ok_or_else(|| ReportsError::Data("the ledger has no account".to_owned()))?;
    let person = typed.person_key.as_deref();
    let from = typed.from.as_deref();
    let to = typed.to.as_deref();

    // The lines asked for: the account (or one person of it) and the days.
    let filter = LineFilter {
        account: Some(account_id),
        person: typed.person_key.clone(),
        from: typed.from.clone(),
        to: typed.to.clone(),
    };
    // The balance before the first day: everything dated before `from`, as the whole account less
    // what is dated from that day on (both are kept sums, so the next page does not read the
    // account again).
    let (open_dr, open_cr) = match from {
        Some(first) => {
            let (dr, cr) = lines::before_day(conn, &filter, first)?;
            (paisa(dr)?, paisa(cr)?)
        }
        None => (Paisa::ZERO, Paisa::ZERO),
    };
    let opening = Balance::from_totals(open_dr, open_cr)?;

    // Count and totals over the whole filter.
    let sums = lines::totals(conn, &filter)?;
    let (count, sum_dr, sum_cr) = (sums.count, sums.dr_paisa, sums.cr_paisa);
    let (total_dr, total_cr) = (paisa(sum_dr)?, paisa(sum_cr)?);
    let closing = Balance::from_totals(
        open_dr.checked_add(total_dr)?,
        open_cr.checked_add(total_cr)?,
    )?;

    let party_type: Option<String> = conn
        .query_row(
            "SELECT party_type FROM accounts WHERE id = ?1",
            [account_id],
            |r| r.get(0),
        )
        .optional()?;
    let party_type = party_type.unwrap_or_default();

    let rows = if with_lines {
        page_rows(
            conn,
            typed,
            &filter,
            spec,
            (open_dr, open_cr),
            party_type == "buyer",
        )?
    } else {
        Vec::new()
    };

    let mut summary = vec![SummaryLine {
        label_id: ids::OPENING_BALANCE.to_owned(),
        cell: balance_cell(opening),
    }];
    if id == ReportId::R14 {
        // Bought or sold, then paid (PRD 19.2): a buyer is debited for what he bought and
        // credited for what he paid; a seller is credited for what he sold and debited for what
        // was paid to him. Only live entries count: an edited or cancelled bill and its reversal
        // are left out, so the lines read what is really owed (E2E-13: bought 13,092, not 25,972).
        let live = memo::sums(
            conn,
            &format!("live {account_id} {person:?} {from:?} {to:?}"),
            || {
                conn.query_row(
                    &live_totals_sql(),
                    params![account_id, person, from, to],
                    |r| {
                        Ok(memo::Sums {
                            count: 0,
                            dr_paisa: r.get(0)?,
                            cr_paisa: r.get(1)?,
                        })
                    },
                )
            },
        )?;
        let (total_dr, total_cr) = (paisa(live.dr_paisa)?, paisa(live.cr_paisa)?);
        let (first, first_label, second, second_label) = match party_type.as_str() {
            "buyer" => (
                total_dr,
                ids::STATEMENT_BOUGHT,
                total_cr,
                ids::STATEMENT_PAID,
            ),
            "seller" => (total_cr, ids::STATEMENT_SOLD, total_dr, ids::STATEMENT_PAID),
            _ => (
                total_dr,
                ids::STATEMENT_DEBIT,
                total_cr,
                ids::STATEMENT_CREDIT,
            ),
        };
        summary.push(SummaryLine {
            label_id: first_label.to_owned(),
            cell: Cell::Money { paisa: first },
        });
        summary.push(SummaryLine {
            label_id: second_label.to_owned(),
            cell: Cell::Money { paisa: second },
        });
    }
    summary.push(SummaryLine {
        label_id: ids::CLOSING_BALANCE.to_owned(),
        cell: balance_cell(closing),
    });

    Ok(ReportPage {
        group_above: None,
        id,
        title_id: ids::title_id(id.code()),
        filters: typed.resolved.clone(),
        columns: columns(),
        rows,
        offset: if with_lines { spec.offset } else { 0 },
        limit: if with_lines { spec.reported_limit() } else { 0 },
        total_count: if with_lines { count } else { 0 },
        totals_label_id: ids::TOTAL.to_owned(),
        totals: vec![
            Cell::Empty,
            Cell::Empty,
            Cell::Empty,
            Cell::Empty,
            Cell::Money { paisa: total_dr },
            Cell::Money { paisa: total_cr },
            balance_cell(closing),
        ],
        summary,
    })
}

fn page_rows(
    conn: &Connection,
    typed: &Typed,
    filter: &LineFilter,
    spec: PageSpec,
    (open_dr, open_cr): (Paisa, Paisa),
    ledger_is_buyer: bool,
) -> Result<Vec<Row>, ReportsError> {
    // Sorted by an amount, the page is the best lines of the filter by SQL and has no running
    // balance (it is the order of the days that gives one, plan 38 B2).
    let window = (spec.offset, spec.sql_limit());
    let sorted = matches!(typed.line_sort, Some(LineSort::Amount { .. }));
    let page: Vec<(RawLine, i64, i64)> = match typed.line_sort {
        Some(sort @ LineSort::Amount { .. }) => lines::page_by(conn, filter, Some(sort), window)?
            .into_iter()
            .map(|raw| (raw, 0, 0))
            .collect(),
        Some(LineSort::Newest) => lines::page_with_running_newest(conn, filter, window)?,
        None => lines::page_with_running(conn, filter, window)?,
    };
    let raws: Vec<&RawLine> = page.iter().map(|(raw, _, _)| raw).collect();

    let queries: Vec<DetailQuery> = raws
        .iter()
        .map(|raw| DetailQuery {
            entry_id: raw.entry_id,
            line_no: raw.line_no,
            bikri_base_entry: (raw.source_type == "bikri")
                .then(|| raw.reverses.unwrap_or(raw.entry_id)),
            voucher: (raw.source_type == "voucher")
                .then(|| {
                    raw.voucher_type
                        .clone()
                        .map(|t| (t, raw.voucher_reason.clone()))
                })
                .flatten(),
        })
        .collect();
    let mut details = load(conn, &queries, ledger_is_buyer)?;

    let mut rows = Vec::with_capacity(page.len());
    for (raw, run_dr, run_cr) in &page {
        let balance = Balance::from_totals(
            open_dr.checked_add(paisa(*run_dr)?)?,
            open_cr.checked_add(paisa(*run_cr)?)?,
        )?;
        let mut parts = details
            .remove(&(raw.entry_id, raw.line_no))
            .unwrap_or_default();
        // On the whole of a پختہ کھاتہ account, a line says which person it belongs to (PRD
        // FR-13.5). The key is the name and phone as stored.
        if typed.person_key.is_none()
            && let Some(key) = raw.person_key.as_deref().filter(|k| !k.is_empty())
        {
            parts.insert(
                0,
                DetailPart::Text {
                    text: key.to_owned(),
                },
            );
        }
        let detail = if parts.is_empty() {
            Cell::Empty
        } else {
            Cell::Detail { parts }
        };
        let narration = raw
            .narration
            .as_deref()
            .map(str::trim)
            .filter(|t| !t.is_empty())
            .map_or(Cell::Empty, |t| Cell::Text { text: t.to_owned() });
        rows.push(Row {
            cells: vec![
                Cell::Date {
                    iso: raw.entry_date.clone(),
                },
                Cell::Doc {
                    type_id: ids::doc_id(&raw.posting_type),
                    number: raw.doc_number,
                },
                detail,
                narration,
                amount_or_empty(paisa(raw.dr)?),
                amount_or_empty(paisa(raw.cr)?),
                if sorted {
                    Cell::Empty
                } else {
                    balance_cell(balance)
                },
            ],
            source: Some(source_of(&raw.source_type, raw.source_id, raw.entry_id)),
            kind: RowKind::Normal,
        });
    }
    Ok(rows)
}
