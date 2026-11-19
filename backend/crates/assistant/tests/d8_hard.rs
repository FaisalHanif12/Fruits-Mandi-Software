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
//! - H1, H2, H5 and H6 are about the money paid on the spot (plan 65): on the D8 world (below) the app reads
//!   the register R5 ahead, for the model, and gives no instant line (a buyer's plain balance would be a real
//!   number for the wrong thing).
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

// ---- H1, H2, H5, H6: the D8 world (support/worked.rs `d8`) --------------------------------------

use worked::d8;

/// One column of the totals row of a report, as the SCREEN shows it, in rupees.
fn screen_total(
    w: &world::World,
    id: mandi_reports::reports::ReportId,
    filters: mandi_reports::reports::ReportFilters,
    column: &str,
) -> i64 {
    let page = mandi_reports::reports::run(
        w.env.ctx.db(),
        &mandi_reports::reports::ReportRunInput {
            group_by_month: false,
            sort: None,
            id,
            filters,
            offset: 0,
            limit: 0,
        },
    )
    .unwrap();
    let at = page
        .columns
        .iter()
        .position(|c| c.key == column)
        .unwrap_or_else(|| panic!("the report has no column {column}"));
    match &page.totals[at] {
        mandi_reports::reports::Cell::Money { paisa } => paisa.rupees(),
        other => panic!("{column}: {other:?}"),
    }
}

/// The totals of R5 that the SCREEN shows for these filters: value, buyer total, paid, left.
fn r5_on_screen(w: &world::World, filters: mandi_reports::reports::ReportFilters) -> [i64; 4] {
    ["value", "buyer_total", "paid", "left"].map(|key| {
        screen_total(
            w,
            mandi_reports::reports::ReportId::R5,
            filters.clone(),
            key,
        )
    })
}

/// The totals of R5 in the result the model is shown: value, buyer total, paid, left.
fn r5_in_pre_read(result: &Value) -> [i64; 4] {
    ["value", "buyer_total", "paid", "left"].map(|key| rs(&result["totals"][key]))
}

fn tools_of(read: &[Value]) -> Vec<&str> {
    read.iter().filter_map(|r| r["tool"].as_str()).collect()
}

fn r5_filters(
    from: &str,
    to: &str,
    party: Option<i64>,
    seller: Option<i64>,
    item: Option<i64>,
) -> mandi_reports::reports::ReportFilters {
    mandi_reports::reports::ReportFilters {
        from: from.into(),
        to: to.into(),
        party: party.map(|account_id| mandi_reports::reports::PartyFilter {
            account_id,
            person_key: None,
        }),
        seller_id: seller,
        item_id: item,
        ..Default::default()
    }
}

/// Asks a question of a model that only says "x", and gives the results the app read ahead for it.
fn read_ahead(h: &Harness, question: &str) -> Vec<Value> {
    let (reply, requests) = h.ask(question, None, vec![model_says_x(), model_says_x()]);
    assert!(
        !requests.is_empty(),
        "{question}: the app answered by itself ({:?})",
        reply.outcome
    );
    h.scan_requests();
    pre_read(&requests[0])
}

#[test]
fn h1_one_buyer_over_a_range_his_items_and_what_he_paid_in_cash() {
    // Today 08-11-2026. "pichle saal December se aaj tak" is ONE range, 01-12-2025 to 08-11-2026: Haroon's
    // line of 20-11-2025 is before it. His three lines in it: apple 20 at 800 (17,120, paid 6,000), banana 30
    // at 600 (19,320, paid in full) and mango 130 at 900 (125,060, paid 0).
    let d = d8();
    let haroon = d.haroon;
    let screen = r5_on_screen(
        &d.w,
        r5_filters("01-12-2025", "08-11-2026", Some(haroon), None, None),
    );
    assert_eq!(screen, [151_000, 161_500, 25_320, 136_180]);
    let h = production(d.w);
    let read = read_ahead(
        &h,
        "pichle saal December se aaj tak haroon ne kya kya khareeda aur naqdi kitni jama karwai",
    );
    assert_eq!(
        tools_of(&read),
        ["period", "find", "analytics", "report"],
        "{read:?}"
    );
    assert_eq!(read[0]["from"]["date"], "2025-12-01");
    assert_eq!(read[0]["to"]["date"], "2026-11-08");
    let (items, bills) = (&read[2], &read[3]);
    assert_eq!(bills["source"], "R5");
    // What he paid on the spot and what he left: the register's totals, the screen's.
    assert_eq!(r5_in_pre_read(bills), screen);
    // His items, with qty and rate, one row for him; the 20-11-2025 banana is not in them.
    let lines = bills["rows"][0]["detail"].as_array().expect("his lines");
    let mut got: Vec<(String, i64, i64)> = lines
        .iter()
        .map(|l| {
            (
                l["item"]["en"].as_str().unwrap().to_owned(),
                rs(&l["qty"]),
                rs(&l["rate"]),
            )
        })
        .collect();
    got.sort();
    assert_eq!(
        got,
        [
            ("Apple".to_owned(), 20, 800),
            ("Banana".to_owned(), 30, 600),
            ("Mango".to_owned(), 130, 900)
        ]
    );
    // View A for him holds the same items with their nag: 20 + 30 + 130.
    assert_eq!(items["source"], "A");
    assert_eq!(rs(&items["totals"]["nag"]), 180);
}

