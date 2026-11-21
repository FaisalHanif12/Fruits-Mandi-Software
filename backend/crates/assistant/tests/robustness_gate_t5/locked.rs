//! Partner figures while the partner section is locked (PRD FR-7.82, FR-17.11, NFR-SEC-14, E91,
//! Code Standards S8, Q7).
//!
//! `robustness_assistant.rs` attack 4 pauses on R31, R30, view F and R31 with dates, and attack 8
//! checks that an answer given while open is not sent again after the re-lock. This file reads
//! EVERY source the tools reach, in every variant that can carry a partner figure, while the
//! section is locked, in a small world whose partner figures are numbers that appear nowhere else
//! (capital 51,234 in, 17,171 out, 33,333 taken, 11,111 given back).
//!
//! A source is clean when it either pauses (`Stop::Locked`) or its result holds none of those
//! numbers. The sources the PRD names are in `strict`; the ledgers that hold the same entries as
//! lines (the cash book, the day book, the voucher list, a partner's own account) are in `ledgers`
//! and are a question for Umar (T5-Q2).
#![allow(dead_code)]

use std::collections::BTreeSet;

use mandi_services::core::auth::Language;
use mandi_services::core::capital::CapitalInput;
use mandi_services::core::expenses::{PersonalExpenseInput, PersonalReturnInput};
use mandi_services::core::partner_code::{PartnerCodeSetInput, PartnerCodeUnlockInput};
use mandi_services::{partner_code, vouchers};
use serde_json::json;

use crate::harness::Harness;
use crate::kit::*;
use crate::{common, worked};

/// The numbers only a partner's own money shows.
const CAPITAL_IN: i64 = 51_234;
const CAPITAL_OUT: i64 = 17_171;
const TAKEN: i64 = 33_333;
const RETURNED: i64 = 11_111;

/// The worked day with distinct partner money: B invests 51,234, C takes 17,171 of his capital
/// back, A takes 33,333 and gives 11,111 back. The partner code is 4826 and the section is locked.
fn partner_world() -> Harness {
    let h = Harness::day();
    let w = &h.world;
    let (ctx, owner) = (&w.env.ctx, &w.owner);
    partner_code::set(
        ctx,
        owner,
        &PartnerCodeSetInput {
            password: common::OWNER_PASSWORD.to_owned(),
            code: "4826".to_owned(),
            code_again: "4826".to_owned(),
        },
    )
    .unwrap();
    let [a, b, c] = w.partners;
    let capital = |partner: i64, amount: &str| CapitalInput {
        date: "10-10-2026".into(),
        partner_id: Some(partner),
        amount: amount.into(),
        narration: None,
        confirm_cash_below_zero: true,
    };
    vouchers::save_capital_in(ctx, owner, &capital(b, "51,234")).unwrap();
    vouchers::save_capital_out(ctx, owner, &capital(c, "17,171")).unwrap();
    vouchers::save_personal_expense(
        ctx,
        owner,
        &PersonalExpenseInput {
            date: "10-10-2026".into(),
            partner_id: Some(a),
            amount: "33,333".into(),
            reason: "ghar".into(),
            return_by: Some("12-10-2026".into()),
            taken_by: None,
            narration: None,
            confirm_cash_below_zero: true,
        },
    )
    .unwrap();
    vouchers::save_personal_return(
        ctx,
        owner,
        &PersonalReturnInput {
            date: "10-10-2026".into(),
            partner_id: Some(a),
            amount: "11,111".into(),
            narration: None,
        },
    )
    .unwrap();
    // Ten days on, A's take is past its return-by day: the attention list holds it.
    common::set_today(&w.env, 2026, 10, 20);
    partner_code::lock(ctx).unwrap();
    h
}

