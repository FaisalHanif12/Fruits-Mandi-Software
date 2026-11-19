//! The forecast gaps F1 to F5 (plan 55 section 11.1, PRD FR-7.86): the estimate tool for the
//! measures that had no monthly series (buyer commission, لاگا, receipts, garden and trip sales),
//! for one expense head, for one partner's profit, and for a balance at the same pace (a party,
//! the cash in hand, the buyers' dues).
//!
//! The books are "fixture B" of the design note: Rahim's banana sold to Akram on the 15th of each
//! month April to September 2026 (100 نگ, rate 100, 200 ... 600), on credit; each month Akram pays
//! a receipt on the 20th, and the shop pays an electricity bill and a tea bill on the 25th. Today
//! is 10-10-26, so April to September are the six full months. Partners 40 / 30 / 30.
//!
//! | Month | Value | Buyer comm | لاگا | Akram's bill | Receipt | Electricity | Tea |
//! |---|---|---|---|---|---|---|---|
//! | Apr | 10,000 | 600 | 800 | 11,400 | 10,000 | 4,000 | 1,000 |
//! | May | 20,000 | 1,200 | 800 | 22,000 | 12,000 | 4,000 | 1,000 |
//! | Jun | 30,000 | 1,800 | 800 | 32,600 | 14,000 | 5,000 | 1,000 |
//! | Jul | 40,000 | 2,400 | 800 | 43,200 | 16,000 | 5,000 | 1,000 |
//! | Aug | 50,000 | 3,000 | 800 | 53,800 | 18,000 | 6,000 | 1,000 |
//! | Sep | 60,000 | 3,600 | 800 | 64,400 | 20,000 | 6,000 | 1,000 |
#![cfg(test)]

#[path = "../../services/tests/common/mod.rs"]
mod common;
#[path = "support/worked.rs"]
mod worked;
#[allow(dead_code)]
#[path = "../../services/tests/bikri_preview/support.rs"]
mod world;

use mandi_assistant::results::{Node, ToolResult, Words, fit, tool_result_json};
use mandi_assistant::tools::{Env, State, Stop, ToolError, call};
use mandi_reports::dashboard::BackupFacts;
use mandi_services::core::accounts::{OpeningLineInput, OpeningsInput};
use mandi_services::core::auth::Language;
use mandi_services::core::bikri::{BikriSaveInput, LotInput};
use mandi_services::core::capital::CapitalInput;
use mandi_services::core::dates::BusinessDate;
use mandi_services::core::dead::{DeadMarkInput, DeadReason};
use mandi_services::core::expenses::ShopExpenseInput;
use mandi_services::core::gardens::{GardenEntryInput, GardenInput};
use mandi_services::core::purchases::PurchaseInput;
use mandi_services::core::trips::{OtherSaleInput, TripInput};
use mandi_services::core::ventures::{MoneySource, VenturePartnerInput};
use mandi_services::core::vouchers::{ReceiptInput, RemovedPaymentFields, VoucherCancelInput};
use mandi_services::domain::figures::{QuestionNumber, question_numbers};
use mandi_services::{accounts, bikri, dead, gardens, purchases, trips, vouchers};
use serde_json::{Value, json};

/// The sales of fixture B: (month, quantity, rate). Banana, one sale to Akram on the 15th.
const SALES: [(u32, bool, i64, i64); 6] = [
    (4, true, 100, 100),
    (5, true, 100, 200),
    (6, true, 100, 300),
    (7, true, 100, 400),
    (8, true, 100, 500),
    (9, true, 100, 600),
];
/// What Akram pays each month, on the 20th.
const RECEIPTS: [i64; 6] = [10_000, 12_000, 14_000, 16_000, 18_000, 20_000];
/// The electricity bill of each month, paid on the 25th.
const ELECTRICITY: [i64; 6] = [4_000, 4_000, 5_000, 5_000, 6_000, 6_000];
/// The tea bill of each month, paid on the 25th.
const TEA: i64 = 1_000;

/// The world of fixture B and the ids of its two expense heads.
struct Books {
    world: world::World,
    electricity: i64,
    tea: i64,
}

fn books() -> Books {
    let w = worked::months_world(&SALES);
    let electricity = worked::expense_head(&w, "بجلی", "Electricity");
    let tea = worked::expense_head(&w, "چائے", "Tea");
    for (i, month) in (4..=9_u32).enumerate() {
        vouchers::save_receipt(
            &w.env.ctx,
            &w.owner,
            &ReceiptInput {
                date: format!("20-{month:02}-2026"),
                account_id: w.akram,
                amount: RECEIPTS[i].to_string(),
                removed: RemovedPaymentFields::default(),
                collector_staff_id: None,
                narration: String::new(),
            },
        )
        .unwrap();
        for (head, amount) in [(electricity, ELECTRICITY[i]), (tea, TEA)] {
            vouchers::save_shop_expense(
                &w.env.ctx,
                &w.owner,
                &ShopExpenseInput {
                    date: format!("25-{month:02}-2026"),
                    expense_head_id: Some(head),
                    amount: amount.to_string(),
                    paid_to: None,
                    narration: None,
                    confirm_cash_below_zero: false,
                },
            )
            .unwrap();
        }
    }
    Books {
        world: w,
        electricity,
        tea,
    }
}

/// Akram's opening debit of `dr` rupees, dated 15-05-2026 (inside the window of the pace). Saving it
/// again with another amount reverses the old opening on 15-05 and posts the new one.
fn open_akram(w: &world::World, dr: &str) {
    accounts::openings_save(
        &w.env.ctx,
        &w.owner,
        &OpeningsInput {
            lines: vec![OpeningLineInput {
                account_id: w.akram,
                version: accounts::get(&w.env.ctx, w.akram).unwrap().version,
                opening_date: "15-05-2026".into(),
                opening_dr: dr.into(),
                opening_cr: String::new(),
                opening_is_advance: false,
            }],
        },
    )
    .unwrap();
}

/// Arshad's opening on 31-03-2026 (before the window of the pace): `dr` or `cr` in rupees.
fn open_arshad(w: &world::World, dr: &str, cr: &str) {
    accounts::openings_save(
        &w.env.ctx,
        &w.owner,
        &OpeningsInput {
            lines: vec![OpeningLineInput {
                account_id: w.arshad,
                version: accounts::get(&w.env.ctx, w.arshad).unwrap().version,
                opening_date: "31-03-2026".into(),
                opening_dr: dr.into(),
                opening_cr: cr.into(),
                opening_is_advance: false,
            }],
        },
    )
    .unwrap();
}

