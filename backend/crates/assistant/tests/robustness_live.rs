//! The live run of the question bank (D6 Section 3, plan 50 R2, plan 52 items 12 and 13, NFR-AI-2).
//! The engine under `scripts/ai-eval/run.mjs`: it asks every question of
//! `scripts/ai-eval/questions.tsv` through the app's own question path (`run::ask`, the function
//! behind the `assistant_ask` command) on a seeded database, with the real client, and writes what
//! happened as JSON.
//!
//! IGNORED: `cargo test` never runs it. Only `run.mjs` runs it, with `--ignored`, after the Owner
//! has saved his OpenRouter key in Settings. The key is read from the app's own place for it
//! (Windows Credential Manager, `KeyringStore`) just before each call by the loop; this file never
//! sees it, and no argument or environment variable carries it. What the variables below hold is
//! paths, a list of ids and a number, nothing secret.
//!
//! - `MANDI_EVAL_QUESTIONS`: the path of `questions.tsv`.
//! - `MANDI_EVAL_OUT`: where to write the JSON results (rewritten after every question).
//! - `MANDI_EVAL_ONLY`: optional, a comma list of ids to run (for example `Q001,Q016`).
//! - `MANDI_EVAL_MODEL`: optional model name (the default is the app's default model).
//! - `MANDI_EVAL_THREADS`: optional, questions at once (default 3).
//!
//! Each question gets a fresh database, so the daily limit of 20 never stops the run and no
//! question sees another's answer. A follow-up first asks the question it follows, on the same
//! database and in the same conversation, and its own time and calls are the ones counted. The
//! worlds are the PRD's: `day` (Events 1 to 5, 10-10-26), `late` (the same day, 15-11-26), `e59`
//! (six months of commission, E2E-59), `e42` (E2E-16 with the fruit of E2E-42) and `e34` (a trip
//! purchase, E2E-34); and the two big ones of `robustness_year/`: `year` (a rich year of books,
//! built once through the app's own saves, with its oracle) and `thin` (set up, the opening
//! position and no sale, like the Owner's own data today). Their database is copied for every
//! question.
//!
//! The results hold, besides the answer, what the app wrote in its own question log (`calls_json`,
//! `answer_json`, `finish_reason`, `total_millis`, `model_millis`, `tool_millis`), the tokens of
//! the question itself (`user_tokens`, to measure the fixed part of the prompt from the first call)
//! and the oracle (`oracle`: the true answers of the books, worked out by the app's own reports).
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

use std::sync::Mutex;
use std::time::{Duration, Instant};

use mandi_assistant::ask::{
    AskOutcome, AskReply, AssistantAskInput, AssistantPickInput, PickChoice,
};
use mandi_assistant::client::{CallError, ModelClient, OpenRouterClient, Reply};
use mandi_assistant::run::{self, Deps};
use mandi_assistant::runtime::Runtime;
use mandi_assistant::settings::DEFAULT_MODEL;
use mandi_reports::dashboard::BackupFacts;
use mandi_services::core::fruit::{FruitInput, FruitSource};
use mandi_services::core::purchases::PurchaseInput;
use mandi_services::core::trips::TripInput;
use mandi_services::core::ventures::{MoneySource, VenturePartnerInput};
use mandi_services::{assistant, fruit, purchases, trips};
use serde_json::{Value, json};

/// The E2E-59 months: (month, qty, rate) of one banana sale a month, April to September.
const HISTORY: [(u32, i64, i64); 6] = [
    (4, 180, 1_600),
    (5, 150, 2_100),
    (6, 120, 2_850),
    (7, 225, 1_600),
    (8, 150, 2_600),
    (9, 173, 2_390),
];

struct Question {
    id: String,
    world: String,
    after: String,
    text: String,
    /// The `filters` column: where the party of the question is named (`seller=Rahim`).
    filters: String,
    /// True when the name is spelled badly on purpose.
    misspelled: bool,
}

/// The one party a question is about, from its `filters` (`seller=Rahim`, `buyer=Faisal`). None when
/// it names none or several.
fn party_of(filters: &str) -> Option<String> {
    filters.split(';').find_map(|piece| {
        let (key, value) = piece.split_once('=')?;
        let party = [
            "buyer", "seller", "partner", "item", "garden", "trip", "person",
        ]
        .contains(&key);
        (party && !value.contains(',') && !value.is_empty()).then(|| value.trim().to_owned())
    })
}

fn questions(path: &str, only: &[String]) -> Vec<Question> {
    let text = std::fs::read_to_string(path).expect("the questions file");
    let mut rows = text.lines();
    let head: Vec<&str> = rows.next().expect("a header").split('\t').collect();
    let at = |name: &str| head.iter().position(|h| *h == name).expect(name);
    let (id, world, after, question) = (at("id"), at("world"), at("after"), at("question"));
    let (filters, misspelled) = (at("filters"), at("misspelled"));
    rows.filter(|l| !l.trim().is_empty())
        .map(|l| {
            let cells: Vec<&str> = l.split('\t').collect();
            Question {
                id: cells[id].to_owned(),
                world: cells[world].to_owned(),
                after: cells[after].to_owned(),
                text: cells[question].to_owned(),
                filters: cells.get(filters).copied().unwrap_or("").to_owned(),
                misspelled: cells.get(misspelled).copied() == Some("1"),
            }
        })
        .filter(|q| only.is_empty() || only.contains(&q.id))
        .collect()
}

/// One seeded database for one question. `masters` hold the rich year and the thin books, each
/// built once for the run.
fn build(world_name: &str, masters: Option<&Masters>) -> world::World {
    match world_name {
        "year" => year_copy(
            masters
                .and_then(|m| m.year.as_ref())
                .expect("the rich year is built when a question needs it"),
        ),
        "thin" => year_copy(
            masters
                .and_then(|m| m.thin.as_ref())
                .expect("the thin books are built when a question needs them"),
        ),
        "late" | "day" => {
            let w = worked::worked_day();
            worked::events_1_to_5(&w);
            if world_name == "late" {
                common::set_today(&w.env, 2026, 11, 15);
            }
            w
        }
        "e59" => worked::history_world(&HISTORY),
        "e42" => {
            // E2E-16 (45 of 50 sold), then الف takes 2 کیلا home and the guests have 3 سیب.
            let w = worked::part_sold_world();
            let nazeer = world::add_account(
                &w.env,
                &w.owner,
                mandi_services::core::accounts::AccountType::Seller,
                "حاجی نذیر",
                "Haji Nazeer",
                true,
                "",
            );
            let input = |guests: bool| FruitInput {
                for_guests: guests,
                partner_id: (!guests).then_some(w.partners[0]),
                date: "10-10-2026".into(),
                source: if guests {
                    FruitSource::OtherShop
                } else {
                    FruitSource::Shop
                },
                lot_id: (!guests).then_some(1),
                agent_account_id: guests.then_some(nazeer),
                item_id: guests.then_some(w.apple),
                packing_id: guests.then_some(w.box_),
                qty: if guests { "3" } else { "2" }.into(),
                weight: None,
                rate: if guests { "1,000" } else { "580" }.into(),
                note: None,
                editing_id: None,
            };
            fruit::save(&w.env.ctx, &w.owner, &input(false)).expect("fruit for the home");
            fruit::save(&w.env.ctx, &w.owner, &input(true)).expect("fruit for the guests");
            w
        }
        "e34" => {
            // E2E-34: a trip of the three partners at 40 / 30 / 30 from the shop's cash buys 20
            // boxes at 500 with 400 freight (10,400).
            let w = worked::worked_day();
            let [a, b, c] = w.partners;
            let share = |partner_id, share: &str| VenturePartnerInput {
                partner_id,
                share: share.into(),
                source: MoneySource::Shop,
            };
            let trip = trips::create(
                &w.env.ctx,
                &w.owner,
                &TripInput {
                    date: "10-10-2026".into(),
                    market: Some("Lahore".into()),
                    sent_person: None,
                    note: None,
                    partners: vec![share(a, "40"), share(b, "30"), share(c, "30")],
                },
            )
            .expect("the trip");
            purchases::save(
                &w.env.ctx,
                &w.owner,
                &PurchaseInput {
                    other_qty: None,
                    shop_name: None,
                    sell_date: None,
                    trip_id: Some(trip.id),
                    date: "10-10-2026".into(),
                    supplier: None,
                    item_id: Some(w.banana),
                    packing_id: Some(w.box_),
                    qty: "20".into(),
                    weight: None,
                    rate: "500".into(),
                    freight: Some("400".into()),
                    labour: None,
                    other_cost: None,
                    note: None,
                    parts: Vec::new(),
                    confirm_cash_below_zero: false,
                },
            )
            .expect("the purchase");
            w
        }
        other => panic!("unknown world {other}"),
    }
}

/// One model call as the run saw it.
#[derive(Debug, Clone)]
struct CallRecord {
    millis: u128,
    prompt: i64,
    cached: i64,
    completion: i64,
    reasoning: i64,
    tool_calls: usize,
    finish: String,
    model: Option<String>,
    /// When the call began, in milliseconds from the start of the question (plan 54: the time
    /// before the first call, between calls and after the last one).
    at_ms: u128,
}

