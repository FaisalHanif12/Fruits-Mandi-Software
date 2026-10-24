//! Refusals of the garden section that need a world other than the golden day's (plan 17c,
//! D4R-A6): a partner of an open garden cannot be made inactive (PRD A63, E69), and the closing of a
//! garden and the cancel of its entries cannot be undone once their day is closed (FR-12.38, A62).
//! The numbers of the garden itself are tested in `golden_day/gardens.rs`.
#![cfg(test)]

mod common;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use common::set_today;
use mandi_services::core::error::ErrorCode;
use mandi_services::core::gardens::{
    GardenCloseInput, GardenEntryCancelInput, GardenEntryInput, GardenGetInput, GardenInput,
    GardenPaymentInput, GardenReopenInput,
};
use mandi_services::core::partners::PartnerUpdateInput;
use mandi_services::core::ventures::{MoneySource, VenturePartnerInput, VentureStatus};
use mandi_services::core::vouchers::{DayCloseInput, DayReopenInput};
use mandi_services::{ServiceError, day, gardens, partners};
use support::{World, world, world_with};

fn partner(partner_id: i64, share: &str) -> VenturePartnerInput {
    VenturePartnerInput {
        partner_id,
        share: share.into(),
        source: MoneySource::Shop,
    }
}

fn garden_input(w: &World, partners: Vec<VenturePartnerInput>) -> GardenInput {
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

fn first_error(error: &ServiceError) -> (ErrorCode, Option<String>) {
    match error {
        ServiceError::Rejected(items) => {
            let first = items.first().unwrap();
            (first.code, first.field.clone())
        }
        other => panic!("not a rejection: {other:?}"),
    }
}

fn version_of(w: &World, garden: i64) -> i64 {
    gardens::get(
        &w.env.ctx,
        &GardenGetInput {
            id: garden,
            history_offset: 0,
            history_limit: 0,
        },
    )
    .unwrap()
    .row
    .version
}

#[test]
fn a_partner_of_an_open_garden_cannot_be_made_inactive_until_it_is_closed() {
    // No firm shares in this world, so the "has a share" check does not come first.
    let w = world_with(50_000, false);
    let [a, b, c] = w.partners;
    let garden = gardens::create(
        &w.env.ctx,
        &w.owner,
        &garden_input(&w, vec![partner(a, "50"), partner(b, "50")]),
    )
    .unwrap()
    .row;
    let update = |id: i64, active: bool| {
        let version = partners::get(&w.env.ctx, id).unwrap().row.version;
        partners::update(
            &w.env.ctx,
            &w.owner,
            &PartnerUpdateInput {
                id,
                version,
                name_ur: "الف".into(),
                name_en: "Partner A".into(),
                is_active: active,
                confirmed_rules_words: Vec::new(),
            },
        )
    };
    let error = update(a, false).unwrap_err();
    assert_eq!(
        first_error(&error),
        (ErrorCode::ErrPartnerInOpenVenture, Some("is_active".into()))
    );
    // ج is not in the garden.
    update(c, false).unwrap();
    // Once the garden is closed the partner is free (a garden with nothing in it closes at 0).
    set_today(&w.env, 2026, 10, 11);
    gardens::close(
        &w.env.ctx,
        &w.owner,
        &GardenCloseInput {
            id: garden.id,
            version: garden.version,
            date: "11-10-2026".into(),
        },
    )
    .unwrap();
    update(a, false).unwrap();
}

#[test]
fn the_closing_and_the_entries_cannot_be_undone_once_their_day_is_closed() {
    let w = world();
    let [a, b, _] = w.partners;
    let garden = gardens::create(
        &w.env.ctx,
        &w.owner,
        &garden_input(&w, vec![partner(a, "60"), partner(b, "40")]),
    )
    .unwrap()
    .row;
    let paid = gardens::entry_save(
        &w.env.ctx,
        &w.owner,
        &GardenEntryInput::Installment(GardenPaymentInput {
            garden_id: Some(garden.id),
            date: "10-10-2026".into(),
            amount: "10,000".into(),
            reason: "first".into(),
            parts: Vec::new(),
            confirm_cash_below_zero: false,
        }),
    )
    .unwrap();
    set_today(&w.env, 2026, 10, 11);
    gardens::close(
        &w.env.ctx,
        &w.owner,
        &GardenCloseInput {
            id: garden.id,
            version: version_of(&w, garden.id),
            date: "11-10-2026".into(),
        },
    )
    .unwrap();
    // The day of the closing is closed: the closing cannot be undone (FR-12.38).
    set_today(&w.env, 2026, 10, 12);
    day::close(
        &w.env.ctx,
        &w.owner,
        &DayCloseInput {
            up_to: "11-10-2026".into(),
        },
    )
    .unwrap();
    let undo = GardenReopenInput {
        id: garden.id,
        version: version_of(&w, garden.id),
        reason: "wrong".into(),
    };
    let error = gardens::reopen(&w.env.ctx, &w.owner, &undo).unwrap_err();
    assert_eq!(
        first_error(&error),
        (ErrorCode::ErrDateClosed, Some("id".into()))
    );
    // The Owner opens 11-10 again, which leaves 10-10 closed: the undo works, but the cancel of the
    // installment of 10-10 is refused at its date.
    day::reopen(
        &w.env.ctx,
        &w.owner,
        &DayReopenInput {
            date: "11-10-2026".into(),
            reason: "check".into(),
        },
    )
    .unwrap();
    let reopened = gardens::reopen(
        &w.env.ctx,
        &w.owner,
        &GardenReopenInput {
            id: garden.id,
            version: version_of(&w, garden.id),
            reason: "wrong".into(),
        },
    )
    .unwrap();
    assert_eq!(reopened.row.status, VentureStatus::Open);
    let error = gardens::entry_cancel(
        &w.env.ctx,
        &w.owner,
        &GardenEntryCancelInput {
            id: paid.id,
            version: paid.version,
            reason: "wrong".into(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap_err();
    assert_eq!(
        first_error(&error),
        (ErrorCode::ErrDateClosed, Some("id".into()))
    );
}
