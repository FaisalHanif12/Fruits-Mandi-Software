//! The balance queries read the index only and give the same numbers as the old join
//! (migration 0002; PRD FR-6.1; NFR-SPD-2, NFR-SPD-3, NFR-SPD-9, NFR-SPD-13, NFR-SPD-21).
//!
//! The old queries are kept here as text, as the reference. A generated journal (back-dated
//! entries, reversals, a پختہ کھاتہ with persons, advances, a draft that is not posted, and an
//! account with more lines than the first step of `last_entry_date` reads) is summed both ways,
//! for every account, with and without a person and a day.
#![cfg(test)]

mod common;

use common::{account_id, setup};
use mandi_db::ledger::{
    self, ACCOUNT_TOTALS_ALL_POSTED_SQL, DayMethod, LAST_DATE_LATER_ENTRIES,
    LAST_DATE_SAMPLE_LINES, PER_LINE_UP_TO_LINES, Totals, account_totals, account_totals_sql,
    day_method, day_method_with, last_entry_date, last_entry_date_with, totals_for_accounts,
    totals_for_accounts_sql, totals_of, totals_of_by, totals_of_per_line_sql, totals_of_sql,
};
use mandi_db::rusqlite::{Connection, params};

// ---- The old queries, as they were before migration 0002 ----

const OLD_TOTALS_OF: &str = "SELECT COALESCE(SUM(l.dr_paisa), 0), COALESCE(SUM(l.cr_paisa), 0)
     FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
     WHERE e.status = 'posted' AND l.account_id = ?1 AND (?2 IS NULL OR l.pukhta_person_key = ?2)
       AND (?3 IS NULL OR e.entry_date <= ?3)";

const OLD_ACCOUNT_TOTALS: &str =
    "SELECT l.account_id, COALESCE(SUM(l.dr_paisa), 0), COALESCE(SUM(l.cr_paisa), 0)
     FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
     WHERE e.status = 'posted' AND (?1 IS NULL OR e.entry_date <= ?1)
     GROUP BY l.account_id ORDER BY l.account_id";

const OLD_LAST_DATE: &str =
    "SELECT MAX(e.entry_date) FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
     WHERE l.account_id = ?1 AND e.status = 'posted'";

fn old_totals_of(
    conn: &Connection,
    account: i64,
    person: Option<&str>,
    as_of: Option<&str>,
) -> Totals {
    conn.query_row(OLD_TOTALS_OF, params![account, person, as_of], |r| {
        Ok(Totals {
            dr_paisa: r.get(0)?,
            cr_paisa: r.get(1)?,
        })
    })
    .unwrap()
}

fn old_account_totals(conn: &Connection, as_of: Option<&str>) -> Vec<(i64, Totals)> {
    let mut statement = conn.prepare(OLD_ACCOUNT_TOTALS).unwrap();
    statement
        .query_map([as_of], |r| {
            Ok((
                r.get(0)?,
                Totals {
                    dr_paisa: r.get(1)?,
                    cr_paisa: r.get(2)?,
                },
            ))
        })
        .unwrap()
        .map(Result::unwrap)
        .collect()
}

fn old_last_date(conn: &Connection, account: i64) -> Option<String> {
    conn.query_row(OLD_LAST_DATE, [account], |r| r.get(0))
        .unwrap()
}

// ---- A generated journal ----

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
}

struct Line {
    account: i64,
    dr: i64,
    cr: i64,
    person: Option<&'static str>,
    advance: bool,
}

fn day(rng: &mut Rng) -> String {
    let n = rng.below(270);
    let month = 1 + n / 30;
    let dom = 1 + n % 28;
    format!("2026-{month:02}-{dom:02}")
}

/// Inserts a draft with these lines. Returns its id.
fn draft(conn: &Connection, date: &str, lines: &[Line], reverses: Option<i64>) -> i64 {
    let total: i64 = lines.iter().map(|l| l.dr).sum();
    // Only an advance voucher (or an opening) may carry the advance flag (trigger FR-15.8).
    let posting = if reverses.is_some() {
        "reversal"
    } else if lines.iter().any(|l| l.advance) {
        "advance"
    } else {
        "general"
    };
    conn.execute(
        "INSERT INTO journal_entries (entry_date, posting_type, source_type, source_id, reverses_entry_id, total_paisa)
         VALUES (?1, ?2, 'voucher', 1, ?3, ?4)",
        params![date, posting, reverses, total],
    )
    .unwrap();
    let entry = conn.last_insert_rowid();
    for (index, l) in lines.iter().enumerate() {
        conn.execute(
            "INSERT INTO journal_lines (entry_id, line_no, account_id, pukhta_person_key, dr_paisa, cr_paisa, is_advance)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                entry,
                i64::try_from(index + 1).unwrap(),
                l.account,
                l.person,
                l.dr,
                l.cr,
                l.advance
            ],
        )
        .unwrap();
    }
    entry
}

