//! The finding R6-x of plan 80 (Section 5, "R6-x"): with money lent open, a model answer for
//! "kul ugrahi" that cites the tile `buyers_dues` shows a checked figure that is smaller than the
//! shop's total dues (PRD FR-7.76, E2E-51, glossary: کل اگراہی is the tile `total_dues`; since
//! v1.44 it includes the open money lent).
#![allow(dead_code)]

use mandi_assistant::ask::AskOutcome;
use mandi_services::core::lent::{LentPersonCreateInput, MoneyLentInput};
use mandi_services::{lent, vouchers};
use serde_json::json;

use crate::harness::{Harness, answer_of, figures, money};
use crate::kit::*;

/// The worked day and 5,000 lent to Aslam (staff): buyers owe 21,564, the shop is owed 26,564.
fn lent_day() -> Harness {
    let h = Harness::day();
    let w = &h.world;
    let person = lent::person_create(
        &w.env.ctx,
        &w.owner,
        &LentPersonCreateInput {
            name_ur: "اسلم".into(),
            name_en: "Aslam".into(),
            phone: None,
            staff_id: None,
            confirmed_rules_words: vec!["اسلم".into(), "Aslam".into()],
        },
    )
    .unwrap();
    vouchers::save_money_lent(
        &w.env.ctx,
        &w.owner,
        &MoneyLentInput {
            date: "10-10-2026".into(),
            person_account_id: Some(person.account.id),
            amount: "5,000".into(),
            reason: "ghar".into(),
            return_by: None,
            narration: None,
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    h
}

/// The model reads the tiles and answers "kul ugrahi" with the given tile.
fn kul_ugrahi_with(tile: &str) -> mandi_assistant::ask::AskReply {
    let h = lent_day();
    let text = format!("کل اگراہی {{{{r1.{tile}.amount}}}} روپے ہے۔");
    let (reply, _) = ask(
        &h,
        "kul ugrahi kitni hai",
        None,
        vec![
            tool_reply(&[("tiles", json!({}))]),
            final_reply(&answer("answer", &text)),
            final_reply(&answer("answer", &text)),
        ],
    );
    reply
}

/// The control: citing `total_dues` gives 26,564 (21,564 of the buyers and 5,000 lent), checked.
#[test]
fn t5_kul_ugrahi_from_the_total_dues_tile_is_the_whole_total() {
    let reply = kul_ugrahi_with("total_dues");
    assert_eq!(reply.outcome, AskOutcome::Answered, "{reply:#?}");
    let figs = figures(answer_of(&reply));
    assert_eq!(money(figs[0].0), 26_564);
    assert!(figs[0].1);
}

/// R6-x: the same answer citing `buyers_dues` is shown as کل اگراہی 21,564 ✓: a checked figure that
/// is 5,000 short of what the shop is owed. The tile result now says which tile is the total, but
/// nothing refuses the other one for this question.
#[test]
fn t5_r6_x_kul_ugrahi_is_never_answered_with_the_buyers_dues_alone() {
    let reply = kul_ugrahi_with("buyers_dues");
    if reply.outcome == AskOutcome::Answered {
        let figs = figures(answer_of(&reply));
        assert_ne!(
            money(figs[0].0),
            21_564,
            "the buyers' dues were shown as the total dues"
        );
    }
}

/// R6-x, the tiles tool: it still sends the retired `last_backup` tile (FR-9.1 v1.41 replaced it).
#[test]
fn t5_r6_x_the_tiles_tool_has_the_tiles_the_dashboard_has() {
    let h = lent_day();
    with_reader(
        &h.world,
        mandi_services::core::auth::Language::En,
        false,
        |r| {
            r.call("tiles", json!({}));
            let text = r.text();
            assert!(!text.contains("last_backup"), "a retired tile");
            assert!(
                text.contains("cash_sales")
                    || text.contains("today_cash")
                    || text.contains("paid_on_spot"),
                "no tile for آج کی نقدی (FR-9.1 v1.44)"
            );
        },
    );
}
