//! The grouped sums of R24 and R20 against the per-seller functions they replace (money review
//! finding 7, Code Standards B22). 500 sellers with credits, payments, advances, a reversed
//! entry, matching rows (active and undone) and one-time people on the پختہ کھاتہ seller account:
//! for every key the grouped answer must equal `mandi_db::vouchers::payable_totals`, the number
//! of queries must not depend on the number of sellers, and the R24 totals must equal the sum of
//! the old per-seller workings. No expected number of any other test changes.
#![cfg(test)]

mod common;
mod reports_world;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use common::*;
use mandi_reports::reports::grouped_sums::{payable_sums, pukhta_names};
use mandi_reports::reports::{ReportFilters, ReportId};
use mandi_services::core::accounts::AccountType;
use mandi_services::core::dates::BusinessDate;
use mandi_services::core::ids::AccountId;
use mandi_services::core::money::Paisa;
use mandi_services::db::vouchers::{payable_totals, pukhta_person};
use mandi_services::domain::ledger::{LedgerLine, PostingType, Side, SourceType, build_entry};
use mandi_services::domain::payables::from_totals;
use mandi_services::ledger::PostingEvent;
use reports_world::*;

/// The sellers made for the test, and the keys of the one-time people.
struct Many {
    sellers: Vec<i64>,
    pukhta_keys: Vec<String>,
}

fn post(
    w: &support::World,
    tx: &mandi_services::db::rusqlite::Transaction<'_>,
    lines: Vec<LedgerLine>,
) -> i64 {
    post_as(w, tx, PostingType::General, lines)
}

/// Posts an entry of a given type; an advance line is only allowed in an `Advance` entry.
fn post_as(
    w: &support::World,
    tx: &mandi_services::db::rusqlite::Transaction<'_>,
    posting_type: PostingType,
    lines: Vec<LedgerLine>,
) -> i64 {
    let entry = build_entry(lines).unwrap();
    mandi_services::ledger::post(
        &w.env.ctx,
        tx,
        w.owner.user_id,
        &PostingEvent {
            date: BusinessDate::from_ymd(2026, 10, 10).unwrap(),
            posting_type,
            source_type: if posting_type == PostingType::BikriFirst {
                SourceType::Bikri
            } else {
                SourceType::Voucher
            },
            source_id: 0,
            narration: None,
            reverses: None,
            entry,
        },
    )
    .unwrap()
    .entry_id
}

