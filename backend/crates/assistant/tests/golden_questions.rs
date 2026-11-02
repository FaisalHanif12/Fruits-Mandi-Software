//! The golden questions of D6 (PRD Section 27, E2E-47 to E2E-63; Code Standards 9.2). Each one
//! replays a recorded transcript of the model (`fixtures/model/*.json`) through the real loop on
//! the PRD worked day, so the numbers are the PRD's and the model is not called (no commit test
//! uses the network). The expected figures are the PRD's and are never changed to suit the code.
//!
//! Every request body any test sends is scanned (E2E-54): no phone, address, father or firm name,
//! note, reason or key, and always the provider block of Tech Stack 4.8.
#![cfg(test)]

#[path = "../../services/tests/common/mod.rs"]
mod common;
#[path = "support/harness.rs"]
mod harness;
#[path = "support/worked.rs"]
mod worked;
#[allow(dead_code)]
#[path = "../../services/tests/bikri_preview/support.rs"]
mod world;
use harness::{Harness, answer_of, balance, count, figures, money, transcript};

use std::collections::BTreeSet;

use mandi_assistant::answer::AnswerPart;
use mandi_assistant::ask::{
    AskOutcome, AssistantNewConversationInput, FailureKind, PickChoice, QuickChip,
};
use mandi_assistant::client::{CallError, RecordedClient};
use mandi_assistant::prompt::SYSTEM;
use mandi_assistant::service;
use mandi_reports::dashboard::BackupFacts;
use mandi_reports::reports::Cell;
use mandi_reports::reports::{ArgValue, Names};
use mandi_services::core::accounts::{AccountType, BalanceSide};
use mandi_services::core::partner_code::{PartnerCodeSetInput, PartnerCodeUnlockInput};
use mandi_services::{assistant, partner_code};

macro_rules! fixture {
    ($name:literal) => {
        transcript(include_str!(concat!("fixtures/model/", $name, ".json")))
    };
}

// ---- E2E-47 ---------------------------------------------------------------------------------

#[test]
fn e2e_47_two_balances_are_cited_checked_and_two_questions_are_counted() {
    // PRD 27.2 E2E-47: "ikram ke total bakaya kitne hai" after Events 1 to 5. اکرم owes 2,880
    // بنام, checked against his ledger; the R13 table shows his receipt of 10,000 and his bill of
    // 12,880, newest first.
    let h = Harness::day();
    let (reply, requests) = h.ask("ikram ke total bakaya kitne hai", None, fixture!("e2e-47"));
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    assert_eq!((reply.count_today, reply.limit), (1, 20));
    let answer = answer_of(&reply);
    let figs = figures(answer);
    assert_eq!(balance(figs[0].0), (2_880, BalanceSide::Dr));
    assert!(figs[0].1, "2,880 is checked (party card = R13 closing)");
    assert_eq!(money(figs[1].0), 12_880);
    assert_eq!(money(figs[2].0), 10_000);
    assert_eq!(answer.next.len(), 2);
    // The table: the receipt first (10-10-26), then بکری 1.
    assert_eq!(answer.tables.len(), 1);
    let rows = &answer.tables[0].rows;
    assert_eq!(rows.len(), 2);
    assert!(
        rows[0]
            .row
            .cells
            .iter()
            .any(|c| matches!(c, Cell::Money { paisa } if paisa.rupees() == 10_000))
    );
    assert!(
        rows[1]
            .row
            .cells
            .iter()
            .any(|c| matches!(c, Cell::Money { paisa } if paisa.rupees() == 12_880))
    );
    // "پوری رپورٹ کھولیں" opens R13 for اکرم.
    assert!(answer.sources.iter().any(|s| matches!(s,
        mandi_assistant::answer::AnswerSource::Report { id, filters, .. }
            if id.code() == "R13" && filters.party.as_ref().is_some_and(|p| p.record.id == h.world.akram))));
    // Three model calls: the third has every result.
    assert_eq!(requests.len(), 3);
    // The step lines, in the panel's own words: finding, the card, the ledger, then the check.
    let steps: Vec<String> = h
        .steps
        .lock()
        .unwrap()
        .iter()
        .map(|s| s.id.clone())
        .collect();
    assert_eq!(
        steps,
        [
            "assistant-step-finding",
            "assistant-step-reading-card",
            "assistant-step-reading-ledger",
            "assistant-step-checking",
        ]
    );
    // Answer 2: کریم, 10,160 جمع ✓, in the same conversation. Two questions counted.
    let (second, _) = h.ask(
        "karim ko kitne paise dene hain",
        Some(&reply.conversation_id),
        fixture!("e2e-47b"),
    );
    assert_eq!(second.outcome, AskOutcome::Answered, "{second:#?}");
    let figs = figures(answer_of(&second));
    assert_eq!(balance(figs[0].0), (10_160, BalanceSide::Cr));
    assert!(figs[0].1);
    assert_eq!(second.count_today, 2);
    h.scan_requests();
}

