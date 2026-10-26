//! Plan 32 R9, backend part: the clean-data tool keeps the Owner (same user name and password), the
//! firm's settings and the recovery code, and nothing of the business; the first seller then gets
//! 0010; the live file is never touched; a live file with partners is not cleaned by mistake.
#![cfg(test)]

mod common;

use std::path::{Path, PathBuf};

use common::*;
use mandi_services::clean_data::make_clean_file;
use mandi_services::core::accounts::{AccountInput, AccountType, OpeningLineInput, OpeningsInput};
use mandi_services::core::error::ErrorCode;
use mandi_services::core::masters::{ItemInput, MasterInput, MasterKind, PackingInput};
use mandi_services::core::partners::PartnerCreateInput;
use mandi_services::db::DbOptions;
use mandi_services::db::audit::{AuditKey, MemoryKeyStore};
use mandi_services::testing::TestEnv;
use mandi_services::{Secret, accounts, auth, masters, partners};
use std::sync::Arc;
use tempfile::TempDir;

fn live_file(dir: &TempDir) -> PathBuf {
    dir.path().join("data").join("mandi.db")
}

fn options(dir: &Path) -> DbOptions {
    let mut options = DbOptions::new(dir.join("backups"));
    options.key_store = Arc::new(MemoryKeyStore::new(AuditKey::from_bytes([7; 32])));
    options
}

fn account(kind: AccountType, ur: &str, en: &str) -> AccountInput {
    AccountInput {
        id: None,
        version: None,
        account_type: kind,
        control_id: None,
        code: String::new(),
        name_ur: ur.into(),
        name_en: en.into(),
        confirmed_rules_words: vec!["رحیم".into(), "اکرم".into(), "rahim".into(), "akram".into()],
        father_or_firm_ur: String::new(),
        father_or_firm_en: String::new(),
        city_id: None,
        adda_id: None,
        phone1: String::new(),
        phone2: String::new(),
        address: String::new(),
        linked_account_id: None,
        opening_date: String::new(),
        opening_dr: String::new(),
        opening_cr: String::new(),
        opening_is_advance: false,
        credit_allowed: true,
        credit_limit: String::new(),
        default_marka_id: None,
        collector_staff_id: None,
        collection_weekday: None,
        is_active: true,
        note: String::new(),
        tag: String::new(),
    }
}

/// A live file with an Owner, a firm with contacts and a trade mark, a seller, a buyer, an item, a
/// packing and a posted opening: a file with some business in it.
fn live_world() -> (TempDir, TestEnv, String) {
    let (dir, env) = env();
    let recovery = set_up(&env);
    login_owner(&env);
    let owner = auth::require(&env.ctx).unwrap();
    // The firm contacts and the trade mark are settings rows like any other.
    raw(&dir)
        .execute_batch(
            "INSERT INTO settings (key, value_json) VALUES
               ('firm_contacts', '[{\"name_ur\":\"منشی\",\"name_en\":\"Munshi\",\"phone\":\"0300-4058860\"}]'),
               ('trade_mark', '\"RH\"'),
               ('journal_head', '{\"id\":1,\"hash\":\"x\"}')",
        )
        .unwrap();
    let seller = accounts::save(
        &env.ctx,
        &owner,
        &account(AccountType::Seller, "رحیم", "Rahim"),
    )
    .unwrap();
    accounts::save(
        &env.ctx,
        &owner,
        &account(AccountType::Buyer, "اکرم", "Akram"),
    )
    .unwrap();
    masters::save(
        &env.ctx,
        &owner,
        &MasterInput::Item(ItemInput {
            id: None,
            version: None,
            code: String::new(),
            name_ur: "کیلا".into(),
            name_en: "Banana".into(),
            item_group: "fruit".into(),
            default_packing_id: None,
            sold_by: "unit".into(),
            confirmed_rules_words: Vec::new(),
        }),
    )
    .unwrap();
    masters::save(
        &env.ctx,
        &owner,
        &MasterInput::Packing(PackingInput {
            id: None,
            version: None,
            code: String::new(),
            name_ur: "پیٹی".into(),
            name_en: "Box".into(),
            approx_kg: String::new(),
            confirmed_rules_words: Vec::new(),
        }),
    )
    .unwrap();
    accounts::openings_save(
        &env.ctx,
        &owner,
        &OpeningsInput {
            lines: vec![OpeningLineInput {
                account_id: seller.id,
                version: seller.version,
                opening_date: "31-08-2026".into(),
                opening_dr: String::new(),
                opening_cr: "10,000".into(),
                opening_is_advance: false,
            }],
        },
    )
    .unwrap();
    (dir, env, recovery)
}

