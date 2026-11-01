//! Migration 12 (plan 44, PRD FR-7.70; Architecture 15.3): the edit window. Outside a window the trip
//! and garden entries stay read-only exactly as before; inside the window of ONE entry that entry
//! (and only that one) can be changed. The table is empty when no edit is running.
#![cfg(test)]

mod common;

use mandi_db::rusqlite::Connection;
use tempfile::tempdir;

fn count(conn: &Connection, sql: &str) -> i64 {
    conn.query_row(sql, [], |r| r.get(0)).unwrap()
}

fn open_window(conn: &Connection, entity: &str, id: i64) {
    conn.execute(
        "INSERT INTO edit_windows (entity, entity_id) VALUES (?1, ?2)",
        (entity, id),
    )
    .unwrap();
}

fn close_window(conn: &Connection, entity: &str, id: i64) {
    conn.execute(
        "DELETE FROM edit_windows WHERE entity = ?1 AND entity_id = ?2",
        (entity, id),
    )
    .unwrap();
}

/// A trip with two purchases (10,400 and 5,000), a purchase entry with two parts, a trip cost spread
/// over the first purchase, and a garden with one arrival.
struct Rows {
    purchase: i64,
    other_purchase: i64,
    entry: i64,
    cost_entry: i64,
    arrival: i64,
    garden: i64,
}

fn rows(conn: &Connection) -> Rows {
    conn.execute(
        "INSERT INTO packings (code, name_ur, name_en, search_ur, search_en, search_phonetic)
         VALUES ('1', 'پیٹی', 'Box', 'p', 'box', 'bx')",
        [],
    )
    .unwrap();
    let packing = conn.last_insert_rowid();
    conn.execute(
        "INSERT INTO items (code, name_ur, name_en, search_ur, search_en, search_phonetic, item_group)
         VALUES ('1', 'مالٹا', 'Orange', 'm', 'orange', 'rng', 'fruit')",
        [],
    )
    .unwrap();
    let item = conn.last_insert_rowid();
    let accounts: Vec<i64> = conn
        .prepare("SELECT id FROM accounts ORDER BY id LIMIT 3")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    let mut partners = Vec::new();
    for (ur, en) in [("الف", "A"), ("ب", "B")] {
        conn.execute(
            "INSERT INTO partners (name_ur, name_en, capital_account_id, drawings_account_id)
             VALUES (?1, ?2, ?3, ?4)",
            (ur, en, accounts[0], accounts[1]),
        )
        .unwrap();
        partners.push(conn.last_insert_rowid());
    }
    conn.execute(
        "INSERT INTO ventures (kind, number, entry_date, name_ur, name_en, search_ur, search_en, search_phonetic)
         VALUES ('trip', 1, '2026-10-10', 't', 't', 't', 't', 't')",
        [],
    )
    .unwrap();
    let trip = conn.last_insert_rowid();
    let purchase = |number: i64, qty: i64| {
        let value = qty * 50_000;
        conn.execute(
            "INSERT INTO purchases (number, venture_id, entry_date, item_id, packing_id, qty, rate_paisa,
                 value_paisa, freight_paisa, paid_now_paisa, total_cost_paisa)
             VALUES (?1, ?2, '2026-10-10', ?3, ?4, ?5, 50_000, ?6, 0, ?6, ?6)",
            (number, trip, item, packing, qty, value),
        )
        .unwrap();
        conn.last_insert_rowid()
    };
    let first = purchase(1, 20);
    let second = purchase(2, 10);
    conn.execute(
        "INSERT INTO venture_entries (venture_id, kind, number, entry_date, amount_paisa, purchase_id)
         VALUES (?1, 'purchase', 1, '2026-10-10', 1_000_000, ?2)",
        (trip, first),
    )
    .unwrap();
    let entry = conn.last_insert_rowid();
    for (partner, share, amount) in [(partners[0], 6_000, 600_000), (partners[1], 4_000, 400_000)] {
        conn.execute(
            "INSERT INTO venture_entry_parts (entry_id, partner_id, share_bps, source, amount_paisa)
             VALUES (?1, ?2, ?3, 'shop', ?4)",
            (entry, partner, share, amount),
        )
        .unwrap();
    }
    conn.execute(
        "INSERT INTO venture_entries (venture_id, kind, number, entry_date, amount_paisa, reason)
         VALUES (?1, 'cost', 1, '2026-10-11', 40_000, 'truck')",
        [trip],
    )
    .unwrap();
    let cost_entry = conn.last_insert_rowid();
    conn.execute(
        "INSERT INTO purchase_cost_parts (entry_id, purchase_id, amount_paisa) VALUES (?1, ?2, 40_000)",
        (cost_entry, first),
    )
    .unwrap();
    conn.execute(
        "INSERT INTO ventures (kind, number, entry_date, name_ur, name_en, search_ur, search_en,
             search_phonetic, account_id, rent_paisa)
         VALUES ('garden', 1, '2026-10-10', 'g', 'g', 'g', 'g', 'g', ?1, 5_000_000)",
        [accounts[2]],
    )
    .unwrap();
    let garden = conn.last_insert_rowid();
    conn.execute(
        "INSERT INTO garden_arrivals (number, venture_id, entry_date, sell_date, item_id, packing_id, qty)
         VALUES (1, ?1, '2026-10-10', '2026-10-10', ?2, ?3, 100)",
        (garden, item, packing),
    )
    .unwrap();
    Rows {
        purchase: first,
        other_purchase: second,
        entry,
        cost_entry,
        arrival: conn.last_insert_rowid(),
        garden,
    }
}

