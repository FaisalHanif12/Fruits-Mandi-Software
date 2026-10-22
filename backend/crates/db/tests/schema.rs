//! The schema equals Architecture Section 15 (plan 03 Section 6, item 1).
//!
//! `EXPECTED` is written by hand from Architecture 15.3 to 15.7. It is NOT generated from the
//! SQL, so a column missing from `0001_init.sql` fails here.
#![cfg(test)]

mod common;

use std::collections::{BTreeMap, BTreeSet};

use mandi_db::rusqlite::Connection;
use tempfile::tempdir;

/// Architecture 15.2: on every table.
const SYSTEM_COLUMNS: [&str; 6] = [
    "id",
    "created_at",
    "created_by",
    "updated_at",
    "updated_by",
    "version",
];
/// Architecture 15.4: "+ search columns".
const SEARCH: [&str; 3] = ["search_ur", "search_en", "search_phonetic"];

/// (table, columns besides the system columns), from Architecture 15.3 to 15.7.
fn expected() -> Vec<(&'static str, Vec<&'static str>)> {
    let named = |first: &[&'static str], last: &[&'static str]| -> Vec<&'static str> {
        let mut v: Vec<&'static str> = first.to_vec();
        v.extend(["name_ur", "name_en"]);
        v.extend(SEARCH);
        v.extend(last);
        v
    };
    let with_code = |last: &[&'static str]| {
        let mut v = vec!["code"];
        v.extend(named(&[], last));
        v
    };
    vec![
        // 15.3
        (
            "users",
            vec![
                "username",
                "password_hash",
                "display_name",
                "role",
                "partner_id",
                "is_active",
                "failed_attempts",
                "locked_until_at",
                "last_login_at",
            ],
        ),
        ("settings", vec!["key", "value_json"]),
        (
            "audit_log",
            vec![
                "at",
                "user_id",
                "action",
                "entity",
                "entity_id",
                "old_json",
                "new_json",
                "reason",
                "chain_hash",
            ],
        ),
        ("day_state", vec!["closed_up_to_date"]),
        (
            "day_closes",
            vec![
                "entry_date",
                "action",
                "by_user_id",
                "at",
                "reason",
                "summary_json",
            ],
        ),
        (
            "month_closes",
            vec![
                "month",
                "journal_entry_id",
                "is_active",
                "reversed_entry_id",
            ],
        ),
        (
            "drafts",
            vec!["screen", "user_id", "payload_json", "saved_at"],
        ),
        (
            "backup_log",
            vec![
                "at",
                "kind",
                "path_or_key",
                "size_bytes",
                "checksum",
                "integrity_ok",
                "row_counts_json",
                "error",
            ],
        ),
        ("user_prefs", vec!["user_id", "key", "value_json"]),
        (
            "translit_pairs",
            vec!["roman", "urdu", "kind", "use_count", "last_used_at"],
        ),
        // 15.4
        (
            "account_heads",
            vec!["code", "name_ur", "name_en", "nature"],
        ),
        (
            "account_controls",
            vec!["head_id", "code", "name_ur", "name_en", "party_type"],
        ),
        ("accounts", {
            let mut v = vec![
                "control_id",
                "code",
                "serial",
                "party_type",
                "name_ur",
                "name_en",
            ];
            v.extend(SEARCH);
            v.extend([
                "father_or_firm_ur",
                "father_or_firm_en",
                "city_id",
                "adda_id",
                "phone1",
                "phone2",
                "address",
                "linked_account_id",
                "opening_date",
                "opening_dr_paisa",
                "opening_cr_paisa",
                "opening_is_advance",
                "opening_entry_id",
                "credit_allowed",
                "credit_limit_paisa",
                "default_marka_id",
                "collector_staff_id",
                "collection_weekday",
                "is_system",
                "is_pukhta_khata",
                "is_active",
                "note",
                // Migration 0006 (PRD v1.7): the tag after a name (null for none).
                "tag",
            ]);
            v
        }),
        ("cities", with_code(&["is_active"])),
        (
            "items",
            with_code(&["item_group", "default_packing_id", "sold_by", "is_active"]),
        ),
        ("packings", with_code(&["approx_grams", "is_active"])),
        ("addas", with_code(&["city_id", "is_active"])),
        (
            "markas",
            vec!["code", "mark_text", "owner_account_id", "is_active"],
        ),
        ("stores", with_code(&["is_active"])),
        (
            "narrations",
            vec!["text_ur", "text_en", "used_in_json", "is_active"],
        ),
        ("expense_heads", with_code(&["account_id", "is_active"])),
        (
            "staff",
            named(&[], &["role", "phone", "account_id", "is_active"]),
        ),
        (
            "bank_accounts",
            vec!["bank_name", "title", "number", "account_id", "is_active"],
        ),
        // 15.5
        (
            "charge_rules",
            vec![
                "side",
                "charge_type",
                "account_id",
                "item_id",
                "item_group",
                "method",
                "value_bps",
                "value_paisa",
                "from_date",
                "to_date",
                "is_active",
                "note",
            ],
        ),
        (
            "partners",
            vec![
                "name_ur",
                "name_en",
                "user_id",
                "capital_account_id",
                "drawings_account_id",
                // Migration 0006 (plan 16b): his gardens, trips and fruit account (0604).
                "venture_account_id",
                "is_active",
            ],
        ),
        (
            "partner_shares",
            vec!["partner_id", "share_bps", "effective_from"],
        ),
        (
            "partner_share_rows",
            vec![
                "journal_entry_id",
                "entry_date",
                "partner_id",
                "kind",
                "base_paisa",
                "share_bps",
                "amount_paisa",
            ],
        ),
        // 15.6
        (
            "bikri_entries",
            vec![
                "number",
                "book",
                "entry_date",
                "bill_date",
                "truck_no",
                "zamindar_account_id",
                "adda_id",
                "driver_name",
                "driver_phone",
                "total_freight_paisa",
                "store_id",
                "note",
                "status",
                "cancel_reason",
                "cancelled_by",
                "cancelled_at",
                "print_count",
            ],
        ),
        (
            "bikri_lots",
            vec![
                "bikri_id",
                "lot_no",
                "seller_account_id",
                "pukhta_name_ur",
                "pukhta_name_en",
                "pukhta_phone",
                "pukhta_person_key",
                "item_id",
                "packing_id",
                "marka_id",
                "store_id",
                "qty_received",
                "weight_received_grams",
                "freight_paisa",
                "freight_paid_by",
                "labour_paisa",
                "labour_method",
                "labour_value_paisa",
                "labour_rule_id",
                "munshiana_paisa",
                "munshiana_method",
                "munshiana_value_paisa",
                "munshiana_rule_id",
                "phone_paisa",
                "phone_method",
                "phone_value_paisa",
                "phone_rule_id",
                "cash_paid_paisa",
                "store_rent_paisa",
                "adjust_advance",
                "first_entry_id",
                "qty_sold",
                "qty_written_off",
                "gross_paisa",
                "seller_comm_paisa",
                "deductions_paisa",
                "net_paisa",
                // Migration 0006 (plan 16b): the purchase a trip lot takes its goods from, and the fruit
                // taken from the lot (PRD FR-16.11).
                "purchase_id",
                "qty_taken",
            ],
        ),
        (
            "bikri_sales",
            vec![
                "bikri_id",
                "lot_id",
                "line_no",
                "journal_entry_id",
                "entry_date",
                "buyer_account_id",
                "pukhta_name_ur",
                "pukhta_name_en",
                "pukhta_phone",
                "pukhta_person_key",
                "qty",
                "weight_grams",
                "seller_rate_paisa",
                "seller_amount_paisa",
                "seller_comm_method",
                "seller_comm_value_bps",
                "seller_comm_value_paisa",
                "seller_comm_rule_id",
                "seller_comm_paisa",
                "buyer_rate_paisa",
                "buyer_value_paisa",
                "buyer_comm_method",
                "buyer_comm_value_bps",
                "buyer_comm_value_paisa",
                "buyer_comm_rule_id",
                "buyer_comm_paisa",
                "laga_method",
                "laga_value_bps",
                "laga_value_paisa",
                "laga_rule_id",
                "laga_paisa",
                "buyer_total_paisa",
                "rate_diff_paisa",
                "payment_type",
                "is_active",
                "replaced_by_sale_id",
                "note",
                // Migration 0005 (plan 16): how a sale line left force.
                "ended",
                // Migration 0006 (plan 16b): the cost of the own goods this line sold (C21).
                "cost_paisa",
            ],
        ),
        (
            "stock_moves",
            vec![
                "lot_id",
                "journal_entry_id",
                "move_date",
                "kind",
                "qty",
                "reason",
                // Migration 0005 (plan 16): the move this one reverses.
                "reverses_move_id",
                // Migration 0006 (plan 16b): a purchase move has a purchase and no lot, and the cost of
                // the trip goods it releases (C21).
                "purchase_id",
                "cost_paisa",
            ],
        ),
        (
            "purchases",
            vec![
                "number",
                "venture_id",
                "entry_date",
                "supplier_text",
                "item_id",
                "packing_id",
                "qty",
                "weight_grams",
                "rate_paisa",
                "value_paisa",
                "freight_paisa",
                "labour_paisa",
                "other_cost_paisa",
                "paid_now_paisa",
                "trip_cost_paisa",
                "total_cost_paisa",
                "journal_entry_id",
                "status",
                "cancel_reason",
                "note",
                "qty_sold",
                "qty_written_off",
                "cost_released_paisa",
            ],
        ),
        (
            "ventures",
            named(
                &["kind", "number", "entry_date"],
                &[
                    "item_id",
                    "rent_paisa",
                    "season_end_date",
                    "market",
                    "sent_person",
                    "account_id",
                    "status",
                    "closed_date",
                    "close_entry_id",
                    "note",
                ],
            ),
        ),
        (
            "venture_partners",
            vec!["venture_id", "partner_id", "share_bps", "source"],
        ),
        (
            "venture_entries",
            vec![
                "venture_id",
                "kind",
                "number",
                "entry_date",
                "amount_paisa",
                "reason",
                "purchase_id",
                "qty",
                "shop_name",
                "cost_paisa",
                "journal_entry_id",
                "status",
                "cancel_reason",
                "current_entry_id",
            ],
        ),
        (
            "venture_entry_parts",
            vec![
                "entry_id",
                "partner_id",
                "share_bps",
                "source",
                "amount_paisa",
            ],
        ),
        (
            "purchase_cost_parts",
            vec!["entry_id", "purchase_id", "amount_paisa"],
        ),
        (
            "lot_credit_parts",
            vec![
                "lot_id",
                "journal_entry_id",
                "partner_id",
                "share_bps",
                "amount_paisa",
            ],
        ),
        (
            "fruit_takes",
            vec![
                "number",
                "entry_date",
                "partner_id",
                "source",
                "lot_id",
                "credit_account_id",
                "pukhta_person_key",
                "item_id",
                "packing_id",
                "qty",
                "weight_grams",
                "rate_paisa",
                "value_paisa",
                "note",
                "journal_entry_id",
                "status",
                "cancel_reason",
                "current_entry_id",
            ],
        ),
        (
            "analytics_facts",
            vec![
                "journal_entry_id",
                "entry_date",
                "year",
                "month",
                "bikri_id",
                "lot_id",
                "sale_id",
                "item_id",
                "seller_account_id",
                "buyer_account_id",
                "pukhta_person_key",
                "city_id",
                "adda_id",
                "qty",
                "seller_amount_paisa",
                "seller_comm_paisa",
                "buyer_value_paisa",
                "buyer_comm_paisa",
                "laga_paisa",
                // Migration 0006 (plan 16b): trip goods in the facts.
                "purchase_id",
                "venture_id",
                "cost_paisa",
                "sign",
            ],
        ),
        // 15.7
        (
            "vouchers",
            vec![
                "type",
                "number",
                "entry_date",
                "account_id",
                "pukhta_person_key",
                "amount_paisa",
                "mode",
                "bank_account_id",
                "cheque_no",
                "cheque_date",
                "collector_staff_id",
                "partner_id",
                "expense_head_id",
                "expected_goods",
                "reason",
                "return_by_date",
                "taken_by",
                "narration",
                "status",
                "cancel_reason",
                "current_entry_id",
            ],
        ),
        (
            "journal_entries",
            vec![
                "entry_date",
                "posting_type",
                "source_type",
                "source_id",
                "reverses_entry_id",
                "narration",
                "status",
                "total_paisa",
                "chain_hash",
                // Migration 0005 (plan 16): keeps an edited posting in its place for matching.
                "order_key",
            ],
        ),
        (
            "journal_lines",
            vec![
                "entry_id",
                "line_no",
                "account_id",
                "pukhta_person_key",
                "dr_paisa",
                "cr_paisa",
                "is_advance",
                "narration",
            ],
        ),
        (
            "bill_allocations",
            vec![
                "bill_line_id",
                "payment_line_id",
                "account_id",
                "pukhta_person_key",
                "amount_paisa",
                "is_active",
                "undone_at",
                "undone_by",
            ],
        ),
        (
            "advance_allocations",
            vec![
                "advance_line_id",
                "credit_line_id",
                "account_id",
                "pukhta_person_key",
                "amount_paisa",
                "is_active",
                "undone_at",
                "undone_by",
            ],
        ),
    ]
}

