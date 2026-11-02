//! Speed of "ask the app" on the NFR-SCL-1 volume (plan 50 R4, NFR-AI-3, D6-A17). Owner:
//! `robustness`.
//!
//! The model is not in these numbers: its calls are recorded replies that return at once, so what
//! is measured is the app's own part.
//!
//! - `assistant/quick_*`: a quick question (PRD FR-7.80), no model: the figure is read, checked in
//!   its two places and written to the log. Target 300 ms, like opening a screen (NFR-SPD-3).
//! - `assistant/round_*`: a whole question over recorded model replies, which is the app's part of
//!   its rounds: building each request, running the tools, the checks of the answer, filling its
//!   figures and writing the log row. One bench is the D6 worked question (find, then the party
//!   card and the ledger together, then the answer) for the busiest buyer (about 55,000 lines); the
//!   other is advice from the dues report R20 over all buyers. Target 300 ms for the app's part in
//!   each round (NFR-AI-3). A question is two rounds of tools, so holding the whole question
//!   under 300 ms holds each round under it.
//!
//! The volume is the shared one of `services/benches/support` (20,000 accounts, 3.5 million journal
//! lines). It has no sale lines and no analytics facts (that gap is in its own header), so the
//! estimate and analytics tools are not measured here: on this data they would read empty tables.
//!
//! Each bench writes (a question log row), so it runs on a private copy of the volume. A counted
//! question uses one of 20 a day, so the clock of the copy moves one day forward for each question.

#[path = "../../services/benches/support/mod.rs"]
mod support;

use std::cell::Cell;
use std::hint::black_box;
use std::sync::Arc;
use std::time::Duration;

use chrono::{TimeZone, Utc};
use criterion::{BatchSize, Criterion, SamplingMode, criterion_group, criterion_main};
use mandi_assistant::ask::{AskOutcome, AssistantAskInput, AssistantQuickInput, QuickChip};
use mandi_assistant::client::RecordedClient;
use mandi_assistant::keys::MemoryKeyStore;
use mandi_assistant::quick::quick;
use mandi_assistant::run::{self, Deps};
use mandi_assistant::runtime::Runtime;
use mandi_reports::dashboard::BackupFacts;
use mandi_services::assistant;
use mandi_services::auth::Actor;
use mandi_services::testing::TestEnv;
use serde_json::{Value, json};
use support::{copy_of_volume, describe, ensure_volume, fail, open};

/// A key-looking placeholder. The benches never call the network.
const PLACEHOLDER_KEY: &str = "sk-bench-placeholder";

struct Bed {
    _copy: tempfile::TempDir,
    env: TestEnv,
    actor: Actor,
    runtime: Runtime,
    buyer: i64,
    /// The next question's day (days after 10-10-2026).
    day: Cell<i64>,
}