/// Adds `count` sellers with a mix of postings, on top of the worked day.
fn add_sellers(w: &support::World, count: i64) -> Many {
    let cash = support::id_of(&w.env, "01010001");
    let mut sellers = Vec::new();
    for i in 0..count {
        sellers.push(support::add_account(
            &w.env,
            &w.owner,
            AccountType::Seller,
            &format!("تاجر {i}"),
            &format!("Trader {i}"),
            true,
            "",
        ));
    }
    let pukhta_keys: Vec<String> = (0..6).map(|n| format!("person-{n}")).collect();
    let pukhta = w.pukhta_seller;
    let p = |rupees: i64| rupees_paisa(rupees);
    w.env
        .ctx
        .db()
        .write(|tx| {
            for (i, seller) in sellers.iter().enumerate() {
                let i = i64::try_from(i).unwrap();
                let seller = AccountId::new(*seller);
                let cash = AccountId::new(cash);
                // A credit of 1,000 to 7,000. A matching row may only point at a credit that comes
                // from a بکری posting or a receipt, so it is posted as the first posting of a بکری.
                post_as(
                    w,
                    tx,
                    PostingType::BikriFirst,
                    vec![
                        LedgerLine::new(cash, Side::Dr, p(1_000 * (1 + i % 7))),
                        LedgerLine::new(seller, Side::Cr, p(1_000 * (1 + i % 7))),
                    ],
                );
                if i % 3 == 0 {
                    post(
                        w,
                        tx,
                        vec![
                            LedgerLine::new(seller, Side::Dr, p(500 * (1 + i % 4))),
                            LedgerLine::new(cash, Side::Cr, p(500 * (1 + i % 4))),
                        ],
                    );
                }
                if i % 5 == 0 {
                    post_as(
                        w,
                        tx,
                        PostingType::Advance,
                        vec![
                            LedgerLine::new(seller, Side::Dr, p(2_000)).as_advance(),
                            LedgerLine::new(cash, Side::Cr, p(2_000)),
                        ],
                    );
                }
                if i % 11 == 0 {
                    // Posted and then reversed: neither line is live.
                    let id = post_as(
                        w,
                        tx,
                        PostingType::BikriFirst,
                        vec![
                            LedgerLine::new(cash, Side::Dr, p(900)),
                            LedgerLine::new(seller, Side::Cr, p(900)),
                        ],
                    );
                    mandi_services::ledger::reverse(&w.env.ctx, tx, w.owner.user_id, id, "test")
                        .unwrap();
                }
            }
            // One-time sellers on the پختہ کھاتہ seller account.
            let pukhta_id = AccountId::new(pukhta);
            let cash = AccountId::new(cash);
            for (n, key) in pukhta_keys.iter().enumerate() {
                let n = i64::try_from(n).unwrap();
                post_as(
                    w,
                    tx,
                    PostingType::BikriFirst,
                    vec![
                        LedgerLine::new(cash, Side::Dr, p(3_000 + 100 * n)),
                        LedgerLine::new(pukhta_id, Side::Cr, p(3_000 + 100 * n)).with_person(key),
                    ],
                );
                if n % 2 == 0 {
                    post(
                        w,
                        tx,
                        vec![
                            LedgerLine::new(pukhta_id, Side::Dr, p(4_000)).with_person(key),
                            LedgerLine::new(cash, Side::Cr, p(4_000)),
                        ],
                    );
                }
                if n % 3 == 0 {
                    post_as(
                        w,
                        tx,
                        PostingType::Advance,
                        vec![
                            LedgerLine::new(pukhta_id, Side::Dr, p(1_500))
                                .with_person(key)
                                .as_advance(),
                            LedgerLine::new(cash, Side::Cr, p(1_500)),
                        ],
                    );
                }
            }
            Ok::<(), mandi_services::db::DbError>(())
        })
        .unwrap();

    // Matching rows: an active row for every seller who has an advance and a credit, an undone
    // row for every tenth, and rows for the one-time people.
    let conn = raw(&w.dir);
    let line_of = |account: i64, key: Option<&str>, column: &str| -> Option<i64> {
        conn.query_row(
            &format!(
                "SELECT l.id FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
                 WHERE l.account_id = ?1 AND l.pukhta_person_key IS ?2 AND {column}
                   AND e.reverses_entry_id IS NULL
                   AND NOT EXISTS (SELECT 1 FROM journal_entries r WHERE r.reverses_entry_id = e.id)
                 ORDER BY l.id LIMIT 1"
            ),
            mandi_services::db::rusqlite::params![account, key],
            |r| r.get(0),
        )
        .ok()
    };
    let insert = |advance: i64, credit: i64, account: i64, key: Option<&str>, active: bool| {
        conn.execute(
            "INSERT INTO advance_allocations (advance_line_id, credit_line_id, account_id,
                 pukhta_person_key, amount_paisa, is_active, undone_at, undone_by)
             VALUES (?1, ?2, ?3, ?4, 100000, ?5, ?6, ?7)",
            mandi_services::db::rusqlite::params![
                advance,
                credit,
                account,
                key,
                i64::from(active),
                (!active).then_some("2026-10-10T00:00:00Z"),
                (!active).then_some(1_i64),
            ],
        )
        .unwrap();
    };
    for (i, seller) in sellers.iter().enumerate() {
        if i % 5 != 0 {
            continue;
        }
        if let (Some(advance), Some(credit)) = (
            line_of(*seller, None, "l.is_advance = 1"),
            line_of(*seller, None, "l.cr_paisa > 0"),
        ) {
            insert(advance, credit, *seller, None, i % 10 != 0);
        }
    }
    for key in pukhta_keys.iter().step_by(3) {
        if let (Some(advance), Some(credit)) = (
            line_of(pukhta, Some(key), "l.is_advance = 1"),
            line_of(pukhta, Some(key), "l.cr_paisa > 0"),
        ) {
            insert(advance, credit, pukhta, Some(key), true);
        }
    }
    Many {
        sellers,
        pukhta_keys,
    }
}

fn rupees_paisa(value: i64) -> Paisa {
    Paisa::from_rupees(value).unwrap()
}

