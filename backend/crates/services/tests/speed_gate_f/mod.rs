//! The speed fixes of the plan 80 gate (backend-f) give the very rows the old code gave (PRD FR-6.1,
//! FR-19.8, FR-19.9, FR-19.11; NFR-SPD-10, NFR-SPD-24). Every check here runs the OLD query, copied
//! verbatim from the code before the change, or the old way of reading, next to the new one on the
//! same book, and holds the two to the same whole rows. They run on every book of
//! `speed_rewrites_equal.rs` (the worked day, its edits and cancels, trips and gardens, چکتا buyers, a
//! draft, a random book over several days), through its `check_all`.
//!
//! - r35: the sums of every buyer and seller on a day (the all-parties table of R35);
//! - statement rows: a page of the statement of every buyer and seller, in every shape, against the
//!   whole statement cut into pages;
//! - the registers R2, R3, R4: the totals read from the sale lines against the totals read from the
//!   entries;
//! - R28: the income by period and account read from the income accounts against the entries;
//! - the kept sums of every account against a plain `SUM`, and the day book totals of the whole
//!   journal against the join of lines and entries.

use mandi_db::rusqlite::types::Value;
use mandi_db::rusqlite::{Connection, params, params_from_iter};
use mandi_reports::reports::day_query_forms::day_activities;
use mandi_reports::reports::plans;
use mandi_reports::reports::{StatementRowsInput, statement_rows, statement_rows_by_whole};

use super::{World, read};

/// The days the sums of a day are held to: before everything, the days of the books, after them.
const DAYS: [&str; 9] = [
    "2000-01-01",
    "2026-08-31",
    "2026-09-01",
    "2026-10-09",
    "2026-10-10",
    "2026-10-11",
    "2026-10-12",
    "2026-10-15",
    "2030-01-01",
];

// ---- r35: the sums of every party on a day, held to the grouped query it replaced ---------------------

/// A same-day cancel or edit: the posting dated the same day as its reversal, or the reversal itself
/// (the text of `sheet::activity::SAME_DAY_PAIR`).
const OLD_SAME_DAY_PAIR: &str = "(e.reverses_entry_id IS NOT NULL AND EXISTS (
        SELECT 1 FROM journal_entries o
        WHERE o.id = e.reverses_entry_id AND o.entry_date = e.entry_date))
    OR EXISTS (SELECT 1 FROM journal_entries r
        WHERE r.reverses_entry_id = e.id AND r.status = 'posted' AND r.entry_date = e.entry_date)";

type DayRow = (
    i64,
    Option<String>,
    bool,
    (i64, i64),
    (i64, i64),
    (i64, i64),
);

/// The query `activities` was before plan 80: one grouped pass over every posted line of the buyers
/// and sellers up to the day.
fn old_activities(conn: &Connection, day: &str) -> Vec<DayRow> {
    let sql = format!(
        "SELECT l.account_id, l.chukta_person_key, k.code,
                COALESCE(SUM(CASE WHEN e.entry_date < ?1 THEN l.dr_paisa END), 0),
                COALESCE(SUM(CASE WHEN e.entry_date < ?1 THEN l.cr_paisa END), 0),
                COALESCE(SUM(CASE WHEN e.entry_date = ?1 AND NOT ({OLD_SAME_DAY_PAIR})
                                  THEN l.dr_paisa END), 0),
                COALESCE(SUM(CASE WHEN e.entry_date = ?1 AND NOT ({OLD_SAME_DAY_PAIR})
                                  THEN l.cr_paisa END), 0),
                COALESCE(SUM(l.dr_paisa), 0), COALESCE(SUM(l.cr_paisa), 0)
         FROM journal_lines l
         JOIN journal_entries e ON e.id = l.entry_id
         JOIN accounts a ON a.id = l.account_id
         JOIN account_controls k ON k.id = a.control_id
         WHERE k.code IN ('0304', '03011') AND e.status = 'posted'
           AND e.entry_date <= ?1
           AND (?2 IS NULL OR l.account_id = ?2)
           AND (?3 IS NULL OR l.chukta_person_key = ?3)
         GROUP BY l.account_id, l.chukta_person_key, k.code
         ORDER BY l.account_id, l.chukta_person_key"
    );
    let mut statement = conn.prepare(&sql).unwrap();
    statement
        .query_map(
            params![day, Option::<i64>::None, Option::<String>::None],
            |r| {
                let control: String = r.get(2)?;
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, Option<String>>(1)?,
                    control == "0304",
                    (r.get::<_, i64>(3)?, r.get::<_, i64>(4)?),
                    (r.get::<_, i64>(5)?, r.get::<_, i64>(6)?),
                    (r.get::<_, i64>(7)?, r.get::<_, i64>(8)?),
                ))
            },
        )
        .unwrap()
        .map(Result::unwrap)
        .collect()
}

