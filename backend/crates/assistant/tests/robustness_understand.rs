//! The "understand" step of plan 55, asked the questions of the robustness bank (offline, no model).
//!
//! `scripts/ai-eval/questions.tsv` holds 108 print questions (a buyer, a seller, no party, a khata,
//! for every period in words, in Roman Urdu, Urdu and English, and mixed), the two Akrams, a name
//! nobody has, and Umar's own print question. Their expectations (the report, the party in his real
//! role, the exact dates) come from the plan 54 table and from the oracle's calendar, which the app's
//! own period helper makes. This test runs the real `understand` over every one of them on the rich
//! year (today 05-11-2026) and compares. It is independent of `tests/understand.rs`, which the
//! builder wrote: here the bank is the judge.
//!
//! A second test asks the words that must win over the habit of "the latest month whose 15th has
//! come": "is mahine", "pichle mahine", "September", a day that has come.
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

use std::collections::BTreeMap;

use mandi_assistant::understand::{Intent, Outcome, PartyRole, Reader, Understanding, understand};
use mandi_services::core::auth::Language;
use mandi_services::core::dates::BusinessDate;

/// One row of the bank, with the columns the test needs.
struct Row {
    id: String,
    group: String,
    kind: String,
    question: String,
    outcome: String,
    filters: String,
    figures: String,
}

fn bank() -> Vec<Row> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("..")
        .join("scripts")
        .join("ai-eval")
        .join("questions.tsv");
    let text = std::fs::read_to_string(path).expect("the question bank");
    let mut lines = text.lines();
    let head: Vec<&str> = lines.next().expect("a header").split('\t').collect();
    let at = |name: &str| head.iter().position(|h| *h == name).expect(name);
    let (id, group, kind) = (at("id"), at("group"), at("kind"));
    let (question, outcome, filters, figures) =
        (at("question"), at("outcome"), at("filters"), at("figures"));
    lines
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            let c: Vec<&str> = l.split('\t').collect();
            Row {
                id: c[id].to_owned(),
                group: c[group].to_owned(),
                kind: c[kind].to_owned(),
                question: c[question].to_owned(),
                outcome: c[outcome].to_owned(),
                filters: c[filters].to_owned(),
                figures: c[figures].to_owned(),
            }
        })
        .collect()
}

/// The `print.` pieces of a figures text, with the `@cal.` facts filled in from the oracle.
fn print_want(figures: &str, facts: &BTreeMap<String, String>) -> BTreeMap<String, String> {
    figures
        .split(';')
        .filter_map(|piece| {
            let (label, value) = piece.split_once('=')?;
            let key = label.trim().strip_prefix("print.")?;
            let value = match value.trim().strip_prefix('@') {
                Some(fact) => facts
                    .get(fact)
                    .unwrap_or_else(|| panic!("no oracle fact {fact}"))
                    .clone(),
                None => value.trim().to_owned(),
            };
            Some((key.to_owned(), value))
        })
        .collect()
}

fn iso(d: chrono::NaiveDate) -> String {
    d.format("%Y-%m-%d").to_string()
}

fn role_word(role: PartyRole) -> &'static str {
    match role {
        PartyRole::Seller => "seller",
        PartyRole::Buyer => "buyer",
        PartyRole::Partner => "partner",
        PartyRole::Garden => "garden",
        PartyRole::Trip => "trip",
        PartyRole::Khata => "khata",
        PartyRole::Staff => "staff",
    }
}

