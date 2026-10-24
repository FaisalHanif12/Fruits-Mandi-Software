//! The reads of saved بکری (PRD FR-12.7, FR-12.13, FR-5.4, FR-7.29, FR-13.6, plan 13): a saved
//! بکری with its stored values, the list, the find, first / previous / next / last, global
//! search, the one-time people and the register.

use mandi_services::bikri;
use mandi_services::core::accounts::{GlobalHit, SearchGlobalInput};
use mandi_services::core::bikri::{
    BikriFindInput, BikriListInput, BikriMatch, BikriNavigateInput, BikriStatus, NavigateTo,
    PaidState, PaymentType, PukhtaPeopleInput, PukhtaRegisterInput, PukhtaSide,
};
use mandi_services::core::error::ErrorCode;
use mandi_services::{ledger, search};

use crate::helpers::*;
use crate::support::*;

/// Event 1 with a truck number, so the list and the find have something to tell.
fn event1_truck(w: &World, truck: &str) -> mandi_services::core::bikri::BikriDraft {
    let mut d = event1(w);
    d.header.truck_no = Some(truck.into());
    d
}

fn list(w: &World, input: BikriListInput) -> Vec<mandi_services::core::bikri::BikriListRow> {
    bikri::list(&w.env.ctx, &input).unwrap().rows
}

#[test]
fn a_saved_bikri_that_does_not_exist_is_not_found() {
    let w = world();
    let error = bikri::get(&w.env.ctx, 99).unwrap_err();
    assert!(rejects(&error, ErrorCode::ErrNotFound, "id"), "{error:?}");
}

#[test]
fn the_list_shows_the_columns_of_the_day_list_and_filters() {
    let w = world();
    save_ok(&w, &event1_truck(&w, "LES-1234"));
    let mut other = event1(&w);
    other.header.truck_no = Some("LHR-0077".into());
    other.lots[0].seller_account_id = Some(w.karim);
    other.lots[0].item_id = Some(w.apple);
    save_ok(&w, &other);

    let all = list(
        &w,
        BikriListInput {
            date: Some("10-10-2026".into()),
            ..Default::default()
        },
    );
    assert_eq!(all.len(), 2);
    // Newest first.
    assert_eq!(all.iter().map(|r| r.number).collect::<Vec<_>>(), [2, 1]);
    let first = &all[1];
    assert_eq!(first.truck_no.as_deref(), Some("LES-1234"));
    assert_eq!(
        first.sellers.iter().map(|s| s.id).collect::<Vec<_>>(),
        [w.rahim]
    );
    assert_eq!(
        first.items.iter().map(|s| s.id).collect::<Vec<_>>(),
        [w.banana]
    );
    assert_eq!((first.qty_received, first.qty_sold), (50, 50));
    assert_eq!(
        (
            first.gross.rupees(),
            first.net.rupees(),
            first.buyer_total.rupees()
        ),
        (29_400, 19_316, 31_564)
    );
    assert_eq!(first.status, BikriStatus::Saved);
    assert!(!first.created_at.is_empty());

    // Filters.
    let by_seller = list(
        &w,
        BikriListInput {
            seller_id: Some(w.karim),
            ..Default::default()
        },
    );
    assert_eq!(by_seller.iter().map(|r| r.number).collect::<Vec<_>>(), [2]);
    let by_item = list(
        &w,
        BikriListInput {
            item_id: Some(w.banana),
            ..Default::default()
        },
    );
    assert_eq!(by_item.iter().map(|r| r.number).collect::<Vec<_>>(), [1]);
    assert_eq!(
        list(
            &w,
            BikriListInput {
                date: Some("09-10-2026".into()),
                ..Default::default()
            }
        )
        .len(),
        0
    );
    assert_eq!(
        list(
            &w,
            BikriListInput {
                from: Some("09-10-2026".into()),
                to: Some("10-10-2026".into()),
                ..Default::default()
            }
        )
        .len(),
        2
    );
    assert_eq!(
        list(
            &w,
            BikriListInput {
                status: Some(BikriStatus::Cancelled),
                ..Default::default()
            }
        )
        .len(),
        0
    );
    // A bad date is an error at its field.
    let error = bikri::list(
        &w.env.ctx,
        &BikriListInput {
            date: Some("32-13-2026".into()),
            ..Default::default()
        },
    )
    .unwrap_err();
    assert!(
        rejects(&error, ErrorCode::ErrDateInvalid, "date"),
        "{error:?}"
    );
}