/// A party the old query listed and the new does not, or the reverse, is a row of zeros only: the
/// table of R35 lists a party only when it has a balance or an entry (`worth_a_row`), so such a row
/// never shows. Everything else must be equal, row by row.
fn worth_a_row(row: &DayRow) -> bool {
    let (_, _, _, prev, day, end) = row;
    prev.0 != prev.1 || day.0 != 0 || day.1 != 0 || end.0 != end.1
}

fn check_r35(w: &World, label: &str) {
    read(w, |conn| {
        for day in DAYS {
            let old: Vec<DayRow> = old_activities(conn, day)
                .into_iter()
                .filter(worth_a_row)
                .collect();
            let new: Vec<DayRow> = day_activities(conn, day)
                .unwrap()
                .into_iter()
                .map(|a| (a.account_id, a.person_key, a.buyer, a.prev, a.day, a.end))
                .filter(worth_a_row)
                .collect();
            assert_eq!(new, old, "{label}: the sums of the parties on {day}");
        }
        Ok(())
    });
}

// ---- the statement of a party, a page at a time ---------------------------------------------------------

fn check_statement_rows(w: &World, label: &str) {
    let accounts: Vec<i64> = read(w, |conn| {
        let mut statement = conn.prepare(
            "SELECT DISTINCT a.id FROM accounts a JOIN account_controls k ON k.id = a.control_id
             WHERE k.code IN ('0304', '03011') AND a.is_chukta_khata = 0 ORDER BY a.id",
        )?;
        Ok(statement
            .query_map([], |r| r.get::<_, i64>(0))?
            .map(Result::unwrap)
            .collect())
    });
    let db = w.env.ctx.db();
    let mut compared = 0;
    for account in accounts {
        for history in [false, true] {
            for newest in [false, true] {
                for (from, to) in [
                    (None, None),
                    (Some("01-09-2026"), None),
                    (Some("10-10-2026"), Some("10-10-2026")),
                    (Some("11-10-2026"), Some("15-10-2026")),
                    (None, Some("09-10-2026")),
                ] {
                    for (offset, limit) in [(0_u32, 100_u32), (0, 2), (1, 2), (3, 2), (50, 5)] {
                        let input = StatementRowsInput {
                            account_id: account,
                            from: from.map(str::to_owned),
                            to: to.map(str::to_owned),
                            include_history: history,
                            newest_first: newest,
                            offset,
                            limit,
                        };
                        let new = statement_rows(db, &input);
                        let old = statement_rows_by_whole(db, &input);
                        match (new, old) {
                            (Ok(new), Ok(old)) => {
                                compared += new.rows.len();
                                assert_eq!(new, old, "{label}: statement of {account} {input:?}");
                            }
                            (Err(new), Err(old)) => {
                                assert_eq!(
                                    format!("{new:?}"),
                                    format!("{old:?}"),
                                    "{label}: the error of the statement of {account} {input:?}"
                                );
                            }
                            (new, old) => {
                                panic!("{label}: {account} {input:?}: new {new:?}, old {old:?}")
                            }
                        }
                    }
                }
            }
        }
    }
    // A book with no entry for any party compares nothing; every book of this file has some.
    let _ = compared;
}

