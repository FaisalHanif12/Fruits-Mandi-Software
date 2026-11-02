//! The speed rewrites of plan 48 P48-S3 give the very rows the old SQL gave (PRD FR-6.1, FR-19.9,
//! FR-7.59, FR-15.4; NFR-AI-3, NFR-SPD-2, NFR-SPD-10). The old SQL is copied here verbatim from the
//! code before the change, run on the same book, and compared row by row, whole rows, sorted:
//!
//! - a1: the balances of the buyers and the sellers before a day (R34 base rows and the buyers'
//!   dues of R34): `party_reads::totals_before_by`, both methods and the one it picks;
//! - a2: the credits of the sellers behind the age of what is payable (`held_credits`);
//! - a3: the open bills of the buyers behind the ages of R20 (`open_buyer_bills_by`, line by line
//!   and in bulk);
//! - a4: the totals of the buyers or the sellers only (`ledger::account_totals_in_controls`).
//!
//! The books: the worked day (PRD 4.9) and the E2E runs the narrow-tile tests use (E2E-15, 21, 28,
//! 32, 34, 36, 40, 43), the hand-made cases of the money review (a draft reversal, a keyed line on
//! an ordinary account, a draft with lines), and a random book made through the services with a
//! fixed seed. The expected numbers are the PRD's; no golden number is read or changed here.
#![cfg(test)]

mod common;
mod reports_world;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use common::*;
use mandi_db::ledger::{Totals, account_totals, account_totals_in_controls};
use mandi_db::party_reads::{
    self, BeforeMethod, OpenBill, OpenMethod, PartyScope, held_credits, open_bills_per_line,
    open_buyer_bills_by, totals_before, totals_before_by,
};
use mandi_db::rusqlite::types::Value;
use mandi_db::rusqlite::{Connection, params_from_iter};
use mandi_reports::reports::{Cell, ReportFilters, ReportId, day_query_forms};
use mandi_services::core::accounts::AccountType;
use mandi_services::core::bikri::{LineChargeMethod, PaymentType, PukhtaInput};
use mandi_services::{bikri, vouchers};
use reports_world::*;
use support::World;

const BOTH: [&str; 2] = ["0304", "03011"];
const BUYERS: [&str; 1] = ["0304"];
const SELLERS: [&str; 1] = ["03011"];

fn read<T>(w: &World, action: impl FnOnce(&Connection) -> mandi_db::rusqlite::Result<T>) -> T {
    w.env.ctx.db().read(action).unwrap()
}

// ---- the OLD SQL, verbatim from before plan 48 P48-S3 ---------------------------------------------

/// `figures_load::base_rows`, with the party filter of `Filter::sql` appended by the caller.
fn old_base_sql(filter: &str) -> String {
    format!(
        "SELECT l.account_id, COALESCE(l.pukhta_person_key, ''), k.code,
                COALESCE(SUM(l.dr_paisa), 0), COALESCE(SUM(l.cr_paisa), 0)
         FROM journal_lines l
         JOIN journal_entries e ON e.id = l.entry_id
         JOIN accounts a ON a.id = l.account_id
         JOIN account_controls k ON k.id = a.control_id
         WHERE k.code IN ('0304', '03011') AND e.status = 'posted'
           AND e.entry_date < ?1{filter}
         GROUP BY l.account_id, l.pukhta_person_key, k.code"
    )
}

/// `day::buyer_moves`, the balances before `from`.
const OLD_BUYER_BEFORE: &str = "SELECT ?1, l.account_id, COALESCE(l.pukhta_person_key, ''),
                       COALESCE(SUM(l.dr_paisa), 0), COALESCE(SUM(l.cr_paisa), 0)
                FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
                JOIN accounts a ON a.id = l.account_id
                JOIN account_controls k ON k.id = a.control_id
                WHERE e.status = 'posted' AND k.code = '0304' AND e.entry_date < ?1 GROUP BY l.account_id, l.pukhta_person_key";

/// `attention::held_by_key`, with the `LIVE` text of `reports::common`.
const OLD_HELD: &str = "SELECT l.account_id, l.pukhta_person_key, e.posting_type = 'opening', e.entry_date,
                l.cr_paisa,
                COALESCE((SELECT SUM(a.amount_paisa) FROM advance_allocations a
                          WHERE a.credit_line_id = l.id AND a.is_active = 1), 0)
         FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
         JOIN accounts ac ON ac.id = l.account_id
         JOIN account_controls k ON k.id = ac.control_id
         WHERE k.code = '03011' AND l.cr_paisa > 0 AND e.status = 'posted' AND e.reverses_entry_id IS NULL
    AND NOT EXISTS (SELECT 1 FROM journal_entries r WHERE r.reverses_entry_id = e.id AND r.status = 'posted')
           AND (e.posting_type = 'opening' OR e.entry_date >= ?1)";

/// `dues::open_bills`, for one chunk of accounts (the test never has more than 500).
fn old_open_sql(count: usize) -> String {
    let matched = "COALESCE((SELECT SUM(a.amount_paisa) FROM bill_allocations a
                                WHERE a.bill_line_id = l.id AND a.is_active = 1), 0)";
    format!(
        "SELECT l.id, l.account_id, l.pukhta_person_key, e.id, e.entry_date, l.dr_paisa, {matched}
             FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
             WHERE l.account_id IN ({marks}) AND l.dr_paisa > 0 AND e.status = 'posted' AND e.reverses_entry_id IS NULL
    AND NOT EXISTS (SELECT 1 FROM journal_entries r WHERE r.reverses_entry_id = e.id AND r.status = 'posted')
               AND l.dr_paisa > {matched}
             ORDER BY e.entry_date, e.id, l.line_no",
        marks = vec!["?"; count].join(", ")
    )
}

// ---- the comparisons ----------------------------------------------------------------------------

/// (account, person, control code, Σ debit, Σ credit).
type BaseRow = (i64, String, String, i64, i64);

fn old_base(
    conn: &Connection,
    from: &str,
    account: Option<i64>,
    person: Option<&str>,
) -> Vec<BaseRow> {
    let mut args = vec![Value::Text(from.to_owned())];
    let mut filter = String::new();
    if let Some(a) = account {
        args.push(Value::Integer(a));
        filter.push_str(&format!(" AND l.account_id = ?{}", args.len()));
    }
    if let Some(p) = person {
        args.push(Value::Text(p.to_owned()));
        filter.push_str(&format!(" AND l.pukhta_person_key = ?{}", args.len()));
    }
    let mut statement = conn.prepare(&old_base_sql(&filter)).unwrap();
    let mut rows: Vec<BaseRow> = statement
        .query_map(params_from_iter(args.iter()), |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
        })
        .unwrap()
        .map(Result::unwrap)
        .collect();
    rows.sort();
    rows
}

