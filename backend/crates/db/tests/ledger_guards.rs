//! Guards added after the money review (plan 03): sale line checks that really fire, frozen
//! sale lines, matching amounts, reversals that mirror, replace guards and limits.
//! Every negative case here is an INSERT, so a CHECK is what refuses it, not a trigger.
#![cfg(test)]

mod common;

use common::{balanced_entry, bill_and_payment, classify, draft_entry, fixture, line, post, setup};
use mandi_db::DbError;
use mandi_db::rusqlite::{Connection, Error, params};

fn is_check(error: Error) -> bool {
    matches!(classify(error), DbError::Constraint(m) if m.contains("CHECK constraint failed"))
}

/// The values of a sale line that tests change. `Sale::prd()` is Akram's line of PRD 4.4.
struct Sale {
    qty: i64,
    seller_rate: i64,
    buyer_value: i64,
    buyer_comm: i64,
    laga_method: &'static str,
    laga_bps: Option<i64>,
    laga_paisa: Option<i64>,
    buyer_total: i64,
    rate_diff: i64,
}

impl Sale {
    /// 20 x 600 = 12,000; commission 720; laga 20 x 8 = 160; he pays 12,880 (in paisa).
    fn prd() -> Sale {
        Sale {
            qty: 20,
            seller_rate: 60_000,
            buyer_value: 1_200_000,
            buyer_comm: 72_000,
            laga_method: "per_unit",
            laga_bps: None,
            laga_paisa: Some(800),
            buyer_total: 1_288_000,
            rate_diff: 0,
        }
    }
}

fn insert_sale(
    conn: &Connection,
    f: &common::Fixture,
    line_no: i64,
    s: &Sale,
) -> Result<usize, Error> {
    conn.execute(
        "INSERT INTO bikri_sales (bikri_id, lot_id, line_no, journal_entry_id, entry_date, buyer_account_id, qty,
            seller_rate_paisa, seller_amount_paisa, seller_comm_method, seller_comm_value_bps, seller_comm_paisa,
            buyer_rate_paisa, buyer_value_paisa, buyer_comm_method, buyer_comm_value_bps, buyer_comm_paisa,
            laga_method, laga_value_bps, laga_value_paisa, laga_paisa, buyer_total_paisa, rate_diff_paisa, payment_type)
         VALUES (?1, ?2, ?3, ?4, '2026-10-10', ?5, ?6, ?7, 1200000, 'percent', 600, 72000,
            ?7, ?8, 'percent', 600, ?9, ?10, ?11, ?12, 16000, ?13, ?14, 'credit')",
        params![
            f.bikri, f.lot, line_no, f.entry, f.buyer, s.qty, s.seller_rate, s.buyer_value, s.buyer_comm,
            s.laga_method, s.laga_bps, s.laga_paisa, s.buyer_total, s.rate_diff
        ],
    )
}