#[test]
fn a_one_time_seller_is_listed_by_name_and_find_matches_each_filter() {
    let w = world();
    save_ok(&w, &event1_truck(&w, "LES-1234"));
    let mut line = sale("s-1", w.akram, "10", "1200");
    line.payment_type = None;
    let mut l = lot(&w, "lot-1", w.pukhta_seller, w.apple, "10");
    l.pukhta = person("ناصر", "Nasir", "0300-1111111");
    l.cash_paid = None;
    l.sales = vec![line];
    let mut d = draft(vec![l]);
    d.header.truck_no = Some("les-9999".into());
    save_ok(&w, &d);

    let rows = list(&w, BikriListInput::default());
    let second = rows.iter().find(|r| r.number == 2).unwrap();
    assert_eq!(second.pukhta_sellers.len(), 1);
    assert_eq!(second.pukhta_sellers[0].name_en, "Nasir");
    assert!(second.sellers.is_empty());

    let find = |input: BikriFindInput| -> Vec<i64> {
        bikri::find(&w.env.ctx, &input)
            .unwrap()
            .iter()
            .map(|r| r.number)
            .collect()
    };
    assert_eq!(
        find(BikriFindInput {
            number: Some(2),
            ..Default::default()
        }),
        [2]
    );
    assert_eq!(
        find(BikriFindInput {
            number: Some(7),
            ..Default::default()
        }),
        Vec::<i64>::new()
    );
    // The start of a truck number, letter case ignored.
    assert_eq!(
        find(BikriFindInput {
            truck_no: Some("LES".into()),
            ..Default::default()
        }),
        [2, 1]
    );
    assert_eq!(
        find(BikriFindInput {
            truck_no: Some("les-12".into()),
            ..Default::default()
        }),
        [1]
    );
    assert_eq!(
        find(BikriFindInput {
            seller_id: Some(w.rahim),
            ..Default::default()
        }),
        [1]
    );
    assert_eq!(
        find(BikriFindInput {
            item_id: Some(w.apple),
            ..Default::default()
        }),
        [2]
    );
    assert_eq!(
        find(BikriFindInput {
            date: Some("10-10-2026".into()),
            ..Default::default()
        }),
        [2, 1]
    );
    // `%` and `_` are plain characters.
    assert_eq!(
        find(BikriFindInput {
            truck_no: Some("%".into()),
            ..Default::default()
        }),
        Vec::<i64>::new()
    );
}

#[test]
fn first_previous_next_and_last_go_by_number() {
    let w = world();
    let go = |current: Option<i64>, to: NavigateTo| {
        bikri::navigate(
            &w.env.ctx,
            &BikriNavigateInput {
                current_id: current,
                to,
            },
        )
        .unwrap()
    };
    // An empty book has none.
    assert_eq!(go(None, NavigateTo::Last), None);
    assert_eq!(go(None, NavigateTo::First), None);
    for _ in 0..3 {
        save_ok(&w, &event1(&w));
    }
    assert_eq!(go(None, NavigateTo::First), Some(1));
    assert_eq!(go(None, NavigateTo::Last), Some(3));
    // Previous of a new, unsaved بکری is the last; next of it is the first.
    assert_eq!(go(None, NavigateTo::Previous), Some(3));
    assert_eq!(go(None, NavigateTo::Next), Some(1));
    assert_eq!(go(Some(2), NavigateTo::Previous), Some(1));
    assert_eq!(go(Some(2), NavigateTo::Next), Some(3));
    assert_eq!(go(Some(1), NavigateTo::Previous), None);
    assert_eq!(go(Some(3), NavigateTo::Next), None);
    assert_eq!(go(Some(2), NavigateTo::First), Some(1));
    assert_eq!(go(Some(2), NavigateTo::Last), Some(3));
    let error = bikri::navigate(
        &w.env.ctx,
        &BikriNavigateInput {
            current_id: Some(99),
            to: NavigateTo::Next,
        },
    )
    .unwrap_err();
    assert!(
        rejects(&error, ErrorCode::ErrNotFound, "current_id"),
        "{error:?}"
    );
}