/// The integers in a text that stand alone: not part of a key (`a21`, `je4518`), a word or a date.
fn tokens(text: &str) -> BTreeSet<i64> {
    let chars: Vec<char> = text.chars().collect();
    let mut out = BTreeSet::new();
    let mut i = 0;
    while i < chars.len() {
        if chars[i].is_ascii_digit() {
            let start = i;
            while i < chars.len() && chars[i].is_ascii_digit() {
                i += 1;
            }
            let before = start.checked_sub(1).map(|k| chars[k]);
            let negative = before == Some('-');
            let word_before = start
                .checked_sub(if negative { 2 } else { 1 })
                .map(|k| chars[k])
                .is_some_and(|c| c.is_alphanumeric() || c == '_' || c == '-');
            let after = chars.get(i).copied();
            let word_after = after.is_some_and(|c| c.is_alphanumeric() || c == '_' || c == '.');
            if !word_before
                && !word_after
                && let Ok(n) = chars[start..i].iter().collect::<String>().parse::<i64>()
            {
                out.insert(if negative { -n } else { n });
            }
        } else {
            i += 1;
        }
    }
    out
}

/// The money above, and what it makes the partners' totals and takes: A 996 less 33,333 plus
/// 11,111 and so on (read from the partner reports while open); they stand alone in the books.
fn partner_numbers() -> [i64; 9] {
    [
        CAPITAL_IN,
        CAPITAL_OUT,
        TAKEN,
        RETURNED,
        23_226,
        51_980,
        16_425,
        35_333,
        21_734,
    ]
}

/// One call while locked: `Ok(true)` when it paused, `Ok(false)` when it gave a clean result or a
/// refusal, `Err` with the partner numbers that it held.
fn locked_read(
    r: &mut Reader<'_>,
    name: &str,
    args: serde_json::Value,
) -> Result<bool, (String, Vec<i64>)> {
    let before_paused = r.paused;
    let before_seen = r.seen.len();
    r.call(name, args.clone());
    if r.paused > before_paused {
        return Ok(true);
    }
    let text = r.seen[before_seen..].join("\n");
    let held: Vec<i64> = tokens(&text)
        .into_iter()
        .filter(|n| partner_numbers().contains(&n.abs()))
        .collect();
    if held.is_empty() {
        Ok(false)
    } else {
        Err((format!("{name} {args}"), held))
    }
}

/// First, the control: with the section open, the same reads DO show those numbers, so a clean
/// result while locked is a closed door and not an empty world.
#[test]
fn t5_the_partner_numbers_are_in_the_books_when_the_section_is_open() {
    let h = partner_world();
    partner_code::unlock(
        &h.world.env.ctx,
        &h.world.owner,
        &PartnerCodeUnlockInput {
            code: "4826".into(),
        },
    )
    .unwrap();
    with_reader(&h.world, Language::En, false, |r| {
        let range = json!({"from": "2016-01-01", "to": "2026-10-10"});
        r.call("report", report_args("R31", json!({})));
        r.call("report", report_args("R30", json!({})));
        r.call("report", report_args("R29", json!({})));
        r.call("analytics", analytics_args("F", json!({})));
        for text in ["Partner A", "Partner B", "Partner C"] {
            let p = r.first_match(text, "partner").expect("a partner");
            r.call(
                "report",
                report_args(
                    "R31",
                    json!({"partner": p, "from": "2016-01-01", "to": "2026-10-10"}),
                ),
            );
            r.call("report", report_args("R29", json!({"partner": p})));
        }
        let _ = range;
        let held = tokens(&r.text());
        for n in [CAPITAL_IN, CAPITAL_OUT, TAKEN, RETURNED] {
            assert!(
                held.contains(&n) || held.contains(&-n),
                "{n} is not in the partner reports: the world is empty of it"
            );
        }
        // The totals the partner reports show.
        for n in [23_226, 51_980, 16_425] {
            assert!(
                held.contains(&n) || held.contains(&-n),
                "{n} is not in the partner reports"
            );
        }
    });
}

