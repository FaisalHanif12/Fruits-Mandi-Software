//! The master repositories (plan 06): codes are unique, the next code and serial, the account of
//! an expense head, and the fact that nothing here deletes.
#![cfg(test)]

mod common;

use common::{account_id, classify, setup};
use mandi_core::masters::{AccountToMake, ColValue, MasterKind};
use mandi_core::text;
use mandi_db::DbError;
use mandi_db::masters as repo;
use mandi_db::rusqlite;

fn fields(ur: &str, en: &str) -> Vec<(&'static str, ColValue)> {
    vec![
        ("name_ur", ColValue::Text(ur.into())),
        ("name_en", ColValue::Text(en.into())),
        ("search_ur", ColValue::Text(text::search_ur(ur))),
        ("search_en", ColValue::Text(text::search_en(en))),
        ("search_phonetic", ColValue::Text(text::search_phonetic(en))),
    ]
}

const NOW: &str = "2026-10-10T07:30:12.000Z";

#[test]
fn a_code_can_only_be_used_once_in_a_list_but_once_in_each_list() {
    let (_dir, conn) = setup();
    for kind in [
        MasterKind::City,
        MasterKind::Store,
        MasterKind::Packing,
        MasterKind::Adda,
    ] {
        repo::insert(
            &conn,
            kind,
            Some("1"),
            None,
            &fields("لاہور", "Lahore"),
            1,
            NOW,
        )
        .unwrap();
        let again = repo::insert(
            &conn,
            kind,
            Some("1"),
            None,
            &fields("ملتان", "Multan"),
            1,
            NOW,
        );
        assert!(
            matches!(classify(again.unwrap_err()), DbError::Constraint(m) if m.contains("UNIQUE")),
            "{kind:?}"
        );
        assert!(repo::code_taken(&conn, kind, "1", None).unwrap());
        assert!(
            !repo::code_taken(&conn, kind, "1", Some(1)).unwrap(),
            "its own code is free"
        );
        assert!(!repo::code_taken(&conn, kind, "2", None).unwrap());
    }
    // Items and marks are lists of their own.
    conn.execute(
        "INSERT INTO items (code, name_ur, name_en, search_ur, search_en, search_phonetic, item_group) VALUES ('1', 'کیلا', 'Banana', 'کیلا', 'banana', 'bnn', 'fruit')",
        [],
    )
    .unwrap();
    assert!(
        conn.execute(
            "INSERT INTO items (code, name_ur, name_en, search_ur, search_en, search_phonetic, item_group) VALUES ('1', 'سیب', 'Apple', 'سیب', 'apple', 'ppl', 'fruit')",
            [],
        )
        .is_err()
    );
    conn.execute(
        "INSERT INTO markas (code, mark_text) VALUES ('1', 'AR')",
        [],
    )
    .unwrap();
    assert!(
        conn.execute(
            "INSERT INTO markas (code, mark_text) VALUES ('1', 'BR')",
            []
        )
        .is_err()
    );
}

#[test]
fn the_next_code_is_the_largest_number_plus_one_as_wide_as_the_widest_code() {
    let (_dir, conn) = setup();
    let next = |conn: &mandi_db::rusqlite::Connection| {
        repo::next_code(conn, MasterKind::City)
            .unwrap()
            .unwrap_or_else(|| "FULL".to_owned())
    };
    assert_eq!(next(&conn), "1");
    // `+12` is not a code of digits and does not count (it would widen the next code).
    for code in ["07", "9", "abc", "+12"] {
        repo::insert(
            &conn,
            MasterKind::City,
            Some(code),
            None,
            &fields("لاہور", "Lahore"),
            1,
            NOW,
        )
        .unwrap();
    }
    assert_eq!(
        next(&conn),
        "10",
        "9 + 1, as wide as the widest code (07 is 2 wide)"
    );
    repo::insert(
        &conn,
        MasterKind::City,
        Some("0099"),
        None,
        &fields("ملتان", "Multan"),
        1,
        NOW,
    )
    .unwrap();
    assert_eq!(next(&conn), "0100");
    // The biggest code the app accepts (10 digits) does not overflow the suggestion.
    repo::insert(
        &conn,
        MasterKind::City,
        Some("9999999999"),
        None,
        &fields("قصور", "Kasur"),
        1,
        NOW,
    )
    .unwrap();
    // The code after it would have 11 digits, which the app refuses to take: the list is full.
    assert_eq!(next(&conn), "FULL");
    assert_eq!(
        repo::next_code(&conn, MasterKind::Narration)
            .unwrap()
            .as_deref(),
        Some("")
    );
}

