//! P5 of D6-system-prompt.md Section 7: each of the 16 worked examples of `system.md` is also a
//! recorded golden question (`fixtures/model/prompt-ex-01.json` to `-16.json`). Each one's tool
//! calls run, every reference resolves, `check.rs` passes it, and the filled text equals the
//! expected text in the fixture.
//!
//! The examples name keys such as `je4518` (a ledger line), `i41` (an item) and `lt88` (a lot).
//! The test worlds are made by the real services, whose row ids start at 1, so each fixture lists
//! its `aliases`: the key as written in the prompt and the key the world gives it. A key is the
//! row's own id with a letter prefix, so only the number differs; the lead decides whether the
//! prompt's numbers should be made real by starting the ids higher in a test database.
#![cfg(test)]

#[path = "../../services/tests/common/mod.rs"]
mod common;
#[path = "support/worked.rs"]
mod worked;
#[allow(dead_code)]
#[path = "../../services/tests/bikri_preview/support.rs"]
mod world;

use std::sync::Arc;

use mandi_assistant::answer::{Answer, AnswerPart};
use mandi_assistant::ask::{AskReply, AssistantAskInput};
use mandi_assistant::client::RecordedClient;
use mandi_assistant::keys::MemoryKeyStore;
use mandi_assistant::results::Words;
use mandi_assistant::run::{Deps, ask};
use mandi_assistant::runtime::Runtime;
use mandi_reports::dashboard::BackupFacts;
use mandi_reports::reports::Cell;
use mandi_services::core::accounts::BalanceSide;
use mandi_services::core::auth::Language;
use mandi_services::core::partner_code::{PartnerCodeSetInput, PartnerCodeUnlockInput};
use mandi_services::{assistant, partner_code};
use serde_json::Value;

const TEST_KEY: &str = "sk-or-v1-0000000000000000000000000000000000000000000000000000";

fn build(name: &str) -> world::World {
    match name {
        "day" | "partner" | "day_1115" => {
            let w = worked::worked_day();
            worked::events_1_to_5(&w);
            if name == "day_1115" {
                common::set_today(&w.env, 2026, 11, 15);
            }
            if name == "partner" {
                let ctx = &w.env.ctx;
                partner_code::set(
                    ctx,
                    &w.owner,
                    &PartnerCodeSetInput {
                        password: common::OWNER_PASSWORD.to_owned(),
                        code: "4826".to_owned(),
                        code_again: "4826".to_owned(),
                    },
                )
                .unwrap();
                partner_code::unlock(
                    ctx,
                    &w.owner,
                    &PartnerCodeUnlockInput {
                        code: "4826".to_owned(),
                    },
                )
                .unwrap();
            }
            w
        }
        // April to September, one banana sale a month (E2E-59).
        "history" => worked::history_world(&[
            (4, 180, 1_600),
            (5, 150, 2_100),
            (6, 120, 2_850),
            (7, 225, 1_600),
            (8, 150, 2_600),
            (9, 173, 2_390),
        ]),
        // Two items sold in August and September: the banana falls, the apple stays.
        "months_ab" => worked::months_world(&[
            (8, true, 100, 1_000),
            (8, false, 50, 2_000),
            (9, true, 60, 1_000),
            (9, false, 50, 2_000),
        ]),
        "stock_1013" => worked::part_sold_world(),
        other => panic!("no world {other}"),
    }
}

fn thousands(n: i64) -> String {
    let digits = n.abs().to_string();
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    if n < 0 { format!("-{out}") } else { out }
}

fn month_name(language: Language, m: i64) -> &'static str {
    const UR: [&str; 12] = [
        "جنوری",
        "فروری",
        "مارچ",
        "اپریل",
        "مئی",
        "جون",
        "جولائی",
        "اگست",
        "ستمبر",
        "اکتوبر",
        "نومبر",
        "دسمبر",
    ];
    const EN: [&str; 12] = [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ];
    let names = if language == Language::Ur { UR } else { EN };
    names
        .get(usize::try_from(m - 1).unwrap_or(0))
        .copied()
        .unwrap_or("?")
}

/// The answer text as the panel would write it, in plain characters, for comparing.
fn plain_text(answer: &Answer, language: Language, words: &Words) -> String {
    let mut out = String::new();
    for part in &answer.parts {
        match part {
            AnswerPart::Text { text } => out.push_str(text),
            AnswerPart::Figure {
                cell, dir, checked, ..
            } => {
                match cell {
                    Cell::Money { paisa } => out.push_str(&thousands(paisa.rupees())),
                    Cell::Balance { paisa, side } => {
                        out.push_str(&thousands(paisa.rupees()));
                        match side {
                            BalanceSide::Dr => out.push_str(" بنام"),
                            BalanceSide::Cr => out.push_str(" جمع"),
                            BalanceSide::Zero => {}
                        }
                    }
                    Cell::Percent { hundredths } => {
                        out.push_str(&format!("{}.{:02}%", hundredths / 100, hundredths % 100));
                    }
                    Cell::Number { value } => out.push_str(&thousands(*value)),
                    other => panic!("a figure of this kind: {other:?}"),
                }
                if let Some(dir) = dir {
                    out.push_str(match (dir, language) {
                        (mandi_assistant::answer::FigureDirection::Up, Language::Ur) => " زیادہ",
                        (mandi_assistant::answer::FigureDirection::Down, Language::Ur) => " کم",
                        (mandi_assistant::answer::FigureDirection::Same, Language::Ur) => " برابر",
                        (mandi_assistant::answer::FigureDirection::Up, Language::En) => " more",
                        (mandi_assistant::answer::FigureDirection::Down, Language::En) => " less",
                        (mandi_assistant::answer::FigureDirection::Same, Language::En) => {
                            " the same"
                        }
                    });
                }
                if *checked {
                    out.push_str(" ✓");
                }
            }
            AnswerPart::Name { name } => {
                let base = if language == Language::Ur {
                    &name.name_ur
                } else {
                    &name.name_en
                };
                out.push_str(base);
                if let Some(tag) = &name.tag {
                    out.push(' ');
                    out.push_str(tag);
                }
            }
            AnswerPart::Date { iso } => {
                let (y, rest) = iso.split_at(4);
                out.push_str(&format!("{}-{}-{}", &rest[4..6], &rest[1..3], &y[2..]));
            }
            AnswerPart::Month { year, month } => {
                out.push_str(&format!("{} {year}", month_name(language, *month)));
            }
            AnswerPart::Doc { type_id, number } => {
                let label = words.text(language, type_id);
                match number {
                    Some(n) => out.push_str(&format!("{label} {n}")),
                    None => out.push_str(&label),
                }
            }
        }
    }
    out
}