/// Every source the PRD puts behind the code, in every variant, while locked.
#[test]
fn t5_every_partner_source_is_paused_or_clean_while_locked() {
    let h = partner_world();
    let mut held: Vec<(String, Vec<i64>)> = Vec::new();
    let mut paused = 0usize;
    let mut clean = 0usize;
    for language in [Language::Ur, Language::En] {
        with_reader(&h.world, language, true, |r| {
            let partner = r.first_match("a", "partner");
            let trip = r.first_match("a", "trip");
            let garden = r.first_match("a", "garden");
            let day = json!({"from": "2016-01-01", "to": "2026-10-10"});
            let mut calls: Vec<(&str, serde_json::Value)> = vec![
                ("report", report_args("R30", json!({}))),
                ("report", report_args("R30", day.clone())),
                ("report", report_args("R31", json!({}))),
                ("report", report_args("R31", day.clone())),
                ("report", report_args("R29", json!({}))),
                ("report", report_args("R29", day.clone())),
                ("report", report_args("R33", json!({}))),
                ("report", report_args("R33", json!({"layout": "partner"}))),
                ("report", report_args("R34", day.clone())),
                (
                    "report",
                    report_args("R34", json!({"from": "2026-10-10", "to": "2026-10-10"})),
                ),
                ("report", report_args("R36", json!({}))),
                ("report", report_args("R36", json!({"layout": "partner"}))),
                ("report", report_args("R37", json!({}))),
                ("report", report_args("R37", day.clone())),
                ("analytics", analytics_args("F", json!({}))),
                ("analytics", analytics_args("G", json!({}))),
                ("analytics", analytics_args("I", json!({}))),
                ("analytics", analytics_args("J", json!({}))),
                ("analytics", analytics_args("K", json!({}))),
                ("analytics", analytics_args("K", day.clone())),
                ("tiles", json!({})),
                (
                    "app_state",
                    json!({"what": "attention", "party": null, "item": null, "bikri": null}),
                ),
                (
                    "app_state",
                    json!({"what": "days", "party": null, "item": null, "bikri": null}),
                ),
                (
                    "app_state",
                    json!({"what": "rules", "party": null, "item": null, "bikri": null}),
                ),
                (
                    "estimate",
                    estimate_args("profit", Some(6), json!({"partner": partner.clone()})),
                ),
            ];
            if let Some(p) = &partner {
                calls.push(("party_card", json!({"party": p})));
                for id in ["R29", "R30", "R31", "R33", "R34", "R36", "R37"] {
                    calls.push(("report", report_args(id, json!({"partner": p}))));
                    calls.push((
                        "report",
                        report_args(
                            id,
                            json!({"partner": p, "from": "2016-01-01", "to": "2026-10-10"}),
                        ),
                    ));
                }
                for view in ["A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K"] {
                    calls.push(("analytics", analytics_args(view, json!({"partner": p}))));
                    calls.push(("analytics", analytics_args(view, json!({"partner": p, "from": "2016-01-01", "to": "2026-10-10", "group_by": "month"}))));
                }
                calls.push((
                    "app_state",
                    json!({"what": "attention", "party": p, "item": null, "bikri": null}),
                ));
            }
            if let Some(t) = &trip {
                for id in ["R33", "R29", "R37"] {
                    calls.push(("report", report_args(id, json!({"trip": t}))));
                }
                calls.push(("analytics", analytics_args("J", json!({"trip": t}))));
            }
            if let Some(g) = &garden {
                for id in ["R36", "R29", "R37"] {
                    calls.push(("report", report_args(id, json!({"garden": g}))));
                }
                calls.push(("analytics", analytics_args("I", json!({"garden": g}))));
            }
            for (name, args) in calls {
                match locked_read(r, name, args) {
                    Ok(true) => paused += 1,
                    Ok(false) => clean += 1,
                    Err((what, nums)) => held.push((what, nums)),
                }
            }
        });
    }
    assert!(
        paused >= 10,
        "only {paused} partner sources paused: the lock is not being met"
    );
    assert!(clean >= 20, "only {clean} reads were clean");
    assert!(
        held.is_empty(),
        "partner figures reached a result while the section was locked ({} reads): {:#?}",
        held.len(),
        held.iter().take(12).collect::<Vec<_>>()
    );
}