fn text_at(dir: &TempDir, sql: &str) -> Vec<String> {
    use mandi_services::db::rusqlite::types::Value;
    let conn = raw(dir);
    let mut statement = conn.prepare(sql).unwrap();
    statement
        .query_map([], |r| {
            Ok(match r.get::<_, Value>(0)? {
                Value::Integer(i) => i.to_string(),
                Value::Text(t) => t,
                other => format!("{other:?}"),
            })
        })
        .unwrap()
        .map(Result::unwrap)
        .collect()
}

#[test]
fn a_clean_file_keeps_the_owner_and_the_firm_and_nothing_of_the_business() {
    let (live_dir, live_env, _recovery) = live_world();
    let live = live_file(&live_dir);
    // The live app is running: its write-ahead file is beside the data file. Nothing may change.
    let bytes = |path: &Path| std::fs::read(path).unwrap_or_default();
    let wal = {
        let mut p = live.as_os_str().to_owned();
        p.push("-wal");
        PathBuf::from(p)
    };
    let (main_before, wal_before) = (bytes(&live), bytes(&wal));

    let clean_dir = tempfile::tempdir().unwrap();
    let new_path = live_file(&clean_dir);
    let report = make_clean_file(&live, &new_path, options(clean_dir.path()), false).unwrap();

    // The live file is exactly as it was.
    assert_eq!(bytes(&live), main_before, "the live data file changed");
    assert_eq!(bytes(&wal), wal_before, "the live write-ahead file changed");
    let left: std::collections::BTreeMap<&str, i64> = report.left_behind.iter().copied().collect();
    assert_eq!(left["items"], 1);
    assert_eq!(left["partners"], 0);
    assert!(left["accounts added (sellers, buyers, people, khatas, gardens ...)"] >= 2);

    // The report: the Owner kept, the books empty, the starting rules and the system accounts there.
    assert_eq!(report.username, "owner");
    assert_eq!(
        (report.sellers, report.buyers, report.items, report.entries),
        (0, 0, 0, 0)
    );
    assert_eq!(
        report.rules, 3,
        "the starting rules: 6%, 6% and لاگا 8 on fruit"
    );
    assert!(report.system_accounts >= 17);
    for key in ["firm", "firm_contacts", "trade_mark", "recovery_code_hash"] {
        assert!(
            report.kept_settings.contains(&key.to_owned()),
            "{key} is kept: {:?}",
            report.kept_settings
        );
    }
    assert!(!report.kept_settings.contains(&"journal_head".to_owned()));

    // The new file: the settings rows are the live ones, byte for byte.
    for key in ["firm", "firm_contacts", "trade_mark", "recovery_code_hash"] {
        let sql = format!("SELECT value_json FROM settings WHERE key = '{key}'");
        assert_eq!(
            text_at(&clean_dir, &sql),
            text_at(&live_dir, &sql),
            "setting {key}"
        );
    }
    assert!(
        text_at(
            &clean_dir,
            "SELECT key FROM settings WHERE key = 'journal_head'"
        )
        .is_empty()
    );
    for table in [
        "partners",
        "items",
        "packings",
        "staff",
        "cities",
        "bikri_entries",
        "journal_entries",
        "drafts",
    ] {
        let found = text_at(&clean_dir, &format!("SELECT COUNT(*) FROM {table}"));
        assert_eq!(found, ["0"], "{table} is empty");
    }
    assert_eq!(
        text_at(
            &clean_dir,
            "SELECT COUNT(*) FROM accounts WHERE is_system = 0"
        ),
        ["0"]
    );
    assert_eq!(
        text_at(
            &clean_dir,
            "SELECT COALESCE(closed_up_to_date, 'none') FROM day_state"
        ),
        ["none"],
        "no day is closed"
    );
    assert_chain_ok(&clean_dir);

    // The Owner signs in with the password he had, in the new file.
    let clean_env = TestEnv::new(clean_dir.path()).unwrap();
    auth::login(&clean_env.ctx, "owner", &Secret::new(OWNER_PASSWORD)).unwrap();
    let owner = auth::require(&clean_env.ctx).unwrap();
    // The first seller, buyer, item, packing and staff get the first codes (plan 32 R2).
    let seller = accounts::save(
        &clean_env.ctx,
        &owner,
        &account(AccountType::Seller, "رحیم", "Rahim"),
    )
    .unwrap();
    assert_eq!(seller.code, "0010");
    let buyer = accounts::save(
        &clean_env.ctx,
        &owner,
        &account(AccountType::Buyer, "اکرم", "Akram"),
    )
    .unwrap();
    assert_eq!(buyer.code, "0020");
    assert_eq!(
        masters::next_code(&clean_env.ctx, MasterKind::Item).unwrap(),
        "0040"
    );
    // A wrong password is still refused, and the audit chain still verifies after the saves.
    assert!(auth::login(&clean_env.ctx, "owner", &Secret::new("not-the-password")).is_err());
    assert_chain_ok(&clean_dir);
    drop(live_env);
}