fn posted(conn: &Connection, date: &str, lines: &[Line], reverses: Option<i64>) -> i64 {
    let entry = draft(conn, date, lines, reverses);
    conn.execute(
        "UPDATE journal_entries SET status = 'posted' WHERE id = ?1",
        [entry],
    )
    .unwrap();
    entry
}

struct Book {
    accounts: Vec<i64>,
    cash: i64,
    pukhta: i64,
    seller: i64,
}

/// Builds a journal of `entries` postings, about four in five of them on the cash account.
fn generate(conn: &Connection, entries: usize) -> Book {
    let codes = [
        "01010001",
        "04010001",
        "04010002",
        "04010003",
        "02010001",
        "02010002",
        "03040000",
        "03049999",
        "030119999",
    ];
    let accounts: Vec<i64> = codes.iter().map(|c| account_id(conn, c)).collect();
    let cash = account_id(conn, "01010001");
    let pukhta = account_id(conn, "03049999");
    let seller = account_id(conn, "030119999");
    let persons = [Some("ali|0300"), Some("bilal|0301"), Some("chand|0302")];
    let mut rng = Rng(0x00D2_C0DE_1234_5678);
    let mut earlier: Vec<i64> = Vec::new();
    // The first entry is dated the latest of all, with the smallest id: the second step of
    // `last_entry_date` has to find it behind a thousand newer ids.
    posted(
        conn,
        "2026-12-31",
        &[
            Line {
                account: cash,
                dr: 500_000,
                cr: 0,
                person: None,
                advance: false,
            },
            Line {
                account: accounts[1],
                dr: 0,
                cr: 500_000,
                person: None,
                advance: false,
            },
        ],
        None,
    );
    for n in 0..entries {
        let amount = 100 * i64::try_from(1 + rng.below(5_000)).unwrap();
        let date = day(&mut rng);
        let debit = if rng.below(5) < 4 {
            cash
        } else {
            accounts[usize::try_from(rng.below(9)).unwrap()]
        };
        let mut credit = accounts[usize::try_from(rng.below(9)).unwrap()];
        if credit == debit {
            credit = accounts[1];
        }
        let person_of = |account: i64, rng: &mut Rng| -> Option<&'static str> {
            if account == pukhta || account == seller {
                persons[usize::try_from(rng.below(3)).unwrap()]
            } else {
                None
            }
        };
        let debit_person = person_of(debit, &mut rng);
        let credit_person = person_of(credit, &mut rng);
        let advance = debit == seller && rng.below(3) == 0;
        let mut lines = vec![
            Line {
                account: debit,
                dr: amount,
                cr: 0,
                person: debit_person,
                advance,
            },
            Line {
                account: credit,
                dr: 0,
                cr: amount,
                person: credit_person,
                advance: false,
            },
        ];
        if n % 7 == 0 {
            // A split credit: three lines.
            let part = amount / 200 * 100;
            if part > 0 {
                lines[1].cr = amount - part;
                lines.push(Line {
                    account: accounts[2],
                    dr: 0,
                    cr: part,
                    person: None,
                    advance: false,
                });
            }
        }
        let id = posted(conn, &date, &lines, None);
        earlier.push(id);
        if n % 31 == 5 {
            // A reversal of an earlier entry: the mirror, on the same date (or the original's).
            let original = earlier[usize::try_from(rng.below(earlier.len() as u64)).unwrap()];
            let already: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM journal_entries WHERE reverses_entry_id = ?1 OR posting_type = 'reversal' AND id = ?1",
                    [original],
                    |r| r.get(0),
                )
                .unwrap();
            if already == 0 {
                let (date, rows) = original_rows(conn, original);
                posted(conn, &date, &rows, Some(original));
            }
        }
    }
    Book {
        accounts,
        cash,
        pukhta,
        seller,
    }
}

/// A draft with lines that is never posted: no balance may count it.
fn leave_a_draft(conn: &Connection, book: &Book) {
    draft(
        conn,
        "2026-10-01",
        &[
            Line {
                account: book.cash,
                dr: 900_000_000,
                cr: 0,
                person: None,
                advance: false,
            },
            Line {
                account: book.accounts[1],
                dr: 0,
                cr: 900_000_000,
                person: None,
                advance: false,
            },
        ],
        None,
    );
}