#[test]
fn global_search_finds_a_bikri_by_number_and_by_truck() {
    // PRD FR-5.4: after the accounts, the بکری of this number, then the newest of this truck.
    let w = world();
    save_ok(&w, &event1_truck(&w, "LES-1234"));
    save_ok(&w, &event1_truck(&w, "LES-5555"));
    let ask = |text: &str, limit: u32| {
        search::global(
            &w.env.ctx,
            &w.owner,
            &SearchGlobalInput {
                text: text.into(),
                limit,
            },
        )
        .unwrap()
    };
    let bikri_hits = |hits: &[GlobalHit]| -> Vec<(i64, BikriMatch)> {
        hits.iter()
            .filter_map(|h| match h {
                GlobalHit::Bikri { hit } => Some((hit.number, hit.matched)),
                GlobalHit::Account { .. } | GlobalHit::Voucher { .. } => None,
            })
            .collect()
    };
    // The digits 2 are the number of the second بکری.
    let hits = ask("2", 20);
    assert_eq!(bikri_hits(&hits), [(2, BikriMatch::Number)]);
    // The accounts come first.
    let first_bikri = hits
        .iter()
        .position(|h| matches!(h, GlobalHit::Bikri { .. }))
        .unwrap();
    assert!(
        hits[..first_bikri]
            .iter()
            .all(|h| matches!(h, GlobalHit::Account { .. }))
    );
    // A truck: newest first.
    let hits = ask("les", 20);
    assert_eq!(
        bikri_hits(&hits),
        [(2, BikriMatch::Truck), (1, BikriMatch::Truck)]
    );
    // One letter is too short for a truck; nothing for a number that does not exist.
    assert!(bikri_hits(&ask("l", 20)).is_empty());
    assert!(bikri_hits(&ask("77", 20)).is_empty());
    // The whole answer stays within the limit, and the accounts keep a place.
    assert!(ask("les", 1).len() <= 1);
    assert!(ask("2", 2).len() <= 2);
}

// ---- The one-time people and the register ----

fn people(w: &World, text: &str, side: PukhtaSide) -> Vec<String> {
    bikri::search_people(
        &w.env.ctx,
        &PukhtaPeopleInput {
            text: text.into(),
            side,
        },
    )
    .unwrap()
    .iter()
    .map(|p| p.name_en.clone())
    .collect()
}

/// Shahid (cash), Nasir (credit) as one-time buyers and Bashir as a one-time seller, paid in
/// part: his net is left to pay.
fn three_people(w: &World) {
    let mut cash = sale("s-1", w.pukhta_buyer, "10", "1200");
    cash.pukhta = person("شاہد", "Shahid", "0300-0000000");
    let mut credit = sale("s-2", w.pukhta_buyer, "10", "1000");
    credit.pukhta = person("ناصر", "Nasir", "0300-1111111");
    credit.payment_type = Some(PaymentType::Credit);
    save_ok(
        w,
        &draft(vec![with_sales(
            bare_lot(w, "lot-1", w.rahim, w.apple, "20"),
            vec![cash, credit],
        )]),
    );
    // A one-time seller with نقد 2,000 typed: his net 12,000 - 720 - 50 - 50 - 20 - 1,000 - 2,000.
    let mut line = sale("s-3", w.akram, "10", "1200");
    line.payment_type = None;
    let mut l = lot(w, "lot-2", w.pukhta_seller, w.banana, "10");
    l.pukhta = person("بشیر", "Bashir", "0300-2222222");
    l.freight = Some("1,000".into());
    l.cash_paid = Some("2,000".into());
    l.sales = vec![line];
    save_ok(w, &draft(vec![l]));
}