#[test]
fn sale_line_checks_fire_on_insert() {
    let (_dir, conn) = setup();
    let f = fixture(&conn);
    let mut next = 2;
    let mut try_insert = |s: Sale| {
        next += 1;
        insert_sale(&conn, &f, next, &s)
    };
    // The PRD line itself is fine (line 3; the fixture holds line 1).
    assert!(try_insert(Sale::prd()).is_ok());
    // C5: one rupee out of line in the buyer total.
    assert!(is_check(
        try_insert(Sale {
            buyer_total: 1_288_100,
            ..Sale::prd()
        })
        .unwrap_err()
    ));
    // C19: the rate difference must be buyer value minus seller amount (here 0).
    assert!(is_check(
        try_insert(Sale {
            rate_diff: 100,
            ..Sale::prd()
        })
        .unwrap_err()
    ));
    // PRD 11.2: the buyer side has no per-lot method.
    assert!(is_check(
        try_insert(Sale {
            laga_method: "per_lot",
            ..Sale::prd()
        })
        .unwrap_err()
    ));
    // A percent method needs the percent, a per-unit method needs the amount.
    assert!(is_check(
        try_insert(Sale {
            laga_method: "percent",
            ..Sale::prd()
        })
        .unwrap_err()
    ));
    assert!(
        try_insert(Sale {
            laga_method: "percent",
            laga_bps: Some(100),
            laga_paisa: None,
            ..Sale::prd()
        })
        .is_ok()
    );
    // Part of a rupee in a commission (also breaks the total).
    assert!(is_check(
        try_insert(Sale {
            buyer_comm: 72_050,
            buyer_total: 1_288_050,
            ..Sale::prd()
        })
        .unwrap_err()
    ));
    // PRD E24: no zero quantity and no zero rate.
    assert!(is_check(
        try_insert(Sale {
            qty: 0,
            ..Sale::prd()
        })
        .unwrap_err()
    ));
    assert!(is_check(
        try_insert(Sale {
            seller_rate: 0,
            ..Sale::prd()
        })
        .unwrap_err()
    ));
    // A negative rate difference is real (PRD 12.3): buyer 590 against seller 600 on 20 nag.
    // Value 11,800, commission 708, laga 160, he pays 12,668; difference -200.
    let negative = Sale {
        buyer_value: 1_180_000,
        buyer_comm: 70_800,
        buyer_total: 1_266_800,
        rate_diff: -20_000,
        ..Sale::prd()
    };
    assert!(try_insert(negative).is_ok());
}

#[test]
fn a_saved_sale_line_is_frozen_except_for_being_replaced() {
    let (_dir, conn) = setup();
    let f = fixture(&conn);
    insert_sale(&conn, &f, 2, &Sale::prd()).unwrap();
    let second: i64 = conn
        .query_row("SELECT id FROM bikri_sales WHERE line_no = 2", [], |r| {
            r.get(0)
        })
        .unwrap();
    let allowed = [
        "is_active",
        "replaced_by_sale_id",
        "note",
        "created_at",
        "created_by",
        "updated_at",
        "updated_by",
        "version",
    ];
    let columns: Vec<String> = conn
        .prepare("SELECT name FROM pragma_table_info('bikri_sales')")
        .unwrap()
        .query_map([], |r| r.get::<_, String>(0))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    let frozen: Vec<&String> = columns
        .iter()
        .filter(|c| !allowed.contains(&c.as_str()))
        .collect();
    // 34 business columns of Architecture 15.6 (the 37 listed minus is_active,
    // replaced_by_sale_id and note) plus the id.
    assert_eq!(frozen.len(), 35, "{frozen:?}");
    for column in frozen {
        // Any change is refused by the trigger, which runs before the checks.
        // A different value for every column (the id gets a free number: it cannot be NULL).
        let value = if column == "id" {
            "9999".to_owned()
        } else {
            format!("CASE WHEN {column} IS NULL THEN 1 ELSE NULL END")
        };
        let sql = format!("UPDATE bikri_sales SET {column} = {value} WHERE id = ?1");
        let error = conn.execute(&sql, [f.sale]).unwrap_err();
        assert!(
            matches!(classify(error), DbError::AppendOnly),
            "{column} is not frozen"
        );
    }
    // An edit replaces the line: the old one becomes inactive and points at the new one.
    conn.execute(
        "UPDATE bikri_sales SET is_active = 0, replaced_by_sale_id = ?2, note = 'edited', version = 2, updated_at = '2026-10-10T08:00:00.000Z' WHERE id = ?1",
        params![f.sale, second],
    )
    .unwrap();
    // It cannot come back, and it cannot be pointed at another line.
    let back = conn
        .execute(
            "UPDATE bikri_sales SET is_active = 1 WHERE id = ?1",
            [f.sale],
        )
        .unwrap_err();
    assert!(matches!(classify(back), DbError::AppendOnly));
    let other = conn
        .execute(
            "UPDATE bikri_sales SET replaced_by_sale_id = NULL WHERE id = ?1",
            [f.sale],
        )
        .unwrap_err();
    assert!(matches!(classify(other), DbError::AppendOnly));
    // A replaced line is inactive: an active line cannot carry replaced_by_sale_id.
    let active_replaced = conn.execute(
        "INSERT INTO bikri_sales (bikri_id, lot_id, line_no, journal_entry_id, entry_date, buyer_account_id, qty,
            seller_rate_paisa, seller_amount_paisa, seller_comm_method, seller_comm_value_bps, seller_comm_paisa,
            buyer_rate_paisa, buyer_value_paisa, buyer_comm_method, buyer_comm_value_bps, buyer_comm_paisa,
            laga_method, laga_value_paisa, laga_paisa, buyer_total_paisa, payment_type, replaced_by_sale_id)
         VALUES (?1, ?2, 9, ?3, '2026-10-10', ?4, 20, 60000, 1200000, 'percent', 600, 72000, 60000, 1200000, 'percent', 600, 72000,
            'per_unit', 800, 16000, 1288000, 'credit', ?5)",
        params![f.bikri, f.lot, f.entry, f.buyer, second],
    );
    assert!(is_check(active_replaced.unwrap_err()));
}

