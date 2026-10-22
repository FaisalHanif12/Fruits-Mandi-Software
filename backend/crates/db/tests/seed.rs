//! The seeded rows equal PRD FR-10.4, row for row (plan 03 Section 6, item 3). They are the rows of
//! migration 1 and of migration 6 (the seller of trip goods, the cash name "Cash" and the controls of
//! v1.6); the 0604 account of each partner is made with the partner (plan 11), so none exists here.
//!
//! `ACCOUNTS` is typed from the FR-10.4 table, not taken from the seed code.
#![cfg(test)]

mod common;

use mandi_core::text;
use mandi_db::seed;
use tempfile::tempdir;

/// (code, Urdu name, English name, party type, is Pukhta Khata), FR-10.4 table order.
const ACCOUNTS: [(&str, &str, &str, &str, i64); 17] = [
    ("01010001", "روکڑ", "Cash", "other", 0),
    ("01020001", "بینک", "Bank", "other", 0),
    ("02010001", "مزدوری کھاتہ", "Labour Payable", "other", 0),
    ("02010002", "کرایہ کھاتہ", "Freight Payable", "other", 0),
    (
        "02010003",
        "کرایہ سٹور کھاتہ",
        "Store Rent Payable",
        "other",
        0,
    ),
    ("02010004", "منشیانہ کھاتہ", "Munshiana Payable", "other", 0),
    ("030119998", "اپنا مال", "Own Goods (Trips)", "seller", 0),
    (
        "030119999",
        "پختہ کھاتہ (بیوپاری)",
        "Pukhta Khata (Seller)",
        "seller",
        1,
    ),
    (
        "03040000",
        "نقدی منڈی",
        "Naqdi Mandi (Cash Buyer)",
        "buyer",
        0,
    ),
    (
        "03049999",
        "پختہ کھاتہ (خریدار)",
        "Pukhta Khata (Buyer)",
        "buyer",
        1,
    ),
    (
        "04010001",
        "کمیشن آمدن (بیوپاری)",
        "Commission Income (Seller)",
        "other",
        0,
    ),
    (
        "04010002",
        "کمیشن آمدن (گاہک)",
        "Commission Income (Buyer)",
        "other",
        0,
    ),
    ("04010003", "لاگا آمدن", "Laga Income", "other", 0),
    ("04010004", "فون آمدن", "Phone Charge Income", "other", 0),
    ("04010005", "ریٹ فرق", "Rate Difference", "other", 0),
    ("06010001", "ابتدائی سرمایہ", "Opening Capital", "other", 0),
    ("06019999", "ابتدائی فرق", "Opening Difference", "other", 0),
];

/// One row of `accounts` as the test reads it.
struct Row {
    code: String,
    name_ur: String,
    name_en: String,
    party_type: String,
    pukhta: i64,
    system: i64,
    active: i64,
    serial: i64,
    search_ur: String,
    search_en: String,
    search_phonetic: String,
}

#[test]
fn the_fixed_accounts_match_fr_10_4() {
    let dir = tempdir().unwrap();
    drop(common::open(dir.path()));
    let conn = common::raw(dir.path());
    let mut statement = conn
        .prepare("SELECT code, name_ur, name_en, party_type, is_pukhta_khata, is_system, is_active, serial, search_ur, search_en, search_phonetic FROM accounts ORDER BY code")
        .unwrap();
    let rows: Vec<Row> = statement
        .query_map([], |r| {
            Ok(Row {
                code: r.get(0)?,
                name_ur: r.get(1)?,
                name_en: r.get(2)?,
                party_type: r.get(3)?,
                pukhta: r.get(4)?,
                system: r.get(5)?,
                active: r.get(6)?,
                serial: r.get(7)?,
                search_ur: r.get(8)?,
                search_en: r.get(9)?,
                search_phonetic: r.get(10)?,
            })
        })
        .unwrap()
        .map(Result::unwrap)
        .collect();
    let mut want = ACCOUNTS.to_vec();
    want.sort_by_key(|row| row.0);
    assert_eq!(rows.len(), 17);
    for (row, (code, ur, en, party, pukhta)) in rows.iter().zip(want) {
        assert_eq!(
            (
                row.code.as_str(),
                row.name_ur.as_str(),
                row.name_en.as_str(),
                row.party_type.as_str(),
                row.pukhta
            ),
            (code, ur, en, party, pukhta)
        );
        assert_eq!(
            (row.system, row.active),
            (1, 1),
            "{code} is a system account and active"
        );
        // The code is the control code plus the 4-digit serial (FR-10.2).
        assert!(
            code.ends_with(&format!("{:04}", row.serial)),
            "{code} serial {}",
            row.serial
        );
        // Search keys come from core::text only (B26).
        assert_eq!(row.search_ur, text::search_ur(ur));
        assert_eq!(row.search_en, text::search_en(en));
        assert_eq!(row.search_phonetic, text::search_phonetic(en));
    }
}