/// A credit bill of Arshad's on `date`: Rahim's 10 banana at 700, so he owes 7,000 + 420 + 80 =
/// 7,500.
fn arshad_bill(w: &world::World, date: &str) {
    let lot = world::with_sales(
        world::bare_lot(w, "lot-1", w.rahim, w.banana, "10"),
        vec![world::sale("s-1", w.arshad, "10", "700")],
    );
    let mut draft = world::draft(vec![lot]);
    draft.header.entry_date = date.into();
    bikri::save(
        &w.env.ctx,
        &w.owner,
        &BikriSaveInput {
            draft,
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
}

/// A receipt from Arshad.
fn arshad_pays(w: &world::World, date: &str, amount: &str) {
    vouchers::save_receipt(
        &w.env.ctx,
        &w.owner,
        &ReceiptInput {
            date: date.into(),
            account_id: w.arshad,
            amount: amount.into(),
            removed: RemovedPaymentFields::default(),
            collector_staff_id: None,
            narration: String::new(),
        },
    )
    .unwrap();
}

/// The total of R20 as of a day, in rupees.
fn r20_as_of(w: &world::World, day: &str) -> i64 {
    let page = mandi_reports::reports::run(
        w.env.ctx.db(),
        &serde_json::from_value(json!({"id": "R20", "filters": {"as_of": day}})).unwrap(),
    )
    .unwrap();
    let at = page
        .columns
        .iter()
        .position(|c| c.key == "balance")
        .unwrap();
    match page.totals.get(at).unwrap() {
        mandi_reports::reports::Cell::Money { paisa } => paisa.rupees(),
        mandi_reports::reports::Cell::Balance { paisa, .. } => paisa.rupees(),
        other => panic!("{other:?}"),
    }
}

/// A session of tool calls on one world, as in `tools.rs`: results get their ids and stay for the
/// next call.
struct Session<'a> {
    world: &'a world::World,
    words: Words,
    numbers: Vec<QuestionNumber>,
    question: String,
    state: State,
    locked: bool,
    today: &'static str,
}

impl<'a> Session<'a> {
    fn new(world: &'a world::World, question: &str) -> Self {
        Self {
            world,
            words: Words::new().unwrap(),
            numbers: question_numbers(question),
            question: question.to_owned(),
            state: State::default(),
            locked: false,
            today: "2026-10-10",
        }
    }

    fn locked(mut self) -> Self {
        self.locked = true;
        self
    }

    /// Shows the model a record key, as a `find` result would (so the key is "known").
    fn show_key(&mut self, key: &str, code: &str, name_en: &str) {
        use mandi_assistant::results::{NameVal, Val};
        let node = Node::obj(vec![(
            "matches",
            Node::List(vec![Node::obj(vec![
                ("key", Node::Str(key.to_owned())),
                ("code", Node::Str(code.to_owned())),
                (
                    "name",
                    Node::Val(Val::Name(NameVal {
                        name_ur: name_en.to_owned(),
                        name_en: name_en.to_owned(),
                        tag: None,
                        code: Some(code.to_owned()),
                    })),
                ),
            ])]),
        )]);
        let id = self.state.next_id('r');
        self.state.results.push(ToolResult {
            id,
            tool: "find",
            input: json!({}),
            node,
            shown: None,
            source: None,
            partner_parts_locked: false,
            partner_data: false,
            turn: 0,
            working: None,
        });
    }

    /// The estimate tool with the nullable fields all null, then `extra` laid over them.
    fn estimate(&mut self, extra: Value) -> Result<Value, ToolError> {
        let mut input = json!({
            "measure": "commission_all", "item": null, "seller": null, "buyer": null,
            "garden": null, "trip": null, "head": null, "partner": null, "party": null,
            "ahead": null, "until": null, "what_if": null, "percent": null, "target": null
        });
        for (k, v) in extra.as_object().unwrap() {
            input[k] = v.clone();
        }
        self.run("estimate", &input)
    }

    /// Any tool, and its result kept in the state like the estimate's (so `e1.*` or `c1.*` can be
    /// read by the next call).
    fn run(&mut self, tool: &str, input: &Value) -> Result<Value, ToolError> {
        let env = Env {
            ctx: &self.world.env.ctx,
            actor: &self.world.owner,
            words: &self.words,
            language: Language::Ur,
            today: BusinessDate::parse_iso(self.today).unwrap(),
            partner_locked: self.locked,
            numbers: &self.numbers,
            question: &self.question,
            backup: BackupFacts {
                cloud_on: false,
                cloud_age_hours: None,
            },
        };
        let draft = match call(&env, &self.state, tool, &input.to_string()) {
            Ok(draft) => draft,
            Err(Stop::Error(e)) => return Err(e),
            Err(Stop::Locked) => {
                return Err(ToolError::new("LOCKED", "partner", "the section is locked"));
            }
        };
        let id = self
            .state
            .next_id(if draft.tool == "calculate" { 'c' } else { 'e' });
        let mut result = ToolResult {
            id: id.clone(),
            tool: draft.tool,
            input: draft.input,
            node: draft.node,
            shown: draft.shown,
            source: draft.source,
            partner_parts_locked: draft.partner_parts_locked,
            partner_data: draft.partner_data,
            turn: 0,
            working: draft.working,
        };
        fit(&mut result, &self.words);
        self.state.results.push(result);
        Ok(tool_result_json(self.state.find(&id).unwrap(), &self.words))
    }
}

/// A money value of the tool result in whole rupees (positive, or signed when it has no side).
fn rs(v: &Value) -> i64 {
    v["rs"].as_i64().unwrap_or_else(|| panic!("not money: {v}"))
}

/// A change or a difference of the result: its size and its direction word.
fn diff(v: &Value) -> (i64, String) {
    (
        rs(v),
        v["dir"]
            .as_str()
            .unwrap_or_else(|| panic!("no direction: {v}"))
            .to_owned(),
    )
}

/// The months of the `history` of a result, as whole-rupee values.
fn history(v: &Value) -> Vec<i64> {
    v["history"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| rs(&m["value"]))
        .collect()
}

/// The months of the `history` of a result of a balance, signed: a change comes as a size and a
/// direction word, `down` meaning below 0.
fn changes_of(v: &Value) -> Vec<i64> {
    v["history"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| {
            let size = rs(&m["value"]);
            if m["value"]["dir"] == "down" {
                -size
            } else {
                size
            }
        })
        .collect()
}

/// The estimate for 6 months ahead with no number in the question (the default of the rule).
fn six(question: &str) -> String {
    format!("{question} agle mahino mein")
}

// ---- F1: the series that had none ---------------------------------------------------------------

#[test]
fn buyer_commission_has_a_monthly_series_and_adds_up_to_view_a() {
    let b = books();
    let mut s = Session::new(&b.world, &six("buyer commission"));
    let v = s.estimate(json!({"measure": "commission_buyer"})).unwrap();
    assert_eq!(history(&v), vec![600, 1_200, 1_800, 2_400, 3_000, 3_600]);
    assert_eq!(v["history"][0]["key"], "m202604");
    // Same pace: (2,400 + 3,000 + 3,600) / 3 = 3,000 a month, 18,000 in 6 months.
    assert_eq!(rs(&v["same_pace"]["per_month"]), 3_000);
    assert_eq!(rs(&v["same_pace"]["total"]), 18_000);
    // Same growth: (3,600 - 600) / 5 = 600 a month; 4,200 ... 7,200 = 34,200.
    assert_eq!(diff(&v["same_growth"]["step"]), (600, "up".into()));
    assert_eq!(rs(&v["same_growth"]["total"]), 34_200);
    // The months add up to the buyer commission column of view A for the same dates.
    let page = mandi_reports::analytics::run(
        b.world.env.ctx.db(),
        &serde_json::from_value(json!({"view": "A", "filters": {}, "offset": 0, "limit": 100}))
            .unwrap(),
    )
    .unwrap();
    let at = page
        .columns
        .iter()
        .position(|c| c.key == "buyer_comm")
        .unwrap();
    let total = match page.totals.get(at).unwrap() {
        mandi_reports::reports::Cell::Money { paisa } => paisa.rupees(),
        other => panic!("{other:?}"),
    };
    assert_eq!(total, history(&v).iter().sum::<i64>());
    // It opens view A, which shows the buyer commission, and not view G.
    let result = s.state.results.last().unwrap();
    assert!(matches!(
        result.source,
        Some(mandi_assistant::answer::AnswerSource::Analytics {
            view: mandi_reports::analytics::AnalyticsView::A,
            ..
        })
    ));
}

#[test]
fn laga_has_a_monthly_series() {
    let b = books();
    let mut s = Session::new(&b.world, &six("laga"));
    let v = s.estimate(json!({"measure": "laga"})).unwrap();
    assert_eq!(history(&v), vec![800; 6]);
    assert_eq!(rs(&v["same_pace"]["per_month"]), 800);
    assert_eq!(rs(&v["same_pace"]["total"]), 4_800);
    assert_eq!(rs(&v["same_growth"]["total"]), 4_800);
}

#[test]
fn receipts_have_a_monthly_series_and_a_buyer_narrows_them() {
    let b = books();
    let mut s = Session::new(&b.world, &six("wasooli"));
    let v = s.estimate(json!({"measure": "receipts"})).unwrap();
    assert_eq!(
        history(&v),
        vec![10_000, 12_000, 14_000, 16_000, 18_000, 20_000]
    );
    assert_eq!(rs(&v["same_pace"]["per_month"]), 18_000);
    assert_eq!(rs(&v["same_pace"]["total"]), 108_000);
    assert_eq!(rs(&v["same_growth"]["total"]), 162_000);
    // The months add up to the total row of R23.
    let page = mandi_reports::reports::run(
        b.world.env.ctx.db(),
        &serde_json::from_value(json!({"id": "R23", "filters": {}})).unwrap(),
    )
    .unwrap();
    let at = page.columns.iter().position(|c| c.key == "amount").unwrap();
    let total = match page.totals.get(at).unwrap() {
        mandi_reports::reports::Cell::Money { paisa } => paisa.rupees(),
        other => panic!("{other:?}"),
    };
    assert_eq!(total, 90_000);
    assert_eq!(total, history(&v).iter().sum::<i64>());
    // Akram is the only buyer who paid, so his receipts are the same series.
    let key = format!("a{}", b.world.akram);
    s.show_key(&key, "03040001", "Akram");
    let akram = s
        .estimate(json!({"measure": "receipts", "buyer": key}))
        .unwrap();
    assert_eq!(history(&akram), history(&v));
    // Arshad paid nothing: a series of nothing, so the estimate is 0 and not an error.
    let arshad = format!("a{}", b.world.arshad);
    s.show_key(&arshad, "03040002", "Arshad");
    let none = s
        .estimate(json!({"measure": "receipts", "buyer": arshad}))
        .unwrap();
    assert_eq!(none["history"].as_array().unwrap().len(), 0);
    // A filter that does not belong to receipts is refused, never ignored.
    let item = format!("i{}", b.world.banana);
    s.show_key(&item, "0001", "Banana");
    let refused = s
        .estimate(json!({"measure": "receipts", "item": item}))
        .unwrap_err();
    assert_eq!(refused.code, "ERR_OUT_OF_RANGE");
    assert_eq!(refused.field.as_deref(), Some("item"));
}

// ---- F2: one expense head ----------------------------------------------------------------------

#[test]
fn one_expense_head_has_its_own_series() {
    let b = books();
    let mut s = Session::new(&b.world, &six("bijli ka kharcha"));
    let key = format!("eh{}", b.electricity);
    s.show_key(&key, "05010001", "Electricity");
    let v = s
        .estimate(json!({"measure": "shop_expenses", "head": key}))
        .unwrap();
    assert_eq!(history(&v), vec![4_000, 4_000, 5_000, 5_000, 6_000, 6_000]);
    // Same pace: 17,000 / 3 = 5,666.67, which is 5,667 a month; 5,667 x 6 = 34,002.
    assert_eq!(rs(&v["same_pace"]["per_month"]), 5_667);
    assert_eq!(rs(&v["same_pace"]["total"]), 34_002);
    // Same growth: (6,000 - 4,000) / 5 = 400; 6,400 ... 8,400 = 44,400.
    assert_eq!(rs(&v["same_growth"]["total"]), 44_400);
    // Tea is a series of its own, and not part of electricity.
    let tea_key = format!("eh{}", b.tea);
    s.show_key(&tea_key, "05010002", "Tea");
    let tea = s
        .estimate(json!({"measure": "shop_expenses", "head": tea_key}))
        .unwrap();
    assert_eq!(history(&tea), vec![1_000; 6]);
    assert_eq!(rs(&tea["same_pace"]["total"]), 6_000);
    // The heads add up to the shop expenses of R29 for the same dates.
    let page = mandi_reports::reports::run(
        b.world.env.ctx.db(),
        &serde_json::from_value(json!({"id": "R29", "filters": {}})).unwrap(),
    )
    .unwrap();
    let at = page
        .columns
        .iter()
        .position(|c| c.key == "expense")
        .unwrap();
    let total = match page.totals.get(at).unwrap() {
        mandi_reports::reports::Cell::Money { paisa } => paisa.rupees(),
        other => panic!("{other:?}"),
    };
    assert_eq!(
        total,
        history(&v).iter().sum::<i64>() + history(&tea).iter().sum::<i64>()
    );
    // The shop's whole shop expenses (no head) are the two heads together.
    let all = s.estimate(json!({"measure": "shop_expenses"})).unwrap();
    assert_eq!(
        history(&all),
        vec![5_000, 5_000, 6_000, 6_000, 7_000, 7_000]
    );
    // A head is refused with another measure, and a key the model was never shown is unknown.
    let wrong = s
        .estimate(json!({"measure": "sales", "head": tea_key}))
        .unwrap_err();
    assert_eq!(wrong.field.as_deref(), Some("head"));
    let unknown = s
        .estimate(json!({"measure": "shop_expenses", "head": "eh999"}))
        .unwrap_err();
    assert_eq!(unknown.code, "ERR_UNKNOWN_REFERENCE");
}

#[test]
fn a_month_close_does_not_change_a_head_series() {
    let b = books();
    let mut s = Session::new(&b.world, &six("bijli"));
    let key = format!("eh{}", b.electricity);
    s.show_key(&key, "05010001", "Electricity");
    let before = s
        .estimate(json!({"measure": "shop_expenses", "head": key}))
        .unwrap();
    // The days up to July are closed and July is closed as a month (PRD FR-17.6): the entry moves
    // the expense accounts' July balance to the partners' capital accounts.
    mandi_services::day::close(
        &b.world.env.ctx,
        &b.world.owner,
        &mandi_services::core::vouchers::DayCloseInput {
            up_to: "31-07-2026".into(),
        },
    )
    .unwrap();
    mandi_services::month::close(
        &b.world.env.ctx,
        &b.world.owner,
        &mandi_services::month::MonthInput {
            month: "2026-07".into(),
        },
    )
    .unwrap();
    let mut again = Session::new(&b.world, &six("bijli"));
    again.show_key(&key, "05010001", "Electricity");
    let after = again
        .estimate(json!({"measure": "shop_expenses", "head": key}))
        .unwrap();
    assert_eq!(history(&after), history(&before));
    assert_eq!(rs(&after["same_pace"]["total"]), 34_002);
}

// ---- F3: a partner's profit ---------------------------------------------------------------------

#[test]
fn a_partners_profit_has_a_series_and_a_locked_section_pauses_it() {
    let b = books();
    let partner = format!("s{}", b.world.partners[0]);
    let mut s = Session::new(&b.world, &six("partner alif ka munafa"));
    s.show_key(&partner, "", "Partner A");
    let v = s
        .estimate(json!({"measure": "profit", "partner": partner}))
        .unwrap();
    // Alif has 40%: his share of the income less his share of the shop expenses, month by month.
    assert_eq!(history(&v), vec![-1_200, -720, -640, -160, -80, 400]);
    // Same pace: (-160 - 80 + 400) / 3 = 53.33, which is 53 a month; x 6 = 318.
    assert_eq!(rs(&v["same_pace"]["per_month"]), 53);
    assert_eq!(rs(&v["same_pace"]["total"]), 318);
    // Same growth: (400 + 1,200) / 5 = 320; 720 + 1,040 + 1,360 + 1,680 + 2,000 + 2,320 = 9,120.
    assert_eq!(rs(&v["same_growth"]["total"]), 9_120);
    // The result holds a partner figure, so it is not kept once the section is locked again.
    let id = s.state.results.last().unwrap().id.clone();
    assert!(s.state.find(&id).unwrap().partner_data);
    // All three partners together are the shop's profit.
    let shop = s.estimate(json!({"measure": "profit"})).unwrap();
    assert_eq!(
        history(&shop),
        vec![-3_000, -1_800, -1_600, -400, -200, 1_000]
    );
    // A locked section pauses the question, as for view G with a partner.
    let mut locked = Session::new(&b.world, &six("partner alif ka munafa")).locked();
    locked.show_key(&partner, "", "Partner A");
    let paused = locked
        .estimate(json!({"measure": "profit", "partner": partner}))
        .unwrap_err();
    assert_eq!(paused.code, "LOCKED");
    // A partner is refused with another measure.
    let wrong = s
        .estimate(json!({"measure": "sales", "partner": partner}))
        .unwrap_err();
    assert_eq!(wrong.field.as_deref(), Some("partner"));
}

// ---- F4: a party's balance at this pace --------------------------------------------------------

#[test]
fn a_buyers_balance_at_this_pace() {
    let b = books();
    let mut s = Session::new(&b.world, &six("akram ka balance"));
    let key = format!("a{}", b.world.akram);
    s.show_key(&key, "03040001", "Akram");
    let v = s
        .estimate(json!({"measure": "party_balance", "party": key}))
        .unwrap();
    // The balance now is a book figure, with its side.
    assert_eq!(rs(&v["now"]), 137_400);
    assert_eq!(v["now"]["side"], "بنام");
    // The net change of each month: the bill less what he paid.
    assert_eq!(
        history(&v),
        vec![1_400, 10_000, 18_600, 27_200, 35_800, 44_400]
    );
    assert_eq!(v["too_short"], false);
    // The last 3 months: (27,200 + 35,800 + 44,400) / 3 = 35,800 a month more; after 6 months
    // 137,400 + 6 x 35,800 = 352,200.
    assert_eq!(diff(&v["same_pace"]["per_month"]), (35_800, "up".into()));
    assert_eq!(rs(&v["same_pace"]["total"]), 352_200);
    assert_eq!(v["same_pace"]["total"]["side"], "بنام");
    // After 1 and after 3 months.
    assert_eq!(rs(&v["ahead"][0]["same_pace"]), 173_200);
    assert_eq!(rs(&v["ahead"][2]["same_pace"]), 244_800);
    assert_eq!(v["ahead"][0]["key"], "ah1");
    assert_eq!(v["ahead"][0]["after"]["n"], 1);
    // After 1 month is the month that day falls in: today is in October, so November.
    assert_eq!(v["ahead"][0]["month"]["month"], "2026-11");
    assert_eq!(v["ahead"][5]["month"]["month"], "2027-04");
    // No growth or season method for a balance.
    assert!(v["same_growth"].is_null() && v["same_season"].is_null());
    // The card: the change with its direction, the balance after the months, and its working.
    let result = s.state.results.last().unwrap();
    let Some(mandi_assistant::results::Shown::Chart(_, Some(card))) = &result.shown else {
        panic!("no estimate card");
    };
    let method = card.methods.first().unwrap();
    assert_eq!(
        method.kind,
        mandi_assistant::answer::EstimateMethodKind::SamePace
    );
    assert_eq!(method.working.steps.len(), 3);
    assert!(matches!(
        result.source,
        Some(mandi_assistant::answer::AnswerSource::PartyCard { .. })
    ));
}

#[test]
fn a_sellers_credit_balance_grows_on_the_credit_side() {
    let b = books();
    let mut s = Session::new(&b.world, &six("rahim ka balance"));
    let key = format!("a{}", b.world.rahim);
    s.show_key(&key, "030110001", "Rahim");
    let v = s
        .estimate(json!({"measure": "party_balance", "party": key}))
        .unwrap();
    // Rahim is owed 197,400 (210,000 of goods less 12,600 of commission), all of it unpaid.
    assert_eq!(rs(&v["now"]), 197_400);
    assert_eq!(v["now"]["side"], "جمع");
    assert_eq!(
        history(&v),
        vec![9_400, 18_800, 28_200, 37_600, 47_000, 56_400]
    );
    // The change is a credit: the debit balance falls by 47,000 a month (the last 3 months).
    assert_eq!(diff(&v["same_pace"]["per_month"]), (47_000, "down".into()));
    assert_eq!(rs(&v["same_pace"]["total"]), 479_400);
    assert_eq!(v["same_pace"]["total"]["side"], "جمع");
}

#[test]
fn a_party_with_too_little_history_gets_the_balance_and_no_estimate() {
    let b = books();
    let mut s = Session::new(&b.world, &six("karim ka balance"));
    let key = format!("a{}", b.world.karim);
    s.show_key(&key, "030110002", "Karim");
    let v = s
        .estimate(json!({"measure": "party_balance", "party": key}))
        .unwrap();
    // Karim has no posting at all.
    assert_eq!(v["too_short"], true);
    assert_eq!(v["full_months"]["n"], 0);
    assert!(v["same_pace"].is_null());
    assert_eq!(rs(&v["now"]), 0);
}

#[test]
fn a_party_with_two_months_of_history_is_too_short_and_one_with_more_uses_the_last_three() {
    let b = books();
    let w = &b.world;
    // Arshad buys in August and September only: two full months, so no estimate (PRD FR-7.86 needs 3).
    arshad_bill(w, "16-08-2026");
    arshad_bill(w, "16-09-2026");
    let mut s = Session::new(w, &six("arshad ka balance"));
    let key = format!("a{}", w.arshad);
    s.show_key(&key, "03040002", "Arshad");
    let short = s
        .estimate(json!({"measure": "party_balance", "party": key}))
        .unwrap();
    assert_eq!(short["too_short"], true);
    assert_eq!(short["full_months"]["n"], 2);
    assert_eq!(rs(&short["now"]), 15_000);
    // Akram has six months of change but his pace is the last three.
    let akram = format!("a{}", w.akram);
    s.show_key(&akram, "03040001", "Akram");
    let v = s
        .estimate(json!({"measure": "party_balance", "party": akram}))
        .unwrap();
    assert_eq!(history(&v).len(), 6);
    assert_eq!(v["same_pace"]["months_used"]["n"], 3);
    assert_eq!(v["full_months"]["n"], 6);
}

#[test]
fn a_party_balance_needs_a_real_party() {
    let b = books();
    let mut s = Session::new(&b.world, &six("balance"));
    // No party.
    let none = s.estimate(json!({"measure": "party_balance"})).unwrap_err();
    assert_eq!(none.code, "ERR_REQUIRED");
    // The چکتا کھاتہ account holds walk-in buyers and no balance, so there is nothing to estimate.
    let chukta = format!("a{}", b.world.chukta_khata);
    s.show_key(&chukta, "0030", "Chukta");
    let refused = s
        .estimate(json!({"measure": "party_balance", "party": chukta}))
        .unwrap_err();
    assert_eq!(refused.code, "ERR_OUT_OF_RANGE");
    assert_eq!(refused.field.as_deref(), Some("party"));
    // A party with a measure that is not a party's.
    let akram = format!("a{}", b.world.akram);
    s.show_key(&akram, "03040001", "Akram");
    let wrong = s
        .estimate(json!({"measure": "cash_in_hand", "party": akram}))
        .unwrap_err();
    assert_eq!(wrong.field.as_deref(), Some("party"));
}

// ---- F5: the cash in hand and what the buyers owe ----------------------------------------------

#[test]
fn the_cash_in_hand_at_this_pace() {
    let b = books();
    let mut s = Session::new(&b.world, &six("cash"));
    let v = s.estimate(json!({"measure": "cash_in_hand"})).unwrap();
    // Receipts less the bills: 54,000 in hand, a plain amount with no side.
    assert_eq!(rs(&v["now"]), 54_000);
    assert!(v["now"]["side"].is_null());
    assert_eq!(
        history(&v),
        vec![5_000, 7_000, 8_000, 10_000, 11_000, 13_000]
    );
    // (10,000 + 11,000 + 13,000) / 3 = 11,333.33, which is 11,333 a month.
    assert_eq!(diff(&v["same_pace"]["per_month"]), (11_333, "up".into()));
    assert_eq!(rs(&v["ahead"][0]["same_pace"]), 65_333);
    assert_eq!(rs(&v["ahead"][2]["same_pace"]), 87_999);
    assert_eq!(rs(&v["same_pace"]["total"]), 121_998);
    let result = s.state.results.last().unwrap();
    assert!(matches!(
        result.source,
        Some(mandi_assistant::answer::AnswerSource::Report {
            id: mandi_reports::reports::ReportId::R12,
            ..
        })
    ));
}

#[test]
fn what_the_buyers_owe_at_this_pace_follows_r20() {
    let b = books();
    let mut s = Session::new(&b.world, &six("gahakon ke zimme"));
    let v = s.estimate(json!({"measure": "buyers_dues"})).unwrap();
    assert_eq!(rs(&v["now"]), 137_400);
    assert_eq!(
        history(&v),
        vec![1_400, 10_000, 18_600, 27_200, 35_800, 44_400]
    );
    assert_eq!(diff(&v["same_pace"]["per_month"]), (35_800, "up".into()));
    assert_eq!(rs(&v["same_pace"]["total"]), 352_200);
    // The changes are the very difference of two R20 totals: the dues at the end of September less
    // the dues at the end of March.
    let at = |day: &str| -> i64 {
        let page = mandi_reports::reports::run(
            b.world.env.ctx.db(),
            &serde_json::from_value(json!({"id": "R20", "filters": {"as_of": day}})).unwrap(),
        )
        .unwrap();
        let at = page
            .columns
            .iter()
            .position(|c| c.key == "balance")
            .unwrap();
        match page.totals.get(at).unwrap() {
            mandi_reports::reports::Cell::Money { paisa } => paisa.rupees(),
            mandi_reports::reports::Cell::Balance { paisa, .. } => paisa.rupees(),
            other => panic!("{other:?}"),
        }
    };
    assert_eq!(
        at("2026-09-30") - at("2026-03-31"),
        history(&v).iter().sum::<i64>()
    );
    let result = s.state.results.last().unwrap();
    assert!(matches!(
        result.source,
        Some(mandi_assistant::answer::AnswerSource::Report {
            id: mandi_reports::reports::ReportId::R20,
            ..
        })
    ));
}

#[test]
fn a_balance_is_estimated_for_months_from_today_and_nothing_else() {
    let b = books();
    let mut s = Session::new(&b.world, "saal ke aakhir tak cash kitna hoga");
    // To the end of the year: refused, the Owner is asked how many months.
    let year = s
        .estimate(json!({"measure": "cash_in_hand", "until": "year_end"}))
        .unwrap_err();
    assert_eq!(year.field.as_deref(), Some("until"));
    let year = s.estimate(json!({"measure": "cash_in_hand"})).unwrap_err();
    assert_eq!(year.field.as_deref(), Some("until"));
    // No what-if, no percent, no target.
    for (field, value) in [
        ("what_if", json!("double")),
        ("percent", json!("q.n1")),
        ("target", json!("q.n1")),
    ] {
        let mut s = Session::new(&b.world, &six("cash agar double ho"));
        let refused = s
            .estimate(json!({"measure": "cash_in_hand", field: value}))
            .unwrap_err();
        assert_eq!(refused.field.as_deref(), Some(field), "{field}");
    }
    // A sale filter does not belong to a balance.
    let mut s = Session::new(&b.world, &six("cash"));
    let item = format!("i{}", b.world.banana);
    s.show_key(&item, "0001", "Banana");
    let refused = s
        .estimate(json!({"measure": "buyers_dues", "item": item}))
        .unwrap_err();
    assert_eq!(refused.field.as_deref(), Some("item"));
}

// ---- openings are not a pace (money review B2) -------------------------------------------------

#[test]
fn an_opening_balance_and_its_correction_do_not_move_the_pace() {
    use mandi_reports::analytics::{BalanceSubject, balance_series};
    let b = books();
    let w = &b.world;
    let today = BusinessDate::parse_iso("2026-10-10").unwrap();
    let before = balance_series(w.env.ctx.db(), BalanceSubject::Party(w.akram), today).unwrap();
    assert_eq!(before.now.rupees(), 137_400);
    let amounts = |series: &mandi_reports::analytics::BalanceSeries| -> Vec<i64> {
        series.changes.iter().map(|m| m.amount.rupees()).collect()
    };
    assert_eq!(
        amounts(&before),
        vec![1_400, 10_000, 18_600, 27_200, 35_800, 44_400]
    );
    // An opening of 7,000 dated in May, inside the window: the balance now has it, the changes not.
    let open = |dr: &str| open_akram(w, dr);
    open("7,000");
    let with = balance_series(w.env.ctx.db(), BalanceSubject::Party(w.akram), today).unwrap();
    assert_eq!(with.now.rupees(), 144_400);
    assert_eq!(amounts(&with), amounts(&before));
    // The opening is corrected to 9,000: the old one is reversed on 15-05 and a new one is posted.
    // Neither the reversal nor the new entry is a pace.
    open("9,000");
    let again = balance_series(w.env.ctx.db(), BalanceSubject::Party(w.akram), today).unwrap();
    assert_eq!(again.now.rupees(), 146_400);
    assert_eq!(amounts(&again), amounts(&before));
}

// ---- F1: the garden sales ---------------------------------------------------------------------

#[test]
fn garden_sales_have_a_monthly_series_and_add_up_to_view_i() {
    let w = worked::months_world(&[]);
    let [a, b_partner, _] = w.partners;
    let garden_json = json!({
        "name_ur": "شاہ پور باغ",
        "name_en": "Shahpur Garden",
        "confirmed_rules_words": ["شاہ", "پور", "باغ", "Shahpur", "Garden"],
        "date": "01-04-2026",
        "item_id": w.banana,
        "rent": "50,000",
        "season_end": null,
        "note": null,
        "partners": [
            {"partner_id": a, "share": "60", "source": "shop"},
            {"partner_id": b_partner, "share": "40", "source": "pocket"}
        ],
        "paid_now": null,
        "paid_now_reason": null,
        "parts": [],
        "confirm_cash_below_zero": false
    });
    let created = gardens::create(
        &w.env.ctx,
        &w.owner,
        &serde_json::from_value::<GardenInput>(garden_json).unwrap(),
    )
    .unwrap();
    // Each month: 100 پیٹی sold in our shop at rate 100, 200 ... 600 on the 12th (the seller credit
    // is the value less 6% commission: 9,400 ... 56,400), and a sale in another shop on the 20th
    // that brought 3,000 ... 8,000.
    let other = [3_000, 4_000, 5_000, 6_000, 7_000, 8_000];
    for (i, month) in (4..=9_u32).enumerate() {
        let rate = (i64::try_from(i).unwrap() + 1) * 100;
        let mut lot = world::with_sales(
            world::bare_lot(&w, "garden-1", created.row.account.id, w.banana, "100"),
            vec![world::sale("s-1", w.arshad, "100", &rate.to_string())],
        );
        lot.adjust_advance = false;
        let mut draft = world::draft(vec![lot]);
        draft.header.entry_date = format!("12-{month:02}-2026");
        bikri::save(
            &w.env.ctx,
            &w.owner,
            &BikriSaveInput {
                draft,
                confirm_cash_below_zero: false,
            },
        )
        .unwrap();
        gardens::entry_save(
            &w.env.ctx,
            &w.owner,
            &serde_json::from_value::<GardenEntryInput>(json!({
                "type": "other_sale", "garden_id": created.row.id,
                "date": format!("20-{month:02}-2026"), "shop": "Haji Traders",
                "qty": "20", "amount": other[i].to_string(), "note": null
            }))
            .unwrap(),
        )
        .unwrap();
    }
    let mut s = Session::new(&w, &six("bagh ki bikri"));
    let v = s.estimate(json!({"measure": "garden_sales"})).unwrap();
    assert_eq!(
        history(&v),
        vec![12_400, 22_800, 33_200, 43_600, 54_000, 64_400]
    );
    // (43,600 + 54,000 + 64,400) / 3 = 54,000 a month; x 6 = 324,000.
    assert_eq!(rs(&v["same_pace"]["per_month"]), 54_000);
    assert_eq!(rs(&v["same_pace"]["total"]), 324_000);
    // The months add up to view I: the money from our shop plus the money from other shops.
    let page = mandi_reports::analytics::run(
        w.env.ctx.db(),
        &serde_json::from_value(json!({"view": "I", "filters": {}, "offset": 0, "limit": 100}))
            .unwrap(),
    )
    .unwrap();
    let money = |key: &str| -> i64 {
        let at = page.columns.iter().position(|c| c.key == key).unwrap();
        match page.totals.get(at).unwrap() {
            mandi_reports::reports::Cell::Money { paisa } => paisa.rupees(),
            other => panic!("{other:?}"),
        }
    };
    assert_eq!(
        money("sales_our_shop") + money("sales_other_shops"),
        230_400
    );
    assert_eq!(
        money("sales_our_shop") + money("sales_other_shops"),
        history(&v).iter().sum::<i64>()
    );
    // A trip filter does not belong to garden sales.
    let trip = "t1";
    s.show_key(trip, "0070", "Trip");
    let refused = s
        .estimate(json!({"measure": "garden_sales", "trip": trip}))
        .unwrap_err();
    assert_eq!(refused.field.as_deref(), Some("trip"));
}

// ---- F1: the trip sales ----------------------------------------------------------------------

#[test]
fn trip_sales_have_a_monthly_series_and_add_up_to_view_j() {
    let w = worked::months_world(&[]);
    let [a, b, c] = w.partners;
    let pocket = |partner_id: i64, share: &str| VenturePartnerInput {
        partner_id,
        share: share.into(),
        source: MoneySource::Pocket,
    };
    // A trip of the three partners from their own pockets (so no cash is needed), made in April.
    let trip = trips::create(
        &w.env.ctx,
        &w.owner,
        &TripInput {
            date: "01-04-2026".into(),
            market: Some("Lahore".into()),
            sent_person: None,
            note: None,
            partners: vec![pocket(a, "40"), pocket(b, "30"), pocket(c, "30")],
        },
    )
    .unwrap();
    let bought = purchases::save(
        &w.env.ctx,
        &w.owner,
        &PurchaseInput {
            other_qty: None,
            shop_name: None,
            sell_date: None,
            trip_id: Some(trip.id),
            date: "02-04-2026".into(),
            supplier: Some("Orange Traders".into()),
            item_id: Some(w.banana),
            packing_id: Some(w.box_),
            qty: "100".into(),
            weight: None,
            rate: "100".into(),
            freight: None,
            labour: None,
            other_cost: None,
            note: None,
            parts: Vec::new(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    // Sales in another shop on the 12th of each month: 10 نگ for 1,000 ... 3,500.
    let other = [1_000, 1_500, 2_000, 2_500, 3_000, 3_500];
    for (i, month) in (4..=9_u32).enumerate() {
        trips::other_sale_save(
            &w.env.ctx,
            &w.owner,
            &OtherSaleInput {
                trip_id: Some(trip.id),
                date: format!("12-{month:02}-2026"),
                shop: Some("Haji Shop".into()),
                purchase_id: Some(bought.id),
                qty: "10".into(),
                amount: other[i].to_string(),
                note: None,
            },
        )
        .unwrap();
    }
    // And in June 10 نگ in our shop by an own lot at 300: 3,000 less 6% commission = 2,820 to the
    // trip's partners.
    let mut lot = world::with_sales(
        LotInput {
            seller_account_id: Some(world::id_of(&w.env, "030119998")),
            purchase_id: Some(bought.id),
            item_id: None,
            packing_id: None,
            adjust_advance: false,
            ..world::bare_lot(&w, "own-1", w.rahim, w.banana, "10")
        },
        vec![world::sale("s-1", w.akram, "10", "300")],
    );
    lot.adjust_advance = false;
    let mut draft = world::draft(vec![lot]);
    draft.header.entry_date = "20-06-2026".into();
    bikri::save(
        &w.env.ctx,
        &w.owner,
        &BikriSaveInput {
            draft,
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();

    let mut s = Session::new(&w, &six("safar ki bikri"));
    let v = s.estimate(json!({"measure": "trip_sales"})).unwrap();
    assert_eq!(history(&v), vec![1_000, 1_500, 4_820, 2_500, 3_000, 3_500]);
    // (2,500 + 3,000 + 3,500) / 3 = 3,000 a month; x 6 = 18,000.
    assert_eq!(rs(&v["same_pace"]["per_month"]), 3_000);
    assert_eq!(rs(&v["same_pace"]["total"]), 18_000);
    // The months add up to the sales money of view J.
    let page = mandi_reports::analytics::run(
        w.env.ctx.db(),
        &serde_json::from_value(json!({"view": "J", "filters": {}, "offset": 0, "limit": 100}))
            .unwrap(),
    )
    .unwrap();
    let at = page
        .columns
        .iter()
        .position(|c| c.key == "sales_money")
        .unwrap();
    let total = match page.totals.get(at).unwrap() {
        mandi_reports::reports::Cell::Money { paisa } => paisa.rupees(),
        other => panic!("{other:?}"),
    };
    assert_eq!(total, 16_320);
    assert_eq!(total, history(&v).iter().sum::<i64>());
    // It opens view J, which shows the trips.
    let result = s.state.results.last().unwrap();
    assert!(matches!(
        result.source,
        Some(mandi_assistant::answer::AnswerSource::Analytics {
            view: mandi_reports::analytics::AnalyticsView::J,
            ..
        })
    ));
}

// ---- F6: seasonal items with quiet months -----------------------------------------------------

/// Apple is sold in April, June and September only; banana fills the other months. Each sale is
/// 100 نگ, so the commission of a month is: seller 6% + buyer 6% + لاگا 8 per نگ.
fn seasonal_world() -> world::World {
    worked::months_world(&[
        (4, false, 100, 100), // apple: 600 + 600 + 800 = 2,000
        (5, true, 100, 200),
        (6, false, 100, 300), // apple: 1,800 + 1,800 + 800 = 4,400
        (7, true, 100, 400),
        (8, true, 100, 500),
        (9, false, 100, 600), // apple: 3,600 + 3,600 + 800 = 8,000
    ])
}

#[test]
fn a_month_with_no_sales_of_a_seasonal_item_counts_as_zero() {
    let w = seasonal_world();
    let mut s = Session::new(&w, &six("seb ka commission"));
    let apple = format!("i{}", w.apple);
    s.show_key(&apple, "0002", "Apple");
    let v = s
        .estimate(json!({"measure": "commission_all", "item": apple}))
        .unwrap();
    // April to September, the quiet months are 0 and not missing.
    assert_eq!(history(&v), vec![2_000, 0, 4_400, 0, 0, 8_000]);
    assert_eq!(v["too_short"], false);
    // Same pace: (0 + 0 + 8,000) / 3 = 2,666.67, which is 2,667 a month; x 6 = 16,002.
    assert_eq!(rs(&v["same_pace"]["per_month"]), 2_667);
    assert_eq!(rs(&v["same_pace"]["total"]), 16_002);
    // Same growth: (8,000 - 2,000) / 5 = 1,200; 9,200 ... 15,200 = 73,200.
    assert_eq!(rs(&v["same_growth"]["total"]), 73_200);
}

#[test]
fn an_item_with_fewer_than_three_months_of_history_is_too_short() {
    let w = worked::months_world(&[
        (4, true, 100, 100),
        (5, true, 100, 200),
        (6, true, 100, 300),
        (7, true, 100, 400),
        (8, false, 100, 500), // apple starts in August
        (9, false, 100, 600),
    ]);
    let mut s = Session::new(&w, &six("seb ka commission"));
    let apple = format!("i{}", w.apple);
    s.show_key(&apple, "0002", "Apple");
    let v = s
        .estimate(json!({"measure": "commission_all", "item": apple}))
        .unwrap();
    assert_eq!(v["too_short"], true);
    assert_eq!(v["full_months"]["n"], 2);
    assert!(v["same_pace"].is_null());
}

#[test]
fn quiet_months_count_as_zero_for_a_seller_and_a_buyer_too() {
    // Arshad buys from Karim in April, June and September only (100 نگ at 100, 300, 600); the
    // months between are quiet for both of them. The same rule as for an item: inside the range
    // from the first month a quiet month is 0, and the estimate is made.
    let w = worked::months_world(&[]);
    for (month, rate) in [(4, "100"), (6, "300"), (9, "600")] {
        let lot = world::with_sales(
            world::bare_lot(&w, "lot-1", w.karim, w.banana, "100"),
            vec![world::sale("s-1", w.arshad, "100", rate)],
        );
        let mut draft = world::draft(vec![lot]);
        draft.header.entry_date = format!("15-{month:02}-2026");
        bikri::save(
            &w.env.ctx,
            &w.owner,
            &BikriSaveInput {
                draft,
                confirm_cash_below_zero: false,
            },
        )
        .unwrap();
    }
    let mut s = Session::new(&w, &six("commission"));
    let arshad = format!("a{}", w.arshad);
    let karim = format!("a{}", w.karim);
    s.show_key(&arshad, "03040002", "Arshad");
    s.show_key(&karim, "030110002", "Karim");
    for filter in [json!({"buyer": arshad}), json!({"seller": karim})] {
        let mut input = json!({"measure": "commission_all"});
        for (k, v) in filter.as_object().unwrap() {
            input[k] = v.clone();
        }
        let v = s.estimate(input).unwrap();
        assert_eq!(history(&v), vec![2_000, 0, 4_400, 0, 0, 8_000], "{filter}");
        assert_eq!(v["too_short"], false);
        assert_eq!(rs(&v["same_pace"]["total"]), 16_002);
    }
    // Only a month or two of history: too short, as for any series.
    let mut late = Session::new(&w, &six("commission"));
    late.today = "2026-05-10";
    late.show_key(&arshad, "03040002", "Arshad");
    let short = late
        .estimate(json!({"measure": "commission_all", "buyer": arshad}))
        .unwrap();
    assert_eq!(short["too_short"], true);
}

#[test]
fn an_item_sold_long_ago_and_again_now_is_estimated_over_its_whole_range() {
    // Banana in 2025 is too far for this world, so the earliest sale is April and the item's
    // range is April to September whatever it did in between: the seller series behaves the same.
    let w = seasonal_world();
    let mut s = Session::new(&w, &six("rahim ka commission"));
    let rahim = format!("a{}", w.rahim);
    s.show_key(&rahim, "030110001", "Rahim");
    let v = s
        .estimate(json!({"measure": "commission_all", "seller": rahim}))
        .unwrap();
    assert_eq!(history(&v).len(), 6);
}

// ---- what a pace leaves out (money review K1, K2, K5) ------------------------------------------

#[test]
fn an_opening_inside_the_window_does_not_move_the_dues_pace_either() {
    let b = books();
    let w = &b.world;
    let dues = |question: &str| {
        let mut s = Session::new(w, &six(question));
        s.estimate(json!({"measure": "buyers_dues"})).unwrap()
    };
    let before = dues("gahakon ke zimme");
    assert_eq!(rs(&before["now"]), 137_400);
    // An opening of 7,000 on 15-05: the dues now have it, the changes of the months do not.
    open_akram(w, "7,000");
    let with = dues("gahakon ke zimme");
    assert_eq!(rs(&with["now"]), 144_400);
    assert_eq!(history(&with), history(&before));
    assert_eq!(diff(&with["same_pace"]["per_month"]), (35_800, "up".into()));
    assert_eq!(rs(&with["same_pace"]["total"]), 359_200);
    // The opening is corrected to 9,000: the old one is reversed on 15-05 and a new one posted.
    open_akram(w, "9,000");
    let again = dues("gahakon ke zimme");
    assert_eq!(rs(&again["now"]), 146_400);
    assert_eq!(history(&again), history(&before));
    // The very rule of R20, for the plain case: the levels are R20's totals.
    let r20 = |day: &str| -> i64 {
        let page = mandi_reports::reports::run(
            w.env.ctx.db(),
            &serde_json::from_value(json!({"id": "R20", "filters": {"as_of": day}})).unwrap(),
        )
        .unwrap();
        let at = page
            .columns
            .iter()
            .position(|c| c.key == "balance")
            .unwrap();
        match page.totals.get(at).unwrap() {
            mandi_reports::reports::Cell::Money { paisa } => paisa.rupees(),
            mandi_reports::reports::Cell::Balance { paisa, .. } => paisa.rupees(),
            other => panic!("{other:?}"),
        }
    };
    // R20 has the 9,000 from May on; the trading changes of April to September add up to the
    // dues at the end of September less that opening.
    assert_eq!(
        r20("2026-09-30") - 9_000,
        history(&again).iter().sum::<i64>()
    );
}

#[test]
fn a_move_to_the_dead_amount_is_a_one_off_and_not_a_pace() {
    let b = books();
    let w = &b.world;
    // Arshad buys on credit in April (11,400) and is moved to the dead amount on 15-07: his
    // balance leaves the dues from then on, but nothing posts and nothing is a trend.
    let lot = world::with_sales(
        world::bare_lot(w, "lot-1", w.rahim, w.banana, "100"),
        vec![world::sale("s-1", w.arshad, "100", "100")],
    );
    let mut draft = world::draft(vec![lot]);
    draft.header.entry_date = "16-04-2026".into();
    bikri::save(
        &w.env.ctx,
        &w.owner,
        &BikriSaveInput {
            draft,
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    let mut s = Session::new(w, &six("gahakon ke zimme"));
    let plain = s.estimate(json!({"measure": "buyers_dues"})).unwrap();
    assert_eq!(rs(&plain["now"]), 137_400 + 11_400);
    dead::mark(
        &w.env.ctx,
        &w.owner,
        &DeadMarkInput {
            account_id: w.arshad,
            entry_date: "15-07-2026".into(),
            reason: DeadReason::Left,
            note: String::new(),
        },
    )
    .unwrap();
    let mut s = Session::new(w, &six("gahakon ke zimme"));
    let v = s.estimate(json!({"measure": "buyers_dues"})).unwrap();
    // The dues now (the tile) no longer hold him ...
    assert_eq!(rs(&v["now"]), 137_400);
    // ... but July shows no drop: his 11,400 stays in the April change and nothing leaves in July.
    assert_eq!(
        history(&v),
        vec![12_800, 10_000, 18_600, 27_200, 35_800, 44_400]
    );
    // The last 3 months are July to September, which his move does not touch: (27,200 + 35,800 +
    // 44,400) / 3 = 35,800 a month; 137,400 + 6 x 35,800 = 352,200.
    assert_eq!(diff(&v["same_pace"]["per_month"]), (35_800, "up".into()));
    assert_eq!(rs(&v["same_pace"]["total"]), 352_200);
}

#[test]
fn capital_put_in_or_taken_out_is_not_the_cash_pace() {
    let b = books();
    let w = &b.world;
    let capital = |date: &str, amount: &str| CapitalInput {
        date: date.into(),
        partner_id: Some(w.partners[0]),
        amount: amount.into(),
        narration: None,
        confirm_cash_below_zero: false,
    };
    let cash = |question: &str| {
        let mut s = Session::new(w, &six(question));
        s.estimate(json!({"measure": "cash_in_hand"})).unwrap()
    };
    let before = cash("cash");
    assert_eq!(rs(&before["now"]), 54_000);
    // 50,000 put in on 20-06, 10,000 taken out on 05-08, and 30,000 put in on 10-07 and cancelled:
    // the cash now moves, the pace does not.
    vouchers::save_capital_in(&w.env.ctx, &w.owner, &capital("20-06-2026", "50,000")).unwrap();
    vouchers::save_capital_out(&w.env.ctx, &w.owner, &capital("05-08-2026", "10,000")).unwrap();
    let saved =
        vouchers::save_capital_in(&w.env.ctx, &w.owner, &capital("10-07-2026", "30,000")).unwrap();
    let version: i64 = common::raw(&w.dir)
        .query_row(
            "SELECT version FROM vouchers WHERE id = ?1",
            [saved.voucher_id],
            |r| r.get(0),
        )
        .unwrap();
    vouchers::cancel(
        &w.env.ctx,
        &w.owner,
        &VoucherCancelInput {
            id: saved.voucher_id,
            version,
            reason: "typing mistake".into(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    let v = cash("cash");
    assert_eq!(rs(&v["now"]), 94_000);
    assert_eq!(history(&v), history(&before));
    assert_eq!(diff(&v["same_pace"]["per_month"]), (11_333, "up".into()));
    assert_eq!(rs(&v["ahead"][0]["same_pace"]), 105_333);
    // The working says what it leaves out: the cash has a title of its own, apart from the party's
    // and the dues'.
    let mut s = Session::new(w, &six("cash"));
    s.estimate(json!({"measure": "cash_in_hand"})).unwrap();
    let Some(mandi_assistant::results::Shown::Chart(_, Some(card))) =
        &s.state.results.last().unwrap().shown
    else {
        panic!("no estimate card");
    };
    assert_eq!(
        card.methods.first().unwrap().working.title_id,
        "assistant-working-balance-pace-cash"
    );
}

#[test]
fn a_buyer_paying_down_an_opening_moves_the_dues_down() {
    // Money review B1. Arshad starts with an opening of 50,000 owed. Each month April to September
    // he buys 7,500 and pays 10,000: R20 has him at 47,500, 45,000 ... 35,000, a fall of 2,500 a
    // month. The pace must fall by 2,500 too: the 0 floor acts on his whole balance, not on his
    // trading lines alone (which would be -2,500, -5,000 ... and floor to 0).
    let b = books();
    let w = &b.world;
    open_arshad(w, "50,000", "");
    for month in 4..=9 {
        arshad_bill(w, &format!("16-{month:02}-2026"));
        arshad_pays(w, &format!("21-{month:02}-2026"), "10,000");
    }
    let mut s = Session::new(w, &six("gahakon ke zimme"));
    let v = s.estimate(json!({"measure": "buyers_dues"})).unwrap();
    // The dues now: Akram's 137,400 and Arshad's 35,000.
    assert_eq!(rs(&v["now"]), 172_400);
    // Akram's changes less 2,500 each month (the first month falls by 1,100).
    assert_eq!(
        changes_of(&v),
        vec![-1_100, 7_500, 16_100, 24_700, 33_300, 41_900]
    );
    // (24,700 + 33,300 + 41,900) / 3 = 33,300 a month; 172,400 + 6 x 33,300 = 372,200.
    assert_eq!(diff(&v["same_pace"]["per_month"]), (33_300, "up".into()));
    assert_eq!(rs(&v["same_pace"]["total"]), 372_200);
}

#[test]
fn a_buyer_with_an_opening_credit_adds_nothing_until_he_owes() {
    // Arshad starts 20,000 in credit and buys 7,500 a month on credit: -12,500, -5,000, then he owes
    // 2,500, 10,000, 17,500, 25,000. Only the part above 0 is dues: 0, 0, 2,500, 7,500, 7,500, 7,500.
    let b = books();
    let w = &b.world;
    open_arshad(w, "", "20,000");
    for month in 4..=9 {
        arshad_bill(w, &format!("16-{month:02}-2026"));
    }
    let mut s = Session::new(w, &six("gahakon ke zimme"));
    let v = s.estimate(json!({"measure": "buyers_dues"})).unwrap();
    assert_eq!(rs(&v["now"]), 137_400 + 25_000);
    assert_eq!(
        history(&v),
        vec![1_400, 10_000, 21_100, 34_700, 43_300, 51_900]
    );
}

#[test]
fn every_month_of_the_dues_pace_is_the_difference_of_two_r20_totals() {
    // No openings and nobody dead: the change of each month is R20 at its end less R20 at the end
    // of the month before. Arshad appears in June, overpays in August (a credit that R20 counts as
    // 0 dues) and owes again in September.
    let b = books();
    let w = &b.world;
    arshad_bill(w, "16-06-2026");
    arshad_pays(w, "10-08-2026", "15,000");
    arshad_bill(w, "16-09-2026");
    arshad_bill(w, "17-09-2026");
    let mut s = Session::new(w, &six("gahakon ke zimme"));
    let v = s.estimate(json!({"measure": "buyers_dues"})).unwrap();
    let ends = [
        "2026-03-31",
        "2026-04-30",
        "2026-05-31",
        "2026-06-30",
        "2026-07-31",
        "2026-08-31",
        "2026-09-30",
    ];
    let levels: Vec<i64> = ends.iter().map(|d| r20_as_of(w, d)).collect();
    let by_r20: Vec<i64> = levels.windows(2).map(|p| p[1] - p[0]).collect();
    assert_eq!(history(&v), by_r20);
    assert_eq!(rs(&v["now"]), *levels.last().unwrap());
}

#[test]
fn a_recovery_from_a_dead_buyer_is_a_one_off_too() {
    // Money review S4. Arshad owes 7,500 from April and is moved to the dead amount on 15-07; on
    // 20-08 he pays 5,000. R20 and the tile do not move for it (he is already out of them), so the
    // pace must not show a drop in August either.
    let b = books();
    let w = &b.world;
    arshad_bill(w, "16-04-2026");
    dead::mark(
        &w.env.ctx,
        &w.owner,
        &DeadMarkInput {
            account_id: w.arshad,
            entry_date: "15-07-2026".into(),
            reason: DeadReason::Left,
            note: String::new(),
        },
    )
    .unwrap();
    let mut s = Session::new(w, &six("gahakon ke zimme"));
    let before = s.estimate(json!({"measure": "buyers_dues"})).unwrap();
    arshad_pays(w, "20-08-2026", "5,000");
    let mut s = Session::new(w, &six("gahakon ke zimme"));
    let after = s.estimate(json!({"measure": "buyers_dues"})).unwrap();
    assert_eq!(history(&after), history(&before));
    assert_eq!(rs(&after["now"]), rs(&before["now"]));
}

#[test]
fn calculate_refuses_every_estimated_figure_and_still_reads_the_books() {
    // PRD FR-7.76 point 4: an estimate must never come back as a book figure. The estimate result
    // has book figures (`now`, `history`) and estimated ones (`same_pace`, `ahead`, ...).
    let b = books();
    let w = &b.world;
    let mut s = Session::new(w, &six("akram ka bakaya"));
    s.estimate(json!({"measure": "buyers_dues"})).unwrap();
    let id = s.state.results.last().unwrap().id.clone();
    // An estimated figure is refused, alone, beside a book figure, and as a row of `ahead`.
    for path in [
        format!("{id}.same_pace.per_month"),
        format!("{id}.same_pace.total"),
        format!("{id}.ahead.*.same_pace"),
    ] {
        let e = s
            .run(
                "calculate",
                &json!({"op": "sum", "inputs": [path, format!("{id}.now")]}),
            )
            .unwrap_err();
        assert_eq!(e.code, "ERR_OUT_OF_RANGE", "{path}");
    }
    // The book figures of the same result still go into a calculation: nothing was owed before
    // April, so the net changes of the months of `history` add up to the dues now, 137,400.
    let ok = s
        .run(
            "calculate",
            &json!({"op": "sum", "inputs": [format!("{id}.history.*.value")]}),
        )
        .unwrap();
    assert_eq!(rs(&ok["result"]), 137_400);
}