#[test]
fn a_matching_row_never_passes_the_line_it_matches() {
    let (_dir, conn) = setup();
    let (bill, payment, buyer, _seller) = bill_and_payment(&conn);
    // A second receipt of 5,000 for the same buyer (PRD 4.6 bill 12,880, first receipt 10,000).
    let more_entry = draft_entry(&conn, 500_000);
    let more = line(&conn, more_entry, 1, "03040001", 0, 500_000).unwrap();
    let alloc = |bill_line: i64, payment_line: i64, amount: i64| {
        conn.execute(
            "INSERT INTO bill_allocations (bill_line_id, payment_line_id, account_id, amount_paisa) VALUES (?1, ?2, ?3, ?4)",
            params![bill_line, payment_line, buyer, amount],
        )
    };
    let refused = |e: Error| matches!(classify(e), DbError::Constraint(m) if m.contains("ERR_ALLOCATION_AMOUNT"));
    // The receipt of 10,000 cannot settle 10,001.
    assert!(refused(alloc(bill, payment, 1_000_100).unwrap_err()));
    assert!(alloc(bill, payment, 1_000_000).is_ok());
    assert!(
        refused(alloc(bill, payment, 100).unwrap_err()),
        "the receipt is used up"
    );
    // 12,880 - 10,000 = 2,880 is still open on the bill. 3,000 is too much, 2,880 fits... but
    // the second receipt is only 5,000, so 2,880 is allowed and the bill is then settled.
    assert!(refused(alloc(bill, more, 300_000).unwrap_err()));
    assert!(alloc(bill, more, 288_000).is_ok());
    assert!(
        refused(alloc(bill, more, 100).unwrap_err()),
        "the bill is settled"
    );
    // An undo gives the amount back: the undone row no longer counts.
    conn.execute("INSERT INTO users (username, password_hash, display_name, role) VALUES ('owner', 'x', 'Owner', 'owner')", []).unwrap();
    let user = conn.last_insert_rowid();
    conn.execute(
        "UPDATE bill_allocations SET is_active = 0, undone_at = '2026-10-10T09:00:00.000Z', undone_by = ?1 WHERE amount_paisa = 1000000",
        [user],
    )
    .unwrap();
    assert!(alloc(bill, payment, 1_000_000).is_ok());
}