/// The real client, with a note of every call: time, tokens, how many tools it asked for.
struct Recorder {
    inner: Box<dyn ModelClient>,
    calls: Mutex<Vec<CallRecord>>,
    /// The start of the question the calls belong to.
    clock: Mutex<Option<Instant>>,
}

impl ModelClient for Recorder {
    fn call(&self, key: &str, body: &Value, timeout: Duration) -> Result<Reply, CallError> {
        let started = Instant::now();
        let at_ms = self
            .clock
            .lock()
            .expect("the clock")
            .map_or(0, |begin| begin.elapsed().as_millis());
        let result = self.inner.call(key, body, timeout);
        let millis = started.elapsed().as_millis();
        if let Ok(reply) = &result {
            self.calls.lock().expect("the notes").push(CallRecord {
                at_ms,
                millis,
                prompt: reply.usage.prompt_tokens,
                cached: reply.usage.cached_tokens,
                completion: reply.usage.completion_tokens,
                reasoning: reply.usage.reasoning_tokens,
                tool_calls: reply.tool_calls.len(),
                finish: reply.finish_reason.clone(),
                model: reply.model.clone(),
            });
        }
        result
    }
}

/// The numbers of the rich year that a copy of its database needs to be a [`world::World`].
#[derive(Clone, Copy)]
struct YearIds {
    rahim: i64,
    karim: i64,
    akram: i64,
    arshad: i64,
    naqdi: i64,
    pukhta_buyer: i64,
    pukhta_seller: i64,
    banana: i64,
    apple: i64,
    mooli: i64,
    weight_fruit: i64,
    box_: i64,
    partners: [i64; 3],
}

/// The rich year, built once for the run (plan 52 item 12): a snapshot of its database that every
/// question copies, so no question sees another's log (the daily limit of 20 never stops the run)
/// and the build is paid once, and the oracle: the true answers from the app's own reports.
struct YearMaster {
    ids: YearIds,
    snapshot: std::path::PathBuf,
    _snapshot_dir: tempfile::TempDir,
    oracle: std::collections::BTreeMap<String, String>,
}

fn year_master(thin: bool) -> YearMaster {
    let started = Instant::now();
    let y = if thin {
        year::seed::build_thin()
    } else {
        year::seed::build()
    };
    let oracle = if thin {
        year::oracle::compute_thin(&y)
    } else {
        year::oracle::compute(&y)
    };
    let dir = tempfile::tempdir().expect("a folder for the snapshot");
    let snapshot = dir.path().join("year.db");
    // VACUUM INTO writes a whole, consistent copy of the database file.
    common::raw(&y.w.dir)
        .execute_batch(&format!("VACUUM INTO '{}'", snapshot.display()))
        .expect("the snapshot of the year");
    eprintln!(
        "world {}: {} بکری, {} oracle facts, built in {} s",
        if thin { "thin" } else { "year" },
        y.bikri_count,
        oracle.facts.len(),
        started.elapsed().as_secs()
    );
    YearMaster {
        ids: YearIds {
            rahim: y.w.rahim,
            karim: y.w.karim,
            akram: y.w.akram,
            arshad: y.w.arshad,
            naqdi: y.w.naqdi,
            pukhta_buyer: y.w.pukhta_buyer,
            pukhta_seller: y.w.pukhta_seller,
            banana: y.w.banana,
            apple: y.w.apple,
            mooli: y.w.mooli,
            weight_fruit: y.w.weight_fruit,
            box_: y.w.box_,
            partners: y.w.partners,
        },
        snapshot,
        _snapshot_dir: dir,
        oracle: oracle.facts,
    }
}

/// The two big worlds of the run, each built once when a question needs it.
struct Masters {
    year: Option<YearMaster>,
    thin: Option<YearMaster>,
}

/// A fresh copy of the rich year's database, opened as the app opens it, Owner signed in, today
/// the last day of the books.
fn year_copy(m: &YearMaster) -> world::World {
    let dir = tempfile::tempdir().expect("a folder for the copy");
    std::fs::create_dir_all(dir.path().join("data")).expect("the data folder");
    std::fs::copy(&m.snapshot, dir.path().join("data").join("mandi.db")).expect("the copy");
    let env = mandi_services::testing::TestEnv::new(dir.path()).expect("open the copy");
    common::set_today(
        &env,
        year::seed::TODAY.0,
        year::seed::TODAY.1,
        year::seed::TODAY.2,
    );
    common::login_owner(&env);
    let owner = mandi_services::auth::require(&env.ctx).expect("the Owner");
    let i = m.ids;
    world::World {
        dir,
        env,
        owner,
        rahim: i.rahim,
        karim: i.karim,
        akram: i.akram,
        arshad: i.arshad,
        naqdi: i.naqdi,
        pukhta_buyer: i.pukhta_buyer,
        pukhta_seller: i.pukhta_seller,
        banana: i.banana,
        apple: i.apple,
        mooli: i.mooli,
        weight_fruit: i.weight_fruit,
        box_: i.box_,
        partners: i.partners,
    }
}

fn try_ask(deps: &Deps<'_>, text: &str, conversation: Option<&str>) -> Result<AskReply, String> {
    run::ask(
        deps,
        &AssistantAskInput {
            text: text.to_owned(),
            conversation_id: conversation.map(str::to_owned),
        },
    )
    .map_err(|e| format!("{e:?}"))
}

/// One question on its own database: the row the scorer reads.
fn run_one(
    q: &Question,
    all: &[Question],
    by_id: &std::collections::HashMap<String, usize>,
    masters: Option<&Masters>,
    model: &str,
    runtime: &Runtime,
    make_client: &dyn Fn() -> Box<dyn ModelClient>,
) -> Value {
    let failed = |why: String| {
        eprintln!("{} {} FAILED: {why}", q.id, q.world);
        json!({"id": q.id, "question": q.text, "world": q.world, "reply": null,
               "error": why, "tool_calls": [], "model_calls": 0,
               "tokens": {"in": 0, "cached": 0, "out": 0, "reasoning": 0},
               "calls": [], "millis": 0})
    };
    let w = build(&q.world, masters);
    if let Err(e) = assistant::set_config(&w.env.ctx, &w.owner, true, model, true) {
        return failed(format!("switch on: {e:?}"));
    }
    let client = Recorder {
        inner: make_client(),
        calls: Mutex::new(Vec::new()),
        clock: Mutex::new(None),
    };
    let emit = |_s| {};
    let deps = Deps {
        ctx: &w.env.ctx,
        actor: &w.owner,
        runtime,
        client: &client,
        emit: &emit,
        backup: BackupFacts {
            cloud_on: false,
            cloud_age_hours: None,
        },
    };
    // A follow-up asks the question it follows first, in the same conversation.
    let mut conversation: Option<String> = None;
    if let Some(base) = by_id.get(&q.after).and_then(|i| all.get(*i)) {
        match try_ask(&deps, &base.text, None) {
            Ok(r) => conversation = Some(r.conversation_id),
            Err(e) => return failed(format!("the question it follows: {e}")),
        }
        client.calls.lock().expect("notes").clear();
    }
    // The clock starts here: the time of a question is the question's, not the world's.
    let started = Instant::now();
    *client.clock.lock().expect("the clock") = Some(started);
    let reply = match try_ask(&deps, &q.text, conversation.as_deref()) {
        Ok(r) => r,
        Err(e) => return failed(e),
    };
    let mut millis = started.elapsed().as_millis();
    let Ok(Some(first_row)) = assistant::get_question(&w.env.ctx, reply.question_id) else {
        return failed("the question log has no row".to_owned());
    };
    // A misspelled name is asked "which one" by design. On the panel one click answers it, so the
    // harness clicks the option that is the party of the question, and scores the answer after
    // the click; both times are counted and the row says it clicked.
    // A name that exists only in another role ends as not found with `found_as` (plan 52 Section
    // 8.5): the question waits, and the panel's "show his purchases" button sends the same pick on
    // that one party. The harness clicks it too. The row keeps the first reply (`first_reply`: the
    // outcome and the roles) apart from the answer after the click.
    let mut rows = vec![first_row];
    let mut reply = reply;
    let mut clicked: Value = Value::Null;
    let mut first_reply: Value = Value::Null;
    let target: Option<(String, String, &str)> = if q.misspelled
        && reply.outcome == AskOutcome::WhichOne
        && let Some(want) = party_of(&q.filters)
        && let Some(option) = reply
            .options
            .iter()
            .find(|o| o.name.name_en.eq_ignore_ascii_case(&want))
    {
        Some((option.key.clone(), want, "which_one"))
    } else if reply.outcome == AskOutcome::NotFound
        && let Some(found) = reply.found_as.as_ref()
    {
        Some((found.key.clone(), found.name.name_en.clone(), "not_found"))
    } else {
        None
    };
    if let Some((key, name, first_outcome)) = target {
        let click_started = Instant::now();
        let done = match run::pick(
            &deps,
            &AssistantPickInput {
                question_id: reply.question_id,
                choice: PickChoice::Match { key: key.clone() },
            },
        ) {
            Ok(done) => done,
            Err(e) => return failed(format!("the click: {e:?}")),
        };
        let click_millis = click_started.elapsed().as_millis();
        let Ok(Some(second)) = assistant::get_question(&w.env.ctx, done.question_id) else {
            return failed("the log has no row for the click".to_owned());
        };
        clicked = json!({"key": key, "name": name, "first_outcome": first_outcome,
                         "first_millis": millis, "click_millis": click_millis});
        if first_outcome == "not_found" {
            first_reply = serde_json::to_value(&reply).expect("the first reply as JSON");
        }
        millis += click_millis;
        rows.push(second);
        reply = done;
    }
    // What the question cost, over its rows (one, or the question and its click).
    let sum =
        |f: &dyn Fn(&mandi_services::db::assistant::QuestionRow) -> Option<i64>| -> Option<i64> {
            rows.iter().map(f).try_fold(0i64, |a, v| v.map(|x| a + x))
        };
    let row = rows.last().cloned().expect("a row of the question");
    let calls_json = {
        let mut all: Vec<Value> = Vec::new();
        for r in &rows {
            if let Some(Ok(Value::Array(items))) =
                r.calls_json.as_deref().map(serde_json::from_str::<Value>)
            {
                all.extend(items);
            }
        }
        Value::Array(all).to_string()
    };
    let tool_calls_last: Value =
        serde_json::from_str(&row.tool_calls_json).unwrap_or(Value::Array(Vec::new()));
    let calls: Vec<Value> = client
        .calls
        .lock()
        .expect("notes")
        .iter()
        .map(|c| {
            json!({"millis": c.millis, "at_ms": c.at_ms, "prompt": c.prompt, "cached": c.cached,
                   "completion": c.completion, "reasoning": c.reasoning,
                   "tool_calls": c.tool_calls, "finish": c.finish, "model": c.model})
        })
        .collect();
    let buttons = open_buttons(&w, &reply);
    eprintln!(
        "{} {} {:?} ({} ms){}",
        q.id,
        q.world,
        reply.outcome,
        millis,
        if clicked.is_null() { "" } else { " [clicked]" }
    );
    json!({
        "id": q.id,
        "question": q.text,
        "world": q.world,
        "reply": serde_json::to_value(&reply).expect("the reply as JSON"),
        // The row of a click carries the totals and the tool list of the whole question (it goes
        // on from the first one); only its times and its calls are its own.
        "tool_calls": tool_calls_last,
        "model_calls": row.model_calls,
        "tokens": {"in": row.tokens_in, "cached": row.tokens_cached,
                   "out": row.tokens_out, "reasoning": row.tokens_reasoning},
        "calls": calls,
        "millis": millis,
        // The log row of this very question (lead's item 0b): what the app itself wrote down.
        "calls_json": calls_json,
        "answer_json": row.answer_json,
        "finish_reason": row.finish_reason,
        "total_millis": sum(&|r| r.total_millis),
        "model_millis": sum(&|r| r.model_millis),
        "tool_millis": sum(&|r| r.tool_millis),
        // The reports and views the answer offers as buttons, opened with the button's own filters
        // through the same backend command, and the figures they hold (plan 55 gate 3).
        "buttons": buttons,
        // Set when the question was asked "which one" and the harness clicked the right option.
        "clicked": clicked,
        // The reply before the click, when the click answered a `found_as` (null otherwise).
        "first_reply": first_reply,
        // The question as the model counts tokens, to take it off the first call's prompt tokens
        // when the fixed part is measured (lead's item 0c).
        "user_tokens": user_tokens(&q.text),
        // This build puts `asked_role` and `found_as` in a not_found reply (plan 52 Section 8.2): a
        // not_found without them is a miss, not a question waiting for the field.
        "asked_role_field": true,
        "followup": !q.after.is_empty(),
    })
}

