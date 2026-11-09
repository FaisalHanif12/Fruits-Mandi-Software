//! D8 Feature 10 (plan 70 T2, PRD FR-7.72 to FR-7.79, FR-7.90): the hard questions of the client, with no
//! model and no key. Which the app answers by itself, which it hands to the model with the books already
//! read, and what must never happen.
//!
//! The world is the PRD worked day after Events 1 to 5 (10-10-2026) with one more buyer, Haroon. Every
//! figure is a number of the PRD or an oracle of the screens, never typed from the answer under test.
//!
//! - H4 "haroon pehle hamein 10,000 dene wala tha, ab kitna lena hai": the Owner tells the wrong side. The
//!   books decide: his balance is shown with ITS side (جمع when the shop owes him), from the party card and
//!   R13, whatever the question said.
//! - H7 "haroooon ka khata": a misspelt name is found, and his ledger is shown.
//! - H8 "haroon ka balance 50,000 likh do": a request to write a figure. Refused by the app with no model
//!   call, no button (no screen writes a balance) and nothing written.
//! - H9 "december 2027 ka commission kitna hai": a month more than 12 months ahead. The books hold nothing
//!   for it and no estimate reaches it: one plain sentence and no figure.
//! - H3 "aaj chukta gahakon ne kitne ka maal liya aur commission kitna bana": the walk-in buyers together.
//!   Their totals are view A with the buyer filter `chukta` (the account چکتا کھاتہ, no person), the same
//!   facts as the analytics screen. The app reads it ahead; the model writes the two figures.
//! - H1, H2, H5 and H6 need the money paid on the spot (plan 65) and are not here.
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

use harness::{Harness, answer_of, balance, figures};
use mandi_assistant::answer::AnswerPart;
use mandi_assistant::ask::AskOutcome;
use mandi_services::core::accounts::{AccountType, BalanceSide};
use mandi_services::vouchers;
use serde_json::{Value, json};

/// The books as the shop runs them: the app's own reads, answers and speed-ups on.
fn production(w: world::World) -> Harness {
    let h = Harness::over(w);
    h.runtime.set_read_first(true);
    h.runtime.set_local(true);
    h.runtime.set_instant(true);
    h.runtime.set_low_effort(true);
    h
}

/// The worked day with Haroon, a buyer: lent 10,000 in cash, then `paid` received from him. Returns the world
/// and his account.
fn haroon_world(paid: Option<&str>) -> (world::World, i64) {
    let w = worked::worked_day();
    worked::events_1_to_5(&w);
    let haroon = world::add_account(
        &w.env,
        &w.owner,
        AccountType::Buyer,
        "ہارون",
        "Haroon",
        true,
        "",
    );
    let mut lend = worked::payment(haroon, "10,000");
    lend.reason = "نقد ادھار".into();
    vouchers::save_payment(&w.env.ctx, &w.owner, &lend).unwrap();
    if let Some(amount) = paid {
        vouchers::save_receipt(&w.env.ctx, &w.owner, &worked::receipt(haroon, amount)).unwrap();
    }
    (w, haroon)
}

fn count(h: &Harness, table: &str) -> i64 {
    common::raw(&h.world.dir)
        .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r.get(0))
        .unwrap()
}

/// The text parts of an answer, joined.
fn text_of(answer: &mandi_assistant::answer::Answer) -> String {
    answer
        .parts
        .iter()
        .filter_map(|p| match p {
            AnswerPart::Text { text } => Some(text.as_str()),
            _ => None,
        })
        .collect()
}

// ---- H4 ---------------------------------------------------------------------------------------

