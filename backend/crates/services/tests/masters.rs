//! The master lists of PRD 10.3 (plan 06, acceptance D2-A5): add, edit, make inactive, with both
//! names, search columns, accounts of expense heads and bank accounts, the spelling check and
//! the audit trail.
#![cfg(test)]

mod common;

use common::*;
use mandi_services::core::error::{ErrorCode, ErrorValue};
use mandi_services::core::masters::{
    AddaInput, BankAccountInput, CityInput, ExpenseHeadInput, ItemInput, MarkaInput, MasterInput,
    MasterKind, MasterRow, NarrationInput, PackingInput, StaffInput, StoreInput,
};
use mandi_services::testing::TestEnv;
use mandi_services::{Rejection, ServiceError, auth, masters};
use tempfile::TempDir;

fn ready() -> (TempDir, TestEnv, auth::Actor) {
    let (dir, env) = env();
    set_up(&env);
    login_owner(&env);
    let owner = auth::require(&env.ctx).unwrap();
    (dir, env, owner)
}

fn city(code: &str, ur: &str, en: &str) -> MasterInput {
    MasterInput::City(CityInput {
        id: None,
        version: None,
        code: code.into(),
        name_ur: ur.into(),
        name_en: en.into(),
        confirmed_rules_words: Vec::new(),
    })
}

fn expect_rejected(error: ServiceError) -> Vec<Rejection> {
    match error {
        ServiceError::Rejected(items) => items,
        other => panic!("not a rejection: {other:?}"),
    }
}

fn first_code(error: ServiceError) -> (ErrorCode, Option<String>) {
    let items = expect_rejected(error);
    (items[0].code, items[0].field.clone())
}

fn count(dir: &TempDir, sql: &str) -> i64 {
    raw(dir).query_row(sql, [], |r| r.get(0)).unwrap()
}