#[test]
fn the_known_one_time_people_are_listed_newest_first_and_found_by_either_name_or_phone() {
    let w = world();
    three_people(&w);
    assert_eq!(people(&w, "", PukhtaSide::Buyer), ["Nasir", "Shahid"]);
    assert_eq!(people(&w, "", PukhtaSide::Seller), ["Bashir"]);
    for typed in [
        "shah",
        "SHAHID",
        "شاہ",
        "شاهد",
        "03000000",
        "0300-0000000",
        "shahd",
    ] {
        assert_eq!(people(&w, typed, PukhtaSide::Buyer), ["Shahid"], "{typed}");
    }
    assert!(people(&w, "bashir", PukhtaSide::Buyer).is_empty());
    assert_eq!(people(&w, "bashir", PukhtaSide::Seller), ["Bashir"]);
    let one = &bikri::search_people(
        &w.env.ctx,
        &PukhtaPeopleInput {
            text: "nasir".into(),
            side: PukhtaSide::Buyer,
        },
    )
    .unwrap()[0];
    assert_eq!(
        (
            one.name_ur.as_str(),
            one.phone.as_str(),
            one.last_date.as_str(),
            one.person_key.as_str()
        ),
        ("ناصر", "03001111111", "2026-10-10", "ناصر|03001111111")
    );
}

#[test]
fn the_register_shows_paid_and_due_with_what_is_open() {
    let w = world();
    three_people(&w);
    let all = bikri::register(&w.env.ctx, &PukhtaRegisterInput::default()).unwrap();
    let find = |name: &str| all.rows.iter().find(|r| r.name_en == name).unwrap();
    // شاہد bought for cash: paid at once.
    let shahid = find("Shahid");
    assert_eq!(
        (
            shahid.side,
            shahid.paid,
            shahid.open.rupees(),
            shahid.amount.rupees()
        ),
        (PukhtaSide::Buyer, true, 0, 12_800)
    );
    // ناصر bought on credit: 10,000 + 600 + 80 = 10,680 due.
    let nasir = find("Nasir");
    assert_eq!(
        (
            nasir.paid,
            nasir.open.rupees(),
            nasir.amount.rupees(),
            nasir.commission.rupees()
        ),
        (false, 10_680, 10_680, 600)
    );
    // بشیر is paid 2,000 on the spot; the rest of his net is still to pay him: 7,... read back.
    let bashir = find("Bashir");
    assert_eq!(bashir.side, PukhtaSide::Seller);
    let lot = &bikri::get(&w.env.ctx, 2).unwrap().lots[0];
    assert_eq!(bashir.open.rupees(), lot.net.rupees());
    assert!(!bashir.paid);
    assert_eq!(bashir.amount.rupees(), 12_000);
    assert_eq!(bashir.commission.rupees(), 720);
    assert_eq!(all.rows.len(), 3);

    // Filters.
    let reg = |input: PukhtaRegisterInput| -> Vec<String> {
        bikri::register(&w.env.ctx, &input)
            .unwrap()
            .rows
            .iter()
            .map(|r| r.name_en.clone())
            .collect()
    };
    assert_eq!(
        reg(PukhtaRegisterInput {
            state: Some(PaidState::Due),
            ..Default::default()
        }),
        ["Nasir", "Bashir"]
    );
    assert_eq!(
        reg(PukhtaRegisterInput {
            state: Some(PaidState::Paid),
            ..Default::default()
        }),
        ["Shahid"]
    );
    assert_eq!(
        reg(PukhtaRegisterInput {
            side: Some(PukhtaSide::Seller),
            ..Default::default()
        }),
        ["Bashir"]
    );
    assert_eq!(
        reg(PukhtaRegisterInput {
            name: Some("ناص".into()),
            ..Default::default()
        }),
        ["Nasir"]
    );
    assert_eq!(
        reg(PukhtaRegisterInput {
            phone: Some("2222".into()),
            ..Default::default()
        }),
        ["Bashir"]
    );
    assert_eq!(
        reg(PukhtaRegisterInput {
            from: "10-10-2026".into(),
            to: "10-10-2026".into(),
            ..Default::default()
        })
        .len(),
        3
    );
    assert!(
        reg(PukhtaRegisterInput {
            from: "09-10-2026".into(),
            to: "09-10-2026".into(),
            ..Default::default()
        })
        .is_empty()
    );
}