/// The date and the mirrored lines of a posted entry.
fn original_rows(conn: &Connection, entry: i64) -> (String, Vec<Line>) {
    let date: String = conn
        .query_row(
            "SELECT entry_date FROM journal_entries WHERE id = ?1",
            [entry],
            |r| r.get(0),
        )
        .unwrap();
    let mut statement = conn
        .prepare("SELECT account_id, pukhta_person_key, dr_paisa, cr_paisa FROM journal_lines WHERE entry_id = ?1 ORDER BY line_no")
        .unwrap();
    let persons = ["ali|0300", "bilal|0301", "chand|0302"];
    let rows = statement
        .query_map([entry], |r| {
            let person: Option<String> = r.get(1)?;
            Ok(Line {
                account: r.get(0)?,
                dr: r.get(3)?,
                cr: r.get(2)?,
                person: person.and_then(|p| persons.iter().find(|x| **x == p).copied()),
                advance: false,
            })
        })
        .unwrap()
        .map(Result::unwrap)
        .collect();
    (date, rows)
}

#[test]
fn every_balance_query_gives_the_numbers_of_the_old_join() {
    let (_dir, conn) = setup();
    let book = generate(&conn, 1_300);
    // First with every entry posted (the all-posted pass of `account_totals`), then with a draft
    // lying around (the pass that leaves its lines out). Both must equal the old join.
    check_against_the_old_join(&conn, &book, 0);
    leave_a_draft(&conn, &book);
    check_against_the_old_join(&conn, &book, 1);
}

fn check_against_the_old_join(conn: &Connection, book: &Book, drafts: i64) {
    let unposted: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM journal_entries WHERE status <> 'posted'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(unposted, drafts);
    let lines_on_cash: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM journal_lines WHERE account_id = ?1",
            [book.cash],
            |r| r.get(0),
        )
        .unwrap();
    assert!(
        lines_on_cash > LAST_DATE_SAMPLE_LINES,
        "the cash account has {lines_on_cash} lines: not enough to reach the second step"
    );
    let reversals: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM journal_entries WHERE posting_type = 'reversal'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert!(reversals >= 10, "{reversals} reversals");

    let days: [Option<&str>; 6] = [
        None,
        Some("2025-12-31"),
        Some("2026-01-01"),
        Some("2026-05-15"),
        Some("2026-10-01"),
        Some("2026-12-31"),
    ];
    let persons: [Option<&str>; 4] = [None, Some("ali|0300"), Some("bilal|0301"), Some("nobody|0")];
    for &as_of in &days {
        // All accounts in one pass.
        assert_eq!(
            account_totals(conn, as_of).unwrap(),
            old_account_totals(conn, as_of),
            "account_totals up to {as_of:?}"
        );
        for &account in &book.accounts {
            for &person in &persons {
                let old = old_totals_of(conn, account, person, as_of);
                assert_eq!(
                    totals_of(conn, account, person, as_of).unwrap(),
                    old,
                    "account {account}, person {person:?}, up to {as_of:?}"
                );
                // Each method on its own, so the one a test file this small never picks is
                // checked as well.
                for method in [DayMethod::LeftOut, DayMethod::PerLine] {
                    assert_eq!(
                        totals_of_by(conn, method, account, person, as_of).unwrap(),
                        old,
                        "{method:?}, account {account}, person {person:?}, up to {as_of:?}"
                    );
                }
            }
        }
    }
    // An account with no line at all.
    let unused = account_id(conn, "06019999");
    assert_eq!(
        totals_of(conn, unused, None, None).unwrap(),
        Totals {
            dr_paisa: 0,
            cr_paisa: 0
        }
    );
    // A page of accounts, and one id that has no lines.
    let mut ids = book.accounts.clone();
    ids.push(unused);
    let page = totals_for_accounts(conn, &ids).unwrap();
    let old = old_account_totals(conn, None);
    assert_eq!(page.len(), old.len().min(ids.len()).max(page.len()));
    for (account, totals) in &old {
        if ids.contains(account) {
            assert_eq!(page.get(account), Some(totals), "page, account {account}");
        }
    }
    assert!(!page.contains_key(&unused));
    assert!(totals_for_accounts(conn, &[]).unwrap().is_empty());

    // The date of the last entry: the busy accounts reach the second step.
    for &account in &book.accounts {
        assert_eq!(
            last_entry_date(conn, account).unwrap(),
            old_last_date(conn, account),
            "last entry date of account {account}"
        );
    }
    assert_eq!(last_entry_date(conn, unused).unwrap(), None);
    // The cash account's newest entry is the first one made, dated far ahead of the rest.
    assert_eq!(
        last_entry_date(conn, book.cash).unwrap().as_deref(),
        Some("2026-12-31")
    );
    // The seller and the پختہ کھاتہ account both have persons and advances in the data.
    let advances: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM journal_lines WHERE is_advance = 1 AND account_id = ?1",
            [book.seller],
            |r| r.get(0),
        )
        .unwrap();
    assert!(advances > 0);
    let persons_seen: i64 = conn
        .query_row(
            "SELECT COUNT(DISTINCT pukhta_person_key) FROM journal_lines WHERE account_id = ?1",
            [book.pukhta],
            |r| r.get(0),
        )
        .unwrap();
    assert!(persons_seen >= 3);
}

