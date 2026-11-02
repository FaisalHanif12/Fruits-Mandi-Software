//! One test for each rule of PRD FR-7.76 (Code Standards Q4, Q6): what the checks refuse in the
//! text of an answer and in its next questions, and what they let through. The results are built
//! by hand, so each case shows the rule and nothing else.
#![cfg(test)]

#[path = "../../services/tests/common/mod.rs"]
mod common;

use mandi_assistant::check::{Check, Failure, NameSet, Verdict};
use mandi_assistant::model_answer::ModelAnswer;
use mandi_assistant::results::{Money, NameVal, Node, Shown, Side, ToolResult, Val, Words};
use mandi_assistant::tools::{Env, State};
use mandi_reports::dashboard::BackupFacts;
use mandi_services::auth;
use mandi_services::core::auth::Language;
use mandi_services::core::dates::BusinessDate;
use mandi_services::core::money::Paisa;
use serde_json::{Value, json};

fn rs(rupees: i64) -> Paisa {
    Paisa::from_rupees(rupees).unwrap()
}

fn name(ur: &str, en: &str) -> Node {
    Node::Val(Val::Name(NameVal {
        name_ur: ur.into(),
        name_en: en.into(),
        tag: None,
        code: Some("0020".into()),
    }))
}

fn balance(rupees: i64, side: Side, checked: bool) -> Node {
    Node::Val(Val::Money(Money {
        paisa: rs(rupees),
        side: Some(side),
        checked: Some(checked),
        places: vec![("party_card", rs(rupees)), ("R13", rs(rupees))],
    }))
}

fn plain(rupees: i64) -> Node {
    Node::Val(Val::Money(Money::plain(rs(rupees))))
}

fn result(id: &str, tool: &'static str, turn: u32, node: Node) -> ToolResult {
    ToolResult {
        id: id.into(),
        tool,
        input: Value::Null,
        node,
        shown: None,
        source: None,
        partner_parts_locked: false,
        partner_data: false,
        turn,
        working: None,
    }
}

/// The results of the question being answered (turn 1) and one of an earlier question (turn 0).
fn state() -> State {
    let find = Node::obj(vec![
        (
            "matches",
            Node::List(vec![Node::obj(vec![
                ("key", Node::Str("a20".into())),
                ("name", name("اکرم ٹول ٹیکس", "Akram Toll Tax")),
                ("balance", balance(2_880, Side::Dr, true)),
            ])]),
        ),
        ("more", Node::Bool(false)),
    ]);
    let card = Node::obj(vec![
        ("balance", balance(2_880, Side::Dr, true)),
        ("open_advance", plain(5_000)),
        ("days", Node::Val(Val::Count(36))),
        ("day", Node::Val(Val::Date("2026-10-10".into()))),
        (
            "month",
            Node::Val(Val::Month {
                year: 2026,
                month: 9,
            }),
        ),
    ]);
    let seller = Node::obj(vec![("balance", balance(10_160, Side::Cr, true))]);
    let differing = Node::obj(vec![("balance", balance(2_880, Side::Dr, false))]);
    let estimate = Node::obj(vec![
        ("full_months", Node::Val(Val::Count(6))),
        (
            "same_pace",
            Node::obj(vec![
                ("total", plain(288_000)),
                ("per_month", plain(48_000)),
            ]),
        ),
    ]);
    let mut table = result(
        "r6",
        "report",
        1,
        Node::obj(vec![("count", Node::Val(Val::Count(2)))]),
    );
    table.shown = Some(Shown::Table(mandi_assistant::answer::AnswerTable {
        result_id: String::new(),
        title_id: "report-r13-title".into(),
        columns: Vec::new(),
        rows: Vec::new(),
        count: 0,
        totals_label_id: "report-total".into(),
        totals: Vec::new(),
    }));
    State {
        turn: 1,
        results: vec![
            result("r0", "report", 0, Node::obj(vec![("balance", plain(1))])),
            result("r1", "find", 1, find),
            result("r2", "party_card", 1, card),
            result("r3", "party_card", 1, seller),
            result("r4", "party_card", 1, differing),
            result("e1", "estimate", 1, estimate),
            table,
        ],
    }
}