// ---- E2E-48 ---------------------------------------------------------------------------------

#[test]
fn e2e_48_the_days_sales_and_commission_come_from_the_tiles_checked_against_view_a() {
    let h = Harness::day();
    let (reply, _) = h.ask(
        "aaj ki bikri kitni hui aur commission kitna bana",
        None,
        fixture!("e2e-48"),
    );
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    let figs = figures(answer_of(&reply));
    // 2 بکری, خام بکری 41,400 ✓, commission 5,448 ✓.
    assert_eq!(count(figs[0].0), 2);
    assert_eq!(money(figs[1].0), 41_400);
    assert!(figs[1].1);
    assert_eq!(money(figs[2].0), 5_448);
    assert!(figs[2].1);
    // One question counted, whatever the number of tools.
    assert_eq!(reply.count_today, 1);
    h.scan_requests();
}

// ---- E2E-49 ---------------------------------------------------------------------------------

#[test]
fn e2e_49_commission_on_banana_this_month_is_read_from_view_a_with_its_filters() {
    let h = Harness::day();
    let (reply, _) = h.ask(
        "kele se is mahine kitna commission aya",
        None,
        fixture!("e2e-49"),
    );
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    let answer = answer_of(&reply);
    let figs = figures(answer);
    assert_eq!(money(figs[0].0), 3_928);
    // 72.10% of all the commission: the row's share in view A over the month. (With the item
    // filter set the engine's share is of the filtered total, 100.00%, so the transcript reads the
    // view for the whole month and cites the banana row, as the PRD's table E2E-11 does.)
    assert!(
        matches!(figs[1].0, Cell::Percent { hundredths: 7_210 }),
        "{:?}",
        figs[1].0
    );
    // The button opens view A with the month's dates.
    assert!(answer.sources.iter().any(|s| matches!(s,
        mandi_assistant::answer::AnswerSource::Analytics { view, filters, .. }
            if view.code() == "A"
                && filters.from == "2026-10-01" && filters.to == "2026-10-10")));
    h.scan_requests();
}

// ---- E2E-50 and the relock part of E2E-54 ----------------------------------------------------

#[test]
fn e2e_50_partner_totals_wait_for_the_code_then_go_on_without_counting_again() {
    let h = Harness::day();
    let ctx = &h.world.env.ctx;
    partner_code::set(
        ctx,
        &h.world.owner,
        &PartnerCodeSetInput {
            password: common::OWNER_PASSWORD.to_owned(),
            code: "4826".to_owned(),
            code_again: "4826".to_owned(),
        },
    )
    .unwrap();
    partner_code::lock(ctx).unwrap();
    let (waiting, requests) = h.ask(
        "partner ki kiya total hain abhi tak",
        None,
        fixture!("e2e-50a"),
    );
    assert_eq!(waiting.outcome, AskOutcome::Locked, "{waiting:#?}");
    assert!(waiting.locked && waiting.answer.is_none());
    // The partner tool was asked for; no model call was made with a locked result.
    assert_eq!(requests.len(), 1);
    assert_eq!(waiting.count_today, 1);
    // The code is given; the paused call runs and the model goes on, not counted again.
    partner_code::unlock(
        ctx,
        &h.world.owner,
        &PartnerCodeUnlockInput {
            code: "4826".to_owned(),
        },
    )
    .unwrap();
    let (done, _) = h.pick(
        waiting.question_id,
        PickChoice::PartnerCode,
        fixture!("e2e-50b"),
    );
    assert_eq!(done.outcome, AskOutcome::Answered, "{done:#?}");
    assert_eq!(done.count_today, 1);
    let figs = figures(answer_of(&done));
    // کل رقم: الف −1,004, ب 746, ج 746, each ✓ against R31.
    assert_eq!(money(figs[0].0), -1_004);
    assert_eq!(money(figs[1].0), 746);
    assert_eq!(money(figs[2].0), 746);
    assert!(figs.iter().all(|(_, checked)| *checked));
    // E2E-54: the section locks again and a follow-up is asked: its body holds none of the
    // earlier partner figures.
    partner_code::lock(ctx).unwrap();
    let (follow, requests) = h.ask(
        "aaj ka commission",
        Some(&done.conversation_id),
        fixture!("e2e-50c"),
    );
    assert_eq!(follow.outcome, AskOutcome::Answered, "{follow:#?}");
    for body in &requests {
        let text = body.to_string();
        assert!(
            !text.contains("\\\"rs\\\":-1004"),
            "an earlier partner figure was sent"
        );
        assert!(
            !text.contains("\\\"rs\\\":746"),
            "an earlier partner figure was sent"
        );
        assert!(text.contains("partner section locked"));
    }
    h.scan_requests();
}

// ---- E2E-51 ---------------------------------------------------------------------------------