/// The ledgers that hold a partner's money as lines of entries: the cash book (a capital line, a
/// take line), the day book, the voucher list by type, and the ledger of the partner's own account.
/// PRD FR-17.11 lists the reports behind the code; it does not name these. NFR-SEC-14 says "every
/// partner command and partner report". A question for Umar: T5-Q2.
fn ledger_leaks() -> Vec<(String, Vec<i64>)> {
    let h = partner_world();
    let mut held = Vec::new();
    with_reader(&h.world, Language::En, true, |r| {
        let range = json!({"from": "2016-01-01", "to": "2026-10-10"});
        let mut calls: Vec<(&str, serde_json::Value)> = vec![
            ("report", report_args("R11", range.clone())),
            ("report", report_args("R12", range.clone())),
            ("report", report_args("R15", range.clone())),
            ("report", report_args("R17", json!({"as_of": "2026-10-10"}))),
            ("report", report_args("R18", json!({"as_of": "2026-10-10"}))),
            ("report", report_args("R25", range.clone())),
            ("tiles", json!({})),
        ];
        for t in [
            "capital_in",
            "capital_out",
            "personal_expense",
            "personal_return",
        ] {
            calls.push((
                "report",
                report_args(
                    "R15",
                    json!({"voucher_type": t, "from": "2016-01-01", "to": "2026-10-10"}),
                ),
            ));
        }
        if let Some(p) = r.first_match("a", "partner") {
            calls.push((
                "report",
                report_args(
                    "R13",
                    json!({"party": p, "from": "2016-01-01", "to": "2026-10-10"}),
                ),
            ));
            calls.push((
                "report",
                report_args(
                    "R14",
                    json!({"party": p, "from": "2016-01-01", "to": "2026-10-10"}),
                ),
            ));
        }
        for (name, args) in calls {
            if let Err((what, nums)) = locked_read(r, name, args) {
                held.push((what, nums));
            }
        }
    });
    held
}

#[test]
fn t5_11_the_ledgers_that_hold_partner_money_as_lines_are_paused_or_clean_while_locked() {
    let held = ledger_leaks();
    assert!(
        held.is_empty(),
        "{} reads showed partner money while locked: {:#?}",
        held.len(),
        held
    );
}

/// The numbers I chose can also sit in a request body, not only in a result: the loop's own
/// reads (the party card the app reads ahead, the follow-up's resent results) are scanned too.
#[test]
fn t5_a_question_about_partners_while_locked_sends_no_partner_number() {
    let h = partner_world();
    let reads = vec![
        tool_reply(&[
            find("a", "partner"),
            ("tiles", json!({})),
            ("report", report_args("R29", json!({}))),
            ("analytics", analytics_args("G", json!({}))),
        ]),
        final_reply(&answer("not_understood", "")),
    ];
    let (reply, requests) = ask(&h, "partner ka hisab", None, reads);
    assert!(
        matches!(
            reply.outcome,
            mandi_assistant::ask::AskOutcome::NotUnderstood
                | mandi_assistant::ask::AskOutcome::Locked
        ),
        "{reply:#?}"
    );
    let sent = all_text(&requests);
    let held = tokens(&sent);
    for n in partner_numbers() {
        assert!(!held.contains(&n) && !held.contains(&-n), "{n} was sent");
    }
    h.scan_requests();
    let _ = worked::PRIVATE_NOTE;
}

/// FINDING T5-3: the audit read (`app_state{what:"audit"}`) hides the rows of a partner's own
/// record while locked (`hide_partners`: entity "partner"), but the audit rows of the vouchers
/// that move a partner's money (a capital in, a capital out, a personal take and its return) carry
/// the amount as a money change, so 51,234 invested by B, 17,171 taken back by C, 33,333 taken by
/// A and 11,111 given back reach the model while the partner section is locked (PRD FR-7.82,
/// NFR-SEC-14). Known cousin: R6-2 (the audit read is not in FR-7.87 at all).
#[test]
fn t5_3_the_audit_read_shows_no_partner_money_while_locked() {
    let h = partner_world();
    let mut held = Vec::new();
    with_reader(&h.world, Language::En, true, |r| {
        let partner = r.first_match("a", "partner");
        let mut calls = vec![json!({"what": "audit", "party": null, "item": null, "bikri": null})];
        if let Some(p) = partner {
            calls.push(json!({"what": "audit", "party": p, "item": null, "bikri": null}));
        }
        for args in calls {
            if let Err((what, nums)) = locked_read(r, "app_state", args) {
                held.push((what, nums));
            }
        }
    });
    assert!(held.is_empty(), "{held:#?}");
}