thread_local! {
    /// One set-up firm for each test (a test runs on its own thread), made on first use.
    static FIRM: (tempfile::TempDir, mandi_services::testing::TestEnv, auth::Actor) = {
        let (dir, env) = common::env();
        common::set_up(&env);
        common::login_owner(&env);
        let owner = auth::require(&env.ctx).unwrap();
        (dir, env, owner)
    };
}

fn verdict_in(language: Language, answer: Value) -> Verdict {
    FIRM.with(|(_dir, env, owner)| verdict_with(env, owner, language, answer))
}

fn verdict_with(
    env: &mandi_services::testing::TestEnv,
    owner: &auth::Actor,
    language: Language,
    answer: Value,
) -> Verdict {
    let words = Words::new().unwrap();
    let numbers = Vec::new();
    let e = Env {
        ctx: &env.ctx,
        actor: owner,
        words: &words,
        language,
        today: BusinessDate::parse_iso("2026-10-10").unwrap(),
        partner_locked: false,
        numbers: &numbers,
        backup: BackupFacts {
            cloud_on: false,
            cloud_age_hours: None,
        },
    };
    let state = state();
    let names = NameSet::new(&state, &[]);
    let mut full = json!({"kind": "answer", "text": "", "show": [], "next": [], "choose": null,
                          "entry": null, "nearest": null});
    for (k, v) in answer.as_object().unwrap() {
        full[k] = v.clone();
    }
    let parsed = ModelAnswer::parse(&full.to_string()).unwrap();
    Check {
        env: &e,
        state: &state,
        names: &names,
    }
    .run(&parsed)
}

fn verdict(answer: Value) -> Verdict {
    verdict_in(Language::Ur, answer)
}

fn rules(v: Verdict) -> Vec<&'static str> {
    match v {
        Verdict::Retry(failures) => failures.iter().map(|f: &Failure| f.rule).collect(),
        Verdict::Ok => Vec::new(),
        Verdict::Differ(_) => vec!["differ"],
    }
}

fn text(t: &str) -> Value {
    json!({"text": t})
}

#[test]
fn a_clean_answer_with_references_passes() {
    let v = verdict(json!({
        "text": "{{r1.matches.0.name}} کے ذمے {{r2.balance}} باقی ہیں۔\n• ان کا بل {{r2.days}} دن پرانا ہے۔",
        "next": ["پچھلے مہینے کا حساب؟", "{{r1.matches.0.name}} کی پرانی بکریاں"],
    }));
    assert_eq!(v, Verdict::Ok);
}

#[test]
fn a_digit_of_any_script_is_refused_in_the_text_and_in_the_next_questions() {
    for typed in ["2,880", "٢٨٨٠", "۲۸۸۰", "２８８０", "28", "½"] {
        let v = verdict(text(&format!("اکرم کے ذمے {typed} باقی ہیں۔")));
        assert!(rules(v).contains(&"digit"), "{typed}");
    }
    let v = verdict(json!({"text": "{{r2.balance}}", "next": ["کیا 5000 دیے؟"]}));
    assert!(rules(v).contains(&"digit"));
}

#[test]
fn a_number_word_is_refused_but_both_is_not() {
    for typed in [
        "دو ہزار آٹھ سو",
        "ڈھائی لاکھ",
        "ایک",
        "سو",
        "آدھا",
        "ڈیڑھ",
        "کروڑ",
    ] {
        let v = verdict(text(&format!("بقایا {typed} ہے۔")));
        assert!(rules(v).contains(&"number_word"), "{typed}");
    }
    assert_eq!(
        verdict(text("دونوں کے ذمے {{r2.balance}} باقی ہیں۔")),
        Verdict::Ok
    );
    let v = verdict_in(Language::En, text("He owes two thousand and one lakh."));
    assert!(rules(v).contains(&"number_word"));
}