/// The NFR-SCL-1 volume with the matching of every buyer's bills and every seller's advances
/// written once, as every save of a real file keeps them. The generator makes the volume with
/// plain SQL, so its old lines have no matching rows, and the dues report, the day summary and the
/// dashboard then work every match out again at each read (6 to 10 seconds here). Kept beside the
/// plain volume, with a marker, so it is made once.
fn matched_volume(plain: &support::Volume) -> support::Volume {
    use std::time::Instant;
    let dir = plain.dir.with_file_name("nfr_scl1_matched");
    let marker = dir.join("matched.ok");
    if std::fs::read_to_string(&marker).unwrap_or_default().trim() != "matched-v1" {
        let started = Instant::now();
        support::note("assistant bench: writing the matching of every buyer and seller (once)");
        let _ = std::fs::remove_dir_all(&dir);
        let to = dir.join("data");
        std::fs::create_dir_all(&to).unwrap_or_else(|e| fail(&format!("no folder: {e}")));
        std::fs::copy(plain.dir.join("data").join("mandi.db"), to.join("mandi.db"))
            .unwrap_or_else(|e| fail(&format!("cannot copy the volume: {e}")));
        let (env, actor) = open(&dir);
        let ids = |kind: &str| -> Vec<i64> {
            env.ctx
                .db()
                .read(|conn| {
                    let mut statement =
                        conn.prepare("SELECT id FROM accounts WHERE party_type = ?1 ORDER BY id")?;
                    let rows = statement.query_map([kind], |r| r.get::<_, i64>(0))?;
                    rows.collect::<Result<Vec<i64>, _>>()
                })
                .unwrap_or_else(|e| fail(&format!("cannot list the accounts: {e}")))
        };
        let now = env.ctx.now_text();
        for chunk in ids("buyer").chunks(250) {
            env.ctx
                .db()
                .write(|tx| {
                    for id in chunk {
                        mandi_services::matching::rematch(tx, actor.user_id, &now, *id, None)?;
                    }
                    Ok::<(), mandi_services::ServiceError>(())
                })
                .unwrap_or_else(|e| fail(&format!("the matching was not written: {e}")));
        }
        for chunk in ids("seller").chunks(250) {
            env.ctx
                .db()
                .write(|tx| {
                    for id in chunk {
                        mandi_services::matching::rematch_advances(tx, actor.user_id, &now, *id)?;
                    }
                    Ok::<(), mandi_services::ServiceError>(())
                })
                .unwrap_or_else(|e| fail(&format!("the advances were not matched: {e}")));
        }
        drop(env);
        let _ = std::fs::write(&marker, "matched-v1");
        support::note(&format!(
            "assistant bench: matching written in {:?}",
            started.elapsed()
        ));
    }
    support::Volume {
        dir,
        ..plain.clone()
    }
}

fn bed() -> Bed {
    let volume = matched_volume(&ensure_volume());
    describe(&volume);
    let copy = copy_of_volume(&volume);
    let (env, actor) = open(copy.path());
    assistant::set_config(&env.ctx, &actor, true, "openai/gpt-6-luna", true)
        .unwrap_or_else(|e| fail(&format!("cannot switch the assistant on: {e}")));
    let runtime = Runtime::new(Arc::new(MemoryKeyStore::with_key(PLACEHOLDER_KEY)));
    Bed {
        _copy: copy,
        env,
        actor,
        runtime,
        buyer: volume.busy_buyer,
        day: Cell::new(0),
    }
}

impl Bed {
    /// Moves the clock to a new day, so the daily count of counted questions starts again.
    fn next_day(&self) {
        let n = self.day.get().saturating_add(1);
        self.day.set(n);
        let base = Utc
            .with_ymd_and_hms(2026, 10, 10, 9, 0, 0)
            .single()
            .unwrap_or_default();
        self.env.clock.set(base + chrono::Duration::days(n));
    }

    fn ask(&self, text: &str, replies: Vec<Value>) -> AskOutcome {
        let client = RecordedClient::new(replies);
        let emit = |_s| {};
        let started = std::time::Instant::now();
        let deps = Deps {
            ctx: &self.env.ctx,
            actor: &self.actor,
            runtime: &self.runtime,
            client: &client,
            emit: &emit,
            backup: BackupFacts {
                cloud_on: false,
                cloud_age_hours: None,
            },
        };
        let outcome = match run::ask(
            &deps,
            &AssistantAskInput {
                text: text.to_owned(),
                conversation_id: None,
            },
        ) {
            Ok(reply) => {
                if std::env::var("MANDI_AI_DEBUG").is_ok() {
                    support::note(&format!(
                        "debug: failure {:?}, requests {}, took {:?}",
                        reply.failure,
                        client.requests().len(),
                        started.elapsed()
                    ));
                }
                reply.outcome
            }
            Err(e) => fail(&format!("the question failed: {e}")),
        };
        if std::env::var("MANDI_AI_DEBUG").is_ok() {
            // What the loop sent last, to see why a recorded question did not end as expected.
            let last = client.requests().last().map(|b| b["messages"].to_string());
            let tail: String = last
                .unwrap_or_default()
                .chars()
                .rev()
                .take(900)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect();
            support::note(&format!(
                "debug: {outcome:?}, the last request ends: {tail}"
            ));
        }
        outcome
    }

