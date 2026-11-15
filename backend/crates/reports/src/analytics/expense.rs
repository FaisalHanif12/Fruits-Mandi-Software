//! View M, expenses by head, خرچہ مد وار (PRD 18.2, FR-18.6, FR-18.9, plan 77).
//!
//! One row for each shop expense head (دکان خرچہ, an account of head 05) with its net expense in
//! each day, month or year of the period (`group_by`, month by default, at most 36 columns), and
//! the row total. The numbers are read from the journal, not from the sale facts (FR-18.6): the
//! posted lines of the head, debit less credit, by the date of their entry, with the entries of a
//! month close left out (they only move amounts between accounts, PRD FR-17.6). The conditions
//! are those of the expense report R29, so the totals row equals its shop expense total and the
//! expenses line of R30 for the same dates. A reversal is dated like the entry it undoes, so a
//! cancelled voucher nets to 0 inside its own day.
//!
//! A cell opens the posted lines of its head in its period, each opening its voucher (FR-18.2),
//! and they add up to the cell. The totals row opens every head.
//!
//! Speed (NFR-SPD-24, Code Standards B18, plan 77 robustness finding 1): the read starts where
//! there are fewer rows. A long period (more than 92 days) starts at the head: the 20 or so
//! accounts of head 05, then each account's lines through the covering index
//! `idx_journal_lines_account_amounts`, then each entry by its key for its status, its kind and its
//! day. Ten years hold about 24,000 such lines among 3.5 million journal lines, and the planner's own
//! order (the entries by date, then every line of every entry) read all 3.5 million: 47 seconds. A
//! short period starts at the days (the date index of the entries, then their lines), which reads
//! only the entries of those days: a month is 0.09 seconds of CPU there and 0.3 seconds from the
//! head. `CROSS JOIN` fixes the order in SQLite, so the plan does not depend on the statistics of
//! the file (the shop's file may never have been analysed). The conditions and the figures are the
//! same in both orders.
//!
//! The partner part (`expense_part` partner, PRD 18.2 M, FR-9.5, FR-17.11, plan 77) has one row for
//! each partner with the kharch he spent himself (see `expense_partners`). The kharch page of the
//! dashboard is this view read for one period, so the page and the view can never differ.
//!
//! Example (the worked day, October 2026, by month): بجلی 3,000; the totals row 3,000. Partner part:
//! الف 2,000, ب 0, ج 0, total 2,000.

use std::collections::{BTreeMap, HashMap};

use mandi_core::error::{ErrorCode, FieldError};
use mandi_core::money::Paisa;
use mandi_db::rusqlite::{Connection, params_from_iter};

use super::filters::Typed;
use super::ids;
use super::matrix::period_column;
use super::model::{
    AnalyticsExpensePart, AnalyticsGroupBy, AnalyticsKey, AnalyticsLinesInput, AnalyticsLinesPage,
    AnalyticsRow,
};
use super::table::{Table, col, count_of, row};
use crate::error::ReportsError;
use crate::reports::common::{Conds, add, amount_or_empty, paisa};
use crate::reports::{Cell, ColumnKind, Names, PageSpec, Row, RowKind, RowSource};

/// Most columns of view M.
const MAX_PERIOD_COLUMNS: usize = 36;

/// The conditions of the shop expense lines: posted entries of head 05 accounts, a month close
/// left out, the dates of the run and, for the lines behind a number, the head and the days of
/// the key.
fn conds(typed: &Typed, key: &AnalyticsKey) -> Conds {
    conds_between(typed.from.as_deref(), typed.to.as_deref(), key)
}

/// The conditions for the days of the run (`from`, `to`) and the head and days of the key.
fn conds_between(from: Option<&str>, to: Option<&str>, key: &AnalyticsKey) -> Conds {
    let mut c = Conds::new();
    c.always("e.status = 'posted'")
        .always("e.source_type <> 'month_close'")
        .always("h.code = '05'")
        .text("e.entry_date >= ?", from)
        .text("e.entry_date <= ?", to)
        .text("e.entry_date >= ?", key.from.as_deref())
        .text("e.entry_date <= ?", key.to.as_deref())
        .id("a.id = ?", key.head_id);
    c
}