/// Every Urdu number word of a decade is refused (robustness finding attack_03: `words.tsv` has
/// 1 to 25 and the tens only; `number_words.rs` adds the rest).
macro_rules! decade {
    ($name:ident, [$($word:literal),+ $(,)?]) => {
        #[test]
        fn $name() {
            for typed in [$($word),+] {
                let v = verdict(text(&format!("بقایا {typed} ہے۔")));
                assert!(rules(v).contains(&"number_word"), "{typed}");
            }
        }
    };
}

decade!(
    number_words_of_the_twenties_are_refused,
    [
        "بیس",
        "اکیس",
        "بائیس",
        "تئیس",
        "چوبیس",
        "پچیس",
        "چھبیس",
        "ستائیس",
        "اٹھائیس",
        "انتیس"
    ]
);
decade!(
    number_words_of_the_thirties_are_refused,
    [
        "تیس",
        "اکتیس",
        "بتیس",
        "تینتیس",
        "چونتیس",
        "پینتیس",
        "چھتیس",
        "سینتیس",
        "اڑتیس",
        "انتالیس"
    ]
);
decade!(
    number_words_of_the_forties_are_refused,
    [
        "چالیس",
        "اکتالیس",
        "بیالیس",
        "تینتالیس",
        "چوالیس",
        "پینتالیس",
        "چھیالیس",
        "سینتالیس",
        "اڑتالیس",
        "انچاس"
    ]
);
decade!(
    number_words_of_the_fifties_are_refused,
    [
        "پچاس",
        "اکاون",
        "باون",
        "ترپن",
        "چون",
        "پچپن",
        "چھپن",
        "ستاون",
        "اٹھاون",
        "انسٹھ"
    ]
);
decade!(
    number_words_of_the_sixties_are_refused,
    [
        "ساٹھ",
        "اکسٹھ",
        "باسٹھ",
        "ترسٹھ",
        "چونسٹھ",
        "پینسٹھ",
        "چھیاسٹھ",
        "سڑسٹھ",
        "اڑسٹھ",
        "انہتر"
    ]
);
decade!(
    number_words_of_the_seventies_are_refused,
    [
        "ستر",
        "اکہتر",
        "بہتر روپے",
        "تہتر",
        "چوہتر",
        "پچھتر",
        "چھہتر",
        "ستتر",
        "اٹھہتر",
        "اناسی"
    ]
);
decade!(
    number_words_of_the_eighties_are_refused,
    [
        "اسی روپے",
        "اکیاسی",
        "بیاسی",
        "تراسی",
        "چوراسی",
        "پچاسی",
        "چھیاسی",
        "ستاسی",
        "اٹھاسی",
        "نواسی"
    ]
);
decade!(
    number_words_of_the_nineties_are_refused,
    [
        "نوے",
        "اکانوے",
        "بانوے",
        "ترانوے",
        "چورانوے",
        "پچانوے",
        "چھیانوے",
        "ستانوے",
        "اٹھانوے",
        "ننانوے"
    ]
);

#[test]
fn zero_and_the_other_spellings_the_review_found_are_refused() {
    for typed in [
        "صفر",
        "چوتیس",
        "اکیاون",
        "تریپن",
        "تریسٹھ",
        "انھتر",
        "اکھتر",
        "چھیہتر",
        "ستہتر",
        "اٹھتر",
        "اکیانوے",
        "ہزاروں",
        "سینکڑوں",
        "کروڑوں",
        "اربوں",
        "اسی پیٹیاں",
        "بہتر کریٹ",
    ] {
        let v = verdict(text(&format!("بقایا {typed} ہے۔")));
        assert!(rules(v).contains(&"number_word"), "{typed}");
    }
    for typed in ["sifar", "zero", "dus", "chodah", "assee", "karor"] {
        let v = verdict_in(Language::En, text(&format!("He owes {typed} bills.")));
        assert!(rules(v).contains(&"number_word"), "{typed}");
    }
    // The usual word for spoiled goods is not a number.
    let v = verdict_in(Language::En, text("The goods are kharab."));
    assert!(!rules(v).contains(&"number_word"));
}

