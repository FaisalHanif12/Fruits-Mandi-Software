//! Digits on prints, plan 63 (D8 Bug 3; PRD FR-7.32, FR-19.6, NFR-URD-14, Tech Stack 4.5, decision
//! D1 of plan 60).
//!
//! Every number on every print, in Urdu and in English, is written with the digits 0 to 9 and
//! grouping: `2026`, `10,000`, `10-10-2026`. There is no setting for Urdu digits. The cause of the
//! bug of 08-11-2026 was a stored Printing setting `urdu_digits: true` on the Owner's computer
//! (ticked once in Settings, kept when the data was copied): the print builders then turned every
//! digit of an Urdu print into ۰ to ۹. The tests below store that old value by hand, in the way an
//! old computer holds it, and prove that no print uses it.
//!
//! The worked numbers are the PRD worked day (E2E-36, E2E-43): اکرم paid 10,000 on 10-10-2026.
#![cfg(test)]

mod common;
mod reports_world;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use mandi_reports::print::{BikriPrintDataInput, PrintFrameInput, bikri_print_data, print_frame};
use mandi_reports::reports::{
    PartyFilter, ReportFilters, ReportId, ReportPrintInput, buyer_bills_print_data, print_data,
};
use mandi_services::core::auth::Language;
use mandi_services::core::printing::PrintDoc;
use mandi_services::printing;
use reports_world::*;
use support::World;

/// The Printing setting as an old computer holds it: the field `urdu_digits` set to true.
const OLD_STORED_SETTING: &str = r#"{"copies":1,"paper_buyer_slips":"a5","paper_pukhta_slip":"a5","paper_receipt_slip":"a5","paper_seller_bill":"a4","printer":"","urdu_digits":true}"#;

/// True for an Arabic-Indic digit (U+0660 to U+0669) or an Eastern Arabic-Indic one (U+06F0 to
/// U+06F9).
fn has_urdu_digit(text: &str) -> bool {
    text.chars()
        .any(|c| ('\u{0660}'..='\u{0669}').contains(&c) || ('\u{06F0}'..='\u{06F9}').contains(&c))
}

/// Stores the old value of the Printing setting the way the Owner's computer held it.
fn store_old_setting(w: &World) {
    common::raw(&w.dir)
        .execute(
            "INSERT INTO settings (key, value_json) VALUES ('printing', ?1)
             ON CONFLICT (key) DO UPDATE SET value_json = excluded.value_json",
            [OLD_STORED_SETTING],
        )
        .unwrap();
}

/// The print data (as JSON text) of the seller bill R1 of بکری 1, the buyer bills R5 of 10-10-2026
/// and the R35 slips of اکرم (buyer) and رحیم (seller), in one language, with a frame of the
/// R5 print, so the footer time is in the text too.
fn prints_of_the_day(w: &World, language: Language) -> Vec<(&'static str, String)> {
    let seller_bill = bikri_print_data(
        w.env.ctx.db(),
        &BikriPrintDataInput {
            bikri_id: 1,
            doc: PrintDoc::SellerBill,
            language,
            only_buyer_id: None,
        },
    )
    .unwrap();
    let day = ReportFilters {
        from: "10-10-2026".into(),
        to: "10-10-2026".into(),
        ..ReportFilters::default()
    };
    let buyer_bills = buyer_bills_print_data(
        w.env.ctx.db(),
        &ReportPrintInput {
            group_by_month: false,
            sort: None,
            id: ReportId::R5,
            filters: day,
            language,
            paper: None,
        },
    )
    .unwrap();
    let slip = |account: i64| {
        print_data(
            w.env.ctx.db(),
            &ReportPrintInput {
                group_by_month: false,
                sort: None,
                id: ReportId::R35,
                filters: ReportFilters {
                    as_of: "10-10-2026".into(),
                    party: Some(PartyFilter {
                        account_id: account,
                        person_key: None,
                    }),
                    ..ReportFilters::default()
                },
                language,
                paper: None,
            },
        )
        .unwrap()
    };
    let frame = print_frame(
        w.env.ctx.db(),
        &PrintFrameInput {
            language,
            title_id: "report-r5-title".into(),
            facts: Vec::new(),
        },
    )
    .unwrap();
    vec![
        ("R1 seller bill", json(&seller_bill)),
        ("R5 buyer bills", json(&buyer_bills)),
        ("R35 buyer slip", json(&slip(w.akram))),
        ("R35 seller slip", json(&slip(w.rahim))),
        ("frame", json(&frame)),
    ]
}

fn json<T: serde::Serialize>(value: &T) -> String {
    serde_json::to_string(value).unwrap()
}

#[test]
fn urdu_prints_of_r1_r5_and_r35_hold_no_urdu_digits_even_when_an_old_setting_asks_for_them() {
    let w = full_day();
    store_old_setting(&w);
    let mut all_text = String::new();
    for language in [Language::Ur, Language::En] {
        for (name, text) in prints_of_the_day(&w, language) {
            assert!(
                !has_urdu_digit(&text),
                "{name} in {language:?} holds an Urdu digit: {text}"
            );
            all_text.push_str(&text);
            if name != "frame" {
                // Every print carries its dates, written with the year in full.
                assert!(
                    text.contains("2026"),
                    "{name} in {language:?} has no 2026: {text}"
                );
            }
        }
    }
    // The 10,000 of اکرم's receipt, with its grouping comma, is on the slips.
    assert!(all_text.contains("10,000"), "no 10,000 in the prints");
}

#[test]
fn the_urdu_prints_are_the_same_with_the_old_setting_stored_or_never_stored() {
    let w = full_day();
    let without = prints_of_the_day(&w, Language::Ur);
    store_old_setting(&w);
    let with = prints_of_the_day(&w, Language::Ur);
    assert_eq!(with, without);
}

#[test]
fn the_printing_settings_have_no_urdu_digits_field_and_an_old_stored_value_never_stops_a_read() {
    let w = full_day();
    store_old_setting(&w);
    // The stored value still carries the old field; the settings read back without it.
    let read = printing::get_settings(&w.env.ctx, &w.owner).unwrap();
    let value = serde_json::to_value(&read).unwrap();
    assert!(
        value.get("urdu_digits").is_none(),
        "the settings still carry urdu_digits: {value}"
    );
    // The paper, printer and copies of the old value are kept.
    assert_eq!(read.copies, 1);
    assert_eq!(read.printer, "");
}