#[test]
fn e2e_51_total_dues_are_checked_through_their_three_parts_and_a_chip_needs_no_model() {
    let h = Harness::day();
    let (reply, _) = h.ask("kul ugrahi kitni hai", None, fixture!("e2e-51"));
    let figs = figures(answer_of(&reply));
    // 21,564 ✓ (R20 21,564, R24 0, R38 0).
    assert_eq!(money(figs[0].0), 21_564);
    assert!(figs[0].1);
    // The chip موجود روکڑ: 58,484 ✓ (R12 closing), no model call, not counted.
    let chip = h.chip(QuickChip::CashInHand);
    let figs = figures(answer_of(&chip));
    assert_eq!(money(figs[0].0), 58_484);
    assert!(figs[0].1);
    assert_eq!(chip.count_today, 1, "the chip is not counted");
    h.scan_requests();
}

// ---- E2E-52 ---------------------------------------------------------------------------------

#[test]
fn e2e_52_two_akrams_ask_which_one_and_a_click_answers_without_a_new_question() {
    let w = worked::worked_day();
    worked::events_1_to_5(&w);
    // A new buyer اکرم with the tag K, no entries (code 0022).
    let k = world::add_account_tagged(
        &w.env,
        &w.owner,
        AccountType::Buyer,
        ("اکرم", "Akram"),
        (true, ""),
        "K",
    );
    let h = Harness::over(w);
    let (waiting, _) = h.ask("akram ka bakaya", None, fixture!("e2e-52a"));
    assert_eq!(waiting.outcome, AskOutcome::WhichOne, "{waiting:#?}");
    assert_eq!(waiting.options.len(), 2);
    let akram = waiting
        .options
        .iter()
        .find(|o| o.key == format!("a{}", h.world.akram))
        .expect("اکرم ٹول ٹیکس is offered");
    assert_eq!(
        balance(akram.balance.as_ref().unwrap()),
        (2_880, BalanceSide::Dr)
    );
    let tagged = waiting
        .options
        .iter()
        .find(|o| o.key == format!("a{k}"))
        .expect("اکرم K is offered");
    assert_eq!(tagged.name.tag.as_deref(), Some("K"));
    assert_eq!(balance(tagged.balance.as_ref().unwrap()).0, 0);
    assert_eq!(waiting.count_today, 1);
    // A click on 0020 shows 2,880 ✓; still one question counted.
    let (done, _) = h.pick(
        waiting.question_id,
        PickChoice::Match {
            key: format!("a{}", h.world.akram),
        },
        fixture!("e2e-52b"),
    );
    assert_eq!(done.outcome, AskOutcome::Answered, "{done:#?}");
    let figs = figures(answer_of(&done));
    assert_eq!(balance(figs[0].0), (2_880, BalanceSide::Dr));
    assert!(figs[0].1);
    assert_eq!(done.count_today, 1);
    h.scan_requests();
}

// ---- E2E-53 ---------------------------------------------------------------------------------

/// The number of rows of every table of the database, by table name.
fn row_counts(dir: &tempfile::TempDir) -> Vec<(String, i64)> {
    let conn = common::raw(dir);
    let mut names: Vec<String> = conn
        .prepare("SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%' ORDER BY name")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    names.sort();
    names
        .into_iter()
        .map(|name| {
            let n: i64 = conn
                .query_row(&format!("SELECT COUNT(*) FROM \"{name}\""), [], |r| {
                    r.get(0)
                })
                .unwrap();
            (name, n)
        })
        .collect()
}

#[test]
fn e2e_53_a_change_is_refused_and_nothing_but_the_log_is_written() {
    let h = Harness::day();
    let before = row_counts(&h.world.dir);
    let (reply, _) = h.ask("akram ki 5000 wasooli likh do", None, fixture!("e2e-53"));
    assert_eq!(reply.outcome, AskOutcome::ChangeRefused, "{reply:#?}");
    let route = reply.entry_route.as_ref().expect("the entry to open");
    assert_eq!(format!("{:?}", route.screen), "Wasooli");
    assert_eq!(route.party.as_ref().map(|p| p.id), Some(h.world.akram));
    let after = row_counts(&h.world.dir);
    for ((name, was), (_, now)) in before.iter().zip(after.iter()) {
        let expected = if name == "assistant_questions" {
            was + 1
        } else {
            *was
        };
        assert_eq!(*now, expected, "table {name}");
    }
    h.scan_requests();
}