#[test]
fn roman_urdu_number_words_are_refused_and_plain_english_words_are_not() {
    for typed in [
        "ek", "chaar", "paanch", "saat", "aath", "nau", "das", "bees", "atthaees", "ikatees",
        "chalees", "pachaas", "saath", "sattar", "assi", "nabbe", "ninanwe", "sau", "hazaar",
    ] {
        let v = verdict_in(Language::En, text(&format!("He owes {typed} bills.")));
        assert!(rules(v).contains(&"number_word"), "{typed}");
    }
    // Words that look like Roman Urdu numbers but are English (do, teen, char) or Urdu words that
    // are also number words (اسی رفتار, بہتر) are not numbers here.
    let v = verdict_in(
        Language::En,
        text("Do not call; a teen may char the toast."),
    );
    assert!(!rules(v).contains(&"number_word"));
    for ordinary in ["اسی رفتار سے بکری چلتی رہی۔", "یہ بات بہتر ہے۔"]
    {
        let v = verdict(text(ordinary));
        assert!(!rules(v).contains(&"number_word"), "{ordinary}");
    }
}

#[test]
fn a_month_name_is_refused_but_a_lower_case_may_is_not() {
    for typed in ["اکتوبر", "ستمبر", "دسمبر"] {
        assert!(
            rules(verdict(text(&format!("{typed} میں {{{{r2.balance}}}}")))).contains(&"month"),
            "{typed}"
        );
    }
    assert!(
        rules(verdict_in(
            Language::En,
            text("In October he owed {{r2.balance}}.")
        ))
        .contains(&"month")
    );
    assert!(rules(verdict_in(Language::En, text("May {{r2.balance}}."))).contains(&"month"));
    assert!(
        !rules(verdict_in(Language::En, text("He may owe {{r2.balance}}."))).contains(&"month")
    );
}

#[test]
fn a_name_from_the_results_is_refused_outside_a_reference_but_a_glossary_word_is_not() {
    let v = verdict(text("اکرم ٹول ٹیکس کے ذمے {{r2.balance}} باقی ہیں۔"));
    assert!(rules(v).contains(&"name"));
    let v = verdict_in(
        Language::En,
        text("Akram Toll Tax owes the shop {{r2.balance}}."),
    );
    assert!(rules(v).contains(&"name"));
    // A next question too.
    assert!(
        rules(verdict(
            json!({"text": "{{r2.balance}}", "next": ["اکرم ٹول ٹیکس کی بکریاں؟"]})
        ))
        .contains(&"name")
    );
}

#[test]
fn a_typed_check_mark_is_refused() {
    for mark in ["✓", "✔", "☑"] {
        let v = verdict(text(&format!("{{{{r2.balance}}}} {mark} باقی ہیں۔")));
        assert!(rules(v).contains(&"check_mark"), "{mark}");
    }
}

#[test]
fn a_reference_that_is_not_a_value_of_this_question_is_refused() {
    // Unknown result, unknown field, a path that ends on a record and not on a value.
    for path in [
        "r9.balance",
        "r2.nothing",
        "r1.matches.0",
        "r1.matches",
        "r1.matches.4.name",
        "q.n1",
    ] {
        let v = verdict(text(&format!("بقایا {{{{{path}}}}} ہے۔")));
        assert!(rules(v).contains(&"reference"), "{path}");
    }
    // A result of an earlier question.
    let v = verdict(text("بقایا {{r0.balance}} ہے۔"));
    assert!(rules(v).contains(&"earlier_question"));
    // A brace that never closes.
    assert!(rules(verdict(text("بقایا {{r2.balance ہے۔"))).contains(&"reference"));
}