#[test]
fn an_advance_matching_row_never_passes_the_advance() {
    let (_dir, conn) = setup();
    let f = fixture(&conn);
    let advance_entry = draft_entry(&conn, 1_500_000);
    conn.execute(
        "INSERT INTO journal_lines (entry_id, line_no, account_id, dr_paisa, cr_paisa, is_advance) VALUES (?1, 1, ?2, 1500000, 0, 1)",
        params![advance_entry, f.seller],
    )
    .unwrap();
    let advance = conn.last_insert_rowid();
    let credit_entry = draft_entry(&conn, 2_431_600);
    let credit = line(&conn, credit_entry, 1, "030110001", 0, 2_431_600).unwrap();
    let alloc = |amount: i64| {
        conn.execute(
            "INSERT INTO advance_allocations (advance_line_id, credit_line_id, account_id, amount_paisa) VALUES (?1, ?2, ?3, ?4)",
            params![advance, credit, f.seller, amount],
        )
    };
    // Rahim's 15,000 advance is adjusted against the 24,316 credit (PRD 15.8).
    assert!(alloc(1_500_100).is_err());
    assert!(alloc(1_500_000).is_ok());
    assert!(alloc(100).is_err(), "the advance is fully adjusted");
}

#[test]
fn a_person_key_must_match_on_both_lines() {
    let (_dir, conn) = setup();
    let pukhta = common::account_id(&conn, "03049999");
    let entry_a = draft_entry(&conn, 100_000);
    let entry_b = draft_entry(&conn, 100_000);
    let make = |entry: i64, dr: i64, cr: i64, key: &str| {
        conn.execute(
            "INSERT INTO journal_lines (entry_id, line_no, account_id, pukhta_person_key, dr_paisa, cr_paisa) VALUES (?1, 1, ?2, ?3, ?4, ?5)",
            params![entry, pukhta, key, dr, cr],
        )
        .unwrap();
        conn.last_insert_rowid()
    };
    // Shahid buys for 1,000 and pays 1,000 (PRD E2E-21). Nasir's payment never settles his bill.
    let shahid_bill = make(entry_a, 100_000, 0, "shahid|03000000000");
    let shahid_pay = make(entry_b, 0, 100_000, "shahid|03000000000");
    let nasir_pay = {
        let entry = draft_entry(&conn, 100_000);
        make(entry, 0, 100_000, "nasir|03111111111")
    };
    let alloc = |bill: i64, pay: i64, key: &str| {
        conn.execute(
            "INSERT INTO bill_allocations (bill_line_id, payment_line_id, account_id, pukhta_person_key, amount_paisa) VALUES (?1, ?2, ?3, ?4, 100000)",
            params![bill, pay, pukhta, key],
        )
    };
    assert!(alloc(shahid_bill, nasir_pay, "shahid|03000000000").is_err());
    assert!(alloc(shahid_bill, shahid_pay, "nasir|03111111111").is_err());
    assert!(alloc(shahid_bill, shahid_pay, "shahid|03000000000").is_ok());
}

/// Builds the mirror of `original` as a draft reversal and returns its id.
fn mirror(conn: &Connection, original: i64) -> i64 {
    conn.execute(
        "INSERT INTO journal_entries (entry_date, posting_type, source_type, source_id, reverses_entry_id, total_paisa)
         SELECT entry_date, 'reversal', source_type, source_id, id, total_paisa FROM journal_entries WHERE id = ?1",
        [original],
    )
    .unwrap();
    let reversal = conn.last_insert_rowid();
    conn.execute(
        "INSERT INTO journal_lines (entry_id, line_no, account_id, pukhta_person_key, dr_paisa, cr_paisa)
         SELECT ?1, line_no, account_id, pukhta_person_key, cr_paisa, dr_paisa FROM journal_lines WHERE entry_id = ?2",
        params![reversal, original],
    )
    .unwrap();
    reversal
}