fn new_base(
    conn: &Connection,
    method: Option<BeforeMethod>,
    controls: &[&str],
    from: &str,
    scope: PartyScope<'_>,
) -> Vec<BaseRow> {
    let got = match method {
        Some(m) => totals_before_by(conn, m, controls, from, scope),
        None => totals_before(conn, controls, from, scope),
    }
    .unwrap();
    let mut rows: Vec<BaseRow> = got
        .into_iter()
        .map(|t| {
            (
                t.account_id,
                t.person_key,
                t.control_code,
                t.dr_paisa,
                t.cr_paisa,
            )
        })
        .collect();
    rows.sort();
    rows
}

/// The days the books are asked about: before everything, the worked day's own days, after all.
const FROMS: [&str; 9] = [
    "2000-01-01",
    "2026-08-30",
    "2026-08-31",
    "2026-09-01",
    "2026-10-09",
    "2026-10-10",
    "2026-10-11",
    "2026-10-12",
    "2030-01-01",
];

/// a1: every method, every day, equals the old SQL; the buyers' part equals the old buyers' SQL.
fn check_a1(w: &World, label: &str) {
    read(w, |conn| a1_on(conn, label, &FROMS, true));
}

fn a1_on(
    conn: &Connection,
    label: &str,
    froms: &[&str],
    with_slips: bool,
) -> mandi_db::rusqlite::Result<()> {
    {
        for from in froms.iter().copied() {
            let old = old_base(conn, from, None, None);
            for method in [
                None,
                Some(BeforeMethod::PerLine),
                Some(BeforeMethod::LeftOut),
            ] {
                assert_eq!(
                    new_base(conn, method, &BOTH, from, PartyScope::default()),
                    old,
                    "{label}: base rows before {from} by {method:?}"
                );
            }
            // The buyers alone, as `buyer_moves` reads them.
            let mut old_buyers: Vec<(i64, String, i64, i64)> = {
                let mut statement = conn.prepare(OLD_BUYER_BEFORE)?;
                statement
                    .query_map([from], |r| Ok((r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)))?
                    .map(Result::unwrap)
                    .collect()
            };
            old_buyers.sort();
            for method in [
                None,
                Some(BeforeMethod::PerLine),
                Some(BeforeMethod::LeftOut),
            ] {
                let mut new_buyers: Vec<(i64, String, i64, i64)> =
                    new_base(conn, method, &BUYERS, from, PartyScope::default())
                        .into_iter()
                        .map(|(a, p, _, dr, cr)| (a, p, dr, cr))
                        .collect();
                new_buyers.sort();
                assert_eq!(
                    new_buyers, old_buyers,
                    "{label}: buyers before {from} by {method:?}"
                );
            }
        }
        if !with_slips {
            return Ok(());
        }
        // A slip: one account, and one person of a shared account.
        let mut statement = conn.prepare(
            "SELECT DISTINCT l.account_id, l.pukhta_person_key FROM journal_lines l
             JOIN accounts a ON a.id = l.account_id JOIN account_controls k ON k.id = a.control_id
             WHERE k.code IN ('0304', '03011')",
        )?;
        let parties: Vec<(i64, Option<String>)> = statement
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
            .map(Result::unwrap)
            .collect();
        for from in ["2026-09-01", "2026-10-10", "2026-10-11", "2030-01-01"] {
            for (account, person) in &parties {
                let scope = PartyScope {
                    account_id: Some(*account),
                    person_key: person.as_deref(),
                };
                assert_eq!(
                    new_base(conn, None, &BOTH, from, scope),
                    old_base(conn, from, Some(*account), person.as_deref()),
                    "{label}: slip of {account} {person:?} before {from}"
                );
                let by_account = PartyScope {
                    account_id: Some(*account),
                    person_key: None,
                };
                assert_eq!(
                    new_base(conn, None, &BOTH, from, by_account),
                    old_base(conn, from, Some(*account), None),
                    "{label}: slip of {account} before {from}"
                );
            }
        }
        Ok(())
    }
}

/// a2: the held credits, whole rows.
fn check_a2(w: &World, label: &str) {
    read(w, |conn| a2_on(conn, label));
}

fn a2_on(conn: &Connection, label: &str) -> mandi_db::rusqlite::Result<()> {
    {
        for cutoff in [
            "2000-01-01",
            "2026-08-31",
            "2026-09-01",
            "2026-09-02",
            "2026-10-03",
            "2026-10-10",
            "2026-10-11",
            "2030-01-01",
        ] {
            let mut statement = conn.prepare(OLD_HELD)?;
            let mut old: Vec<(i64, Option<String>, bool, String, i64, i64)> = statement
                .query_map([cutoff], |r| {
                    Ok((
                        r.get(0)?,
                        r.get(1)?,
                        r.get(2)?,
                        r.get(3)?,
                        r.get(4)?,
                        r.get(5)?,
                    ))
                })?
                .map(Result::unwrap)
                .collect();
            old.sort();
            let mut new: Vec<(i64, Option<String>, bool, String, i64, i64)> =
                held_credits(conn, "03011", cutoff)?
                    .into_iter()
                    .map(|h| {
                        (
                            h.account_id,
                            h.person_key,
                            h.is_opening,
                            h.entry_date,
                            h.cr_paisa,
                            h.adjusted_paisa,
                        )
                    })
                    .collect();
            new.sort();
            assert_eq!(new, old, "{label}: held credits from {cutoff}");
        }
        Ok(())
    }
}

/// The open bills as sorted whole rows.
fn open_rows(bills: Vec<OpenBill>) -> Vec<(i64, i64, Option<String>, i64, String, i64)> {
    let mut rows: Vec<_> = bills
        .into_iter()
        .map(|b| {
            (
                b.line_id,
                b.account_id,
                b.person_key,
                b.entry_id,
                b.entry_date,
                b.open_paisa,
            )
        })
        .collect();
    rows.sort();
    rows
}

fn old_open(conn: &Connection, ids: &[i64]) -> Vec<(i64, i64, Option<String>, i64, String, i64)> {
    let mut statement = conn.prepare(&old_open_sql(ids.len())).unwrap();
    let mut rows: Vec<_> = statement
        .query_map(params_from_iter(ids.iter()), |r| {
            let dr: i64 = r.get(5)?;
            let matched: i64 = r.get(6)?;
            Ok((
                r.get(0)?,
                r.get(1)?,
                r.get(2)?,
                r.get(3)?,
                r.get(4)?,
                dr - matched,
            ))
        })
        .unwrap()
        .map(Result::unwrap)
        .collect();
    rows.sort();
    rows
}

