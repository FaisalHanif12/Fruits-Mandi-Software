//! Plan 55 Section 7: the answers the app writes by itself, with no model call. Each case asks a
//! question of the real books (the PRD worked day, today 10-10-2026), understands it, composes the
//! local answer and checks what the panel will get: the guide's words, the buttons (sources with an
//! empty `result_id`), the steps (`how`) with the real values, and for a name the books do not have,
//! `not_found` with `asked_role` and `found_as`.
//!
//! The button and the steps must agree: both are made from the same plan, and a test reads the
//! dates and the party out of the steps and out of the button and compares them.
#![cfg(test)]

#[path = "../../services/tests/common/mod.rs"]
mod common;
#[path = "support/worked.rs"]
mod worked;
#[allow(dead_code)]
#[path = "../../services/tests/bikri_preview/support.rs"]
mod world;
#[path = "robustness_year/mod.rs"]
mod year;

use mandi_assistant::answer::{Answer, AnswerPart, AnswerSource, EntryScreen};
use mandi_assistant::ask::{AskOutcome, AskedRole};
use mandi_assistant::prompt::guide;
use mandi_assistant::results::Words;
use mandi_assistant::understand::{
    Composed, Doubt, OutsideKind, Reader, Understanding, compose, outside_message, plan_reads,
    understand,
};
use mandi_reports::reports::{ArgValue, ReportId};
use mandi_services::core::accounts::AccountType;
use mandi_services::core::auth::Language;
use mandi_services::core::dates::BusinessDate;
use mandi_services::core::masters::{AddaInput, CityInput, MasterInput};

const READ_AT: &str = "2026-10-10T08:00:00Z";

fn books() -> world::World {
    let w = worked::worked_day();
    let add = |kind, ur: &str, en: &str| {
        world::add_account(&w.env, &w.owner, kind, ur, en, true, "");
    };
    add(AccountType::Buyer, "فیصل", "Faisal");
    worked::expense_head(&w, "بجلی", "Electricity");
    let city = save(&w, |words| {
        MasterInput::City(CityInput {
            id: None,
            version: None,
            code: String::new(),
            name_ur: "لاہور".into(),
            name_en: "Lahore".into(),
            confirmed_rules_words: words.to_vec(),
        })
    });
    save(&w, |words| {
        MasterInput::Adda(AddaInput {
            id: None,
            version: None,
            code: String::new(),
            name_ur: "سوات اڈا".into(),
            name_en: "Swat Adda".into(),
            city_id: Some(city),
            confirmed_rules_words: words.to_vec(),
        })
    });
    w
}

fn save(w: &world::World, input: impl Fn(&[String]) -> MasterInput) -> i64 {
    use mandi_services::core::error::{ErrorCode, ErrorValue};
    let mut words: Vec<String> = Vec::new();
    loop {
        match mandi_services::masters::save(&w.env.ctx, &w.owner, &input(&words)) {
            Ok(row) => return row.id(),
            Err(mandi_services::ServiceError::Rejected(items))
                if items
                    .iter()
                    .all(|i| i.code == ErrorCode::ErrSpellingUnconfirmed) =>
            {
                for item in items {
                    if let Some(ErrorValue::Text(text)) = item.values.get("words") {
                        words.extend(text.split(' ').map(str::to_owned));
                    }
                }
            }
            Err(other) => panic!("{other:?}"),
        }
    }
}

fn compose_for(w: &world::World, question: &str, language: Language) -> Option<Composed> {
    let reader = Reader {
        ctx: &w.env.ctx,
        actor: &w.owner,
        language,
        today: BusinessDate::from_ymd(2026, 10, 10).expect("a date"),
        partner_locked: false,
    };
    let words = Words::new().expect("the words");
    compose(&understand(&reader, question), language, &words, READ_AT)
}

fn answer_of(c: &Composed) -> &Answer {
    c.answer.as_ref().expect("an answer")
}

/// The ids of the steps, and the text arguments of one step.
fn step_ids(a: &Answer) -> Vec<&str> {
    a.how.iter().map(|s| s.id.as_str()).collect()
}

fn arg<'a>(a: &'a Answer, step: &str, name: &str) -> Option<&'a ArgValue> {
    a.how
        .iter()
        .find(|s| s.id == step)?
        .args
        .iter()
        .find(|x| x.name == name)
        .map(|x| &x.value)
}

fn text_arg(a: &Answer, step: &str, name: &str) -> Option<String> {
    match arg(a, step, name)? {
        ArgValue::Text { text } => Some(text.clone()),
        _ => None,
    }
}

/// The first button: its report, its dates and its party.
fn button(a: &Answer, at: usize) -> (ReportId, String, String, String, Option<String>) {
    match a.sources.get(at) {
        Some(AnswerSource::Report {
            result_id,
            id,
            filters,
            ..
        }) => {
            assert!(result_id.is_empty(), "a link only has no result id");
            (
                *id,
                filters.from.clone(),
                filters.to.clone(),
                filters.as_of.clone(),
                filters
                    .party
                    .as_ref()
                    .map(|p| p.record.name_en.clone())
                    .or_else(|| filters.seller.as_ref().map(|r| r.name_en.clone())),
            )
        }
        other => panic!("not a report button: {other:?}"),
    }
}

/// The button and the steps tell the same: the report, the dates, the day.
fn agree(a: &Answer) {
    let (id, from, to, as_of, _) = button(a, 0);
    let code = serde_json::to_value(id)
        .ok()
        .and_then(|v| v.as_str().map(str::to_owned))
        .expect("a report code");
    assert_eq!(
        text_arg(a, "assistant-how-open-report", "report"),
        Some(code)
    );
    if !from.is_empty() && from != to {
        assert_eq!(text_arg(a, "assistant-how-set-dates", "from"), Some(from));
        assert_eq!(text_arg(a, "assistant-how-set-dates", "to"), Some(to));
    } else if !as_of.is_empty() {
        assert_eq!(text_arg(a, "assistant-how-set-day", "day"), Some(as_of));
    } else if !from.is_empty() {
        assert_eq!(text_arg(a, "assistant-how-set-day", "day"), Some(from));
    } else {
        assert!(
            arg(a, "assistant-how-set-day", "day").is_none()
                && arg(a, "assistant-how-set-dates", "from").is_none(),
            "no day in the button, none in the steps"
        );
    }
}

fn guide_text(topic: &str, language: Language) -> &'static mandi_assistant::prompt::GuideText {
    let t = guide().topics.get(topic).expect("a topic");
    match language {
        Language::Ur => &t.ur,
        Language::En => &t.en,
    }
}

#[test]
fn a_print_gives_the_button_and_the_steps_with_the_real_values() {
    let w = books();
    // A buyer, a period: R5 and R13 as the second button.
    let c = compose_for(
        &w,
        "faisal ki pichle mahine ki bikri ka print",
        Language::En,
    )
    .expect("a local answer");
    assert_eq!(c.outcome, AskOutcome::Answered);
    let a = answer_of(&c);
    assert_eq!(a.sources.len(), 2);
    let (id, from, to, _, party) = button(a, 0);
    assert_eq!(
        (id, from.as_str(), to.as_str(), party.as_deref()),
        (ReportId::R5, "2026-09-01", "2026-09-30", Some("Faisal"))
    );
    assert_eq!(button(a, 1).0, ReportId::R13);
    assert_eq!(
        step_ids(a),
        [
            "assistant-how-open-report",
            "assistant-how-set-party",
            "assistant-how-set-dates",
            "assistant-how-press-print"
        ]
    );
    assert_eq!(
        text_arg(a, "assistant-how-set-party", "role").as_deref(),
        Some("buyer")
    );
    assert!(text_arg(a, "assistant-how-set-party", "code").is_some_and(|c| !c.is_empty()));
    // Only the guide's own words.
    assert_eq!(
        a.parts,
        [AnswerPart::Text {
            text: guide_text("printing", Language::En).summary.clone()
        }]
    );
    agree(a);

    // A buyer, one day: the day slip R35, one button, the day.
    let c = compose_for(
        &w,
        "main ne faisal ki aj ki bikri ka print nikalna hai?",
        Language::Ur,
    )
    .expect("a local answer");
    let a = answer_of(&c);
    assert_eq!(a.sources.len(), 1);
    assert_eq!(button(a, 0).0, ReportId::R35);
    assert_eq!(
        step_ids(a),
        [
            "assistant-how-open-report",
            "assistant-how-set-party",
            "assistant-how-set-day",
            "assistant-how-press-print"
        ]
    );
    assert_eq!(
        text_arg(a, "assistant-how-set-day", "day").as_deref(),
        Some("2026-10-10")
    );
    // The Urdu app gets the Urdu guide's words.
    assert_eq!(
        a.parts,
        [AnswerPart::Text {
            text: guide_text("printing", Language::Ur).summary.clone()
        }]
    );
    agree(a);

    // A seller: R6 with the seller, and R13.
    let c =
        compose_for(&w, "rahim ki bikri ka pichle mahine ka print", Language::En).expect("local");
    let a = answer_of(&c);
    assert_eq!(
        (button(a, 0).0, button(a, 0).4.as_deref(), button(a, 1).0),
        (ReportId::R6, Some("Rahim"), ReportId::R13)
    );
    assert_eq!(
        text_arg(a, "assistant-how-set-party", "role").as_deref(),
        Some("seller")
    );
    agree(a);

    // No party: a day is R34, a period is R2.
    let a =
        answer_of(&compose_for(&w, "aaj ki bikri ka print", Language::En).expect("local")).clone();
    assert_eq!(button(&a, 0).0, ReportId::R34);
    assert_eq!(
        step_ids(&a),
        [
            "assistant-how-open-report",
            "assistant-how-set-day",
            "assistant-how-press-print"
        ]
    );
    agree(&a);
    let a = answer_of(
        &compose_for(&w, "pichle mahine ki bikri ka print", Language::En).expect("local"),
    )
    .clone();
    assert_eq!(button(&a, 0).0, ReportId::R2);
    agree(&a);
}