#[test]
fn items_packings_and_staff_take_the_type_number_and_the_next_running_number() {
    // Plan 32 R2: the first code is 0040 / 0050 / 0060, the next is the largest running number + 1
    // (so 0049 is followed by 00410, never by a reused code).
    let (_dir, conn) = setup();
    let staff_fields = || {
        let mut f = fields("اسلم", "Aslam");
        f.push(("role", ColValue::Text("munshi".into())));
        f
    };
    for (kind, type_number) in [
        (MasterKind::Item, "004"),
        (MasterKind::Packing, "005"),
        (MasterKind::Staff, "006"),
    ] {
        let next = |conn: &rusqlite::Connection| repo::next_code(conn, kind).unwrap().unwrap();
        assert_eq!(next(&conn), format!("{type_number}0"), "{kind:?} first");
        let insert = |code: &str| {
            let f = match kind {
                MasterKind::Staff => staff_fields(),
                MasterKind::Item => {
                    let mut f = fields("کیلا", "Banana");
                    f.push(("item_group", ColValue::Text("fruit".into())));
                    f
                }
                _ => fields("پیٹی", "Box"),
            };
            repo::insert(&conn, kind, Some(code), None, &f, 1, NOW).unwrap();
        };
        insert(&format!("{type_number}0"));
        assert_eq!(next(&conn), format!("{type_number}1"));
        insert(&format!("{type_number}9"));
        assert_eq!(next(&conn), format!("{type_number}10"), "{kind:?} after 9");
        insert(&format!("{type_number}10"));
        assert_eq!(next(&conn), format!("{type_number}11"));
        // A code of another type in the same table does not count.
        assert!(repo::code_taken(&conn, kind, &format!("{type_number}9"), None).unwrap());
    }
}

#[test]
fn the_account_of_an_expense_head_takes_the_next_normal_serial() {
    let (_dir, conn) = setup();
    let make = |name: &str| {
        repo::create_account(
            &conn,
            &AccountToMake {
                control_code: "0501",
                name_ur: "بجلی".into(),
                name_en: name.into(),
                search_ur: "بجلی".into(),
                search_en: name.to_lowercase(),
                search_phonetic: "bjl".into(),
            },
            1,
            NOW,
        )
        .unwrap()
        .unwrap()
    };
    assert_eq!(make("Electricity").1, "05010001");
    assert_eq!(make("Tea").1, "05010002");
    // The reserved serials (0000 and 9000 to 9999) never count, and the series ends at 8999.
    conn.execute(
        "INSERT INTO accounts (control_id, code, serial, party_type, name_ur, name_en, search_ur, search_en, search_phonetic)
         SELECT id, '05019999', 9999, 'other', 'نظام', 'System', 'نظام', 'system', 'sstm' FROM account_controls WHERE code = '0501'",
        [],
    )
    .unwrap();
    assert_eq!(make("Rent").1, "05010003", "a reserved serial is ignored");
    conn.execute(
        "INSERT INTO accounts (control_id, code, serial, party_type, name_ur, name_en, search_ur, search_en, search_phonetic)
         SELECT id, '05018999', 8999, 'other', 'آخری', 'Last', 'آخری', 'last', 'lst' FROM account_controls WHERE code = '0501'",
        [],
    )
    .unwrap();
    assert_eq!(
        repo::next_serial(&conn, "0501").unwrap(),
        None,
        "the series is full"
    );
    // The bank control already has the seeded default bank 01020001.
    assert_eq!(repo::next_serial(&conn, "0102").unwrap(), Some(2));
    assert_eq!(repo::next_serial(&conn, "0103").unwrap(), Some(1));
    // The account exists with the control's party type and is not a system account.
    let id = account_id(&conn, "05010001");
    let (system, party): (i64, String) = conn
        .query_row(
            "SELECT is_system, party_type FROM accounts WHERE id = ?1",
            [id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!((system, party.as_str()), (0, "other"));
}

#[test]
fn only_the_listed_tables_can_be_asked_about_by_name() {
    let (_dir, conn) = setup();
    assert!(repo::exists(&conn, "cities", 1).is_ok());
    assert!(repo::exists(&conn, "accounts; DROP TABLE accounts", 1).is_err());
    assert!(repo::exists(&conn, "users", 1).is_err());
}

#[test]
fn a_control_that_does_not_exist_is_an_error_and_never_a_wrong_account_id() {
    let (_dir, conn) = setup();
    // An insert into another table first, so a stale `last_insert_rowid` would be non-zero.
    repo::insert(
        &conn,
        MasterKind::City,
        Some("1"),
        None,
        &fields("لاہور", "Lahore"),
        1,
        NOW,
    )
    .unwrap();
    assert!(repo::next_serial(&conn, "7777").is_err());
    let made = repo::create_account(
        &conn,
        &AccountToMake {
            control_code: "7777",
            name_ur: "بجلی".into(),
            name_en: "Electricity".into(),
            search_ur: "بجلی".into(),
            search_en: "electricity".into(),
            search_phonetic: "elctrcty".into(),
        },
        1,
        NOW,
    );
    assert!(made.is_err());
    let accounts: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM accounts WHERE code LIKE '7777%'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(accounts, 0);
}