/// The accounts of the buyer control, ordinary and shared.
fn buyer_ids(conn: &Connection) -> Vec<i64> {
    let mut statement = conn
        .prepare(
            "SELECT a.id FROM accounts a JOIN account_controls k ON k.id = a.control_id
             WHERE k.code = '0304' ORDER BY a.id",
        )
        .unwrap();
    statement
        .query_map([], |r| r.get(0))
        .unwrap()
        .map(Result::unwrap)
        .collect()
}

/// a3: the open bills, line by line, in bulk and as picked, equal the old SQL; every open amount is
/// above 0 and whole rupees.
fn check_a3(w: &World, label: &str) {
    read(w, |conn| {
        a3_on(
            conn,
            label,
            &[OpenMethod::Auto, OpenMethod::PerLine, OpenMethod::Bulk],
            true,
        )
    });
}

fn a3_on(
    conn: &Connection,
    label: &str,
    methods: &[OpenMethod],
    each_account: bool,
) -> mandi_db::rusqlite::Result<()> {
    {
        let ids = buyer_ids(conn);
        let old = old_open(conn, &ids);
        for method in methods.iter().copied() {
            assert_eq!(
                open_rows(open_buyer_bills_by(conn, &ids, method)?),
                old,
                "{label}: open bills by {method:?}"
            );
        }
        if methods.contains(&OpenMethod::PerLine) {
            assert_eq!(
                open_rows(open_bills_per_line(conn, &ids)?),
                old,
                "{label}: per line"
            );
        }
        for row in &old {
            assert!(
                row.5 > 0 && row.5 % 100 == 0,
                "{label}: an open amount {row:?}"
            );
        }
        // One account at a time, as a report for one buyer asks.
        if each_account {
            for id in &ids {
                assert_eq!(
                    open_rows(open_buyer_bills_by(conn, &[*id], OpenMethod::Bulk)?),
                    old_open(conn, &[*id]),
                    "{label}: open bills of {id}"
                );
            }
        }
        Ok(())
    }
}

/// a4: the totals of the buyers or the sellers equal `account_totals` filtered to the control, for
/// all time and for days; and the base at a day after everything equals those totals (P2).
fn check_a4(w: &World, label: &str) {
    read(w, |conn| a4_on(conn, label));
}

fn a4_on(conn: &Connection, label: &str) -> mandi_db::rusqlite::Result<()> {
    {
        for (controls, code) in [(&BUYERS, "0304"), (&SELLERS, "03011")] {
            let members: std::collections::HashSet<i64> = {
                let mut statement = conn.prepare(
                    "SELECT a.id FROM accounts a JOIN account_controls k ON k.id = a.control_id
                     WHERE k.code = ?1",
                )?;
                statement
                    .query_map([code], |r| r.get(0))?
                    .map(Result::unwrap)
                    .collect()
            };
            for as_of in [
                None,
                Some("2026-08-31"),
                Some("2026-10-09"),
                Some("2026-10-10"),
                Some("2030-01-01"),
            ] {
                let old: Vec<(i64, Totals)> = account_totals(conn, as_of)?
                    .into_iter()
                    .filter(|(id, _)| members.contains(id))
                    .collect();
                let new = account_totals_in_controls(conn, controls, as_of)?;
                assert_eq!(new, old, "{label}: totals of {code} as of {as_of:?}");
            }
        }
        // P2: the base after the last day is the trial balance of the buyers and the sellers.
        let mut base_dr = 0_i64;
        let mut base_cr = 0_i64;
        for t in totals_before(conn, &BOTH, "2030-01-01", PartyScope::default())? {
            base_dr += t.dr_paisa;
            base_cr += t.cr_paisa;
        }
        let members: std::collections::HashSet<i64> = {
            let mut statement = conn.prepare(
                "SELECT a.id FROM accounts a JOIN account_controls k ON k.id = a.control_id
                 WHERE k.code IN ('0304', '03011')",
            )?;
            statement
                .query_map([], |r| r.get(0))?
                .map(Result::unwrap)
                .collect()
        };
        let (mut all_dr, mut all_cr) = (0_i64, 0_i64);
        for (id, t) in account_totals(conn, None)? {
            if members.contains(&id) {
                all_dr += t.dr_paisa;
                all_cr += t.cr_paisa;
            }
        }
        assert_eq!(
            (base_dr, base_cr),
            (all_dr, all_cr),
            "{label}: the base ties to the trial balance"
        );
        Ok(())
    }
}

/// The rows compared are not empty: a test that compares two empty lists proves nothing.
fn assert_rows_exist(w: &World, label: &str, base_after: usize, open: usize, held: usize) {
    read(w, |conn| {
        let base = old_base(conn, "2030-01-01", None, None).len();
        let bills = old_open(conn, &buyer_ids(conn)).len();
        let mut statement = conn.prepare(OLD_HELD)?;
        let credits = statement
            .query_map(["2000-01-01"], |r| r.get::<_, i64>(0))?
            .count();
        assert!(base >= base_after, "{label}: only {base} base rows");
        assert!(bills >= open, "{label}: only {bills} open bills");
        assert!(credits >= held, "{label}: only {credits} held credits");
        Ok(())
    });
}

fn check_all(w: &World, label: &str) {
    check_forms(w, label);
    check_r20_ages(w, label);
    check_a1(w, label);
    check_a2(w, label);
    check_a3(w, label);
    check_a4(w, label);
}

// ---- the day queries of R34: the entries-first form against the lines-first form --------------------

/// The days asked of the day queries: single days and ranges, from before everything to after it.
const DAY_RANGES: [(&str, &str); 7] = [
    ("2000-01-01", "2000-01-01"),
    ("2026-08-31", "2026-08-31"),
    ("2026-09-01", "2026-09-01"),
    ("2026-10-10", "2026-10-10"),
    ("2026-10-11", "2026-10-11"),
    ("2026-08-31", "2026-10-12"),
    ("2000-01-01", "2030-01-01"),
];