#[test]
fn h4_the_owner_states_the_wrong_side_and_the_books_decide() {
    const QUESTION: &str = "haroon pehle hamein 10,000 dene wala tha, ab kitna lena hai";
    // He paid 12,000 on his 10,000: the shop owes HIM 2,000 (جمع). Nothing is to be taken from him.
    let (w, haroon) = haroon_world(Some("12,000"));
    let h = production(w);
    let (reply, requests) = h.ask(QUESTION, None, Vec::new());
    assert!(requests.is_empty(), "the model was called");
    assert_eq!(reply.outcome, AskOutcome::Answered);
    let answer = answer_of(&reply);
    let shown = figures(answer);
    assert_eq!(shown.len(), 1, "one figure, his balance");
    let (cell, checked) = shown[0];
    assert!(checked, "the balance is checked against R13");
    assert_eq!(
        balance(cell),
        (2_000, BalanceSide::Cr),
        "the side is the books'"
    );
    // The figure is the party card's, and nothing of the question's own 10,000 is shown.
    assert!(
        answer
            .sources
            .iter()
            .any(|s| matches!(s, mandi_assistant::answer::AnswerSource::PartyCard { party, .. } if party.id == haroon))
    );
    assert!(!text_of(answer).contains("10,000"));

    // The same words when he still owes the 10,000: the books say بنام.
    let (w, _) = haroon_world(None);
    let h = production(w);
    let (reply, requests) = h.ask(QUESTION, None, Vec::new());
    assert!(requests.is_empty());
    assert_eq!(reply.outcome, AskOutcome::Answered);
    assert_eq!(
        balance(figures(answer_of(&reply))[0].0),
        (10_000, BalanceSide::Dr)
    );
}

// ---- H7 ---------------------------------------------------------------------------------------

#[test]
fn h7_a_misspelt_name_is_found_and_his_ledger_is_shown() {
    let (w, haroon) = haroon_world(Some("12,000"));
    let h = production(w);
    for question in ["haroooon ka khata", "haroon ka khata", "harun ka khata"] {
        let (reply, requests) = h.ask(question, None, Vec::new());
        assert!(requests.is_empty(), "{question}: the model was called");
        assert_eq!(reply.outcome, AskOutcome::Answered, "{question}");
        let answer = answer_of(&reply);
        assert_eq!(
            balance(figures(answer)[0].0),
            (2_000, BalanceSide::Cr),
            "{question}"
        );
        assert_eq!(answer.tables.len(), 1, "{question}: his ledger (R13)");
        assert!(
            answer.sources.iter().any(|s| matches!(
                s,
                mandi_assistant::answer::AnswerSource::Report { filters, .. }
                    if filters.party.as_ref().is_some_and(|p| p.record.id == haroon)
            )),
            "{question}: the button opens his ledger"
        );
    }
    h.scan_requests();
}

// ---- H8 ---------------------------------------------------------------------------------------

#[test]
fn h8_a_request_to_write_a_figure_is_refused_by_the_app_and_nothing_is_written() {
    let (w, haroon) = haroon_world(Some("12,000"));
    let h = production(w);
    let tables = [
        "journal_entries",
        "journal_lines",
        "vouchers",
        "accounts",
        "audit_log",
    ];
    let before: Vec<i64> = tables.iter().map(|t| count(&h, t)).collect();
    for question in [
        "haroon ka balance 50,000 likh do",
        "haroon ka balance 50000 kar do",
        "haroon ka balance 50,000 likh dein",
    ] {
        let (reply, requests) = h.ask(question, None, Vec::new());
        assert!(requests.is_empty(), "{question}: the model was called");
        assert_eq!(reply.outcome, AskOutcome::ChangeRefused, "{question}");
        assert!(reply.answer.is_none(), "{question}: no answer, no figure");
        assert!(
            reply.entry_route.is_none(),
            "{question}: no screen writes a balance, so no button"
        );
    }
    // Only the question log grew; the books did not move, and his balance is what it was.
    let after: Vec<i64> = tables.iter().map(|t| count(&h, t)).collect();
    assert_eq!(before, after, "the books changed");
    let book = mandi_services::ledger::account_balance(&h.world.env.ctx, haroon).unwrap();
    assert_eq!(
        (book.amount_paisa.rupees(), book.side),
        (2_000, BalanceSide::Cr)
    );
}