/// What is wrong with the understanding of one print question, if anything.
fn print_problems(u: &Understanding, want: &BTreeMap<String, String>) -> Vec<String> {
    let mut problems = Vec::new();
    // An open request ("open the analytics for banana") is `print.intent=navigate`: the same plan, no press-print.
    let wanted_intent = if want.get("intent").is_some_and(|v| v == "navigate") {
        Intent::Navigate
    } else {
        Intent::Print
    };
    if u.intent.value != wanted_intent {
        problems.push(format!(
            "intent {:?}, wanted {wanted_intent:?}",
            u.intent.value
        ));
    }
    let Some(plan) = &u.report else {
        problems.push(format!("no report plan; doubts {:?}", u.doubts));
        return problems;
    };
    let wanted_reports = want.get("report").map_or("", String::as_str);
    if !wanted_reports.split('|').any(|r| r == plan.id) {
        problems.push(format!("report {}, wanted {wanted_reports}", plan.id));
    }
    // A print that names no period follows the app's rule (understand/plan.rs `dated`): a report of a
    // person or a thing opens on the whole books (`print.dates=none`, no dates), a report of a period
    // with no person takes today to today and says so (`print.assumed=1`). Any other print reads its period.
    let assumed_wanted = want.get("assumed").is_some_and(|v| v == "1");
    if plan.assumed_period != assumed_wanted {
        problems.push(if plan.assumed_period {
            "the period was assumed, not read".to_owned()
        } else {
            "the period should have been assumed (no period named), it was read".to_owned()
        });
    }
    let party_of = |key: &str| -> Option<(String, PartyRole)> {
        let hit = match key {
            "party" => plan.filters.party.as_ref(),
            _ => plan.filters.seller.as_ref(),
        }?;
        Some((hit.name_en.clone(), hit.role))
    };
    for key in ["party", "seller"] {
        match (want.get(key), party_of(key)) {
            (Some(name), Some((got, role))) => {
                if !got.eq_ignore_ascii_case(name) {
                    problems.push(format!("{key} {got}, wanted {name}"));
                }
                if let Some(wanted_role) = want.get("role")
                    && role_word(role) != wanted_role
                {
                    problems.push(format!(
                        "{name} is read as a {}, he is a {wanted_role}",
                        role_word(role)
                    ));
                }
            }
            (Some(name), None) => problems.push(format!("no {key}, wanted {name}")),
            (None, Some((got, _))) if key == "seller" => {
                problems.push(format!("a seller {got} on a report with no seller"));
            }
            _ => {}
        }
    }
    // An item, the shop's guests and one بکری are filters of the plan too (plan 55 W3, P1, R1).
    if let Some(item) = want.get("item") {
        let got = plan
            .filters
            .item
            .as_ref()
            .map(|i| i.name_en.clone())
            .unwrap_or_default();
        if !got.eq_ignore_ascii_case(item) {
            problems.push(format!("item {got:?}, wanted {item}"));
        }
    }
    if want.get("guests").is_some_and(|v| v == "1") && !plan.filters.guests {
        problems.push("the plan is not set to the guests".to_owned());
    }
    if let Some(number) = want.get("bikri") {
        let got = plan.filters.bikri.map(|(_, n)| n.to_string());
        if got.as_deref() != Some(number.as_str()) {
            problems.push(format!("bikri {got:?}, wanted {number}"));
        }
    }
    if let Some(day) = want.get("as_of") {
        let got = plan.filters.as_of.map(iso).unwrap_or_default();
        if &got != day {
            problems.push(format!("as_of {got:?}, wanted {day}"));
        }
    } else if want.get("dates").is_some_and(|v| v == "none") {
        for (key, got) in [
            ("from", plan.filters.from.map(iso)),
            ("to", plan.filters.to.map(iso)),
        ] {
            if let Some(got) = got {
                problems.push(format!("{key} {got:?}, wanted none (the whole books)"));
            }
        }
    } else {
        for (key, got) in [
            ("from", plan.filters.from.map(iso)),
            ("to", plan.filters.to.map(iso)),
        ] {
            let got = got.unwrap_or_default();
            if want.get(key) != Some(&got) {
                problems.push(format!("{key} {got:?}, wanted {:?}", want.get(key)));
            }
        }
    }
    problems
}