#[test]
fn a_draft_is_never_part_of_a_balance_and_posting_it_adds_it() {
    let (_dir, conn) = setup();
    let cash = account_id(&conn, "01010001");
    let income = account_id(&conn, "04010001");
    let lines = [
        Line {
            account: cash,
            dr: 1_000_000,
            cr: 0,
            person: None,
            advance: false,
        },
        Line {
            account: income,
            dr: 0,
            cr: 1_000_000,
            person: None,
            advance: false,
        },
    ];
    let entry = draft(&conn, "2026-10-10", &lines, None);
    for as_of in [None, Some("2026-12-31")] {
        assert_eq!(
            totals_of(&conn, cash, None, as_of).unwrap(),
            Totals {
                dr_paisa: 0,
                cr_paisa: 0
            }
        );
        assert!(account_totals(&conn, as_of).unwrap().is_empty());
        assert_eq!(
            old_totals_of(&conn, cash, None, as_of),
            Totals {
                dr_paisa: 0,
                cr_paisa: 0
            }
        );
    }
    assert_eq!(last_entry_date(&conn, cash).unwrap(), None);
    conn.execute(
        "UPDATE journal_entries SET status = 'posted' WHERE id = ?1",
        [entry],
    )
    .unwrap();
    assert_eq!(
        totals_of(&conn, cash, None, None).unwrap(),
        Totals {
            dr_paisa: 1_000_000,
            cr_paisa: 0
        }
    );
    // A day before the entry leaves it out; the day itself includes it.
    assert_eq!(
        totals_of(&conn, cash, None, Some("2026-10-09")).unwrap(),
        Totals {
            dr_paisa: 0,
            cr_paisa: 0
        }
    );
    assert_eq!(
        totals_of(&conn, cash, None, Some("2026-10-10")).unwrap(),
        Totals {
            dr_paisa: 1_000_000,
            cr_paisa: 0
        }
    );
    assert_eq!(
        last_entry_date(&conn, cash).unwrap().as_deref(),
        Some("2026-10-10")
    );
}

// ---- Query plans (Code Standards B18) ----

fn plan(conn: &Connection, sql: &str, binds: usize) -> String {
    let mut statement = conn.prepare(&format!("EXPLAIN QUERY PLAN {sql}")).unwrap();
    let values: Vec<i64> = vec![1; binds];
    statement
        .query_map(rusqlite_params(&values), |r| r.get::<_, String>(3))
        .unwrap()
        .map(Result::unwrap)
        .collect::<Vec<_>>()
        .join("\n")
}

fn rusqlite_params(
    values: &[i64],
) -> mandi_db::rusqlite::ParamsFromIter<std::slice::Iter<'_, i64>> {
    mandi_db::rusqlite::params_from_iter(values.iter())
}

const COVERING: &str = "COVERING INDEX idx_journal_lines_account_amounts";