#[test]
fn the_table_is_there_and_empty() {
    let dir = tempdir().unwrap();
    drop(common::open(dir.path()));
    let conn = common::raw(dir.path());
    assert_eq!(count(&conn, "SELECT COUNT(*) FROM edit_windows"), 0);
    assert!(
        conn.execute(
            "INSERT INTO edit_windows (entity, entity_id) VALUES ('journal_entry', 1)",
            []
        )
        .is_err(),
        "only the four kinds of entry have a window"
    );
    open_window(&conn, "purchase", 1);
    assert!(
        conn.execute(
            "INSERT INTO edit_windows (entity, entity_id) VALUES ('purchase', 1)",
            []
        )
        .is_err(),
        "one window for an entry"
    );
}

#[test]
fn a_purchase_changes_only_inside_its_own_window() {
    let dir = tempdir().unwrap();
    drop(common::open(dir.path()));
    let conn = common::raw(dir.path());
    let r = rows(&conn);
    let change = |id: i64| {
        conn.execute(
            "UPDATE purchases SET qty = 30, value_paisa = 1_500_000, paid_now_paisa = 1_500_000,
                 total_cost_paisa = 1_500_000 WHERE id = ?1",
            [id],
        )
    };
    let again = |id: i64| {
        conn.execute(
            "UPDATE purchases SET qty = 31, value_paisa = 1_550_000, paid_now_paisa = 1_550_000,
                 total_cost_paisa = 1_550_000 WHERE id = ?1",
            [id],
        )
    };
    assert!(
        change(r.purchase).is_err(),
        "no window: read-only as before"
    );
    open_window(&conn, "purchase", r.other_purchase);
    assert!(
        change(r.purchase).is_err(),
        "the window of another purchase does not open this one"
    );
    close_window(&conn, "purchase", r.other_purchase);
    open_window(&conn, "venture_entry", r.entry);
    assert!(
        change(r.purchase).is_err(),
        "the window of an entry is not the window of a purchase"
    );
    close_window(&conn, "venture_entry", r.entry);
    open_window(&conn, "purchase", r.purchase);
    change(r.purchase).unwrap();
    assert_eq!(
        count(
            &conn,
            &format!("SELECT qty FROM purchases WHERE id = {}", r.purchase)
        ),
        30
    );
    // Even inside the window the identity of the purchase never changes.
    for sql in [
        "UPDATE purchases SET number = 9 WHERE id = ?1",
        "UPDATE purchases SET entry_date = '2026-10-12' WHERE id = ?1",
        "UPDATE purchases SET venture_id = venture_id + 1 WHERE id = ?1",
    ] {
        assert!(conn.execute(sql, [r.purchase]).is_err(), "{sql}");
    }
    close_window(&conn, "purchase", r.purchase);
    assert!(again(r.purchase).is_err(), "closed again");
    assert_eq!(count(&conn, "SELECT COUNT(*) FROM edit_windows"), 0);
}

#[test]
fn a_cancelled_purchase_stays_final_even_inside_a_window() {
    let dir = tempdir().unwrap();
    drop(common::open(dir.path()));
    let conn = common::raw(dir.path());
    let r = rows(&conn);
    conn.execute(
        "UPDATE purchases SET status = 'cancelled', cancel_reason = 'x' WHERE id = ?1",
        [r.other_purchase],
    )
    .unwrap();
    open_window(&conn, "purchase", r.other_purchase);
    assert!(
        conn.execute(
            "UPDATE purchases SET qty = 30, value_paisa = 1_500_000, paid_now_paisa = 1_500_000,
                 total_cost_paisa = 1_500_000 WHERE id = ?1",
            [r.other_purchase],
        )
        .is_err()
    );
}

