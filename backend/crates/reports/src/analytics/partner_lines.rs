//! The lines behind a number of view F (PRD FR-18.2, FR-17.5): the share rows of a partner, one
//! line for each posting that gave him a share, with the base it was worked on, his share and his
//! amount. A share of a shop expense is negative; a reversal row is the opposite of the row it
//! takes back and keeps its own date, so the lines add up to exactly the profit share that was
//! opened.
//!
//! The gardens, trips and fruit columns of the view open his lines on his gardens, trips and fruit
//! account (0604) instead ([`venture_run`]): one line for each journal line of that kind, with his
//! credit less his debit, so the lines add up to the cell that was opened.
//!
//! Example (the worked day, الف, 10-10-26): بکری 1 income 3,948 at 40% = 1,580; بکری 2 income 1,540
//! at 40% = 616; the electricity −3,000 at 40% = −1,200. The total row is 996. The fruit column
//! of E2E-42 opens two lines: −1,160 (home) and −1,000 (his part of the guests' fruit).

use mandi_db::rusqlite::{Connection, params_from_iter, types::Value};

use super::filters::Typed;
use super::ids;
use super::model::{AnalyticsKey, AnalyticsLinesInput, AnalyticsLinesPage, AnalyticsPartnerMoney};
use super::partner::{FRUIT_POSTINGS, GARDEN_POSTINGS, TRIP_POSTINGS};
use crate::error::ReportsError;
use crate::reports::common::{LIVE, paisa};
use crate::reports::{Cell, Column, ColumnKind, PageSpec, Row, RowKind, RowSource, SourceKind};

fn column(key: &str, title: &str, kind: ColumnKind) -> Column {
    Column {
        key: key.to_owned(),
        title_id: title.to_owned(),
        kind,
    }
}

fn columns() -> Vec<Column> {
    vec![
        column("date", ids::COL_DATE, ColumnKind::Date),
        column("doc", ids::COL_DOC, ColumnKind::Doc),
        column("kind", ids::COL_SHARE_KIND, ColumnKind::Label),
        column("base", ids::COL_SHARE_BASE, ColumnKind::Money),
        column("share", ids::COL_SHARE_PERCENT, ColumnKind::Percent),
        column("amount", ids::COL_SHARE_AMOUNT, ColumnKind::Money),
    ]
}

/// Where a row opens, by the kind of the journal entry's source.
fn source_of(source_type: &str, source_id: i64) -> RowSource {
    let kind = match source_type {
        "bikri" => SourceKind::Bikri,
        "voucher" => SourceKind::Voucher,
        "account" => SourceKind::Account,
        "month_close" => SourceKind::MonthClose,
        "purchase" => SourceKind::Purchase,
        "venture" => SourceKind::Venture,
        "fruit" => SourceKind::Fruit,
        _ => SourceKind::Entry,
    };
    RowSource {
        kind,
        id: source_id,
    }
}