/// Every figure a JSON value holds as a cell, as text the scorer compares: money and balances as
/// whole rupees (without the sign), counts as they are, percents as `p<hundredths>`.
fn cells_in(value: &Value, out: &mut std::collections::BTreeSet<String>) {
    match value {
        Value::Object(map) => {
            if let Some(Value::String(kind)) = map.get("type") {
                let number = |key: &str| map.get(key).and_then(Value::as_i64);
                match kind.as_str() {
                    "money" | "balance" => {
                        if let Some(paisa) = number("paisa") {
                            out.insert((paisa / 100).abs().to_string());
                        }
                    }
                    "number" => {
                        if let Some(n) = number("value") {
                            out.insert(n.abs().to_string());
                        }
                    }
                    "percent" => {
                        if let Some(h) = number("hundredths") {
                            out.insert(format!("p{h}"));
                        }
                    }
                    "weight" => {
                        if let Some(g) = number("grams") {
                            out.insert(format!("g{g}"));
                        }
                    }
                    _ => {}
                }
            }
            for v in map.values() {
                cells_in(v, out);
            }
        }
        Value::Array(items) => {
            for v in items {
                cells_in(v, out);
            }
        }
        _ => {}
    }
}

/// Plan 55 gate 3: every report or view an answer offers as a button is opened through the same
/// backend command, with the button's own filters, and the figures it holds are written into the
/// row. The scorer then requires the answer's figures to be in them. A party card opens his ledger
/// (R13); the tiles open the dashboard, which is not run here (`skipped`).
fn open_buttons(w: &world::World, reply: &mandi_assistant::ask::AskReply) -> Vec<Value> {
    use mandi_assistant::answer::AnswerSource;
    use mandi_reports::analytics::{self, AnalyticsRunInput};
    use mandi_reports::reports::{self, PartyFilter, ReportFilters, ReportId, ReportRunInput};
    use std::collections::BTreeSet;

    const PAGE: i64 = 1_000;
    const MAX_PAGES: i64 = 10;
    let db = w.env.ctx.db();
    let report_button = |id: ReportId, filters: ReportFilters, label: String| -> Value {
        let mut numbers = BTreeSet::new();
        let mut offset = 0;
        let mut error: Option<String> = None;
        let mut total = 0;
        for _ in 0..MAX_PAGES {
            let page = reports::run(
                db,
                &ReportRunInput {
                    id,
                    filters: filters.clone(),
                    offset,
                    limit: PAGE,
                    sort: None,
                    group_by_month: false,
                },
            );
            match page {
                Ok(page) => {
                    total = page.total_count;
                    cells_in(
                        &serde_json::to_value(&page).expect("a page as JSON"),
                        &mut numbers,
                    );
                    offset += PAGE;
                    if offset >= page.total_count {
                        break;
                    }
                }
                Err(e) => {
                    error = Some(format!("{e:?}"));
                    break;
                }
            }
        }
        json!({"kind": "report", "label": label, "rows": total, "error": error,
               "numbers": numbers.into_iter().collect::<Vec<_>>()})
    };
    let Some(answer) = reply.answer.as_ref() else {
        return Vec::new();
    };
    answer
        .sources
        .iter()
        .map(|source| match source {
            // R31 (a partner's statement) is put together by the services, not by `reports::run`;
            // the app opens it with `statement_request` and `statement_page` (reports_engine.rs).
            AnswerSource::Report { id, filters, .. } if *id == ReportId::R31 => {
                statement_button(w, &filters.to_filters())
            }
            AnswerSource::Report { id, filters, .. } => {
                report_button(*id, filters.to_filters(), id.code().to_owned())
            }
            AnswerSource::PartyCard { party, .. } => report_button(
                ReportId::R13,
                ReportFilters {
                    party: Some(PartyFilter {
                        account_id: party.id,
                        person_key: None,
                    }),
                    ..ReportFilters::default()
                },
                format!("R13 {}", party.name_en),
            ),
            AnswerSource::Analytics { view, filters, .. } => {
                let mut numbers = BTreeSet::new();
                let mut offset = 0;
                let mut error: Option<String> = None;
                let mut total = 0;
                for _ in 0..MAX_PAGES {
                    let page = analytics::run(
                        db,
                        &AnalyticsRunInput {
                            view: *view,
                            filters: filters.to_filters(),
                            offset,
                            limit: PAGE,
                            sort: None,
                        },
                    );
                    match page {
                        Ok(page) => {
                            total = page.total_count;
                            cells_in(
                                &serde_json::to_value(&page).expect("a page as JSON"),
                                &mut numbers,
                            );
                            offset += PAGE;
                            if offset >= page.total_count {
                                break;
                            }
                        }
                        Err(e) => {
                            error = Some(format!("{e:?}"));
                            break;
                        }
                    }
                }
                json!({"kind": "analytics", "label": format!("view {}", view.code()), "rows": total,
                       "error": error, "numbers": numbers.into_iter().collect::<Vec<_>>()})
            }
            AnswerSource::Tiles { .. } => {
                json!({"kind": "tiles", "label": "tiles", "skipped": true})
            }
        })
        .collect()
}