#[test]
fn p5_the_sixteen_examples_run_as_golden_questions() {
    let words = Words::new().unwrap();
    // `MANDI_WRITE_FIXTURES=1` writes the filled texts into the fixtures (to be read against the
    // PRD numbers before they are committed); a normal run compares with them.
    let write = std::env::var("MANDI_WRITE_FIXTURES").is_ok_and(|v| v == "1");
    let mut actual_report = Vec::new();
    let mut failures = Vec::new();
    for n in 1..=16 {
        let path = format!(
            "{}/tests/fixtures/model/prompt-ex-{n:02}.json",
            env!("CARGO_MANIFEST_DIR")
        );
        let mut fixture: Value =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        let mut texts: Vec<(usize, String)> = Vec::new();
        let language = if fixture["language"] == "en" {
            Language::En
        } else {
            Language::Ur
        };
        let w = build(fixture["world"].as_str().unwrap());
        if language == Language::En {
            mandi_services::settings::set_pref(
                &w.env.ctx,
                &w.owner,
                mandi_services::core::auth::PrefKey::Language,
                "en",
            )
            .unwrap();
        }
        let runtime = Runtime::new(Arc::new(MemoryKeyStore::default()));
        runtime.keys().set(TEST_KEY).unwrap();
        assistant::set_config(&w.env.ctx, &w.owner, true, "openai/gpt-6-luna", true).unwrap();
        let mut conversation: Option<String> = None;
        for (i, step) in fixture["steps"].as_array().unwrap().iter().enumerate() {
            // The prompt's keys, as the world gives them.
            let mut replies = step["replies"].to_string();
            if let Some(aliases) = fixture["aliases"].as_object() {
                for (from, to) in aliases {
                    replies = replies.replace(from.as_str(), to.as_str().unwrap());
                }
            }
            let replies: Vec<Value> = serde_json::from_str(&replies).unwrap();
            let client = RecordedClient::new(replies);
            let emit = |_s| {};
            let deps = Deps {
                ctx: &w.env.ctx,
                actor: &w.owner,
                runtime: &runtime,
                client: &client,
                emit: &emit,
                backup: BackupFacts {
                    cloud_on: false,
                    cloud_age_hours: None,
                },
            };
            let reply: AskReply = ask(
                &deps,
                &AssistantAskInput {
                    text: step["question"].as_str().unwrap().to_owned(),
                    conversation_id: conversation.clone(),
                },
            )
            .unwrap();
            conversation = Some(reply.conversation_id.clone());
            let want = step["outcome"].as_str().unwrap();
            let got = serde_json::to_value(reply.outcome).unwrap();
            if got != want {
                failures.push(format!(
                    "example {n} step {i}: outcome {got} not {want}: {reply:#?}"
                ));
                continue;
            }
            if let Some(route) = step["route"].as_str() {
                let r = reply.entry_route.as_ref().expect("a route");
                let got = format!(
                    "{}:a{}",
                    format!("{:?}", r.screen).to_lowercase(),
                    r.party.as_ref().unwrap().id
                );
                if got != route {
                    failures.push(format!("example {n} step {i}: route {got} not {route}"));
                }
            }
            if let Some(source) = step["source"].as_str() {
                let answer = reply.answer.as_ref().expect("the nearest source");
                let got = match &answer.sources[0] {
                    mandi_assistant::answer::AnswerSource::PartyCard { .. } => {
                        "party_card".to_owned()
                    }
                    mandi_assistant::answer::AnswerSource::Analytics { view, .. } => {
                        format!("analytics:{}", view.code())
                    }
                    other => format!("{other:?}"),
                };
                if got != source {
                    failures.push(format!("example {n} step {i}: source {got} not {source}"));
                }
            }
            if let Some(answer) = &reply.answer
                && !answer.parts.is_empty()
            {
                let text = plain_text(answer, language, &words);
                actual_report.push(format!("example {n} step {i}:\n{text}\n"));
                texts.push((i, text.clone()));
                if write {
                    continue;
                }
                match step["expected"].as_str() {
                    Some(expected) if expected == text => {}
                    Some(expected) => failures.push(format!(
                        "example {n} step {i}: expected\n{expected}\nbut got\n{text}"
                    )),
                    None => failures.push(format!(
                        "example {n} step {i}: no expected text; the filled text is\n{text}"
                    )),
                }
            }
        }
        if write {
            for (i, text) in texts {
                fixture["steps"][i]["expected"] = Value::String(text);
            }
            let mut out = serde_json::to_string_pretty(&fixture).unwrap();
            out.push('\n');
            std::fs::write(&path, out).unwrap();
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n---\n"));
}