#[test]
fn a_figure_whose_two_places_differ_is_final() {
    let v = verdict(text(
        "{{r1.matches.0.name}} کے ذمے {{r4.balance}} باقی ہیں۔",
    ));
    assert_eq!(v, Verdict::Differ(vec!["r4.balance".to_owned()]));
}

#[test]
fn the_words_of_the_other_side_are_refused_next_to_a_balance() {
    // A debit balance (he owes the shop) with "the shop owes him" words.
    let v = verdict(text("{{r1.matches.0.name}} کو {{r2.balance}} دینے ہیں۔"));
    assert!(rules(v).contains(&"side_words"));
    // A credit balance with "he owes" words.
    let v = verdict(text(
        "{{r1.matches.0.name}} کے ذمے {{r3.balance}} باقی ہیں۔",
    ));
    assert!(rules(v).contains(&"side_words"));
    // The figure first, the words after it: still the same window.
    let v = verdict(text("{{r2.balance}} {{r1.matches.0.name}} کو دینے ہیں۔"));
    assert!(rules(v).contains(&"side_words"));
    // English.
    let v = verdict_in(Language::En, text("The shop owes him {{r2.balance}}."));
    assert!(rules(v).contains(&"side_words"));
    // The right words pass, and so does a sentence with the other side's words after another
    // figure or a comma: "کریم کو ... دینے ہیں، ان کے ذمے پیشگی ...".
    assert_eq!(
        verdict(text("{{r1.matches.0.name}} کو {{r3.balance}} دینے ہیں۔")),
        Verdict::Ok
    );
    assert_eq!(
        verdict(text(
            "{{r1.matches.0.name}} کو {{r3.balance}} دینے ہیں، ان کے ذمے پیشگی {{r2.open_advance}} ہے۔"
        )),
        Verdict::Ok
    );
    assert_eq!(
        verdict(text(
            "{{r1.matches.0.name}} کے ذمے {{r2.balance}} باقی ہیں۔\n• {{r3.balance}} دینے ہیں۔"
        )),
        Verdict::Ok
    );
}

#[test]
fn the_wrong_script_and_the_never_use_words_are_refused() {
    assert!(
        rules(verdict(text(
            "{{r1.matches.0.name}} کے ذمے {{r2.balance}} balance ہے۔"
        )))
        .contains(&"script")
    );
    assert!(
        rules(verdict_in(
            Language::En,
            text("{{r1.matches.0.name}} owes {{r2.balance}} باقی")
        ))
        .contains(&"script")
    );
    for bad in ["ڈیٹا", "سسٹم", "ٹول", "ماڈل", "پکا کھاتہ", "ووچر"] {
        assert!(
            rules(verdict(text(&format!("{bad} {{{{r2.balance}}}}")))).contains(&"never_word"),
            "{bad}"
        );
    }
    for bad in [
        "data",
        "Data",
        "system",
        "tool",
        "model",
        "JSON",
        "Pakka Khata",
    ] {
        assert!(
            rules(verdict_in(
                Language::En,
                text(&format!("The {bad} says {{{{r2.balance}}}}."))
            ))
            .contains(&"never_word"),
            "{bad}"
        );
    }
}

#[test]
fn an_answer_is_at_most_130_words_with_each_reference_as_one() {
    let words = |n: usize| vec!["لفظ"; n].join(" ");
    assert_eq!(
        verdict(text(&format!("{} {{{{r2.balance}}}}", words(129)))),
        Verdict::Ok
    );
    assert!(
        rules(verdict(text(&format!("{} {{{{r2.balance}}}}", words(130))))).contains(&"too_long")
    );
}