/// The button R31, the statement of a partner, opened the way the app opens it: the request is read
/// from the button's filters (a partner is required), the services put the statement together, and
/// the report engine turns it into a page. Every line is read (the same chunk the print uses).
fn statement_button(w: &world::World, filters: &mandi_reports::reports::ReportFilters) -> Value {
    use mandi_reports::reports::{statement_page, statement_request};
    use mandi_services::core::dates::{BusinessDate, format_ddmmyyyy};
    use mandi_services::core::statement::PartnersStatementInput;
    use std::collections::BTreeSet;

    const CHUNK: u32 = 10_000;
    let typed = |iso: Option<&str>| iso.and_then(BusinessDate::parse_iso).map(format_ddmmyyyy);
    let shown = |label: &str, rows: i64, error: Option<String>, numbers: BTreeSet<String>| {
        json!({"kind": "report", "label": label, "rows": rows, "error": error,
               "numbers": numbers.into_iter().collect::<Vec<_>>()})
    };
    let request = match statement_request(w.env.ctx.db(), filters) {
        Ok(request) => request,
        Err(e) => return shown("R31", 0, Some(format!("{e:?}")), BTreeSet::new()),
    };
    let statement = mandi_services::partners::statement(
        &w.env.ctx,
        &PartnersStatementInput {
            group_by_month: false,
            sort: None,
            partner_id: request.partner_id,
            from: typed(request.from.as_deref()),
            to: typed(request.to.as_deref()),
            offset: 0,
            limit: CHUNK,
        },
    );
    let statement = match statement {
        Ok(statement) => statement,
        Err(e) => return shown("R31", 0, Some(format!("{e:?}")), BTreeSet::new()),
    };
    match statement_page(&request, &statement, 0, i64::from(CHUNK)) {
        Ok(page) => {
            let mut numbers = BTreeSet::new();
            cells_in(
                &serde_json::to_value(&page).expect("a page as JSON"),
                &mut numbers,
            );
            shown("R31", page.total_count, None, numbers)
        }
        Err(e) => shown("R31", 0, Some(format!("{e:?}")), BTreeSet::new()),
    }
}

/// The tokens of a question text (o200k_base, the count the prompt test uses).
fn user_tokens(text: &str) -> usize {
    tiktoken_rs::o200k_base()
        .map(|bpe| bpe.encode_with_special_tokens(text).len())
        .unwrap_or(0)
}

/// Questions asked at once by default (`--threads` of run.mjs).
const DEFAULT_THREADS: usize = 3;
/// How long the harness waits for the internet to come back before it gives a question up.
const NET_WAIT_LIMIT: Duration = Duration::from_secs(600);
/// How many times one question is asked again after the internet came back.
const NET_MAX_WAITS: u32 = 5;

/// True when the question ended because the internet itself was lost: the app said offline, or a
/// connection that was made dropped. Not a timeout (that is the service, and a real result).
fn lost_network(row: &Value) -> bool {
    let reply = &row["reply"];
    match reply["outcome"].as_str() {
        Some("offline") => true,
        Some("error") => matches!(
            reply["failure"]["kind"].as_str(),
            Some("offline" | "dropped")
        ),
        _ => false,
    }
}

/// True when the machine can reach the model's host now. It opens a connection and sends nothing,
/// so no key and no question leave the machine.
fn can_reach_the_host() -> bool {
    use std::net::{TcpStream, ToSocketAddrs};
    "openrouter.ai:443"
        .to_socket_addrs()
        .ok()
        .and_then(|mut addrs| {
            addrs.find(|a| TcpStream::connect_timeout(a, Duration::from_secs(5)).is_ok())
        })
        .is_some()
}

/// Waits, up to `limit`, until the host can be reached again. True when it can.
fn internet_back(limit: Duration) -> bool {
    let started = Instant::now();
    loop {
        if can_reach_the_host() {
            return true;
        }
        if started.elapsed() >= limit {
            return false;
        }
        eprintln!("the internet is gone; waiting (a check every 10 s)");
        std::thread::sleep(Duration::from_secs(10));
    }
}

/// Runs a question; if the internet was lost, waits for it and asks the SAME question again (on a
/// fresh copy of its books), up to `max_waits` times. A row that is still lost has `network_lost`:
/// the scorer never counts it as a result. A row that was asked again has `net_retries`.
fn run_until_online(
    mut run: impl FnMut() -> Value,
    mut wait_for_internet: impl FnMut() -> bool,
    max_waits: u32,
) -> Value {
    let mut row = run();
    let mut waits = 0;
    while lost_network(&row) {
        if waits >= max_waits || !wait_for_internet() {
            row["network_lost"] = json!(true);
            row["net_retries"] = json!(waits);
            return row;
        }
        waits += 1;
        row = run();
        row["net_retries"] = json!(waits);
    }
    row
}