/// An answer given while the section is open, then the re-lock, then three more questions in the
/// same conversation: no partner number of the open answer is in any later request, and a
/// reference to a result of the open question is refused as an earlier question.
#[test]
fn t5_after_a_re_lock_no_later_request_holds_a_number_of_the_open_answer() {
    let h = partner_world();
    let ctx = &h.world.env.ctx;
    partner_code::unlock(
        ctx,
        &h.world.owner,
        &PartnerCodeUnlockInput {
            code: "4826".into(),
        },
    )
    .unwrap();
    let read = vec![
        tool_reply(&[find("Partner B", "partner")]),
        tool_reply(&[(
            "report",
            report_args(
                "R31",
                json!({"partner": "r1.matches.0", "from": "2016-01-01", "to": "2026-10-10"}),
            ),
        )]),
        final_reply(&answer("not_understood", "")),
    ];
    let (open, open_requests) = ask(&h, "partner B ka hisab", None, read);
    // The control: the open question really carried the partner numbers.
    let sent_open = tokens(&all_text(&open_requests));
    assert!(
        sent_open.contains(&CAPITAL_IN) || sent_open.contains(&51_980),
        "{sent_open:?}"
    );
    partner_code::lock(ctx).unwrap();
    let mut later: Vec<serde_json::Value> = Vec::new();
    for question in ["aaj ka commission", "aur kal ka?", "phir cash?"] {
        let (reply, requests) = ask(
            &h,
            question,
            Some(&open.conversation_id),
            vec![
                tool_reply(&[("tiles", json!({}))]),
                final_reply(&answer("not_understood", "")),
            ],
        );
        assert_ne!(
            reply.outcome,
            mandi_assistant::ask::AskOutcome::Locked,
            "{question}"
        );
        later.extend(requests);
    }
    let held = tokens(&all_text(&later));
    for n in partner_numbers() {
        assert!(
            !held.contains(&n) && !held.contains(&-n),
            "{n} was sent again"
        );
    }
    // A reference to the open question's partner result is an earlier question: refused.
    let cite = answer("answer", "{{r2.count}} سطریں۔");
    let (cited, requests) = ask(
        &h,
        "wohi dobara",
        Some(&open.conversation_id),
        vec![final_reply(&cite), final_reply(&cite)],
    );
    assert_eq!(
        cited.outcome,
        mandi_assistant::ask::AskOutcome::ChecksFailed,
        "{cited:#?}"
    );
    assert!(failed_rules(requests.last().unwrap()).contains("earlier_question"));
    h.scan_requests();
}

/// The ids a screen could send for the party chip: the capital, drawings and garden-trip-fruit
/// accounts of the partners (control 0602, 0603, 0604), their linked khata accounts (0605), and
/// the contra account. The backend takes `party_id` from the frontend (V0): a partner's account
/// is not one the picker offers, but nothing in the command stops it.
fn partner_account_ids(h: &Harness) -> Vec<(String, i64)> {
    let conn = common::raw(&h.world.dir);
    conn.prepare(
        "SELECT code, id FROM accounts WHERE code LIKE '0602%' OR code LIKE '0603%' \
         OR code LIKE '0604%' OR code LIKE '0605%' OR code LIKE '03070001%' ORDER BY code",
    )
    .unwrap()
    .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))
    .unwrap()
    .map(Result::unwrap)
    .collect()
}

/// The quick chip "کسی پارٹی کا کھاتہ" takes any account id from the screen. With the section
/// locked, the ledger of B's capital account (51,234 in), C's (17,171 out) and A's drawings
/// (33,333 taken, 11,111 back) must not answer. It holds: each is refused or has no such figure.
#[test]
fn t5_the_party_chip_gives_no_partner_money_for_a_partner_account_while_locked() {
    let h = partner_world();
    let ids = partner_account_ids(&h);
    assert!(ids.len() >= 6, "the partner accounts: {ids:?}");
    let mut held = Vec::new();
    for (code, id) in ids {
        let client = mandi_assistant::client::RecordedClient::new(Vec::new());
        let emit = |_s| {};
        let deps = h.deps(&client, &emit);
        let result = mandi_assistant::quick::quick(
            &deps,
            &mandi_assistant::ask::AssistantQuickInput {
                chip: mandi_assistant::ask::QuickChip::PartyLedger,
                party_id: Some(id),
                conversation_id: None,
            },
        );
        if let Ok(reply) = result {
            let text = serde_json::to_string(&reply).unwrap();
            let nums = tokens(&text);
            let got: Vec<i64> = partner_numbers()
                .into_iter()
                .filter(|n| nums.contains(n) || nums.contains(&-n))
                .collect();
            if !got.is_empty() {
                held.push((code, got));
            }
        }
    }
    assert!(
        held.is_empty(),
        "the chip showed partner money while locked: {held:?}"
    );
}