/// The SQL of the period of a line for the grouping chosen (constants of this crate).
const fn bucket_of(group_by: AnalyticsGroupBy) -> &'static str {
    match group_by {
        AnalyticsGroupBy::Day => "e.entry_date",
        AnalyticsGroupBy::Month => "substr(e.entry_date, 1, 7)",
        AnalyticsGroupBy::Year => "substr(e.entry_date, 1, 4)",
    }
}

/// A period of at most this many days is read from the days of the entries, a longer one (or one
/// with no start or no end) from the expense accounts. On the NFR-SCL-1 file (283 entries a day,
/// 24,000 expense lines) the two cost the same at about four months; this keeps every period on the
/// cheaper side with room to spare.
const DAYS_READ_FROM_THE_DATES: i64 = 92;

/// Where a read of the expense lines starts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReadFrom {
    /// The entries of the days of the period, then their lines.
    TheDates,
    /// The accounts of head 05, then their lines, then the entry of each line.
    TheAccounts,
}

impl ReadFrom {
    /// The tables of the read, in the order they are read. `CROSS JOIN` keeps the order.
    const fn tables(self) -> &'static str {
        match self {
            Self::TheDates => {
                "FROM journal_entries e
     CROSS JOIN journal_lines l ON l.entry_id = e.id
     CROSS JOIN accounts a ON a.id = l.account_id
     CROSS JOIN account_controls c ON c.id = a.control_id
     CROSS JOIN account_heads h ON h.id = c.head_id"
            }
            Self::TheAccounts => {
                "FROM account_heads h
     CROSS JOIN account_controls c ON c.head_id = h.id
     CROSS JOIN accounts a ON a.control_id = c.id
     CROSS JOIN journal_lines l ON l.account_id = a.id
     CROSS JOIN journal_entries e ON e.id = l.entry_id"
            }
        }
    }

    /// The order for a period: the days of the run narrowed by the days of the key. A period with no
    /// start or no end, a date that does not read, or more than [`DAYS_READ_FROM_THE_DATES`] days
    /// starts at the accounts.
    fn of_period(from: Option<&str>, to: Option<&str>, key: &AnalyticsKey) -> Self {
        // `YYYY-MM-DD` sorts as a date: the later start and the earlier end.
        fn both<'a>(
            a: Option<&'a str>,
            b: Option<&'a str>,
            pick: fn(&'a str, &'a str) -> &'a str,
        ) -> Option<&'a str> {
            match (a, b) {
                (Some(a), Some(b)) => Some(pick(a, b)),
                (a, b) => a.or(b),
            }
        }
        let start = both(from, key.from.as_deref(), Ord::max);
        let end = both(to, key.to.as_deref(), Ord::min);
        let day = |text: Option<&str>| {
            text.and_then(|t| chrono::NaiveDate::parse_from_str(t, "%Y-%m-%d").ok())
        };
        match (day(start), day(end)) {
            (Some(start), Some(end))
                if end.signed_duration_since(start).num_days() <= DAYS_READ_FROM_THE_DATES =>
            {
                Self::TheDates
            }
            _ => Self::TheAccounts,
        }
    }
}

/// The query of view M: each head's debit and credit sums in each period of the grouping.
fn heads_sql(read: ReadFrom, group_by: AnalyticsGroupBy, where_: &str) -> String {
    let bucket = bucket_of(group_by);
    let tables = read.tables();
    format!(
        "SELECT a.id, a.name_ur, a.name_en, a.code, {bucket},
                COALESCE(SUM(l.dr_paisa), 0), COALESCE(SUM(l.cr_paisa), 0)
         {tables} WHERE {where_} GROUP BY a.id, {bucket}"
    )
}

/// The count and the sums behind a number of view M.
fn totals_sql(read: ReadFrom, where_: &str) -> String {
    let tables = read.tables();
    format!(
        "SELECT COUNT(*), COALESCE(SUM(l.dr_paisa), 0), COALESCE(SUM(l.cr_paisa), 0)
         {tables} WHERE {where_}"
    )
}

/// One page of the lines behind a number of view M, in day order.
fn page_sql(read: ReadFrom, where_: &str) -> String {
    let tables = read.tables();
    format!(
        "SELECT e.entry_date, e.posting_type, e.source_type, e.source_id, v.number,
                a.name_ur, a.name_en, l.dr_paisa, l.cr_paisa, e.id,
                COALESCE(NULLIF(v.reason, ''), v.narration)
         {tables}
         LEFT JOIN vouchers v ON e.source_type = 'voucher' AND v.id = e.source_id
         WHERE {where_} ORDER BY e.entry_date, l.id LIMIT ? OFFSET ?"
    )
}

