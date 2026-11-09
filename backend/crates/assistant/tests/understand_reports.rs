//! Plan 55 ruling 1 and the lead's order of 05-11-2026: a print or an open plan for every report of
//! the catalogue (PRD 19.2). This file holds the 18 reports that had none (R3, R4, R7, R9, R15, R16,
//! R17, R19, R22, R23, R25, R26, R27, R28, R30, R32, R38) and the title words of R8, R33 and R36.
//!
//! Each case asks the real `understand` of the PRD worked day (today 10-10-2026) in Roman Urdu,
//! English or Urdu, and checks the report, the dates it carries (`from` and `to`, or `as_of`), that the
//! question is sure, and that a local answer with a button and steps is made from it. The filters of
//! every report are the ones the report engine takes (`reports/src/reports/mod.rs`, SPECS), which
//! is also the list frontend2 checks the links against.
//!
//! R1 (one بکری in full) is not planned: it needs the بکری's number looked up in the books, so the
//! model reads it with the `report` tool and its `bikri` filter.
#![cfg(test)]

#[path = "../../services/tests/common/mod.rs"]
mod common;
#[path = "support/worked.rs"]
mod worked;
#[allow(dead_code)]
#[path = "../../services/tests/bikri_preview/support.rs"]
mod world;

use mandi_assistant::answer::AnswerSource;
use mandi_assistant::results::Words;
use mandi_assistant::understand::{Intent, Reader, compose, understand};
use mandi_services::core::auth::Language;
use mandi_services::core::dates::BusinessDate;

const READ_AT: &str = "2026-10-10T08:00:00Z";

/// (question, report, from, to, as_of): the dates the plan carries, empty when it carries none.
#[rustfmt::skip]
const CASES: &[(&str, &str, &str, &str, &str)] = &[
    // R3 cash sales, R4 credit sales: the registers of the sales by payment type.
    ("naqdi bikri ka print is mahine ka", "R3", "2026-10-01", "2026-10-10", ""),
    ("print cash sales for today", "R3", "2026-10-10", "2026-10-10", ""),
    ("نقدی بکری کا پرنٹ آج کا", "R3", "2026-10-10", "2026-10-10", ""),
    ("udhari bikri ka print is mahine ka", "R4", "2026-10-01", "2026-10-10", ""),
    ("print credit sales this month", "R4", "2026-10-01", "2026-10-10", ""),
    ("ادھاری بکری کا پرنٹ پچھلے مہینے کا", "R4", "2026-09-01", "2026-09-30", ""),
    // R7 item-wise sales.
    ("item wise bikri ka print is mahine ka", "R7", "2026-10-01", "2026-10-10", ""),
    ("print item wise sales for this month", "R7", "2026-10-01", "2026-10-10", ""),
    ("قسم وار بکری کا پرنٹ اس مہینے کا", "R7", "2026-10-01", "2026-10-10", ""),
    // R8 the truck stand report (named by its title; an adda named is a filter).
    ("adda report ka print is mahine ka", "R8", "2026-10-01", "2026-10-10", ""),
    ("اڈا رپورٹ کا اس مہینے کا پرنٹ نکالنا ہے", "R8", "2026-10-01", "2026-10-10", ""),
    // R15 the list of vouchers.
    ("voucher list ka print aaj ka", "R15", "2026-10-10", "2026-10-10", ""),
    ("print the voucher list for this month", "R15", "2026-10-01", "2026-10-10", ""),
    // R16 the account index, R17 the head balances, R19 the opening trial.
    ("account index ka print", "R16", "", "", "2026-10-10"),
    ("print the accounts list", "R16", "", "", "2026-10-10"),
    ("head balances ka print", "R17", "", "", "2026-10-10"),
    ("print head balances", "R17", "", "", "2026-10-10"),
    ("opening trial ka print", "R19", "", "", ""),
    ("print the opening trial balance", "R19", "", "", ""),
    // R22 dues by collector.
    ("collector wise ugrahi ka print", "R22", "", "", "2026-10-10"),
    ("print collector wise dues", "R22", "", "", "2026-10-10"),
    // R23 receipts register, R25 payments register.
    ("wasooli register ka print is mahine ka", "R23", "2026-10-01", "2026-10-10", ""),
    ("print the receipts register for this month", "R23", "2026-10-01", "2026-10-10", ""),
    ("adaigi register ka print is mahine ka", "R25", "2026-10-01", "2026-10-10", ""),
    ("print the payments register for this month", "R25", "2026-10-01", "2026-10-10", ""),
    // R26 advances: the figure "peshgi" with no party.
    ("peshgi ka print", "R26", "", "", "2026-10-10"),
    ("print the advances", "R26", "", "", "2026-10-10"),
    // R27 the bill series.
    ("bijak series ka print is mahine ka", "R27", "2026-10-01", "2026-10-10", ""),
    ("print the bill series for this month", "R27", "2026-10-01", "2026-10-10", ""),
    // R28 income report.
    ("amdan report ka print is mahine ka", "R28", "2026-10-01", "2026-10-10", ""),
    ("print the income report for this month", "R28", "2026-10-01", "2026-10-10", ""),
    // R30 profit and partner split: the figure "munafa" with no party.
    ("munafa ka print is mahine ka", "R30", "2026-10-01", "2026-10-10", ""),
    ("print the profit for this month", "R30", "2026-10-01", "2026-10-10", ""),
    ("منافع و تقسیم کا پرنٹ اس مہینے کا", "R30", "2026-10-01", "2026-10-10", ""),
    // R32 money lent, R38 the dead amount.
    ("udhar raqam ka print", "R32", "", "", "2026-10-10"),
    ("print money lent report", "R32", "", "", "2026-10-10"),
    ("dubi raqam ka print", "R38", "", "", ""),
    ("print the dead amount report", "R38", "", "", ""),
    // R33 and R36 by their words, with no trip or garden named: all of them, for the days.
    ("market safar ki is mahine ki report ka print nikalna hai", "R33", "2026-10-01", "2026-10-10", ""),
    ("print the market trips report for this month", "R33", "2026-10-01", "2026-10-10", ""),
    ("print the trips report for this month", "R33", "2026-10-01", "2026-10-10", ""),
    ("print trip report for today", "R33", "2026-10-10", "2026-10-10", ""),
    ("bagh ki is mahine ki bikri ka print", "R36", "2026-10-01", "2026-10-10", ""),
    ("print gardens report for this month", "R36", "2026-10-01", "2026-10-10", ""),
    // What was collected on a day is the receipts register, not the list of what is still due.
    ("aaj ki wasooli ka print", "R23", "2026-10-10", "2026-10-10", ""),
    // The day book in the plural (Urdu oblique).
    ("کل کے روزنامچے کا پرنٹ", "R11", "2026-10-09", "2026-10-09", ""),
    // A partner's fruit is the fruit register for him.
    ("print Partner A's fruit for this month", "R37", "2026-10-01", "2026-10-10", ""),
];