#[test]
fn two_lots_of_one_one_time_seller_are_each_paid_or_due_on_their_own() {
    // Money agent L: lot A has a credit of 5,000 and no نقد; lot B a credit of 3,000 with نقد
    // 3,000. A shows 5,000 open and B nothing: B's نقد is not used to pay A.
    let w = world();
    let make = |key: &str, qty: &str, rate: &str, cash: Option<&str>| {
        let mut line = sale(&format!("s-{key}"), w.akram, qty, rate);
        line.payment_type = None;
        let mut l = bare_lot(&w, key, w.pukhta_seller, w.banana, qty);
        l.pukhta = person("بشیر", "Bashir", "0300-2222222");
        l.cash_paid = cash.map(str::to_owned);
        l.sales = vec![line];
        l
    };
    // Lot A: 10 × 540 = 5,400, commission 324: credit 5,076. Lot B: 10 × 340 = 3,400 comm 204:
    // credit 3,196 with نقد 3,196 typed: net 0.
    let a = make("lot-a", "10", "540", Some("0"));
    let b = make("lot-b", "10", "340", Some("3,196"));
    save_ok(&w, &draft(vec![a, b]));
    let rows = bikri::register(&w.env.ctx, &PukhtaRegisterInput::default()).unwrap();
    let seller_rows: Vec<(i64, bool)> = rows
        .rows
        .iter()
        .filter(|r| r.side == PukhtaSide::Seller)
        .map(|r| (r.open.rupees(), r.paid))
        .collect();
    assert_eq!(seller_rows, [(5_076, false), (0, true)]);
}

#[test]
fn a_lot_with_a_negative_net_is_set_against_the_same_persons_other_lots() {
    // Money agent P5: lot A has a net of 5,076; lot B (freight 5,000 against 600 of sales) has a
    // credit of -4,436, so the person owes the firm 4,436. His balance is 640 جمع, and the
    // register shows A open 640 and B paid: the open amounts add up to the balance.
    let w = world();
    let person_lot = |key: &str, qty: &str, rate: &str, freight: Option<&str>| {
        let mut line = sale(&format!("s-{key}"), w.akram, qty, rate);
        line.payment_type = None;
        let mut l = bare_lot(&w, key, w.pukhta_seller, w.banana, qty);
        l.pukhta = person("بشیر", "Bashir", "0300-2222222");
        l.cash_paid = Some("0".into());
        l.freight = freight.map(str::to_owned);
        l.sales = vec![line];
        l
    };
    let a = person_lot("lot-a", "10", "540", None);
    let b = person_lot("lot-b", "1", "600", Some("5,000"));
    save_ok(&w, &draft(vec![a, b]));
    let balance = ledger::person_balance(&w.env.ctx, w.pukhta_seller, "بشیر|03002222222").unwrap();
    assert_eq!(
        (balance.side, balance.amount_paisa.rupees()),
        (mandi_services::core::accounts::BalanceSide::Cr, 640)
    );
    let rows = bikri::register(&w.env.ctx, &PukhtaRegisterInput::default()).unwrap();
    let sellers: Vec<(i64, bool)> = rows
        .rows
        .iter()
        .filter(|r| r.side == PukhtaSide::Seller)
        .map(|r| (r.open.rupees(), r.paid))
        .collect();
    assert_eq!(sellers, [(640, false), (0, true)]);
    assert_eq!(
        sellers.iter().map(|s| s.0).sum::<i64>(),
        balance.amount_paisa.rupees()
    );
}