#[test]
fn a_reversal_must_mirror_a_posted_entry() {
    let (_dir, conn) = setup();
    let original = balanced_entry(&conn);
    // A draft cannot be reversed (this uses a different draft, so `original` stays free).
    let draft_original = balanced_entry(&conn);
    let too_early = mirror(&conn, draft_original);
    let refused = |e: Error| matches!(classify(e), DbError::Constraint(m) if m.contains("ERR_JOURNAL_REVERSAL"));
    assert!(refused(post(&conn, too_early).unwrap_err()));
    post(&conn, original).unwrap();
    // The exact mirror posts, and entry plus reversal is zero on every account (PRD FR-6.3).
    let good = mirror(&conn, original);
    post(&conn, good).unwrap();
    let not_zero: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM (SELECT account_id, SUM(dr_paisa - cr_paisa) AS net FROM journal_lines
               WHERE entry_id IN (?1, ?2) GROUP BY account_id) WHERE net <> 0",
            params![original, good],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(not_zero, 0);
    // A balanced entry that only says it is a reversal is refused: it reverses half of the
    // lines and moves the rest to another account. Sums still balance (1,288,000 each side).
    let second_original = balanced_entry(&conn);
    post(&conn, second_original).unwrap();
    let wrong = {
        conn.execute(
            "INSERT INTO journal_entries (entry_date, posting_type, source_type, source_id, reverses_entry_id, total_paisa)
             VALUES ('2026-10-10', 'reversal', 'voucher', 1, ?1, 1288000)",
            [second_original],
        )
        .unwrap();
        let id = conn.last_insert_rowid();
        line(&conn, id, 1, "01010001", 0, 1_288_000).unwrap();
        line(&conn, id, 2, "02010002", 1_288_000, 0).unwrap();
        id
    };
    assert!(refused(post(&conn, wrong).unwrap_err()));
}

#[test]
fn a_second_reversal_cannot_slip_in_with_replace() {
    let (_dir, conn) = setup();
    let original = balanced_entry(&conn);
    post(&conn, original).unwrap();
    let reversal = mirror(&conn, original);
    post(&conn, reversal).unwrap();
    // The sqlite3 shell has no recursive triggers: a replace through the unique index must
    // still be refused.
    conn.pragma_update(None, "recursive_triggers", "OFF")
        .unwrap();
    let replace = conn.execute(
        "INSERT OR REPLACE INTO journal_entries (entry_date, posting_type, source_type, source_id, reverses_entry_id)
         VALUES ('2026-10-10', 'reversal', 'voucher', 1, ?1)",
        [original],
    );
    assert!(replace.is_err());
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM journal_entries WHERE reverses_entry_id = ?1",
            [original],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(count, 1);
}

#[test]
fn the_limit_of_1e14_paisa_is_exact() {
    let (_dir, conn) = setup();
    let entry = draft_entry(&conn, 0);
    assert!(line(&conn, entry, 1, "01010001", 100_000_000_000_000, 0).is_ok());
    assert!(line(&conn, entry, 2, "01010001", 100_000_000_000_100, 0).is_err());
    assert!(line(&conn, entry, 3, "01010001", 0, 100_000_000_000_100).is_err());
    assert!(
        conn.execute(
            "UPDATE journal_entries SET total_paisa = 100000000000000 WHERE id = ?1",
            [entry]
        )
        .is_ok()
    );
    assert!(
        conn.execute(
            "UPDATE journal_entries SET total_paisa = 100000000000100 WHERE id = ?1",
            [entry]
        )
        .is_err()
    );
}