#[test]
fn e2e_53_the_entry_route_carries_the_whole_record_and_a_partner_on_the_partner_screens() {
    let h = Harness::day();
    let route_of = |text: &str, replies| {
        let (reply, requests) = h.ask(text, None, replies);
        assert_eq!(reply.outcome, AskOutcome::ChangeRefused, "{reply:#?}");
        (reply.entry_route.expect("the entry to open"), requests)
    };
    // An account screen: the whole account record, id = accounts.id.
    let (route, _) = route_of("akram ki 5000 wasooli likh do", fixture!("e2e-53"));
    let party = route.party.expect("the party");
    assert_eq!(
        (
            party.id,
            party.code.as_deref(),
            party.name_ur.as_str(),
            party.name_en.as_str(),
            party.tag.as_deref(),
        ),
        (
            h.world.akram,
            Some("0020"),
            "اکرم ٹول ٹیکس",
            "Akram Toll Tax",
            None
        )
    );
    let (route, _) = route_of("rahim ko 5000 peshgi do", fixture!("e2e-53-peshgi"));
    assert_eq!(format!("{:?}", route.screen), "Peshgi");
    assert_eq!(route.party.map(|p| p.id), Some(h.world.rahim));
    // The partner screens: the PARTNER record, id = partners.id, his own names, no code, no tag.
    for (screen, replies) in [
        ("Kharcha", fixture!("e2e-53-kharcha")),
        ("Sarmaya", fixture!("e2e-53-sarmaya")),
        ("Phal", fixture!("e2e-53-phal")),
    ] {
        let (route, requests) = route_of("partner a ka kaam likh do", replies);
        assert_eq!(format!("{:?}", route.screen), screen);
        let party = route.party.expect("the partner");
        assert_eq!(party.id, h.world.partners[0], "{screen}: partners.id");
        assert_eq!(
            (party.name_ur.as_str(), party.name_en.as_str()),
            ("الف", "Partner A")
        );
        assert_eq!((party.code, party.tag), (None, None));
        // The find that showed him is the partner find: key `s<id>`, never one of his accounts.
        let sent = requests[1].to_string();
        assert!(
            sent.contains(&format!("s{}", h.world.partners[0])),
            "{screen}"
        );
    }
    // A buyer on a partner screen is the wrong kind of record: the screen opens with no party.
    let (route, _) = route_of("akram ka kharcha likh do", fixture!("e2e-53-wrong-kind"));
    assert_eq!(format!("{:?}", route.screen), "Kharcha");
    assert!(route.party.is_none());
    h.scan_requests();
}

#[test]
fn e2e_53_a_change_request_an_answer_or_advice_may_carry_the_entry_route_an_estimate_not() {
    // PRD E98 and D6-system-prompt `entry`: required for a change request, allowed with an answer
    // or advice to a question that also asks for a change. The schema does not allow it with an
    // estimate, so an estimate carries none.
    let h = Harness::day();
    let (advice, _) = h.ask(
        "kis se pehle wasooli karun",
        None,
        fixture!("e2e-53-advice"),
    );
    assert_eq!(advice.outcome, AskOutcome::Advice, "{advice:#?}");
    let route = advice.entry_route.expect("the route of the change asked");
    assert_eq!(format!("{:?}", route.screen), "Wasooli");
    assert_eq!(route.party.map(|p| p.id), Some(h.world.akram));
    let (answer, _) = h.ask(
        "akram ka hisab aur wasooli",
        None,
        fixture!("e2e-53-answer"),
    );
    assert_eq!(answer.outcome, AskOutcome::Answered, "{answer:#?}");
    let route = answer.entry_route.expect("the route of the change asked");
    assert_eq!(format!("{:?}", route.screen), "Wasooli");
    assert_eq!(route.party.map(|p| p.id), Some(h.world.akram));
    // An estimate that sends an entry anyway carries no route. The recorded estimate of E2E-59 is
    // read with an entry added to its last reply.
    let mut replies = fixture!("e2e-59a");
    let last = replies.last_mut().expect("the final reply");
    let content = last["choices"][0]["message"]["content"]
        .as_str()
        .expect("the answer text")
        .to_owned();
    let mut model: serde_json::Value = serde_json::from_str(&content).expect("json");
    model["entry"] = serde_json::json!({"screen": "wasooli", "party": null});
    last["choices"][0]["message"]["content"] = serde_json::Value::String(model.to_string());
    let history = Harness::over(worked::history_world(&HISTORY));
    let (estimate, _) = history.ask(
        "agar commission isi raftar se chalta raha to agle 6 mahine mein kitna hoga",
        None,
        replies,
    );
    assert_eq!(estimate.outcome, AskOutcome::Estimate, "{estimate:#?}");
    assert!(estimate.entry_route.is_none(), "an estimate opens nothing");
}

// ---- E2E-55 ---------------------------------------------------------------------------------

#[test]
fn e2e_55_the_twenty_first_question_is_refused_and_offline_counts_nothing() {
    let h = Harness::day();
    for _ in 0..20 {
        let (reply, _) = h.ask("aaj ka commission", None, fixture!("limit-question"));
        assert_eq!(reply.outcome, AskOutcome::Answered);
    }
    let (limit, requests) = h.ask("aaj ka commission", None, fixture!("limit-question"));
    assert_eq!(limit.outcome, AskOutcome::LimitReached);
    assert_eq!((limit.count_today, limit.limit), (20, 20));
    assert!(requests.is_empty(), "no request is sent over the limit");
    // The chip still works.
    let chip = h.chip(QuickChip::CashInHand);
    assert_eq!(money(figures(answer_of(&chip))[0].0), 58_484);
    // Offline, on a fresh day: the offline outcome, nothing counted, the chip still works.
    let fresh = Harness::day();
    // Both tries fail: the call is tried once more (plan 52 item 6) and then it is offline.
    let client = RecordedClient::with_results(vec![
        Err(CallError::new(FailureKind::Offline)),
        Err(CallError::new(FailureKind::Offline)),
    ]);
    let offline = fresh.ask_with("aaj ka commission", None, &client);
    assert_eq!(offline.outcome, AskOutcome::Offline);
    assert_eq!(offline.failure.map(|f| f.kind), Some(FailureKind::Offline));
    assert_eq!(offline.count_today, 0);
    let chip = fresh.chip(QuickChip::CashInHand);
    assert_eq!(money(figures(answer_of(&chip))[0].0), 58_484);
    h.scan_requests();
}

