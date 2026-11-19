//! A note or a reason made of zero-width characters and direction marks only is empty (gate
//! finding Q-T4-3, plan 80, Code Standards 4.2). Names already lose such marks; free text now does
//! too, so a pasted zero-width space is not a saved note nobody can see and not a reason.
#![cfg(test)]

mod common;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use mandi_services::core::error::ErrorCode;
use mandi_services::core::trips::{TripEntryCancelInput, TripInput};
use mandi_services::core::ventures::{MoneySource, VenturePartnerInput};
use mandi_services::{ServiceError, trips};

/// Zero-width space, joiner, word joiner, BOM, left-to-right mark, right-to-left mark, Arabic
/// letter mark, with spaces round them.
const ONLY_MARKS: &str = " \u{200B}\u{200D}\u{2060}\u{FEFF}\u{200E} \u{200F}\u{061C} ";

fn first(error: &ServiceError) -> (ErrorCode, Option<String>) {
    let ServiceError::Rejected(items) = error else {
        panic!("not a refusal of the user's entry: {error:?}");
    };
    let item = items.first().expect("a refusal has an error");
    (item.code, item.field.clone())
}

/// A reason of only invisible marks is a missing reason, whatever entry it is given for.
#[test]
fn a_reason_of_only_invisible_marks_is_a_missing_reason() {
    let w = support::world();
    let error = trips::cost_cancel(
        &w.env.ctx,
        &w.owner,
        &TripEntryCancelInput {
            id: 1,
            version: 1,
            reason: ONLY_MARKS.into(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap_err();
    assert_eq!(
        first(&error),
        (ErrorCode::ErrReasonRequired, Some("reason".to_owned()))
    );
}

/// A note of only invisible marks is saved as no note, and a note with marks in it is saved
/// without them.
#[test]
fn a_note_loses_its_invisible_marks_and_an_invisible_only_note_is_no_note() {
    let w = support::world();
    let [a, ..] = w.partners;
    let make = |note: &str| {
        trips::create(
            &w.env.ctx,
            &w.owner,
            &TripInput {
                date: "10-10-2026".into(),
                market: Some("Lahore".into()),
                sent_person: None,
                note: Some(note.to_owned()),
                partners: vec![VenturePartnerInput {
                    partner_id: a,
                    share: "100".into(),
                    source: MoneySource::Shop,
                }],
            },
        )
        .unwrap()
        .id
    };
    let stored = |id: i64| -> Option<String> {
        common::raw(&w.dir)
            .query_row("SELECT note FROM ventures WHERE id = ?1", [id], |r| {
                r.get(0)
            })
            .unwrap()
    };
    let empty = make(ONLY_MARKS);
    assert!(
        stored(empty).is_none_or(|note| note.is_empty()),
        "{:?}",
        stored(empty)
    );
    let marked = make("\u{200B}نوٹ\u{200F} 12\u{FEFF}");
    assert_eq!(stored(marked).as_deref(), Some("نوٹ 12"));
}