    fn quick(&self, chip: QuickChip, party_id: Option<i64>) -> AskOutcome {
        let client = RecordedClient::new(Vec::new());
        let emit = |_s| {};
        let deps = Deps {
            ctx: &self.env.ctx,
            actor: &self.actor,
            runtime: &self.runtime,
            client: &client,
            emit: &emit,
            backup: BackupFacts {
                cloud_on: false,
                cloud_age_hours: None,
            },
        };
        match quick(
            &deps,
            &AssistantQuickInput {
                chip,
                party_id,
                conversation_id: None,
            },
        ) {
            Ok(reply) => reply.outcome,
            Err(e) => fail(&format!("the quick question failed: {e}")),
        }
    }
}

// ---- the recorded model ----------------------------------------------------------------------

fn usage() -> Value {
    json!({"prompt_tokens": 5200, "completion_tokens": 60,
           "prompt_tokens_details": {"cached_tokens": 0},
           "completion_tokens_details": {"reasoning_tokens": 20}})
}

fn tools(calls: &[(&str, Value)]) -> Value {
    let calls: Vec<Value> = calls
        .iter()
        .enumerate()
        .map(|(i, (name, args))| {
            json!({"id": format!("call_{i}"), "type": "function",
                   "function": {"name": name, "arguments": args.to_string()}})
        })
        .collect();
    json!({"model": "openai/gpt-6-luna",
           "choices": [{"finish_reason": "tool_calls",
                        "message": {"role": "assistant", "content": null, "tool_calls": calls}}],
           "usage": usage()})
}

fn said(content: &Value) -> Value {
    json!({"model": "openai/gpt-6-luna",
           "choices": [{"finish_reason": "stop",
                        "message": {"role": "assistant", "content": content.to_string()}}],
           "usage": usage()})
}

fn report_args(id: &str, extra: Value) -> Value {
    let mut base = json!({"id": id, "from": null, "to": null, "as_of": null, "party": null,
        "item": null, "city": null, "adda": null, "collector": null, "partner": null,
        "trip": null, "garden": null, "bikri": null, "group_by": null, "where": null,
        "sort": null, "rows": null});
    if let (Some(b), Value::Object(e)) = (base.as_object_mut(), extra) {
        b.extend(e);
    }
    base
}

fn answer(kind: &str, text: &str, show: Value) -> Value {
    json!({"kind": kind, "text": text, "show": show, "next": [], "choose": null,
           "entry": null, "nearest": null})
}

/// The D6 worked question for the busiest buyer: find by code, then the card and the ledger
/// together, then the answer with the ledger as a table.
fn ledger_replies() -> Vec<Value> {
    vec![
        tools(&[("find", json!({"text": "0020", "role": "buyer"}))]),
        tools(&[
            ("party_card", json!({"party": "r1.matches.0"})),
            (
                "report",
                report_args("R13", json!({"party": "r1.matches.0"})),
            ),
        ]),
        said(&answer(
            "answer",
            "{{r1.matches.0.name}} کے ذمے {{r2.balance}} باقی ہیں۔",
            json!([{"result": "r3", "as": "table", "rows": 7}]),
        )),
    ]
}

/// Whom to collect from first: R20 over all buyers, oldest first, ten rows, then the advice. The
/// answer cites no row (their keys are the volume's), so the checks and the fill work on the
/// result as the loop built it.
fn advice_replies() -> Vec<Value> {
    vec![
        tools(&[(
            "report",
            report_args(
                "R20",
                json!({"sort": [{"column": "days", "order": "desc"},
                                {"column": "balance", "order": "desc"}], "rows": 10}),
            ),
        )]),
        said(&answer(
            "advice",
            "پہلے سب سے پرانے بقایا والوں سے وصولی کریں۔",
            json!([{"result": "r1", "as": "table", "rows": 7}]),
        )),
    ]
}

// ---- the benches -------------------------------------------------------------------------------

fn flat<'a>(
    c: &'a mut Criterion,
    samples: usize,
    seconds: u64,
) -> criterion::BenchmarkGroup<'a, criterion::measurement::WallTime> {
    let mut group = c.benchmark_group("assistant");
    group.sampling_mode(SamplingMode::Flat);
    group.sample_size(samples);
    group.measurement_time(Duration::from_secs(seconds));
    group
}