// ---- E2E-56 ---------------------------------------------------------------------------------

#[test]
fn e2e_56_wrong_replies_get_an_error_result_and_a_clear_ending() {
    let h = Harness::day();
    // A report the app does not have, and then the model gives up.
    let (reply, _) = h.ask("koi report", None, fixture!("e2e-56a"));
    assert_eq!(reply.outcome, AskOutcome::NotUnderstood, "{reply:#?}");
    assert!(reply.answer.is_none());
    let row = assistant::get_question(&h.world.env.ctx, reply.question_id)
        .unwrap()
        .unwrap();
    assert!(
        row.tool_calls_json.contains("R99"),
        "the refused call is in the log"
    );
    // 31-02: the tool refuses the date and the answer asks for the dates.
    let (reply, _) = h.ask("31 february ka hisab", None, fixture!("e2e-56b"));
    assert_eq!(reply.outcome, AskOutcome::NeedDates, "{reply:#?}");
    h.scan_requests();
}

// ---- E2E-57 ---------------------------------------------------------------------------------

#[test]
fn e2e_57_a_whole_ledger_is_shown_newest_first_from_the_first_date() {
    let h = Harness::day();
    let (reply, _) = h.ask("rahim ka pura hisaab", None, fixture!("e2e-57"));
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    let answer = answer_of(&reply);
    let figs = figures(answer);
    // رحیم: balance 0 ✓ (party card 0 = R13 closing 0).
    assert_eq!(balance(figs[0].0).0, 0);
    assert!(figs[0].1);
    let rows = &answer.tables[0].rows;
    // Newest first: the ادائیگی of 10-10-26 (بنام 4,316) first, the پیشگی of 01-09-26 (بنام 15,000) last.
    assert!(
        rows.first()
            .unwrap()
            .row
            .cells
            .iter()
            .any(|c| matches!(c, Cell::Money { paisa } if paisa.rupees() == 4_316))
    );
    assert!(
        rows.last()
            .unwrap()
            .row
            .cells
            .iter()
            .any(|c| matches!(c, Cell::Money { paisa } if paisa.rupees() == 15_000))
    );
    h.scan_requests();
}

// ---- E2E-58 ---------------------------------------------------------------------------------

#[test]
fn e2e_58_a_follow_up_keeps_the_party_and_carries_the_earlier_questions() {
    let h = Harness::day();
    let (first, _) = h.ask("ikram ke total bakaya kitne hai", None, fixture!("e2e-47"));
    let (second, requests) = h.ask(
        "aur pichle mahine?",
        Some(&first.conversation_id),
        fixture!("e2e-58"),
    );
    assert_eq!(second.outcome, AskOutcome::Answered, "{second:#?}");
    let figs = figures(answer_of(&second));
    // No entries last month, balance 0.
    assert_eq!(balance(figs[0].0).0, 0);
    // The request carries the earlier question and its answer (PRD FR-7.78).
    let text = requests[0].to_string();
    assert!(text.contains("ikram ke total bakaya kitne hai"));
    assert!(text.contains("r1.matches.0.name"), "the earlier answer");
    h.scan_requests();
}

#[test]
fn a_new_conversation_drops_the_old_one_at_once_and_an_ask_with_its_id_starts_fresh() {
    let h = Harness::day();
    let ctx = &h.world.env.ctx;
    // Not the words of a prompt example, so the request can be searched for them.
    let first_words = "zq first question about akram";
    let (first, _) = h.ask(first_words, None, fixture!("e2e-47"));
    let old = first.conversation_id.clone();
    let hold = |id: &str| {
        let kept = h.runtime.take(id);
        let there = kept.is_some();
        if let Some(conversation) = kept {
            h.runtime.keep(conversation);
        }
        there
    };
    assert!(hold(&old), "the conversation is in memory");
    // No id: a new id, and nothing is dropped.
    let fresh = service::new_conversation(
        ctx,
        &h.runtime,
        &AssistantNewConversationInput {
            conversation_id: None,
        },
    );
    assert_ne!(fresh.conversation_id, old);
    assert!(hold(&old));
    // An id the app does not know: a new id, and nothing is dropped.
    let other = service::new_conversation(
        ctx,
        &h.runtime,
        &AssistantNewConversationInput {
            conversation_id: Some("c-unknown".to_owned()),
        },
    );
    assert_ne!(other.conversation_id, fresh.conversation_id);
    assert!(hold(&old));
    // The old id: its memory is gone at once.
    let started = service::new_conversation(
        ctx,
        &h.runtime,
        &AssistantNewConversationInput {
            conversation_id: Some(old.clone()),
        },
    );
    assert_ne!(started.conversation_id, old);
    assert!(!hold(&old), "the old conversation was dropped");
    // An ask with the old id starts fresh: the model is not shown the earlier question.
    let (second, requests) = h.ask("aaj ka commission", Some(&old), fixture!("limit-question"));
    assert_eq!(second.outcome, AskOutcome::Answered, "{second:#?}");
    let sent = requests[0].to_string();
    assert!(sent.contains("aaj ka commission"));
    assert!(!sent.contains(first_words));
    h.scan_requests();
}