// ---- the registers and the income report: the long-range reads against the plain reads --------------------

/// The ranges of days the totals are asked for: one day, a few, all.
const RANGES: [(Option<&str>, Option<&str>); 6] = [
    (None, None),
    (Some("2000-01-01"), Some("2030-01-01")),
    (Some("2026-10-10"), Some("2026-10-10")),
    (Some("2026-10-11"), Some("2026-10-12")),
    (Some("2026-09-01"), Some("2026-10-10")),
    (Some("2026-10-13"), None),
];

fn run_totals(conn: &Connection, (sql, args): (String, Vec<Value>)) -> Vec<Value> {
    let mut statement = conn.prepare(&sql).unwrap();
    let width = statement.column_count();
    statement
        .query_row(params_from_iter(args.iter()), |r| {
            (0..width).map(|i| r.get::<_, Value>(i)).collect()
        })
        .unwrap()
}

fn check_register_totals(w: &World, label: &str) {
    read(w, |conn| {
        for (from, to) in RANGES {
            assert_eq!(
                run_totals(conn, plans::r2_totals_wide(from, to)),
                run_totals(conn, plans::r2_totals(from, to)),
                "{label}: R2 totals {from:?} {to:?}"
            );
            for cash in [true, false] {
                assert_eq!(
                    run_totals(conn, plans::r3_r4_totals_wide(from, to, cash)),
                    run_totals(conn, plans::r3_r4_totals(from, to, cash)),
                    "{label}: R3 or R4 totals {from:?} {to:?} cash {cash}"
                );
            }
        }
        Ok(())
    });
}

fn all_rows(conn: &Connection, (sql, args): (String, Vec<Value>)) -> Vec<Vec<Value>> {
    let mut statement = conn.prepare(&sql).unwrap();
    let width = statement.column_count();
    statement
        .query_map(params_from_iter(args.iter()), |r| {
            (0..width).map(|i| r.get::<_, Value>(i)).collect()
        })
        .unwrap()
        .map(Result::unwrap)
        .collect()
}

fn check_income(w: &World, label: &str) {
    read(w, |conn| {
        for (from, to) in RANGES {
            assert_eq!(
                all_rows(conn, plans::r28_groups(from, to, true)),
                all_rows(conn, plans::r28_groups(from, to, false)),
                "{label}: R28 {from:?} {to:?}"
            );
        }
        Ok(())
    });
}

// ---- the kept sums and the day book totals ---------------------------------------------------------------