#[test]
fn an_entry_and_its_parts_change_only_inside_the_entry_window() {
    let dir = tempdir().unwrap();
    drop(common::open(dir.path()));
    let conn = common::raw(dir.path());
    let r = rows(&conn);
    let change = || {
        conn.execute(
            "UPDATE venture_entries SET amount_paisa = 1_500_000, journal_entry_id = NULL
             WHERE id = ?1",
            [r.entry],
        )
    };
    let again = || {
        conn.execute(
            "UPDATE venture_entries SET amount_paisa = 1_600_000 WHERE id = ?1",
            [r.entry],
        )
    };
    assert!(change().is_err(), "no window");
    open_window(&conn, "venture_entry", r.cost_entry);
    assert!(change().is_err(), "another entry's window");
    assert!(
        conn.execute(
            "DELETE FROM venture_entry_parts WHERE entry_id = ?1",
            [r.entry]
        )
        .is_err(),
        "the parts of an entry need its own window"
    );
    close_window(&conn, "venture_entry", r.cost_entry);

    open_window(&conn, "venture_entry", r.entry);
    change().unwrap();
    // The parts are replaced as a set: delete, then insert. They are never updated in place.
    assert!(
        conn.execute(
            "UPDATE venture_entry_parts SET amount_paisa = 100 WHERE entry_id = ?1",
            [r.entry]
        )
        .is_err(),
        "no update of a part, even inside the window"
    );
    conn.execute(
        "DELETE FROM venture_entry_parts WHERE entry_id = ?1",
        [r.entry],
    )
    .unwrap();
    assert_eq!(
        count(
            &conn,
            &format!(
                "SELECT COUNT(*) FROM venture_entry_parts WHERE entry_id = {}",
                r.entry
            )
        ),
        0
    );
    // The identity of the entry never changes, nor the purchase it rests on.
    for sql in [
        "UPDATE venture_entries SET purchase_id = NULL WHERE id = ?1",
        "UPDATE venture_entries SET arrival_id = 1 WHERE id = ?1",
        "UPDATE venture_entries SET kind = 'other_sale' WHERE id = ?1",
        "UPDATE venture_entries SET number = 9 WHERE id = ?1",
        "UPDATE venture_entries SET entry_date = '2026-10-12' WHERE id = ?1",
        "UPDATE venture_entries SET venture_id = venture_id + 1 WHERE id = ?1",
    ] {
        assert!(conn.execute(sql, [r.entry]).is_err(), "{sql}");
    }
    close_window(&conn, "venture_entry", r.entry);
    assert!(again().is_err(), "closed again");
}

#[test]
fn the_spread_of_a_trip_cost_is_replaced_only_inside_the_cost_window() {
    let dir = tempdir().unwrap();
    drop(common::open(dir.path()));
    let conn = common::raw(dir.path());
    let r = rows(&conn);
    let delete = || {
        conn.execute(
            "DELETE FROM purchase_cost_parts WHERE entry_id = ?1",
            [r.cost_entry],
        )
    };
    assert!(delete().is_err(), "no window");
    open_window(&conn, "venture_entry", r.entry);
    assert!(delete().is_err(), "another entry's window");
    close_window(&conn, "venture_entry", r.entry);
    open_window(&conn, "venture_entry", r.cost_entry);
    assert!(
        conn.execute(
            "UPDATE purchase_cost_parts SET amount_paisa = 100 WHERE entry_id = ?1",
            [r.cost_entry]
        )
        .is_err(),
        "no update in place, even inside the window"
    );
    delete().unwrap();
}

#[test]
fn an_arrival_and_the_rent_of_a_garden_change_only_inside_their_window() {
    let dir = tempdir().unwrap();
    drop(common::open(dir.path()));
    let conn = common::raw(dir.path());
    let r = rows(&conn);
    let arrival = |qty: i64| {
        conn.execute(
            "UPDATE garden_arrivals SET qty = ?2, note = 'x' WHERE id = ?1",
            (r.arrival, qty),
        )
    };
    let rent = |paisa: i64| {
        conn.execute(
            "UPDATE ventures SET rent_paisa = ?2 WHERE id = ?1",
            (r.garden, paisa),
        )
    };
    assert!(
        arrival(80).is_err() && rent(6_000_000).is_err(),
        "no window"
    );
    open_window(&conn, "garden_arrival", r.arrival);
    arrival(80).unwrap();
    assert!(
        rent(6_000_000).is_err(),
        "an arrival window does not open the rent"
    );
    assert!(
        conn.execute(
            "UPDATE garden_arrivals SET number = 9 WHERE id = ?1",
            [r.arrival]
        )
        .is_err(),
        "the number never changes"
    );
    close_window(&conn, "garden_arrival", r.arrival);
    open_window(&conn, "venture", r.garden);
    rent(6_000_000).unwrap();
    assert!(
        conn.execute(
            "UPDATE ventures SET entry_date = '2026-10-12' WHERE id = ?1",
            [r.garden]
        )
        .is_err(),
        "the identity of a venture never changes"
    );
    close_window(&conn, "venture", r.garden);
    // A window of a venture opens the rent of a garden only, never the rent of a trip.
    let trip: i64 = conn
        .query_row("SELECT id FROM ventures WHERE kind = 'trip'", [], |row| {
            row.get(0)
        })
        .unwrap();
    open_window(&conn, "venture", trip);
    assert!(
        conn.execute("UPDATE ventures SET rent_paisa = 100 WHERE id = ?1", [trip])
            .is_err(),
        "a trip has no rent"
    );
    close_window(&conn, "venture", trip);
    assert!(rent(7_000_000).is_err(), "closed again");
    assert!(arrival(70).is_err(), "the arrival window is closed too");
}