/// The share rows of the partner in the key (or the filter), inside the period of the run and of
/// the key.
///
/// # Errors
/// A database error; `ReportsError::Money` if a stored amount is not whole rupees.
pub(crate) fn run(
    conn: &Connection,
    typed: &Typed,
    key: &AnalyticsKey,
    input: &AnalyticsLinesInput,
    spec: PageSpec,
) -> Result<AnalyticsLinesPage, ReportsError> {
    if let Some(money) = key.money {
        return venture_run(conn, typed, key, money, input, spec);
    }
    // `YYYY-MM-DD` sorts as a date: the later start and the earlier end.
    let from = [typed.from.as_ref(), key.from.as_ref()]
        .into_iter()
        .flatten()
        .max()
        .cloned();
    let to = [typed.to.as_ref(), key.to.as_ref()]
        .into_iter()
        .flatten()
        .min()
        .cloned();
    let partner = key.partner_id.or(typed.partner_id);
    let filter = "(?1 IS NULL OR s.partner_id = ?1) AND (?2 IS NULL OR s.entry_date >= ?2)
                  AND (?3 IS NULL OR s.entry_date <= ?3)";
    let args = vec![
        partner.map_or(Value::Null, Value::Integer),
        from.map_or(Value::Null, Value::Text),
        to.map_or(Value::Null, Value::Text),
    ];

    // The count and the sum over the whole filter (INTEGER columns only).
    let (total_count, total_amount): (i64, i64) = conn.query_row(
        &format!(
            "SELECT COUNT(*), COALESCE(SUM(s.amount_paisa), 0)
             FROM partner_share_rows s WHERE {filter}"
        ),
        params_from_iter(args.iter()),
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;

    let mut page_args = args;
    page_args.push(Value::Integer(spec.sql_limit()));
    page_args.push(Value::Integer(spec.offset));
    let mut statement = conn.prepare(&format!(
        "SELECT s.entry_date, e.posting_type, e.source_type, e.source_id,
                COALESCE(b.number, v.number), s.kind, s.base_paisa, s.share_bps, s.amount_paisa
         FROM partner_share_rows s
         JOIN journal_entries e ON e.id = s.journal_entry_id
         LEFT JOIN bikri_entries b ON e.source_type = 'bikri' AND b.id = e.source_id
         LEFT JOIN vouchers v ON e.source_type = 'voucher' AND v.id = e.source_id
         WHERE {filter} ORDER BY s.entry_date, s.id LIMIT ?4 OFFSET ?5"
    ))?;
    let raws = statement
        .query_map(params_from_iter(page_args.iter()), |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, i64>(3)?,
                r.get::<_, Option<i64>>(4)?,
                r.get::<_, String>(5)?,
                r.get::<_, i64>(6)?,
                r.get::<_, i64>(7)?,
                r.get::<_, i64>(8)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    let mut rows = Vec::with_capacity(raws.len());
    for (date, posting_type, source_type, source_id, number, kind, base, bps, amount) in raws {
        rows.push(Row {
            cells: vec![
                Cell::Date { iso: date },
                Cell::Doc {
                    type_id: crate::reports::ids::doc_id(&posting_type),
                    number,
                },
                Cell::Label {
                    id: if kind == "income" {
                        ids::COL_INCOME_SHARE.to_owned()
                    } else {
                        ids::COL_EXPENSE_SHARE.to_owned()
                    },
                },
                Cell::Money {
                    paisa: paisa(base)?,
                },
                Cell::Percent { hundredths: bps },
                Cell::Money {
                    paisa: paisa(amount)?,
                },
            ],
            source: Some(source_of(&source_type, source_id)),
            kind: RowKind::Normal,
        });
    }
    Ok(AnalyticsLinesPage {
        view: input.view,
        title_id: ids::LINES_TITLE.to_owned(),
        filters: typed.resolved.clone(),
        columns: columns(),
        rows,
        offset: spec.offset,
        limit: spec.reported_limit(),
        total_count,
        totals_label_id: ids::TOTAL.to_owned(),
        totals: vec![
            Cell::Empty,
            Cell::Empty,
            Cell::Empty,
            Cell::Empty,
            Cell::Empty,
            Cell::Money {
                paisa: paisa(total_amount)?,
            },
        ],
    })
}

fn venture_columns() -> Vec<Column> {
    vec![
        column("date", ids::COL_DATE, ColumnKind::Date),
        column("doc", ids::COL_DOC, ColumnKind::Doc),
        column("partner", ids::COL_PARTNER, ColumnKind::Names),
        column("amount", ids::COL_SHARE_AMOUNT, ColumnKind::Money),
    ]
}

/// The lines behind the gardens, trips or fruit column of view F: the journal lines on the
/// partners' 0604 accounts of that kind of posting, live entries only and not of a month close,
/// inside the period of the run and of the key. Each is his credit less his debit (a fruit take is
/// negative), so the lines add up to exactly the cell that was opened; the totals row is the same
/// sum over all of them. With a partner picked (in the key or the filter) only his account; without
/// one, every partner's, as the totals row of the view.
///
/// # Errors
/// A database error; `ReportsError::Money` if a stored amount is not whole rupees.
fn venture_run(
    conn: &Connection,
    typed: &Typed,
    key: &AnalyticsKey,
    money: AnalyticsPartnerMoney,
    input: &AnalyticsLinesInput,
    spec: PageSpec,
) -> Result<AnalyticsLinesPage, ReportsError> {
    let from = [typed.from.as_ref(), key.from.as_ref()]
        .into_iter()
        .flatten()
        .max()
        .cloned();
    let to = [typed.to.as_ref(), key.to.as_ref()]
        .into_iter()
        .flatten()
        .min()
        .cloned();
    let postings = match money {
        AnalyticsPartnerMoney::Trips => TRIP_POSTINGS,
        AnalyticsPartnerMoney::Gardens => GARDEN_POSTINGS,
        AnalyticsPartnerMoney::Fruit => FRUIT_POSTINGS,
    };
    let partner = key.partner_id.or(typed.partner_id);
    // Every fragment is a constant of this crate; the values are bound (Code Standards B14).
    let filter = format!(
        "p.venture_account_id IS NOT NULL AND (?1 IS NULL OR p.id = ?1) AND {LIVE}
         AND e.source_type <> 'month_close' AND (?2 IS NULL OR e.entry_date >= ?2)
         AND (?3 IS NULL OR e.entry_date <= ?3) AND {postings}"
    );
    let from_clause = "FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
         JOIN partners p ON p.venture_account_id = l.account_id";
    let args = vec![
        partner.map_or(Value::Null, Value::Integer),
        from.map_or(Value::Null, Value::Text),
        to.map_or(Value::Null, Value::Text),
    ];

    let (total_count, total_dr, total_cr): (i64, i64, i64) = conn.query_row(
        &format!(
            "SELECT COUNT(*), COALESCE(SUM(l.dr_paisa), 0), COALESCE(SUM(l.cr_paisa), 0)
             {from_clause} WHERE {filter}"
        ),
        params_from_iter(args.iter()),
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    )?;

    let mut page_args = args;
    page_args.push(Value::Integer(spec.sql_limit()));
    page_args.push(Value::Integer(spec.offset));
    let raws = conn
        .prepare(&format!(
            "SELECT e.entry_date, e.posting_type, e.source_type, e.source_id,
                    COALESCE(b.number, pu.number, ve.number, fr.number), p.name_ur, p.name_en,
                    l.dr_paisa, l.cr_paisa
             {from_clause}
             LEFT JOIN bikri_entries b ON e.source_type = 'bikri' AND b.id = e.source_id
             LEFT JOIN purchases pu ON e.source_type = 'purchase' AND pu.id = e.source_id
             LEFT JOIN venture_entries ve ON e.source_type = 'venture' AND ve.id = e.source_id
             LEFT JOIN fruit_takes fr ON e.source_type = 'fruit' AND fr.id = e.source_id
             WHERE {filter} ORDER BY e.entry_date, l.id LIMIT ?4 OFFSET ?5"
        ))?
        .query_map(params_from_iter(page_args.iter()), |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, i64>(3)?,
                r.get::<_, Option<i64>>(4)?,
                crate::reports::Names {
                    ur: r.get(5)?,
                    en: r.get(6)?,
                },
                r.get::<_, i64>(7)?,
                r.get::<_, i64>(8)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    let mut rows = Vec::with_capacity(raws.len());
    for (date, posting_type, source_type, source_id, number, names, debit, credit) in raws {
        rows.push(Row {
            cells: vec![
                Cell::Date { iso: date },
                Cell::Doc {
                    type_id: crate::reports::ids::doc_id(&posting_type),
                    number,
                },
                Cell::Names { names },
                Cell::Money {
                    paisa: paisa(credit)?.checked_sub(paisa(debit)?)?,
                },
            ],
            source: Some(source_of(&source_type, source_id)),
            kind: RowKind::Normal,
        });
    }
    Ok(AnalyticsLinesPage {
        view: input.view,
        title_id: ids::LINES_TITLE.to_owned(),
        filters: typed.resolved.clone(),
        columns: venture_columns(),
        rows,
        offset: spec.offset,
        limit: spec.reported_limit(),
        total_count,
        totals_label_id: ids::TOTAL.to_owned(),
        totals: vec![
            Cell::Empty,
            Cell::Empty,
            Cell::Empty,
            Cell::Money {
                paisa: paisa(total_cr)?.checked_sub(paisa(total_dr)?)?,
            },
        ],
    })
}