#[test]
fn a_step_line_carries_a_partys_name_in_both_scripts_and_what_was_typed_as_text() {
    let h = Harness::day();
    h.ask("ikram ke total bakaya kitne hai", None, fixture!("e2e-47"));
    let steps = h.steps.lock().unwrap().clone();
    let names = ArgValue::Names {
        names: Names {
            ur: "اکرم ٹول ٹیکس".to_owned(),
            en: "Akram Toll Tax".to_owned(),
        },
    };
    for id in [
        "assistant-step-reading-card",
        "assistant-step-reading-ledger",
    ] {
        let step = steps
            .iter()
            .find(|s| s.id == id)
            .unwrap_or_else(|| panic!("{id}"));
        assert_eq!(step.args.len(), 1, "{id}");
        assert_eq!(step.args[0].name, "name", "{id}");
        assert_eq!(step.args[0].value, names, "{id}");
    }
    // What the Owner typed is not a record's name: it stays text.
    let finding = steps
        .iter()
        .find(|s| s.id == "assistant-step-finding")
        .expect("the find step");
    assert_eq!(
        finding.args[0].value,
        ArgValue::Text {
            text: "ikram".to_owned()
        }
    );
}

// ---- E2E-60 ---------------------------------------------------------------------------------

#[test]
fn e2e_60_a_typed_figure_is_refused_once_then_the_model_writes_it_with_a_reference() {
    let h = Harness::day();
    let (reply, requests) = h.ask("ikram ke total bakaya kitne hai", None, fixture!("e2e-60a"));
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    let figs = figures(answer_of(&reply));
    assert_eq!(balance(figs[0].0), (2_880, BalanceSide::Dr));
    assert!(figs[0].1);
    assert_eq!(reply.count_today, 1);
    // The rewrite was told what failed.
    let last = requests.last().unwrap().to_string();
    assert!(last.contains("check_failed"));
    assert!(last.contains("digit"));
    assert_eq!(requests.len(), 4);
    h.scan_requests();
}

#[test]
fn e2e_60_two_bad_answers_show_the_records_as_plain_tables_and_no_typed_figure() {
    let h = Harness::day();
    let (reply, _) = h.ask("ikram ke total bakaya kitne hai", None, fixture!("e2e-60b"));
    assert_eq!(reply.outcome, AskOutcome::ChecksFailed, "{reply:#?}");
    let answer = answer_of(&reply);
    assert!(answer.parts.is_empty(), "nothing the model wrote is shown");
    assert!(!answer.sources.is_empty());
    assert_eq!(reply.count_today, 1);
    h.scan_requests();
}

#[test]
fn e2e_60_each_kind_of_bad_answer_is_refused() {
    // Words, Arabic-Indic digits, a name typed outside a reference, a figure in a next question,
    // a typed ✓, the other side's words, a month name, a never-use word, the wrong script, and a
    // reference that does not exist: each fails twice and the records are shown instead.
    for (variant, rule) in [
        ("words", "number_word"),
        ("indic", "digit"),
        ("name", "name"),
        ("next", "next"),
        ("check", "check_mark"),
        ("side", "side_words"),
        ("month", "month"),
        ("never", "never_word"),
        ("script", "script"),
        ("unknown", "reference"),
    ] {
        let h = Harness::day();
        let replies = match variant {
            "words" => fixture!("e2e-60-words"),
            "indic" => fixture!("e2e-60-indic"),
            "name" => fixture!("e2e-60-name"),
            "next" => fixture!("e2e-60-next"),
            "check" => fixture!("e2e-60-check"),
            "side" => fixture!("e2e-60-side"),
            "month" => fixture!("e2e-60-month"),
            "never" => fixture!("e2e-60-never"),
            "script" => fixture!("e2e-60-script"),
            _ => fixture!("e2e-60-unknown"),
        };
        let (reply, requests) = h.ask("ikram ke total bakaya kitne hai", None, replies);
        assert_eq!(
            reply.outcome,
            AskOutcome::ChecksFailed,
            "{variant}: {reply:#?}"
        );
        assert!(answer_of(&reply).parts.is_empty(), "{variant}");
        // It was refused for the rule it breaks, and the model was told which.
        let told = requests.last().unwrap().to_string();
        assert!(
            told.contains(&format!("\\\"rule\\\":\\\"{rule}\\\"")),
            "{variant}: the rewrite was not told {rule}: {told}"
        );
        h.scan_requests();
    }
}

