//! Plan 53 T4 (PRD v1.23): the people in every print header are the people typed in Settings, in
//! their order, then the active staff whose job is منشی, by staff id. At most 3 munshis from the
//! staff list and at most 6 people in all. A staff munshi who is also in Settings (the same Urdu
//! name and phone) prints once. A deactivated munshi no longer prints. No money figure changes.
//!
//! The worked example of plan 53 Section 8.1: Settings has Maher Adnan (owner, 0300-1111111). Staff
//! has آصف (منشی, active, 0300-4058860) and اسلم (اگراہی والا, active). Every header shows Maher
//! Adnan, then آصف. Make آصف inactive and add بلال (منشی, 0301-2222222): Maher Adnan, then بلال.
#![cfg(test)]

#[path = "common/mod.rs"]
mod common;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use mandi_reports::print::{BikriPrintDataInput, PrintFrameInput, bikri_print_data, print_frame};
use mandi_services::core::auth::Language;
use mandi_services::core::bikri::BikriSaveInput;
use mandi_services::core::error::{ErrorCode, ErrorValue};
use mandi_services::core::firm::{FirmContact, FirmRole};
use mandi_services::core::masters::{MasterInput, MasterKind, StaffInput};
use mandi_services::core::printing::PrintDoc;
use mandi_services::settings::{self, FirmPrintInfo};
use mandi_services::{ServiceError, bikri, masters};
use support::*;

/// Repeats a save that asks the user to confirm rule-made spellings, confirming what it asks.
fn confirming<T>(mut attempt: impl FnMut(&[String]) -> Result<T, ServiceError>) -> T {
    let mut words: Vec<String> = Vec::new();
    for _ in 0..5 {
        match attempt(&words) {
            Ok(done) => return done,
            Err(ServiceError::Rejected(items))
                if items
                    .iter()
                    .all(|i| i.code == ErrorCode::ErrSpellingUnconfirmed) =>
            {
                for item in items {
                    if let Some(ErrorValue::Text(t)) = item.values.get("words") {
                        words.extend(t.split(' ').map(str::to_owned));
                    }
                }
            }
            Err(other) => panic!("{other:?}"),
        }
    }
    panic!("the spelling question did not end");
}

fn contact(ur: &str, en: &str, role: FirmRole, phone: &str) -> FirmContact {
    FirmContact {
        name_ur: ur.into(),
        name_en: en.into(),
        role,
        phone: phone.into(),
    }
}

fn set_people(w: &World, people: Vec<FirmContact>) {
    let firm = settings::get(&w.env.ctx).unwrap().firm;
    let info = FirmPrintInfo {
        firm_contacts: people,
        trade_mark: "RH".into(),
    };
    settings::set_firm_with_print_info(&w.env.ctx, &w.owner, &firm, &info, &[]).unwrap();
}

/// Adds a staff member and gives his id.
fn staff(w: &World, ur: &str, en: &str, job: &str, phone: &str) -> i64 {
    let row = confirming(|words| {
        masters::save(
            &w.env.ctx,
            &w.owner,
            &MasterInput::Staff(StaffInput {
                id: None,
                version: None,
                name_ur: ur.into(),
                name_en: en.into(),
                code: String::new(),
                job: job.into(),
                phone: phone.into(),
                confirmed_rules_words: words.to_vec(),
            }),
        )
    });
    row.id()
}

fn set_staff_active(w: &World, id: i64, active: bool) {
    masters::set_active(&w.env.ctx, &w.owner, MasterKind::Staff, id, active).unwrap();
}

/// The people the header of an R35 print would show, as (name, phone) in the print language.
fn r35_people(w: &World, language: Language) -> Vec<(String, String)> {
    let frame = print_frame(
        w.env.ctx.db(),
        &PrintFrameInput {
            language,
            title_id: "report-r35-title".into(),
            facts: Vec::new(),
        },
    )
    .unwrap();
    frame
        .header
        .contacts
        .iter()
        .map(|c| (c.name.clone(), c.phone.clone()))
        .collect()
}

/// The people of the seller bill print of a saved بکری.
fn bill_people(w: &World, id: i64, language: Language) -> Vec<(String, String)> {
    let data = bikri_print_data(
        w.env.ctx.db(),
        &BikriPrintDataInput {
            bikri_id: id,
            doc: PrintDoc::SellerBill,
            language,
            only_buyer_id: None,
        },
    )
    .unwrap();
    data.header
        .contacts
        .iter()
        .map(|c| (c.name.clone(), c.phone.clone()))
        .collect()
}