#[test]
fn a_city_is_added_with_both_names_and_the_search_columns_in_one_transaction() {
    // Plan 06 worked example: Kot Radha Kishan.
    let (dir, env, owner) = ready();
    let audit_before = audit_total(&dir);
    let row = masters::save(
        &env.ctx,
        &owner,
        &city("", "کوٹ رادھا کشن", "Kot Radha Kishan"),
    );
    // The words are not all in the word list, so the first try asks for the spelling check.
    let row = match row {
        Ok(row) => row,
        Err(error) => {
            let items = expect_rejected(error);
            assert!(
                items
                    .iter()
                    .all(|i| i.code == ErrorCode::ErrSpellingUnconfirmed)
            );
            assert_eq!(audit_total(&dir), audit_before, "nothing was written");
            assert_eq!(count(&dir, "SELECT COUNT(*) FROM cities"), 0);
            let words: Vec<String> = items
                .iter()
                .flat_map(|i| match i.values.get("words") {
                    Some(ErrorValue::Text(w)) => w.split(' ').map(str::to_owned).collect(),
                    _ => Vec::new(),
                })
                .collect();
            let MasterInput::City(mut input) = city("", "کوٹ رادھا کشن", "Kot Radha Kishan")
            else {
                panic!("not a city")
            };
            input.confirmed_rules_words = words;
            masters::save(&env.ctx, &owner, &MasterInput::City(input)).unwrap()
        }
    };
    let MasterRow::City(saved) = &row else {
        panic!("not a city")
    };
    assert_eq!(
        (
            saved.code.as_str(),
            saved.name_ur.as_str(),
            saved.name_en.as_str(),
            saved.is_active
        ),
        ("1", "کوٹ رادھا کشن", "Kot Radha Kishan", true)
    );
    // Search columns written by core::text in the same save.
    let (ur, en, ph): (String, String, String) = raw(&dir)
        .query_row(
            "SELECT search_ur, search_en, search_phonetic FROM cities",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .unwrap();
    assert_eq!(
        (ur.as_str(), en.as_str(), ph.as_str()),
        ("کوٹ رادھا کشن", "kot radha kishan", "ktrdhkshn")
    );
    assert_eq!(audit_count_for(&dir, "create", "city"), 1);
    assert_chain_ok(&dir);
}

fn audit_count_for(dir: &TempDir, action: &str, entity: &str) -> i64 {
    raw(dir)
        .query_row(
            "SELECT COUNT(*) FROM audit_log WHERE action = ?1 AND entity = ?2",
            [action, entity],
            |r| r.get(0),
        )
        .unwrap()
}

#[test]
fn codes_are_suggested_taken_and_unique() {
    let (dir, env, owner) = ready();
    assert_eq!(masters::next_code(&env.ctx, MasterKind::City).unwrap(), "1");
    masters::save(&env.ctx, &owner, &city("", "لاہور", "Lahore")).unwrap();
    masters::save(&env.ctx, &owner, &city("07", "ملتان", "Multan")).unwrap();
    // The widest code so far sets the width: 07 -> 08.
    assert_eq!(
        masters::next_code(&env.ctx, MasterKind::City).unwrap(),
        "08"
    );
    // A code in use by another record.
    let error = masters::save(&env.ctx, &owner, &city("07", "قصور", "Kasur")).unwrap_err();
    assert_eq!(
        first_code(error),
        (ErrorCode::ErrCodeTaken, Some("code".to_owned()))
    );
    assert_eq!(count(&dir, "SELECT COUNT(*) FROM cities"), 2);
    // Kinds without a code give an empty suggestion.
    for kind in [
        MasterKind::Narration,
        MasterKind::Staff,
        MasterKind::BankAccount,
    ] {
        assert_eq!(masters::next_code(&env.ctx, kind).unwrap(), "");
    }
    // Each list has its own codes.
    assert_eq!(masters::next_code(&env.ctx, MasterKind::Item).unwrap(), "1");
}

#[test]
fn an_edit_checks_the_version_and_keeps_the_code_when_none_is_typed() {
    let (dir, env, owner) = ready();
    let MasterRow::City(first) =
        masters::save(&env.ctx, &owner, &city("5", "لاہور", "Lahore")).unwrap()
    else {
        panic!()
    };
    let edit = |version: i64, code: &str, ur: &str, en: &str| {
        MasterInput::City(CityInput {
            id: Some(first.id),
            version: Some(version),
            code: code.into(),
            name_ur: ur.into(),
            name_en: en.into(),
            confirmed_rules_words: Vec::new(),
        })
    };
    let MasterRow::City(second) = masters::save(
        &env.ctx,
        &owner,
        &edit(first.version, "", "لاہور شہر", "Lahore City"),
    )
    .unwrap() else {
        panic!()
    };
    assert_eq!(
        (second.code.as_str(), second.version),
        ("5", first.version + 1)
    );
    assert_eq!(second.name_en, "Lahore City");
    // The old version is refused, and nothing changed.
    let stale = masters::save(
        &env.ctx,
        &owner,
        &edit(first.version, "5", "لاہور", "Lahore"),
    )
    .unwrap_err();
    assert_eq!(first_code(stale).0, ErrorCode::ErrVersionChanged);
    let name: String = raw(&dir)
        .query_row("SELECT name_en FROM cities", [], |r| r.get(0))
        .unwrap();
    assert_eq!(name, "Lahore City");
    // An edit of a record that is not there.
    let mut ghost = edit(1, "9", "کراچی", "Karachi");
    if let MasterInput::City(c) = &mut ghost {
        c.id = Some(999);
    }
    assert_eq!(
        first_code(masters::save(&env.ctx, &owner, &ghost).unwrap_err()),
        (ErrorCode::ErrNotFound, Some("id".to_owned()))
    );
    // The search columns follow the names; the audit row has the old and the new value.
    let search_en: String = raw(&dir)
        .query_row("SELECT search_en FROM cities", [], |r| r.get(0))
        .unwrap();
    assert_eq!(search_en, "lahore city");
    let (old, new): (String, String) = raw(&dir)
        .query_row(
            "SELECT old_json, new_json FROM audit_log WHERE action = 'edit' AND entity = 'city'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert!(old.contains("Lahore") && new.contains("Lahore City"));
    assert_chain_ok(&dir);
}

#[test]
fn the_rows_every_kind_needs_can_be_added_edited_and_made_inactive() {
    // D2-A5: every PRD 10.3 master, with both names.
    let (dir, env, owner) = ready();
    let save = |input: MasterInput| masters::save(&env.ctx, &owner, &input).unwrap();
    let packing = save(MasterInput::Packing(PackingInput {
        id: None,
        version: None,
        code: "".into(),
        name_ur: "پیٹی".into(),
        name_en: "Box".into(),
        approx_kg: "12.5".into(),
        confirmed_rules_words: Vec::new(),
    }));
    let MasterRow::Packing(packing) = packing else {
        panic!()
    };
    assert_eq!(packing.approx_kg.as_deref(), Some("12.5"));
    let item = save(MasterInput::Item(ItemInput {
        id: None,
        version: None,
        code: "".into(),
        name_ur: "کیلا".into(),
        name_en: "Banana".into(),
        item_group: "fruit".into(),
        default_packing_id: Some(packing.id),
        sold_by: "unit".into(),
        confirmed_rules_words: Vec::new(),
    }));
    let MasterRow::Item(item) = item else {
        panic!()
    };
    assert_eq!(
        (
            item.item_group.as_str(),
            item.default_packing_name_en.as_deref(),
            item.sold_by.as_str()
        ),
        ("fruit", Some("Box"), "unit")
    );
    let radish = save(MasterInput::Item(ItemInput {
        id: None,
        version: None,
        code: "".into(),
        name_ur: "مولی".into(),
        name_en: "Radish".into(),
        item_group: "vegetable".into(),
        default_packing_id: None,
        sold_by: "weight".into(),
        confirmed_rules_words: Vec::new(),
    }));
    let MasterRow::Item(radish) = radish else {
        panic!()
    };
    assert_eq!(
        (radish.code.as_str(), radish.item_group.as_str()),
        ("2", "vegetable")
    );
    let c = save(city("", "لاہور", "Lahore"));
    let MasterRow::City(c) = c else { panic!() };
    let adda = save(MasterInput::Adda(AddaInput {
        id: None,
        version: None,
        code: "".into(),
        name_ur: "سوات اڈا".into(),
        name_en: "Swat Adda".into(),
        city_id: Some(c.id),
        confirmed_rules_words: Vec::new(),
    }));
    let MasterRow::Adda(adda) = adda else {
        panic!()
    };
    assert_eq!(adda.city_name_en.as_deref(), Some("Lahore"));
    save(MasterInput::Store(StoreInput {
        id: None,
        version: None,
        code: "".into(),
        name_ur: "سٹور".into(),
        name_en: "Store".into(),
        confirmed_rules_words: Vec::new(),
    }));
    let marka = save(MasterInput::Marka(MarkaInput {
        id: None,
        version: None,
        code: "".into(),
        mark_text: "AR".into(),
        owner_account_id: None,
    }));
    let MasterRow::Marka(marka) = marka else {
        panic!()
    };
    assert_eq!(marka.mark_text, "AR");
    let narration = save(MasterInput::Narration(NarrationInput {
        id: None,
        version: None,
        text_ur: "بجلی کا بل".into(),
        text_en: "Electricity bill".into(),
        used_in: vec!["payment".into(), "shop_expense".into()],
    }));
    let MasterRow::Narration(narration) = narration else {
        panic!()
    };
    assert_eq!(narration.used_in, ["payment", "shop_expense"]);
    let staff = save(MasterInput::Staff(StaffInput {
        id: None,
        version: None,
        name_ur: "اسلم".into(),
        name_en: "Aslam".into(),
        job: "collector".into(),
        phone: "0300-1234567".into(),
        confirmed_rules_words: Vec::new(),
    }));
    let MasterRow::Staff(staff) = staff else {
        panic!()
    };
    assert_eq!(
        (staff.job.as_str(), staff.phone.as_deref(), staff.account_id),
        ("collector", Some("03001234567"), None)
    );
    // Make each inactive: gone from the list, kept with include_inactive, flagged.
    for kind in MasterKind::ALL {
        let all = masters::list(&env.ctx, kind, true).unwrap();
        let before = masters::list(&env.ctx, kind, false).unwrap();
        if all.is_empty() {
            continue; // expense heads and bank accounts are in the next test
        }
        assert_eq!(all.len(), before.len());
        let target = all[0].id();
        let row = masters::set_active(&env.ctx, &owner, kind, target, false).unwrap();
        assert!(!row.is_active());
        assert_eq!(
            masters::list(&env.ctx, kind, false).unwrap().len(),
            before.len() - 1
        );
        let kept = masters::list(&env.ctx, kind, true).unwrap();
        assert_eq!(kept.len(), all.len(), "{kind:?} is kept, never deleted");
        assert!(kept.iter().any(|r| r.id() == target && !r.is_active()));
        // Back again.
        assert!(
            masters::set_active(&env.ctx, &owner, kind, target, true)
                .unwrap()
                .is_active()
        );
    }
    assert_chain_ok(&dir);
}

#[test]
fn an_expense_head_makes_its_account_under_0501_and_a_bank_account_under_0102() {
    let (dir, env, owner) = ready();
    let head = |code: &str, ur: &str, en: &str| {
        MasterInput::ExpenseHead(ExpenseHeadInput {
            id: None,
            version: None,
            code: code.into(),
            name_ur: ur.into(),
            name_en: en.into(),
            confirmed_rules_words: Vec::new(),
        })
    };
    let MasterRow::ExpenseHead(first) =
        masters::save(&env.ctx, &owner, &head("", "بجلی", "Electricity")).unwrap()
    else {
        panic!()
    };
    let MasterRow::ExpenseHead(second) =
        masters::save(&env.ctx, &owner, &head("", "چائے", "Tea")).unwrap()
    else {
        panic!()
    };
    assert_eq!(
        (first.account_code.as_str(), second.account_code.as_str()),
        ("05010001", "05010002")
    );
    // The account has the same names, its search keys, the party type of the control, is not a
    // system account and is active.
    let (code, ur, en, search_en, party, system, active): (String, String, String, String, String, i64, i64) = raw(&dir)
        .query_row(
            "SELECT code, name_ur, name_en, search_en, party_type, is_system, is_active FROM accounts WHERE id = ?1",
            [first.account_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?, r.get(6)?)),
        )
        .unwrap();
    assert_eq!(
        (
            code.as_str(),
            ur.as_str(),
            en.as_str(),
            search_en.as_str(),
            party.as_str(),
            system,
            active
        ),
        (
            "05010001",
            "بجلی",
            "Electricity",
            "electricity",
            "other",
            0,
            1
        )
    );
    // Renaming the head renames its account; making it inactive does the same to the account.
    let MasterRow::ExpenseHead(renamed) = masters::save(
        &env.ctx,
        &owner,
        &MasterInput::ExpenseHead(ExpenseHeadInput {
            id: Some(first.id),
            version: Some(first.version),
            code: "".into(),
            name_ur: "بجلی کا بل".into(),
            name_en: "Electricity Bill".into(),
            confirmed_rules_words: Vec::new(),
        }),
    )
    .unwrap() else {
        panic!()
    };
    assert_eq!(
        renamed.account_code, "05010001",
        "the account keeps its code"
    );
    let name: String = raw(&dir)
        .query_row(
            "SELECT name_en FROM accounts WHERE id = ?1",
            [first.account_id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(name, "Electricity Bill");
    masters::set_active(&env.ctx, &owner, MasterKind::ExpenseHead, first.id, false).unwrap();
    let active: i64 = raw(&dir)
        .query_row(
            "SELECT is_active FROM accounts WHERE id = ?1",
            [first.account_id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(active, 0);
    // A bank account: the seeded default bank is 01020001, so the next is 01020002.
    let bank = masters::save(
        &env.ctx,
        &owner,
        &MasterInput::BankAccount(BankAccountInput {
            id: None,
            version: None,
            bank_name: "HBL".into(),
            title: "Maher Adnan and Brothers".into(),
            number: "PK36-SCBL-0000001123456702".into(),
            name_ur: "ایچ بی ایل".into(),
            name_en: "HBL".into(),
            confirmed_rules_words: vec!["ایچ".into(), "بی".into(), "ایل".into(), "HBL".into()],
        }),
    )
    .unwrap();
    let MasterRow::BankAccount(bank) = bank else {
        panic!()
    };
    assert_eq!(bank.account_code, "01020002");
    assert_eq!(
        (bank.bank_name.as_str(), bank.number.as_str()),
        ("HBL", "PK36-SCBL-0000001123456702")
    );
    // The accounts and the masters were each audited.
    assert!(audit_count_for(&dir, "create", "account") >= 3);
    assert_chain_ok(&dir);
}

#[test]
fn a_record_it_points_at_must_exist() {
    let (dir, env, owner) = ready();
    let bad_item = MasterInput::Item(ItemInput {
        id: None,
        version: None,
        code: "".into(),
        name_ur: "کیلا".into(),
        name_en: "Banana".into(),
        item_group: "fruit".into(),
        default_packing_id: Some(999),
        sold_by: "unit".into(),
        confirmed_rules_words: Vec::new(),
    });
    assert_eq!(
        first_code(masters::save(&env.ctx, &owner, &bad_item).unwrap_err()),
        (
            ErrorCode::ErrNotFound,
            Some("default_packing_id".to_owned())
        )
    );
    let bad_adda = MasterInput::Adda(AddaInput {
        id: None,
        version: None,
        code: "".into(),
        name_ur: "سوات اڈا".into(),
        name_en: "Swat Adda".into(),
        city_id: Some(999),
        confirmed_rules_words: Vec::new(),
    });
    assert_eq!(
        first_code(masters::save(&env.ctx, &owner, &bad_adda).unwrap_err()).1,
        Some("city_id".to_owned())
    );
    assert_eq!(
        count(&dir, "SELECT COUNT(*) FROM items") + count(&dir, "SELECT COUNT(*) FROM addas"),
        0
    );
}

#[test]
fn a_word_only_the_rules_can_spell_is_confirmed_once_and_then_learned() {
    // PRD FR-7.66, E49, D2-A17 on a master save.
    let (dir, env, owner) = ready();
    let make = |confirmed: &[&str]| {
        MasterInput::City(CityInput {
            id: None,
            version: None,
            code: "".into(),
            name_ur: "علی قزکس".into(),
            name_en: "Ali Qzxwv".into(),
            confirmed_rules_words: confirmed.iter().map(|s| (*s).to_owned()).collect(),
        })
    };
    let error = masters::save(&env.ctx, &owner, &make(&[])).unwrap_err();
    let items = expect_rejected(error);
    assert_eq!(items.len(), 2);
    assert!(
        items
            .iter()
            .all(|i| i.code == ErrorCode::ErrSpellingUnconfirmed)
    );
    assert_eq!(count(&dir, "SELECT COUNT(*) FROM cities"), 0);
    assert_eq!(count(&dir, "SELECT COUNT(*) FROM translit_pairs"), 0);
    masters::save(&env.ctx, &owner, &make(&["قزکس", "Qzxwv"])).unwrap();
    assert_eq!(count(&dir, "SELECT COUNT(*) FROM cities"), 1);
    assert_eq!(
        count(&dir, "SELECT COUNT(*) FROM translit_pairs"),
        1,
        "learned in the same save"
    );
    // The next city with the same word needs no confirmation.
    let again = MasterInput::City(CityInput {
        id: None,
        version: None,
        code: "".into(),
        name_ur: "قزکس پور".into(),
        name_en: "Qzxwv Pur".into(),
        confirmed_rules_words: Vec::new(),
    });
    let result = masters::save(&env.ctx, &owner, &again);
    // `pur` may or may not be listed; the learned word is never asked about again.
    if let Err(error) = result {
        for item in expect_rejected(error) {
            let ErrorValue::Text(words) = &item.values["words"] else {
                panic!()
            };
            assert!(
                !words.contains("Qzxwv") && !words.contains("قزکس"),
                "{words}"
            );
        }
    }
}

#[test]
fn a_save_that_fails_leaves_no_account_no_pair_and_no_row() {
    let (dir, env, owner) = ready();
    masters::save(
        &env.ctx,
        &owner,
        &MasterInput::ExpenseHead(ExpenseHeadInput {
            id: None,
            version: None,
            code: "3".into(),
            name_ur: "بجلی".into(),
            name_en: "Electricity".into(),
            confirmed_rules_words: Vec::new(),
        }),
    )
    .unwrap();
    let accounts = count(&dir, "SELECT COUNT(*) FROM accounts");
    let audit = audit_total(&dir);
    // The second head asks for the same code: refused after the account would have been made.
    let error = masters::save(
        &env.ctx,
        &owner,
        &MasterInput::ExpenseHead(ExpenseHeadInput {
            id: None,
            version: None,
            code: "3".into(),
            name_ur: "چائے".into(),
            name_en: "Tea".into(),
            confirmed_rules_words: Vec::new(),
        }),
    )
    .unwrap_err();
    assert_eq!(first_code(error).0, ErrorCode::ErrCodeTaken);
    assert_eq!(count(&dir, "SELECT COUNT(*) FROM accounts"), accounts);
    assert_eq!(count(&dir, "SELECT COUNT(*) FROM expense_heads"), 1);
    assert_eq!(audit_total(&dir), audit);
}

#[test]
fn setting_the_same_state_writes_nothing_and_a_missing_record_is_not_found() {
    let (dir, env, owner) = ready();
    let MasterRow::City(c) = masters::save(&env.ctx, &owner, &city("", "لاہور", "Lahore")).unwrap()
    else {
        panic!()
    };
    let audit = audit_total(&dir);
    let row = masters::set_active(&env.ctx, &owner, MasterKind::City, c.id, true).unwrap();
    assert_eq!(row.version(), c.version, "no change, no new version");
    assert_eq!(audit_total(&dir), audit);
    let error = masters::set_active(&env.ctx, &owner, MasterKind::City, 999, false).unwrap_err();
    assert_eq!(
        first_code(error),
        (ErrorCode::ErrNotFound, Some("id".to_owned()))
    );
    masters::set_active(&env.ctx, &owner, MasterKind::City, c.id, false).unwrap();
    let (old, new): (String, String) = raw(&dir)
        .query_row(
            "SELECT old_json, new_json FROM audit_log WHERE entity = 'city' ORDER BY id DESC LIMIT 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert!(old.contains("true") && new.contains("false"));
}

#[test]
fn field_errors_come_back_together_and_write_nothing() {
    let (dir, env, owner) = ready();
    let error = masters::save(&env.ctx, &owner, &city("12a", "Lahore", "لاہور")).unwrap_err();
    let items = expect_rejected(error);
    let got: Vec<(String, ErrorCode)> = items
        .into_iter()
        .map(|i| (i.field.unwrap_or_default(), i.code))
        .collect();
    assert_eq!(
        got,
        [
            ("code".to_owned(), ErrorCode::ErrFormatInvalid),
            ("name_ur".to_owned(), ErrorCode::ErrNameUrScript),
            ("name_en".to_owned(), ErrorCode::ErrNameEnScript),
        ]
    );
    assert_eq!(count(&dir, "SELECT COUNT(*) FROM cities"), 0);
}

/// The `DELETE FROM` statements in the source of a crate folder, as (file, table).
fn deletes_in(folder: &str) -> Vec<(String, String)> {
    fn walk(path: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
        if path.is_dir() {
            for entry in std::fs::read_dir(path).unwrap() {
                walk(&entry.unwrap().path(), out);
            }
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path.to_owned());
        }
    }
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(folder);
    let mut files = Vec::new();
    walk(&root, &mut files);
    let mut found = Vec::new();
    for file in files {
        let text = std::fs::read_to_string(&file).unwrap();
        for (at, _) in text.match_indices("DELETE FROM") {
            let table = text[at + "DELETE FROM".len()..]
                .split_whitespace()
                .next()
                .unwrap_or("")
                .trim_matches(|c: char| !c.is_alphanumeric() && c != '_')
                .to_owned();
            found.push((file.display().to_string(), table));
        }
    }
    found
}

#[test]
fn there_is_no_way_to_delete_a_master_record() {
    // End-gate review 19, PRD E21 and Code Standards D3: a master in use is made inactive, never
    // deleted. Nothing in the services, the database layer or the commands deletes from a
    // master table; the only DELETE FROM in the app are a learned word pair (audited) and the
    // user's own unsaved draft of a screen (plan 13: a draft is scratch work, not a record).
    for folder in ["src", "../db/src", "../../app/src"] {
        for (file, table) in deletes_in(folder) {
            assert!(
                table == "translit_pairs" || table == "drafts",
                "{file} deletes from {table}: master records are made inactive, never deleted"
            );
        }
    }
    // The list of commands has no delete or remove, except for learned pairs.
    let build_rs = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../app/build.rs"),
    )
    .unwrap();
    let listed = build_rs.split("APP_COMMANDS").nth(1).unwrap();
    let listed = &listed[listed.find('[').unwrap()..listed.find("];").unwrap()];
    let deleting: Vec<&str> = listed
        .split('"')
        .enumerate()
        .filter(|(i, _)| i % 2 == 1)
        .map(|(_, name)| name)
        .filter(|name| name.contains("delete") || name.contains("remove"))
        .collect();
    assert_eq!(deleting, ["translit_pairs_remove"]);
}

#[test]
fn a_master_in_use_is_made_inactive_and_stays_in_the_table() {
    let (dir, env, owner) = ready();
    let row = masters::save(&env.ctx, &owner, &city("", "لاہور", "Lahore")).unwrap();
    masters::set_active(&env.ctx, &owner, MasterKind::City, row.id(), false).unwrap();
    let (active, rows): (i64, i64) = raw(&dir)
        .query_row(
            "SELECT MAX(is_active), COUNT(*) FROM cities WHERE id = ?1",
            [row.id()],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!((active, rows), (0, 1));
}