fn tables(conn: &Connection) -> BTreeSet<String> {
    let mut statement = conn
        .prepare("SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%'")
        .unwrap();
    statement
        .query_map([], |row| row.get::<_, String>(0))
        .unwrap()
        .map(Result::unwrap)
        .collect()
}

fn columns(conn: &Connection, table: &str) -> BTreeSet<String> {
    let mut statement = conn
        .prepare("SELECT name FROM pragma_table_info(?1)")
        .unwrap();
    statement
        .query_map([table], |row| row.get::<_, String>(0))
        .unwrap()
        .map(Result::unwrap)
        .collect()
}

#[test]
fn table_list_equals_architecture_15() {
    let dir = tempdir().unwrap();
    drop(common::open(dir.path()));
    let conn = common::raw(dir.path());
    let want: BTreeSet<String> = expected().iter().map(|(t, _)| (*t).to_owned()).collect();
    assert_eq!(want.len(), 45);
    assert_eq!(tables(&conn), want);
}

#[test]
fn columns_equal_architecture_15() {
    let dir = tempdir().unwrap();
    drop(common::open(dir.path()));
    let conn = common::raw(dir.path());
    for (table, listed) in expected() {
        let mut want: BTreeSet<String> = SYSTEM_COLUMNS.iter().map(|c| (*c).to_owned()).collect();
        let before = want.len();
        want.extend(listed.iter().map(|c| (*c).to_owned()));
        assert_eq!(
            want.len(),
            before + listed.len(),
            "{table}: a column is listed twice"
        );
        assert_eq!(columns(&conn, table), want, "columns of {table}");
    }
}