#[test]
fn the_rows_of_ruling_one_have_a_button_and_steps_that_agree() {
    let w = books();
    let cases: [(&str, ReportId, &[&str]); 11] = [
        (
            "alif ka statement print karo",
            ReportId::R31,
            &[
                "assistant-how-open-report",
                "assistant-how-set-record",
                "assistant-how-press-print",
            ],
        ),
        (
            "alif ka khata pichle mahine ka print",
            ReportId::R31,
            &[
                "assistant-how-open-report",
                "assistant-how-set-record",
                "assistant-how-set-dates",
                "assistant-how-press-print",
            ],
        ),
        (
            "roqar ka print is mahine ka",
            ReportId::R12,
            &[
                "assistant-how-open-report",
                "assistant-how-set-dates",
                "assistant-how-press-print",
            ],
        ),
        (
            "pichle mahine ke kharche ka print",
            ReportId::R29,
            &[
                "assistant-how-open-report",
                "assistant-how-set-dates",
                "assistant-how-press-print",
            ],
        ),
        (
            "bijli ka kharcha print pichle mahine ka",
            ReportId::R29,
            &[
                "assistant-how-open-report",
                "assistant-how-set-dates",
                "assistant-how-find-row",
                "assistant-how-press-print",
            ],
        ),
        (
            "dues ka print",
            ReportId::R20,
            &[
                "assistant-how-open-report",
                "assistant-how-set-day",
                "assistant-how-press-print",
            ],
        ),
        (
            "sellers ka payable print karo",
            ReportId::R24,
            &[
                "assistant-how-open-report",
                "assistant-how-set-day",
                "assistant-how-press-print",
            ],
        ),
        (
            "baqi maal ka print",
            ReportId::R10,
            &[
                "assistant-how-open-report",
                "assistant-how-set-dates",
                "assistant-how-press-print",
            ],
        ),
        (
            "trial balance print karo",
            ReportId::R18,
            &[
                "assistant-how-open-report",
                "assistant-how-set-day",
                "assistant-how-press-print",
            ],
        ),
        (
            "roznamcha print aaj ka",
            ReportId::R11,
            &[
                "assistant-how-open-report",
                "assistant-how-set-day",
                "assistant-how-press-print",
            ],
        ),
        (
            "swat adda ki bikri ka print pichle mahine ka",
            ReportId::R8,
            &[
                "assistant-how-open-report",
                "assistant-how-set-party",
                "assistant-how-set-dates",
                "assistant-how-press-print",
            ],
        ),
    ];
    for (question, report, steps) in cases {
        let c = compose_for(&w, question, Language::En)
            .unwrap_or_else(|| panic!("no local answer for {question:?}"));
        let a = answer_of(&c);
        assert_eq!(button(a, 0).0, report, "{question}");
        assert_eq!(step_ids(a), steps, "{question}");
        agree(a);
    }
    // A city: R21 with the city, as of the day; the city has a code, so the step names it.
    let a = answer_of(&compose_for(&w, "lahore ki ugrahi ka print", Language::En).expect("local"))
        .clone();
    assert_eq!(button(&a, 0).0, ReportId::R21);
    match &a.sources[0] {
        AnswerSource::Report { filters, .. } => {
            assert_eq!(
                filters.city.as_ref().map(|c| c.name_en.as_str()),
                Some("Lahore")
            );
            assert_eq!(filters.as_of, "2026-10-10");
        }
        other => panic!("{other:?}"),
    }
    assert_eq!(
        text_arg(&a, "assistant-how-set-party", "role").as_deref(),
        Some("city")
    );
    agree(&a);
}

/// The links carry only the filters the screens read (lead ruling of 04-11-2026, checked by
/// frontend2 against the report screens): R10 has `from` and `to` and no `as_of`; R11 has `from` =
/// `to`; R20, R24 and R18 have `as_of`; R29 has the period and no expense head, so the steps say
/// which row to read.
#[test]
fn a_link_carries_only_the_filters_its_screen_reads() {
    let w = books();
    // (question, report, from, to, as_of)
    for (question, report, from, to, as_of) in [
        // Stock now: from the first day of the books to today.
        (
            "baqi maal ka print",
            ReportId::R10,
            "2026-08-31",
            "2026-10-10",
            "",
        ),
        (
            "roznamcha print aaj ka",
            ReportId::R11,
            "2026-10-10",
            "2026-10-10",
            "",
        ),
        ("dues ka print", ReportId::R20, "", "", "2026-10-10"),
        (
            "sellers ka payable print karo",
            ReportId::R24,
            "",
            "",
            "2026-10-10",
        ),
        (
            "trial balance print karo",
            ReportId::R18,
            "",
            "",
            "2026-10-10",
        ),
        (
            "bijli ka kharcha print pichle mahine ka",
            ReportId::R29,
            "2026-09-01",
            "2026-09-30",
            "",
        ),
    ] {
        let c = compose_for(&w, question, Language::En)
            .unwrap_or_else(|| panic!("no local answer for {question:?}"));
        let a = answer_of(&c);
        let (id, got_from, got_to, got_as_of, _) = button(a, 0);
        assert_eq!(id, report, "{question}");
        assert_eq!(
            (got_from.as_str(), got_to.as_str(), got_as_of.as_str()),
            (from, to, as_of),
            "{question}"
        );
    }
    // R29 for one expense head: the step names the row, in both scripts.
    let c =
        compose_for(&w, "bijli ka kharcha print pichle mahine ka", Language::En).expect("local");
    let a = answer_of(&c);
    let step = a
        .how
        .iter()
        .find(|s| s.id == "assistant-how-find-row")
        .expect("a step that names the row");
    match &step.args.first().expect("an argument").value {
        ArgValue::Names { names } => {
            assert_eq!(names.en, "Electricity");
            assert_eq!(names.ur, "بجلی");
        }
        other => panic!("{other:?}"),
    }
    assert_eq!(step.args.first().map(|x| x.name.as_str()), Some("name"));
}

