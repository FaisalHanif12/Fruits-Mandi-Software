//! A garden with several items (plan 78 D2, PRD 12.11 v1.43, Architecture 15.5 v1.28): the items are
//! made with the garden, changed in its edit, listed on its row with the main fruit first, and the
//! garden keeps the first one in `ventures.item_id`. Shares typed as a percent with up to two
//! decimals total exactly 100% (plan 78 D1, PRD FR-12.29).
#![cfg(test)]

mod common;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use mandi_services::core::edit::GardenHeaderEditInput;
use mandi_services::core::error::ErrorCode;
use mandi_services::core::gardens::{GardenGetInput, GardenInput};
use mandi_services::core::ventures::{MoneySource, VenturePartnerInput};
use mandi_services::{ServiceError, edit, gardens};
use support::{World, world_with};

fn partner(partner_id: i64, share: &str) -> VenturePartnerInput {
    VenturePartnerInput {
        partner_id,
        share: share.into(),
        source: MoneySource::Shop,
    }
}

fn garden_input(w: &World, item_ids: Vec<i64>, partners: Vec<VenturePartnerInput>) -> GardenInput {
    GardenInput {
        name_ur: "شاہ پور باغ".into(),
        name_en: "Shahpur Garden".into(),
        confirmed_rules_words: vec![
            "شاہ".into(),
            "پور".into(),
            "باغ".into(),
            "Shahpur".into(),
            "Garden".into(),
        ],
        date: "10-10-2026".into(),
        item_id: Some(w.banana),
        item_ids,
        rent: "50,000".into(),
        season_end: None,
        note: None,
        partners,
        paid_now: None,
        paid_now_reason: None,
        parts: Vec::new(),
        confirm_cash_below_zero: false,
    }
}

fn error_of(error: &ServiceError) -> Vec<(ErrorCode, Option<String>)> {
    match error {
        ServiceError::Rejected(items) => items.iter().map(|i| (i.code, i.field.clone())).collect(),
        other => panic!("not a rejection: {other:?}"),
    }
}

fn row_of(w: &World, id: i64) -> mandi_services::core::gardens::GardenRow {
    gardens::get(
        &w.env.ctx,
        &GardenGetInput {
            id,
            history_offset: 0,
            history_limit: 0,
        },
    )
    .unwrap()
    .row
}

#[test]
fn a_garden_has_one_or_several_items_with_the_first_as_the_main_fruit() {
    let w = world_with(50_000, false);
    let [a, b, _] = w.partners;
    let pair = || vec![partner(a, "50"), partner(b, "50")];

    // Only `item_id`: one item, as before.
    let one = gardens::create(&w.env.ctx, &w.owner, &garden_input(&w, Vec::new(), pair()))
        .unwrap()
        .row;
    let names: Vec<i64> = one.items.iter().map(|i| i.id).collect();
    assert_eq!(names, vec![w.banana]);
    assert_eq!(one.item.as_ref().map(|i| i.id), Some(w.banana));

    // `item_ids` wins over `item_id`: the apple is the main fruit, the banana comes second, and a
    // repeated item counts once.
    let mut input = garden_input(&w, vec![w.apple, w.banana, w.apple], pair());
    input.name_ur = "شاہ پور باغ دو".into();
    input.name_en = "Shahpur Garden Two".into();
    input
        .confirmed_rules_words
        .extend(["دو".into(), "Two".into()]);
    let two = gardens::create(&w.env.ctx, &w.owner, &input).unwrap().row;
    let names: Vec<i64> = two.items.iter().map(|i| i.id).collect();
    assert_eq!(names, vec![w.apple, w.banana]);
    assert_eq!(two.item.as_ref().map(|i| i.id), Some(w.apple));
    assert_eq!(row_of(&w, two.id).items.len(), 2);
}

#[test]
fn an_item_that_does_not_exist_is_refused_on_its_own_place_in_the_list() {
    let w = world_with(50_000, false);
    let [a, b, _] = w.partners;
    let input = garden_input(
        &w,
        vec![w.banana, 9_999_999],
        vec![partner(a, "50"), partner(b, "50")],
    );
    let error = gardens::create(&w.env.ctx, &w.owner, &input).unwrap_err();
    assert_eq!(
        error_of(&error),
        vec![(ErrorCode::ErrNotFound, Some("item_ids[1]".into()))]
    );
}

