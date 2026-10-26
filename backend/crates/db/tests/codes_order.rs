//! Plan 32 R2, R7: the next code of a type and the order "by code" in SQL, on a real file.
#![cfg(test)]

mod common;

use common::setup;
use mandi_core::codes::CodeType;
use mandi_db::codes::{CodeTable, largest_running, next_code, order_by_code};

fn add(conn: &mandi_db::rusqlite::Connection, control: &str, code: &str, serial: i64, party: &str) {
    conn.execute(
        "INSERT INTO accounts (control_id, code, serial, party_type, name_ur, name_en, search_ur,
             search_en, search_phonetic)
         SELECT id, ?2, ?3, ?4, 'n', 'n', 'n', 'n', 'n' FROM account_controls WHERE code = ?1",
        (control, code, serial, party),
    )
    .unwrap();
}

#[test]
fn the_next_code_of_a_type_goes_on_from_the_highest_running_number() {
    let (_dir, conn) = setup();
    // A fresh file has the two پختہ کھاتہ accounts (type 003) and old-style codes only: no seller
    // and no buyer yet.
    assert_eq!(
        largest_running(&conn, CodeTable::Accounts, CodeType::Seller).unwrap(),
        None
    );
    assert_eq!(
        next_code(&conn, CodeTable::Accounts, CodeType::Seller).unwrap(),
        Some((0, "0010".to_owned()))
    );
    assert_eq!(
        largest_running(&conn, CodeTable::Accounts, CodeType::Pukhta).unwrap(),
        Some(1),
        "0030 and 0031 are the type 003"
    );
    add(&conn, "03011", "0019", 9, "seller");
    add(&conn, "03011", "00110", 10, "seller");
    add(&conn, "03011", "0013", 3, "seller");
    // Codes that only look like the scheme are not counted: padded, letters, too long.
    add(&conn, "03011", "00199", 99, "seller"); // running number 99: counted
    add(&conn, "0305", "0011x", 1, "other");
    add(&conn, "0305", "00107", 7, "other"); // padded (0 then 7): not a running number
    assert_eq!(
        largest_running(&conn, CodeTable::Accounts, CodeType::Seller).unwrap(),
        Some(99)
    );
    assert_eq!(
        next_code(&conn, CodeTable::Accounts, CodeType::Seller).unwrap(),
        Some((100, "001100".to_owned()))
    );
    // The last running number: the list is full.
    add(&conn, "0304", "0029999999", 9_999_999, "buyer");
    assert_eq!(
        next_code(&conn, CodeTable::Accounts, CodeType::Buyer).unwrap(),
        None
    );
    // Staff, items and packings have their own tables.
    assert_eq!(
        next_code(&conn, CodeTable::Staff, CodeType::Staff).unwrap(),
        Some((0, "0060".to_owned()))
    );
}

#[test]
fn order_by_code_puts_running_numbers_in_number_order_and_other_codes_after() {
    let (_dir, conn) = setup();
    for (control, code, serial, party) in [
        ("03011", "00110", 10, "seller"),
        ("03011", "0019", 9, "seller"),
        ("03011", "0011", 1, "seller"),
        ("0304", "0020", 0, "buyer"),
        ("0304", "00210", 10, "buyer"),
        ("0304", "0029", 9, "buyer"),
    ] {
        add(&conn, control, code, serial, party);
    }
    let sql = format!(
        "SELECT code FROM accounts WHERE code GLOB '00[12]*' ORDER BY {}",
        order_by_code("code")
    );
    let got: Vec<String> = conn
        .prepare(&sql)
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    assert_eq!(got, ["0011", "0019", "00110", "0020", "0029", "00210"]);
    // Across the whole table: the scheme first by type, then every other code as text.
    let sql = format!(
        "SELECT code FROM accounts ORDER BY {}",
        order_by_code("code")
    );
    let all: Vec<String> = conn
        .prepare(&sql)
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    assert_eq!(
        &all[..8],
        [
            "0011", "0019", "00110", "0020", "0029", "00210", "0030", "0031"
        ]
    );
    assert_eq!(all[8], "01010001");
}