#[test]
fn every_print_question_of_the_bank_is_understood_as_the_bank_expects() {
    let y = year::seed::build();
    let facts = year::oracle::compute(&y).facts;
    let today = BusinessDate::from_ymd(2026, 11, 5).expect("today");
    let reader = Reader {
        ctx: &y.w.env.ctx,
        actor: &y.w.owner,
        language: Language::En,
        today,
        partner_locked: false,
    };
    let rows = bank();
    let mut checked = 0;
    let mut problems: Vec<String> = Vec::new();
    for row in &rows {
        let facet_print = row.filters.contains("facet=print") || row.group.starts_with("U08");
        let is_print_row = row.figures.contains("print.report=");
        if !(facet_print || is_print_row) {
            continue;
        }
        let u = understand(&reader, &row.question);
        if is_print_row {
            let want = print_want(&row.figures, &facts);
            let found = print_problems(&u, &want);
            checked += 1;
            if !found.is_empty() {
                problems.push(format!(
                    "{} {:?}: {}",
                    row.id,
                    row.question,
                    found.join("; ")
                ));
            }
        } else if row.filters.contains("pkind=ambiguous") {
            checked += 1;
            let several = u
                .parties
                .iter()
                .any(|p| matches!(p.outcome, Outcome::Several(_)));
            if !several || u.report.is_some() && u.sure {
                problems.push(format!(
                    "{} {:?}: two Akrams must be asked, never guessed (several: {several}, sure: {})",
                    row.id, row.question, u.sure
                ));
            }
        } else if row.filters.contains("pkind=nosuch") {
            checked += 1;
            if !u.has_not_found() || u.sure && u.report.is_some() {
                problems.push(format!(
                    "{} {:?}: a name nobody has must be not found, with no report",
                    row.id, row.question
                ));
            }
        }
        let _ = (&row.kind, &row.outcome);
    }
    assert!(
        checked >= 110,
        "only {checked} print questions were checked"
    );
    assert!(
        problems.is_empty(),
        "{} of {checked} print questions are not understood as the bank expects:\n{}",
        problems.len(),
        problems.join("\n")
    );
}