#[test]
#[ignore = "live: calls the real model with the key saved in the app; run it with scripts/ai-eval/run.mjs"]
fn live_100_questions() {
    let questions_path = std::env::var("MANDI_EVAL_QUESTIONS").expect("MANDI_EVAL_QUESTIONS");
    let out_path = std::env::var("MANDI_EVAL_OUT").expect("MANDI_EVAL_OUT");
    let only: Vec<String> = std::env::var("MANDI_EVAL_ONLY")
        .map(|v| v.split(',').map(|s| s.trim().to_owned()).collect())
        .unwrap_or_default();
    let model = std::env::var("MANDI_EVAL_MODEL").unwrap_or_else(|_| DEFAULT_MODEL.to_owned());
    // How many questions run at once (default 3, so the full bank takes about 30 minutes instead
    // of 90). Each question has its own copy of its books and its own conversation, and its time
    // is its own. 1 is the quietest line, for a timing check; the provider may answer a little
    // slower or say it is busy when many run at once.
    let threads: usize = std::env::var("MANDI_EVAL_THREADS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_THREADS)
        .clamp(1, 8);
    // The key is in the app's own store. Is it there? (Never printed, never kept.)
    let key_saved = Runtime::real()
        .keys()
        .get()
        .map(|k| k.is_some())
        .unwrap_or(false);
    if !key_saved {
        std::fs::write(&out_path, json!({"waiting_for_key": true}).to_string())
            .expect("the results file");
        return;
    }
    let list = questions(&questions_path, &only);
    let all = questions(&questions_path, &[]);
    let by_id: std::collections::HashMap<String, usize> = all
        .iter()
        .enumerate()
        .map(|(i, q)| (q.id.clone(), i))
        .collect();
    // The rich year is built once, and only when a question of the run needs it.
    let masters = Masters {
        year: list
            .iter()
            .any(|q| q.world == "year")
            .then(|| year_master(false)),
        thin: list
            .iter()
            .any(|q| q.world == "thin")
            .then(|| year_master(true)),
    };
    let meta = json!({
        "model": model,
        "endpoint": "azure (only: azure, allow_fallbacks: false, zdr: true, data_collection: deny)",
        "questions": list.len(),
        "threads": threads,
        // The find before the first call is the app's own setting (off by default): the run uses
        // the real runtime, so it asks as the app asks.
        "prefind": Runtime::real().prefind(),
    });
    let mut facts = std::collections::BTreeMap::new();
    for m in [masters.year.as_ref(), masters.thin.as_ref()]
        .into_iter()
        .flatten()
    {
        facts.extend(m.oracle.iter().map(|(k, v)| (k.clone(), v.clone())));
    }
    let oracle = json!(facts);
    // The file is rewritten after every question, so a stopped run keeps what it has.
    let flush = |rows: &Vec<Value>| {
        std::fs::write(
            &out_path,
            json!({"meta": meta, "oracle": oracle, "results": rows}).to_string(),
        )
        .expect("the results file");
    };
    let rows = run_rows(
        &list,
        &all,
        &by_id,
        &masters,
        &model,
        threads,
        &Runtime::real,
        &|| Box::new(OpenRouterClient),
        &|| internet_back(NET_WAIT_LIMIT),
        &flush,
    );
    flush(&rows);
}

/// Runs the questions of `list` on `threads` threads, each question on its own copy of its books
/// and in its own conversation, and gives the rows in the order they finished (the scorer finds a
/// row by its id, so the order does not matter). `flush` is called with all rows so far after every
/// question. The time of a question is its own: the clock starts when that question is asked.
#[allow(clippy::too_many_arguments)]
fn run_rows(
    list: &[Question],
    all: &[Question],
    by_id: &std::collections::HashMap<String, usize>,
    masters: &Masters,
    model: &str,
    threads: usize,
    make_runtime: &(dyn Fn() -> Runtime + Sync),
    make_client: &(dyn Fn() -> Box<dyn ModelClient> + Sync),
    wait_for_internet: &(dyn Fn() -> bool + Sync),
    flush: &(dyn Fn(&Vec<Value>) + Sync),
) -> Vec<Value> {
    let results: Mutex<Vec<Value>> = Mutex::new(Vec::new());
    let next = std::sync::atomic::AtomicUsize::new(0);
    std::thread::scope(|scope| {
        for _ in 0..threads {
            scope.spawn(|| {
                let runtime = make_runtime();
                loop {
                    let at = next.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    let Some(q) = list.get(at) else {
                        break;
                    };
                    let row = run_until_online(
                        || run_one(q, all, by_id, Some(masters), model, &runtime, make_client),
                        wait_for_internet,
                        NET_MAX_WAITS,
                    );
                    let mut rows = results.lock().expect("the results");
                    rows.push(row);
                    flush(&rows);
                }
            });
        }
    });
    results.into_inner().expect("the results")
}

#[test]
fn the_five_worlds_of_the_list_are_the_prds() {
    // Not live: no model, no key. It proves that the databases the live run asks about hold the
    // PRD numbers its expected figures come from, so a miss is never the seed's fault.
    use mandi_services::{cash, ledger};
    let balance = |w: &world::World, id: i64| {
        let b = ledger::account_balance(&w.env.ctx, id).expect("balance");
        (b.amount_paisa.rupees(), format!("{:?}", b.side))
    };
    let cash_now = |w: &world::World| {
        w.env
            .ctx
            .db()
            .read(cash::cash_in_hand)
            .expect("cash")
            .rupees()
    };
    let day = build("day", None);
    assert_eq!(balance(&day, day.akram), (2_880, "Dr".to_owned()), "E2E-47");
    assert_eq!(balance(&day, day.arshad), (18_684, "Dr".to_owned()));
    assert_eq!(cash_now(&day), 58_484, "E2E-8");
    let late = build("late", None);
    assert_eq!(
        assistant::pakistan_day(&late.env.ctx),
        "2026-11-15",
        "E2E-63"
    );
    assert_eq!(balance(&late, late.akram), (2_880, "Dr".to_owned()));
    let e34 = build("e34", None);
    assert_eq!(cash_now(&e34), 39_600, "E2E-34: 50,000 less 10,400");
    let e42 = build("e42", None);
    assert_eq!(cash_now(&e42), 45_000, "E2E-42: cash does not move");
    let e59 = build("e59", None);
    assert_eq!(assistant::pakistan_day(&e59.env.ctx), "2026-10-10");
}

#[test]
fn the_year_is_balanced_whole_and_the_same_on_every_run() {
    // Not live: no model, no key. The books of the rich year are built only through the app's own
    // saves, so the trial balance is equal, and a second build is the same books (a fixed seed).
    use mandi_reports::reports::{
        Cell, ReportFilters, ReportId, ReportRunInput, run as run_report,
    };
    let started = Instant::now();
    let master = year_master(false);
    let y = year::seed::build();
    let second = year::oracle::compute(&y);
    assert_eq!(
        master.oracle, second.facts,
        "the same seed gives the same books and the same answers"
    );
    let trial = run_report(
        y.w.env.ctx.db(),
        &ReportRunInput {
            id: ReportId::R18,
            filters: ReportFilters::default(),
            offset: 0,
            limit: 100,
            sort: None,
            group_by_month: false,
        },
    )
    .expect("the trial balance");
    let money = |cell: &Cell| match cell {
        Cell::Money { paisa } | Cell::Balance { paisa, .. } => paisa.rupees(),
        _ => 0,
    };
    let dr = trial
        .columns
        .iter()
        .position(|c| c.key == "debit")
        .expect("a debit column");
    let cr = trial
        .columns
        .iter()
        .position(|c| c.key == "credit")
        .expect("a credit column");
    let (total_dr, total_cr) = (money(&trial.totals[dr]), money(&trial.totals[cr]));
    assert!(total_dr > 0, "the books are not empty");
    assert_eq!(total_dr, total_cr, "the trial balance of the year is equal");
    // The size the plan asks for.
    assert!(y.bikri_count >= 300, "{} بکری", y.bikri_count);
    assert!(master.oracle.contains_key("a.year.banana.total_comm"));
    // A copy of the snapshot opens like the app opens its data, and shows the very same cash.
    let copy = year_copy(&master);
    let cash = mandi_services::cash::cash_in_hand;
    let cash_now = copy.env.ctx.db().read(cash).expect("cash").rupees();
    assert_eq!(cash_now.to_string(), master.oracle["tiles.cash"]);
    eprintln!("the year proof took {} s", started.elapsed().as_secs());
}

/// The keys of every world's oracle, one a line: `scripts/ai-eval/oracle-keys.txt`.
fn manifest_text(
    year: &std::collections::BTreeMap<String, String>,
    thin: &std::collections::BTreeMap<String, String>,
) -> String {
    year.keys()
        .chain(thin.keys())
        .map(|k| format!("{k}\n"))
        .collect()
}

#[test]
fn the_oracle_keys_are_the_ones_the_question_bank_may_use() {
    // Not live. `scripts/ai-eval/run.mjs --check` checks every `@key` of the question bank against
    // the file `oracle-keys.txt`; this proves the file is the keys the oracle really gives. To
    // write it again: MANDI_ORACLE_MANIFEST=<path of the file> cargo test -p mandi-assistant
    // --test robustness_live dump_the_oracle -- --nocapture
    let year = year_master(false).oracle;
    let thin = year_master(true).oracle;
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("..")
        .join("scripts")
        .join("ai-eval")
        .join("oracle-keys.txt");
    let on_disk = std::fs::read_to_string(&path).expect("scripts/ai-eval/oracle-keys.txt");
    assert!(
        on_disk == manifest_text(&year, &thin),
        "oracle-keys.txt is out of date; write it again with MANDI_ORACLE_MANIFEST"
    );
}

#[test]
fn the_thin_books_have_a_cash_balance_and_nothing_else() {
    // Not live. The thin books are what the Owner has today: set up, the opening position, no sale.
    let thin = year_master(true);
    assert_eq!(thin.oracle["thin.sales_today"], "0");
    assert_eq!(thin.oracle["thin.count_today"], "0");
    assert_eq!(thin.oracle["thin.buyers_dues"], "0");
    assert_eq!(thin.oracle["thin.sellers_payable"], "0");
    assert_eq!(thin.oracle["thin.stock_qty"], "0");
    assert_eq!(thin.oracle["thin.cash"], "2500000");
    let copy = year_copy(&thin);
    let cash = copy
        .env
        .ctx
        .db()
        .read(mandi_services::cash::cash_in_hand)
        .expect("cash")
        .rupees();
    assert_eq!(cash, 2_500_000);
}

#[test]
fn dump_the_oracle_for_reading() {
    // Not live. With MANDI_ORACLE_DUMP=<file> it writes every fact (key, tab, value); with
    // MANDI_ORACLE_MANIFEST=<file> it writes the keys only (scripts/ai-eval/oracle-keys.txt).
    let y = year::seed::build();
    let o = year::oracle::compute(&y);
    if let Ok(path) = std::env::var("MANDI_ORACLE_DUMP") {
        let text: String = o.facts.iter().map(|(k, v)| format!("{k}\t{v}\n")).collect();
        std::fs::write(path, text).expect("the dump");
    }
    if let Ok(path) = std::env::var("MANDI_ORACLE_MANIFEST") {
        std::fs::write(path, manifest_text(&o.facts, &year_master(true).oracle))
            .expect("the manifest");
    }
    eprintln!("{} oracle facts", o.facts.len());
}

#[test]
fn umars_quick_chips_give_the_oracle_figures_with_no_model_call() {
    // Plan 55 gate 4: the quick chips Umar uses (total_dues, today_sales, party_ledger,
    // sellers_payable) need no internet and no model. On the rich year each gives the figure the
    // oracle holds, and no request is sent.
    use mandi_assistant::ask::{AssistantQuickInput, QuickChip};
    use mandi_assistant::client::RecordedClient;
    use mandi_assistant::keys::MemoryKeyStore;
    use std::sync::Arc;

    let y = year::seed::build();
    let oracle = year::oracle::compute(&y);
    let runtime = Runtime::new(Arc::new(MemoryKeyStore::default()));
    let client = RecordedClient::new(Vec::new());
    let emit = |_s| {};
    let deps = Deps {
        ctx: &y.w.env.ctx,
        actor: &y.w.owner,
        runtime: &runtime,
        client: &client,
        emit: &emit,
        backup: BackupFacts {
            cloud_on: false,
            cloud_age_hours: None,
        },
    };
    // Every whole-rupee number the answer shows, as text.
    let shown = |reply: &mandi_assistant::ask::AskReply| -> Vec<i64> {
        let json = serde_json::to_value(reply).expect("the reply as JSON");
        json["answer"]["parts"]
            .as_array()
            .into_iter()
            .flatten()
            .chain(json["answer"]["tables"].as_array().into_iter().flatten())
            .filter_map(|part| part["cell"]["paisa"].as_i64())
            .map(|paisa| paisa / 100)
            .collect()
    };
    let fact = |key: &str| -> i64 {
        oracle
            .facts
            .get(key)
            .unwrap_or_else(|| panic!("no oracle fact {key}"))
            .split_whitespace()
            .next()
            .and_then(|n| n.parse().ok())
            .unwrap_or_else(|| panic!("{key} is not a number"))
    };
    let faisal = y
        .buyers
        .iter()
        .find(|b| b.key == "faisal")
        .expect("Faisal is a buyer of the year");
    for (chip, party, key) in [
        (QuickChip::TotalDues, None, "tiles.total_dues"),
        (QuickChip::TodaySales, None, "tiles.sales_today"),
        (QuickChip::SellersPayable, None, "tiles.sellers_payable"),
        (
            QuickChip::PartyLedger,
            Some(faisal.id),
            "buyer.faisal.balance",
        ),
    ] {
        let reply = mandi_assistant::quick::quick(
            &deps,
            &AssistantQuickInput {
                chip,
                party_id: party,
                conversation_id: None,
            },
        )
        .unwrap_or_else(|e| panic!("{chip:?}: {e:?}"));
        let want = fact(key);
        assert!(
            shown(&reply).contains(&want),
            "{chip:?}: the oracle says {want} ({key}); the chip shows {:?}",
            shown(&reply)
        );
        // Plan 55 gate 3: the button of the answer opens a place that holds the figure shown.
        let buttons = open_buttons(&y.w, &reply);
        for button in &buttons {
            assert!(
                button["error"].is_null(),
                "{chip:?}: a button fails: {button}"
            );
        }
        let held: std::collections::BTreeSet<String> = buttons
            .iter()
            .flat_map(|b| b["numbers"].as_array().into_iter().flatten())
            .filter_map(|n| n.as_str().map(str::to_owned))
            .collect();
        let tiles_only = buttons.iter().all(|b| b["skipped"] == true);
        assert!(
            tiles_only || held.contains(&want.abs().to_string()),
            "{chip:?}: the button does not hold {want}: {buttons:?}"
        );
    }
    assert!(client.requests().is_empty(), "a chip never calls the model");
}

#[test]
fn the_harness_writes_the_row_the_scorer_reads_for_a_question_and_its_follow_up() {
    // Not live: a scripted model, a key in memory. It proves the harness (plan 52 items 0b and 0c):
    // the rows carry the log fields and the user's tokens, a follow-up runs on the same copy in the
    // same conversation and counts its own calls and time only, and the thin books are served.
    use mandi_assistant::client::RecordedClient;
    use mandi_assistant::keys::MemoryKeyStore;
    use std::sync::Arc;

    let usage = json!({"prompt_tokens": 9_800, "completion_tokens": 40,
        "prompt_tokens_details": {"cached_tokens": 9_000},
        "completion_tokens_details": {"reasoning_tokens": 0}});
    let tiles_call = json!({"model": "openai/gpt-6-luna", "usage": usage,
        "choices": [{"finish_reason": "tool_calls", "message": {"role": "assistant",
            "content": null, "tool_calls": [{"id": "call_0", "type": "function",
                "function": {"name": "tiles", "arguments": "{}"}}]}}]});
    let answer = json!({"model": "openai/gpt-6-luna", "usage": usage,
        "choices": [{"finish_reason": "stop", "message": {"role": "assistant", "content": json!({
            "kind": "answer", "text": "جی ہاں",
            "show": [], "next": [], "choose": null, "entry": null, "nearest": null
        }).to_string()}}]});
    let script = || -> Box<dyn ModelClient> {
        Box::new(RecordedClient::new(vec![
            tiles_call.clone(),
            answer.clone(),
            tiles_call.clone(),
            answer.clone(),
        ]))
    };
    let runtime = Runtime::new(Arc::new(MemoryKeyStore::default()));
    // This test is about the harness, not the planner: the scripted model must be asked in the
    // order of the script (a tool call, then the answer), so the app's own first read (plan 55,
    // read first) and its local answers (no model call) are off here. The live run keeps both on.
    runtime.set_local(false);
    runtime.set_read_first(false);
    runtime
        .keys()
        .set("sk-or-v1-0000000000000000000000000000000000000000000000000000")
        .expect("a key in memory");
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("..")
        .join("scripts")
        .join("ai-eval")
        .join("questions.tsv");
    let all = questions(path.to_str().expect("a path"), &[]);
    let by_id: std::collections::HashMap<String, usize> = all
        .iter()
        .enumerate()
        .map(|(i, q)| (q.id.clone(), i))
        .collect();
    let masters = Masters {
        year: Some(year_master(false)),
        thin: Some(year_master(true)),
    };
    let follow = all
        .iter()
        .find(|q| q.world == "year" && !q.after.is_empty())
        .expect("a follow-up on the year");
    let row = run_one(
        follow,
        &all,
        &by_id,
        Some(&masters),
        "openai/gpt-6-luna",
        &runtime,
        &script,
    );
    assert_eq!(row["followup"], true);
    assert_eq!(row["reply"]["outcome"], "answered");
    // Each call says when it began (plan 54: the time before the first call, between calls and after
    // the last one), counted from the start of the question.
    let starts: Vec<u64> = row["calls"]
        .as_array()
        .expect("the calls")
        .iter()
        .map(|c| c["at_ms"].as_u64().expect("at_ms"))
        .collect();
    assert_eq!(starts.len(), 2);
    assert!(starts[0] <= starts[1], "the calls are in order: {starts:?}");
    // Its own two model calls, not the base question's four.
    assert_eq!(row["calls"].as_array().map(Vec::len), Some(2), "{row}");
    assert_eq!(row["model_calls"], 2);
    for field in [
        "calls_json",
        "answer_json",
        "finish_reason",
        "total_millis",
        "model_millis",
        "tool_millis",
    ] {
        assert!(!row[field].is_null(), "the row has no {field}: {row}");
    }
    assert!(row["user_tokens"].as_u64().unwrap_or(0) > 0);
    // A misspelled name is asked "which one"; the harness clicks the option that is the party of
    // the question, counts both times and says it clicked.
    let which_one = json!({"model": "openai/gpt-6-luna", "usage": usage,
        "choices": [{"finish_reason": "stop", "message": {"role": "assistant", "content": json!({
            "kind": "which_one", "text": "", "show": [], "next": [],
            "choose": "r1", "entry": null, "nearest": null
        }).to_string()}}]});
    let find_a = json!({"model": "openai/gpt-6-luna", "usage": usage,
        "choices": [{"finish_reason": "tool_calls", "message": {"role": "assistant",
            "content": null, "tool_calls": [{"id": "call_0", "type": "function",
                "function": {"name": "find", "arguments": json!({"text": "a", "role": "buyer"}).to_string()}}]}}]});
    let click_script = || -> Box<dyn ModelClient> {
        Box::new(RecordedClient::new(vec![
            find_a.clone(),
            which_one.clone(),
            answer.clone(),
        ]))
    };
    let misspelled = Question {
        id: "T1".into(),
        world: "day".into(),
        after: String::new(),
        text: "a ka khata dikhao".into(),
        filters: "buyer=Akram Toll Tax".into(),
        misspelled: true,
    };
    let row = run_one(
        &misspelled,
        &all,
        &by_id,
        Some(&masters),
        "openai/gpt-6-luna",
        &runtime,
        &click_script,
    );
    assert_eq!(row["clicked"]["name"], "Akram Toll Tax", "{row}");
    assert_eq!(row["clicked"]["first_outcome"], "which_one");
    assert_eq!(
        row["reply"]["outcome"], "answered",
        "the answer after the click: {row}"
    );
    assert_eq!(
        row["model_calls"], 3,
        "the question's two calls and the click's one"
    );
    let both = row["clicked"]["first_millis"].as_u64().unwrap_or(0)
        + row["clicked"]["click_millis"].as_u64().unwrap_or(0);
    assert_eq!(row["millis"].as_u64(), Some(both), "both times are counted");
    // The same question with a correct spelling is not clicked for.
    let plain = Question {
        misspelled: false,
        id: "T2".into(),
        world: "day".into(),
        after: String::new(),
        text: "a ka khata dikhao".into(),
        filters: "buyer=Akram Toll Tax".into(),
    };
    let row = run_one(
        &plain,
        &all,
        &by_id,
        Some(&masters),
        "openai/gpt-6-luna",
        &runtime,
        &click_script,
    );
    assert!(row["clicked"].is_null());
    assert!(
        row["first_reply"].is_null(),
        "nothing was clicked: no first reply"
    );
    assert_eq!(row["reply"]["outcome"], "which_one");

    // A name that exists only in another role ends as not found with `found_as` (plan 52 Section
    // 8.5). The question waits; the harness clicks his name as the panel's button does, keeps the
    // reply before the click as `first_reply`, and the row's reply is the answer after the click.
    let find_seller = json!({"model": "openai/gpt-6-luna", "usage": usage,
        "choices": [{"finish_reason": "tool_calls", "message": {"role": "assistant",
            "content": null, "tool_calls": [{"id": "call_0", "type": "function",
                "function": {"name": "find", "arguments": json!({"text": "faisal", "role": "seller"}).to_string()}}]}}]});
    let other_role_script = || -> Box<dyn ModelClient> {
        Box::new(RecordedClient::new(vec![
            find_seller.clone(),
            answer.clone(),
        ]))
    };
    let wrong_role = Question {
        id: "T3".into(),
        world: "year".into(),
        after: String::new(),
        text: "Faisal ne aaj kya aur kitna becha".into(),
        filters: "asked_role=seller;found_as=buyer".into(),
        misspelled: false,
    };
    let row = run_one(
        &wrong_role,
        &all,
        &by_id,
        Some(&masters),
        "openai/gpt-6-luna",
        &runtime,
        &other_role_script,
    );
    assert_eq!(row["first_reply"]["outcome"], "not_found", "{row}");
    assert_eq!(row["first_reply"]["asked_role"], "seller");
    assert_eq!(row["first_reply"]["found_as"]["role"], "buyer");
    assert_eq!(row["first_reply"]["found_as"]["name"]["name_en"], "Faisal");
    assert_eq!(row["clicked"]["first_outcome"], "not_found");
    assert_eq!(row["clicked"]["name"], "Faisal");
    assert_eq!(
        row["reply"]["outcome"], "answered",
        "after the click: {row}"
    );
    assert_eq!(row["asked_role_field"], true);
    assert_eq!(
        row["model_calls"], 2,
        "the find before the stop (the app ended it) and the one answer after the click: {row}"
    );
    // Plan 55 gate 3: the button of an answer is opened with its own filters, and the row carries
    // what it holds. A report read with no filter (R20, the dues) opens as R20, and the total the
    // answer shows is in it.
    let r20_call = json!({"model": "openai/gpt-6-luna", "usage": usage,
        "choices": [{"finish_reason": "tool_calls", "message": {"role": "assistant",
            "content": null, "tool_calls": [{"id": "call_0", "type": "function",
                "function": {"name": "report", "arguments": json!({
                    "id": "R20", "adda": null, "as_of": null, "bikri": null, "city": null,
                    "collector": null, "from": null, "garden": null, "group_by": null,
                    "item": null, "partner": null, "party": null, "rows": null, "sort": null,
                    "to": null, "trip": null, "where": null}).to_string()}}]}}]});
    let total_dues = json!({"model": "openai/gpt-6-luna", "usage": usage,
        "choices": [{"finish_reason": "stop", "message": {"role": "assistant", "content": json!({
            "kind": "answer", "text": "کل {{r1.totals.balance}} ہے",
            "show": [], "next": [], "choose": null, "entry": null, "nearest": null
        }).to_string()}}]});
    let button_script = || -> Box<dyn ModelClient> {
        Box::new(RecordedClient::new(vec![
            r20_call.clone(),
            total_dues.clone(),
        ]))
    };
    let dues = Question {
        id: "T4".into(),
        world: "year".into(),
        after: String::new(),
        text: "gahakon ke kul bakaya".into(),
        filters: String::new(),
        misspelled: false,
    };
    let row = run_one(
        &dues,
        &all,
        &by_id,
        Some(&masters),
        "openai/gpt-6-luna",
        &runtime,
        &button_script,
    );
    assert_eq!(row["reply"]["outcome"], "answered", "{row}");
    let buttons = row["buttons"].as_array().expect("buttons");
    assert_eq!(buttons.len(), 1, "{row}");
    assert_eq!(buttons[0]["kind"], "report");
    assert_eq!(buttons[0]["label"], "R20");
    assert!(buttons[0]["error"].is_null(), "{buttons:?}");
    let dues_total = masters
        .year
        .as_ref()
        .and_then(|m| m.oracle.get("tiles.buyers_dues"))
        .and_then(|v| v.split_whitespace().next())
        .expect("the buyers' dues in the oracle")
        .to_owned();
    assert!(
        buttons[0]["numbers"]
            .as_array()
            .expect("numbers")
            .iter()
            .any(|n| n == &json!(dues_total)),
        "R20 holds the total {dues_total}: {buttons:?}"
    );

    // A question on the thin books that asks for a figure (it reads the tiles).
    let thin = all
        .iter()
        .find(|q| q.world == "thin" && q.text.contains("bikri kitni"))
        .expect("a thin question that asks a figure");
    let row = run_one(
        thin,
        &all,
        &by_id,
        Some(&masters),
        "openai/gpt-6-luna",
        &runtime,
        &script,
    );
    assert_eq!(row["reply"]["outcome"], "answered", "{row}");
    assert_eq!(row["followup"], false);
    // A general how-to question on the thin books goes out with the tools off: one call, an answer
    // of kind general_advice, which the app labels as general advice and which reads nothing.
    let general_reply = json!({"model": "openai/gpt-6-luna", "usage": usage,
        "choices": [{"finish_reason": "stop", "message": {"role": "assistant", "content": json!({
            "kind": "general_advice",
            "text": "ریٹ مناسب اور مستقل رکھیں۔\n• پرانے بقایا کی وصولی ہر ہفتے کریں۔\n• ہر بکری اسی دن درج کریں۔",
            "show": [], "next": [], "choose": null, "entry": null, "nearest": null
        }).to_string()}}]});
    let general_script =
        || -> Box<dyn ModelClient> { Box::new(RecordedClient::new(vec![general_reply.clone()])) };
    let general = all
        .iter()
        .find(|q| q.world == "thin" && q.filters.contains("general=1"))
        .expect("a general advice question");
    let row = run_one(
        general,
        &all,
        &by_id,
        Some(&masters),
        "openai/gpt-6-luna",
        &runtime,
        &general_script,
    );
    assert_eq!(row["reply"]["outcome"], "advice", "{row}");
    assert_eq!(row["reply"]["answer"]["general_advice"], true, "{row}");
    assert_eq!(row["model_calls"], 1, "one call");
    assert_eq!(
        row["tool_calls"].as_array().map(Vec::len),
        Some(0),
        "it read nothing"
    );
}

#[test]
fn a_question_lost_to_the_internet_is_asked_again_and_never_counted_as_a_result() {
    // Not live. The harness waits for the connection and asks the same question again; a question
    // that stays lost is marked and the scorer leaves it out.
    let offline = || json!({"reply": {"outcome": "offline", "failure": {"kind": "offline"}}});
    let dropped = || json!({"reply": {"outcome": "error", "failure": {"kind": "dropped"}}});
    let timeout = || json!({"reply": {"outcome": "error", "failure": {"kind": "timeout"}}});
    let answered = || json!({"reply": {"outcome": "answered", "failure": null}});
    assert!(lost_network(&offline()) && lost_network(&dropped()));
    assert!(
        !lost_network(&timeout()),
        "a timeout is the service, a real result"
    );
    assert!(!lost_network(&answered()));
    // Offline twice, then the answer: asked three times, waited twice, the answer is the row.
    let mut calls = 0;
    let mut waits = 0;
    let row = run_until_online(
        || {
            calls += 1;
            if calls <= 2 { offline() } else { answered() }
        },
        || {
            waits += 1;
            true
        },
        5,
    );
    assert_eq!(row["reply"]["outcome"], "answered");
    assert_eq!(row["net_retries"], 2);
    assert_eq!((calls, waits), (3, 2));
    // The internet does not come back: one question, no more asks, marked lost.
    let mut calls = 0;
    let row = run_until_online(
        || {
            calls += 1;
            dropped()
        },
        || false,
        5,
    );
    assert_eq!(row["network_lost"], true);
    assert_eq!(calls, 1);
    // It keeps going offline: stops after the most asks and is marked lost.
    let mut calls = 0;
    let row = run_until_online(
        || {
            calls += 1;
            offline()
        },
        || true,
        3,
    );
    assert_eq!(row["network_lost"], true);
    assert_eq!(calls, 4, "the first ask and three more");
    // A timeout is not asked again by the harness (the app has its own retry).
    let mut calls = 0;
    let row = run_until_online(
        || {
            calls += 1;
            timeout()
        },
        || true,
        5,
    );
    assert!(row.get("network_lost").is_none());
    assert_eq!(calls, 1);
}

/// Takes out of a row what two runs of the same question may differ in: the times, and the ids the
/// clock or a random source makes. Texts that hold JSON (`calls_json`, `answer_json`) are opened,
/// scrubbed and written back.
fn scrub(value: &mut Value) {
    match value {
        Value::Object(map) => {
            map.retain(|k, _| {
                !(k.ends_with("millis")
                    || k == "ms"
                    || k.ends_with("_ms")
                    || k == "conversation_id")
            });
            map.values_mut().for_each(scrub);
        }
        Value::Array(items) => items.iter_mut().for_each(scrub),
        Value::String(text) if text.starts_with(['{', '[']) => {
            if let Ok(mut inner) = serde_json::from_str::<Value>(text) {
                scrub(&mut inner);
                *text = inner.to_string();
            }
        }
        _ => {}
    }
}

#[test]
fn a_scripted_run_gives_the_same_rows_on_one_thread_and_on_three() {
    // Not live: no key, no network, no model. The model is a script (a tool call for the tiles,
    // then an answer), the same for every question. It proves the threaded loop of the live run:
    // each question has its own books and its own conversation, no row is lost or doubled, and
    // the rows (after the times are taken out) are the same with 1 thread as with 3 or 4.
    use mandi_assistant::client::RecordedClient;
    use mandi_assistant::keys::MemoryKeyStore;
    use std::sync::Arc;

    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("..")
        .join("scripts")
        .join("ai-eval")
        .join("questions.tsv");
    let path = path.to_str().expect("a path").to_owned();
    let all = questions(&path, &[]);
    let by_id: std::collections::HashMap<String, usize> = all
        .iter()
        .enumerate()
        .map(|(i, q)| (q.id.clone(), i))
        .collect();
    // Twelve questions of the small worlds, follow-ups among them.
    let mut ids: Vec<String> = Vec::new();
    for world in ["day", "late", "e59", "e42", "e34"] {
        ids.extend(
            all.iter()
                .filter(|q| q.world == world)
                .take(2)
                .map(|q| q.id.clone()),
        );
    }
    ids.extend(
        all.iter()
            .filter(|q| {
                !q.after.is_empty()
                    && ["day", "late", "e59", "e42", "e34"].contains(&q.world.as_str())
            })
            .take(2)
            .map(|q| q.id.clone()),
    );
    ids.sort();
    ids.dedup();
    let list = questions(&path, &ids);
    assert!(list.len() >= 12, "{} questions", list.len());

    let usage = json!({
        "prompt_tokens": 7_200, "completion_tokens": 60,
        "prompt_tokens_details": {"cached_tokens": 7_000},
        "completion_tokens_details": {"reasoning_tokens": 20}
    });
    let tool = json!({
        "model": "openai/gpt-6-luna",
        "choices": [{"finish_reason": "tool_calls", "message": {
            "role": "assistant", "content": null,
            "tool_calls": [{"id": "call_0", "type": "function",
                            "function": {"name": "tiles", "arguments": "{}"}}]}}],
        "usage": usage,
    });
    let done = json!({
        "model": "openai/gpt-6-luna",
        "choices": [{"finish_reason": "stop", "message": {"role": "assistant", "content": json!({
            "kind": "answer", "text": "{{r1.today_sales.amount}}", "show": [], "next": [],
            "choose": null, "entry": null, "nearest": null
        }).to_string()}}],
        "usage": usage,
    });
    // Enough replies for a question and the question it follows.
    let script = vec![tool.clone(), done.clone(), tool, done];
    let masters = Masters {
        year: None,
        thin: None,
    };
    let run = |threads: usize| -> (Vec<Value>, Vec<Value>, Duration) {
        let started = Instant::now();
        let mut rows = run_rows(
            &list,
            &all,
            &by_id,
            &masters,
            DEFAULT_MODEL,
            threads,
            &|| {
                Runtime::new(Arc::new(MemoryKeyStore::with_key(
                    "sk-or-v1-test0123456789",
                )))
            },
            &|| Box::new(RecordedClient::new(script.clone())),
            &|| false,
            &|_| {},
        );
        let took = started.elapsed();
        rows.sort_by(|a, b| a["id"].as_str().cmp(&b["id"].as_str()));
        let mut scrubbed = rows.clone();
        scrubbed.iter_mut().for_each(scrub);
        (scrubbed, rows, took)
    };
    let (one, _, took_one) = run(1);
    let (three, three_raw, took_three) = run(3);
    let (four, _, _) = run(4);
    eprintln!(
        "{} questions: 1 thread {took_one:?}, 3 threads {took_three:?}",
        list.len()
    );
    assert_eq!(one.len(), list.len(), "one row for every question");
    assert_eq!(
        three.len(),
        list.len(),
        "no row lost or doubled with 3 threads"
    );
    assert_eq!(four.len(), list.len());
    let answered = one
        .iter()
        .filter(|r| r["reply"]["outcome"] == "answered")
        .count();
    assert!(
        answered * 2 >= one.len(),
        "the script really ran: {answered} of {} answered",
        one.len()
    );
    for (a, b) in one.iter().zip(&three) {
        assert_eq!(a, b, "{}: 3 threads gave another row than 1", a["id"]);
    }
    for (a, b) in one.iter().zip(&four) {
        assert_eq!(a, b, "{}: 4 threads gave another row than 1", a["id"]);
    }
    // The time of a question is the question's own, never the whole run's.
    for row in &three_raw {
        let millis = row["millis"].as_u64().unwrap_or(u64::MAX);
        assert!(
            u128::from(millis) <= took_three.as_millis() + 1_000,
            "{} took {millis} ms but the whole run took {took_three:?}",
            row["id"]
        );
    }
}

/// The rows of a query as JSON objects (column name to value). The queries are written here, never
/// built from input.
fn rows_as_json(conn: &mandi_services::db::rusqlite::Connection, sql: &str) -> Vec<Value> {
    use mandi_services::db::rusqlite::types::Value as Sql;
    let mut statement = conn.prepare(sql).expect("the dump query");
    let names: Vec<String> = statement
        .column_names()
        .iter()
        .map(|n| (*n).to_owned())
        .collect();
    statement
        .query_map([], |row| {
            let mut object = serde_json::Map::new();
            for (at, name) in names.iter().enumerate() {
                let cell: Sql = row.get(at)?;
                object.insert(
                    name.clone(),
                    match cell {
                        Sql::Null | Sql::Blob(_) => Value::Null,
                        Sql::Integer(i) => json!(i),
                        Sql::Real(f) => json!(f),
                        Sql::Text(t) => json!(t),
                    },
                );
            }
            Ok(Value::Object(object))
        })
        .expect("the dump rows")
        .map(|r| r.expect("a dump row"))
        .collect()
}

/// Who and what is in the books of one database: the names a question may use.
fn people_and_things(path: &std::path::Path) -> Value {
    use mandi_services::db::rusqlite::{Connection, OpenFlags};
    let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .expect("open the books to read the names");
    json!({
        "parties": rows_as_json(&conn,
            "SELECT a.id, a.code, a.party_type AS role, k.code AS control, a.name_ur, a.name_en, \
                    a.tag, c.name_en AS city_en, a.is_active \
             FROM accounts a JOIN account_controls k ON k.id = a.control_id \
             LEFT JOIN cities c ON c.id = a.city_id \
             WHERE a.is_system = 0 ORDER BY a.code"),
        "items": rows_as_json(&conn,
            "SELECT id, code, name_ur, name_en, item_group FROM items ORDER BY id"),
        "partners": rows_as_json(&conn,
            "SELECT p.id, p.name_ur, p.name_en, p.is_active, \
                    (SELECT s.share_bps FROM partner_shares s WHERE s.partner_id = p.id \
                     ORDER BY s.effective_from DESC, s.id DESC LIMIT 1) AS share_bps \
             FROM partners p ORDER BY p.id"),
        "gardens": rows_as_json(&conn,
            "SELECT v.id, v.number, v.entry_date, v.name_ur, v.name_en, v.status, v.closed_date, \
                    a.code AS account_code \
             FROM ventures v LEFT JOIN accounts a ON a.id = v.account_id \
             WHERE v.kind = 'garden' ORDER BY v.number"),
        "trips": rows_as_json(&conn,
            "SELECT v.id, v.number, v.entry_date AS day, v.name_ur, v.name_en, v.status, \
                    v.closed_date, v.market \
             FROM ventures v WHERE v.kind = 'trip' ORDER BY v.number"),
        "trip_purchases": rows_as_json(&conn,
            "SELECT v.number AS trip_number, p.number AS purchase_number, p.entry_date AS day, \
                    i.name_en AS item_en, p.qty \
             FROM purchases p JOIN ventures v ON v.id = p.venture_id \
             JOIN items i ON i.id = p.item_id ORDER BY v.number, p.number"),
    })
}

#[test]
fn the_button_r31_is_opened_through_the_statement_like_the_app_does() {
    // Run 1 showed 10 answers whose R31 button "failed to open": `reports::run` never builds R31.
    // The harness now opens it as the app does (statement_request, the services' statement,
    // statement_page). On the PRD worked day partner Alif has a total of -1,004 and Ba of 746.
    use mandi_reports::reports::ReportFilters;
    let w = build("day", None);
    let alif = ReportFilters {
        partner_id: Some(1),
        ..ReportFilters::default()
    };
    let button = statement_button(&w, &alif);
    assert_eq!(button["error"], Value::Null, "{button}");
    assert!(button["rows"].as_i64().unwrap_or(0) > 0, "{button}");
    let numbers: Vec<&str> = button["numbers"]
        .as_array()
        .expect("numbers")
        .iter()
        .filter_map(Value::as_str)
        .collect();
    assert!(
        numbers.contains(&"1004"),
        "Alif's total of -1,004 is in his statement: {numbers:?}"
    );
    // A statement with no partner is refused with the field, as on the screen.
    let none = statement_button(&w, &ReportFilters::default());
    let error = none["error"].as_str().unwrap_or("");
    assert!(error.contains("partner_id"), "{none}");
}

#[test]
fn dump_the_oracle_json() {
    // Not live. With MANDI_ORACLE_JSON=<file> (run.mjs --oracle-dump) it writes the oracle facts
    // WITH their values, and the names of the people and things in the books, for the independent
    // tester who fixes his names from this file. Without the variable it does nothing.
    let Ok(out) = std::env::var("MANDI_ORACLE_JSON") else {
        return;
    };
    let year = year_master(false);
    let thin = year_master(true);
    let mut facts = std::collections::BTreeMap::new();
    facts.extend(year.oracle.iter().map(|(k, v)| (k.clone(), v.clone())));
    facts.extend(thin.oracle.iter().map(|(k, v)| (k.clone(), v.clone())));
    // The PRD worked day too (the day, late, e42, e34 and e59 worlds hold these people).
    let day = build("day", None);
    let mut dump = json!({
        "note": "Oracle of the rich year ('year', facts without a prefix) and the thin books \
                 ('thin.', the same people, no sales). Money values are whole rupees as the app \
                 shows them; see oracle-keys.txt for the keys. Today in the year world is `today`.",
        "today": format!(
            "{:04}-{:02}-{:02}",
            year::seed::TODAY.0,
            year::seed::TODAY.1,
            year::seed::TODAY.2
        ),
        "facts": facts,
    });
    let people = people_and_things(&year.snapshot);
    if let (Some(target), Some(source)) = (dump.as_object_mut(), people.as_object()) {
        target.extend(source.iter().map(|(k, v)| (k.clone(), v.clone())));
        target.insert(
            "worlds".to_owned(),
            json!({
                "year": people,
                "day": people_and_things(&day.dir.path().join("data").join("mandi.db")),
            }),
        );
    }
    if let Some(parent) = std::path::Path::new(&out).parent() {
        std::fs::create_dir_all(parent).expect("the folder of the dump");
    }
    std::fs::write(
        &out,
        serde_json::to_string_pretty(&dump).expect("the dump as JSON"),
    )
    .expect("the dump");
    eprintln!("oracle dump written: {} facts", facts.len());
}