/// Where a quick question spends its time, once, when `MANDI_AI_PROBE` is set.
fn probe(bed: &Bed) {
    use std::time::Instant;
    let t = Instant::now();
    let snap = bed
        .env
        .ctx
        .snapshot()
        .unwrap_or_else(|e| fail(&format!("{e}")));
    support::note(&format!("probe: snapshot {:?}", t.elapsed()));
    let t = Instant::now();
    let snap2 = bed
        .env
        .ctx
        .snapshot()
        .unwrap_or_else(|e| fail(&format!("{e}")));
    support::note(&format!("probe: second snapshot {:?}", t.elapsed()));
    let facts = BackupFacts {
        cloud_on: false,
        cloud_age_hours: None,
    };
    let t = Instant::now();
    let _ = mandi_reports::dashboard::get(bed.env.ctx.db(), facts);
    support::note(&format!("probe: tiles on the live data {:?}", t.elapsed()));
    let t = Instant::now();
    let _ = mandi_reports::dashboard::get(snap.db(), facts);
    support::note(&format!("probe: tiles on the snapshot {:?}", t.elapsed()));
    let t = Instant::now();
    let _ = snap2.db().read(mandi_services::cash::cash_in_hand);
    support::note(&format!(
        "probe: cash in hand on a snapshot {:?}",
        t.elapsed()
    ));
    for id in ["R10", "R12", "R20", "R24", "R26", "R34", "R38"] {
        let Ok(id) = serde_json::from_value::<mandi_reports::reports::ReportId>(json!(id)) else {
            continue;
        };
        let t = Instant::now();
        let page = mandi_reports::reports::run(
            bed.env.ctx.db(),
            &mandi_reports::reports::ReportRunInput {
                id,
                filters: mandi_reports::reports::ReportFilters::default(),
                offset: 0,
                limit: 20,
                sort: None,
                group_by_month: false,
            },
        );
        support::note(&format!(
            "probe: report {id:?} {:?} ({})",
            t.elapsed(),
            page.is_ok()
        ));
    }
    // The report tool reads every row when a question sorts on two keys or filters with `where`
    // (tools/report.rs, `read_all`): one call per page of 1,000 rows, and each call works the whole
    // report out again. Time of two pages and the number of rows, to see what that costs.
    if let Ok(id) = serde_json::from_value::<mandi_reports::reports::ReportId>(json!("R20")) {
        for offset in [0, 1_000] {
            let t = Instant::now();
            let page = mandi_reports::reports::run(
                bed.env.ctx.db(),
                &mandi_reports::reports::ReportRunInput {
                    id,
                    filters: mandi_reports::reports::ReportFilters::default(),
                    offset,
                    limit: 1_000,
                    sort: None,
                    group_by_month: false,
                },
            );
            support::note(&format!(
                "probe: R20 page at {offset} (1,000 rows) {:?}, rows in the report {:?}",
                t.elapsed(),
                page.map(|p| p.total_count).ok()
            ));
        }
    }
    let t = Instant::now();
    let _ = bed.quick(QuickChip::CashInHand, None);
    support::note(&format!(
        "probe: the whole quick question {:?}",
        t.elapsed()
    ));
    let t = Instant::now();
    let _ = bed.quick(QuickChip::CashInHand, None);
    support::note(&format!(
        "probe: the whole quick question again {:?}",
        t.elapsed()
    ));
}