/// The four rewritten day queries read in both forms give the same rows. `ledger_rows`,
/// `seller_lines` (now a total order) and `lot_charges` are held to the same ORDER; the buyers'
/// movements are ordered by day only, so their rows are compared sorted. Returns how many rows
/// were compared, so a book can say it compared something.
fn forms_on(conn: &Connection, label: &str, ranges: &[(&str, &str)]) -> usize {
    let mut compared = 0;
    for (from, to) in ranges.iter().copied() {
        let ledger = day_query_forms::ledger_rows(conn, from, to).unwrap();
        assert_eq!(
            ledger.entries_first, ledger.lines_first,
            "{label}: ledger rows {from} {to}"
        );
        let sellers = day_query_forms::seller_lines(conn, from, to).unwrap();
        assert_eq!(
            sellers.entries_first, sellers.lines_first,
            "{label}: seller lines {from} {to}"
        );
        let lots = day_query_forms::lot_charges(conn, from, to).unwrap();
        assert_eq!(
            lots.entries_first, lots.lines_first,
            "{label}: lot charges {from} {to}"
        );
        let mut moves = day_query_forms::buyer_moves(conn, from, to).unwrap();
        moves.entries_first.sort();
        moves.lines_first.sort();
        assert_eq!(
            moves.entries_first, moves.lines_first,
            "{label}: buyer moves {from} {to}"
        );
        compared += ledger.entries_first.len()
            + sellers.entries_first.len()
            + lots.entries_first.len()
            + moves.entries_first.len();
    }
    compared
}

fn check_forms(w: &World, label: &str) {
    read(w, |conn| {
        forms_on(conn, label, &DAY_RANGES);
        Ok(())
    });
}

// ---- the ages of R20, held to the old rule ------------------------------------------------------

/// Days from `a` to `b` for two `YYYY-MM-DD` texts.
fn days_between(a: &str, b: &str) -> i64 {
    let parse = |t: &str| mandi_services::core::dates::BusinessDate::parse_iso(t).unwrap();
    parse(b).days_after(parse(a))
}

/// The age bands of the old rule (0 to 7, 8 to 30, 31 to 90, 91 and more days), worked out here from
/// the OLD SQL rows, and compared with every age cell of R20, row by row: the oldest day, its age,
/// the four bands. A buyer with no open bill has no day and four empty bands. Also, for a buyer who
/// owes, the bands add up to his balance (reported by the PRD's matching, FR-15.4).
fn check_r20_ages(w: &World, label: &str) {
    let today = mandi_reports::dashboard::get(
        w.env.ctx.db(),
        mandi_reports::dashboard::BackupFacts {
            cloud_on: false,
            cloud_age_hours: None,
        },
    )
    .unwrap()
    .today;
    let old = read(w, |conn| Ok(old_open(conn, &buyer_ids(conn))));
    let mut expected: std::collections::HashMap<(i64, Option<String>), (String, [i64; 4])> =
        std::collections::HashMap::new();
    for (_, account, person, _, date, open) in old {
        let days = days_between(&date, &today).max(0);
        let band = [7, 30, 90].iter().position(|end| days <= *end).unwrap_or(3);
        let entry = expected
            .entry((account, person))
            .or_insert((date.clone(), [0; 4]));
        if date < entry.0 {
            entry.0 = date;
        }
        entry.1[band] += open;
    }
    let page = report(w, ReportId::R20, ReportFilters::default(), 0, 1_000);
    let col = |key: &str| page.columns.iter().position(|c| c.key == key).unwrap();
    let (buyer, balance, oldest, days_col) = (
        col("buyer"),
        col("balance"),
        col("oldest_date"),
        col("days"),
    );
    let bands = [
        col("age_0_7"),
        col("age_8_30"),
        col("age_31_90"),
        col("age_91"),
    ];
    let mut checked = 0;
    for row in &page.rows {
        let Cell::Party { party } = &row.cells[buyer] else {
            continue;
        };
        checked += 1;
        let key = (party.account_id, party.person_key.clone());
        let want = expected.get(&key);
        match want {
            Some((date, amounts)) => {
                assert_eq!(
                    row.cells[oldest],
                    Cell::Date { iso: date.clone() },
                    "{label}: oldest day of {key:?}"
                );
                assert_eq!(
                    row.cells[days_col],
                    Cell::Number {
                        value: days_between(date, &today).max(0)
                    },
                    "{label}: age of {key:?}"
                );
                for (cell, amount) in bands.iter().zip(amounts) {
                    let expect = if *amount == 0 {
                        Cell::Empty
                    } else {
                        Cell::Money {
                            paisa: mandi_services::core::money::Paisa::try_from_paisa(*amount)
                                .unwrap(),
                        }
                    };
                    assert_eq!(row.cells[*cell], expect, "{label}: a band of {key:?}");
                }
                // Reported, not gated: the bands add up to the balance when the matching is whole.
                let total: i64 = amounts.iter().sum();
                if let Cell::Money { paisa } = &row.cells[balance] {
                    assert_eq!(
                        paisa.as_paisa(),
                        total,
                        "{label}: bands add up to the balance of {key:?}"
                    );
                }
            }
            None => {
                assert_eq!(
                    row.cells[oldest],
                    Cell::Empty,
                    "{label}: no day for {key:?}"
                );
                for cell in &bands {
                    assert_eq!(
                        row.cells[*cell],
                        Cell::Empty,
                        "{label}: no band for {key:?}"
                    );
                }
            }
        }
    }
    // A comparison of nothing proves nothing: a book with an open bill has a buyer row to check.
    if !expected.is_empty() {
        assert!(checked > 0, "{label}: R20 had no buyer row");
    }
}

// ---- the books --------------------------------------------------------------------------------

#[test]
fn the_worked_day_events_1_and_2() {
    let w = worked_day();
    check_all(&w, "opening position only");
    save(&w, &support::event1(&w));
    check_all(&w, "event 1");
    save(&w, &event2(&w));
    check_all(&w, "events 1 and 2");
}

#[test]
fn the_worked_day_e2e_8_43_events_1_to_5() {
    let w = full_day();
    check_all(&w, "events 1 to 5");
    assert_rows_exist(&w, "events 1 to 5", 5, 2, 2);
    let compared = read(&w, |conn| Ok(forms_on(conn, "events 1 to 5", &DAY_RANGES)));
    assert!(
        compared >= 40,
        "only {compared} rows of the day queries were compared"
    );
    // PRD 4.9: the buyers before 11-10-26 owe 2,880 + 18,684 = 21,564 (the PRD's own figure).
    read(&w, |conn| {
        let rows = totals_before(conn, &BUYERS, "2026-10-11", PartyScope::default())?;
        let owing: i64 = rows
            .iter()
            .filter(|t| t.dr_paisa > t.cr_paisa)
            .map(|t| t.dr_paisa - t.cr_paisa)
            .sum();
        assert_eq!(owing, 21_564 * 100);
        // Before the worked day itself no buyer has a line.
        assert!(totals_before(conn, &BUYERS, "2026-10-10", PartyScope::default())?.is_empty());
        Ok(())
    });
    // R20's open bills: اکرم's 12,880 is matched for 10,000, so 2,880 is open; ارشد's 18,684 is open.
    read(&w, |conn| {
        let ids = buyer_ids(conn);
        let mut open: Vec<i64> = open_buyer_bills_by(conn, &ids, OpenMethod::Bulk)?
            .into_iter()
            .map(|b| b.open_paisa / 100)
            .collect();
        open.sort_unstable();
        assert_eq!(open, vec![2_880, 18_684]);
        Ok(())
    });
}