#[test]
fn replace_is_refused_on_the_other_append_only_tables_too() {
    let (_dir, conn) = setup();
    let f = fixture(&conn);
    let cash = common::account_id(&conn, "01010001");
    let capital = common::account_id(&conn, "06010001");
    conn.execute("INSERT INTO partners (name_ur, name_en, capital_account_id, drawings_account_id) VALUES ('الف', 'Partner A', ?1, ?2)", params![capital, cash]).unwrap();
    let partner = conn.last_insert_rowid();
    conn.execute(
        "INSERT INTO partner_share_rows (journal_entry_id, entry_date, partner_id, kind, base_paisa, share_bps, amount_paisa) VALUES (?1, '2026-10-10', ?2, 'income', 394800, 4000, 158000)",
        params![f.entry, partner],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO analytics_facts (journal_entry_id, entry_date, year, month, bikri_id, lot_id, sale_id, item_id, seller_account_id, buyer_account_id,
            qty, seller_amount_paisa, seller_comm_paisa, buyer_value_paisa, buyer_comm_paisa, laga_paisa, sign)
         VALUES (?1, '2026-10-10', 2026, 10, ?2, ?3, ?4, ?5, ?6, ?7, 20, 1200000, 72000, 1200000, 72000, 16000, 1)",
        params![f.entry, f.bikri, f.lot, f.sale, f.item, f.seller, f.buyer],
    )
    .unwrap();
    let bill_entry = draft_entry(&conn, 1_288_000);
    let bill = line(&conn, bill_entry, 1, "03040001", 1_288_000, 0).unwrap();
    let pay_entry = draft_entry(&conn, 1_000_000);
    let payment = line(&conn, pay_entry, 1, "03040001", 0, 1_000_000).unwrap();
    conn.execute("INSERT INTO bill_allocations (bill_line_id, payment_line_id, account_id, amount_paisa) VALUES (?1, ?2, ?3, 1000000)", params![bill, payment, f.buyer]).unwrap();
    // A replace of the allocation that already exists, with real lines and an amount that fits,
    // so only the replace guard can refuse it.
    let more_entry = draft_entry(&conn, 500_000);
    let more = line(&conn, more_entry, 1, "03040001", 0, 500_000).unwrap();
    let replace_allocation = format!(
        "INSERT OR REPLACE INTO bill_allocations (id, bill_line_id, payment_line_id, account_id, amount_paisa) VALUES (1, {bill}, {more}, {}, 100)",
        f.buyer
    );
    conn.pragma_update(None, "recursive_triggers", "OFF")
        .unwrap();
    for (sql, table) in [
        (
            "INSERT OR REPLACE INTO partner_share_rows (id, journal_entry_id, entry_date, partner_id, kind, base_paisa, share_bps, amount_paisa) VALUES (1, 1, '2026-10-10', 1, 'income', 100, 100, 100)",
            "partner_share_rows",
        ),
        (
            "INSERT OR REPLACE INTO analytics_facts (id, journal_entry_id, entry_date, year, month, bikri_id, lot_id, sale_id, item_id, seller_account_id, buyer_account_id, qty, seller_amount_paisa, seller_comm_paisa, buyer_value_paisa, buyer_comm_paisa, laga_paisa, sign) VALUES (1, 1, '2026-10-10', 2026, 10, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1)",
            "analytics_facts",
        ),
        (replace_allocation.as_str(), "bill_allocations"),
    ] {
        let error = conn.execute(sql, []).unwrap_err();
        assert!(matches!(classify(error), DbError::AppendOnly), "{table}");
    }
}

#[test]
fn replace_by_id_and_by_line_number_are_both_refused() {
    let (_dir, conn) = setup();
    let entry = draft_entry(&conn, 0);
    let id = line(&conn, entry, 1, "01010001", 100_000, 0).unwrap();
    let cash = common::account_id(&conn, "01010001");
    conn.pragma_update(None, "recursive_triggers", "OFF")
        .unwrap();
    // Same id, another line number: the by-id trigger.
    let by_id = conn.execute(
        "INSERT OR REPLACE INTO journal_lines (id, entry_id, line_no, account_id, dr_paisa, cr_paisa) VALUES (?1, ?2, 77, ?3, 100, 0)",
        params![id, entry, cash],
    );
    assert!(matches!(classify(by_id.unwrap_err()), DbError::AppendOnly));
    // New id, same line number: the by-line-number trigger.
    let by_number = conn.execute(
        "INSERT OR REPLACE INTO journal_lines (id, entry_id, line_no, account_id, dr_paisa, cr_paisa) VALUES (9999, ?1, 1, ?2, 100, 0)",
        params![entry, cash],
    );
    assert!(matches!(
        classify(by_number.unwrap_err()),
        DbError::AppendOnly
    ));
    // A plain insert without an id is never mistaken for a replace.
    assert!(line(&conn, entry, 2, "01010001", 100, 0).is_ok());
}
