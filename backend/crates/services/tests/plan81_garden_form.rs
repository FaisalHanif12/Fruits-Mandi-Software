//! The new garden form of plan 81 (PRD 12.11, FR-12.34, FR-12.35, FR-12.29): the first installment
//! needs no reason typed on the form, and the season end and the note are optional, with a season end
//! that may be in the future.
#![cfg(test)]

mod common;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use mandi_services::core::error::ErrorCode;
use mandi_services::core::gardens::{
    FIRST_INSTALLMENT_REASON_ID, GardenGetInput, GardenHistoryKind, GardenInput,
};
use mandi_services::core::ventures::{MoneySource, VenturePartnerInput};
use mandi_services::{ServiceError, gardens};
use support::{World, world_with};

fn partner(partner_id: i64, share: &str) -> VenturePartnerInput {
    VenturePartnerInput {
        partner_id,
        share: share.into(),
        source: MoneySource::Shop,
    }
}

/// The new garden form with only what it requires: names, date, rent and shares (50 / 50). The
/// season end, the note, the items and the first installment are empty.
fn required_only(w: &World) -> GardenInput {
    let [a, b, _] = w.partners;
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
        item_id: None,
        item_ids: Vec::new(),
        rent: "500,000".into(),
        season_end: None,
        note: None,
        partners: vec![partner(a, "50"), partner(b, "50")],
        paid_now: None,
        paid_now_reason: None,
        parts: Vec::new(),
        confirm_cash_below_zero: false,
    }
}

fn rejected(error: &ServiceError) -> Vec<(ErrorCode, Option<String>)> {
    match error {
        ServiceError::Rejected(items) => items.iter().map(|i| (i.code, i.field.clone())).collect(),
        other => panic!("not a rejection: {other:?}"),
    }
}

#[test]
fn a_garden_is_made_with_only_name_date_rent_and_shares() {
    let w = world_with(500_000, false);
    let made = gardens::create(&w.env.ctx, &w.owner, &required_only(&w)).unwrap();
    assert_eq!(made.row.season_end_date, None);
    assert_eq!(made.row.note, None);
    assert_eq!(made.first_entry_id, None);
}

#[test]
fn a_first_installment_with_no_reason_is_saved_with_the_standard_reason() {
    // Cash 500,000 so the shop's half of 300,000 does not need the cash-below-zero box.
    let w = world_with(500_000, false);
    let mut input = required_only(&w);
    input.paid_now = Some("300,000".into());
    input.paid_now_reason = None;
    let made = gardens::create(&w.env.ctx, &w.owner, &input).unwrap();
    assert!(made.first_entry_id.is_some());

    let detail = gardens::get(
        &w.env.ctx,
        &GardenGetInput {
            id: made.row.id,
            history_offset: 0,
            history_limit: 0,
        },
    )
    .unwrap();
    let installments: Vec<_> = detail
        .history
        .iter()
        .filter(|row| row.kind == GardenHistoryKind::Installment)
        .collect();
    assert_eq!(installments.len(), 1);
    assert_eq!(installments[0].amount_paisa.as_paisa(), 30_000_000);
    assert_eq!(
        installments[0].reason.as_deref(),
        Some(FIRST_INSTALLMENT_REASON_ID)
    );
}

#[test]
fn a_blank_reason_counts_as_none_and_a_typed_reason_is_kept() {
    let w = world_with(500_000, false);
    let mut blank = required_only(&w);
    blank.paid_now = Some("100,000".into());
    blank.paid_now_reason = Some("   ".into());
    let made = gardens::create(&w.env.ctx, &w.owner, &blank).unwrap();
    let reason_of = |id: i64| {
        gardens::get(
            &w.env.ctx,
            &GardenGetInput {
                id,
                history_offset: 0,
                history_limit: 0,
            },
        )
        .unwrap()
        .history
        .into_iter()
        .find(|row| row.kind == GardenHistoryKind::Installment)
        .and_then(|row| row.reason)
    };
    assert_eq!(
        reason_of(made.row.id).as_deref(),
        Some(FIRST_INSTALLMENT_REASON_ID)
    );

    // An older caller that still sends a reason keeps it.
    let mut typed = required_only(&w);
    typed.name_ur = "شاہ پور باغ دو".into();
    typed.name_en = "Shahpur Garden Two".into();
    typed
        .confirmed_rules_words
        .extend(["دو".into(), "Two".into()]);
    typed.paid_now = Some("100,000".into());
    typed.paid_now_reason = Some("پہلی قسط نقد".into());
    let second = gardens::create(&w.env.ctx, &w.owner, &typed).unwrap();
    assert_eq!(reason_of(second.row.id).as_deref(), Some("پہلی قسط نقد"));
}

#[test]
fn a_reason_is_not_needed_for_the_preview_of_a_first_installment_either() {
    let w = world_with(500_000, false);
    let mut input = required_only(&w);
    input.paid_now = Some("300,000".into());
    let preview = gardens::create_preview(&w.env.ctx, &input).unwrap();
    assert!(preview.errors.is_empty(), "{:?}", preview.errors);
    assert!(preview.first_installment.is_some());
}

#[test]
fn a_season_end_after_today_is_accepted() {
    let w = world_with(500_000, false);
    // Today is 10-10-2026 in the test world; the season ends next year.
    let mut input = required_only(&w);
    input.season_end = Some("31-03-2027".into());
    let made = gardens::create(&w.env.ctx, &w.owner, &input).unwrap();
    assert_eq!(made.row.season_end_date.as_deref(), Some("2027-03-31"));
}

#[test]
fn a_season_end_on_the_garden_date_is_accepted_and_one_before_it_is_refused() {
    let w = world_with(500_000, false);
    let mut same_day = required_only(&w);
    same_day.season_end = Some("10-10-2026".into());
    let made = gardens::create(&w.env.ctx, &w.owner, &same_day).unwrap();
    assert_eq!(made.row.season_end_date.as_deref(), Some("2026-10-10"));

    let mut before = required_only(&w);
    before.name_ur = "شاہ پور باغ دو".into();
    before.name_en = "Shahpur Garden Two".into();
    before
        .confirmed_rules_words
        .extend(["دو".into(), "Two".into()]);
    before.season_end = Some("09-10-2026".into());
    let error = gardens::create(&w.env.ctx, &w.owner, &before).unwrap_err();
    assert_eq!(
        rejected(&error),
        vec![(ErrorCode::ErrDateBeforeGarden, Some("season_end".into()))]
    );
}

#[test]
fn an_empty_season_end_and_note_never_stop_the_save() {
    let w = world_with(500_000, false);
    let mut input = required_only(&w);
    input.season_end = Some("   ".into());
    input.note = Some("  ".into());
    let made = gardens::create(&w.env.ctx, &w.owner, &input).unwrap();
    assert_eq!(made.row.season_end_date, None);
    assert_eq!(made.row.note, None);
}