#[test]
fn heads_and_controls_are_the_ones_fr_10_4_implies() {
    let dir = tempdir().unwrap();
    drop(common::open(dir.path()));
    let conn = common::raw(dir.path());
    let heads: Vec<(String, String, String)> = conn
        .prepare("SELECT code, name_en, nature FROM account_heads ORDER BY code")
        .unwrap()
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    let want_heads = [
        ("01", "Assets", "asset"),
        ("02", "Payables", "payable"),
        ("03", "Parties", "party"),
        ("04", "Income", "income"),
        ("05", "Expenses", "expense"),
        ("06", "Capital", "capital"),
    ];
    assert_eq!(heads.len(), 6);
    for (have, want) in heads.iter().zip(want_heads) {
        assert_eq!((have.0.as_str(), have.1.as_str(), have.2.as_str()), want);
    }
    let controls: Vec<(String, String, String, String)> = conn
        .prepare("SELECT c.code, c.name_en, c.party_type, h.code FROM account_controls c JOIN account_heads h ON h.id = c.head_id ORDER BY c.code")
        .unwrap()
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    let want_controls = [
        ("0101", "Cash", "other", "01"),
        ("0102", "Bank", "other", "01"),
        ("0103", "Money Lent", "other", "01"),
        ("0201", "Payables", "other", "02"),
        ("03011", "Sellers", "seller", "03"),
        ("0304", "Buyers", "buyer", "03"),
        ("0305", "Separate Khatas", "separate", "03"),
        ("0306", "Gardens", "garden", "03"),
        ("0401", "Income", "other", "04"),
        ("0501", "Shop Expenses", "other", "05"),
        ("0601", "Capital", "other", "06"),
        ("0602", "Partner Capital", "other", "06"),
        ("0603", "Drawings", "other", "06"),
        ("0604", "Gardens Trips and Fruit", "other", "06"),
    ];
    assert_eq!(controls.len(), 14);
    for (have, want) in controls.iter().zip(want_controls) {
        assert_eq!(
            (
                have.0.as_str(),
                have.1.as_str(),
                have.2.as_str(),
                have.3.as_str()
            ),
            want
        );
    }
    // Every account sits under the control its code starts with.
    let misplaced: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM accounts a JOIN account_controls c ON c.id = a.control_id WHERE substr(a.code, 1, length(c.code)) <> c.code OR length(a.code) <> length(c.code) + 4",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(misplaced, 0);
}

#[test]
fn a_second_run_changes_nothing() {
    let dir = tempdir().unwrap();
    drop(common::open(dir.path()));
    let conn = common::raw(dir.path());
    let before = seed::counts(&conn).unwrap();
    assert_eq!(
        (before.heads, before.controls, before.accounts),
        (6, 14, 17)
    );
    let changes_before = conn.total_changes();
    assert_eq!(seed::system_accounts(&conn).unwrap(), 0);
    assert_eq!(seed::trip_accounts(&conn).unwrap(), 0);
    assert_eq!(conn.total_changes(), changes_before);
    assert_eq!(seed::counts(&conn).unwrap(), before);
}

#[test]
fn the_seed_only_fills_what_is_missing() {
    let dir = tempdir().unwrap();
    drop(common::open(dir.path()));
    let conn = common::raw(dir.path());
    // Remove one head's controls and accounts is blocked for system accounts, so simulate a
    // missing control by checking the idempotent insert on a fresh, schema-only file.
    let fresh = tempdir().unwrap();
    let path = fresh.path().join("x.db");
    let mut other = mandi_db::rusqlite::Connection::open(&path).unwrap();
    other.pragma_update(None, "foreign_keys", "ON").unwrap();
    other.execute_batch(mandi_db::migrate::INIT_SQL).unwrap();
    let tx = other.transaction().unwrap();
    assert_eq!(seed::system_accounts(&tx).unwrap(), 6 + 11 + 16);
    tx.commit().unwrap();
    assert_eq!(seed::system_accounts(&other).unwrap(), 0);
    drop(conn);
}