/// Builds view M.
///
/// # Errors
/// `ReportsError::Rejected` (`ERR_OUT_OF_RANGE`, `filters.group_by`) when there would be more
/// than 36 columns; a database error; `ReportsError::Money`.
pub(crate) fn run_m(
    conn: &Connection,
    typed: &Typed,
    spec: PageSpec,
) -> Result<Table, ReportsError> {
    if typed.expense_part == AnalyticsExpensePart::Partner {
        return super::expense_partners::run(conn, typed, spec);
    }
    let key = AnalyticsKey::default();
    let c = conds(typed, &key);
    let read = ReadFrom::of_period(typed.from.as_deref(), typed.to.as_deref(), &key);
    let sql = heads_sql(read, typed.group_by, &c.sql());
    let mut statement = conn.prepare(&sql)?;
    let rows = statement
        .query_map(params_from_iter(c.args().iter()), |r| {
            Ok((
                r.get::<_, i64>(0)?,
                Names {
                    ur: r.get(1)?,
                    en: r.get(2)?,
                },
                r.get::<_, String>(3)?,
                r.get::<_, String>(4)?,
                r.get::<_, i64>(5)?,
                r.get::<_, i64>(6)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;

    // A head's net by bucket; a head and a bucket with nothing left have no cell.
    let mut buckets: BTreeMap<String, ()> = BTreeMap::new();
    let mut cells: HashMap<(i64, String), Paisa> = HashMap::new();
    let mut heads: BTreeMap<(String, i64), Names> = BTreeMap::new();
    for (head, names, code, bucket, dr, cr) in rows {
        let net = paisa(dr)?.checked_sub(paisa(cr)?)?;
        if net == Paisa::ZERO {
            continue;
        }
        buckets.insert(bucket.clone(), ());
        heads.insert((code, head), names);
        cells.insert((head, bucket), net);
    }
    if buckets.len() > MAX_PERIOD_COLUMNS {
        return Err(ReportsError::Rejected(vec![FieldError::new(
            ErrorCode::ErrOutOfRange,
            "filters.group_by",
        )]));
    }
    let bucket_list: Vec<String> = buckets.into_keys().collect();

    let mut columns = vec![col("head", ids::COL_HEAD, ColumnKind::Names)];
    for bucket in &bucket_list {
        columns.push(period_column(typed.group_by, bucket)?);
    }
    columns.push(col("total", ids::TOTAL, ColumnKind::Money));

    // The biggest heads first, then by code.
    let mut entries: Vec<(i64, Names, Paisa, Vec<Paisa>)> = Vec::new();
    for ((_, head), names) in &heads {
        let mut total = Paisa::ZERO;
        let mut per: Vec<Paisa> = Vec::with_capacity(bucket_list.len());
        for bucket in &bucket_list {
            let amount = cells
                .get(&(*head, bucket.clone()))
                .copied()
                .unwrap_or(Paisa::ZERO);
            add(&mut total, amount)?;
            per.push(amount);
        }
        entries.push((*head, names.clone(), total, per));
    }
    // `heads` is ordered by code, so a stable sort by total keeps the code order for ties.
    entries.sort_by_key(|entry| std::cmp::Reverse(entry.2));

    let mut column_totals = vec![Paisa::ZERO; bucket_list.len()];
    let mut grand = Paisa::ZERO;
    let mut all_rows: Vec<AnalyticsRow> = Vec::with_capacity(entries.len());
    for (head, names, total, per) in &entries {
        let mut out = vec![Cell::Names {
            names: names.clone(),
        }];
        for (index, amount) in per.iter().enumerate() {
            if let Some(slot) = column_totals.get_mut(index) {
                add(slot, *amount)?;
            }
            out.push(amount_or_empty(*amount));
        }
        out.push(Cell::Money { paisa: *total });
        add(&mut grand, *total)?;
        all_rows.push(row(
            out,
            AnalyticsKey {
                head_id: Some(*head),
                ..AnalyticsKey::default()
            },
        ));
    }
    let mut totals = vec![Cell::Empty];
    totals.extend(column_totals.into_iter().map(amount_or_empty));
    totals.push(Cell::Money { paisa: grand });
    Ok(Table {
        charts: Vec::new(),
        columns,
        total_count: count_of(all_rows.len()),
        rows: spec.slice(&all_rows),
        totals,
    })
}

/// The lines behind a number of view M: the posted journal lines of the head (or of every head),
/// inside the period of the run and of the key, each its debit less its credit, so they add up to
/// exactly the number that was opened. Each opens its voucher.
///
/// # Errors
/// A database error; `ReportsError::Money`.
pub(crate) fn lines(
    conn: &Connection,
    typed: &Typed,
    key: &AnalyticsKey,
    input: &AnalyticsLinesInput,
    spec: PageSpec,
) -> Result<AnalyticsLinesPage, ReportsError> {
    let c = conds(typed, key);
    let read = ReadFrom::of_period(typed.from.as_deref(), typed.to.as_deref(), key);
    let (total_count, total_dr, total_cr): (i64, i64, i64) = conn.query_row(
        &totals_sql(read, &c.sql()),
        params_from_iter(c.args().iter()),
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    )?;
    let args = c.page_args(spec.sql_limit(), spec.offset);
    let raws = conn
        .prepare(&page_sql(read, &c.sql()))?
        .query_map(params_from_iter(args.iter()), |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, i64>(3)?,
                r.get::<_, Option<i64>>(4)?,
                Names {
                    ur: r.get(5)?,
                    en: r.get(6)?,
                },
                r.get::<_, i64>(7)?,
                r.get::<_, i64>(8)?,
                r.get::<_, i64>(9)?,
                r.get::<_, Option<String>>(10)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    let mut rows = Vec::with_capacity(raws.len());
    for (date, posting_type, source_type, source_id, number, names, dr, cr, entry_id, reason) in
        raws
    {
        rows.push(Row {
            cells: vec![
                Cell::Date { iso: date },
                Cell::Doc {
                    type_id: crate::reports::ids::doc_id(&posting_type),
                    number,
                },
                Cell::Names { names },
                super::expense_partners::reason_cell(reason),
                Cell::Money {
                    paisa: paisa(dr)?.checked_sub(paisa(cr)?)?,
                },
            ],
            source: Some(RowSource::of_stored(
                &source_type,
                source_id,
                entry_id,
                None,
                None,
            )),
            kind: RowKind::Normal,
        });
    }
    Ok(AnalyticsLinesPage {
        view: input.view,
        title_id: ids::LINES_TITLE.to_owned(),
        filters: typed.resolved.clone(),
        columns: super::expense_partners::lines_columns(),
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
            Cell::Money {
                paisa: paisa(total_dr)?.checked_sub(paisa(total_cr)?)?,
            },
        ],
    })
}

/// The query plans of view M and its lines (Code Standards B18, NFR-SPD-24, plan 77 robustness
/// findings 1 and 4). The shop part starts at the accounts of head 05 and the partner part at the
/// partners, and both read the journal lines through the covering index of the account, so a read of
/// ten years touches only the lines of those accounts and never every line of every entry of the
/// period. The text is the one the code runs, on a data file with the real indexes.
#[cfg(test)]
mod tests {
    use mandi_db::clock::FixedClock;
    use mandi_db::migrate::migrate;
    use mandi_db::rusqlite::{Connection, params_from_iter};

    use super::*;

    /// The planner statistics of the ten-year file of NFR-SCL-1 (1.03 million entries, 3.5 million
    /// lines, 20,000 accounts), as `ANALYZE` wrote them there (the small `partners` table had none). The plan on an empty file is not the
    /// plan on a full one: without these rows the planner already prefers the account index, and the
    /// test would pass on the join order that took 47 seconds.
    const VOLUME_STATS: [(&str, &str, &str); 11] = [
        ("account_controls", "idx_account_controls_head", "14 3"),
        ("account_heads", "sqlite_autoindex_account_heads_1", "6 1"),
        ("accounts", "idx_accounts_control", "20017 2503"),
        ("accounts", "idx_accounts_type_serial", "20017 6673 1"),
        ("accounts", "sqlite_autoindex_accounts_1", "20017 1"),
        (
            "journal_entries",
            "idx_journal_entries_entry_date",
            "1030000 283",
        ),
        (
            "journal_entries",
            "idx_journal_entries_source",
            "1030000 1030000 1",
        ),
        (
            "journal_lines",
            "idx_journal_lines_account_amounts",
            "3503940 195 1 1 1",
        ),
        (
            "journal_lines",
            "idx_journal_lines_account_entry",
            "3503940 195 1",
        ),
        (
            "journal_lines",
            "idx_journal_lines_account_person",
            "3503940 195 195",
        ),
        (
            "journal_lines",
            "sqlite_autoindex_journal_lines_1",
            "3503940 4 1",
        ),
    ];

    /// A data file with every migration run and the statistics of the ten-year file, reopened so
    /// the planner reads them. The folder is returned to keep it alive.
    fn volume_like() -> (tempfile::TempDir, Connection) {
        let dir = tempfile::tempdir().expect("a temporary folder");
        let path = dir.path().join("mandi.db");
        {
            let mut conn = Connection::open(&path).expect("a data file");
            let clock = FixedClock(chrono::DateTime::<chrono::Utc>::UNIX_EPOCH);
            migrate(&mut conn, dir.path(), &clock).expect("the migrations run");
            conn.execute_batch("ANALYZE; DELETE FROM sqlite_stat1;")
                .expect("the statistics table is made");
            for (table, index, stat) in VOLUME_STATS {
                conn.execute(
                    "INSERT INTO sqlite_stat1 (tbl, idx, stat) VALUES (?1, ?2, ?3)",
                    [table, index, stat],
                )
                .expect("a statistics row");
            }
        }
        let conn = Connection::open(&path).expect("the data file again");
        (dir, conn)
    }

    fn plan(
        conn: &Connection,
        sql: &str,
        args: &[mandi_db::rusqlite::types::Value],
    ) -> Vec<String> {
        let mut statement = conn
            .prepare(&format!("EXPLAIN QUERY PLAN {sql}"))
            .expect("the query is valid SQL");
        statement
            .query_map(params_from_iter(args.iter()), |r| r.get::<_, String>(3))
            .expect("the plan is read")
            .map(|line| line.expect("a plan line"))
            .collect()
    }

    /// A read from the accounts: the lines through the account index, the entries by key. The steps
    /// that start at the entries by date, or read a table whole, are what made ten years take 17 to
    /// 47 seconds.
    fn assert_driven_from_the_accounts(steps: &[String], what: &str) {
        let text = steps.join("\n");
        assert!(
            steps.iter().any(|s| s.starts_with(
                "SEARCH l USING COVERING INDEX idx_journal_lines_account_amounts (account_id=?)"
            )),
            "{what}: the lines are not found through the account index:\n{text}"
        );
        assert!(
            steps
                .iter()
                .any(|s| s.starts_with("SEARCH e USING INTEGER PRIMARY KEY")),
            "{what}: the entries are not found by key:\n{text}"
        );
        assert!(
            !text.contains("idx_journal_entries_entry_date")
                && !text.contains("sqlite_autoindex_journal_lines_1")
                && !steps.iter().any(|s| s.starts_with("SCAN l")
                    || s.starts_with("SCAN e")
                    || s.starts_with("SCAN journal")),
            "{what}: the read starts at the entries or scans the journal:\n{text}"
        );
    }

    /// A read from the dates: the entries through the date index, their lines through the entry.
    fn assert_driven_from_the_dates(steps: &[String], what: &str) {
        let text = steps.join("\n");
        assert!(
            steps.iter().any(|s| s
                .starts_with("SEARCH e USING INDEX idx_journal_entries_entry_date (entry_date")),
            "{what}: the entries are not found through the date index:\n{text}"
        );
        // One head: the planner also knows the account of the line, and probes it with the entry.
        assert!(
            steps.iter().any(|s| s.starts_with(
                "SEARCH l USING INDEX sqlite_autoindex_journal_lines_1 (entry_id=?)"
            ) || s.starts_with(
                "SEARCH l USING COVERING INDEX idx_journal_lines_account_amounts (account_id=? AND entry_id=?)"
            )),
            "{what}: the lines are not found through their entry:\n{text}"
        );
        assert!(
            !steps
                .iter()
                .any(|s| s.starts_with("SCAN l") || s.starts_with("SCAN e")),
            "{what}: the read scans the journal:\n{text}"
        );
    }

    fn assert_read(read: ReadFrom, steps: &[String], what: &str) {
        match read {
            ReadFrom::TheAccounts => assert_driven_from_the_accounts(steps, what),
            ReadFrom::TheDates => assert_driven_from_the_dates(steps, what),
        }
    }

    #[test]
    fn a_long_period_starts_at_the_expense_accounts_and_a_short_one_at_the_days() {
        let (_dir, conn) = volume_like();
        let whole = AnalyticsKey::default();
        let one_head = AnalyticsKey {
            head_id: Some(1),
            ..AnalyticsKey::default()
        };
        let october_2021 = AnalyticsKey {
            from: Some("2021-10-01".into()),
            to: Some("2021-10-31".into()),
            ..AnalyticsKey::default()
        };
        // (what, from, to, key, where the read starts)
        type Period<'a> = (
            &'a str,
            Option<&'a str>,
            Option<&'a str>,
            &'a AnalyticsKey,
            ReadFrom,
        );
        let periods: [Period<'_>; 12] = [
            (
                "ten years",
                Some("2016-10-11"),
                Some("2026-10-10"),
                &whole,
                ReadFrom::TheAccounts,
            ),
            (
                "ten years, one head",
                Some("2016-10-11"),
                Some("2026-10-10"),
                &one_head,
                ReadFrom::TheAccounts,
            ),
            (
                "a year",
                Some("2021-01-01"),
                Some("2021-12-31"),
                &whole,
                ReadFrom::TheAccounts,
            ),
            ("no dates at all", None, None, &whole, ReadFrom::TheAccounts),
            (
                "no end",
                Some("2026-01-01"),
                None,
                &whole,
                ReadFrom::TheAccounts,
            ),
            (
                "a date that does not read",
                Some("2026-13-45"),
                Some("2026-10-10"),
                &whole,
                ReadFrom::TheAccounts,
            ),
            // 01-01-2026 to 04-04-2026 is 93 days after the first day; to 03-04-2026 is 92.
            (
                "93 days",
                Some("2026-01-01"),
                Some("2026-04-04"),
                &whole,
                ReadFrom::TheAccounts,
            ),
            (
                "92 days",
                Some("2026-01-01"),
                Some("2026-04-03"),
                &whole,
                ReadFrom::TheDates,
            ),
            (
                "one day",
                Some("2026-10-10"),
                Some("2026-10-10"),
                &whole,
                ReadFrom::TheDates,
            ),
            (
                "a month, one head",
                Some("2021-10-01"),
                Some("2021-10-31"),
                &one_head,
                ReadFrom::TheDates,
            ),
            // The days of a row's key narrow the period of the run.
            (
                "ten years narrowed by the key to a month",
                Some("2016-10-11"),
                Some("2026-10-10"),
                &october_2021,
                ReadFrom::TheDates,
            ),
            (
                "no dates narrowed by the key to a month",
                None,
                None,
                &october_2021,
                ReadFrom::TheDates,
            ),
        ];
        for (what, from, to, key, expected) in periods {
            let read = ReadFrom::of_period(from, to, key);
            assert_eq!(read, expected, "{what}: where the read starts");
            let c = conds_between(from, to, key);
            let args = c.args().to_vec();
            for group_by in [
                AnalyticsGroupBy::Day,
                AnalyticsGroupBy::Month,
                AnalyticsGroupBy::Year,
            ] {
                assert_read(
                    read,
                    &plan(&conn, &heads_sql(read, group_by, &c.sql()), &args),
                    &format!("view M {group_by:?}, {what}"),
                );
            }
            assert_read(
                read,
                &plan(&conn, &totals_sql(read, &c.sql()), &args),
                &format!("lines total, {what}"),
            );
            assert_read(
                read,
                &plan(&conn, &page_sql(read, &c.sql()), &c.page_args(100, 0)),
                &format!("lines page, {what}"),
            );
        }
    }

    #[test]
    fn the_partner_entries_start_at_the_drawings_accounts_and_not_at_the_days() {
        use mandi_db::rusqlite::types::Value;
        let (_dir, conn) = volume_like();
        let sql = super::super::expense_partners::taken_sql();
        for (partner, from, to) in [
            (
                Value::Null,
                Value::Text("2016-10-11".into()),
                Value::Text("2026-10-10".into()),
            ),
            (
                Value::Integer(1),
                Value::Text("2016-10-11".into()),
                Value::Text("2026-10-10".into()),
            ),
            (Value::Null, Value::Null, Value::Null),
        ] {
            let steps = plan(&conn, &sql, &[partner, from, to]);
            assert!(
                steps.first().is_some_and(|s| s.starts_with("SCAN p")),
                "the partner entries do not start at the partners:\n{}",
                steps.join("\n")
            );
            assert_driven_from_the_accounts(&steps, "partner entries");
        }
    }
}
