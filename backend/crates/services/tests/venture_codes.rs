//! Plan 34: easy codes for market trips (type 007) and gardens (type 008), backend part.
//!
//! The first trip is 0070, then 0071 ...; the first garden is 0080, then 0081 ..., and the garden's
//! account under control 0306 has the same code as its garden. The codes show on every wire type
//! that had a trip or garden number, in the trip's name, on the partners' statement lines, and the
//! pickers find a trip or a garden by its short code. No amount is touched (E2E-34, 40, 41 are in
//! `trips.rs` and `golden_day`).
#![cfg(test)]

mod common;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use common::set_today;
use mandi_services::core::accounts::AccountType;
use mandi_services::core::gardens::{GardenGetInput, GardenInput};
use mandi_services::core::purchases::{PurchaseInput, PurchasesListInput};
use mandi_services::core::search::SearchScope;
use mandi_services::core::statement::{PartnersStatementInput, StatementHeading};
use mandi_services::core::trips::{TripInput, TripRow};
use mandi_services::core::ventures::{MoneySource, VenturePartnerInput};
use mandi_services::{accounts, gardens, partners, purchases, search, trips};
use support::{World, world};

fn partners_of(w: &World, source: MoneySource) -> Vec<VenturePartnerInput> {
    let [a, b, c] = w.partners;
    [(a, "40"), (b, "30"), (c, "30")]
        .into_iter()
        .map(|(partner_id, share)| VenturePartnerInput {
            partner_id,
            share: share.into(),
            source,
        })
        .collect()
}

fn trip(w: &World, date: &str) -> TripRow {
    trips::create(
        &w.env.ctx,
        &w.owner,
        &TripInput {
            date: date.into(),
            market: Some("Lahore".into()),
            sent_person: None,
            note: None,
            partners: partners_of(w, MoneySource::Shop),
        },
    )
    .unwrap()
}