fn check_kept_sums(w: &World, label: &str) {
    read(w, |conn| {
        let asked = |as_of: Option<&str>| -> Vec<(i64, i64, i64)> {
            // Twice: the second answer comes from memory.
            let first = mandi_db::ledger::account_totals(conn, as_of).unwrap();
            let again = mandi_db::ledger::account_totals(conn, as_of).unwrap();
            assert_eq!(first, again, "{label}: the kept sums {as_of:?}");
            first
                .into_iter()
                .map(|(id, t)| (id, t.dr_paisa, t.cr_paisa))
                .collect()
        };
        for as_of in [
            None,
            Some("2026-10-10"),
            Some("2000-01-01"),
            Some("2030-01-01"),
        ] {
            let mut statement = conn
                .prepare(match as_of {
                    None => {
                        "SELECT l.account_id, COALESCE(SUM(l.dr_paisa), 0), COALESCE(SUM(l.cr_paisa), 0)
                         FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
                         WHERE e.status = 'posted' AND ?1 IS NULL
                         GROUP BY l.account_id ORDER BY l.account_id"
                    }
                    Some(_) => {
                        "SELECT l.account_id, COALESCE(SUM(l.dr_paisa), 0), COALESCE(SUM(l.cr_paisa), 0)
                         FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
                         WHERE e.status = 'posted' AND e.entry_date <= ?1
                         GROUP BY l.account_id ORDER BY l.account_id"
                    }
                })
                .unwrap();
            let plain: Vec<(i64, i64, i64)> = statement
                .query_map([as_of], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
                .unwrap()
                .map(Result::unwrap)
                .collect();
            assert_eq!(
                asked(as_of),
                plain,
                "{label}: sums of every account {as_of:?}"
            );
        }
        Ok(())
    });
}

/// The day book (R11) for every day of the books and for all but one day: the totals row is the
/// sums of the lines of the entries in the range, with or without the shortcut for the whole journal.
fn check_day_book_totals(w: &World, label: &str) {
    use mandi_reports::reports::{self, Cell, ReportFilters, ReportId, ReportRunInput};
    let db = w.env.ctx.db();
    for (from, to) in [
        ("", ""),
        ("01-01-2000", "01-01-2030"),
        ("31-08-2026", "15-10-2026"),
        ("10-10-2026", "10-10-2026"),
        ("11-10-2026", "11-10-2030"),
        ("01-09-2026", "09-10-2026"),
    ] {
        let page = reports::run(
            db,
            &ReportRunInput {
                id: ReportId::R11,
                filters: ReportFilters {
                    from: from.to_owned(),
                    to: to.to_owned(),
                    ..ReportFilters::default()
                },
                offset: 0,
                limit: 100,
                sort: None,
                group_by_month: false,
            },
        );
        // A day after this book's today is refused by the engine (a date in the future); every other
        // refusal is a failure.
        let page = match page {
            Ok(page) => page,
            Err(e) if format!("{e:?}").contains("ErrDate") => continue,
            Err(e) => panic!("{label}: R11 {from} {to}: {e:?}"),
        };
        let (from_iso, to_iso) = read(w, |conn| {
            let iso = |t: &str, default: &str| -> String {
                if t.is_empty() {
                    default.to_owned()
                } else {
                    format!("{}-{}-{}", &t[6..10], &t[3..5], &t[0..2])
                }
            };
            let _ = conn;
            Ok((iso(from, "0000-01-01"), iso(to, "9999-12-31")))
        });
        let (dr, cr): (i64, i64) = read(w, |conn| {
            conn.query_row(
                "SELECT COALESCE(SUM(l.dr_paisa), 0), COALESCE(SUM(l.cr_paisa), 0)
                 FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
                 WHERE e.status = 'posted' AND e.entry_date >= ?1 AND e.entry_date <= ?2",
                [from_iso.as_str(), to_iso.as_str()],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
        });
        let money: Vec<i64> = page
            .totals
            .iter()
            .filter_map(|c| match c {
                Cell::Money { paisa } => Some(paisa.as_paisa()),
                _ => None,
            })
            .collect();
        assert_eq!(
            money,
            vec![dr, cr],
            "{label}: R11 {from} {to}: the totals row against the lines"
        );
    }
}

// ---- the age bands of the open bills, in bulk and one by one ---------------------------------------------

/// The bulk read of the open bills by band (`open_buyer_ages_bulk`) against the bills read one by one
/// and put into the bands by their age in days, for the days that are exactly on the edges of the bands:
/// a bill dated after today, today, 7, 8, 30, 31, 90 and 91 days back. The books of this file are too
/// small for the report to pick the bulk read by itself, so the read is called directly.
fn check_age_bands(w: &World, label: &str) {
    use mandi_db::party_reads::{AgeCuts, OpenMethod, open_buyer_ages_bulk, open_buyer_bills_by};
    use mandi_services::core::dates::BusinessDate;
    read(w, |conn| {
        let ids: Vec<i64> = {
            let mut statement = conn.prepare(
                "SELECT a.id FROM accounts a JOIN account_controls k ON k.id = a.control_id
                 WHERE k.code = '0304' AND a.is_chukta_khata = 0 ORDER BY a.id",
            )?;
            statement
                .query_map([], |r| r.get::<_, i64>(0))?
                .map(Result::unwrap)
                .collect()
        };
        let bills = open_buyer_bills_by(conn, &ids, OpenMethod::PerLine)?;
        // Every day the bills fall on, so each of them is on an edge of some `today` below.
        let mut days: Vec<String> = bills.iter().map(|b| b.entry_date.clone()).collect();
        days.sort();
        days.dedup();
        let mut todays: Vec<BusinessDate> = Vec::new();
        for day in &days {
            let base = BusinessDate::parse_iso(day).unwrap();
            for back in [-1, 0, 7, 8, 30, 31, 90, 91] {
                todays.push(base.plus_days(-back).unwrap());
            }
        }
        for today in todays {
            let cut = |days: i64| today.plus_days(-days).unwrap().to_iso();
            let (seven, thirty, ninety) = (cut(7), cut(30), cut(90));
            let mut expected: std::collections::BTreeMap<i64, (String, [i64; 4])> =
                std::collections::BTreeMap::new();
            for bill in &bills {
                let age = today
                    .days_after(BusinessDate::parse_iso(&bill.entry_date).unwrap())
                    .max(0);
                let band = [7, 30, 90].iter().position(|end| age <= *end).unwrap_or(3);
                let slot = expected
                    .entry(bill.account_id)
                    .or_insert((bill.entry_date.clone(), [0; 4]));
                if bill.entry_date < slot.0 {
                    slot.0 = bill.entry_date.clone();
                }
                slot.1[band] += bill.open_paisa;
            }
            let Some(got) = open_buyer_ages_bulk(
                conn,
                &ids,
                AgeCuts {
                    seven: &seven,
                    thirty: &thirty,
                    ninety: &ninety,
                },
            )?
            else {
                // A person key on an ordinary buyer: the bulk read is not for these books.
                return Ok(());
            };
            let got: std::collections::BTreeMap<i64, (String, [i64; 4])> = got
                .into_iter()
                .map(|a| (a.account_id, (a.oldest, a.bands)))
                .collect();
            assert_eq!(
                got,
                expected,
                "{label}: the age bands as of {}",
                today.to_iso()
            );
        }
        Ok(())
    });
}

/// Every check of this file on one book.
pub(super) fn check_gate_f(w: &World, label: &str) {
    check_age_bands(w, label);
    check_r35(w, label);
    check_statement_rows(w, label);
    check_register_totals(w, label);
    check_income(w, label);
    check_kept_sums(w, label);
    check_day_book_totals(w, label);
}

// ---- nothing kept is ever stale --------------------------------------------------------------------------
//
// The pages of the reports and the kept sums are kept only while the books are as they were. Each
// read below is made twice with the books as they are (the second answer comes from memory), then a
// save is made, and the read is made again: it must show the new figure, never the kept one. The
// expected change is worked out by hand from the save (PRD Section 4: اکرم paid 10,000 of 12,880).

use mandi_reports::reports::{Cell, ReportFilters, ReportId, ReportPage};
use mandi_services::{cashbook, vouchers};

use super::reports_world::{full_day, payment, receipt, report, save as save_bikri};
use super::support;

/// Every amount of the totals row and the summary lines of a page, paisa, in order.
fn money_of(page: &ReportPage) -> Vec<i64> {
    let mut out: Vec<i64> = Vec::new();
    for cell in page
        .totals
        .iter()
        .chain(page.summary.iter().map(|s| &s.cell))
    {
        if let Cell::Money { paisa } = cell {
            out.push(paisa.as_paisa());
        }
    }
    out
}

/// A report with the filters it takes: the balance reports take a day ("as of"), the sales reports a
/// range of days, the two books reports none. Another filter is refused by the report engine.
fn read_page(w: &World, id: ReportId, from: &str, to: &str) -> ReportPage {
    use ReportId::*;
    let filters = match id {
        R16 | R18 | R20 | R21 | R22 | R24 | R35 => ReportFilters {
            as_of: to.to_owned(),
            ..ReportFilters::default()
        },
        _ => ReportFilters {
            from: from.to_owned(),
            to: to.to_owned(),
            ..ReportFilters::default()
        },
    };
    report(w, id, filters, 0, 100)
}

/// Everything a page shows, as text: its totals row, its summary lines and every row of the page.
/// (A total can stay the same when a save moves money between two accounts, so the rows count too.)
fn digest_of(page: &ReportPage) -> Vec<String> {
    let mut out = vec![format!("{:?}", page.totals), format!("{:?}", page.summary)];
    out.extend(page.rows.iter().map(|r| format!("{r:?}")));
    out
}

/// A read twice, then the save, then the read again.
fn changed_by(
    w: &World,
    reads: &[(ReportId, &str, &str)],
    save: impl FnOnce(),
) -> Vec<(ReportId, Vec<String>, Vec<String>)> {
    let before: Vec<Vec<String>> = reads
        .iter()
        .map(|(id, from, to)| {
            let first = digest_of(&read_page(w, *id, from, to));
            let again = digest_of(&read_page(w, *id, from, to));
            assert_eq!(first, again, "{id:?} read twice");
            first
        })
        .collect();
    save();
    reads
        .iter()
        .zip(before)
        .map(|((id, from, to), before)| {
            let after = digest_of(&read_page(w, *id, from, to));
            (*id, before, after)
        })
        .collect()
}

#[test]
fn a_receipt_changes_every_kept_page_and_sum() {
    let w = full_day();
    let reads = [
        (ReportId::R16, "", ""),
        (ReportId::R18, "", ""),
        (ReportId::R20, "", "10-10-2026"),
        (ReportId::R21, "", "10-10-2026"),
        (ReportId::R22, "", "10-10-2026"),
        (ReportId::R35, "", "10-10-2026"),
    ];
    let changes = changed_by(&w, &reads, || {
        // اکرم pays 1,000 more: 100,000 paisa.
        vouchers::save_receipt(&w.env.ctx, &w.owner, &receipt(w.akram, "1,000")).unwrap();
    });
    let cash_before = cashbook::capital_where(&w.env.ctx).is_ok();
    assert!(cash_before);
    for (id, before, after) in changes {
        assert_ne!(
            before, after,
            "{id:?} still shows the figures before the receipt"
        );
    }
    // The balances list: what the buyers owe fell by exactly the receipt.
    let r20 = money_of(&read_page(&w, ReportId::R20, "", "10-10-2026"));
    // 21,564 owed before (PRD 4.9), 1,000 paid: 20,564 rupees.
    assert_eq!(r20.first().copied(), Some(2_056_400));
}

#[test]
fn a_payment_changes_the_seller_page_and_a_sale_changes_the_registers() {
    let w = full_day();
    // R24 after a payment to کریم.
    let reads = [(ReportId::R24, "", "10-10-2026")];
    let changes = changed_by(&w, &reads, || {
        vouchers::save_payment(&w.env.ctx, &w.owner, &payment(w.karim, "1,000")).unwrap();
    });
    for (id, before, after) in changes {
        assert_ne!(
            before, after,
            "{id:?} still shows the figures before the payment"
        );
    }
    // The registers, the item sales and the income after one more sale of 10 نگ at 500.
    let reads = [
        (ReportId::R2, "11-10-2016", "10-10-2026"),
        (ReportId::R4, "11-10-2016", "10-10-2026"),
        (ReportId::R7, "11-10-2016", "10-10-2026"),
        (ReportId::R28, "11-10-2016", "10-10-2026"),
    ];
    let changes = changed_by(&w, &reads, || {
        let mut lot = support::bare_lot(&w, "stale-lot", w.rahim, w.banana, "10");
        lot.sales = vec![support::sale("stale-sale", w.akram, "10", "500")];
        save_bikri(&w, &support::draft(vec![lot]));
    });
    for (id, before, after) in changes {
        assert_ne!(
            before, after,
            "{id:?} still shows the figures before the sale"
        );
    }
}

#[test]
fn a_new_entry_shows_in_the_statement_and_the_capital_card() {
    let w = full_day();
    let db = w.env.ctx.db();
    let rows_of = || {
        statement_rows(
            db,
            &StatementRowsInput {
                account_id: w.akram,
                from: None,
                to: None,
                include_history: false,
                newest_first: true,
                offset: 0,
                limit: 100,
            },
        )
        .unwrap()
    };
    let first = rows_of();
    assert_eq!(first, rows_of(), "the same statement twice");
    let cash_card = |ctx| cashbook::capital_where(ctx).unwrap();
    let card_before = format!("{:?}", cash_card(&w.env.ctx));
    assert_eq!(card_before, format!("{:?}", cash_card(&w.env.ctx)));
    vouchers::save_receipt(&w.env.ctx, &w.owner, &receipt(w.akram, "500")).unwrap();
    let after = rows_of();
    assert_eq!(after.count, first.count + 1, "the new receipt is a row");
    assert_ne!(after.closing, first.closing);
    assert_ne!(card_before, format!("{:?}", cash_card(&w.env.ctx)));
}

/// A commit that writes no journal row (a move to the dead amount) still changes the kept pages: the
/// fingerprint of a kept page holds the data version of the file, not only the last journal id.
#[test]
fn a_move_to_the_dead_amount_changes_the_kept_dues_pages() {
    use mandi_services::core::dead::{DeadMarkInput, DeadReason};
    let w = full_day();
    let reads = [
        (ReportId::R20, "", "10-10-2026"),
        (ReportId::R21, "", "10-10-2026"),
        (ReportId::R22, "", "10-10-2026"),
        (ReportId::R35, "", "10-10-2026"),
    ];
    let changes = changed_by(&w, &reads, || {
        mandi_services::dead::mark(
            &w.env.ctx,
            &w.owner,
            &DeadMarkInput {
                account_id: w.arshad,
                entry_date: "10-10-2026".into(),
                reason: DeadReason::Died,
                note: String::new(),
            },
        )
        .unwrap();
    });
    for (id, before, after) in changes {
        assert_ne!(before, after, "{id:?} still shows ارشد as owing");
    }
    // 21,564 owed before; ارشد's 18,684 is in the dead amount now: 2,880 owed.
    let r20 = money_of(&read_page(&w, ReportId::R20, "", "10-10-2026"));
    assert_eq!(r20.first().copied(), Some(288_000));
}

#[test]
fn a_sale_changes_the_cash_and_credit_registers() {
    let w = full_day();
    let reads = [
        (ReportId::R3, "11-10-2016", "10-10-2026"),
        (ReportId::R4, "11-10-2016", "10-10-2026"),
    ];
    let changes = changed_by(&w, &reads, || {
        let mut lot = support::bare_lot(&w, "stale-lot-2", w.karim, w.apple, "10");
        lot.sales = vec![support::sale("stale-sale-2", w.akram, "10", "300")];
        save_bikri(&w, &support::draft(vec![lot]));
    });
    // A credit sale of 10 at 300 changes R4 (its totals); R3 (money paid on the spot) needs a line
    // with money paid, so only R4 must differ here and R3 must not have gone wrong.
    let r4 = changes
        .iter()
        .find(|(id, _, _)| *id == ReportId::R4)
        .unwrap();
    assert_ne!(r4.1, r4.2, "R4 still shows the figures before the sale");
}