#[test]
fn e2e_13_and_14_an_edit_and_its_edit_back_leave_reversals() {
    let w = worked_day();
    let one = save(&w, &support::event1(&w));
    save(&w, &event2(&w));
    vouchers::save_receipt(&w.env.ctx, &w.owner, &receipt(w.akram, "10,000")).unwrap();
    for rate in ["610", "600"] {
        let opened = bikri::open_for_edit(
            &w.env.ctx,
            &mandi_services::core::bikri::EditOpenInput {
                bikri_id: one.bikri_id,
                entry_id: read(&w, |conn| {
                    conn.query_row(
                        "SELECT e.id FROM journal_entries e
                         WHERE e.source_type = 'bikri' AND e.source_id = ?1
                           AND e.posting_type = 'bikri_first' AND e.reverses_entry_id IS NULL
                           AND NOT EXISTS (SELECT 1 FROM journal_entries r
                                           WHERE r.reverses_entry_id = e.id AND r.status = 'posted')
                         ORDER BY e.id DESC LIMIT 1",
                        [one.bikri_id],
                        |r| r.get(0),
                    )
                }),
            },
        )
        .unwrap();
        let mut draft = opened.draft.clone();
        draft.lots[0].sales[0].seller_rate = rate.into();
        bikri::edit_save(
            &w.env.ctx,
            &w.owner,
            &mandi_services::core::bikri::EditSaveInput {
                entry_id: opened.entry_id,
                version: opened.version,
                draft,
                reason: Some("ریٹ کی درستی".into()),
                confirm_cash_below_zero: false,
            },
        )
        .unwrap();
        check_all(&w, &format!("after the edit to {rate}"));
    }
}