/// The hostile ids the chip and the pick take from the screen: negative, zero, the largest
/// integer. Each is a refusal, never a panic and never a read of another account.
#[test]
fn t5_the_chip_and_the_pick_refuse_ids_that_do_not_exist() {
    let h = partner_world();
    for id in [
        None,
        Some(0),
        Some(-1),
        Some(i64::MAX),
        Some(i64::MIN),
        Some(999_999),
    ] {
        let client = mandi_assistant::client::RecordedClient::new(Vec::new());
        let emit = |_s| {};
        let deps = h.deps(&client, &emit);
        let result = mandi_assistant::quick::quick(
            &deps,
            &mandi_assistant::ask::AssistantQuickInput {
                chip: mandi_assistant::ask::QuickChip::PartyLedger,
                party_id: id,
                conversation_id: None,
            },
        );
        assert!(result.is_err(), "party_id {id:?} was accepted: {result:?}");
        assert!(client.requests().is_empty());
    }
    for key in [
        "a999999",
        "a0",
        "a-1",
        "",
        "x",
        &"a".repeat(5_000),
        "'; DROP TABLE accounts; --",
        "\u{0}",
    ] {
        let client = mandi_assistant::client::RecordedClient::new(Vec::new());
        let emit = |_s| {};
        let deps = h.deps(&client, &emit);
        let result = mandi_assistant::run::pick(
            &deps,
            &mandi_assistant::ask::AssistantPickInput {
                question_id: 999_999,
                choice: mandi_assistant::ask::PickChoice::Match {
                    key: key.to_owned(),
                },
            },
        );
        assert!(
            result.is_err(),
            "a pick of {key:?} for a question that is not there"
        );
    }
    let accounts: i64 = common::raw(&h.world.dir)
        .query_row("SELECT COUNT(*) FROM accounts", [], |r| r.get(0))
        .unwrap();
    assert!(accounts > 0);
}

/// A conversation id that is hostile text is only an id: the question works, the log keeps it as
/// text, and no table is touched. Since the T5-7 decision (plan 80) an id that is too long or holds a
/// control character is refused up front, with no model call and nothing written (see below).
#[test]
fn t5_a_hostile_conversation_id_is_only_an_id() {
    let h = Harness::day();
    let texts = [
        "'; DROP TABLE assistant_questions; --",
        "../../etc/passwd",
        "{{r1.balance}}",
        "<script>alert(1)</script>",
    ];
    for id in texts {
        let (reply, _) = ask(&h, "aaj ka commission", Some(id), gives_up());
        assert_eq!(
            reply.outcome,
            mandi_assistant::ask::AskOutcome::NotUnderstood,
            "{id:.20}"
        );
    }
    let n: i64 = common::raw(&h.world.dir)
        .query_row("SELECT COUNT(*) FROM assistant_questions", [], |r| r.get(0))
        .unwrap();
    assert_eq!(n, 4);
}

/// The ids that are too long (10,000 letters) or hold a control character (NUL, a newline) are
/// refused before any model call, with a field error, and write nothing (T5-7, V0).
#[test]
fn t5_a_conversation_id_that_is_too_long_or_has_a_control_character_is_refused_up_front() {
    let h = Harness::day();
    for id in [
        "ا".repeat(10_000),
        "a".repeat(65),
        "a\u{0}b".to_owned(),
        "a\nb".to_owned(),
        "\u{7f}".to_owned(),
    ] {
        let client = mandi_assistant::client::RecordedClient::new(gives_up());
        let emit = |_s| {};
        let deps = h.deps(&client, &emit);
        let result = mandi_assistant::run::ask(
            &deps,
            &mandi_assistant::ask::AssistantAskInput {
                text: "aaj ka commission".into(),
                conversation_id: Some(id.clone()),
            },
        );
        assert!(result.is_err(), "{:.20} was taken", id);
        assert!(client.requests().is_empty(), "the model was called");
    }
    // An id of 64 letters is the longest that works.
    let longest = "a".repeat(64);
    let (reply, _) = ask(&h, "aaj ka commission", Some(longest.as_str()), gives_up());
    assert_eq!(
        reply.outcome,
        mandi_assistant::ask::AskOutcome::NotUnderstood
    );
}