fn garden(w: &World, name_en: &str, name_ur: &str) -> mandi_services::core::gardens::GardenRow {
    gardens::create(
        &w.env.ctx,
        &w.owner,
        &GardenInput {
            name_ur: name_ur.into(),
            name_en: name_en.into(),
            confirmed_rules_words: vec![
                "شاہ".into(),
                "پور".into(),
                "باغ".into(),
                "ملتان".into(),
                "Shahpur".into(),
                "Garden".into(),
                "Multan".into(),
            ],
            date: "10-10-2026".into(),
            item_id: Some(w.banana),
            rent: "50,000".into(),
            season_end: None,
            note: None,
            partners: partners_of(w, MoneySource::Shop),
            paid_now: Some("10,000".into()),
            paid_now_reason: Some("first installment".into()),
            parts: Vec::new(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap()
    .row
}

fn hits(w: &World, scope: SearchScope, text: &str, offset: u32, limit: u32) -> Vec<String> {
    search::search_page(&w.env.ctx, &w.owner, scope, text, offset, limit)
        .unwrap()
        .into_iter()
        .map(|h| h.code)
        .collect()
}

#[test]
fn the_first_trip_is_0070_and_the_eleventh_is_00710_and_the_name_is_the_code_and_the_day() {
    let w = world();
    let rows: Vec<TripRow> = (0..11).map(|_| trip(&w, "10-10-2026")).collect();
    assert_eq!(rows[0].code, "0070");
    assert_eq!(rows[1].code, "0071");
    assert_eq!(rows[9].code, "0079");
    assert_eq!(rows[10].code, "00710");
    assert_eq!(
        rows[10].number, 11,
        "the number in the series is still 1, 2, 3 ..."
    );
    // The name is the code and the day, the same in both scripts (it replaces سفر-1 and TRP-1).
    assert_eq!(rows[0].name_ur, "0070 10-10-2026");
    assert_eq!(rows[0].name_en, "0070 10-10-2026");
    assert_eq!(rows[10].name_en, "00710 10-10-2026");
    // The code is on every pointer to a trip: the purchase row.
    let made = purchases::save(
        &w.env.ctx,
        &w.owner,
        &PurchaseInput {
            other_qty: None,
            shop_name: None,
            sell_date: None,
            trip_id: Some(rows[1].id),
            date: "10-10-2026".into(),
            supplier: None,
            item_id: Some(w.banana),
            packing_id: Some(w.box_),
            qty: "20".into(),
            weight: None,
            rate: "50".into(),
            freight: None,
            labour: None,
            other_cost: None,
            note: None,
            parts: Vec::new(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    let listed = purchases::list(
        &w.env.ctx,
        &PurchasesListInput {
            from: Some("10-10-2026".into()),
            to: Some("10-10-2026".into()),
            ..PurchasesListInput::default()
        },
    )
    .unwrap();
    let row = listed.rows.iter().find(|r| r.id == made.id).unwrap();
    assert_eq!((row.trip.number, row.trip.code.as_str()), (2, "0071"));
    assert_eq!(row.trip.name_en, "0071 10-10-2026");
}

#[test]
fn the_first_garden_is_0080_and_its_account_has_the_same_code() {
    let w = world();
    let first = garden(&w, "Shahpur Garden", "شاہ پور باغ");
    set_today(&w.env, 2026, 10, 10);
    let second = garden(&w, "Multan Garden", "ملتان باغ");
    assert_eq!((first.number, first.code.as_str()), (1, "0080"));
    assert_eq!((second.number, second.code.as_str()), (2, "0081"));
    // The garden's account (control 0306) reads the same as its garden.
    let account_code = |id: i64| accounts::get(&w.env.ctx, id).unwrap();
    let first_account = account_code(first.account.id);
    let second_account = account_code(second.account.id);
    assert_eq!(first_account.code, "0080");
    assert_eq!(second_account.code, "0081");
    assert_eq!((first_account.serial, second_account.serial), (0, 1));
    assert_eq!(first_account.control.code, "0306");
    assert_eq!(first_account.account_type, AccountType::Other);
    // The account takes no opening balance (by its control, since its code no longer starts with 0306).
    assert!(!first_account.can_take_opening);
    // The garden keeps the name it was typed with.
    assert_eq!(first.name_en, "Shahpur Garden");
    // Reading the garden again gives the same code.
    let read = gardens::get(
        &w.env.ctx,
        &GardenGetInput {
            id: first.id,
            history_offset: 0,
            history_limit: 0,
        },
    )
    .unwrap();
    assert_eq!(read.row.code, "0080");
    // The summary rows and the partners' cards carry the code.
    let summary = gardens::summary(&w.env.ctx).unwrap();
    assert_eq!(
        summary
            .rows
            .iter()
            .map(|r| r.code.as_str())
            .collect::<Vec<_>>(),
        ["0080", "0081"]
    );
    let [a, ..] = w.partners;
    let cards = partners::overview(&w.env.ctx)
        .unwrap()
        .into_iter()
        .find(|row| row.partner_id == a)
        .unwrap()
        .open_ventures;
    assert!(cards.iter().any(|c| c.code == "0080"));
    // The garden still posts as a seller of a lot: its account is offered in the lot seller field.
    let lot_sellers = search::search_accounts(
        &w.env.ctx,
        &w.owner,
        &mandi_services::core::accounts::SearchAccountsInput {
            scope: mandi_services::core::accounts::AccountScope::LotSeller,
            control_id: None,
            text: String::new(),
            limit: 50,
            offset: 0,
        },
    )
    .unwrap();
    let codes: Vec<&str> = lot_sellers.iter().map(|h| h.code.as_str()).collect();
    assert!(
        codes.contains(&"0080") && codes.contains(&"0081"),
        "{codes:?}"
    );
    // And not in the seller field of anything else.
    let sellers = search::search_accounts(
        &w.env.ctx,
        &w.owner,
        &mandi_services::core::accounts::SearchAccountsInput {
            scope: mandi_services::core::accounts::AccountScope::Seller,
            control_id: None,
            text: String::new(),
            limit: 50,
            offset: 0,
        },
    )
    .unwrap();
    assert!(sellers.iter().all(|h| h.code != "0080"));
}

#[test]
fn a_short_code_finds_a_trip_or_a_garden_in_its_own_picker() {
    // Plan 34 R3: `0` is 0070 in a trip field and 0080 in a garden field; an empty text lists the
    // open ones, recent first and then by code, a page at a time.
    let w = world();
    for _ in 0..3 {
        trip(&w, "10-10-2026");
    }
    garden(&w, "Shahpur Garden", "شاہ پور باغ");
    garden(&w, "Multan Garden", "ملتان باغ");
    assert_eq!(
        hits(&w, SearchScope::Trip, "", 0, 50),
        ["0070", "0071", "0072"]
    );
    assert_eq!(hits(&w, SearchScope::Garden, "", 0, 50), ["0080", "0081"]);
    // The short code: the running number alone, never the other kind.
    assert_eq!(
        hits(&w, SearchScope::Trip, "0", 0, 5)
            .first()
            .map(String::as_str),
        Some("0070")
    );
    assert_eq!(
        hits(&w, SearchScope::Trip, "2", 0, 5)
            .first()
            .map(String::as_str),
        Some("0072")
    );
    assert_eq!(
        hits(&w, SearchScope::Garden, "0", 0, 5)
            .first()
            .map(String::as_str),
        Some("0080")
    );
    assert_eq!(
        hits(&w, SearchScope::Garden, "1", 0, 5)
            .first()
            .map(String::as_str),
        Some("0081")
    );
    // The full code and the garden's name also find it; the codes of the other kind do not.
    assert_eq!(
        hits(&w, SearchScope::Garden, "0081", 0, 5)
            .first()
            .map(String::as_str),
        Some("0081")
    );
    assert_eq!(hits(&w, SearchScope::Garden, "multan", 0, 5), ["0081"]);
    assert!(
        hits(&w, SearchScope::Garden, "0070", 0, 5)
            .iter()
            .all(|c| c != "0070")
    );
    // Pages: a page shorter than the limit is the last one.
    assert_eq!(hits(&w, SearchScope::Trip, "", 0, 2), ["0070", "0071"]);
    assert_eq!(hits(&w, SearchScope::Trip, "", 2, 2), ["0072"]);
    // Choosing one puts it first next time.
    let chosen = search::search_page(&w.env.ctx, &w.owner, SearchScope::Trip, "2", 0, 5).unwrap();
    search::touch(&w.env.ctx, &w.owner, SearchScope::Trip, chosen[0].id).unwrap();
    assert_eq!(
        hits(&w, SearchScope::Trip, "", 0, 50),
        ["0072", "0070", "0071"]
    );
    // A trip whose goods are all written off closes by itself and is no longer offered.
    let closing = trips::list(
        &w.env.ctx,
        &mandi_services::core::trips::TripsListInput::default(),
    )
    .unwrap();
    let first_trip = closing.rows.iter().find(|t| t.code == "0070").unwrap().id;
    let purchase = purchases::save(
        &w.env.ctx,
        &w.owner,
        &PurchaseInput {
            other_qty: None,
            shop_name: None,
            sell_date: None,
            trip_id: Some(first_trip),
            date: "10-10-2026".into(),
            supplier: None,
            item_id: Some(w.banana),
            packing_id: Some(w.box_),
            qty: "5".into(),
            weight: None,
            rate: "10".into(),
            freight: None,
            labour: None,
            other_cost: None,
            note: None,
            parts: Vec::new(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    purchases::write_off(
        &w.env.ctx,
        &w.owner,
        &mandi_services::core::purchases::PurchaseWriteOffInput {
            purchase_id: purchase.id,
            qty: "5".into(),
            reason: "spoiled".into(),
        },
    )
    .unwrap();
    assert_eq!(hits(&w, SearchScope::Trip, "", 0, 50), ["0072", "0071"]);
}

#[test]
fn the_partners_statement_lines_of_a_garden_and_a_trip_carry_the_code() {
    // Plan 34 R1, R2: FR-17.10 lines show 0080 and 0070, not باغ-1 and سفر-1.
    let w = world();
    let g = garden(&w, "Shahpur Garden", "شاہ پور باغ");
    let t = trip(&w, "10-10-2026");
    purchases::save(
        &w.env.ctx,
        &w.owner,
        &PurchaseInput {
            other_qty: None,
            shop_name: None,
            sell_date: None,
            trip_id: Some(t.id),
            date: "10-10-2026".into(),
            supplier: None,
            item_id: Some(w.banana),
            packing_id: Some(w.box_),
            qty: "20".into(),
            weight: None,
            rate: "50".into(),
            freight: None,
            labour: None,
            other_cost: None,
            note: None,
            parts: Vec::new(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    let [a, ..] = w.partners;
    let statement = partners::statement(
        &w.env.ctx,
        &PartnersStatementInput {
            group_by_month: false,
            sort: None,
            partner_id: a,
            from: None,
            to: None,
            offset: 0,
            limit: 0,
        },
    )
    .unwrap();
    let code_of = |heading: StatementHeading| -> Vec<(Option<i64>, Option<String>)> {
        statement
            .lines
            .iter()
            .filter(|l| l.heading == heading)
            .map(|l| (l.number, l.venture_code.clone()))
            .collect()
    };
    let garden_lines = code_of(StatementHeading::Garden);
    assert!(!garden_lines.is_empty());
    assert!(
        garden_lines
            .iter()
            .all(|l| *l == (Some(g.number), Some("0080".to_owned()))),
        "{garden_lines:?}"
    );
    let trip_lines = code_of(StatementHeading::Trip);
    assert!(!trip_lines.is_empty());
    assert!(
        trip_lines
            .iter()
            .all(|l| *l == (Some(t.number), Some("0070".to_owned()))),
        "{trip_lines:?}"
    );
    // Every other line has no venture code.
    assert!(
        statement
            .lines
            .iter()
            .filter(|l| !matches!(l.heading, StatementHeading::Garden | StatementHeading::Trip))
            .all(|l| l.venture_code.is_none())
    );
}