#[test]
fn grouped_sums_equal_payable_totals_and_the_query_count_does_not_grow() {
    let w = worked_day();
    save(&w, &support::event1(&w));
    // The count with the worked day's few sellers.
    let few = w
        .env
        .ctx
        .db()
        .read(|conn| payable_sums(conn, None))
        .unwrap()
        .queries;

    let many = add_sellers(&w, 500);
    // A draft entry (never posted) with a seller line is not live (a guard for the cases the
    // posted data does not make).
    {
        let conn = raw(&w.dir);
        conn.execute(
            "INSERT INTO journal_entries (created_at, created_by, updated_at, updated_by, entry_date,
                 posting_type, source_type, source_id, narration, status, total_paisa)
             VALUES ('2026-10-10T00:00:00Z', 1, '2026-10-10T00:00:00Z', 1, '2026-10-10',
                 'general', 'voucher', 0, NULL, 'draft', 0)",
            [],
        )
        .unwrap();
        let draft = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO journal_lines (created_at, created_by, updated_at, updated_by, entry_id,
                 line_no, account_id, dr_paisa, cr_paisa, is_advance)
             VALUES ('2026-10-10T00:00:00Z', 1, '2026-10-10T00:00:00Z', 1, ?1, 1, ?2, 0, 900000, 0)",
            [draft, many.sellers[3]],
        )
        .unwrap();
    }
    let (sums, conn_check) = w
        .env
        .ctx
        .db()
        .read(|conn| {
            let sums = payable_sums(conn, None).unwrap();
            // Every seller, every one-time person and the keyless line of the پختہ account, and
            // the worked day's sellers: each equals the per-seller function.
            let mut keys: Vec<(i64, Option<String>)> =
                many.sellers.iter().map(|id| (*id, None)).collect();
            keys.push((w.rahim, None));
            keys.push((w.karim, None));
            keys.push((w.pukhta_seller, None));
            for key in &many.pukhta_keys {
                keys.push((w.pukhta_seller, Some(key.clone())));
            }
            let mut checked = 0_usize;
            for (account, key) in &keys {
                let old = payable_totals(conn, *account, key.as_deref()).unwrap();
                let new = sums.of(*account, key.as_deref());
                assert_eq!(
                    (
                        new.credit,
                        new.payments,
                        new.advances_given,
                        new.adjusted_on_credits,
                        new.adjusted_on_advances
                    ),
                    (
                        old.credit_paisa,
                        old.payments_paisa,
                        old.advances_given_paisa,
                        old.adjusted_on_credits_paisa,
                        old.adjusted_on_advances_paisa
                    ),
                    "account {account} key {key:?}"
                );
                checked += 1;
            }
            Ok::<_, mandi_services::db::DbError>((sums, checked))
        })
        .unwrap();
    assert!(conn_check >= 509, "{conn_check} keys compared");
    // The same number of queries for 3 sellers and for 500 (B22).
    assert_eq!(sums.queries, few);
    assert_eq!(sums.queries, 6);

    // The single-account path gives the same sums for the one-time people of the پختہ account.
    let pukhta_alone = w
        .env
        .ctx
        .db()
        .read(|conn| payable_sums(conn, Some(w.pukhta_seller)))
        .unwrap();
    for key in &many.pukhta_keys {
        assert_eq!(
            pukhta_alone.of(w.pukhta_seller, Some(key)),
            sums.of(w.pukhta_seller, Some(key)),
            "{key}"
        );
    }

    // One seller asked for alone gives the same sums as the all-sellers load.
    let one = many.sellers[5];
    let alone = w
        .env
        .ctx
        .db()
        .read(|conn| payable_sums(conn, Some(one)))
        .unwrap();
    assert_eq!(alone.of(one, None), sums.of(one, None));

    // R24: the payable, advance and owes-firm totals equal the sum of the per-seller workings.
    let (mut open, mut advance, mut owes) = (Paisa::ZERO, Paisa::ZERO, Paisa::ZERO);
    w.env
        .ctx
        .db()
        .read(|conn| {
            let mut keys: Vec<(i64, Option<String>)> =
                many.sellers.iter().map(|id| (*id, None)).collect();
            keys.push((w.rahim, None));
            keys.push((w.karim, None));
            keys.push((w.pukhta_seller, None));
            for key in &many.pukhta_keys {
                keys.push((w.pukhta_seller, Some(key.clone())));
            }
            for (account, key) in keys {
                let t = payable_totals(conn, account, key.as_deref()).unwrap();
                let p = |v: i64| Paisa::try_from_paisa(v).unwrap();
                let working = from_totals(
                    p(t.credit_paisa),
                    p(t.payments_paisa),
                    p(t.advances_given_paisa),
                    p(t.adjusted_on_credits_paisa),
                    p(t.adjusted_on_advances_paisa),
                )
                .unwrap();
                open = open.checked_add(working.open).unwrap();
                advance = advance.checked_add(working.advances_open).unwrap();
                owes = owes.checked_add(working.excess_paid).unwrap();
            }
            Ok::<_, mandi_services::db::DbError>(())
        })
        .unwrap();
    let page = report(&w, ReportId::R24, ReportFilters::default(), 0, 0);
    assert_eq!(money(page.totals.get(2).unwrap()), open.rupees());
    assert_eq!(money(page.totals.get(3).unwrap()), advance.rupees());
    assert_eq!(money(page.totals.get(4).unwrap()), owes.rupees());
    assert!(open > Paisa::ZERO && advance > Paisa::ZERO);
}

#[test]
fn the_names_of_the_one_time_people_come_from_one_query_and_equal_the_old_lookup() {
    let w = full_day();
    let conn = raw(&w.dir);
    let keys: Vec<String> = {
        let mut statement = conn
            .prepare(
                "SELECT DISTINCT pukhta_person_key FROM journal_lines
                 WHERE account_id = ?1 AND pukhta_person_key IS NOT NULL",
            )
            .unwrap();
        statement
            .query_map([w.pukhta_buyer], |r| r.get(0))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap()
    };
    assert!(!keys.is_empty(), "شاہد has a key");
    let names = pukhta_names(&conn, &[w.pukhta_buyer]).unwrap();
    for key in &keys {
        let old = pukhta_person(&conn, key).unwrap().unwrap();
        assert_eq!(names.get(key), Some(&old));
    }
    assert!(pukhta_names(&conn, &[]).unwrap().is_empty());
}
