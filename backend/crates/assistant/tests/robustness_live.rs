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
//! - `MANDI_EVAL_THREADS`: optional, questions at once (default 1, so the times are honest).
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

use mandi_assistant::ask::{AskReply, AssistantAskInput};
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
}

fn questions(path: &str, only: &[String]) -> Vec<Question> {
    let text = std::fs::read_to_string(path).expect("the questions file");
    let mut rows = text.lines();
    let head: Vec<&str> = rows.next().expect("a header").split('\t').collect();
    let at = |name: &str| head.iter().position(|h| *h == name).expect(name);
    let (id, world, after, question) = (at("id"), at("world"), at("after"), at("question"));
    rows.filter(|l| !l.trim().is_empty())
        .map(|l| {
            let cells: Vec<&str> = l.split('\t').collect();
            Question {
                id: cells[id].to_owned(),
                world: cells[world].to_owned(),
                after: cells[after].to_owned(),
                text: cells[question].to_owned(),
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
}

/// The real client, with a note of every call: time, tokens, how many tools it asked for.
struct Recorder {
    inner: Box<dyn ModelClient>,
    calls: Mutex<Vec<CallRecord>>,
}

impl ModelClient for Recorder {
    fn call(&self, key: &str, body: &Value, timeout: Duration) -> Result<Reply, CallError> {
        let started = Instant::now();
        let result = self.inner.call(key, body, timeout);
        let millis = started.elapsed().as_millis();
        if let Ok(reply) = &result {
            self.calls.lock().expect("the notes").push(CallRecord {
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
    let reply = match try_ask(&deps, &q.text, conversation.as_deref()) {
        Ok(r) => r,
        Err(e) => return failed(e),
    };
    let millis = started.elapsed().as_millis();
    let Ok(Some(row)) = assistant::get_question(&w.env.ctx, reply.question_id) else {
        return failed("the question log has no row".to_owned());
    };
    let calls: Vec<Value> = client
        .calls
        .lock()
        .expect("notes")
        .iter()
        .map(|c| {
            json!({"millis": c.millis, "prompt": c.prompt, "cached": c.cached,
                   "completion": c.completion, "reasoning": c.reasoning,
                   "tool_calls": c.tool_calls, "finish": c.finish, "model": c.model})
        })
        .collect();
    let tool_calls: Value =
        serde_json::from_str(&row.tool_calls_json).unwrap_or(Value::Array(Vec::new()));
    eprintln!("{} {} {:?} ({} ms)", q.id, q.world, reply.outcome, millis);
    json!({
        "id": q.id,
        "question": q.text,
        "world": q.world,
        "reply": serde_json::to_value(&reply).expect("the reply as JSON"),
        "tool_calls": tool_calls,
        "model_calls": row.model_calls,
        "tokens": {"in": row.tokens_in, "cached": row.tokens_cached,
                   "out": row.tokens_out, "reasoning": row.tokens_reasoning},
        "calls": calls,
        "millis": millis,
        // The log row of this very question (lead's item 0b): what the app itself wrote down.
        "calls_json": row.calls_json,
        "answer_json": row.answer_json,
        "finish_reason": row.finish_reason,
        "total_millis": row.total_millis,
        "model_millis": row.model_millis,
        "tool_millis": row.tool_millis,
        // The question as the model counts tokens, to take it off the first call's prompt tokens
        // when the fixed part is measured (lead's item 0c).
        "user_tokens": user_tokens(&q.text),
        "followup": !q.after.is_empty(),
    })
}

/// The tokens of a question text (o200k_base, the count the prompt test uses).
fn user_tokens(text: &str) -> usize {
    tiktoken_rs::o200k_base()
        .map(|bpe| bpe.encode_with_special_tokens(text).len())
        .unwrap_or(0)
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
    // How many questions run at once. 1 keeps the times honest; more only for a quick check of
    // the figures (the provider may then answer slower or say it is busy).
    let threads: usize = std::env::var("MANDI_EVAL_THREADS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1)
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
    });
    let mut facts = std::collections::BTreeMap::new();
    for m in [masters.year.as_ref(), masters.thin.as_ref()]
        .into_iter()
        .flatten()
    {
        facts.extend(m.oracle.iter().map(|(k, v)| (k.clone(), v.clone())));
    }
    let oracle = json!(facts);
    let results: Mutex<Vec<Value>> = Mutex::new(Vec::new());
    let next = std::sync::atomic::AtomicUsize::new(0);
    // The file is rewritten after every question, so a stopped run keeps what it has.
    let flush = |rows: &Vec<Value>| {
        std::fs::write(
            &out_path,
            json!({"meta": meta, "oracle": oracle, "results": rows}).to_string(),
        )
        .expect("the results file");
    };
    std::thread::scope(|scope| {
        for _ in 0..threads {
            scope.spawn(|| {
                let runtime = Runtime::real();
                loop {
                    let at = next.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    let Some(q) = list.get(at) else {
                        break;
                    };
                    let row = run_one(q, &all, &by_id, Some(&masters), &model, &runtime, &|| {
                        Box::new(OpenRouterClient)
                    });
                    let mut rows = results.lock().expect("the results");
                    rows.push(row);
                    flush(&rows);
                }
            });
        }
    });
    let rows = results.lock().expect("the results");
    flush(&rows);
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
    // A question on the thin books.
    let thin = all
        .iter()
        .find(|q| q.world == "thin")
        .expect("a thin question");
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
}