fn quick_questions(c: &mut Criterion) {
    let bed = bed();
    if std::env::var("MANDI_AI_PROBE").is_ok() {
        probe(&bed);
        std::process::exit(0);
    }
    let mut group = flat(c, 30, 8);
    // The chips that read a tile or a report over every buyer can take a second on this volume
    // (see the probe and the numbers in scripts/perf/README.md Section 9), so they take 10
    // samples; the party chip and the payables chip take 30. Every chip has its own bench.
    for (name, chip, party, samples) in [
        ("quick_cash_in_hand", QuickChip::CashInHand, None, 10),
        ("quick_today_sales", QuickChip::TodaySales, None, 10),
        ("quick_total_dues", QuickChip::TotalDues, None, 10),
        ("quick_sellers_payable", QuickChip::SellersPayable, None, 30),
        (
            "quick_party_busiest_buyer",
            QuickChip::PartyLedger,
            Some(bed.buyer),
            30,
        ),
        (
            "quick_today_commission",
            QuickChip::TodayCommission,
            None,
            10,
        ),
        ("quick_buyers_dues", QuickChip::BuyersDues, None, 10),
        ("quick_dead_amount", QuickChip::DeadAmount, None, 10),
        ("quick_advances_open", QuickChip::AdvancesOpen, None, 30),
        ("quick_stock_left", QuickChip::StockLeft, None, 30),
    ] {
        // Once before the clock starts, so a wrong answer is a stop, not a fast number.
        if bed.quick(chip, party) != AskOutcome::Answered {
            fail(&format!("{name} did not answer on the volume"));
        }
        group.sample_size(samples);
        group.bench_function(name, |b| b.iter(|| black_box(bed.quick(chip, party))));
    }
    group.finish();
}

fn rounds(c: &mut Criterion) {
    let bed = bed();
    let mut group = flat(c, 10, 10);
    for (name, question, replies, want) in [
        (
            "round_ledger_busiest_buyer",
            "0020 ke total bakaya kitne hai",
            ledger_replies as fn() -> Vec<Value>,
            AskOutcome::Answered,
        ),
        (
            "round_advice_dues_r20",
            "kis se pehle wasooli karun",
            advice_replies as fn() -> Vec<Value>,
            AskOutcome::Advice,
        ),
    ] {
        bed.next_day();
        let started = std::time::Instant::now();
        let got = bed.ask(question, replies());
        if got != want {
            // Not measured, and loud: the benchmark has no result, so `pnpm perf` reports it as
            // MISSING and fails. (A question that ends in an error after a minute has no speed
            // to report.) It is measured again when the question ends as it should.
            support::note(&format!(
                "OVER: {name} ended as {got:?}, not {want:?}, after {:?} on the volume (the limit of the question is 45 s); not measured",
                started.elapsed()
            ));
            continue;
        }
        group.bench_function(name, |b| {
            b.iter_batched(
                || {
                    bed.next_day();
                    replies()
                },
                |replies| black_box(bed.ask(question, replies)),
                BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

/// The dashboard tiles and the day summary R34 on the same volume (NFR-SPD-2: the dashboard under
/// 1 s; NFR-SPD-10: any report for one day under 1 s). The quick questions and two tools of the
/// assistant read them, so a slow read here is a slow quick question there. Found by the assistant
/// benchmarks, held here at the NFR targets.
fn dashboard_and_day_summary(c: &mut Criterion) {
    let bed = bed();
    let facts = BackupFacts {
        cloud_on: false,
        cloud_age_hours: None,
    };
    let mut group = c.benchmark_group("dashboard");
    group.sampling_mode(SamplingMode::Flat);
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(10));
    group.bench_function("get_full_volume", |b| {
        b.iter(
            || match mandi_reports::dashboard::get(bed.env.ctx.db(), facts) {
                Ok(page) => black_box(page.tiles.sales_gross_paisa.as_paisa()),
                Err(e) => fail(&format!("the dashboard failed: {e}")),
            },
        );
    });
    group.finish();
    let mut group = c.benchmark_group("reports");
    group.sampling_mode(SamplingMode::Flat);
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(10));
    let input = mandi_reports::reports::ReportRunInput {
        id: mandi_reports::reports::ReportId::R34,
        filters: mandi_reports::reports::ReportFilters {
            from: "2026-10-10".to_owned(),
            to: "2026-10-10".to_owned(),
            ..mandi_reports::reports::ReportFilters::default()
        },
        offset: 0,
        limit: 100,
        sort: None,
        group_by_month: false,
    };
    group.bench_function("r34_one_day_full_volume", |b| {
        b.iter(
            || match mandi_reports::reports::run(bed.env.ctx.db(), black_box(&input)) {
                Ok(page) => black_box(page.rows.len()),
                Err(e) => fail(&format!("R34 failed: {e}")),
            },
        );
    });
    group.finish();
}

criterion_group!(benches, quick_questions, rounds, dashboard_and_day_summary);
criterion_main!(benches);