// ---- Plan 09: the default charges of PRD 4.2 are data ----

#[test]
fn the_default_charges_are_three_rules_found_by_their_scope() {
    // PRD 4.2: seller commission 6%, buyer commission 6%, لاگا 8 per نگ on fruit. Nothing for
    // مزدوری, منشیانہ and فون.
    let dir = tempdir().unwrap();
    drop(common::open(dir.path()));
    let conn = common::raw(dir.path());
    struct Rule {
        id: i64,
        side: String,
        charge: String,
        group: Option<String>,
        method: String,
        bps: Option<i64>,
        paisa: Option<i64>,
        from: String,
        to: Option<String>,
        active: i64,
    }
    let rows: Vec<Rule> = conn
        .prepare(
            "SELECT id, side, charge_type, item_group, method, value_bps, value_paisa, from_date, to_date, is_active
             FROM charge_rules ORDER BY id",
        )
        .unwrap()
        .query_map([], |r| {
            Ok(Rule {
                id: r.get(0)?,
                side: r.get(1)?,
                charge: r.get(2)?,
                group: r.get(3)?,
                method: r.get(4)?,
                bps: r.get(5)?,
                paisa: r.get(6)?,
                from: r.get(7)?,
                to: r.get(8)?,
                active: r.get(9)?,
            })
        })
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    assert_eq!(rows.len(), 3);
    assert_eq!(
        rows.iter()
            .map(|r| (
                r.side.as_str(),
                r.charge.as_str(),
                r.group.as_deref(),
                r.method.as_str(),
                r.bps,
                r.paisa
            ))
            .collect::<Vec<_>>(),
        [
            ("seller", "commission", None, "percent", Some(600), None),
            ("buyer", "commission", None, "percent", Some(600), None),
            ("buyer", "laga", Some("fruit"), "per_unit", None, Some(800)),
        ]
    );
    assert!(
        rows.iter()
            .all(|r| r.from == "1000-01-01" && r.to.is_none() && r.active == 1)
    );
    // They are found by their scope and dates: the one in force on a day.
    use mandi_core::rules::{ChargeSide, ChargeType, ItemGroup};
    let on = |side, charge, group, date| {
        mandi_db::rules::default_in_force(&conn, side, charge, group, date)
            .unwrap()
            .map(|r| r.id)
    };
    assert_eq!(
        on(
            ChargeSide::Seller,
            ChargeType::Commission,
            None,
            "2026-10-10"
        ),
        Some(rows[0].id)
    );
    assert_eq!(
        on(
            ChargeSide::Buyer,
            ChargeType::Commission,
            None,
            "1000-01-01"
        ),
        Some(rows[1].id)
    );
    assert_eq!(
        on(
            ChargeSide::Buyer,
            ChargeType::Laga,
            Some(ItemGroup::Fruit),
            "2099-12-31"
        ),
        Some(rows[2].id)
    );
    assert_eq!(
        on(
            ChargeSide::Buyer,
            ChargeType::Laga,
            Some(ItemGroup::Vegetable),
            "2026-10-10"
        ),
        None
    );
    assert_eq!(
        on(
            ChargeSide::Seller,
            ChargeType::Commission,
            None,
            "0999-12-31"
        ),
        None,
        "before the start"
    );
}

#[test]
fn the_default_charges_are_seeded_once() {
    let dir = tempdir().unwrap();
    drop(common::open(dir.path()));
    let conn = common::raw(dir.path());
    let changes_before = conn.total_changes();
    assert_eq!(seed::default_rules(&conn).unwrap(), 0);
    assert_eq!(conn.total_changes(), changes_before);
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM charge_rules", [], |r| r.get(0))
        .unwrap();
    assert_eq!(count, 3);
}

#[test]
fn a_rule_for_a_party_together_with_a_group_has_no_level_and_is_refused() {
    // PRD 11.3 has no level for it; the table refuses it too.
    let dir = tempdir().unwrap();
    drop(common::open(dir.path()));
    let conn = common::raw(dir.path());
    let account: i64 = conn
        .query_row("SELECT id FROM accounts WHERE code = '03040000'", [], |r| {
            r.get(0)
        })
        .unwrap();
    let refused = conn.execute(
        "INSERT INTO charge_rules (side, charge_type, account_id, item_group, method, value_bps, from_date)
         VALUES ('buyer', 'commission', ?1, 'fruit', 'percent', 700, '2026-10-01')",
        [account],
    );
    assert!(refused.is_err());
}