fn pair(name: &str, phone: &str) -> (String, String) {
    (name.to_owned(), phone.to_owned())
}

#[test]
fn the_header_shows_the_settings_people_then_the_active_munshis_and_follows_the_staff_list() {
    let w = world();
    let id = bikri::save(
        &w.env.ctx,
        &w.owner,
        &BikriSaveInput {
            draft: event1(&w),
            confirm_cash_below_zero: true,
        },
    )
    .unwrap()
    .bikri_id;
    set_people(
        &w,
        vec![contact(
            "ماہر عدنان",
            "Maher Adnan",
            FirmRole::Owner,
            "0300-1111111",
        )],
    );
    let asif = staff(&w, "آصف", "Asif", "munshi", "0300-4058860");
    staff(&w, "اسلم", "Aslam", "collector", "");

    // آصف prints, اسلم (a collector) does not. R35 and the seller bill have the same people.
    for language in [Language::En, Language::Ur] {
        let expected = if language == Language::En {
            vec![
                pair("Maher Adnan", "0300-1111111"),
                pair("Asif", "0300-4058860"),
            ]
        } else {
            vec![
                pair("ماہر عدنان", "0300-1111111"),
                pair("آصف", "0300-4058860"),
            ]
        };
        assert_eq!(r35_people(&w, language), expected, "R35 {language:?}");
        assert_eq!(bill_people(&w, id, language), expected, "bill {language:?}");
    }

    // آصف is made inactive and بلال is added: آصف is gone, بلال is in.
    set_staff_active(&w, asif, false);
    staff(&w, "بلال", "Bilal", "munshi", "0301-2222222");
    for language in [Language::En, Language::Ur] {
        let expected = if language == Language::En {
            vec![
                pair("Maher Adnan", "0300-1111111"),
                pair("Bilal", "0301-2222222"),
            ]
        } else {
            vec![
                pair("ماہر عدنان", "0300-1111111"),
                pair("بلال", "0301-2222222"),
            ]
        };
        assert_eq!(r35_people(&w, language), expected, "R35 {language:?}");
        assert_eq!(bill_people(&w, id, language), expected, "bill {language:?}");
    }
    // Made active again, آصف is back, after بلال (by staff id: he was added first).
    set_staff_active(&w, asif, true);
    assert_eq!(
        r35_people(&w, Language::En),
        [
            pair("Maher Adnan", "0300-1111111"),
            pair("Asif", "0300-4058860"),
            pair("Bilal", "0301-2222222"),
        ]
    );
}

#[test]
fn a_munshi_in_both_lists_prints_once_and_the_limits_hold() {
    let w = world();
    set_people(
        &w,
        vec![
            contact("ماہر عدنان", "Maher Adnan", FirmRole::Owner, "0300-1111111"),
            // The same person as a staff munshi: the same Urdu name and phone.
            contact("آصف", "Asif Bhatti", FirmRole::Munshi, "03004058860"),
        ],
    );
    staff(&w, "آصف", "Asif", "munshi", "0300-4058860");
    let names = |w: &World| {
        r35_people(w, Language::En)
            .into_iter()
            .map(|(name, _)| name)
            .collect::<Vec<_>>()
    };
    // Once, with the Settings spelling of his English name.
    assert_eq!(names(&w), ["Maher Adnan", "Asif Bhatti"]);
    // Another munshi with the same name but a different phone is another person.
    staff(&w, "آصف", "Asif Two", "munshi", "0302-0000000");
    assert_eq!(names(&w), ["Maher Adnan", "Asif Bhatti", "Asif Two"]);

    // At most 3 munshis come from the staff list.
    for (ur, en, phone) in [
        ("ایک", "One", "0303-0000001"),
        ("دو", "Two", "0303-0000002"),
        ("تین", "Three", "0303-0000003"),
    ] {
        staff(&w, ur, en, "munshi", phone);
    }
    assert_eq!(
        names(&w),
        ["Maher Adnan", "Asif Bhatti", "Asif Two", "One", "Two"],
        "Asif is the settings one; of the staff the new Asif, One and Two are the first 3 that are not already printed"
    );

    // At most 6 people in all: Settings holds 5, so only one munshi of the staff fits.
    set_people(
        &w,
        (1..=5)
            .map(|n| {
                contact(
                    &format!("مالک {n}"),
                    &format!("Owner {n}"),
                    FirmRole::Owner,
                    "",
                )
            })
            .collect(),
    );
    let all = names(&w);
    assert_eq!(all.len(), 6, "{all:?}");
    assert_eq!(all[5], "Asif");
}