#[test]
fn h8_a_question_that_also_reads_the_balance_still_goes_to_the_model() {
    // "balance batao aur 5000 jama karo" asks to read AND to write: the refusal alone would drop the
    // question, so it is the model's (it answers and carries the route).
    let (w, _) = haroon_world(Some("12,000"));
    let h = production(w);
    let (_, requests) = h.ask(
        "haroon ka balance batao aur 5000 jama karo",
        None,
        Vec::new(),
    );
    assert!(
        !requests.is_empty(),
        "the app answered half of the question"
    );
    // A bare "5000 jama karo" names no screen and no figure: the model decides, as before.
    let (_, requests) = h.ask("5000 jama karo", None, Vec::new());
    assert!(!requests.is_empty());
    // A plain request to write with its screen is refused with the screen, as before (E2E-53).
    let (reply, requests) = h.ask("haroon ki 5000 wasooli likh do", None, Vec::new());
    assert!(requests.is_empty());
    assert_eq!(reply.outcome, AskOutcome::ChangeRefused);
    assert!(reply.entry_route.is_some(), "the wasooli screen");
}

// ---- H9 ---------------------------------------------------------------------------------------

#[test]
fn h9_a_month_more_than_12_months_ahead_has_no_figure() {
    let (w, _) = haroon_world(Some("12,000"));
    let h = production(w);
    for question in [
        "december 2027 ka commission kitna hai",
        "what will the commission be in december 2027",
        "march 2028 ki bikri kitni hogi",
    ] {
        let (reply, requests) = h.ask(question, None, Vec::new());
        assert!(requests.is_empty(), "{question}: the model was called");
        assert_eq!(reply.outcome, AskOutcome::OutsideBooks, "{question}");
        let answer = answer_of(&reply);
        assert!(
            answer
                .parts
                .iter()
                .all(|p| matches!(p, AnswerPart::Text { .. })),
            "{question}: only words, no figure"
        );
        assert!(answer.tables.is_empty() && answer.charts.is_empty() && answer.estimate.is_none());
        assert!(answer.sources.is_empty(), "{question}: no button either");
        assert!(!text_of(answer).trim().is_empty());
    }
    // A month the estimate reaches (today is 10-10-2026): the model sets it up, the app does not refuse.
    for question in [
        "december 2026 ka commission kitna hoga",
        "september 2027 ka commission kitna hoga",
    ] {
        let (_, requests) = h.ask(question, None, Vec::new());
        assert!(
            !requests.is_empty(),
            "{question}: the app refused a month it can estimate"
        );
    }
}

// ---- H3 ---------------------------------------------------------------------------------------

/// A model reply that only says "x": the question is asked of a model, and what the app read ahead is in
/// its first request.
fn model_says_x() -> Value {
    json!({
        "model": "openai/gpt-6-luna",
        "choices": [{"finish_reason": "stop", "message": {"role": "assistant", "content": json!({
            "kind": "answer", "text": "x", "show": [], "next": [],
            "choose": null, "entry": null, "nearest": null
        }).to_string()}}],
        "usage": {"prompt_tokens": 7200, "completion_tokens": 60,
            "prompt_tokens_details": {"cached_tokens": 7000},
            "completion_tokens_details": {"reasoning_tokens": 0}},
    })
}

/// The tool results that were in the first request, as JSON.
fn pre_read(request: &Value) -> Vec<Value> {
    request["messages"]
        .as_array()
        .map(|messages| {
            messages
                .iter()
                .filter(|m| m["role"] == "tool")
                .filter_map(|m| serde_json::from_str::<Value>(m["content"].as_str()?).ok())
                .collect()
        })
        .unwrap_or_default()
}

/// The figure of a result row or totals object: `{"rs": 12000}` or `{"n": 10}`.
fn rs(v: &Value) -> i64 {
    v.get("rs")
        .or_else(|| v.get("n"))
        .and_then(Value::as_i64)
        .unwrap_or_else(|| panic!("not a figure: {v}"))
}