#[test]
fn a_balance_reads_the_amounts_index_and_never_looks_up_an_entry_per_line() {
    let (_dir, conn) = setup();
    // One account, every combination of person and day. Named parameters are given by name, so
    // the plan is read with the same text the code runs.
    for (person, date) in [(false, false), (false, true), (true, false), (true, true)] {
        let sql = totals_of_sql(person, date);
        let mut statement = conn.prepare(&format!("EXPLAIN QUERY PLAN {sql}")).unwrap();
        let mut named: Vec<(&str, &dyn mandi_db::rusqlite::ToSql)> = vec![(":account", &1_i64)];
        if person {
            named.push((":person", &"x"));
        }
        if date {
            named.push((":as_of", &"2026-10-10"));
        }
        let text: String = statement
            .query_map(named.as_slice(), |r| r.get::<_, String>(3))
            .unwrap()
            .map(Result::unwrap)
            .collect::<Vec<_>>()
            .join("\n");
        assert!(
            !text.contains("journal_entries USING INTEGER PRIMARY KEY"),
            "an entry is looked up for each line (person {person}, day {date}):\n{text}"
        );
        if person {
            // With a person the index on (account, person) is the one to use.
            assert!(text.contains("idx_journal_lines_account_person"), "{text}");
        } else {
            assert!(
                text.contains(COVERING),
                "person {person}, day {date}:\n{text}"
            );
        }
    }
    // A small account on a day: each line's entry is looked up (the cost grows with its own few
    // lines, not with the entries of the journal), and the account's index is used.
    for person in [false, true] {
        let sql = totals_of_per_line_sql(person);
        let mut statement = conn.prepare(&format!("EXPLAIN QUERY PLAN {sql}")).unwrap();
        let mut named: Vec<(&str, &dyn mandi_db::rusqlite::ToSql)> = vec![(":account", &1_i64)];
        if person {
            named.push((":person", &"x"));
        }
        named.push((":as_of", &"2026-10-10"));
        let text: String = statement
            .query_map(named.as_slice(), |r| r.get::<_, String>(3))
            .unwrap()
            .map(Result::unwrap)
            .collect::<Vec<_>>()
            .join("\n");
        assert!(text.contains("account_id=?"), "person {person}:\n{text}");
        assert!(!text.contains("SCAN journal_entries"), "{text}");
    }
    // The whole journal in one pass, and a page of accounts.
    for date in [false, true] {
        let sql = account_totals_sql(date);
        let text = plan(&conn, &sql, usize::from(date));
        assert!(
            text.contains(&format!("SCAN journal_lines USING {COVERING}")),
            "{text}"
        );
        assert!(
            !text.contains("TEMP B-TREE"),
            "a sort or group that is not needed:\n{text}"
        );
        assert!(!text.contains("INTEGER PRIMARY KEY"), "{text}");
    }
    let text = plan(&conn, &totals_for_accounts_sql(3), 3);
    assert!(text.contains(COVERING), "{text}");
    assert!(!text.contains("INTEGER PRIMARY KEY"), "{text}");
    // With every entry posted, all time is one plain pass over the index.
    let text = plan(&conn, ACCOUNT_TOTALS_ALL_POSTED_SQL, 0);
    assert!(
        text.contains(&format!("SCAN journal_lines USING {COVERING}")),
        "{text}"
    );
    assert!(
        !text.contains("TEMP B-TREE") && !text.contains("SUBQUERY"),
        "{text}"
    );
    // The check for drafts walks the (empty) partial index, not the entries.
    let text = plan(&conn, &account_totals_sql(false), 0);
    assert!(text.contains("idx_journal_entries_unposted"), "{text}");
}

#[test]
fn the_last_entry_date_and_the_advance_sums_use_their_indexes() {
    let (_dir, conn) = setup();
    // Step two of `last_entry_date`: the entries after a date, each checked by a point lookup.
    let text = plan(
        &conn,
        "SELECT MAX(e.entry_date) FROM journal_entries e
         WHERE e.status = 'posted' AND e.entry_date > ?2
           AND EXISTS (SELECT 1 FROM journal_lines l WHERE l.account_id = ?1 AND l.entry_id = e.id)",
        2,
    );
    assert!(text.contains("idx_journal_entries_entry_date"), "{text}");
    assert!(text.contains("account_id=? AND entry_id=?"), "{text}");
    // Step one: the newest lines of the account, from the index, newest first.
    let text = plan(
        &conn,
        "SELECT entry_id FROM journal_lines WHERE account_id = ?1 ORDER BY entry_id DESC LIMIT ?2",
        2,
    );
    assert!(!text.contains("TEMP B-TREE"), "{text}");
    // The count that chooses between step two and the join: the date index alone, at most the
    // limit plus one entries.
    let text = plan(
        &conn,
        "SELECT COUNT(*) FROM (SELECT 1 FROM journal_entries WHERE entry_date > ?1 LIMIT ?2)",
        2,
    );
    assert!(
        text.contains("COVERING INDEX idx_journal_entries_entry_date"),
        "{text}"
    );
    assert!(!text.contains("TEMP B-TREE"), "{text}");
    // The join for a dormant account: the lines of the account from the amounts index, one
    // point lookup of the entry for each (cost grows with the lines of the account only).
    let text = plan(
        &conn,
        "SELECT MAX(e.entry_date) FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
         WHERE l.account_id = ?1 AND e.status = 'posted'",
        1,
    );
    assert!(text.contains("account_id=?"), "{text}");
    assert!(
        !text.contains("SCAN l") && !text.contains("SCAN e"),
        "{text}"
    );
    // A seller's advances are read from the partial index.
    let text = plan(
        &conn,
        "SELECT COALESCE(SUM(l.dr_paisa), 0)
         FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
         WHERE l.account_id = ?1 AND l.is_advance = 1 AND e.status = 'posted'
           AND NOT EXISTS (SELECT 1 FROM journal_entries r
                           WHERE r.reverses_entry_id = e.id AND r.status = 'posted')",
        1,
    );
    assert!(text.contains("idx_journal_lines_advance"), "{text}");
    let _ = ledger::advance_given(&conn, 1).unwrap();
}