#[test]
fn e2e_15_a_cancelled_bikri_with_a_one_time_buyer() {
    let w = worked_day();
    save(&w, &support::event1(&w));
    let two = save(&w, &event2(&w));
    let version = bikri::get(&w.env.ctx, two.bikri_id).unwrap().version;
    bikri::cancel(
        &w.env.ctx,
        &w.owner,
        &mandi_services::core::bikri::CancelInput {
            id: two.bikri_id,
            version,
            reason: "غلط اندراج".into(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    check_all(&w, "e2e-15");
}

/// E2E-21 on credit: two one-time buyers under the shared account, one pays.
fn two_one_time_credit_buyers() -> World {
    let w = worked_day();
    let line = |key: &str, qty: &str, ur: &str, en: &str, phone: &str| {
        let mut l = support::sale(key, w.pukhta_buyer, qty, "1000");
        l.pukhta = Some(PukhtaInput {
            name_ur: ur.into(),
            name_en: en.into(),
            phone: phone.into(),
        });
        l.payment_type = Some(PaymentType::Credit);
        l.buyer_comm = support::typed(LineChargeMethod::Percent, "0");
        l.seller_comm = support::typed(LineChargeMethod::Percent, "0");
        l.laga = support::typed(LineChargeMethod::PerUnit, "0");
        l
    };
    let mut lot = support::bare_lot(&w, "lot-1", w.rahim, w.banana, "3");
    lot.sales = vec![
        line("s-1", "1", "شاہد", "Shahid", "0300-0000000"),
        line("s-2", "2", "ناصر", "Nasir", "0300-1111111"),
    ];
    save(&w, &support::draft(vec![lot]));
    w
}

#[test]
fn e2e_21_two_persons_on_the_shared_buyer_account() {
    let w = two_one_time_credit_buyers();
    check_all(&w, "two one-time buyers");
    // One of them pays; the person keys stay apart.
    let page = report(&w, ReportId::R20, ReportFilters::default(), 0, 1_000);
    let shahid = page
        .rows
        .iter()
        .find_map(|row| match &row.cells[0] {
            Cell::Party { party } if party.name_en == "Shahid" => party.person_key.clone(),
            _ => None,
        })
        .unwrap();
    let mut paid = receipt(w.pukhta_buyer, "1,000");
    paid.person_key = Some(shahid);
    vouchers::save_receipt(&w.env.ctx, &w.owner, &paid).unwrap();
    check_all(&w, "a one-time buyer paid");
    // The base shows the two persons apart, with the shared account's own keys.
    read(&w, |conn| {
        let rows = totals_before(conn, &BUYERS, "2030-01-01", PartyScope::default())?;
        let persons: Vec<&str> = rows
            .iter()
            .filter(|t| t.account_id == w.pukhta_buyer)
            .map(|t| t.person_key.as_str())
            .collect();
        assert_eq!(persons.len(), 2);
        assert!(persons.iter().all(|p| !p.is_empty()));
        Ok(())
    });
}

#[test]
fn e2e_28_the_advance_stays_open_and_e2e_31_a_seller_returns_advance_money() {
    let w = worked_day();
    let mut draft = support::event1(&w);
    draft.lots[0].adjust_advance = false;
    save(&w, &draft);
    check_all(&w, "e2e-28 tick off");
    vouchers::save_payment(&w.env.ctx, &w.owner, &payment(w.rahim, "19,316")).unwrap();
    check_all(&w, "e2e-28 paid");
    // A seller returns part of the advance in cash (a receipt on his account).
    vouchers::save_receipt(&w.env.ctx, &w.owner, &receipt(w.rahim, "5,000")).unwrap();
    check_all(&w, "e2e-31 advance returned");
}

#[test]
fn e2e_32_cash_lent_to_a_buyer_and_a_buyer_who_overpays() {
    let w = full_day();
    let mut lend = payment(w.akram, "3,000");
    lend.reason = "نقد ادھار".into();
    vouchers::save_payment(&w.env.ctx, &w.owner, &lend).unwrap();
    check_all(&w, "e2e-32");
    // E2E-11: a buyer who pays more than he owes has a credit balance; his bills do not leak.
    vouchers::save_receipt(&w.env.ctx, &w.owner, &receipt(w.arshad, "30,000")).unwrap();
    check_all(&w, "an overpaying buyer");
}

#[test]
fn e2e_34_a_trip_and_e2e_40_a_garden() {
    let (w, _, _, _) = trip_34_world();
    check_all(&w, "e2e-34");
    let (w, _garden) = garden_40();
    check_all(&w, "e2e-40");
}

#[test]
fn a_part_sold_lot_and_a_later_day() {
    let w = worked_day();
    let mut draft = support::event1(&w);
    draft.lots[0].sales[1].qty = "25".into();
    save(&w, &draft);
    set_today(&w.env, 2026, 10, 11);
    check_all(&w, "part sold, next day");
    vouchers::save_receipt(&w.env.ctx, &w.owner, &receipt(w.arshad, "5,000")).unwrap();
    check_all(&w, "a receipt the next day");
}

// ---- the hand-made cases of the money review -------------------------------------------------

/// Writes entries by plain SQL, as a sqlite shell or a restored backup could: a draft is made, its
/// lines are added, and it is posted by the one UPDATE (the balance trigger checks it).
fn raw_entry(
    w: &World,
    date: &str,
    posting_type: &str,
    reverses: Option<i64>,
    post: bool,
    lines: &[(i64, Option<&str>, i64, i64)],
) -> i64 {
    w.env
        .ctx
        .db()
        .write(|tx| -> Result<i64, mandi_services::ServiceError> {
            tx.execute(
                "INSERT INTO journal_entries (entry_date, posting_type, source_type, source_id,
                                              reverses_entry_id, status, total_paisa)
                 VALUES (?1, ?2, 'voucher', 0, ?3, 'draft', 0)",
                mandi_db::rusqlite::params![date, posting_type, reverses],
            )
            .map_err(mandi_db::DbError::from)?;
            let id = tx.last_insert_rowid();
            let mut total = 0_i64;
            for (n, (account, person, dr, cr)) in lines.iter().enumerate() {
                tx.execute(
                    "INSERT INTO journal_lines (entry_id, line_no, account_id, pukhta_person_key,
                                                dr_paisa, cr_paisa)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    mandi_db::rusqlite::params![id, n as i64 + 1, account, person, dr, cr],
                )
                .map_err(mandi_db::DbError::from)?;
                total += dr;
            }
            if post {
                tx.execute(
                    "UPDATE journal_entries SET status = 'posted', total_paisa = ?2 WHERE id = ?1",
                    mandi_db::rusqlite::params![id, total],
                )
                .map_err(mandi_db::DbError::from)?;
            }
            Ok(id)
        })
        .unwrap()
}

#[test]
fn a_draft_with_lines_and_a_draft_reversal_leave_the_originals_live() {
    let w = full_day();
    let cash = support::id_of(&w.env, "01010001");
    // A committed draft with lines on a buyer, a seller and a one-time person: left out everywhere.
    raw_entry(
        &w,
        "2026-10-10",
        "general",
        None,
        false,
        &[
            (w.akram, None, 7_700, 0),
            (w.rahim, None, 0, 7_700),
            (w.pukhta_buyer, Some("draft-person"), 5_500, 0),
            (cash, None, 0, 5_500),
        ],
    );
    check_all(&w, "a draft with lines");
    // A draft REVERSAL of a posted bill: the bill stays live (only a posted reversal reverses).
    let original: i64 = read(&w, |conn| {
        conn.query_row(
            "SELECT e.id FROM journal_entries e JOIN journal_lines l ON l.entry_id = e.id
             WHERE l.account_id = ?1 AND l.dr_paisa > 0 AND e.status = 'posted' LIMIT 1",
            [w.arshad],
            |r| r.get(0),
        )
    });
    raw_entry(
        &w,
        "2026-10-10",
        "reversal",
        Some(original),
        false,
        &[(w.arshad, None, 0, 100)],
    );
    check_all(&w, "a draft reversal");
    let open_after = read(&w, |conn| {
        let ids = buyer_ids(conn);
        open_buyer_bills_by(conn, &ids, OpenMethod::Bulk)
    });
    assert!(
        open_after
            .iter()
            .any(|b| b.account_id == w.arshad && b.open_paisa == 1_868_400),
        "ارشد's 18,684 is still open"
    );
}

#[test]
fn a_keyed_line_on_an_ordinary_account_makes_the_fast_reads_fall_back() {
    let w = full_day();
    let cash = support::id_of(&w.env, "01010001");
    // Nothing in the schema forbids a person key on an ordinary account; no writer makes one.
    assert!(read(&w, |conn| party_reads::person_keys_only_on_shared(
        conn, &BOTH
    )));
    raw_entry(
        &w,
        "2026-10-10",
        "general",
        None,
        true,
        &[(w.akram, Some("stray"), 3_300, 0), (cash, None, 0, 3_300)],
    );
    raw_entry(
        &w,
        "2026-10-10",
        "general",
        None,
        true,
        &[(cash, None, 4_400, 0), (w.rahim, Some("stray2"), 0, 4_400)],
    );
    assert!(!read(&w, |conn| party_reads::person_keys_only_on_shared(
        conn, &BOTH
    )));
    read(&w, |conn| {
        for from in FROMS {
            // The picked method is the line-by-line one, so the stray key keeps its own group.
            assert_eq!(
                party_reads::before_method(conn, &BOTH, from, PartyScope::default())?,
                BeforeMethod::PerLine
            );
            assert_eq!(
                new_base(conn, None, &BOTH, from, PartyScope::default()),
                old_base(conn, from, None, None),
                "the stray key, before {from}"
            );
        }
        let ids = buyer_ids(conn);
        // Bulk degrades to the line-by-line read, so the stray key is kept.
        assert_eq!(
            open_rows(open_buyer_bills_by(conn, &ids, OpenMethod::Bulk)?),
            old_open(conn, &ids)
        );
        Ok(())
    });
    check_a2(&w, "the stray key");
    check_a4(&w, "the stray key");
}

#[test]
fn a_line_without_a_key_on_a_shared_account_is_its_own_group() {
    let w = full_day();
    let cash = support::id_of(&w.env, "01010001");
    raw_entry(
        &w,
        "2026-10-10",
        "general",
        None,
        true,
        &[(w.pukhta_buyer, None, 2_200, 0), (cash, None, 0, 2_200)],
    );
    raw_entry(
        &w,
        "2026-10-10",
        "general",
        None,
        true,
        &[
            (w.pukhta_buyer, Some("Z"), 1_100, 0),
            (cash, None, 0, 1_100),
        ],
    );
    check_a1(&w, "an unkeyed line on the shared buyer");
    check_a3(&w, "an unkeyed line on the shared buyer");
}

#[test]
fn an_opening_balance_changed_leaves_two_openings_and_a_reversal() {
    // The worked day starts with openings on cash and capital; a buyer and a seller get openings
    // too, then the buyer's is changed (the old one is reversed, a new one posted).
    let w = worked_day();
    let account_version = |id: i64| {
        mandi_services::accounts::get(&w.env.ctx, id)
            .unwrap()
            .version
    };
    let open = |id: i64, dr: &str, cr: &str, adv: bool| {
        mandi_services::accounts::openings_save(
            &w.env.ctx,
            &w.owner,
            &mandi_services::core::accounts::OpeningsInput {
                lines: vec![mandi_services::core::accounts::OpeningLineInput {
                    account_id: id,
                    version: account_version(id),
                    opening_date: "10-10-2026".into(),
                    opening_dr: dr.into(),
                    opening_cr: cr.into(),
                    opening_is_advance: adv,
                }],
            },
        )
    };
    open(w.akram, "4,000", "", false).unwrap();
    open(w.karim, "", "6,000", false).unwrap();
    // A seller's opening debit that counts as an advance (PRD FR-15.8).
    open(w.rahim, "3,000", "", true).unwrap();
    check_all(&w, "openings");
    open(w.akram, "5,000", "", false).unwrap();
    open(w.karim, "", "7,000", false).unwrap();
    check_all(&w, "openings changed");
    save(&w, &support::event1(&w));
    vouchers::save_receipt(&w.env.ctx, &w.owner, &receipt(w.akram, "1,000")).unwrap();
    check_all(&w, "openings and a sale");
    // A seller opening credit is held at any cutoff; a buyer opening is a bill with a day.
    read(&w, |conn| {
        let held = held_credits(conn, "03011", "2030-01-01")?;
        assert!(
            held.iter()
                .any(|h| h.is_opening && h.account_id == w.karim && h.cr_paisa == 700_000)
        );
        Ok(())
    });
}

// ---- a random book made through the services -----------------------------------------------------

/// Small deterministic random numbers (splitmix64).
struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n.max(1)
    }
}

