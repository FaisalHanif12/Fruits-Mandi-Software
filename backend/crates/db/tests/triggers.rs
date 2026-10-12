//! The triggers and checks of B16 and B17, with the money numbers of PRD Section 4 (plan 03
//! Section 6, item 4). Money is paisa: 12,880 rupees is 1_288_000.
#![cfg(test)]

mod common;

use common::{balanced_entry, bill_and_payment, classify, draft_entry, fixture, line, post, setup};
use mandi_db::DbError;
use mandi_db::rusqlite::{Connection, params};

#[test]
fn a_balanced_entry_posts() {
    let (_dir, conn) = setup();
    let entry = balanced_entry(&conn);
    assert_eq!(post(&conn, entry).unwrap(), 1);
    let status: String = conn
        .query_row(
            "SELECT status FROM journal_entries WHERE id = ?1",
            [entry],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(status, "posted");
}

#[test]
fn an_unbalanced_entry_is_rejected_and_stays_a_draft() {
    let (_dir, conn) = setup();
    // 36,564 rupees in, 36,562 out: the 2-rupee gap of the old software (PRD 1.1).
    let entry = draft_entry(&conn, 3_656_400);
    line(&conn, entry, 1, "01010001", 3_656_400, 0).unwrap();
    line(&conn, entry, 2, "04010001", 0, 3_656_200).unwrap();
    let error = classify(post(&conn, entry).unwrap_err());
    assert!(matches!(error, DbError::JournalUnbalanced), "{error:?}");
    let status: String = conn
        .query_row(
            "SELECT status FROM journal_entries WHERE id = ?1",
            [entry],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(status, "draft");
}

#[test]
fn total_paisa_must_equal_the_debit_total() {
    let (_dir, conn) = setup();
    let entry = draft_entry(&conn, 1_288_100);
    line(&conn, entry, 1, "01010001", 1_288_000, 0).unwrap();
    line(&conn, entry, 2, "04010001", 0, 1_288_000).unwrap();
    assert!(matches!(
        classify(post(&conn, entry).unwrap_err()),
        DbError::JournalUnbalanced
    ));
}

#[test]
fn an_entry_without_lines_cannot_post() {
    let (_dir, conn) = setup();
    let entry = draft_entry(&conn, 0);
    assert!(matches!(
        classify(post(&conn, entry).unwrap_err()),
        DbError::JournalUnbalanced
    ));
}

#[test]
fn a_posted_line_cannot_be_updated_or_deleted() {
    let (_dir, conn) = setup();
    let entry = balanced_entry(&conn);
    post(&conn, entry).unwrap();
    let update = conn
        .execute(
            "UPDATE journal_lines SET dr_paisa = 1_300_000 WHERE entry_id = ?1 AND line_no = 1",
            [entry],
        )
        .unwrap_err();
    assert!(matches!(classify(update), DbError::AppendOnly));
    let delete = conn
        .execute("DELETE FROM journal_lines WHERE entry_id = ?1", [entry])
        .unwrap_err();
    assert!(matches!(classify(delete), DbError::AppendOnly));
}

#[test]
fn a_line_cannot_be_added_to_a_posted_entry() {
    let (_dir, conn) = setup();
    let entry = balanced_entry(&conn);
    post(&conn, entry).unwrap();
    let error = line(&conn, entry, 6, "01010001", 100, 0).unwrap_err();
    assert!(matches!(classify(error), DbError::JournalPosted));
}

#[test]
fn a_posted_entry_is_final() {
    let (_dir, conn) = setup();
    let entry = balanced_entry(&conn);
    post(&conn, entry).unwrap();
    for sql in [
        "UPDATE journal_entries SET status = 'draft' WHERE id = ?1",
        "UPDATE journal_entries SET narration = 'x' WHERE id = ?1",
        "UPDATE journal_entries SET total_paisa = 0 WHERE id = ?1",
        "DELETE FROM journal_entries WHERE id = ?1",
    ] {
        let error = conn.execute(sql, [entry]).unwrap_err();
        assert!(matches!(classify(error), DbError::JournalPosted), "{sql}");
    }
}

/// A replace deletes the old row. The delete trigger fires only with recursive_triggers ON,
/// which every connection of `Db` sets. The test uses a draft entry, so the posted-entry
/// trigger is not what stops it.
#[test]
fn insert_or_replace_cannot_overwrite_a_line() {
    let (_dir, conn) = setup();
    let entry = draft_entry(&conn, 0);
    let id = line(&conn, entry, 1, "01010001", 1_288_000, 0).unwrap();
    let cash = common::account_id(&conn, "01010001");
    let replace = |conn: &Connection| {
        conn.execute(
            "INSERT OR REPLACE INTO journal_lines (id, entry_id, line_no, account_id, dr_paisa, cr_paisa) VALUES (?1, ?2, 1, ?3, 5000000, 0)",
            params![id, entry, cash],
        )
    };
    let error = replace(&conn).unwrap_err();
    assert!(matches!(classify(error), DbError::AppendOnly));
    let dr: i64 = conn
        .query_row(
            "SELECT dr_paisa FROM journal_lines WHERE id = ?1",
            [id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(dr, 1_288_000);
    // The insert triggers refuse it too, so a connection without the pragma (the sqlite3
    // shell) cannot replace a line either.
    conn.pragma_update(None, "recursive_triggers", "OFF")
        .unwrap();
    assert!(replace(&conn).is_err());
    let dr: i64 = conn
        .query_row(
            "SELECT dr_paisa FROM journal_lines WHERE id = ?1",
            [id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(dr, 1_288_000);
}

#[test]
fn line_checks_reject_bad_amounts() {
    let (_dir, conn) = setup();
    let entry = draft_entry(&conn, 0);
    // Both sides, neither side, a part of a rupee, a negative amount.
    for (dr, cr) in [
        (100, 100),
        (0, 0),
        (150, 0),
        (0, 12_345),
        (-100, 0),
        (0, -100),
    ] {
        assert!(
            line(&conn, entry, 1, "01010001", dr, cr).is_err(),
            "dr {dr} cr {cr}"
        );
    }
    assert!(line(&conn, entry, 1, "01010001", 100, 0).is_ok());
    // A repeated line number in one entry.
    assert!(line(&conn, entry, 1, "01010001", 100, 0).is_err());
    // An advance must be a debit. The entry and account are the right kind, so the CHECK is
    // what refuses it (the trigger journal_lines_advance_flag has its own test).
    let advance_entry = common::draft_entry_of(&conn, "advance", 100);
    let seller = common::account_id(&conn, "030119999");
    let credit_advance = conn.execute(
        "INSERT INTO journal_lines (entry_id, line_no, account_id, dr_paisa, cr_paisa, is_advance) VALUES (?1, 1, ?2, 0, 100, 1)",
        params![advance_entry, seller],
    );
    assert!(matches!(
        classify(credit_advance.unwrap_err()),
        DbError::Constraint(m) if m.contains("CHECK constraint failed")
    ));
}

#[test]
fn whole_rupee_check_on_other_money_columns() {
    let (_dir, conn) = setup();
    let insert = |amount: i64| {
        conn.execute(
            "INSERT INTO vouchers (type, number, entry_date, amount_paisa) VALUES ('receipt', ?1, '2026-10-10', ?2)",
            params![amount, amount],
        )
    };
    // 10,000 rupees = 1_000_000 paisa. 10,000.50 rupees is refused, and so is 0 and below.
    assert!(insert(1_000_050).is_err());
    assert!(insert(0).is_err());
    assert!(insert(-100_000).is_err());
    assert!(insert(1_000_000).is_ok());
    let opening = |dr: i64, cr: i64| {
        conn.execute(
            "UPDATE accounts SET opening_dr_paisa = ?1, opening_cr_paisa = ?2 WHERE code = '01010001'",
            params![dr, cr],
        )
    };
    assert!(opening(6_500_050, 0).is_err());
    assert!(
        opening(6_500_000, 100).is_err(),
        "only one of debit or credit"
    );
    assert!(opening(6_500_000, 0).is_ok());
}

#[test]
fn a_lot_cannot_sell_more_than_it_received() {
    let (_dir, conn) = setup();
    let f = fixture(&conn);
    assert!(
        conn.execute("UPDATE bikri_lots SET qty_sold = 51 WHERE id = ?1", [f.lot])
            .is_err()
    );
    assert!(
        conn.execute(
            "UPDATE bikri_lots SET qty_sold = 45, qty_written_off = 6 WHERE id = ?1",
            [f.lot]
        )
        .is_err()
    );
    assert!(
        conn.execute(
            "UPDATE bikri_lots SET qty_sold = 45, qty_written_off = 5 WHERE id = ?1",
            [f.lot]
        )
        .is_ok()
    );
    // Net can be negative (PRD FR-12.7), but is always gross minus deductions (C7).
    assert!(
        conn.execute(
            "UPDATE bikri_lots SET net_paisa = -100000 WHERE id = ?1",
            [f.lot]
        )
        .is_err()
    );
    assert!(conn.execute("UPDATE bikri_lots SET gross_paisa = 1_000_000, freight_paisa = 1_100_000, deductions_paisa = 1_100_000, net_paisa = -100_000 WHERE id = ?1", [f.lot]).is_ok());
    assert!(
        conn.execute(
            "UPDATE bikri_lots SET gross_paisa = 2_000_000 WHERE id = ?1",
            [f.lot]
        )
        .is_err(),
        "net no longer matches"
    );
}

#[test]
fn append_only_tables_refuse_update_and_delete() {
    let (_dir, conn) = setup();
    let f = fixture(&conn);
    conn.execute(
        "INSERT INTO stock_moves (lot_id, journal_entry_id, move_date, kind, qty) VALUES (?1, ?2, '2026-10-10', 'sold', -20)",
        params![f.lot, f.entry],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO analytics_facts (journal_entry_id, entry_date, year, month, bikri_id, lot_id, sale_id, item_id, seller_account_id, buyer_account_id,
            qty, seller_amount_paisa, seller_comm_paisa, buyer_value_paisa, buyer_comm_paisa, laga_paisa, sign)
         VALUES (?1, '2026-10-10', 2026, 10, ?2, ?3, ?4, ?5, ?6, ?7, 20, 1200000, 72000, 1200000, 72000, 16000, 1)",
        params![f.entry, f.bikri, f.lot, f.sale, f.item, f.seller, f.buyer],
    )
    .unwrap();
    let cash = common::account_id(&conn, "01010001");
    let capital = common::account_id(&conn, "06010001");
    conn.execute(
        "INSERT INTO partners (name_ur, name_en, capital_account_id, drawings_account_id) VALUES ('الف', 'Partner A', ?1, ?2)",
        params![capital, cash],
    )
    .unwrap();
    let partner = conn.last_insert_rowid();
    // Partner A, 40% of the shared income 3,948 = 1,580 after the leftover rupee (PRD 4.4).
    conn.execute(
        "INSERT INTO partner_share_rows (journal_entry_id, entry_date, partner_id, kind, base_paisa, share_bps, amount_paisa)
         VALUES (?1, '2026-10-10', ?2, 'income', 394800, 4000, 158000)",
        params![f.entry, partner],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO audit_log (at, action, entity, chain_hash) VALUES ('2026-10-10T00:00:00.000Z', 'create', 'account', ?1)",
        ["a".repeat(64)],
    )
    .unwrap();
    for table in [
        "stock_moves",
        "analytics_facts",
        "partner_share_rows",
        "audit_log",
    ] {
        let update = conn
            .execute(&format!("UPDATE {table} SET created_by = 9"), [])
            .unwrap_err();
        assert!(
            matches!(classify(update), DbError::AppendOnly),
            "update {table}"
        );
        let delete = conn
            .execute(&format!("DELETE FROM {table}"), [])
            .unwrap_err();
        assert!(
            matches!(classify(delete), DbError::AppendOnly),
            "delete {table}"
        );
    }
    let sign = conn.execute("UPDATE analytics_facts SET sign = 2", []);
    assert!(sign.is_err());
    // A fact must say the month of its date.
    let bad_month = conn.execute(
        "INSERT INTO analytics_facts (journal_entry_id, entry_date, year, month, bikri_id, lot_id, sale_id, item_id, seller_account_id, buyer_account_id,
            qty, seller_amount_paisa, seller_comm_paisa, buyer_value_paisa, buyer_comm_paisa, laga_paisa, sign)
         VALUES (?1, '2026-10-10', 2026, 9, ?2, ?3, ?4, ?5, ?6, ?7, 20, 0, 0, 0, 0, 0, 1)",
        params![f.entry, f.bikri, f.lot, f.sale, f.item, f.seller, f.buyer],
    );
    assert!(bad_month.is_err());
}

#[test]
fn matching_rows_are_never_deleted_only_undone() {
    let (_dir, conn) = setup();
    let buyer = {
        let f = fixture(&conn);
        f.buyer
    };
    let bill_entry = draft_entry(&conn, 1_288_000);
    let bill_line = {
        conn.execute("INSERT INTO journal_lines (entry_id, line_no, account_id, dr_paisa, cr_paisa) VALUES (?1, 1, ?2, 1288000, 0)", params![bill_entry, buyer]).unwrap();
        conn.last_insert_rowid()
    };
    let pay_entry = draft_entry(&conn, 1_000_000);
    let pay_line = {
        conn.execute("INSERT INTO journal_lines (entry_id, line_no, account_id, dr_paisa, cr_paisa) VALUES (?1, 1, ?2, 0, 1000000)", params![pay_entry, buyer]).unwrap();
        conn.last_insert_rowid()
    };
    conn.execute("INSERT INTO users (username, password_hash, display_name, role) VALUES ('owner', 'x', 'Owner', 'owner')", []).unwrap();
    let user = conn.last_insert_rowid();
    // Akram pays 10,000 of his 12,880 (PRD 4.6).
    conn.execute(
        "INSERT INTO bill_allocations (bill_line_id, payment_line_id, account_id, amount_paisa) VALUES (?1, ?2, ?3, 1000000)",
        params![bill_line, pay_line, buyer],
    )
    .unwrap();
    let allocation = conn.last_insert_rowid();

    let delete = conn
        .execute("DELETE FROM bill_allocations", [])
        .unwrap_err();
    assert!(matches!(classify(delete), DbError::AppendOnly));
    let amount = conn
        .execute("UPDATE bill_allocations SET amount_paisa = 1288000", [])
        .unwrap_err();
    assert!(matches!(classify(amount), DbError::AppendOnly));
    // An undo needs who and when.
    assert!(
        conn.execute("UPDATE bill_allocations SET is_active = 0", [])
            .is_err()
    );
    conn.execute(
        "UPDATE bill_allocations SET is_active = 0, undone_at = '2026-10-10T09:00:00.000Z', undone_by = ?1 WHERE id = ?2",
        params![user, allocation],
    )
    .unwrap();
    // Once undone it stays undone.
    let redo = conn
        .execute(
            "UPDATE bill_allocations SET is_active = 1, undone_at = NULL, undone_by = NULL",
            [],
        )
        .unwrap_err();
    assert!(matches!(classify(redo), DbError::AppendOnly));
    let still_deleted = conn
        .execute("DELETE FROM bill_allocations", [])
        .unwrap_err();
    assert!(matches!(classify(still_deleted), DbError::AppendOnly));
}

#[test]
fn system_accounts_cannot_be_deleted_or_recoded() {
    let (_dir, conn) = setup();
    let delete = conn
        .execute("DELETE FROM accounts WHERE code = '01010001'", [])
        .unwrap_err();
    assert!(matches!(classify(delete), DbError::SystemAccount));
    let recode = conn
        .execute(
            "UPDATE accounts SET code = '01010009' WHERE code = '01010001'",
            [],
        )
        .unwrap_err();
    assert!(matches!(classify(recode), DbError::SystemAccount));
    let demote = conn
        .execute(
            "UPDATE accounts SET is_system = 0 WHERE code = '01010001'",
            [],
        )
        .unwrap_err();
    assert!(matches!(classify(demote), DbError::SystemAccount));
    // The opening balance is typed on the account form (PRD 4.3): that update is allowed.
    conn.execute("UPDATE accounts SET opening_dr_paisa = 6_500_000, opening_date = '2026-08-31' WHERE code = '01010001'", []).unwrap();
}

#[test]
fn charge_rule_combinations_follow_prd_11_2() {
    let (_dir, conn) = setup();
    let insert = |side: &str, charge: &str, method: &str, bps: Option<i64>, paisa: Option<i64>| {
        conn.execute(
            // A vegetable-group rule, so these do not meet the one-default indexes (the seed holds
            // the defaults); the combinations are what is under test.
            "INSERT INTO charge_rules (side, charge_type, item_group, method, value_bps, value_paisa, from_date) VALUES (?1, ?2, 'vegetable', ?3, ?4, ?5, '2026-10-01')",
            params![side, charge, method, bps, paisa],
        )
    };
    // Allowed: seller commission 6%, 30 per nag, 50 per lot; labour per nag; buyer laga 8 per nag.
    assert!(insert("seller", "commission", "percent", Some(600), None).is_ok());
    assert!(insert("seller", "commission", "per_unit", None, Some(3000)).is_ok());
    assert!(insert("seller", "commission", "per_lot", None, Some(5000)).is_ok());
    assert!(insert("seller", "labour", "per_unit", None, Some(500)).is_ok());
    assert!(insert("seller", "munshiana", "per_lot", None, Some(5000)).is_ok());
    assert!(insert("seller", "phone", "per_lot", None, Some(2000)).is_ok());
    assert!(insert("buyer", "commission", "percent", Some(600), None).is_ok());
    assert!(insert("buyer", "commission", "per_unit", None, Some(1000)).is_ok());
    assert!(insert("buyer", "laga", "per_unit", None, Some(800)).is_ok());
    assert!(insert("buyer", "laga", "percent", Some(100), None).is_ok());
    // Not allowed: no such combination in PRD 11.2.
    assert!(insert("seller", "labour", "percent", Some(100), None).is_err());
    assert!(insert("seller", "laga", "per_unit", None, Some(800)).is_err());
    assert!(insert("buyer", "labour", "per_unit", None, Some(500)).is_err());
    assert!(insert("buyer", "commission", "per_lot", None, Some(500)).is_err());
    assert!(insert("buyer", "phone", "per_lot", None, Some(500)).is_err());
    // The value must match the method: exactly one of bps and paisa.
    assert!(insert("seller", "commission", "percent", None, Some(100)).is_err());
    assert!(insert("seller", "commission", "per_unit", Some(100), None).is_err());
    assert!(insert("seller", "commission", "percent", Some(600), Some(100)).is_err());
    assert!(insert("seller", "commission", "percent", Some(10_001), None).is_err());
    assert!(
        insert("seller", "commission", "per_unit", None, Some(3050)).is_err(),
        "part of a rupee"
    );
    // An item group only without an item.
    let item = conn.execute(
        "INSERT INTO items (code, name_ur, name_en, search_ur, search_en, search_phonetic, item_group) VALUES ('9', 'سیب', 'Apple', 'سیب', 'apple', 'ppl', 'fruit')",
        [],
    );
    item.unwrap();
    let item_id = conn.last_insert_rowid();
    let both = conn.execute(
        "INSERT INTO charge_rules (side, charge_type, method, value_bps, item_id, item_group, from_date) VALUES ('seller', 'commission', 'percent', 600, ?1, 'fruit', '2026-10-01')",
        [item_id],
    );
    assert!(both.is_err());
    // Dates: a real date, and not ending before it starts.
    let bad_date = conn.execute("INSERT INTO charge_rules (side, charge_type, method, value_bps, from_date) VALUES ('seller', 'commission', 'percent', 600, '2026-02-31')", []);
    assert!(bad_date.is_err());
    let backwards = conn.execute("INSERT INTO charge_rules (side, charge_type, method, value_bps, from_date, to_date) VALUES ('seller', 'commission', 'percent', 600, '2026-10-02', '2026-10-01')", []);
    assert!(backwards.is_err());
}

#[test]
fn day_state_has_one_row_and_keeps_it() {
    let (_dir, conn) = setup();
    let (id, closed): (i64, Option<String>) = conn
        .query_row("SELECT id, closed_up_to_date FROM day_state", [], |r| {
            Ok((r.get(0)?, r.get(1)?))
        })
        .unwrap();
    assert_eq!((id, closed), (1, None));
    assert!(
        conn.execute(
            "INSERT INTO day_state (id, closed_up_to_date) VALUES (2, NULL)",
            []
        )
        .is_err()
    );
    assert!(
        conn.execute(
            "INSERT INTO day_state (closed_up_to_date) VALUES ('2026-10-09')",
            []
        )
        .is_err()
    );
    assert!(conn.execute("DELETE FROM day_state", []).is_err());
    conn.execute("UPDATE day_state SET closed_up_to_date = '2026-10-09'", [])
        .unwrap();
    assert!(
        conn.execute("UPDATE day_state SET closed_up_to_date = '2026-13-09'", [])
            .is_err()
    );
}

#[test]
fn voucher_rules_for_bank_cheque_and_cancel() {
    let (_dir, conn) = setup();
    let insert = |mode: &str,
                  bank: Option<i64>,
                  cheque: Option<&str>,
                  status: &str,
                  reason: Option<&str>,
                  number: i64| {
        conn.execute(
            "INSERT INTO vouchers (type, number, entry_date, amount_paisa, mode, bank_account_id, cheque_no, cheque_date, status, cancel_reason)
             VALUES ('receipt', ?1, '2026-10-10', 1000000, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![number, mode, bank, cheque, cheque.map(|_| "2026-10-10"), status, reason],
        )
    };
    assert!(insert("cash", None, None, "saved", None, 1).is_ok());
    assert!(
        insert("bank", None, None, "saved", None, 2).is_err(),
        "bank needs a bank account"
    );
    assert!(
        insert("cash", None, None, "cancelled", None, 3).is_err(),
        "cancel needs a reason"
    );
    assert!(insert("cash", None, None, "cancelled", Some("غلط اندراج"), 3).is_ok());
    assert!(
        insert("cash", None, None, "saved", None, 1).is_err(),
        "number is unique per type"
    );
    let cash = common::account_id(&conn, "01010001");
    conn.execute("INSERT INTO bank_accounts (bank_name, title, number, account_id) VALUES ('HBL', 'Maher Adnan', '0001', ?1)", [cash]).unwrap();
    let bank = conn.last_insert_rowid();
    assert!(
        insert("cheque", Some(bank), None, "saved", None, 4).is_err(),
        "cheque needs number and date"
    );
    assert!(insert("cheque", Some(bank), Some("CH-100"), "saved", None, 4).is_ok());
}

// ---- Added after the money review (plan 03) ----

#[test]
fn an_entry_cannot_be_born_posted() {
    let (_dir, conn) = setup();
    // The balance check runs only on draft -> posted, so a posted insert would skip it.
    let insert = conn.execute(
        "INSERT INTO journal_entries (entry_date, posting_type, source_type, source_id, status, total_paisa)
         VALUES ('2026-10-10', 'general', 'voucher', 1, 'posted', 999_900)",
        [],
    );
    assert!(matches!(
        classify(insert.unwrap_err()),
        DbError::JournalPosted
    ));
    // Also for INSERT OR REPLACE over an existing posted entry, with the pragma off.
    let entry = balanced_entry(&conn);
    post(&conn, entry).unwrap();
    conn.pragma_update(None, "recursive_triggers", "OFF")
        .unwrap();
    let replace = conn.execute(
        "INSERT OR REPLACE INTO journal_entries (id, entry_date, posting_type, source_type, source_id, status, total_paisa)
         VALUES (?1, '2026-10-10', 'general', 'voucher', 1, 'draft', 0)",
        [entry],
    );
    assert!(matches!(
        classify(replace.unwrap_err()),
        DbError::JournalPosted
    ));
    let status: String = conn
        .query_row(
            "SELECT status FROM journal_entries WHERE id = ?1",
            [entry],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(status, "posted");
}

#[test]
fn an_entry_is_reversed_once_and_only_a_reversal_points_at_an_entry() {
    let (_dir, conn) = setup();
    let original = balanced_entry(&conn);
    post(&conn, original).unwrap();
    let reverse = |posting_type: &str, reverses: Option<i64>| {
        conn.execute(
            "INSERT INTO journal_entries (entry_date, posting_type, source_type, source_id, reverses_entry_id) VALUES ('2026-10-10', ?1, 'voucher', 1, ?2)",
            params![posting_type, reverses],
        )
    };
    assert!(reverse("reversal", Some(original)).is_ok());
    assert!(
        reverse("reversal", Some(original)).is_err(),
        "a second reversal of the same entry"
    );
    assert!(
        reverse("reversal", None).is_err(),
        "a reversal must say what it reverses"
    );
    assert!(
        reverse("general", Some(original)).is_err(),
        "only a reversal reverses"
    );
}

#[test]
fn a_line_is_limited_to_1e14_paisa() {
    let (_dir, conn) = setup();
    let entry = draft_entry(&conn, 0);
    // 10,00,00,00,000 rupees (the PRD limit of one value) is 1e12 paisa: allowed.
    assert!(line(&conn, entry, 1, "01010001", 1_000_000_000_000, 0).is_ok());
    assert!(line(&conn, entry, 2, "01010001", 100_000_000_000_100, 0).is_err());
    assert!(line(&conn, entry, 3, "01010001", 0, 9_000_000_000_000_000_000).is_err());
}

#[test]
fn partner_shares_are_signed_and_never_more_than_the_whole() {
    let (_dir, conn) = setup();
    let f = fixture(&conn);
    let cash = common::account_id(&conn, "01010001");
    let capital = common::account_id(&conn, "06010001");
    conn.execute("INSERT INTO partners (name_ur, name_en, capital_account_id, drawings_account_id) VALUES ('الف', 'Partner A', ?1, ?2)", params![capital, cash]).unwrap();
    let partner = conn.last_insert_rowid();
    let row = |kind: &str, base: i64, bps: i64, amount: i64| {
        conn.execute(
            "INSERT INTO partner_share_rows (journal_entry_id, entry_date, partner_id, kind, base_paisa, share_bps, amount_paisa) VALUES (?1, '2026-10-10', ?2, ?3, ?4, ?5, ?6)",
            params![f.entry, partner, kind, base, bps, amount],
        )
    };
    // E2E-20: a shop expense of 3,384 splits -1,354 / -1,015 / -1,015 (sum -3,384).
    assert!(row("expense", 338_400, 4000, -135_400).is_ok());
    assert!(row("expense", 338_400, 3000, -101_500).is_ok());
    assert!(row("expense", 338_400, 3000, -101_500).is_ok());
    // A reversal writes the opposite rows.
    assert!(row("expense", 338_400, 4000, 135_400).is_ok());
    // Not allowed: part of a rupee, and a part bigger than the whole.
    assert!(row("expense", 338_400, 4000, -135_450).is_err());
    assert!(row("income", 10_000, 4000, 9_999_900).is_err());
    assert!(row("income", 10_000, 4000, 20_000).is_err());
    assert!(
        row("income", 10_000, 4000, 10_000).is_ok(),
        "the leftover rupee can make a part equal the whole"
    );
}

#[test]
fn analytics_rows_take_their_sign_from_the_sign_column() {
    let (_dir, conn) = setup();
    let f = fixture(&conn);
    let fact = |qty: i64, comm: i64, sign: i64| {
        conn.execute(
            "INSERT INTO analytics_facts (journal_entry_id, entry_date, year, month, bikri_id, lot_id, sale_id, item_id, seller_account_id, buyer_account_id,
                qty, seller_amount_paisa, seller_comm_paisa, buyer_value_paisa, buyer_comm_paisa, laga_paisa, sign)
             VALUES (?1, '2026-10-10', 2026, 10, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 1200000, ?9, 1200000, 72000, 16000, ?10)",
            params![f.entry, f.bikri, f.lot, f.sale, f.item, f.seller, f.buyer, qty, comm, sign],
        )
    };
    assert!(fact(20, 72_000, 1).is_ok());
    assert!(fact(20, 72_000, -1).is_ok(), "the reversal row");
    assert!(
        fact(20, -72_000, -1).is_err(),
        "a negative value would add where it should subtract"
    );
    assert!(fact(0, 72_000, 1).is_err());
    let net: i64 = conn
        .query_row(
            "SELECT SUM(sign * seller_comm_paisa) FROM analytics_facts",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(net, 0, "entry plus reversal is zero");
}

#[test]
fn a_bill_matching_row_needs_a_debit_bill_and_a_credit_payment_on_its_account() {
    let (_dir, conn) = setup();
    let (bill, payment, buyer, seller) = bill_and_payment(&conn);
    let insert = |bill_line: i64,
                  payment_line: i64,
                  account: i64,
                  key: Option<&str>,
                  amount: i64| {
        conn.execute(
            "INSERT INTO bill_allocations (bill_line_id, payment_line_id, account_id, pukhta_person_key, amount_paisa) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![bill_line, payment_line, account, key, amount],
        )
    };
    assert!(
        insert(payment, bill, buyer, None, 1_000_000).is_err(),
        "directions swapped"
    );
    assert!(
        insert(bill, payment, seller, None, 1_000_000).is_err(),
        "another account"
    );
    assert!(
        insert(bill, payment, buyer, Some("shahid|0300"), 1_000_000).is_err(),
        "another person key"
    );
    assert!(
        insert(bill, payment, buyer, None, 0).is_err(),
        "zero amount"
    );
    assert!(
        insert(bill, payment, buyer, None, 1_000_050).is_err(),
        "part of a rupee"
    );
    assert!(insert(bill, payment, buyer, None, 1_000_000).is_ok());
    // The row keeps its id and lines.
    assert!(
        conn.execute("UPDATE bill_allocations SET id = 77", [])
            .is_err()
    );
}

#[test]
fn an_advance_matching_row_needs_an_advance_debit_and_a_seller_credit() {
    let (_dir, conn) = setup();
    let f = fixture(&conn);
    // Rahim: advance 15,000 on 01-09-26 (PRD 4.3), and a credit of 24,316 from the bikri.
    let advance_entry = draft_entry(&conn, 1_500_000);
    let seller_code = "030110001";
    let seller_id = f.seller;
    let plain = line(&conn, advance_entry, 1, seller_code, 1_500_000, 0).unwrap();
    // is_advance can only be set when the line is made.
    let flagged = {
        let entry = common::draft_entry_of(&conn, "advance", 1_500_000);
        conn.execute(
            "INSERT INTO journal_lines (entry_id, line_no, account_id, dr_paisa, cr_paisa, is_advance) VALUES (?1, 1, ?2, 1500000, 0, 1)",
            params![entry, seller_id],
        )
        .unwrap();
        conn.last_insert_rowid()
    };
    let credit_entry = common::draft_entry_of(&conn, "bikri_first", 2_431_600);
    let credit = line(&conn, credit_entry, 1, seller_code, 0, 2_431_600).unwrap();
    let insert = |advance_line: i64, credit_line: i64, amount: i64| {
        conn.execute(
            "INSERT INTO advance_allocations (advance_line_id, credit_line_id, account_id, amount_paisa) VALUES (?1, ?2, ?3, ?4)",
            params![advance_line, credit_line, seller_id, amount],
        )
    };
    assert!(
        insert(plain, credit, 1_500_000).is_err(),
        "a debit that is not marked as an advance"
    );
    assert!(
        insert(credit, credit, 1_500_000).is_err(),
        "the advance line must be a debit"
    );
    assert!(
        insert(flagged, flagged, 1_500_000).is_err(),
        "the credit line must be a credit"
    );
    assert!(
        insert(flagged, credit, 1_500_000).is_ok(),
        "15,000 of the 24,316"
    );
}

#[test]
fn replace_is_refused_on_every_append_only_table_even_without_the_pragma() {
    let (_dir, conn) = setup();
    let f = fixture(&conn);
    conn.execute("INSERT INTO audit_log (at, action, entity, chain_hash) VALUES ('2026-10-10T00:00:00.000Z', 'create', 'account', ?1)", ["a".repeat(64)]).unwrap();
    conn.execute("INSERT INTO stock_moves (lot_id, journal_entry_id, move_date, kind, qty) VALUES (?1, ?2, '2026-10-10', 'sold', -20)", params![f.lot, f.entry]).unwrap();
    conn.pragma_update(None, "recursive_triggers", "OFF")
        .unwrap();
    let audit = conn.execute("INSERT OR REPLACE INTO audit_log (id, at, action, entity, chain_hash) VALUES (1, '2026-10-10T00:00:00.000Z', 'edit', 'account', ?1)", ["b".repeat(64)]);
    assert!(matches!(classify(audit.unwrap_err()), DbError::AppendOnly));
    let stock = conn.execute("INSERT OR REPLACE INTO stock_moves (id, lot_id, move_date, kind, qty) VALUES (1, ?1, '2026-10-10', 'sold', -1)", [f.lot]);
    assert!(matches!(classify(stock.unwrap_err()), DbError::AppendOnly));
    let action: String = conn
        .query_row("SELECT action FROM audit_log WHERE id = 1", [], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(action, "create");
}

#[test]
fn an_opening_advance_needs_an_opening_debit() {
    let (_dir, conn) = setup();
    let set = |dr: i64, advance: i64| {
        conn.execute(
            "UPDATE accounts SET opening_dr_paisa = ?1, opening_is_advance = ?2 WHERE code = '030119999'",
            params![dr, advance],
        )
    };
    assert!(set(0, 1).is_err());
    // Rahim's 15,000 advance of PRD 4.3.
    assert!(set(1_500_000, 1).is_ok());
}

#[test]
fn two_rows_of_one_default_never_cover_the_same_day() {
    // The seed holds seller commission, buyer commission and the fruit لاگا from 1000-01-01
    // with no end. A change ends the old row the day before and starts a new one (plan 09).
    let (_dir, conn) = setup();
    let insert = |side: &str,
                  charge: &str,
                  group: Option<&str>,
                  method: &str,
                  bps: Option<i64>,
                  paisa: Option<i64>,
                  from: &str,
                  to: Option<&str>| {
        conn.execute(
            "INSERT INTO charge_rules (side, charge_type, item_group, method, value_bps, value_paisa, from_date, to_date) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![side, charge, group, method, bps, paisa, from, to],
        )
    };
    // Another row from any day is refused while the first has no end.
    assert!(
        insert(
            "seller",
            "commission",
            None,
            "percent",
            Some(700),
            None,
            "2026-10-10",
            None
        )
        .is_err()
    );
    assert!(
        insert(
            "buyer",
            "laga",
            Some("fruit"),
            "per_unit",
            None,
            Some(900),
            "2026-10-10",
            None
        )
        .is_err()
    );
    // End the old row on 09-10, then the new row from 10-10 is fine.
    conn.execute("UPDATE charge_rules SET to_date = '2026-10-09' WHERE side = 'seller' AND charge_type = 'commission'", []).unwrap();
    assert!(
        insert(
            "seller",
            "commission",
            None,
            "percent",
            Some(700),
            None,
            "2026-10-10",
            None
        )
        .is_ok()
    );
    // Moving the new row's start back over the old row's days is refused, and so is a row that
    // meets the old one on its last day.
    assert!(conn.execute("UPDATE charge_rules SET from_date = '2026-10-09' WHERE side = 'seller' AND charge_type = 'commission' AND from_date = '2026-10-10'", []).is_err());
    assert!(
        insert(
            "seller",
            "commission",
            None,
            "percent",
            Some(800),
            None,
            "2026-10-09",
            Some("2026-10-09")
        )
        .is_err()
    );
    // Other scopes are free: the vegetable group, a lot charge default, and a party rule.
    assert!(
        insert(
            "buyer",
            "laga",
            Some("vegetable"),
            "per_unit",
            None,
            Some(300),
            "2026-10-10",
            None
        )
        .is_ok()
    );
    assert!(
        insert(
            "seller",
            "labour",
            None,
            "per_unit",
            None,
            Some(500),
            "2026-10-10",
            None
        )
        .is_ok()
    );
    assert!(
        insert(
            "seller",
            "labour",
            None,
            "per_unit",
            None,
            Some(600),
            "2026-10-10",
            None
        )
        .is_ok(),
        "only the three defaults are guarded here"
    );
    assert!(
        insert(
            "seller",
            "commission",
            Some("fruit"),
            "percent",
            Some(500),
            None,
            "2026-10-10",
            None
        )
        .is_ok()
    );
}