#[test]
fn a_day_on_a_small_account_checks_each_line_and_on_a_busy_one_leaves_out_the_later_entries() {
    // The cost of leaving out the later entries grows with every entry dated after the day (a
    // million for an early day), so a small account must not pay it (measured: 141 ms for an
    // ordinary buyer's 10-year ledger against 0.14 ms). All time is always `LeftOut`.
    let (_dir, conn) = setup();
    let book = generate(&conn, 1_300);
    let ordinary = account_id(&conn, "02010002");
    let lines = |account: i64| -> i64 {
        conn.query_row(
            "SELECT COUNT(*) FROM journal_lines WHERE account_id = ?1",
            [account],
            |r| r.get(0),
        )
        .unwrap()
    };
    const { assert!(PER_LINE_UP_TO_LINES >= 1_000, "the limit is a busy account") };
    assert!(lines(book.cash) > 1_000 && lines(ordinary) < lines(book.cash));
    // With the real limit, neither test account is busy.
    assert_eq!(
        day_method(&conn, book.cash, None, Some("2026-05-15")).unwrap(),
        DayMethod::PerLine
    );
    // With a limit between the two, the cash account is busy and the other one is not.
    let between = (lines(ordinary) + lines(book.cash)) / 2;
    assert_eq!(
        day_method_with(&conn, book.cash, None, Some("2026-05-15"), between).unwrap(),
        DayMethod::LeftOut
    );
    assert_eq!(
        day_method_with(&conn, ordinary, None, Some("2026-05-15"), between).unwrap(),
        DayMethod::PerLine
    );
    // All time never looks at the lines of the account.
    assert_eq!(
        day_method_with(&conn, ordinary, None, None, between).unwrap(),
        DayMethod::LeftOut
    );
    // One person of a پختہ کھاتہ account is counted on his own lines.
    let person_lines: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM journal_lines WHERE account_id = ?1 AND pukhta_person_key = 'ali|0300'",
            [book.pukhta],
            |r| r.get(0),
        )
        .unwrap();
    assert!(person_lines > 0 && person_lines < lines(book.pukhta));
    assert_eq!(
        day_method_with(
            &conn,
            book.pukhta,
            Some("ali|0300"),
            Some("2026-05-15"),
            person_lines
        )
        .unwrap(),
        DayMethod::PerLine
    );
    assert_eq!(
        day_method_with(
            &conn,
            book.pukhta,
            Some("ali|0300"),
            Some("2026-05-15"),
            person_lines - 1
        )
        .unwrap(),
        DayMethod::LeftOut
    );
}