#[test]
fn a_help_answer_is_the_guides_own_words_in_the_app_language() {
    let w = books();
    for language in [Language::Ur, Language::En] {
        let c = compose_for(&w, "wasooli kaise likhun", language).expect("a local answer");
        assert_eq!(c.outcome, AskOutcome::Answered);
        let a = answer_of(&c);
        let words = guide_text("receipt", language);
        let texts: Vec<&str> = a
            .parts
            .iter()
            .filter_map(|p| match p {
                AnswerPart::Text { text } => Some(text.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(texts.first().copied(), Some(words.summary.as_str()));
        for (at, step) in words.steps.iter().enumerate() {
            assert_eq!(
                texts.get(at + 1).copied(),
                Some(format!("\n{}. {step}", at + 1).as_str())
            );
        }
        assert!(texts.iter().any(|t| t.contains(words.example.as_str())));
        assert!(a.sources.is_empty() && a.how.is_empty() && a.tables.is_empty());
    }
    // A topic of the guide for every how-to the guard knows.
    for question in [
        "bikri kaise add karun",
        "market safar kya hai",
        "backup kaise lun",
        "print kaise nikalte hain",
        "what is pukhta khata",
    ] {
        assert!(
            compose_for(&w, question, Language::En).is_some(),
            "{question}"
        );
    }
}

#[test]
fn opening_a_screen_or_a_report_is_a_route_or_a_button() {
    let w = books();
    let c = compose_for(&w, "dashboard kholo", Language::En).expect("local");
    assert_eq!(
        c.entry_route.as_ref().map(|r| r.screen),
        Some(EntryScreen::Dashboard)
    );
    assert!(c.entry_route.as_ref().is_some_and(|r| r.party.is_none()));
    let c = compose_for(&w, "go to the backup screen", Language::En).expect("local");
    assert_eq!(
        c.entry_route.as_ref().map(|r| r.screen),
        Some(EntryScreen::Backup)
    );
    // A party's khata: the button, and the steps without "press print".
    let c = compose_for(&w, "faisal ka khata kholo", Language::En).expect("local");
    let a = answer_of(&c);
    assert_eq!(button(a, 0).0, ReportId::R13);
    assert_eq!(
        step_ids(a),
        ["assistant-how-open-report", "assistant-how-set-party"]
    );
    assert!(c.entry_route.is_none());
}

#[test]
fn a_name_the_books_do_not_have_is_not_found_with_the_role_asked() {
    let w = books();
    let c = compose_for(&w, "zubair ne kya becha", Language::En).expect("local");
    assert_eq!(c.outcome, AskOutcome::NotFound);
    assert!(c.answer.is_none() && c.found_as.is_none());
    assert_eq!(c.asked_role, Some(AskedRole::Seller));

    let c = compose_for(&w, "zubair ka bakaya kitna hai", Language::Ur).expect("local");
    assert_eq!(c.asked_role, Some(AskedRole::Any));

    // The name exists, as a buyer: found_as names him with his role.
    let c = compose_for(&w, "faisal ne kya becha", Language::En).expect("local");
    assert_eq!(c.outcome, AskOutcome::NotFound);
    assert_eq!(c.asked_role, Some(AskedRole::Seller));
    let found = c.found_as.expect("found as");
    assert_eq!(found.role, AskedRole::Buyer);
    assert_eq!(found.name.name_en, "Faisal");
    assert!(found.key.starts_with('a') && !found.name.name_ur.is_empty());

    let c = compose_for(&w, "rahim ne kya khareeda", Language::En).expect("local");
    assert_eq!(c.asked_role, Some(AskedRole::Buyer));
    assert_eq!(c.found_as.map(|f| f.role), Some(AskedRole::Seller));

    // A print for a name that is not there: nothing to print.
    let c = compose_for(&w, "ahmad ki aaj ki bikri ka print", Language::En).expect("local");
    assert_eq!(c.outcome, AskOutcome::NotFound);

    // An item the books do not hold.
    let c = compose_for(&w, "kinnu ka sale kitna hai", Language::En).expect("local");
    assert_eq!(c.asked_role, Some(AskedRole::Item));
}

#[test]
fn a_question_outside_the_books_says_so_with_the_nearest_thing_the_app_keeps() {
    let w = books();
    let words = Words::new().expect("words");
    let table: [(&str, OutsideKind, Option<ReportId>); 7] = [
        ("weather kaisa hai", OutsideKind::Weather, None),
        ("gst kitna lagta hai", OutsideKind::Tax, Some(ReportId::R2)),
        (
            "bank balance kitna hai",
            OutsideKind::Bank,
            Some(ReportId::R12),
        ),
        (
            "bank mein kitne paise hain",
            OutsideKind::Bank,
            Some(ReportId::R12),
        ),
        (
            "lahore mein aaj kele ka rate kya hai",
            OutsideKind::MarketRate,
            Some(ReportId::R2),
        ),
        ("dollar ka rate kya hai", OutsideKind::Prices, None),
        (
            "mandi ka rate kya hai",
            OutsideKind::MarketRate,
            Some(ReportId::R2),
        ),
    ];
    for (question, kind, nearest) in table {
        for language in [Language::Ur, Language::En] {
            let c = compose_for(&w, question, language)
                .unwrap_or_else(|| panic!("no local answer for {question:?}"));
            assert_eq!(c.outcome, AskOutcome::OutsideBooks, "{question}");
            let a = answer_of(&c);
            assert_eq!(
                a.parts,
                [AnswerPart::Text {
                    text: words.text(language, outside_message(kind))
                }],
                "{question}"
            );
            match nearest {
                Some(report) => assert_eq!(button(a, 0).0, report, "{question}"),
                None => assert!(a.sources.is_empty(), "{question}"),
            }
        }
    }
    // Anything else the app does not keep: the dashboard.
    let c = compose_for(&w, "cricket match kab hai", Language::En).expect("local");
    assert!(matches!(
        answer_of(&c).sources.first(),
        Some(AnswerSource::Tiles { .. })
    ));
}

#[test]
fn figures_lists_advice_and_what_is_not_sure_go_on_to_the_model_path() {
    let w = books();
    for question in [
        "faisal ka bakaya kitna hai",
        "kele ka commission is mahine",
        "faisal ne kya kya khareeda",
        "sale kaise barhayen",
        "agle mahine commission kitna hoga",
        "hello",
        "",
    ] {
        assert!(
            compose_for(&w, question, Language::En).is_none(),
            "{question:?}"
        );
    }
}

/// What a refusal opens: the screen and the party chosen on it (its English name, and whether it
/// has a code: an account does, a partner does not).
type Opened = (EntryScreen, Option<(String, bool)>);

fn route(c: &Composed) -> Opened {
    assert_eq!(c.outcome, AskOutcome::ChangeRefused);
    assert!(c.answer.is_none(), "the panel owns the words of a refusal");
    assert!(c.asked_role.is_none() && c.found_as.is_none());
    let route = c.entry_route.as_ref().expect("a route");
    (
        route.screen,
        route
            .party
            .as_ref()
            .map(|p| (p.name_en.clone(), p.code.is_some())),
    )
}

/// An account as the route carries it, and a partner.
fn account(name: &str) -> Option<(String, bool)> {
    Some((name.to_owned(), true))
}

fn partner(name: &str) -> Option<(String, bool)> {
    Some((name.to_owned(), false))
}

/// Live run 2 (G2): "بکری بک کی اندراج کرنی ہے" and "record the journal" are requests to write, refused with
/// the screen that does it (the sale book, the journal), not answered with the guide.
#[test]
fn an_entry_request_in_the_clerks_words_is_refused_with_its_screen() {
    let w = books();
    for (question, screen) in [
        ("بکری بک کی اندراج کرنی ہے", EntryScreen::Bikri),
        ("bikri ki indraaj karni hai", EntryScreen::Bikri),
        ("record the journal", EntryScreen::General),
        ("جرنل کی اندراج کرنی ہے", EntryScreen::General),
    ] {
        for language in [Language::Ur, Language::En] {
            let c = compose_for(&w, question, language)
                .unwrap_or_else(|| panic!("no local answer for {question:?}"));
            let (got, party) = route(&c);
            assert_eq!(got, screen, "{question}");
            assert!(party.is_none(), "{question}");
        }
    }
}

/// Plan 55 open point 4 (lead, 04-11-2026): a request to write is refused locally when the question
/// is understood, with the entry screen that does it and the party as a hint. Nothing else of the
/// question (no amount) reaches the route (PRD FR-7.53, rule 11).
#[test]
fn a_request_to_write_is_refused_with_the_entry_screen_and_the_party_as_a_hint() {
    let w = books();
    let akram = account("Akram Toll Tax");
    let faisal = account("Faisal");
    let rahim = account("Rahim");
    for (question, screen, who) in [
        // The screen is named, and so is the party.
        (
            "akram ki 5000 wasooli likh do",
            EntryScreen::Wasooli,
            akram.clone(),
        ),
        ("اکرم کی وصولی لکھ دو", EntryScreen::Wasooli, akram.clone()),
        ("wasooli likh do", EntryScreen::Wasooli, None),
        (
            "rahim ko 15000 peshgi de do",
            EntryScreen::Peshgi,
            rahim.clone(),
        ),
        (
            "rahim ko 4316 adaigi likh do",
            EntryScreen::Adaigi,
            rahim.clone(),
        ),
        // An expense head is not a party; a partner is (a partner record, with no code).
        ("bijli ka kharcha 3000 likh do", EntryScreen::Kharcha, None),
        (
            "partner a ka zati kharcha 2000 likh do",
            EntryScreen::Kharcha,
            partner("Partner A"),
        ),
        (
            "partner b ka sarmaya 50000 jama karo",
            EntryScreen::Sarmaya,
            partner("Partner B"),
        ),
        ("cancel bikri 5", EntryScreen::BikriCancel, None),
        // The verb and the party say it when no screen is named.
        ("akram ne 5000 diye", EntryScreen::Wasooli, akram.clone()),
        (
            "akram ke 5000 jama karo",
            EntryScreen::Wasooli,
            akram.clone(),
        ),
        // A payment to a buyer is cash lent (PRD FR-15.11).
        ("faisal ko 2000 de do", EntryScreen::Adaigi, faisal.clone()),
        ("delete akram", EntryScreen::Account, akram.clone()),
        // A name the books do not have: the screen, and nobody chosen.
        ("zubair ko 5000 peshgi de do", EntryScreen::Peshgi, None),
    ] {
        for language in [Language::Ur, Language::En] {
            let c = compose_for(&w, question, language)
                .unwrap_or_else(|| panic!("{question:?} was not refused locally"));
            assert_eq!(route(&c), (screen, who.clone()), "{question:?}");
            // No amount, no number of any kind, reaches the reply.
            assert!(
                !format!("{c:?}").contains("5000") && !format!("{c:?}").contains("15000"),
                "{question:?}: an amount in the reply"
            );
        }
    }
}

/// A person who is lent money (a staff member) and a separate khata are accounts of their own kind:
/// the route carries them on their own screens only.
#[test]
fn a_loan_a_khata_and_a_main_screen_open_the_right_screen() {
    let w = books();
    world::add_account(
        &w.env,
        &w.owner,
        AccountType::MoneyLent,
        "اسلم",
        "Aslam",
        true,
        "",
    );
    for (question, screen, who) in [
        (
            "aslam ko 5000 udhar de do",
            EntryScreen::UdharRaqam,
            account("Aslam"),
        ),
        // A buyer is not a person lent money: the screen opens with nobody chosen.
        (
            "akram ko 5000 udhar raqam likh do",
            EntryScreen::UdharRaqam,
            None,
        ),
        // A main screen opens as it is and takes no party.
        (
            "commission rule 5 percent badal do",
            EntryScreen::CommissionRules,
            None,
        ),
    ] {
        let c = compose_for(&w, question, Language::En)
            .unwrap_or_else(|| panic!("{question:?} was not refused locally"));
        assert_eq!(route(&c), (screen, who), "{question:?}");
    }
}

/// A request to write that does not say which screen, or that also asks to read, is for the model
/// path, which can ask or answer both. A figure is never turned into a refusal.
#[test]
fn a_request_that_is_unclear_or_also_asks_something_goes_to_the_model_path() {
    let w = books();
    for question in [
        // Receipt, capital or a journal line: the question does not say.
        "5000 jama karo",
        // Two people.
        "akram aur arshad ko 2000 de do",
        // A question and an instruction in one.
        "akram ka bakaya batao aur 5000 jama karo",
        "akram ki wasooli ka print nikalo aur likh do",
        // Only a figure.
        "akram ka bakaya kitna hai",
        "akram ne kitne paise diye",
    ] {
        let c = compose_for(&w, question, Language::En);
        assert!(
            c.as_ref()
                .is_none_or(|c| c.outcome != AskOutcome::ChangeRefused),
            "{question:?}: {c:?}"
        );
    }
}

/// The message ids the panel must have in both languages. They are written by frontend2 and reviewed
/// (NFR-URD-4) in `frontend/src/i18n/*/assistant.ftl`.
#[test]
fn the_new_messages_exist_in_both_languages() {
    let words = Words::new().expect("words");
    let mut missing = Vec::new();
    for kind in [
        OutsideKind::Weather,
        OutsideKind::Tax,
        OutsideKind::Bank,
        OutsideKind::MarketRate,
        OutsideKind::Prices,
        OutsideKind::Other,
    ] {
        let id = outside_message(kind);
        for language in [Language::Ur, Language::En] {
            if words.text(language, id) == id {
                missing.push(format!("{id} ({language:?})"));
            }
        }
    }
    for id in ["assistant-how-set-record", "assistant-how-find-row"] {
        for language in [Language::Ur, Language::En] {
            if words.text(language, id) == id {
                missing.push(format!("{id} ({language:?})"));
            }
        }
    }
    assert!(missing.is_empty(), "missing: {missing:?}");
}

fn compose_locked(w: &world::World, question: &str, locked: bool) -> Option<Composed> {
    let reader = Reader {
        ctx: &w.env.ctx,
        actor: &w.owner,
        language: Language::En,
        today: BusinessDate::from_ymd(2026, 10, 10).expect("a date"),
        partner_locked: locked,
    };
    let words = Words::new().expect("the words");
    compose(
        &understand(&reader, question),
        Language::En,
        &words,
        READ_AT,
    )
}

/// Backend's review R3: "no such party" is said without the model only when the name is in no account
/// at all (an inactive one is a real account with dues), is no known one-time person of the پختہ
/// کھاتہ (شاہد, Event 2), and the partner section is open (a locked section hides the partners'
/// names). Otherwise the model looks further.
#[test]
fn no_such_party_is_said_only_when_the_name_is_in_no_account_person_or_partner() {
    let w = books();
    // Event 2 of the worked day: the one-time buyer شاہد is recorded on the پختہ کھاتہ.
    worked::events_1_to_5(&w);
    let zafar = world::add_account(
        &w.env,
        &w.owner,
        AccountType::Buyer,
        "ظفر",
        "Zafar",
        true,
        "",
    );
    mandi_services::accounts::set_active(&w.env.ctx, &w.owner, zafar, false).expect("inactive");
    let not_found = |c: &Option<Composed>| {
        c.as_ref()
            .is_some_and(|c| c.outcome == AskOutcome::NotFound)
    };
    // Really missing: still answered locally.
    assert!(not_found(&compose_locked(&w, "zubair ka bakaya", false)));
    // An inactive buyer, a one-time person: not missing.
    for question in ["zafar ka bakaya", "shahid ka bakaya"] {
        let c = compose_locked(&w, question, false);
        assert!(!not_found(&c), "{question}: {c:?}");
    }
    // A locked partner section: a name may be a partner's.
    for question in ["usman ka bakaya", "zubair ka bakaya"] {
        let c = compose_locked(&w, question, true);
        assert!(!not_found(&c), "{question}: {c:?}");
    }
}

/// Backend's review R4: one missing name must not end a question that names a found one too. The
/// figure of the found one would be lost.
#[test]
fn one_missing_name_does_not_end_a_question_that_names_a_found_one() {
    let w = books();
    for question in [
        "akram aur zubair ka bakaya",
        "zubair aur akram ka bakaya",
        "kele aur kinnu ka commission",
    ] {
        let c = compose_locked(&w, question, false);
        assert!(c.is_none(), "{question}: {c:?}");
    }
    // Every name missing, or just the one: answered.
    assert!(compose_locked(&w, "zubair ka bakaya", false).is_some());
}

/// Backend's review R2: two accounts with one name, a buyer's and a seller's, and only a soft hint
/// (adaigi, dues) in the question: the app is not sure which. A payment to a buyer is cash lent
/// (PRD FR-15.11), so choosing the seller could be the wrong one.
#[test]
fn a_soft_word_does_not_choose_between_a_buyer_and_a_seller_of_one_name() {
    let w = books();
    world::add_account(
        &w.env,
        &w.owner,
        AccountType::Seller,
        "فیصل",
        "Faisal",
        true,
        "",
    );
    let reader = Reader {
        ctx: &w.env.ctx,
        actor: &w.owner,
        language: Language::En,
        today: BusinessDate::from_ymd(2026, 10, 10).expect("a date"),
        partner_locked: false,
    };
    for question in ["faisal ki adaigi", "faisal ke dues", "faisal ka payable"] {
        let u = understand(&reader, question);
        assert!(!u.sure, "{question}: {u:?}");
    }
    // A plain word still settles it.
    let u = understand(&reader, "faisal seller ka bakaya");
    assert!(u.sure, "{u:?}");
}

/// A question that is not sure never has a button or steps: it goes to the model path. A wrong,
/// wider date on a button is worse than none (lead ruling of 04-11-2026).
#[test]
fn a_question_that_is_not_sure_never_has_a_button_or_steps() {
    use mandi_assistant::understand::{compose_help, compose_navigate, compose_print};
    let w = books();
    let reader = Reader {
        ctx: &w.env.ctx,
        actor: &w.owner,
        language: Language::En,
        today: BusinessDate::from_ymd(2026, 10, 10).expect("a date"),
        partner_locked: false,
    };
    for question in [
        // Two periods: a comparison.
        "faisal ki bikri ka print pichle mahine ka aur is mahine ka",
        "faisal ki pichle mahine aur is mahine ki 1 se 15 tak ki bikri ka print",
        // Nothing to print.
        "print",
        // Two parties.
        "faisal aur akram ki bikri ka print",
    ] {
        let u = understand(&reader, question);
        assert!(!u.sure, "{question}: {u:?}");
        assert!(
            u.report.is_none(),
            "{question}: a button for an unsure question"
        );
        assert!(
            compose_print(&u, Language::En, READ_AT).is_none(),
            "{question}"
        );
        assert!(
            compose_navigate(&u, Language::En, READ_AT).is_none(),
            "{question}"
        );
        assert!(
            compose_help(&u, Language::En, READ_AT).is_none(),
            "{question}"
        );
        let words = Words::new().expect("words");
        assert!(
            compose(&u, Language::En, &words, READ_AT).is_none(),
            "{question}"
        );
    }
}

// ---- the money review of 04-11-2026 (BUG-1 to BUG-5, R-a to R-i) ---------------------------------------

fn ask(w: &world::World, question: &str, locked: bool) -> Understanding {
    let reader = Reader {
        ctx: &w.env.ctx,
        actor: &w.owner,
        language: Language::En,
        today: BusinessDate::from_ymd(2026, 10, 10).expect("a date"),
        partner_locked: locked,
    };
    understand(&reader, question)
}

/// The question is the model's: not sure, so no button and no steps, no local answer and no reads.
fn assert_model_path(u: &Understanding, question: &str) {
    assert!(!u.sure, "{question}: {u:?}");
    assert!(
        u.report.is_none(),
        "{question}: a button for an unsure question: {:?}",
        u.report
    );
    let words = Words::new().expect("words");
    assert!(
        compose(u, Language::En, &words, READ_AT).is_none(),
        "{question}"
    );
    assert!(plan_reads(u).is_none(), "{question}");
}

/// BUG-1. A print for a name the app cannot place (a one-time person of the پختہ کھاتہ, an inactive
/// account, a partner while the section is locked) must never become a print of the whole shop.
/// "shahid ki aaj ki bikri ka print" was R34 for the whole day.
#[test]
fn a_print_for_a_name_the_app_cannot_place_never_opens_a_report_of_the_whole_shop() {
    let w = books();
    // Event 2 of the worked day: the one-time buyer شاہد is recorded on the پختہ کھاتہ.
    worked::events_1_to_5(&w);
    let zafar = world::add_account(
        &w.env,
        &w.owner,
        AccountType::Buyer,
        "ظفر",
        "Zafar",
        true,
        "",
    );
    mandi_services::accounts::set_active(&w.env.ctx, &w.owner, zafar, false).expect("inactive");
    for question in [
        "shahid ki aaj ki bikri ka print",
        "shahid ka bill ka print aaj ka",
        "zafar ki pichle mahine ki bikri ka print",
        "zafar ki aaj ki bikri ka print",
    ] {
        assert_model_path(&ask(&w, question, false), question);
    }
    // A locked partner section: "usman" may be a partner. R20 (every buyer's dues) is no answer.
    let u = ask(&w, "usman ki wasooli ka print", true);
    assert_model_path(&u, "usman ki wasooli ka print");
    // The same question with no name is a print of the dues: the name is what blocks it.
    let u = ask(&w, "wasooli ka print", false);
    assert!(u.sure, "{u:?}");
    // A name that is placed still prints.
    let c = compose_for(&w, "faisal ki aaj ki bikri ka print", Language::En).expect("a print");
    assert_eq!(button(answer_of(&c), 0).0, ReportId::R35);
}

/// BUG-2. A print drops the item and the scope word: "kele ki aaj ki bikri ka print" was R34 for all
/// items. It is now R7 with the item filter (W3); own goods and the guests' sales have no report, so
/// such a print is the model's (a garden or a trip word alone is the report of all
/// gardens or all trips).
#[test]
fn a_print_that_names_an_item_or_a_scope_the_links_cannot_filter_is_not_sure() {
    let w = books();
    for question in [
        "apne maal ki is mahine ki bikri ka print",
        "apna maal aaj ka print",
        "mehman ki aaj ki bikri ka print",
    ] {
        assert_model_path(&ask(&w, question, false), question);
    }
    // Without the item or the scope word they print as before. A garden or a trip word with no garden
    // or trip named is the gardens or the trips report of the days (R36, R33): they hold the sales.
    for (question, report) in [
        // An item named: the item-wise sales (R7) or the stock (R10) with the item filter.
        ("kele ki aaj ki bikri ka print", "R7"),
        ("faisal ki kele ki aaj ki bikri ka print", "R7"),
        ("kele ka baqi maal print", "R10"),
        ("bagh ki is mahine ki bikri ka print", "R36"),
        ("safar ki aaj ki bikri ka print", "R33"),
        ("baqi maal ka print", "R10"),
        ("aaj ki bikri ka print", "R34"),
        ("faisal ki aaj ki bikri ka print", "R35"),
    ] {
        let u = ask(&w, question, false);
        assert!(u.sure, "{question}: {u:?}");
        assert_eq!(u.report.as_ref().map(|r| r.id), Some(report), "{question}");
    }
}

/// BUG-3. "kal tak" is the books up to and including yesterday, and the button, the steps and the
/// dates all say so.
#[test]
fn a_print_up_to_a_day_is_the_books_up_to_that_day() {
    let w = books();
    let c = compose_for(&w, "faisal ki kal tak ki bikri ka print", Language::En)
        .expect("a local answer");
    let a = answer_of(&c);
    let (id, from, to, _, party) = button(a, 0);
    assert_eq!(
        (id, from.as_str(), to.as_str(), party.as_deref()),
        (ReportId::R5, "2026-08-31", "2026-10-09", Some("Faisal"))
    );
    agree(a);
    let c = compose_for(
        &w,
        "akram ki 30 september tak ki bikri ka print",
        Language::En,
    )
    .expect("a local answer");
    let a = answer_of(&c);
    let (id, from, to, _, party) = button(a, 0);
    assert_eq!(
        (id, from.as_str(), to.as_str(), party.as_deref()),
        (
            ReportId::R5,
            "2026-08-31",
            "2026-09-30",
            Some("Akram Toll Tax")
        )
    );
    agree(a);
}

/// BUG-4. A unit of time alone ("mahine ki") is this one; a time word the app cannot place
/// ("pichli", "hafte mein") makes the print the model's.
#[test]
fn a_time_word_alone_is_this_month_or_the_print_is_not_sure() {
    let w = books();
    let c = compose_for(&w, "faisal ki mahine ki bikri ka print", Language::En)
        .expect("a local answer");
    let a = answer_of(&c);
    let (id, from, to, _, _) = button(a, 0);
    assert_eq!(
        (id, from.as_str(), to.as_str()),
        (ReportId::R5, "2026-10-01", "2026-10-10")
    );
    agree(a);
    for question in [
        "faisal ki pichli bikri ka print",
        "faisal ki bikri ka print hafte mein",
        "faisal ki do mahine ki bikri ka print",
        "faisal ki har hafte ki bikri ka print",
    ] {
        let u = ask(&w, question, false);
        assert_model_path(&u, question);
        assert!(u.doubts.contains(&Doubt::TimeWord), "{question}: {u:?}");
    }
}

/// BUG-5. "apne maal" is the right grammar for "my own goods"; it scopes a figure like "apna maal".
#[test]
fn own_goods_in_the_right_grammar_is_a_scope_word() {
    let w = books();
    for question in [
        "apne maal ki is mahine ki bikri kitni hai",
        "apna maal is mahine ki bikri",
        "hamare apne mal ki bikri aaj",
        "اپنے مال کی بکری آج",
    ] {
        let u = ask(&w, question, false);
        assert!(u.scoped, "{question}: {u:?}");
        assert!(plan_reads(&u).is_none(), "{question}");
    }
    // A word for goods alone is not a scope.
    let u = ask(&w, "baqi maal kitna hai", false);
    assert!(!u.scoped, "{u:?}");
}

/// R-a. A misspelt name must not become another person: "nasir" is one consonant off Nasim. The
/// same name written with other vowels is still the person.
#[test]
fn a_name_one_consonant_off_is_not_taken_for_another_person() {
    let w = books();
    world::add_account(
        &w.env,
        &w.owner,
        AccountType::Buyer,
        "نسیم",
        "Nasim",
        true,
        "",
    );
    for question in [
        "nasir ka bakaya",
        "nasir ki aaj ki bikri ka print",
        "nasir ki aaj ki bikri",
    ] {
        let u = ask(&w, question, false);
        assert_model_path(&u, question);
        assert!(
            u.doubts.contains(&Doubt::CloseSpelling),
            "{question}: {:?}",
            u.doubts
        );
    }
    for question in ["nsim ka bakaya", "nasem ka bakaya", "nasim ka bakaya"] {
        let u = ask(&w, question, false);
        assert!(u.sure, "{question}: {u:?}");
        assert!(plan_reads(&u).is_some(), "{question}");
    }
}

/// R-g. A print of a party says what was asked: a receipt, an advance or a payment is in his khata
/// (R13), not in the sale reports (R5, R6) that would have opened.
#[test]
fn a_print_of_a_party_opens_the_report_that_holds_the_figure_asked() {
    let w = books();
    for (question, party) in [
        ("rahim ki is mahine ki peshgi ka print", "Rahim"),
        ("akram ki is mahine ki wasooli ka print", "Akram Toll Tax"),
        ("karim ki is mahine ki adaigi ka print", "Karim"),
    ] {
        let c = compose_for(&w, question, Language::En).expect("a local answer");
        let a = answer_of(&c);
        let (id, from, to, _, who) = button(a, 0);
        assert_eq!(
            (id, from.as_str(), to.as_str(), who.as_deref()),
            (ReportId::R13, "2026-10-01", "2026-10-10", Some(party)),
            "{question}"
        );
        agree(a);
    }
    // The sale reports still open for a sale.
    let c = compose_for(&w, "rahim ki is mahine ki bikri ka print", Language::En)
        .expect("a local answer");
    assert_eq!(button(answer_of(&c), 0).0, ReportId::R6);
    // A figure no report of a party holds is the model's.
    let u = ask(&w, "akram ka cash ka print aaj ka", false);
    assert!(u.report.is_none(), "{u:?}");
}

/// R-i. A real party is not declared absent for a spelling variant the matcher would not take
/// (اکرام for اکرم, PRD FR-7.40): the model looks further.
#[test]
fn a_real_party_is_not_declared_absent_for_a_spelling_variant() {
    let w = books();
    for question in ["اکرام کا بقایا", "اکرام کی آج کی بکری کا پرنٹ"]
    {
        let u = ask(&w, question, false);
        assert!(!u.has_not_found(), "{question}: {u:?}");
        let c = compose_locked(&w, question, false);
        assert!(
            !c.as_ref()
                .is_some_and(|c| c.outcome == AskOutcome::NotFound),
            "{question}: {c:?}"
        );
    }
    // A name that is really missing is still said to be missing.
    let c = compose_locked(&w, "zubair ka bakaya", false);
    assert!(c.is_some_and(|c| c.outcome == AskOutcome::NotFound));
}

/// One line for each question of the file in `SURVEY`: what the app understood and what it would
/// answer without a model. A tool for the person who builds, not a check:
/// `WORLD=full SURVEY=/path cargo test -p mandi-assistant --test understand_compose -- --ignored
/// --nocapture survey_local`. `WORLD` is `worked`, `year` (thin) or `full`; `LANG=ur` the Urdu app.
#[test]
#[ignore = "a tool for the person who builds, not a check"]
fn survey_local() {
    let Ok(file) = std::env::var("SURVEY") else {
        return;
    };
    let language = if std::env::var("LANG").as_deref() == Ok("ur") {
        Language::Ur
    } else {
        Language::En
    };
    let text = std::fs::read_to_string(file).expect("the file");
    let world = std::env::var("WORLD").unwrap_or_default();
    let y;
    let (w, today) = match world.as_str() {
        "year" | "full" => {
            y = if world == "full" {
                year::seed::build()
            } else {
                year::seed::build_thin()
            };
            (&y.w, BusinessDate::from_ymd(2026, 11, 5).expect("a date"))
        }
        _ => {
            let b = Box::leak(Box::new(books()));
            worked::events_1_to_5(b);
            (&*b, BusinessDate::from_ymd(2026, 10, 10).expect("a date"))
        }
    };
    let words = Words::new().expect("the words");
    for question in text.lines().filter(|l| !l.trim().is_empty()) {
        let reader = Reader {
            ctx: &w.env.ctx,
            actor: &w.owner,
            language,
            today,
            partner_locked: false,
        };
        let u = understand(&reader, question);
        let local = compose(&u, language, &words, READ_AT);
        let names: Vec<String> = u
            .parties
            .iter()
            .map(|p| {
                format!(
                    "{}={}",
                    p.text,
                    match &p.outcome {
                        mandi_assistant::understand::Outcome::Found(h) =>
                            format!("{:?}:{}", h.role, h.name_en),
                        mandi_assistant::understand::Outcome::FoundAs(h) =>
                            format!("as {:?}:{}", h.role, h.name_en),
                        mandi_assistant::understand::Outcome::Several(v) =>
                            format!("several{}", v.len()),
                        mandi_assistant::understand::Outcome::NotFound => "NOTFOUND".to_owned(),
                    }
                )
            })
            .collect();
        println!(
            "{question:55} => {:?}{} {:?} | {:?} | {:?} | {:?} | local {}",
            u.intent.value,
            if u.sure { "" } else { " UNSURE" },
            u.figures.value,
            names,
            u.doubts,
            u.periods.iter().map(|p| p.kind).collect::<Vec<_>>(),
            local.map_or_else(
                || "none".to_owned(),
                |c| format!(
                    "{:?}{}",
                    c.outcome,
                    c.answer
                        .as_ref()
                        .map_or(String::new(), |a| format!(" how={}", a.how.len()))
                )
            )
        );
    }
}

// ---- the defects of live run 1 (lead's list of 04-11-2026, 30 questions) --------------------------------

fn year_ask(
    y: &world::World,
    question: &str,
) -> (
    Understanding,
    Option<Composed>,
    Option<Vec<mandi_assistant::understand::PlannedRead>>,
) {
    let reader = Reader {
        ctx: &y.env.ctx,
        actor: &y.owner,
        language: Language::Ur,
        today: BusinessDate::from_ymd(2026, 11, 5).expect("a date"),
        partner_locked: false,
    };
    let words = Words::new().expect("the words");
    let u = understand(&reader, question);
    let local = compose(&u, Language::Ur, &words, READ_AT);
    let reads = plan_reads(&u);
    (u, local, reads)
}

/// Group 1. A common word, a question word or a misspelt month is never a missing party: "aj ki
/// bikri kitny ki hui" said "no such party kitny" without a model. A local "no such party" is for a
/// token that is clearly a name.
#[test]
fn a_common_word_is_never_a_missing_party() {
    use mandi_assistant::understand::Outcome;
    let y = year::seed::build();
    let w = &y.w;
    for question in [
        "aj ki bikri kitny ki hui",
        "sab gaahkon ka bakaya kitna hai",
        "شراکت داروں کا آج کا منافع حصہ کتنا ہے",
        "شراکت داروں کے خرچے کا حصہ کتنا ہے",
        "پہلے کس سے وصولی کروں",
        "septmber ka comision kitna tha",
        "teeno partnron ka abhi tak ka kul raqam kitna he",
        "تینوں شراکت داروں کی کل رقم اب کتنی ہے",
        "dono market safr ka nafa ya nuqsan kitna raha",
        "دونوں مارکیٹ سفر کا نفع یا نقصان کتنا رہا",
        "udhar rakam kitni baqi hai aur kitni wapsi ki tareekh guzar chuki",
    ] {
        let (u, local, _) = year_ask(w, question);
        assert!(
            u.parties
                .iter()
                .all(|p| !matches!(p.outcome, Outcome::NotFound)),
            "{question}: a missing party in {:?}",
            u.parties
        );
        assert!(
            local
                .as_ref()
                .is_none_or(|c| c.outcome != AskOutcome::NotFound),
            "{question}: a local not_found"
        );
    }
    // What those words mean is read: the month, the figure, the advice.
    let (u, ..) = year_ask(w, "septmber ka comision kitna tha");
    assert!(
        u.periods.iter().any(|p| p.kind
            == mandi_assistant::understand::PeriodKind::Month {
                year: 2026,
                month: 9
            }),
        "{:?}",
        u.periods
    );
    let (u, ..) = year_ask(w, "aj ki bikri kitny ki hui");
    assert!(u.sure && u.figures.value == [mandi_assistant::understand::FigureKind::Sale]);
    let (u, ..) = year_ask(w, "پہلے کس سے وصولی کروں");
    assert_eq!(u.intent.value, mandi_assistant::understand::Intent::Advice);
    let (u, ..) = year_ask(w, "teeno partnron ka abhi tak ka kul raqam kitna he");
    assert_eq!(
        u.figures.value,
        [mandi_assistant::understand::FigureKind::PartnerTotal]
    );
    // A made-up name that stands where a name stands is still missing, said locally.
    let (_, local, _) = year_ask(w, "zubair ka bakaya kitna hai");
    assert!(local.is_some_and(|c| c.outcome == AskOutcome::NotFound));
}

/// Group 2. "what is each partner's total now" is a figure, and how to make the sales bigger is
/// advice, never help: "kaise ziyada / barhayen / behtar" is advice (Umar's own "apni sale ko ziyada
/// kaise karain").
#[test]
fn a_figure_or_advice_is_never_answered_with_help() {
    use mandi_assistant::understand::Intent;
    let y = year::seed::build();
    let w = &y.w;
    let (u, _, reads) = year_ask(w, "what is each partner's total now");
    assert_eq!(u.intent.value, Intent::List);
    assert!(u.sure && reads.is_some(), "{u:?}");
    for question in [
        "apni sale ko ziyada kaise karain ?",
        "اپنی بکری کو زیادہ کیسے کریں؟",
        "ہم اپنی بکری کو کیسے بڑھا سکتے ہیں؟",
        "bikri zyada kaise karen",
    ] {
        let (u, local, _) = year_ask(w, question);
        assert_eq!(u.intent.value, Intent::Advice, "{question}");
        assert!(local.is_none(), "{question}: a local answer");
    }
    // A word that is only explained is still help.
    let (u, ..) = year_ask(w, "wasooli kya hota hai");
    assert_eq!(u.intent.value, Intent::HowTo);
}

/// Group 3. A party's day or his items are read where the lines are: a buyer's day sheet (R35), a
/// seller's lots (R6), a buyer's items over a period (view A with the buyer). The answer then gives
/// each item with its qty and amount.
#[test]
fn a_partys_day_or_items_are_read_where_the_lines_are() {
    let y = year::seed::build();
    let w = &y.w;
    let planned = |question: &str| -> Vec<String> {
        let (_, _, reads) = year_ask(w, question);
        reads
            .unwrap_or_default()
            .iter()
            .filter(|r| r.tool == "report" || r.tool == "analytics")
            .map(|r| {
                let id = r.input["id"].as_str().or(r.input["view"].as_str());
                format!("{}:{}", r.tool, id.unwrap_or(""))
            })
            .collect()
    };
    for question in [
        "Imran Fruit ne 18 September ko kya aur kitna khareeda",
        "عمران فروٹ نے 18 ستمبر کو کیا اور کتنا خریدا",
        "what and how much did Imran Fruit buy on 18 September",
    ] {
        assert_eq!(planned(question), ["report:R35"], "{question}");
    }
    for question in [
        "کریم نے کل کیا اور کتنا بیچا",
        "Rahim ne 25 April ko kya kya becha aur kitna",
    ] {
        assert_eq!(planned(question), ["report:R6"], "{question}");
    }
    for question in [
        "Faisal ne kitne ka saman khareeda, kya kya khareeda, konsi item",
        "faisl ne kitne ka maal kharida kya kya kharida konsi item",
        "فیصل نے کتنے کا سامان خریدا، کیا کیا خریدا، کون سی آئٹم",
        "how much did Faisal buy, what all did he buy, which items",
    ] {
        assert_eq!(planned(question), ["analytics:A"], "{question}");
    }
    // Their figures are not an instant answer: the model writes the items from the tables.
    let reader = Reader {
        ctx: &w.env.ctx,
        actor: &w.owner,
        language: Language::En,
        today: BusinessDate::from_ymd(2026, 11, 5).expect("a date"),
        partner_locked: false,
    };
    let u = understand(&reader, "Rahim ne 25 April ko kya kya becha aur kitna");
    assert!(plan_reads(&u).is_some());
}

/// Group 4. A seller asked as a buyer is "not found, he is a seller"; a name with a word dropped a
/// double letter ("tol" for "toll") is the one buyer, not two; "bag" is a garden; two Akrams are
/// asked which one.
#[test]
fn roles_and_spellings_of_names_are_read_as_they_are() {
    use mandi_assistant::understand::{Outcome, PartyRole};
    let y = year::seed::build();
    let w = &y.w;
    let (u, local, _) = year_ask(w, "Karim ne kal kya khareeda aur kitna");
    assert!(
        matches!(&u.parties[..], [p] if matches!(&p.outcome, Outcome::FoundAs(h) if h.role == PartyRole::Seller)),
        "{:?}",
        u.parties
    );
    let local = local.expect("a local answer");
    assert_eq!(local.outcome, AskOutcome::NotFound);
    assert_eq!(local.asked_role, Some(AskedRole::Buyer));
    assert!(local.found_as.is_some_and(|f| f.role == AskedRole::Seller));
    // "tol" is "toll": one buyer, found, not "akram" (two) and "tol" (one).
    let (u, ..) = year_ask(w, "akram tol tax ka khata dikhao");
    assert!(
        matches!(&u.parties[..], [p] if matches!(&p.outcome, Outcome::Found(h) if h.name_en == "Akram Toll Tax")),
        "{:?}",
        u.parties
    );
    // "bag" is a garden.
    let (u, ..) = year_ask(w, "shahpur bag ka abhi kitna balance he");
    assert!(
        matches!(&u.parties[..], [p] if matches!(&p.outcome, Outcome::Found(h) if h.role == PartyRole::Garden)),
        "{:?}",
        u.parties
    );
    // Two Akrams in a print: which one, never the printing guide, never a button.
    let (u, local, _) = year_ask(w, "print the bill of Akram");
    assert!(
        u.parties
            .iter()
            .any(|p| matches!(p.outcome, Outcome::Several(_))),
        "{:?}",
        u.parties
    );
    assert!(!u.sure && u.report.is_none() && local.is_none());
}

/// The entry of a garden, a trip, or a saved بکری (the lead's order of 05-11-2026): the screen is the
/// new one, the garden is the party of a garden entry, and a trip word with no trip named opens the
/// trip screen with no party.
#[test]
fn a_garden_a_trip_or_a_saved_bikri_request_opens_its_own_screen_with_the_right_party() {
    let y = year::seed::build();
    let words = Words::new().expect("words");
    let route = |question: &str| {
        let u = ask(&y.w, question, false);
        compose(&u, Language::En, &words, READ_AT)
            .and_then(|c| c.entry_route)
            .unwrap_or_else(|| panic!("{question}: no route: {u:?}"))
    };
    for (question, screen) in [
        (
            "shahpur bagh ki qist likh do",
            EntryScreen::GardenInstallment,
        ),
        ("shahpur bagh ka kharcha likh do", EntryScreen::GardenCost),
        ("shahpur bagh band karo", EntryScreen::GardenClose),
    ] {
        let r = route(question);
        assert_eq!(r.screen, screen, "{question}");
        assert_eq!(
            r.party.as_ref().map(|p| p.name_en.as_str()),
            Some("Shahpur Garden"),
            "{question}"
        );
    }
    for (question, screen) in [
        ("safar ki khareedari likh do", EntryScreen::TripPurchase),
        ("safar ka kharcha likh do", EntryScreen::TripCost),
        (
            "safar ki doosri dukan mein bikri likh do",
            EntryScreen::TripSale,
        ),
        ("bikri 5 edit karo", EntryScreen::BikriEdit),
        ("baad mein bikri likh do", EntryScreen::BikriLaterSale),
    ] {
        let r = route(question);
        assert_eq!((r.screen, r.party), (screen, None), "{question}");
    }
}

/// An item the books do not hold is "no such item" (not_found, asked role item), as a party that does
/// not exist is; an item that is in the books with no sales has its own figure, 0, which is another
/// answer (tests/understand_figure.rs has that one). Names of two words, synonyms and Urdu letters all
/// say it.
#[test]
fn an_item_that_is_not_in_the_books_is_no_such_item() {
    use mandi_assistant::ask::AskedRole;
    let w = books();
    let words = Words::new().expect("words");
    for question in [
        "shimla mirch ki is mahine ki sale batao",
        "simla mirch ki is mahine ki sale batao",
        "capsicum ki is mahine ki sale batao",
        "شملہ مرچ کی اس مہینے کی بکری بتاؤ",
        "tamatar ka commission is mahine",
        "ٹماٹر کا کمیشن اس مہینے",
    ] {
        for language in [Language::En, Language::Ur] {
            let u = understand(
                &Reader {
                    ctx: &w.env.ctx,
                    actor: &w.owner,
                    language,
                    today: BusinessDate::from_ymd(2026, 10, 10).expect("a date"),
                    partner_locked: false,
                },
                question,
            );
            let composed = compose(&u, language, &words, READ_AT)
                .unwrap_or_else(|| panic!("{question}: no local answer, {u:?}"));
            assert_eq!(composed.outcome, AskOutcome::NotFound, "{question}");
            assert_eq!(composed.asked_role, Some(AskedRole::Item), "{question}");
        }
    }
}

/// Live run 2 (G1): 34 requests to open a screen, in Roman Urdu, Urdu and English ("take me to", "le
/// chalo", "لے چلیں", "show me the cash book", "wasooli screen dikhao"). Each is understood as opening
/// that screen, with the topic of the guide that describes it, and is answered locally with the route.
#[test]
fn a_request_to_open_a_screen_is_answered_with_its_topic_and_route() {
    let w = books();
    let reader = Reader {
        ctx: &w.env.ctx,
        actor: &w.owner,
        language: Language::En,
        today: BusinessDate::from_ymd(2026, 10, 10).expect("a date"),
        partner_locked: false,
    };
    let cases: &[(&str, &str, EntryScreen)] = &[
        ("take me to the sale book", "bikri_book", EntryScreen::Bikri),
        ("wasooli screen dikhao", "receipt", EntryScreen::Wasooli),
        ("مجھے وصولی پر لے چلیں", "receipt", EntryScreen::Wasooli),
        ("mujhe adaigi par le chalo", "payment", EntryScreen::Adaigi),
        ("پیشگی کھولیں", "advance", EntryScreen::Peshgi),
        ("kharcha kholo", "shop_expense", EntryScreen::Kharcha),
        (
            "مجھے ادھار رقم پر لے چلیں",
            "money_lent",
            EntryScreen::UdharRaqam,
        ),
        (
            "I want to open money lent",
            "money_lent",
            EntryScreen::UdharRaqam,
        ),
        ("سرمایہ کھولیں", "partner_capital", EntryScreen::Sarmaya),
        (
            "show me partner capital",
            "partner_capital",
            EntryScreen::Sarmaya,
        ),
        ("phal kholo", "fruit_taken", EntryScreen::Phal),
        ("take me to fruit taken", "fruit_taken", EntryScreen::Phal),
        (
            "مجھے جنرل پر لے چلیں",
            "general_journal",
            EntryScreen::General,
        ),
        ("index kholo", "masters", EntryScreen::Masters),
        ("take me to the index", "masters", EntryScreen::Masters),
        (
            "I want to open the accounts list",
            "accounts",
            EntryScreen::Accounts,
        ),
        (
            "mujhe commission rules par le chalo",
            "commission_rules",
            EntryScreen::CommissionRules,
        ),
        (
            "open commission rules",
            "commission_rules",
            EntryScreen::CommissionRules,
        ),
        (
            "show me today's sales list",
            "today_sales",
            EntryScreen::TodaySales,
        ),
        ("baqi maal kholo", "unsold_stock", EntryScreen::UnsoldStock),
        (
            "take me to unsold stock",
            "unsold_stock",
            EntryScreen::UnsoldStock,
        ),
        (
            "مجھے پختہ کھاتہ پر لے چلیں",
            "pukhta_khata",
            EntryScreen::PukhtaKhata,
        ),
        (
            "mujhe agrahi list par le chalo",
            "collection_list",
            EntryScreen::CollectionList,
        ),
        (
            "open the collection list",
            "collection_list",
            EntryScreen::CollectionList,
        ),
        ("روکڑ بک کھولیں", "cash_book", EntryScreen::CashBook),
        ("show me the cash book", "cash_book", EntryScreen::CashBook),
        ("take me to gardens", "gardens", EntryScreen::Gardens),
        (
            "مجھے مارکیٹ سفر پر لے چلیں",
            "market_trips",
            EntryScreen::MarketTrips,
        ),
        ("take me to day close", "day_close", EntryScreen::DayClose),
        ("مجھے رپورٹ پر لے چلیں", "reports", EntryScreen::Reports),
        (
            "mujhe doobi raqam par le chalo",
            "dead_amount",
            EntryScreen::DeadAmount,
        ),
        (
            "ڈوبی رقم والا صفحہ کھولنا ہے",
            "dead_amount",
            EntryScreen::DeadAmount,
        ),
        (
            "open the dead amount",
            "dead_amount",
            EntryScreen::DeadAmount,
        ),
        ("take me to backup", "backup", EntryScreen::Backup),
    ];
    assert!(cases.len() >= 34, "{}", cases.len());
    for (question, topic, screen) in cases {
        let u = understand(&reader, question);
        assert_eq!(
            u.topic.as_ref().map(|t| t.value),
            Some(*topic),
            "{question}: the topic"
        );
        for language in [Language::Ur, Language::En] {
            let c = compose_for(&w, question, language)
                .unwrap_or_else(|| panic!("{question:?} was not answered locally"));
            let got = c.entry_route.as_ref().map(|r| r.screen);
            assert_eq!(got, Some(*screen), "{question}: the route");
        }
    }
    // A figure asked with "show me" stays a figure: no screen opens for a name, a period or a figure
    // word alone.
    for question in [
        "cash dikhao",
        "akram ki wasooli dikhao",
        "aaj ki bikri dikhao",
    ] {
        let c = compose_for(&w, question, Language::En);
        assert!(
            c.as_ref().is_none_or(|c| c.entry_route.is_none()),
            "{question}: opened a screen"
        );
    }
}

/// Live run 2 (G9): a partner or a trip that is not in the books is "not found" with its role, never the
/// guide's words or the screen of all trips. "What is partner Zafar Iqbal's total" and "safar 99 ki report
/// dikhao" (no trip has number 99).
#[test]
fn an_unknown_partner_or_trip_is_not_found_with_its_role() {
    let w = books();
    for (question, role) in [
        ("what is partner Zafar Iqbal's total", AskedRole::Partner),
        ("partner Zafar Iqbal ki kul raqam", AskedRole::Partner),
        ("safar 99 ki report dikhao", AskedRole::Trip),
        ("safar 99 ka result", AskedRole::Trip),
    ] {
        for language in [Language::Ur, Language::En] {
            let c = compose_for(&w, question, language)
                .unwrap_or_else(|| panic!("{question:?} was not answered locally"));
            assert_eq!(c.outcome, AskOutcome::NotFound, "{question}");
            assert_eq!(c.asked_role, Some(role), "{question}");
        }
    }
}