#[test]
fn every_table_is_strict_with_system_columns_typed() {
    let dir = tempdir().unwrap();
    drop(common::open(dir.path()));
    let conn = common::raw(dir.path());
    let mut statement = conn.prepare("SELECT name, strict FROM pragma_table_list WHERE schema = 'main' AND name NOT LIKE 'sqlite_%'").unwrap();
    let rows: Vec<(String, i64)> = statement
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    assert_eq!(rows.len(), 45);
    for (name, strict) in rows {
        assert_eq!(strict, 1, "{name} is not STRICT");
        let version: (String, i64, Option<String>) = conn
            .query_row(
                "SELECT type, \"notnull\", dflt_value FROM pragma_table_info(?1) WHERE name = 'version'",
                [&name],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();
        assert_eq!(
            version,
            ("INTEGER".to_owned(), 1, Some("1".to_owned())),
            "{name}.version"
        );
    }
}

/// Code Standards B15 / M1: every `*_paisa` column checks for a whole rupee.
#[test]
fn every_paisa_column_has_the_whole_rupee_check() {
    let dir = tempdir().unwrap();
    drop(common::open(dir.path()));
    let conn = common::raw(dir.path());
    let mut found = 0;
    for (table, _) in expected() {
        let sql: String = conn
            .query_row(
                "SELECT sql FROM sqlite_master WHERE type = 'table' AND name = ?1",
                [&table],
                |row| row.get(0),
            )
            .unwrap();
        for column in columns(&conn, table)
            .into_iter()
            .filter(|c| c.ends_with("_paisa"))
        {
            found += 1;
            assert!(
                sql.contains(&format!("CHECK ({column} % 100 = 0)")),
                "{table}.{column} has no whole rupee check"
            );
            let kind: String = conn
                .query_row(
                    "SELECT type FROM pragma_table_info(?1) WHERE name = ?2",
                    [table, column.as_str()],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(kind, "INTEGER", "{table}.{column}");
        }
    }
    // Counted by hand from Architecture 15: accounts 3, charge_rules 1, partner_share_rows 2,
    // bikri_entries 1, bikri_lots 13, bikri_sales 12, analytics_facts 5, vouchers 1,
    // journal_entries 1, journal_lines 2, bill_allocations 1, advance_allocations 1; and of
    // migration 6: purchases 9, bikri_sales 1, analytics_facts 1, stock_moves 1, ventures 1,
    // venture_entries 2, venture_entry_parts 1, purchase_cost_parts 1, lot_credit_parts 1 and
    // fruit_takes 2 more.
    assert_eq!(found, 63);
}

/// Every `*_date` column checks for a real date.
#[test]
fn every_date_column_has_the_date_check() {
    let dir = tempdir().unwrap();
    drop(common::open(dir.path()));
    let conn = common::raw(dir.path());
    for (table, _) in expected() {
        let sql: String = conn
            .query_row(
                "SELECT sql FROM sqlite_master WHERE type = 'table' AND name = ?1",
                [&table],
                |row| row.get(0),
            )
            .unwrap();
        for column in columns(&conn, table)
            .into_iter()
            .filter(|c| c.ends_with("_date") || c == "effective_from")
        {
            assert!(
                sql.contains(&format!("CHECK (date({column}) IS {column})")),
                "{table}.{column}"
            );
        }
    }
}

/// The indexes of Architecture 15.8, as (table, leading columns). Written by hand.
fn listed_indexes() -> Vec<(&'static str, Vec<&'static str>)> {
    let mut v = vec![
        ("journal_lines", vec!["account_id", "entry_id"]),
        ("journal_lines", vec!["account_id", "pukhta_person_key"]),
        ("journal_entries", vec!["entry_date"]),
        ("journal_entries", vec!["source_type", "source_id"]),
        ("bikri_sales", vec!["lot_id"]),
        ("bikri_sales", vec!["buyer_account_id", "entry_date"]),
        ("bikri_sales", vec!["journal_entry_id"]),
        ("bikri_lots", vec!["bikri_id"]),
        ("bikri_lots", vec!["seller_account_id", "id"]),
        ("bikri_lots", vec!["item_id", "packing_id"]),
        (
            "bikri_lots",
            vec!["qty_received", "qty_sold", "qty_written_off"],
        ),
        ("analytics_facts", vec!["entry_date"]),
        ("analytics_facts", vec!["item_id", "entry_date"]),
        ("analytics_facts", vec!["seller_account_id", "entry_date"]),
        ("analytics_facts", vec!["buyer_account_id", "entry_date"]),
        ("partner_share_rows", vec!["partner_id", "entry_date"]),
        (
            "bill_allocations",
            vec!["account_id", "pukhta_person_key", "is_active"],
        ),
        (
            "advance_allocations",
            vec!["account_id", "pukhta_person_key", "is_active"],
        ),
        ("bikri_entries", vec!["truck_no", "entry_date"]),
        ("accounts", vec!["party_type", "serial"]),
        ("user_prefs", vec!["user_id", "key"]),
        ("translit_pairs", vec!["roman", "urdu"]),
        (
            "charge_rules",
            vec![
                "side",
                "charge_type",
                "account_id",
                "item_id",
                "item_group",
                "from_date",
            ],
        ),
    ];
    for table in [
        "accounts",
        "cities",
        "items",
        "packings",
        "addas",
        "stores",
        "expense_heads",
        "staff",
    ] {
        v.push((table, vec!["search_ur"]));
        v.push((table, vec!["search_en"]));
        v.push((table, vec!["search_phonetic"]));
    }
    for table in [
        "cities",
        "items",
        "packings",
        "addas",
        "markas",
        "stores",
        "expense_heads",
    ] {
        v.push((table, vec!["code"]));
    }
    v
}

/// All indexes of a table as lists of column names (including the automatic ones of UNIQUE).
fn index_columns(conn: &Connection, table: &str) -> Vec<Vec<String>> {
    let names: Vec<String> = conn
        .prepare("SELECT name FROM pragma_index_list(?1)")
        .unwrap()
        .query_map([table], |row| row.get::<_, String>(0))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    names
        .iter()
        .map(|name| {
            conn.prepare("SELECT name FROM pragma_index_info(?1) ORDER BY seqno")
                .unwrap()
                .query_map([name], |row| row.get::<_, String>(0))
                .unwrap()
                .map(Result::unwrap)
                .collect()
        })
        .collect()
}

#[test]
fn every_index_of_architecture_15_8_exists() {
    let dir = tempdir().unwrap();
    drop(common::open(dir.path()));
    let conn = common::raw(dir.path());
    for (table, want) in listed_indexes() {
        let have = index_columns(&conn, table);
        let want: Vec<String> = want.iter().map(|c| (*c).to_owned()).collect();
        assert!(
            have.contains(&want),
            "{table} has no index on {want:?}; has {have:?}"
        );
    }
    // The unique ones are really unique.
    for (table, columns) in [
        ("user_prefs", "user_id, key"),
        ("translit_pairs", "roman, urdu"),
        ("accounts", "code"),
    ] {
        let unique: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_index_list(?1) AS l WHERE l.\"unique\" = 1 AND (SELECT group_concat(name, ', ') FROM pragma_index_info(l.name)) = ?2",
                [table, columns],
                |row| row.get(0),
            )
            .unwrap();
        assert!(unique >= 1, "{table} ({columns}) is not unique");
    }
}

/// Code Standards B18: every foreign key column starts an index.
#[test]
fn every_foreign_key_column_has_an_index() {
    let dir = tempdir().unwrap();
    drop(common::open(dir.path()));
    let conn = common::raw(dir.path());
    let mut checked = 0;
    for (table, _) in expected() {
        let keys: Vec<String> = conn
            .prepare("SELECT \"from\" FROM pragma_foreign_key_list(?1)")
            .unwrap()
            .query_map([table], |row| row.get::<_, String>(0))
            .unwrap()
            .map(Result::unwrap)
            .collect();
        let indexes = index_columns(&conn, table);
        for key in keys {
            checked += 1;
            assert!(
                indexes.iter().any(|columns| columns.first() == Some(&key)),
                "{table}.{key} is a foreign key without an index"
            );
        }
    }
    assert!(checked > 70, "only {checked} foreign keys found");
}

#[test]
fn foreign_keys_point_at_real_tables_and_all_rows_pass_the_check() {
    let dir = tempdir().unwrap();
    drop(common::open(dir.path()));
    let conn = common::raw(dir.path());
    let bad: i64 = conn
        .query_row("SELECT COUNT(*) FROM pragma_foreign_key_check", [], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(bad, 0);
    let all: BTreeMap<String, ()> = tables(&conn).into_iter().map(|t| (t, ())).collect();
    for table in all.keys() {
        let parents: Vec<String> = conn
            .prepare("SELECT \"table\" FROM pragma_foreign_key_list(?1)")
            .unwrap()
            .query_map([table], |row| row.get::<_, String>(0))
            .unwrap()
            .map(Result::unwrap)
            .collect();
        for parent in parents {
            assert!(
                all.contains_key(&parent),
                "{table} points at missing {parent}"
            );
        }
    }
}