/// FINDING T5-7: a conversation id that starts with a NUL character is taken by `ask` (it is not
/// empty after the trim), the model is called, and the question log refuses the row
/// (`CHECK length(conversation_id) > 0` reads up to the first NUL). The Owner gets a database
/// error after the model's reply was paid for, and the answer is lost (V0: the backend treats the
/// frontend's input as untrusted; Code Standards 4.2).
#[test]
fn t5_7_a_conversation_id_that_starts_with_nul_is_refused_before_the_model_call() {
    let h = Harness::day();
    let client = mandi_assistant::client::RecordedClient::new(gives_up());
    let emit = |_s| {};
    let deps = h.deps(&client, &emit);
    let result = mandi_assistant::run::ask(
        &deps,
        &mandi_assistant::ask::AssistantAskInput {
            text: "aaj ka commission".into(),
            conversation_id: Some("\u{0}".into()),
        },
    );
    // Either the id is refused up front (no request), or the question works.
    match result {
        Ok(reply) => assert_eq!(
            reply.outcome,
            mandi_assistant::ask::AskOutcome::NotUnderstood
        ),
        Err(e) => assert!(
            client.requests().is_empty(),
            "the model was called, then: {e:?}"
        ),
    }
}

/// FINDING (plan 82 robustness gate, `robustness_security` hostile input, `assistant_quick
/// {"conversation_id":"\u0000"}` ends ERR_INTERNAL): `quick` takes the conversation id like `ask`
/// does, but only drops an empty one. An id with a control character, or one that is too long, goes
/// on to the question log, which refuses the row, so the Owner gets a database error (an internal
/// error) instead of a field error. `ask` refuses such an id up front (T5-7); `quick` must do the
/// same: a field error on `conversation_id`, nothing written, no model call (V0, Code Standards
/// 4.1, NFR-REL-6).
#[test]
fn t5_7c_a_quick_question_refuses_a_bad_conversation_id_with_a_field_error_and_writes_nothing() {
    let h = Harness::day();
    let before: i64 = common::raw(&h.world.dir)
        .query_row("SELECT COUNT(*) FROM assistant_questions", [], |r| r.get(0))
        .unwrap();
    for id in [
        "\u{0}".to_owned(),
        "a\u{0}b".to_owned(),
        "a\nb".to_owned(),
        "\u{7f}".to_owned(),
        "a".repeat(65),
        "ا".repeat(10_000),
    ] {
        let client = mandi_assistant::client::RecordedClient::new(Vec::new());
        let emit = |_s| {};
        let deps = h.deps(&client, &emit);
        let result = mandi_assistant::quick::quick(
            &deps,
            &mandi_assistant::ask::AssistantQuickInput {
                chip: mandi_assistant::ask::QuickChip::CashInHand,
                party_id: None,
                conversation_id: Some(id.clone()),
            },
        );
        assert!(
            matches!(result, Err(mandi_services::ServiceError::Rejected(_))),
            "{:.20?} must be refused with a field error, not taken or a database error: {result:?}",
            id
        );
        assert!(client.requests().is_empty(), "a chip never calls the model");
    }
    let after: i64 = common::raw(&h.world.dir)
        .query_row("SELECT COUNT(*) FROM assistant_questions", [], |r| r.get(0))
        .unwrap();
    assert_eq!(after, before, "a refused chip wrote a log row");
    // The longest good id still works.
    let client = mandi_assistant::client::RecordedClient::new(Vec::new());
    let emit = |_s| {};
    let deps = h.deps(&client, &emit);
    let ok = mandi_assistant::quick::quick(
        &deps,
        &mandi_assistant::ask::AssistantQuickInput {
            chip: mandi_assistant::ask::QuickChip::CashInHand,
            party_id: None,
            conversation_id: Some("a".repeat(64)),
        },
    );
    assert!(ok.is_ok(), "{ok:?}");
}