fn on_worked_day() -> world::World {
    worked::worked_day()
}

fn reader(w: &world::World, language: Language) -> Reader<'_> {
    Reader {
        ctx: &w.env.ctx,
        actor: &w.owner,
        language,
        today: BusinessDate::from_ymd(2026, 10, 10).expect("a date"),
        partner_locked: false,
    }
}

#[test]
fn every_report_without_a_plan_has_one_with_its_own_filters() {
    let w = on_worked_day();
    let mut wrong: Vec<String> = Vec::new();
    for (question, id, from, to, as_of) in CASES {
        let u = understand(&reader(&w, Language::En), question);
        let Some(plan) = &u.report else {
            wrong.push(format!("{question}: no plan; doubts {:?}", u.doubts));
            continue;
        };
        let iso = |d: Option<chrono::NaiveDate>| {
            d.map(|d| d.format("%Y-%m-%d").to_string())
                .unwrap_or_default()
        };
        let got = (
            plan.id,
            iso(plan.filters.from),
            iso(plan.filters.to),
            iso(plan.filters.as_of),
        );
        if u.intent.value != Intent::Print
            || !u.sure
            || got
                != (
                    *id,
                    (*from).to_owned(),
                    (*to).to_owned(),
                    (*as_of).to_owned(),
                )
        {
            wrong.push(format!(
                "{question}: got {got:?} (intent {:?}, sure {}), wanted {id} {from} {to} {as_of}",
                u.intent.value, u.sure
            ));
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// A local answer with the button and the steps is made for every case, in both languages, and the
/// button is the report of the plan with its dates.
#[test]
fn every_planned_report_has_a_button_and_steps_in_both_languages() {
    let w = on_worked_day();
    let words = Words::new().expect("words");
    for language in [Language::En, Language::Ur] {
        for (question, id, from, to, as_of) in CASES {
            let u = understand(&reader(&w, language), question);
            let composed = compose(&u, language, &words, READ_AT)
                .unwrap_or_else(|| panic!("{question}: no local answer"));
            let answer = composed.answer.as_ref().expect("an answer");
            let Some(AnswerSource::Report {
                id: button,
                filters,
                ..
            }) = answer.sources.first()
            else {
                panic!("{question}: no button");
            };
            assert_eq!(format!("{button:?}"), *id, "{question}");
            assert_eq!(
                (&filters.from[..], &filters.to[..], &filters.as_of[..]),
                (*from, *to, *as_of),
                "{question}"
            );
            assert!(
                answer
                    .how
                    .iter()
                    .any(|s| s.id == "assistant-how-open-report"),
                "{question}: no steps"
            );
        }
    }
}

/// Reports the app cannot place from the words stay with the model: no button for a guess.
#[test]
fn a_report_name_with_a_figure_it_does_not_hold_or_no_day_is_not_planned_wide() {
    let w = on_worked_day();
    for question in [
        // One word of a report title beside another report's: two reports in one question.
        "print voucher list and credit sales",
        // A garden word and a trip word together are two reports.
        "bagh aur safar ki bikri ka print",
        // A garden report cannot hold a cash figure.
        "bagh ki roqar ka print",
    ] {
        let u = understand(&reader(&w, Language::En), question);
        assert!(
            !u.sure || u.report.is_none(),
            "{question}: planned wide: {:?}",
            u.report
        );
        assert!(
            u.report.is_none(),
            "{question}: a button for an unsure question: {:?}",
            u.report
        );
    }
}

/// The entry screen of a request to write for a garden, a trip or a بکری already saved (the lead's
/// order of 05-11-2026). The screen names are the new `EntryScreenName` values the backend wires; the
/// route carries the garden or the trip as its party and a بکری change carries none.
#[test]
fn a_request_for_a_garden_a_trip_or_a_saved_bikri_names_its_own_entry_screen() {
    let w = on_worked_day();
    let mut wrong: Vec<String> = Vec::new();
    for (question, screen) in [
        ("bagh ki qist likh do", "garden_installment"),
        ("bagh ki qist entry karo", "garden_installment"),
        ("باغ کی قسط لکھ دو", "garden_installment"),
        ("bagh ka kharcha likh do", "garden_cost"),
        ("bagh ki doosri dukan mein bikri likh do", "garden_sale"),
        ("bagh band kar do", "garden_close"),
        ("safar ki khareedari likh do", "trip_purchase"),
        ("safar ka kharcha likh do", "trip_cost"),
        ("safar ki doosri dukan mein bikri likh do", "trip_sale"),
        ("bikri 5 cancel karo", "bikri_cancel"),
        ("pichli bikri cancel kar do", "bikri_cancel"),
        ("بکری منسوخ کرو", "bikri_cancel"),
        ("bikri 5 edit karo", "bikri_edit"),
        ("baad mein bikri likh do", "bikri_later_sale"),
        ("baqi maal ki bikri likh do", "bikri_later_sale"),
        // The entries that already had their screen keep it.
        ("bikri likh do", "bikri"),
        ("akram ki 5000 wasooli likh do", "wasooli"),
        ("3000 kharcha likh do", "kharcha"),
        ("peshgi likh do", "peshgi"),
    ] {
        let u = understand(&reader(&w, Language::En), question);
        if u.intent.value != Intent::ChangeRequest || u.screen != Some(screen) {
            wrong.push(format!(
                "{question}: intent {:?}, screen {:?}, wanted {screen}",
                u.intent.value, u.screen
            ));
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

// ---- items, the guests, one بکری, and the analytics views (lead order of 05-11-2026: W1 to W3, P1) -----

/// What a plan carries: the report or view, its item, whether it is for the guests, the بکری number,
/// the dates (`from`, `to`) and the party's English name.
#[derive(Debug, PartialEq, Eq)]
struct Carried {
    id: String,
    item: String,
    guests: bool,
    bikri: Option<i64>,
    from: String,
    to: String,
    party: String,
}

fn carried(u: &mandi_assistant::understand::Understanding) -> Option<Carried> {
    let plan = u.report.as_ref()?;
    let f = &plan.filters;
    let iso = |d: Option<chrono::NaiveDate>| {
        d.map(|d| d.format("%Y-%m-%d").to_string())
            .unwrap_or_default()
    };
    Some(Carried {
        id: plan.id.to_owned(),
        item: f
            .item
            .as_ref()
            .map(|i| i.name_en.clone())
            .unwrap_or_default(),
        guests: f.guests,
        bikri: f.bikri.map(|(_, number)| number),
        from: iso(f.from),
        to: iso(f.to),
        party: f
            .party
            .as_ref()
            .or(f.seller.as_ref())
            .or(f.partner.as_ref())
            .map(|p| p.name_en.clone())
            .unwrap_or_default(),
    })
}

/// (question, report, item, guests, bikri, from, to, party).
type Case = (
    &'static str,
    &'static str,
    &'static str,
    bool,
    Option<i64>,
    &'static str,
    &'static str,
    &'static str,
);

#[rustfmt::skip]
const ITEMS_GUESTS_VIEWS: &[Case] = &[
    // W3: a print that names an item. (question, report, item, guests, bikri, from, to, party)
    ("kele ki aaj ki bikri ka print", "R7", "Banana", false, None, "2026-10-10", "2026-10-10", ""),
    ("kele ka commission print is mahine ka", "R7", "Banana", false, None, "2026-10-01", "2026-10-10", ""),
    ("akram ki kele ki bikri ka print aaj ka", "R7", "Banana", false, None, "2026-10-10", "2026-10-10", "Akram Toll Tax"),
    ("kele ka baqi maal print", "R10", "Banana", false, None, "2026-08-31", "2026-10-10", ""),
    ("kele ki bijak series ka print is mahine ka", "R27", "Banana", false, None, "2026-10-01", "2026-10-10", ""),
    ("print the banana sales for today", "R7", "Banana", false, None, "2026-10-10", "2026-10-10", ""),
    ("print the banana sales up to now", "R7", "Banana", false, None, "2026-08-31", "2026-10-10", ""),
    ("print the banana sales so far", "R7", "Banana", false, None, "2026-08-31", "2026-10-10", ""),
    // P1: the guests' fruit.
    ("mehmano ke phal ka print is mahine ka", "R37", "", true, None, "2026-10-01", "2026-10-10", ""),
    ("print the fruit taken for guests this month", "R37", "", true, None, "2026-10-01", "2026-10-10", ""),
    // W1: one بکری by its number (the worked day has بکری 1 and 2).
    ("bikri number 1 ka print", "R1", "", false, Some(1), "", "", ""),
    ("print sale number 2", "R1", "", false, Some(2), "", "", ""),
    // W2: the analytics views, opened or printed with their filters.
    ("item wise tajziya ka print is mahine ka", "A", "", false, None, "2026-10-01", "2026-10-10", ""),
    ("seller wise tajziya ka print", "B", "", false, None, "", "", ""),
    ("buyer wise tajziya print is mahine ka", "C", "", false, None, "2026-10-01", "2026-10-10", ""),
    ("kele ka month wise commission print", "D", "Banana", false, None, "", "", ""),
    ("commission trend ka print is mahine ka", "G", "", false, None, "2026-10-01", "2026-10-10", ""),
    ("bagh tajziya ka print", "I", "", false, None, "", "", ""),
    ("safar tajziya ka print", "J", "", false, None, "", "", ""),
    ("phal tajziya ka print is mahine ka", "K", "", false, None, "2026-10-01", "2026-10-10", ""),
    ("sarfehrist ka print is mahine ka", "H", "", false, None, "2026-10-01", "2026-10-10", ""),
    ("top 10 tajziya ka print is mahine ka", "H", "", false, None, "2026-10-01", "2026-10-10", ""),
    ("سرفہرست کا پرنٹ اس مہینے کا", "H", "", false, None, "2026-10-01", "2026-10-10", ""),
    ("akram ka buyer wise tajziya print", "C", "", false, None, "", "", "Akram Toll Tax"),
    ("rahim ka seller wise tajziya print", "B", "", false, None, "", "", "Rahim"),
];

#[test]
fn items_guests_one_bikri_and_the_analytics_views_have_plans() {
    let w = on_worked_day();
    worked::events_1_to_5(&w);
    let mut wrong: Vec<String> = Vec::new();
    for (question, id, item, guests, bikri, from, to, party) in ITEMS_GUESTS_VIEWS {
        let u = understand(&reader(&w, Language::En), question);
        let want = Carried {
            id: (*id).to_owned(),
            item: (*item).to_owned(),
            guests: *guests,
            bikri: *bikri,
            from: (*from).to_owned(),
            to: (*to).to_owned(),
            party: (*party).to_owned(),
        };
        match carried(&u) {
            Some(got) if got == want && u.sure && u.intent.value == Intent::Print => {}
            got => wrong.push(format!(
                "{question}: got {got:?} (sure {}, doubts {:?}), wanted {want:?}",
                u.sure, u.doubts
            )),
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// Each of them has a button of the right kind and the steps, in both languages: an analytics view
/// is an analytics source, the others a report source with the item, the guests or the number.
#[test]
fn those_plans_have_a_button_and_steps_in_both_languages() {
    use mandi_assistant::answer::AnswerSource;
    let w = on_worked_day();
    worked::events_1_to_5(&w);
    let words = Words::new().expect("words");
    for language in [Language::En, Language::Ur] {
        for (question, id, item, guests, bikri, ..) in ITEMS_GUESTS_VIEWS {
            let u = understand(&reader(&w, language), question);
            let composed = compose(&u, language, &words, READ_AT)
                .unwrap_or_else(|| panic!("{question}: no local answer"));
            let answer = composed.answer.as_ref().expect("an answer");
            match answer.sources.first() {
                Some(AnswerSource::Analytics { view, filters, .. }) => {
                    assert_eq!(format!("{view:?}"), *id, "{question}");
                    assert_eq!(
                        filters
                            .item
                            .as_ref()
                            .map(|i| i.name_en.as_str())
                            .unwrap_or(""),
                        *item,
                        "{question}"
                    );
                    assert!(
                        answer
                            .how
                            .iter()
                            .any(|s| s.id == "assistant-how-open-analytics"),
                        "{question}: no analytics step"
                    );
                }
                Some(AnswerSource::Report {
                    id: button,
                    filters,
                    ..
                }) => {
                    assert_eq!(format!("{button:?}"), *id, "{question}");
                    assert_eq!(
                        filters
                            .item
                            .as_ref()
                            .map(|i| i.name_en.as_str())
                            .unwrap_or(""),
                        *item,
                        "{question}"
                    );
                    assert_eq!(filters.guests_only, *guests, "{question}");
                    assert_eq!(filters.bikri.map(|b| b.number), *bikri, "{question}");
                }
                other => panic!("{question}: no button: {other:?}"),
            }
            assert!(!answer.how.is_empty(), "{question}: no steps");
        }
    }
}

/// An item with two names, a figure no item report holds, a number that is no بکری, and a garden or
/// a trip word beside a view all stay with the model: no button for a guess.
#[test]
fn an_item_print_that_no_report_holds_is_not_planned() {
    let w = on_worked_day();
    worked::events_1_to_5(&w);
    for question in [
        "kele aur saib ki bikri ka print",
        "kele ki roqar ka print",
        "bikri number 99 ka print",
        "kele ka rate print",
    ] {
        let u = understand(&reader(&w, Language::En), question);
        assert!(
            u.report.is_none(),
            "{question}: a button for a guess: {:?}",
            u.report
        );
    }
}

/// "audit log kholo" is answered by the app with the guide's words and a link-only button that
/// opens the audit log (the screen source, plan 55 §11.1); the question about who changed what is
/// read with `app_state` audit and has no local answer.
#[test]
fn the_audit_log_is_opened_by_words_with_a_screen_button() {
    use mandi_assistant::answer::{AnswerSource, ScreenRoute};
    let w = on_worked_day();
    let words = Words::new().expect("words");
    for language in [Language::En, Language::Ur] {
        for question in ["audit log kholo", "show the audit log", "آڈٹ لاگ کھولیں"] {
            let u = understand(&reader(&w, language), question);
            let composed = compose(&u, language, &words, READ_AT)
                .unwrap_or_else(|| panic!("{question}: no local answer"));
            let answer = composed.answer.as_ref().expect("an answer");
            assert!(
                matches!(
                    answer.sources.first(),
                    Some(AnswerSource::Screen { result_id, route: ScreenRoute::Audit })
                        if result_id.is_empty()
                ),
                "{question}: {:?}",
                answer.sources
            );
        }
    }
}

/// One rule for a print of a period report that names no period (lead ruling of 06-11-2026): the
/// report opens on its own default, today to today, and the steps say the day was taken and how to
/// change it. It does not stop to ask. A person's or a record's report with no period keeps the whole
/// books (R31, R36, R33, R13 from the start).
#[test]
fn a_print_with_no_period_opens_on_today_and_the_steps_say_so() {
    use mandi_assistant::answer::AnswerSource;
    let w = on_worked_day();
    let words = Words::new().expect("words");
    for language in [Language::En, Language::Ur] {
        for (question, report, party) in [
            ("cash book print", "R12", ""),
            ("expense report ka print", "R29", ""),
            ("bikri ka print", "R34", ""),
            ("akram ka print", "R35", ""),
            ("partnaron ki li hui raqam ka print nikalna hai", "R29", ""),
            ("print the money the partners took", "R29", ""),
            ("partner a ki li hui raqam ka print", "R29", "Partner A"),
            ("پارٹنروں کی لی ہوئی رقم کا پرنٹ نکالنا ہے", "R29", ""),
            ("ذاتی خرچہ کا پرنٹ", "R29", ""),
        ] {
            let u = understand(&reader(&w, language), question);
            let plan = u.report.as_ref().unwrap_or_else(|| {
                panic!(
                    "{question}: no plan; sure {}, doubts {:?}",
                    u.sure, u.doubts
                )
            });
            assert_eq!(plan.id, report, "{question}");
            assert!(
                plan.assumed_period,
                "{question}: the day was taken, not read"
            );
            assert!(u.sure, "{question}: it must not stop to ask");
            let day = if report == "R35" {
                plan.filters.as_of
            } else {
                assert_eq!(plan.filters.from, plan.filters.to, "{question}");
                plan.filters.from
            };
            assert_eq!(
                day.map(|d| d.format("%Y-%m-%d").to_string()).as_deref(),
                Some("2026-10-10"),
                "{question}"
            );
            if !party.is_empty() {
                assert_eq!(
                    plan.filters.partner.as_ref().map(|p| p.name_en.as_str()),
                    Some(party),
                    "{question}"
                );
            }
            let composed = compose(&u, language, &words, READ_AT)
                .unwrap_or_else(|| panic!("{question}: no local answer"));
            let answer = composed.answer.as_ref().expect("an answer");
            assert!(
                matches!(answer.sources.first(), Some(AnswerSource::Report { .. })),
                "{question}: no button"
            );
            let ids: Vec<&str> = answer.how.iter().map(|s| s.id.as_str()).collect();
            assert!(
                ids.contains(&"assistant-how-assumed-day"),
                "{question}: {ids:?}"
            );
            // The way to change it: the dates step (a day slip, R35, has its one day).
            assert!(
                ids.contains(&"assistant-how-set-dates") || ids.contains(&"assistant-how-set-day"),
                "{question}: {ids:?}"
            );
        }
    }
    // A person's or a record's report keeps the whole books, with no day assumed.
    for question in ["partner a ka statement print", "bagh ke balance ka print"] {
        let u = understand(&reader(&w, Language::En), question);
        if let Some(plan) = &u.report {
            assert!(!plan.assumed_period, "{question}");
            assert!(
                plan.filters.from.is_none() && plan.filters.to.is_none(),
                "{question}"
            );
        }
    }
}

/// Opening a view is the same plan: "sarfehrist kholo" and "top 10 tajziya kholo" open view H (Top 10)
/// with an analytics button, in both languages; the questions that ask for the top of a list ("top 5
/// buyers", "top 10 mein sab se upar kaun sa item hai") are not changed by the title words.
#[test]
fn view_h_opens_by_its_title_and_the_top_questions_are_unchanged() {
    use mandi_assistant::answer::AnswerSource;
    let w = on_worked_day();
    let words = Words::new().expect("words");
    for language in [Language::En, Language::Ur] {
        for question in ["sarfehrist kholo", "top 10 tajziya kholo", "سرفہرست کھولیں"]
        {
            let u = understand(&reader(&w, language), question);
            let composed = compose(&u, language, &words, READ_AT)
                .unwrap_or_else(|| panic!("{question}: no local answer, {u:?}"));
            let answer = composed.answer.as_ref().expect("an answer");
            assert!(
                matches!(
                    answer.sources.first(),
                    Some(AnswerSource::Analytics { view, .. }) if format!("{view:?}") == "H"
                ),
                "{question}: {:?}",
                answer.sources
            );
        }
    }
    for question in ["top 5 buyers", "top 10 mein sab se upar kaun sa item hai"] {
        let u = understand(&reader(&w, Language::En), question);
        assert_ne!(u.intent.value, Intent::Print, "{question}");
        assert!(
            !u.figures
                .value
                .iter()
                .any(|k| matches!(k, mandi_assistant::understand::FigureKind::Report(_))),
            "{question}: a title word took the ranking: {:?}",
            u.figures.value
        );
    }
}

/// A print of a group of items (sabzi, phal) opens the item-wise sales (R7), the stock (R10) or the bill
/// series (R27) with the GROUP filter, and the answer's steps say which group; never a report of the
/// whole shop (Umar: the shop sells vegetables as well as fruit).
#[test]
fn a_print_of_a_group_carries_the_group_filter() {
    use mandi_assistant::answer::AnswerSource;
    let w = on_worked_day();
    let words = Words::new().expect("words");
    for language in [Language::En, Language::Ur] {
        for (question, report, group, from, to) in [
            (
                "is mahine sabzi ki bikri ka print",
                "R7",
                "vegetable",
                "2026-10-01",
                "2026-10-10",
            ),
            (
                "sabziyon ka commission print is mahine ka",
                "R7",
                "vegetable",
                "2026-10-01",
                "2026-10-10",
            ),
            (
                "phal ki aaj ki bikri ka print",
                "R7",
                "fruit",
                "2026-10-10",
                "2026-10-10",
            ),
            (
                "print the vegetables sales for today",
                "R7",
                "vegetable",
                "2026-10-10",
                "2026-10-10",
            ),
            (
                "sabzi ki bijak series ka print is mahine ka",
                "R27",
                "vegetable",
                "2026-10-01",
                "2026-10-10",
            ),
            (
                "sabzi ki naqdi bikri ka print is mahine ka",
                "R3",
                "vegetable",
                "2026-10-01",
                "2026-10-10",
            ),
            (
                "phal ki udhari bikri ka print aaj ka",
                "R4",
                "fruit",
                "2026-10-10",
                "2026-10-10",
            ),
            (
                "sabzi ka baqi maal print",
                "R10",
                "vegetable",
                "2026-08-31",
                "2026-10-10",
            ),
            (
                "سبزی کی بکری کا پرنٹ اس مہینے کا",
                "R7",
                "vegetable",
                "2026-10-01",
                "2026-10-10",
            ),
            (
                "پھل کی بکری کا پرنٹ آج کا",
                "R7",
                "fruit",
                "2026-10-10",
                "2026-10-10",
            ),
        ] {
            let u = understand(&reader(&w, language), question);
            let plan = u
                .report
                .as_ref()
                .unwrap_or_else(|| panic!("{question}: no plan, {:?}", u.doubts));
            let day = |d: Option<chrono::NaiveDate>| {
                d.map(|d| d.format("%Y-%m-%d").to_string())
                    .unwrap_or_default()
            };
            assert_eq!(
                (
                    plan.id,
                    plan.filters
                        .item_group
                        .map(mandi_assistant::understand::GroupWord::tool_word),
                    day(plan.filters.from),
                    day(plan.filters.to)
                ),
                (report, Some(group), from.to_owned(), to.to_owned()),
                "{question}"
            );
            let composed = compose(&u, language, &words, READ_AT)
                .unwrap_or_else(|| panic!("{question}: no local answer"));
            let answer = composed.answer.as_ref().expect("an answer");
            let Some(AnswerSource::Report { filters, .. }) = answer.sources.first() else {
                panic!("{question}: no button");
            };
            assert_eq!(
                filters.item_group.map(|g| g.as_str()),
                Some(group),
                "{question}: the button lost the group"
            );
            assert!(
                answer.how.iter().any(|s| s.id == "assistant-how-set-group"),
                "{question}: no group step"
            );
        }
    }
}