#[test]
fn the_last_entry_date_is_exact_at_the_edge_of_the_first_step() {
    // `last_entry_date` reads the newest LAST_DATE_SAMPLE_LINES lines first. An account with one
    // line fewer, exactly that many, or one more must give the answer of the old join, with the
    // latest date on the line with the smallest id (behind all the newer ids) or on the newest.
    for lines in [
        LAST_DATE_SAMPLE_LINES - 1,
        LAST_DATE_SAMPLE_LINES,
        LAST_DATE_SAMPLE_LINES + 1,
    ] {
        for latest_first in [true, false] {
            let (_dir, conn) = setup();
            let account = account_id(&conn, "02010003");
            let other = account_id(&conn, "02010004");
            let pair = |dr: i64| {
                [
                    Line {
                        account,
                        dr,
                        cr: 0,
                        person: None,
                        advance: false,
                    },
                    Line {
                        account: other,
                        dr: 0,
                        cr: dr,
                        person: None,
                        advance: false,
                    },
                ]
            };
            let mut rng = Rng(0x00BE_EF00_0000_0001 ^ u64::try_from(lines).unwrap());
            for n in 0..lines {
                let date = if n == 0 && latest_first {
                    "2026-12-31".to_owned()
                } else {
                    day(&mut rng)
                };
                posted(&conn, &date, &pair(100), None);
            }
            if !latest_first {
                // The newest id is also the latest date.
                posted(&conn, "2026-12-31", &pair(100), None);
            }
            let expected = old_last_date(&conn, account);
            assert_eq!(expected.as_deref(), Some("2026-12-31"));
            assert_eq!(
                last_entry_date(&conn, account).unwrap(),
                expected,
                "{lines} lines, latest date on the first entry: {latest_first}"
            );
            // A draft that is the newest line of the account is never its last date.
            draft(&conn, "2027-06-30", &pair(100), None);
            assert_eq!(last_entry_date(&conn, account).unwrap(), expected);
            assert_eq!(old_last_date(&conn, account), expected);
        }
    }
}

/// An account with one line more than the first step reads: the first entry of the account (the
/// smallest id) carries `first_date`, the other 1,000 are dated in January 2026. Returns the
/// account and a second account that holds the other side of every entry.
fn dormant_account(conn: &Connection, first_date: Option<&str>) -> (i64, i64) {
    let account = account_id(conn, "02010003");
    let other = account_id(conn, "02010004");
    let pair = |dr: i64| {
        [
            Line {
                account,
                dr,
                cr: 0,
                person: None,
                advance: false,
            },
            Line {
                account: other,
                dr: 0,
                cr: dr,
                person: None,
                advance: false,
            },
        ]
    };
    let mut lines = 0_i64;
    if let Some(date) = first_date {
        posted(conn, date, &pair(100), None);
        lines += 1;
    }
    let mut n = 0_u64;
    while lines <= LAST_DATE_SAMPLE_LINES {
        posted(
            conn,
            &format!("2026-01-{:02}", 1 + n % 28),
            &pair(100),
            None,
        );
        lines += 1;
        n += 1;
    }
    (account, other)
}

/// Entries that do not touch `account`, dated after January.
fn later_entries_of_others(conn: &Connection, other: i64, count: usize) {
    let third = account_id(conn, "02010002");
    for n in 0..count {
        let date = format!("2026-06-{:02}", 1 + n % 28);
        posted(
            conn,
            &date,
            &[
                Line {
                    account: other,
                    dr: 100,
                    cr: 0,
                    person: None,
                    advance: false,
                },
                Line {
                    account: third,
                    dr: 0,
                    cr: 100,
                    person: None,
                    advance: false,
                },
            ],
            None,
        );
    }
}

/// How many entries are dated after the newest of the account's newest sampled lines: the number
/// `last_entry_date` compares with its limit.
fn entries_after_the_sample(conn: &Connection, account: i64) -> i64 {
    conn.query_row(
        "SELECT COUNT(*) FROM journal_entries WHERE entry_date >
           (SELECT MAX(e.entry_date) FROM journal_entries e WHERE e.status = 'posted' AND e.id IN
              (SELECT entry_id FROM journal_lines WHERE account_id = ?1
               ORDER BY entry_id DESC LIMIT ?2))",
        params![account, LAST_DATE_SAMPLE_LINES],
        |r| r.get(0),
    )
    .unwrap()
}

#[test]
fn a_dormant_account_gives_the_old_answer_on_both_sides_of_the_later_entries_limit() {
    // With more than `limit` entries dated after the sampled lines, the answer comes from the
    // join over the account's lines; with `limit` or fewer, from the look at the later entries.
    // Both must give the old join's answer, whether the account's latest date sits on a line
    // behind the sample (a back-dated first entry) or on a sampled one, and a late draft or the
    // entries of other accounts never count.
    for first_date in [None, Some("2026-03-15")] {
        for others in [0_usize, 1, 7, 25] {
            let (_dir, conn) = setup();
            let (account, other) = dormant_account(&conn, first_date);
            later_entries_of_others(&conn, other, others);
            // A draft of the account, dated after everything, is never its last date.
            let pair = [
                Line {
                    account,
                    dr: 100,
                    cr: 0,
                    person: None,
                    advance: false,
                },
                Line {
                    account: other,
                    dr: 0,
                    cr: 100,
                    person: None,
                    advance: false,
                },
            ];
            draft(&conn, "2027-06-30", &pair, None);

            let expected = old_last_date(&conn, account);
            assert_eq!(
                expected.as_deref(),
                Some(first_date.unwrap_or("2026-01-28")),
                "the oracle: {first_date:?}, {others} later entries of others"
            );
            let later = entries_after_the_sample(&conn, account);
            assert!(later >= i64::try_from(others).unwrap());
            // The limit below, at and above the number of later entries, and the extremes.
            for limit in [0, later - 1, later, later + 1, later + 100, i64::MAX] {
                assert_eq!(
                    last_entry_date_with(&conn, account, limit).unwrap(),
                    expected,
                    "limit {limit}, {later} later entries, back-dated first entry: {first_date:?}"
                );
            }
            assert_eq!(last_entry_date(&conn, account).unwrap(), expected);
        }
    }
}