// ---- E2E-61 ---------------------------------------------------------------------------------

#[test]
fn e2e_61_two_balances_are_added_by_the_calculator_with_its_working() {
    let h = Harness::day();
    let (reply, _) = h.ask(
        "akram aur arshad dono ka mila kar kitna hai",
        None,
        fixture!("e2e-61"),
    );
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    let answer = answer_of(&reply);
    let figs = figures(answer);
    assert_eq!(balance(figs[0].0), (21_564, BalanceSide::Dr));
    assert!(figs[0].1);
    assert_eq!(balance(figs[1].0), (2_880, BalanceSide::Dr));
    assert!(figs[1].1);
    assert_eq!(balance(figs[2].0), (18_684, BalanceSide::Dr));
    assert!(figs[2].1);
    // ⓘ shows the working of the sum.
    let working = answer.parts.iter().find_map(|p| match p {
        AnswerPart::Figure {
            working: Some(w), ..
        } => Some(w),
        _ => None,
    });
    let working = working.expect("the working");
    assert_eq!(working.steps.len(), 1);
    assert_eq!(working.steps[0].inputs.len(), 2);
    h.scan_requests();
}

// ---- E2E-62 ---------------------------------------------------------------------------------

#[test]
fn e2e_62_four_wordings_give_the_same_checked_answer() {
    let h = Harness::day();
    let wordings = [
        "akram ka hisab kya hai",
        "akram ka khata dikhao",
        "اکرم سے کتنا لینا ہے",
        "how much does akram owe",
    ];
    for (i, text) in wordings.iter().enumerate() {
        if i == 3 {
            // The fourth is asked in the English app.
            mandi_services::settings::set_pref(
                &h.world.env.ctx,
                &h.world.owner,
                mandi_services::core::auth::PrefKey::Language,
                "en",
            )
            .unwrap();
        }
        let replies = match i {
            0 => fixture!("e2e-62-1"),
            1 => fixture!("e2e-62-2"),
            2 => fixture!("e2e-62-3"),
            _ => fixture!("e2e-62-4"),
        };
        let (reply, _) = h.ask(text, None, replies);
        assert_eq!(reply.outcome, AskOutcome::Answered, "{text}: {reply:#?}");
        let figs = figures(answer_of(&reply));
        assert_eq!(balance(figs[0].0), (2_880, BalanceSide::Dr), "{text}");
        assert!(figs[0].1, "{text}");
    }
    h.scan_requests();
}

// ---- E2E-63 ---------------------------------------------------------------------------------

#[test]
fn e2e_63_whom_to_collect_from_first_is_advice_from_r20_sorted_by_days_then_balance() {
    let h = Harness::day();
    // Today 15-11-26, nothing entered after Events 1 to 5.
    common::set_today(&h.world.env, 2026, 11, 15);
    let (reply, _) = h.ask("kis se pehle wasooli karun", None, fixture!("e2e-63"));
    assert_eq!(reply.outcome, AskOutcome::Advice, "{reply:#?}");
    let answer = answer_of(&reply);
    assert!(answer.advice, "the band مشورہ، فیصلہ آپ کا shows");
    let figs = figures(answer);
    // ارشد ملتانی 18,684 ✓, 36 days; then اکرم ٹول ٹیکس 2,880 ✓, 36 days.
    assert_eq!(balance(figs[0].0), (18_684, BalanceSide::Dr));
    assert!(figs[0].1);
    assert_eq!(count(figs[1].0), 36);
    assert_eq!(balance(figs[2].0), (2_880, BalanceSide::Dr));
    assert!(figs[2].1);
    assert_eq!(count(figs[3].0), 36);
    // The R20 total is the tile گاہکوں کے ذمے.
    let table = &answer.tables[0];
    assert_eq!(table.count, 2);
    assert!(
        table
            .totals
            .iter()
            .any(|c| matches!(c, Cell::Money { paisa } if paisa.rupees() == 21_564))
    );
    // One question counted; the log says advice.
    assert_eq!(reply.count_today, 1);
    let row = assistant::get_question(&h.world.env.ctx, reply.question_id)
        .unwrap()
        .unwrap();
    assert_eq!(row.outcome, "advice");
    h.scan_requests();
}

// ---- E2E-59 ---------------------------------------------------------------------------------

/// April to September 2026: the commission of each month is 36,000 ... 51,000.
const HISTORY: [(u32, i64, i64); 6] = [
    (4, 180, 1_600),
    (5, 150, 2_100),
    (6, 120, 2_850),
    (7, 225, 1_600),
    (8, 150, 2_600),
    (9, 173, 2_390),
];