#[test]
fn h2_paid_and_left_this_month_for_one_buyer() {
    // Anas in November: mango 50 at 900 (48,100, paid 10,000, left 38,100) and banana 10 at 500 (5,380, paid
    // in full). His 12-09-2026 line is not this month. No balance, no party card: the balance is another figure.
    let d = d8();
    let anas = d.anas;
    let screen = r5_on_screen(
        &d.w,
        r5_filters("01-11-2026", "08-11-2026", Some(anas), None, None),
    );
    assert_eq!(screen, [50_000, 53_480, 15_380, 38_100]);
    let h = production(d.w);
    for question in [
        "is mahine anas ne mauqe par kitna jama karaya aur kitna baqi chhora",
        "how much did anas pay on the spot this month and how much did he leave",
    ] {
        let read = read_ahead(&h, question);
        assert_eq!(
            tools_of(&read),
            ["period", "find", "report"],
            "{question}: {read:?}"
        );
        assert_eq!(read[2]["source"], "R5", "{question}");
        assert_eq!(r5_in_pre_read(&read[2]), screen, "{question}");
    }
}

#[test]
fn h5_one_lot_every_buyer_and_the_cash_of_each() {
    // Ehsan's 200 mango (E2E-64, E2E-65): Mani 20 at 1,000 (21,360, paid in full), Anas 50 at 900 (48,100, paid
    // 10,000), Haroon 130 at 900 (125,060, paid 0). One row for each buyer; the totals are 31,360 paid and
    // 163,160 left.
    let d = d8();
    let (ehsan, mango) = (d.ehsan, d.mango);
    let screen = r5_on_screen(&d.w, r5_filters("", "", None, Some(ehsan), Some(mango)));
    assert_eq!(screen, [182_000, 194_520, 31_360, 163_160]);
    let h = production(d.w);
    let read = read_ahead(
        &h,
        "ehsan ke 200 peti aam kis kis ne liye aur kis ne kitna naqd diya",
    );
    assert_eq!(tools_of(&read), ["find", "find", "report"], "{read:?}");
    let bills = &read[2];
    assert_eq!(bills["source"], "R5");
    assert_eq!(r5_in_pre_read(bills), screen);
    let mut per_buyer: Vec<(String, i64, i64, i64)> = bills["rows"]
        .as_array()
        .expect("rows")
        .iter()
        .map(|r| {
            (
                r["buyer"]["en"].as_str().unwrap().to_owned(),
                rs(&r["detail"][0]["qty"]),
                rs(&r["paid"]),
                rs(&r["left"]),
            )
        })
        .collect();
    per_buyer.sort();
    assert_eq!(
        per_buyer,
        [
            ("Anas".to_owned(), 50, 10_000, 38_100),
            ("Haroon".to_owned(), 130, 0, 125_060),
            ("Mani".to_owned(), 20, 21_360, 0)
        ]
    );
}