#[test]
fn a_range_with_no_month_is_the_latest_month_whose_15th_has_come_and_the_words_win_otherwise() {
    // Today is Thursday 05-11-2026. "1 se 15 tak" with no month cannot end in the future, so it is
    // 1 to 15 October. A word that says otherwise wins: a month named, "is mahine", "pichle mahine".
    // A range whose last day has come ("1 se 5 tak") is this month's.
    let y = year::seed::build_thin();
    let today = BusinessDate::from_ymd(2026, 11, 5).expect("today");
    let reader = Reader {
        ctx: &y.w.env.ctx,
        actor: &y.w.owner,
        language: Language::En,
        today,
        partner_locked: false,
    };
    let mut problems: Vec<String> = Vec::new();
    let range = |question: &str| -> Option<(String, String, bool)> {
        let u = understand(&reader, question);
        u.period().map(|p| (p.from_iso(), p.to_iso(), p.future))
    };
    for (question, from, to) in [
        // The latest month where the 15th has come.
        (
            "Faisal ki 1 se 15 tak ki bikri ka print",
            "2026-10-01",
            "2026-10-15",
        ),
        (
            "فیصل کی 1 سے 15 تک کی بکری کا پرنٹ",
            "2026-10-01",
            "2026-10-15",
        ),
        // The 5th has come: this month.
        (
            "Faisal ki 1 se 5 tak ki bikri ka print",
            "2026-11-01",
            "2026-11-05",
        ),
        // The month is named: the words win.
        (
            "Faisal ki 1 se 15 October tak ki bikri ka print",
            "2026-10-01",
            "2026-10-15",
        ),
        (
            "Faisal ki 1 se 15 September tak ki bikri ka print",
            "2026-09-01",
            "2026-09-15",
        ),
        (
            "print Faisal's sales from 1 to 15 October",
            "2026-10-01",
            "2026-10-15",
        ),
    ] {
        match range(question) {
            Some((got_from, got_to, future)) => {
                if (got_from.as_str(), got_to.as_str()) != (from, to) {
                    problems.push(format!(
                        "{question}: {got_from}..{got_to}, wanted {from}..{to}"
                    ));
                }
                if future {
                    problems.push(format!("{question}: a range cannot lie in the future"));
                }
            }
            None => problems.push(format!("{question}: no period, wanted {from}..{to}")),
        }
    }
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

#[test]
fn a_month_word_and_a_range_inside_it_print_the_range_and_this_month_ends_today() {
    // The words win (lead, 04-11-2026). "pichle mahine ki 1 se 15 tak" is the 1st to the 15th of
    // LAST month, not the whole of last month. "is mahine ki 1 se 15 tak" is this month's 1st to
    // the 15th, which cannot end after today (05-11-2026): the 1st to today.
    let y = year::seed::build_thin();
    let today = BusinessDate::from_ymd(2026, 11, 5).expect("today");
    let reader = Reader {
        ctx: &y.w.env.ctx,
        actor: &y.w.owner,
        language: Language::En,
        today,
        partner_locked: false,
    };
    let mut problems: Vec<String> = Vec::new();
    for (question, from, to) in [
        (
            "Faisal ki pichle mahine ki 1 se 15 tak ki bikri ka print",
            "2026-10-01",
            "2026-10-15",
        ),
        (
            "Faisal ki October ki 1 se 15 tak ki bikri ka print",
            "2026-10-01",
            "2026-10-15",
        ),
        (
            "فیصل کی پچھلے مہینے کی 1 سے 15 تک کی بکری کا پرنٹ",
            "2026-10-01",
            "2026-10-15",
        ),
        (
            "Faisal ki is mahine ki 1 se 15 tak ki bikri ka print",
            "2026-11-01",
            "2026-11-05",
        ),
        (
            "فیصل کی اس مہینے کی 1 سے 15 تک کی بکری کا پرنٹ",
            "2026-11-01",
            "2026-11-05",
        ),
    ] {
        let u = understand(&reader, question);
        let got = u
            .report
            .as_ref()
            .map(|r| (r.filters.from.map(iso), r.filters.to.map(iso)));
        if got != Some((Some(from.to_owned()), Some(to.to_owned()))) {
            problems.push(format!(
                "{question}: the report plan has {got:?}, wanted {from}..{to}; periods {:?}",
                u.periods
                    .iter()
                    .map(|p| (p.from_iso(), p.to_iso()))
                    .collect::<Vec<_>>()
            ));
        }
    }
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

#[test]
fn an_understanding_that_is_not_sure_gives_no_button_and_no_steps() {
    // Lead's rule (plan 55): when the app is not sure, it shows no button and no steps, so a wrong
    // guess never opens the wrong report. Each of these has a doubt: two days, no day, two Akrams, a
    // name nobody has, a comparison, a seller read from a buyer's word.
    let y = year::seed::build_thin();
    let today = BusinessDate::from_ymd(2026, 11, 5).expect("today");
    let reader = Reader {
        ctx: &y.w.env.ctx,
        actor: &y.w.owner,
        language: Language::En,
        today,
        partner_locked: false,
    };
    let mut problems: Vec<String> = Vec::new();
    for question in [
        "Faisal ki aaj aur kal ki bikri ka print",
        "Faisal ki bikri ka print",
        "akram ka bill nikalna",
        "Zulfiqar ki aaj ki bikri ka print",
        "Faisal ki pichle mahine aur is mahine ki bikri ka print",
        "ki aaj ki bikri ka print",
        "print",
        "फ़ैसल का print",
    ] {
        let u = understand(&reader, question);
        if !u.sure && u.report.is_some() {
            problems.push(format!(
                "{question}: not sure ({:?}) and still a report plan {:?}",
                u.doubts,
                u.report.as_ref().map(|r| r.id)
            ));
        }
    }
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

/// A probe for finding out why one wording is not understood: it prints the intent, the report plan with its
/// filters, the doubts and whether the understanding is sure. Run by hand:
/// `MANDI_PROBE='print the mosambi sales so far|print the tomato sales' cargo test -p mandi-assistant --test
/// robustness_understand probe -- --ignored --nocapture`.
#[test]
#[ignore = "a probe, run by hand with MANDI_PROBE"]
fn probe_the_understanding_of_questions() {
    let Ok(text) = std::env::var("MANDI_PROBE") else {
        return;
    };
    let y = year::seed::build();
    let today = BusinessDate::from_ymd(2026, 11, 5).expect("today");
    let reader = Reader {
        ctx: &y.w.env.ctx,
        actor: &y.w.owner,
        language: Language::En,
        today,
        partner_locked: false,
    };
    for question in text.split('|').map(str::trim).filter(|q| !q.is_empty()) {
        let u = understand(&reader, question);
        let plan = u.report.as_ref().map(|p| {
            format!(
                "{} item={:?} party={:?} from={:?} to={:?} assumed={}",
                p.id,
                p.filters.item.as_ref().map(|i| i.name_en.clone()),
                p.filters.party.as_ref().map(|h| h.name_en.clone()),
                p.filters.from.map(iso),
                p.filters.to.map(iso),
                p.assumed_period
            )
        });
        println!(
            "PROBE {question:?}: intent {:?}, sure {}, plan {plan:?}, doubts {:?}",
            u.intent.value, u.sure, u.doubts
        );
    }
}