#[test]
fn a_dormant_account_is_exact_at_the_real_later_entries_limit() {
    // 50,000 entries dated after the sample is the edge: the look at the later entries is used
    // for exactly that many, the join for one more. Both must find the back-dated first entry.
    let (_dir, conn) = setup();
    let (account, _other) = dormant_account(&conn, Some("2026-03-15"));
    // The first entry of the account already counts as one later entry. The others are added in
    // a few statements: drafts, two balanced lines each on two other accounts, then posted.
    let debit = account_id(&conn, "02010004");
    let credit = account_id(&conn, "02010002");
    let bulk = |count: i64| {
        conn.execute(
            "WITH RECURSIVE c(n) AS (SELECT 1 UNION ALL SELECT n + 1 FROM c WHERE n < ?1)
             INSERT INTO journal_entries
               (entry_date, posting_type, source_type, source_id, total_paisa)
             SELECT '2026-06-01', 'general', 'voucher', 1, 100 FROM c",
            [count],
        )
        .unwrap();
        for (line_no, account, dr, cr) in [(1, debit, 100, 0), (2, credit, 0, 100)] {
            conn.execute(
                "INSERT INTO journal_lines (entry_id, line_no, account_id, dr_paisa, cr_paisa)
                 SELECT id, ?1, ?2, ?3, ?4 FROM journal_entries WHERE status = 'draft'",
                params![line_no, account, dr, cr],
            )
            .unwrap();
        }
        conn.execute(
            "UPDATE journal_entries SET status = 'posted' WHERE status = 'draft'",
            [],
        )
        .unwrap();
    };
    bulk(LAST_DATE_LATER_ENTRIES - 1);
    assert_eq!(
        entries_after_the_sample(&conn, account),
        LAST_DATE_LATER_ENTRIES
    );
    let expected = old_last_date(&conn, account);
    assert_eq!(expected.as_deref(), Some("2026-03-15"));
    assert_eq!(last_entry_date(&conn, account).unwrap(), expected);
    bulk(1);
    assert_eq!(
        entries_after_the_sample(&conn, account),
        LAST_DATE_LATER_ENTRIES + 1
    );
    assert_eq!(last_entry_date(&conn, account).unwrap(), expected);
    bulk(1);
    assert_eq!(last_entry_date(&conn, account).unwrap(), expected);
}

#[test]
fn a_dormant_account_whose_newest_lines_are_all_drafts_gives_the_old_answer() {
    // One posted line, then a thousand lines of drafts (a draft exists only inside the posting
    // transaction, but the query must not depend on that): the sampled lines have no posted
    // date at all, so the answer is the posted line behind them.
    let (_dir, conn) = setup();
    let account = account_id(&conn, "02010003");
    let other = account_id(&conn, "02010004");
    let pair = |dr: i64| {
        [
            Line {
                account,
                dr,
                cr: 0,
                person: None,
                advance: false,
            },
            Line {
                account: other,
                dr: 0,
                cr: dr,
                person: None,
                advance: false,
            },
        ]
    };
    posted(&conn, "2026-02-03", &pair(100), None);
    for n in 0..LAST_DATE_SAMPLE_LINES {
        draft(
            &conn,
            &format!("2026-07-{:02}", 1 + n % 28),
            &pair(100),
            None,
        );
    }
    let expected = old_last_date(&conn, account);
    assert_eq!(expected.as_deref(), Some("2026-02-03"));
    for limit in [0, 1, 1_000, 1_001, i64::MAX] {
        assert_eq!(
            last_entry_date_with(&conn, account, limit).unwrap(),
            expected,
            "limit {limit}"
        );
    }
    assert_eq!(last_entry_date(&conn, account).unwrap(), expected);
}