fn month_of(part: &AnswerPart) -> Option<(i64, i64)> {
    match part {
        AnswerPart::Month { year, month } => Some((*year, *month)),
        _ => None,
    }
}

#[test]
fn e2e_59_estimates_work_from_the_full_months_and_say_so() {
    let w = worked::history_world(&HISTORY);
    let h = Harness::over(w);
    let before = row_counts(&h.world.dir);
    // Same pace and same growth, six months ahead.
    let (reply, _) = h.ask(
        "agar commission isi raftar se chalta raha to agle 6 mahine mein kitna hoga",
        None,
        fixture!("e2e-59a"),
    );
    assert_eq!(reply.outcome, AskOutcome::Estimate, "{reply:#?}");
    let answer = answer_of(&reply);
    assert!(
        answer.estimate.is_some(),
        "the band اندازہ، یقینی نہیں shows"
    );
    assert_eq!(answer.charts.len(), 1);
    let months: Vec<(i64, i64)> = answer.parts.iter().filter_map(month_of).collect();
    // October 2026 to March 2027.
    assert_eq!(months, [(2026, 10), (2027, 3)]);
    let figs = figures(answer);
    assert_eq!(count(figs[0].0), 3, "the last 3 full months");
    assert_eq!(
        money(figs[1].0),
        288_000,
        "same pace: 48,000 a month for 6 months"
    );
    assert_eq!(money(figs[2].0), 48_000);
    assert_eq!(
        money(figs[3].0),
        3_000,
        "same growth: 3,000 more each month"
    );
    assert_eq!(money(figs[4].0), 369_000);
    let methods = &answer.estimate.as_ref().unwrap().methods;
    assert_eq!(methods.len(), 2, "same season needs 12 full months");
    // Nothing was saved: only the question log grew.
    let after = row_counts(&h.world.dir);
    for ((name, was), (_, now)) in before.iter().zip(after.iter()) {
        let expected = if name == "assistant_questions" {
            was + 1
        } else {
            *was
        };
        assert_eq!(*now, expected, "table {name}");
    }
    // The target of 3 lakh: same pace April 2027, same growth February 2027.
    let (target, _) = h.ask(
        "commission ka 3 lakh ka target kab pura hoga",
        None,
        fixture!("e2e-59b"),
    );
    assert_eq!(target.outcome, AskOutcome::Estimate, "{target:#?}");
    let months: Vec<(i64, i64)> = answer_of(&target)
        .parts
        .iter()
        .filter_map(month_of)
        .collect();
    assert_eq!(months, [(2027, 4), (2027, 2)]);
    h.scan_requests();
}

#[test]
fn e2e_59_with_two_full_months_the_answer_says_so_and_has_no_estimate_band() {
    let w = worked::history_world(&HISTORY[4..]);
    let h = Harness::over(w);
    let (reply, _) = h.ask(
        "agar commission isi raftar se chalta raha to agle 6 mahine mein kitna hoga",
        None,
        fixture!("e2e-59c"),
    );
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    let answer = answer_of(&reply);
    assert!(answer.estimate.is_none(), "no estimate band");
    assert_eq!(count(figures(answer)[0].0), 2);
    h.scan_requests();
}

// ---- P2, the tiles part ---------------------------------------------------------------------

/// The tile keys of the line "tiles: today_sales (amount, count), today_commission, ...".
fn tile_keys_named() -> BTreeSet<String> {
    let line = SYSTEM
        .lines()
        .find(|l| l.starts_with("tiles:"))
        .expect("the tiles line");
    let body = line
        .trim_start_matches("tiles:")
        .split('.')
        .next()
        .unwrap_or("");
    let mut depth = 0usize;
    let mut cleaned = String::new();
    for c in body.chars() {
        match c {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            _ if depth == 0 => cleaned.push(c),
            _ => {}
        }
    }
    cleaned
        .split(',')
        .map(|k| k.trim().to_owned())
        .filter(|k| !k.is_empty())
        .collect()
}

#[test]
fn p2_the_tile_keys_of_the_prompt_are_the_tiles_of_the_tool_both_ways() {
    use mandi_assistant::results::Words;
    use mandi_assistant::tools::{Env, State, call};
    let h = Harness::day();
    let words = Words::new().unwrap();
    let env = Env {
        ctx: &h.world.env.ctx,
        actor: &h.world.owner,
        words: &words,
        language: mandi_services::core::auth::Language::Ur,
        today: mandi_services::core::dates::BusinessDate::parse_iso("2026-10-10").unwrap(),
        partner_locked: false,
        numbers: &[],
        backup: BackupFacts {
            cloud_on: false,
            cloud_age_hours: None,
        },
    };
    let draft = call(&env, &State::default(), "tiles", "{}").unwrap();
    let mandi_assistant::results::Node::Obj(fields) = &draft.node else {
        panic!("the tiles are an object");
    };
    let in_tool: BTreeSet<String> = fields.iter().map(|(k, _)| k.clone()).collect();
    assert_eq!(tile_keys_named(), in_tool);
}