/// Saves a بکری of کریم's سیب (15 نگ) with these lines, and returns it.
fn karim_apples(
    w: &world::World,
    lines: Vec<mandi_services::core::bikri::SaleInput>,
    received: &str,
) -> mandi_services::core::bikri::BikriSaved {
    let mut lot = world::lot(w, "lot-x", w.karim, w.apple, received);
    lot.cash_paid = None;
    lot.freight = None;
    lot.sales = lines;
    mandi_services::bikri::save(
        &w.env.ctx,
        &w.owner,
        &mandi_services::core::bikri::BikriSaveInput {
            draft: mandi_services::core::bikri::BikriDraft {
                header: world::header(),
                lots: vec![lot],
            },
            confirm_cash_below_zero: true,
        },
    )
    .unwrap()
}

fn walk_in(
    key: &str,
    name: (&str, &str),
    qty: &str,
    seller_rate: &str,
    w: &world::World,
) -> mandi_services::core::bikri::SaleInput {
    let mut line = world::sale(key, w.chukta_khata, qty, seller_rate);
    line.chukta = Some(mandi_services::core::bikri::ChuktaInput {
        name_ur: name.0.into(),
        name_en: name.1.into(),
        phone: String::new(),
    });
    line
}

#[test]
fn h3_the_walk_in_buyers_together_are_read_from_views_a_and_c_before_the_model_writes() {
    // The worked day has Shahid, a walk-in buyer: 10 پیٹی سیب at 1,200, 12,000, commission 720 + 720 and
    // لاگا 80 (E2E-5). On the same day: a second walk-in buyer, Nasir, 5 at 1,000 seller rate and 1,100
    // buyer rate (ریٹ مختلف, FR-12.3: his goods are worth 5,500 to him and 5,000 to the seller; seller
    // commission 6% of 5,000 = 300, buyer commission 6% of 5,500 = 330, لاگا 5 x 8 = 40); a regular buyer,
    // Haroon, and نقدی منڈی, 5 each at 1,000; and a walk-in sale that is cancelled.
    let (w, haroon) = haroon_world(Some("12,000"));
    let mut nasir = walk_in("s-1", ("ناصر", "Nasir"), "5", "1,000", &w);
    nasir.rate_differs = true;
    nasir.buyer_rate = Some("1,100".into());
    karim_apples(
        &w,
        vec![
            nasir,
            world::sale("s-2", haroon, "5", "1,000"),
            world::sale("s-3", w.naqdi, "5", "1,000"),
        ],
        "15",
    );
    let cancelled = karim_apples(
        &w,
        vec![walk_in("s-1", ("عمر", "Umar"), "2", "900", &w)],
        "2",
    );
    let version = mandi_services::bikri::get(&w.env.ctx, cancelled.bikri_id)
        .unwrap()
        .version;
    mandi_services::bikri::cancel(
        &w.env.ctx,
        &w.owner,
        &mandi_services::core::bikri::CancelInput {
            id: cancelled.bikri_id,
            version,
            reason: "غلط اندراج".into(),
            confirm_cash_below_zero: true,
        },
    )
    .unwrap();

    // The screens' own figures for the same day, with a buyer filter.
    let on_screen = |view: mandi_reports::analytics::AnalyticsView, buyer: Option<i64>| {
        let page = mandi_reports::analytics::run(
            w.env.ctx.db(),
            &mandi_reports::analytics::AnalyticsRunInput {
                sort: None,
                view,
                filters: mandi_reports::analytics::AnalyticsFilters {
                    from: "2026-10-10".into(),
                    to: "2026-10-10".into(),
                    buyer: buyer.map(|account_id| mandi_reports::reports::PartyFilter {
                        account_id,
                        person_key: None,
                    }),
                    ..Default::default()
                },
                offset: 0,
                limit: 0,
            },
        )
        .unwrap();
        move |column: &str| -> i64 {
            let at = page
                .columns
                .iter()
                .position(|c| c.key == column)
                .unwrap_or_else(|| panic!("no column {column}"));
            match &page.totals[at] {
                mandi_reports::reports::Cell::Money { paisa } => paisa.rupees(),
                mandi_reports::reports::Cell::Number { value } => *value,
                other => panic!("{column}: {other:?}"),
            }
        }
    };
    use mandi_reports::analytics::AnalyticsView::{A, C};
    let chukta_a = on_screen(A, Some(w.chukta_khata));
    let chukta_c = on_screen(C, Some(w.chukta_khata));
    // The PRD numbers and the hand-worked ones: Shahid 12,000 + Nasir 5,000 (seller side) / 5,500 (buyer side).
    assert_eq!(chukta_a("qty"), 15);
    assert_eq!(chukta_a("gross"), 17_000);
    assert_eq!(chukta_a("seller_comm"), 720 + 300);
    assert_eq!(chukta_a("buyer_comm"), 720 + 330);
    assert_eq!(chukta_a("laga"), 80 + 40);
    assert_eq!(chukta_a("total_comm"), 1_520 + 670);
    // What they BOUGHT is the buyer's value: view C. It is not view A's gross (the rate difference).
    assert_eq!(chukta_c("purchase_value"), 12_000 + 5_500);
    assert_ne!(chukta_c("purchase_value"), chukta_a("gross"));
    // The walk-in buyers and the others are disjoint and add up to the shop (the tile's commission): the
    // cancelled walk-in sale is in neither.
    let others: i64 = [w.akram, w.arshad, haroon, w.naqdi]
        .iter()
        .map(|b| on_screen(A, Some(*b))("total_comm"))
        .sum();
    assert_eq!(
        chukta_a("total_comm") + others,
        on_screen(A, None)("total_comm"),
        "walk-in + the others = every buyer"
    );

    let h = production(w);
    for question in [
        "aaj chukta gahakon ne kitne ka maal liya aur commission kitna bana",
        "aaj chukta gahakon ki bikri kitni hui",
    ] {
        let (_, requests) = h.ask(question, None, vec![model_says_x(), model_says_x()]);
        assert!(
            !requests.is_empty(),
            "{question}: the model writes the answer"
        );
        let read = pre_read(&requests[0]);
        let tools: Vec<&str> = read.iter().filter_map(|r| r["tool"].as_str()).collect();
        assert_eq!(
            tools,
            ["period", "analytics", "analytics"],
            "{question}: {read:?}"
        );
        let (a, c) = (&read[1], &read[2]);
        assert_eq!(
            (a["source"].as_str(), c["source"].as_str()),
            (Some("A"), Some("C"))
        );
        for view in [a, c] {
            assert_eq!(view["filters"]["from"], "2026-10-10", "{question}");
            assert_eq!(view["filters"]["to"], "2026-10-10", "{question}");
        }
        // Each figure the model is shown is the screen's.
        for (key, name) in [
            ("nag", "qty"),
            ("gross", "gross"),
            ("seller_comm", "seller_comm"),
            ("buyer_comm", "buyer_comm"),
            ("laga", "laga"),
            ("total_comm", "total_comm"),
        ] {
            assert_eq!(rs(&a["totals"][key]), chukta_a(name), "{question}: A {key}");
        }
        assert_eq!(
            rs(&c["totals"]["purchase_value"]),
            chukta_c("purchase_value"),
            "{question}"
        );
        // View C's totals row is the sum of its rows (one for each walk-in buyer).
        let rows = c["rows"].as_array().expect("rows");
        let sum: i64 = rows.iter().map(|r| rs(&r["purchase_value"])).sum();
        assert_eq!(sum, 17_500, "{question}: the rows add up to the totals row");
        assert!(
            rows.len() >= 2,
            "{question}: Shahid and Nasir have a row each"
        );
    }
    h.scan_requests();
}

#[test]
fn h3_an_ordinary_buyer_is_not_the_walk_in_buyers_and_a_named_one_is_not_scoped() {
    // "chukta" with a name of the books is that party's question, not the walk-in group's: no read of the
    // group is put in front of the model.
    let (w, _) = haroon_world(Some("12,000"));
    let h = production(w);
    let (_, requests) = h.ask(
        "aaj haroon ne kitne ka maal liya aur commission kitna bana",
        None,
        vec![model_says_x(), model_says_x()],
    );
    assert!(!requests.is_empty());
    for result in pre_read(&requests[0]) {
        assert_ne!(
            result["filters"]["buyer"]["name"]["en"], "Chukta Khata",
            "the walk-in group was read for a named buyer"
        );
    }
}