const BUYER_NAMES: [(&str, &str); 8] = [
    ("بشیر", "Bashir"),
    ("حنیف", "Hanif"),
    ("رفیق", "Rafiq"),
    ("شفیق", "Shafiq"),
    ("منیر", "Munir"),
    ("نذیر", "Nazir"),
    ("اعجاز", "Ijaz"),
    ("ظفر", "Zafar"),
];

#[test]
fn a_random_book_over_several_days() {
    for seed in [7_u64, 1_971] {
        let w = worked_day();
        let mut rng = Rng(seed);
        let mut buyers = vec![w.akram, w.arshad];
        for (ur, en) in BUYER_NAMES {
            buyers.push(support::add_account(
                &w.env,
                &w.owner,
                AccountType::Buyer,
                ur,
                en,
                true,
                "",
            ));
        }
        let mut saved = Vec::new();
        for day in 0..6_u32 {
            set_today(&w.env, 2026, 10, 10 + day);
            for n in 0..4_u64 {
                let seller = if rng.below(2) == 0 { w.rahim } else { w.karim };
                let item = if rng.below(2) == 0 { w.banana } else { w.apple };
                let lines = 1 + rng.below(3);
                let mut lot = support::bare_lot(
                    &w,
                    &format!("lot-{day}-{n}"),
                    seller,
                    item,
                    &format!("{}", lines * 10),
                );
                lot.adjust_advance = rng.below(3) != 0;
                lot.sales = (0..lines)
                    .map(|i| {
                        let buyer = if rng.below(6) == 0 {
                            w.pukhta_buyer
                        } else {
                            buyers[rng.below(buyers.len() as u64) as usize]
                        };
                        let mut line = support::sale(
                            &format!("s-{day}-{n}-{i}"),
                            buyer,
                            "10",
                            &format!("{}", 400 + rng.below(300)),
                        );
                        if buyer == w.pukhta_buyer {
                            let (ur, en, phone) = if rng.below(2) == 0 {
                                ("شاہد", "Shahid", "0300-0000000")
                            } else {
                                ("ناصر", "Nasir", "0300-1111111")
                            };
                            line.pukhta = Some(PukhtaInput {
                                name_ur: ur.into(),
                                name_en: en.into(),
                                phone: phone.into(),
                            });
                            line.payment_type = Some(PaymentType::Credit);
                        }
                        line
                    })
                    .collect();
                saved.push(save(&w, &support::draft(vec![lot])));
            }
            // Receipts and payments.
            for _ in 0..3 {
                let buyer = buyers[rng.below(buyers.len() as u64) as usize];
                let amount = format!("{}", 500 + rng.below(4_000));
                let _ = vouchers::save_receipt(&w.env.ctx, &w.owner, &receipt(buyer, &amount));
            }
            let seller = if rng.below(2) == 0 { w.rahim } else { w.karim };
            let _ = vouchers::save_payment(&w.env.ctx, &w.owner, &payment(seller, "1,000"));
            // A cancel now and then (a reversal entry on the same date as the original).
            if day % 2 == 1
                && let Some(s) = saved.get(rng.below(saved.len() as u64) as usize)
                && let Ok(detail) = bikri::get(&w.env.ctx, s.bikri_id)
            {
                let _ = bikri::cancel(
                    &w.env.ctx,
                    &w.owner,
                    &mandi_services::core::bikri::CancelInput {
                        id: s.bikri_id,
                        version: detail.version,
                        reason: "غلط اندراج".into(),
                        confirm_cash_below_zero: true,
                    },
                );
            }
            check_all(&w, &format!("random seed {seed} day {day}"));
            if day == 5 {
                assert_rows_exist(&w, "the random book", 8, 8, 4);
            }
        }
    }
}

// ---- many buyers: more than one chunk of accounts --------------------------------------------------