#[test]
fn a_live_file_with_partners_is_not_cleaned_by_mistake_and_an_existing_file_is_not_replaced() {
    let (live_dir, live_env, _recovery) = live_world();
    let owner = auth::require(&live_env.ctx).unwrap();
    partners::create(
        &live_env.ctx,
        &owner,
        &PartnerCreateInput {
            name_ur: "رحیم".into(),
            name_en: "Rahim".into(),
            opening_capital: None,
            confirmed_rules_words: vec!["رحیم".into(), "rahim".into()],
        },
    )
    .unwrap();
    let live = live_file(&live_dir);
    let clean_dir = tempfile::tempdir().unwrap();
    let new_path = live_file(&clean_dir);

    // Nothing is made, and nothing is left behind, until the partners are given up on purpose.
    let error = make_clean_file(&live, &new_path, options(clean_dir.path()), false).unwrap_err();
    assert!(error.to_string().contains("partner"), "{error}");
    assert!(!new_path.exists(), "a refused run left a file");
    let report = make_clean_file(&live, &new_path, options(clean_dir.path()), true).unwrap();
    assert_eq!(report.partners, 1);
    assert_eq!(text_at(&clean_dir, "SELECT COUNT(*) FROM partners"), ["0"]);

    // A new file that already exists is never replaced, and the live file is never the target.
    let error = make_clean_file(&live, &new_path, options(clean_dir.path()), true).unwrap_err();
    assert!(error.to_string().contains("already exists"), "{error}");
    let error = make_clean_file(&live, &live, options(live_dir.path()), true).unwrap_err();
    assert!(!error.to_string().is_empty());
    assert!(live.exists());
    // A missing live file is a clear message.
    let none = clean_dir.path().join("nothing.db");
    let error = make_clean_file(
        &none,
        &clean_dir.path().join("other").join("mandi.db"),
        options(clean_dir.path()),
        false,
    )
    .unwrap_err();
    assert!(error.to_string().contains("does not exist"), "{error}");
    let _ = ErrorCode::ErrInternal;
}