#[test]
fn the_edit_of_a_garden_changes_its_items_and_the_old_one_is_switched_off() {
    let w = world_with(50_000, false);
    let ctx = &w.env.ctx;
    let [a, b, _] = w.partners;
    let garden = gardens::create(
        ctx,
        &w.owner,
        &garden_input(&w, vec![w.banana], vec![partner(a, "50"), partner(b, "50")]),
    )
    .unwrap()
    .row;
    let edit_with = |items: Vec<i64>, version: i64| GardenHeaderEditInput {
        id: garden.id,
        version,
        garden: garden_input(&w, items, vec![partner(a, "50"), partner(b, "50")]),
        reason: String::new(),
        sign: None,
    };
    // Apple only: the banana row is switched off, not deleted.
    let saved =
        edit::garden_header_save(ctx, &w.owner, &edit_with(vec![w.apple], garden.version)).unwrap();
    let row = row_of(&w, garden.id);
    assert_eq!(
        row.items.iter().map(|i| i.id).collect::<Vec<_>>(),
        vec![w.apple]
    );
    assert_eq!(row.item.as_ref().map(|i| i.id), Some(w.apple));
    let rows: Vec<(i64, i64)> = w
        .env
        .ctx
        .db()
        .read(|conn| {
            let mut s = conn.prepare(
                "SELECT item_id, is_active FROM venture_items WHERE venture_id = ?1 ORDER BY item_id",
            )?;
            let rows = s.query_map([garden.id], |r| Ok((r.get(0)?, r.get(1)?)))?;
            rows.collect::<Result<Vec<_>, _>>()
        })
        .unwrap();
    assert_eq!(rows.len(), 2, "the banana row stays, switched off");
    assert!(rows.contains(&(w.banana, 0)));
    assert!(rows.contains(&(w.apple, 1)));
    // Both again, the banana first: the old row comes back on.
    let _ = saved;
    let version = row_of(&w, garden.id).version;
    edit::garden_header_save(ctx, &w.owner, &edit_with(vec![w.banana, w.apple], version)).unwrap();
    let row = row_of(&w, garden.id);
    assert_eq!(
        row.items.iter().map(|i| i.id).collect::<Vec<_>>(),
        vec![w.banana, w.apple]
    );
}

#[test]
fn shares_with_two_decimals_total_exactly_one_hundred_percent() {
    // Plan 78 D1: a custom share is any percent with up to 2 decimals; 0% is not in the garden; the
    // total must be exactly 100%.
    let w = world_with(50_000, false);
    let [a, b, c] = w.partners;
    let make = |shares: [&str; 3]| {
        gardens::create(
            &w.env.ctx,
            &w.owner,
            &garden_input(
                &w,
                Vec::new(),
                vec![
                    partner(a, shares[0]),
                    partner(b, shares[1]),
                    partner(c, shares[2]),
                ],
            ),
        )
    };
    // 33.33 + 33.33 + 33.34 = 100.
    let row = make(["33.33", "33.33", "33.34"]).unwrap().row;
    let bps: Vec<i64> = row
        .partners
        .iter()
        .map(|p| i64::from(p.share_bps))
        .collect();
    assert_eq!(bps, vec![3333, 3333, 3334]);
    // 70 / 30 / 0: the partner with 0% is not in the garden.
    let mut input = garden_input(
        &w,
        Vec::new(),
        vec![partner(a, "70"), partner(b, "30"), partner(c, "0")],
    );
    input.name_ur = "شاہ پور باغ دو".into();
    input.name_en = "Shahpur Garden Two".into();
    input
        .confirmed_rules_words
        .extend(["دو".into(), "Two".into()]);
    let row = gardens::create(&w.env.ctx, &w.owner, &input).unwrap().row;
    assert_eq!(row.partners.len(), 2);
    assert_eq!(
        row.partners.iter().map(|p| p.share_bps).collect::<Vec<_>>(),
        vec![7000, 3000]
    );
    // 99.99 is refused with the total.
    let error = make(["33.33", "33.33", "33.33"]).unwrap_err();
    assert_eq!(
        error_of(&error),
        vec![(ErrorCode::ErrSharesNot100, Some("partners".into()))]
    );
}