#[test]
fn h6_a_range_and_the_cash_and_credit_sales_in_it() {
    // 02-11-2026 to 05-11-2026: the lot of Ehsan (paid 31,360, left 163,160), Anas's banana (5,380 paid) and
    // Akram's apple (10,680, paid 3,000, left 7,680). The lines of 01-11 and 06-11 are outside. Cash is what
    // was paid on the spot, 39,740; credit is what was left on account, 170,840. The model works out the
    // difference with `calculate`; the app reads the two totals.
    let d = d8();
    let screen = r5_on_screen(
        &d.w,
        r5_filters("02-11-2026", "05-11-2026", None, None, None),
    );
    assert_eq!(screen, [197_000, 210_580, 39_740, 170_840]);
    // The glossary's reading: cash is what R3 holds as paid, credit what R4 holds as left. R5's totals are the
    // same two figures over the same filter (a part-paid line is in both registers, so the registers' own
    // buyer totals must NOT be added: Akram's 10,680 and Anas's 48,100 would count twice).
    let filters = r5_filters("02-11-2026", "05-11-2026", None, None, None);
    assert_eq!(
        screen_total(
            &d.w,
            mandi_reports::reports::ReportId::R3,
            filters.clone(),
            "paid"
        ),
        screen[2]
    );
    assert_eq!(
        screen_total(&d.w, mandi_reports::reports::ReportId::R4, filters, "left"),
        screen[3]
    );
    let h = production(d.w);
    for question in [
        "2 november se 5 november tak naqd aur udhar bikri mein kitna farq hai",
        "what is the difference between cash and credit sales from 2 november to 5 november",
    ] {
        let read = read_ahead(&h, question);
        assert_eq!(
            tools_of(&read),
            ["period", "report"],
            "{question}: {read:?}"
        );
        assert_eq!(read[0]["from"]["date"], "2026-11-02", "{question}");
        assert_eq!(read[0]["to"]["date"], "2026-11-05", "{question}");
        assert_eq!(read[1]["source"], "R5", "{question}");
        assert_eq!(r5_in_pre_read(&read[1]), screen, "{question}");
    }
}

#[test]
fn a_receipt_wording_is_not_a_question_about_the_sale_line() {
    // "akram ne kitna jama karaya" is about his receipts (وصولی), not the money paid on the spot of his
    // sale lines: no read of R5 is made for it.
    let d = d8();
    let h = production(d.w);
    let (_, requests) = h.ask(
        "is mahine akram ne kitna jama karaya",
        None,
        vec![model_says_x(), model_says_x()],
    );
    if let Some(first) = requests.first() {
        for result in pre_read(first) {
            assert_ne!(result["source"], "R5", "a receipt question read the bills");
        }
    }
}

#[test]
fn h6_the_month_a_range_is_named_with_stays_in_the_answer() {
    // Live re-run of plan 70: the model wrote "۲ نومبر سے ۵ نومبر تک". The range has no `label`, so the month
    // could not be put back, a rewrite call dropped it and the answer read "2 سے 5 تک". The range holds the month
    // of each end, so the app puts the name back as a reference, with no rewrite call, and the answer keeps it.
    let d = d8();
    let h = production(d.w);
    let text = "{{q.n1}} نومبر سے {{q.n2}} نومبر تک ادھاری {{r2.totals.left}} اور نقدی {{r2.totals.paid}} رہی۔";
    let reply_json = json!({
        "model": "openai/gpt-6-luna",
        "choices": [{"finish_reason": "stop", "message": {"role": "assistant", "content": json!({
            "kind": "answer", "text": text, "show": [], "next": [],
            "choose": null, "entry": null, "nearest": null
        }).to_string()}}],
        "usage": {"prompt_tokens": 7200, "completion_tokens": 60,
            "prompt_tokens_details": {"cached_tokens": 7000},
            "completion_tokens_details": {"reasoning_tokens": 0}},
    });
    let (reply, requests) = h.ask(
        "2 november se 5 november tak naqd aur udhar bikri mein kitna farq hai",
        None,
        vec![reply_json],
    );
    h.scan_requests();
    assert_eq!(requests.len(), 1, "one model call, no rewrite");
    assert_eq!(reply.outcome, AskOutcome::Answered);
    let answer = answer_of(&reply);
    // The month is a part of the answer of its own (filled from the reference), twice: 2 November to 5 November.
    let months = answer
        .parts
        .iter()
        .filter(|p| format!("{p:?}").contains("Month"))
        .count();
    assert_eq!(months, 2, "the month was lost: {:?}", answer.parts);
    let shown: Vec<i64> = figures(answer)
        .iter()
        .filter_map(|(cell, _)| match cell {
            mandi_reports::reports::Cell::Money { paisa } => Some(paisa.rupees()),
            _ => None,
        })
        .collect();
    assert_eq!(shown, [170_840, 39_740], "left and paid, from R5");
}
