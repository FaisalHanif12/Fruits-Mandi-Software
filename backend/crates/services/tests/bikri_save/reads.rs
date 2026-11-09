//! The reads of saved بکری (PRD FR-12.7, FR-12.13, FR-5.4, FR-7.29, FR-13.6, plan 13): a saved
//! بکری with its stored values, the list, the find, first / previous / next / last, global
//! search. (The چکتا buyers and the register of چکتا کھاتہ were removed in plan 64.)

use mandi_services::bikri;
use mandi_services::core::accounts::{GlobalHit, SearchGlobalInput};
use mandi_services::core::bikri::{
    BikriFindInput, BikriListInput, BikriMatch, BikriNavigateInput, BikriStatus, NavigateTo,
};
use mandi_services::core::error::ErrorCode;
use mandi_services::search;

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
fn the_list_and_find_match_each_filter() {
    let w = world();
    save_ok(&w, &event1_truck(&w, "LES-1234"));
    let mut line = sale("s-1", w.akram, "10", "1200");
    line.payment_type = None;
    let mut l = lot(&w, "lot-1", w.karim, w.apple, "10");
    l.cash_paid = None;
    l.sales = vec![line];
    let mut d = draft(vec![l]);
    d.header.truck_no = Some("les-9999".into());
    save_ok(&w, &d);

    let rows = list(&w, BikriListInput::default());
    let second = rows.iter().find(|r| r.number == 2).unwrap();
    assert_eq!(second.sellers.len(), 1);

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