#[test]
fn more_buyers_than_one_chunk_of_accounts() {
    let w = worked_day();
    let mut buyers = Vec::new();
    for i in 0..520_u32 {
        // A different Urdu letter pair and English name for each.
        let ur = format!("خریدار {}", urdu_digits(i));
        let en = format!("Buyer {i}");
        buyers.push(support::add_account(
            &w.env,
            &w.owner,
            AccountType::Buyer,
            &ur,
            &en,
            true,
            "",
        ));
    }
    for (n, chunk) in buyers.chunks(260).enumerate() {
        let mut lot = support::bare_lot(&w, &format!("lot-{n}"), w.rahim, w.banana, "260");
        lot.sales = chunk
            .iter()
            .enumerate()
            .map(|(i, buyer)| support::sale(&format!("s-{n}-{i}"), *buyer, "1", "1000"))
            .collect();
        save(&w, &support::draft(vec![lot]));
    }
    for (i, buyer) in buyers.iter().enumerate().step_by(3) {
        let _ = vouchers::save_receipt(
            &w.env.ctx,
            &w.owner,
            &receipt(*buyer, &format!("{}", 500 + (i % 7) * 100)),
        );
    }
    check_a1(&w, "520 buyers");
    check_a3(&w, "520 buyers");
    check_a4(&w, "520 buyers");
    // The picked method reads 520 buyers (more than the chunk of 500) the same as the line-by-line
    // read, with the bulk read forced.
    read(&w, |conn| {
        let ids = buyer_ids(conn);
        assert!(ids.len() > 500);
        assert_eq!(
            open_rows(open_buyer_bills_by(conn, &ids, OpenMethod::Bulk)?),
            open_rows(open_bills_per_line(conn, &ids)?)
        );
        Ok(())
    });
}

/// A number written in Arabic-Indic letters would not be an Urdu letter; words keep the Urdu-name
/// rule (at least one Urdu letter, digits allowed in `0-9`).
fn urdu_digits(i: u32) -> String {
    i.to_string()
}

// ---- the NFR-SCL-1 volume ------------------------------------------------------------------------

/// The same comparisons on robustness's file of 20,000 accounts and 3.5 million lines (copy it,
/// run the app's migrations on the copy, drop its statistics as the app's own file has none), and
/// give the path in `MANDI_SPEED_EQUAL_DB`. The old SQL takes about 10 seconds a day here.
///
/// ```text
/// MANDI_SPEED_EQUAL_DB=/path/mandi.db cargo test -p mandi-services --test speed_rewrites_equal \
///     --release -- --ignored --nocapture
/// ```
#[test]
#[ignore = "needs the NFR-SCL-1 file in MANDI_SPEED_EQUAL_DB"]
fn on_the_nfr_scl1_volume() {
    let path = std::env::var("MANDI_SPEED_EQUAL_DB").unwrap();
    let conn =
        Connection::open_with_flags(&path, mandi_db::rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
            .unwrap();
    a1_on(
        &conn,
        "the volume",
        &["2016-10-12", "2021-10-12", "2026-10-10", "2026-10-11"],
        false,
    )
    .unwrap();
    a2_on(&conn, "the volume").unwrap();
    a3_on(
        &conn,
        "the volume",
        &[OpenMethod::Auto, OpenMethod::Bulk],
        false,
    )
    .unwrap();
    a4_on(&conn, "the volume").unwrap();
    forms_on(
        &conn,
        "the volume",
        &[("2026-10-09", "2026-10-09"), ("2026-09-10", "2026-10-10")],
    );
}

// ---- a copy of the Owner's own file ----------------------------------------------------------------

/// Migration 16 on a COPY of the Owner's data (never the file itself): the copy is opened by the
/// app's own start-up (which migrates it from 14 to 16 after taking a safety copy), no table loses or
/// gains a row, the file passes the data check, the two indexes are there, and the reads equal the
/// old SQL on his real books. Give the folder that holds `data/mandi.db` (and its `-wal` and `-shm`
/// files, copied together) in `MANDI_REAL_COPY`.
#[test]
#[ignore = "needs a copy of the Owner's data folder in MANDI_REAL_COPY"]
fn migration_16_and_the_reads_on_a_copy_of_the_owners_data() {
    let dir = std::path::PathBuf::from(std::env::var("MANDI_REAL_COPY").unwrap());
    let counts = |conn: &Connection| -> Vec<(String, i64)> {
        let mut statement = conn
            .prepare(
                "SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%'
                 ORDER BY name",
            )
            .unwrap();
        let names: Vec<String> = statement
            .query_map([], |r| r.get(0))
            .unwrap()
            .map(Result::unwrap)
            .collect();
        names
            .into_iter()
            .map(|name| {
                let n: i64 = conn
                    .query_row(&format!("SELECT COUNT(*) FROM \"{name}\""), [], |r| {
                        r.get(0)
                    })
                    .unwrap();
                (name, n)
            })
            .collect()
    };
    let before = {
        let conn = Connection::open(dir.join("data").join("mandi.db")).unwrap();
        let version: i64 = conn
            .query_row("PRAGMA user_version", [], |r| r.get(0))
            .unwrap();
        eprintln!("the copy is at version {version}");
        counts(&conn)
    };
    let env = mandi_services::testing::TestEnv::new(&dir).unwrap();
    let db = env.ctx.db();
    db.read(|conn| -> mandi_db::rusqlite::Result<()> {
        let version: i64 = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;
        assert_eq!(version, 16);
        let integrity: String = conn.query_row("PRAGMA integrity_check", [], |r| r.get(0))?;
        assert_eq!(integrity, "ok");
        // Migration 15 (not 16) added the empty table of the question log.
        let after = counts(conn);
        assert!(after.contains(&("assistant_questions".to_owned(), 0)));
        let after: Vec<_> = after
            .into_iter()
            .filter(|(name, _)| name != "assistant_questions")
            .collect();
        assert_eq!(after, before, "no table gained or lost a row");
        for name in [
            "idx_journal_entries_opening",
            "idx_bill_allocations_open_sum",
        ] {
            let found: i64 = conn.query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'index' AND name = ?1",
                [name],
                |r| r.get(0),
            )?;
            assert_eq!(found, 1, "{name}");
        }
        a1_on(conn, "the Owner's copy", &FROMS, true)?;
        a2_on(conn, "the Owner's copy")?;
        a3_on(
            conn,
            "the Owner's copy",
            &[OpenMethod::Auto, OpenMethod::PerLine, OpenMethod::Bulk],
            true,
        )?;
        a4_on(conn, "the Owner's copy")?;
        Ok(())
    })
    .unwrap();
    // The screens that read these (dashboard, R20, R24, R34) still build.
    mandi_reports::dashboard::get(
        db,
        mandi_reports::dashboard::BackupFacts {
            cloud_on: false,
            cloud_age_hours: None,
        },
    )
    .unwrap();
    for id in [ReportId::R20, ReportId::R24, ReportId::R34, ReportId::R35] {
        mandi_reports::reports::run(
            db,
            &mandi_reports::reports::ReportRunInput {
                id,
                filters: ReportFilters::default(),
                offset: 0,
                limit: 100,
                sort: None,
                group_by_month: false,
            },
        )
        .unwrap();
    }
}