#[test]
fn an_estimate_figure_needs_the_estimate_or_advice_kind() {
    let t = "اس رفتار سے {{e1.same_pace.total}} بنے گا۔";
    assert!(rules(verdict(text(t))).contains(&"estimate_kind"));
    assert_eq!(verdict(json!({"kind": "estimate", "text": t})), Verdict::Ok);
    assert_eq!(verdict(json!({"kind": "advice", "text": t})), Verdict::Ok);
    // The number of full months is not an estimate figure.
    assert_eq!(
        verdict(text("کتابوں میں {{e1.full_months}} مکمل مہینے ہیں۔")),
        Verdict::Ok
    );
}

#[test]
fn shows_and_pointers_must_name_results_of_this_question() {
    let table = |result: &str, rows: Option<i64>| json!({"text": "{{r2.balance}}", "show": [{"result": result, "as": "table", "rows": rows}]});
    assert_eq!(verdict(table("r6", Some(7))), Verdict::Ok);
    assert!(rules(verdict(table("r6", Some(8)))).contains(&"show"));
    assert!(rules(verdict(table("r9", None))).contains(&"show"));
    assert!(
        rules(verdict(table("r0", None))).contains(&"show"),
        "a result of an earlier question"
    );
    // A chart of a result that has no chart.
    let chart =
        json!({"text": "{{r2.balance}}", "show": [{"result": "r6", "as": "chart", "rows": null}]});
    assert!(rules(verdict(chart)).contains(&"show"));
    // More than 3 shown, more than 3 next questions.
    let many = json!({"text": "{{r2.balance}}", "show": [
        {"result": "r6", "as": "table", "rows": null}, {"result": "r6", "as": "table", "rows": null},
        {"result": "r6", "as": "table", "rows": null}, {"result": "r6", "as": "table", "rows": null}]});
    assert!(rules(verdict(many)).contains(&"show"));
    assert!(
        rules(verdict(
            json!({"text": "{{r2.balance}}", "next": ["ا", "ب", "ج", "د"]})
        ))
        .contains(&"next")
    );
    // A next question holds a name only as a reference, never a figure.
    assert!(
        rules(verdict(
            json!({"text": "{{r2.balance}}", "next": ["{{r2.balance}} کا حساب؟"]})
        ))
        .contains(&"next")
    );
    assert_eq!(
        verdict(json!({"text": "{{r2.balance}}", "next": ["{{r1.matches.0.name}} کی بکریاں؟"]})),
        Verdict::Ok
    );
}

#[test]
fn which_one_not_found_and_change_request_need_what_they_point_to() {
    // which_one needs a find result with two or more matches; this one has one.
    let v = verdict(json!({"kind": "which_one", "choose": "r1"}));
    assert!(rules(v).contains(&"choose"));
    assert!(rules(verdict(json!({"kind": "which_one", "choose": null}))).contains(&"choose"));
    assert!(
        rules(verdict(json!({"kind": "which_one", "choose": "r2"}))).contains(&"choose"),
        "not a find result"
    );
    // not_found needs a find with no match.
    assert!(rules(verdict(json!({"kind": "not_found", "choose": "r1"}))).contains(&"choose"));
    // A change request needs its screen; a party that is not a find match is refused.
    assert!(rules(verdict(json!({"kind": "change_request"}))).contains(&"entry"));
    assert_eq!(
        verdict(
            json!({"kind": "change_request", "entry": {"screen": "wasooli", "party": "r1.matches.0"}})
        ),
        Verdict::Ok
    );
    assert!(rules(verdict(json!({"kind": "change_request", "entry": {"screen": "wasooli", "party": "r7.matches.0"}}))).contains(&"entry"));
    // The text of a kind that has no text of the model is not checked.
    assert_eq!(
        verdict(json!({"kind": "need_dates", "text": "2,880"})),
        Verdict::Ok
    );
}
