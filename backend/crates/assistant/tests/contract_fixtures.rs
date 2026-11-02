//! Contract fixtures of plan 48 T1 (ask the app, D6 Section 2): one `AskReply` for every outcome,
//! the answers that show a figure, a table, a chart, an estimate and advice, and the other
//! commands' replies, as JSON files for the frontend.
//!
//! Each fixture is a pair: `<name>_request.json` is exactly the `input` the command takes (it is
//! read back into the real input type) and `<name>.json` is the reply. At this step the replies
//! are built by hand from the typed contract, with the numbers of the PRD worked day (اکرم
//! ٹول ٹیکس 0020, 2,880 بنام = 288000 paisa), so a screen can be built before the logic exists.
//! From T3 the golden questions compare their real answers with these shapes. The frontend copies
//! the files to `frontend/tests/fixtures/` and never edits them.
//!
//! The test compares the replies with the files on disk and FAILS if any differ, so a fixture can
//! never go stale. After a deliberate change, write the files again with
//! `MANDI_WRITE_FIXTURES=1 cargo test -p mandi-assistant --test contract_fixtures`, read the diff
//! and tell `frontend`.
#![cfg(test)]

use std::path::{Path, PathBuf};

use mandi_assistant::answer::{
    Answer, AnswerChart, AnswerEstimate, AnswerMonth, AnswerName, AnswerOption, AnswerPart,
    AnswerRow, AnswerSource, AnswerTable, AnswerWorking, ChartSeries, ChartSeriesKind, EntryRoute,
    EntryScreen, EstimateMethod, EstimateMethodKind, EstimateYear, FigureDifference,
    FigureDirection, PlaceValue, WorkingOp, WorkingStep,
};
use mandi_assistant::ask::{
    AskOutcome, AskReply, AssistantAskInput, AssistantFailure, AssistantNewConversationInput,
    AssistantPickInput, AssistantQuickInput, AssistantRerunInput, ConversationStarted, FailureKind,
    StopReason,
};
use mandi_assistant::limits::DAILY_LIMIT;
use mandi_assistant::log::{
    AssistantLogListInput, AssistantLogPage, AssistantLogRow, LogSource, LogTool,
};
use mandi_assistant::settings::{
    AssistantKeyRemoveInput, AssistantKeySaveInput, AssistantSettingsSetInput, AssistantStatus,
    AssistantTestResult, DEFAULT_MODEL,
};
use mandi_assistant::source::{
    AnswerRecord, SourceAnalyticsFilters, SourceParty, SourceReportFilters,
};
use mandi_assistant::step::AssistantStep;
use mandi_reports::reports::{
    ArgValue, Cell, Column, ColumnKind, DetailPart, MessageArg, Names, PartyRef, ReportId, Row,
    RowKind, RowSource, SourceKind,
};
use mandi_services::core::accounts::BalanceSide;
use mandi_services::core::money::Paisa;
use serde::Serialize;
use serde_json::{Value, json};

const WRITE_VARIABLE: &str = "MANDI_WRITE_FIXTURES";
const READ_AT: &str = "2026-10-10T13:12:00Z";

fn folder() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
}

fn write_or_check(file: &str, text: &str) {
    let path = folder().join(file);
    if std::env::var_os(WRITE_VARIABLE).is_some() {
        std::fs::create_dir_all(folder()).unwrap();
        std::fs::write(&path, text).unwrap();
        return;
    }
    let on_disk = std::fs::read_to_string(&path).unwrap_or_else(|_| {
        panic!("{file} is missing: run with {WRITE_VARIABLE}=1 to write the fixtures")
    });
    assert!(
        on_disk == text,
        "{file} is not the contract any more. If the change is on purpose, run with \
         {WRITE_VARIABLE}=1, read the diff and tell frontend."
    );
}

/// The reply file only.
fn reply_only<A: Serialize>(name: &str, reply: &A) {
    write_or_check(
        &format!("{name}.json"),
        &(serde_json::to_string_pretty(reply).unwrap() + "\n"),
    );
}

/// A pair of files: the request and the reply.
fn pair<A: Serialize>(name: &str, request: &Value, reply: &A) {
    write_or_check(
        &format!("{name}_request.json"),
        &(serde_json::to_string_pretty(request).unwrap() + "\n"),
    );
    reply_only(name, reply);
}

// ---- small builders -------------------------------------------------------------------------

fn rs(rupees: i64) -> Paisa {
    Paisa::from_rupees(rupees).unwrap()
}

fn money(rupees: i64) -> Cell {
    Cell::Money { paisa: rs(rupees) }
}

fn dr(rupees: i64) -> Cell {
    Cell::Balance {
        paisa: rs(rupees),
        side: BalanceSide::Dr,
    }
}

fn text(words: &str) -> AnswerPart {
    AnswerPart::Text {
        text: words.to_owned(),
    }
}

fn figure(cell: Cell, checked: bool) -> AnswerPart {
    AnswerPart::Figure {
        cell,
        dir: None,
        checked,
        working: None,
    }
}

fn akram() -> AnswerName {
    AnswerName {
        name_ur: "اکرم ٹول ٹیکس".to_owned(),
        name_en: "Akram Toll Tax".to_owned(),
        tag: None,
        code: Some("0020".to_owned()),
    }
}

fn arshad() -> AnswerName {
    AnswerName {
        name_ur: "ارشد ملتانی".to_owned(),
        name_en: "Arshad Multani".to_owned(),
        tag: None,
        code: Some("0021".to_owned()),
    }
}

fn akram_k() -> AnswerName {
    AnswerName {
        name_ur: "اکرم".to_owned(),
        name_en: "Akram".to_owned(),
        tag: Some("K".to_owned()),
        code: Some("0022".to_owned()),
    }
}

fn party_ref(name: &AnswerName, account_id: i64, short: &str) -> PartyRef {
    PartyRef {
        account_id,
        code: name.code.clone().unwrap(),
        short_code: Some(short.to_owned()),
        name_ur: name.name_ur.clone(),
        name_en: name.name_en.clone(),
        tag: name.tag.clone(),
        person_key: None,
        phone: None,
        dead: None,
    }
}

fn column(key: &str, title_id: &str, kind: ColumnKind) -> Column {
    Column {
        key: key.to_owned(),
        title_id: title_id.to_owned(),
        kind,
    }
}

fn month(year: i64, month: i64) -> AnswerMonth {
    AnswerMonth { year, month }
}

fn row(cells: Vec<Cell>, source: Option<RowSource>) -> AnswerRow {
    AnswerRow {
        row: Row {
            cells,
            source,
            kind: RowKind::Normal,
        },
        analytics_key: None,
    }
}

fn reply(question_id: i64, outcome: AskOutcome) -> AskReply {
    AskReply {
        question_id,
        conversation_id: "c-1".to_owned(),
        outcome,
        answer: None,
        options: Vec::new(),
        more_options: false,
        entry_route: None,
        locked: false,
        failure: None,
        stop_reason: None,
        count_today: 3,
        limit: DAILY_LIMIT,
    }
}

fn answer(parts: Vec<AnswerPart>) -> Answer {
    Answer {
        parts,
        tables: Vec::new(),
        charts: Vec::new(),
        estimate: None,
        advice: false,
        general_advice: false,
        sources: Vec::new(),
        differences: Vec::new(),
        next: Vec::new(),
        read_at: READ_AT.to_owned(),
    }
}

fn record(name: &AnswerName, id: i64) -> AnswerRecord {
    AnswerRecord {
        id,
        code: name.code.clone(),
        name_ur: name.name_ur.clone(),
        name_en: name.name_en.clone(),
        tag: name.tag.clone(),
    }
}

fn r13_filters() -> SourceReportFilters {
    SourceReportFilters {
        party: Some(SourceParty {
            record: record(&akram(), 20),
            person_key: None,
        }),
        ..SourceReportFilters::default()
    }
}

fn card_source(result_id: &str, account_id: i64, name: AnswerName) -> AnswerSource {
    AnswerSource::PartyCard {
        result_id: result_id.to_owned(),
        party: record(&name, account_id),
    }
}

fn r13_source() -> AnswerSource {
    AnswerSource::Report {
        result_id: "r3".to_owned(),
        label_id: "report-r13-title".to_owned(),
        id: ReportId::R13,
        filters: r13_filters(),
    }
}

/// R13 of اکرم ٹول ٹیکس: the receipt of 10,000 (newest first) and the sale of 12,880.
fn r13_table() -> AnswerTable {
    let sale = Cell::Detail {
        parts: vec![DetailPart::Sale {
            bikri_number: 1,
            item: Names {
                ur: "کیلا".to_owned(),
                en: "Banana".to_owned(),
            },
            packing: Names {
                ur: "پیٹی".to_owned(),
                en: "Box".to_owned(),
            },
            qty: 20,
            rate_paisa: rs(600),
            value_paisa: rs(12_000),
            commission_paisa: rs(720),
            laga_paisa: rs(160),
        }],
    };
    let date = Cell::Date {
        iso: "2026-10-10".to_owned(),
    };
    AnswerTable {
        result_id: "r3".to_owned(),
        title_id: "report-r13-title".to_owned(),
        columns: vec![
            column("date", "report-col-date", ColumnKind::Date),
            column("doc", "report-col-doc", ColumnKind::Doc),
            column("detail", "report-col-detail", ColumnKind::Detail),
            column("narration", "report-col-narration", ColumnKind::Text),
            column("debit", "report-col-debit", ColumnKind::Money),
            column("credit", "report-col-credit", ColumnKind::Money),
            column("balance", "report-col-balance", ColumnKind::Balance),
        ],
        rows: vec![
            row(
                vec![
                    date.clone(),
                    Cell::Doc {
                        type_id: "report-doc-receipt".to_owned(),
                        number: Some(1),
                    },
                    Cell::Empty,
                    Cell::Empty,
                    Cell::Empty,
                    money(10_000),
                    dr(2_880),
                ],
                Some(RowSource {
                    kind: SourceKind::Voucher,
                    id: 2,
                }),
            ),
            row(
                vec![
                    date,
                    Cell::Doc {
                        type_id: "report-doc-bikri-first".to_owned(),
                        number: Some(1),
                    },
                    sale,
                    Cell::Empty,
                    money(12_880),
                    Cell::Empty,
                    dr(12_880),
                ],
                Some(RowSource {
                    kind: SourceKind::Bikri,
                    id: 1,
                }),
            ),
        ],
        count: 2,
        totals_label_id: "report-total".to_owned(),
        totals: vec![
            Cell::Empty,
            Cell::Empty,
            Cell::Empty,
            Cell::Empty,
            money(12_880),
            money(10_000),
            dr(2_880),
        ],
    }
}

/// R20 as on 15-11-26 (E2E-63): ارشد ملتانی 18,684 then اکرم ٹول ٹیکس 2,880, both 36 days old.
fn r20_table() -> AnswerTable {
    let line = |name: &AnswerName, id: i64, short: &str, balance: i64| {
        row(
            vec![
                Cell::Party {
                    party: party_ref(name, id, short),
                },
                Cell::Empty,
                Cell::Empty,
                money(balance),
                Cell::Date {
                    iso: "2026-10-10".to_owned(),
                },
                Cell::Number { value: 36 },
                Cell::Empty,
                Cell::Empty,
                money(balance),
                Cell::Empty,
            ],
            Some(RowSource {
                kind: SourceKind::Account,
                id,
            }),
        )
    };
    AnswerTable {
        result_id: "r2".to_owned(),
        title_id: "report-r20-title".to_owned(),
        columns: vec![
            column("buyer", "report-col-buyer", ColumnKind::Party),
            column("city", "report-col-city", ColumnKind::Names),
            column("phone", "report-col-phone", ColumnKind::Text),
            column("balance", "report-col-balance", ColumnKind::Money),
            column("oldest_date", "report-col-oldest-date", ColumnKind::Date),
            column("days", "report-col-days", ColumnKind::Number),
            column("age_0_7", "report-col-age-0-7", ColumnKind::Money),
            column("age_8_30", "report-col-age-8-30", ColumnKind::Money),
            column("age_31_90", "report-col-age-31-90", ColumnKind::Money),
            column("age_91", "report-col-age-91", ColumnKind::Money),
        ],
        rows: vec![
            line(&arshad(), 21, "1", 18_684),
            line(&akram(), 20, "0", 2_880),
        ],
        count: 2,
        totals_label_id: "report-total".to_owned(),
        totals: vec![
            Cell::Empty,
            Cell::Empty,
            Cell::Empty,
            money(21_564),
            Cell::Empty,
            Cell::Empty,
            Cell::Empty,
            Cell::Empty,
            money(21_564),
            Cell::Empty,
        ],
    }
}

// ---- the replies ----------------------------------------------------------------------------

/// E2E-47: "ikram ke total bakaya kitne hai".
fn answered() -> AskReply {
    let mut a = answer(vec![
        AnswerPart::Name { name: akram() },
        text(" کے ذمے "),
        figure(dr(2_880), true),
        text(" باقی ہیں۔ آج "),
        AnswerPart::Doc {
            type_id: "report-doc-bikri-first".to_owned(),
            number: Some(1),
        },
        text(" میں ان کا بل "),
        figure(money(12_880), false),
        text(" بنا اور ان سے "),
        figure(money(10_000), false),
        text(" وصول ہوئے۔"),
    ]);
    a.tables = vec![r13_table()];
    a.sources = vec![card_source("r2", 20, akram()), r13_source()];
    a.next = vec![
        "پچھلے مہینے کا حساب؟".to_owned(),
        "اکرم ٹول ٹیکس کی پرانی بکریاں".to_owned(),
    ];
    let mut r = reply(1, AskOutcome::Answered);
    r.count_today = 1;
    r.answer = Some(a);
    r
}

/// E2E-49: "akram aur arshad dono ka mila kar kitna hai": the calculator's sum with its working.
fn answered_sum() -> AskReply {
    let sum = AnswerPart::Figure {
        cell: dr(21_564),
        dir: None,
        checked: true,
        working: Some(AnswerWorking {
            title_id: "assistant-working-sum".to_owned(),
            steps: vec![WorkingStep {
                op: WorkingOp::Sum,
                inputs: vec![dr(2_880), dr(18_684)],
                result: dr(21_564),
            }],
        }),
    };
    let mut a = answer(vec![
        text("دونوں کے ذمے مل کر "),
        sum,
        text(" باقی ہیں۔ ان میں "),
        AnswerPart::Name { name: akram() },
        text(" کے "),
        figure(dr(2_880), true),
        text(" اور "),
        AnswerPart::Name { name: arshad() },
        text(" کے "),
        figure(dr(18_684), true),
        text(" ہیں۔"),
    ]);
    a.sources = vec![
        card_source("r1", 20, akram()),
        card_source("r2", 21, arshad()),
    ];
    let mut r = reply(2, AskOutcome::Answered);
    r.answer = Some(a);
    r
}

/// A comparison with a direction word and a monthly chart (E2E-59 history, view A).
fn answered_chart() -> AskReply {
    let months: Vec<AnswerMonth> = (4..=9).map(|m| month(2026, m)).collect();
    let values: Vec<Cell> = [36_000, 39_000, 42_000, 45_000, 48_000, 51_000]
        .iter()
        .map(|rupees| money(*rupees))
        .collect();
    let mut a = answer(vec![
        AnswerPart::Month {
            year: 2026,
            month: 9,
        },
        text(" کا کمیشن "),
        AnswerPart::Month {
            year: 2026,
            month: 8,
        },
        text(" سے "),
        AnswerPart::Figure {
            cell: money(3_000),
            dir: Some(FigureDirection::Up),
            checked: false,
            working: Some(AnswerWorking {
                title_id: "assistant-working-difference".to_owned(),
                steps: vec![WorkingStep {
                    op: WorkingOp::Difference,
                    inputs: vec![money(51_000), money(48_000)],
                    result: money(3_000),
                }],
            }),
        },
        text(" رہا۔"),
    ]);
    a.charts = vec![AnswerChart {
        result_id: "r4".to_owned(),
        measure_id: "assistant-measure-commission-all".to_owned(),
        months,
        series: vec![ChartSeries {
            kind: ChartSeriesKind::Actual,
            values,
        }],
        is_estimate: false,
    }];
    a.sources = vec![AnswerSource::Analytics {
        result_id: "r4".to_owned(),
        label_id: "analytics-view-a-title".to_owned(),
        view: mandi_reports::analytics::AnalyticsView::A,
        filters: SourceAnalyticsFilters {
            from: "2026-04-01".to_owned(),
            to: "2026-09-30".to_owned(),
            group_by: Some(mandi_reports::analytics::AnalyticsGroupBy::Month),
            ..SourceAnalyticsFilters::default()
        },
    }];
    let mut r = reply(3, AskOutcome::Answered);
    r.answer = Some(a);
    r
}

/// The estimate of E2E-59 (today 10-10-26): same pace 288,000, same growth 369,000 over 6 months.
/// With a target (the prompt's example 8: 3 lakh) the estimator looks 12 months ahead and says
/// when the running total reaches it: April 2027 by the same pace, February 2027 by the same
/// growth.
fn estimate_reply(with_target: bool) -> AskReply {
    let ahead: i64 = if with_target { 12 } else { 6 };
    // The months ahead, October 2026 on, and the two lines (whole rupees).
    let pace_line: Vec<i64> = (0..ahead).map(|_| 48_000).collect();
    let growth_line: Vec<i64> = (0..ahead).map(|i| 54_000 + 3_000 * i).collect();
    let pace_total: i64 = pace_line.iter().sum();
    let growth_total: i64 = growth_line.iter().sum();
    assert_eq!(
        (pace_total, growth_total),
        if with_target {
            (576_000, 846_000)
        } else {
            (288_000, 369_000)
        }
    );
    let pace = EstimateMethod {
        kind: EstimateMethodKind::SamePace,
        months_used: 3,
        used_from: month(2026, 7),
        used_to: month(2026, 9),
        per_month: Some(money(48_000)),
        step: None,
        step_dir: None,
        total: money(pace_total),
        target_month: with_target.then(|| month(2027, 4)),
        factor: None,
        year_total: None,
        working: AnswerWorking {
            title_id: "assistant-working-same-pace".to_owned(),
            steps: vec![
                WorkingStep {
                    op: WorkingOp::Average,
                    inputs: vec![money(45_000), money(48_000), money(51_000)],
                    result: money(48_000),
                },
                WorkingStep {
                    op: WorkingOp::Times,
                    inputs: vec![money(48_000), Cell::Number { value: ahead }],
                    result: money(pace_total),
                },
            ],
        },
    };
    let growth = EstimateMethod {
        kind: EstimateMethodKind::SameGrowth,
        months_used: 6,
        used_from: month(2026, 4),
        used_to: month(2026, 9),
        per_month: None,
        step: Some(money(3_000)),
        step_dir: Some(FigureDirection::Up),
        total: money(growth_total),
        target_month: with_target.then(|| month(2027, 2)),
        factor: None,
        year_total: None,
        working: AnswerWorking {
            title_id: "assistant-working-same-growth".to_owned(),
            steps: vec![
                WorkingStep {
                    op: WorkingOp::Difference,
                    inputs: vec![money(51_000), money(36_000)],
                    result: money(15_000),
                },
                WorkingStep {
                    op: WorkingOp::DividedBy,
                    inputs: vec![money(15_000), Cell::Number { value: 5 }],
                    result: money(3_000),
                },
                WorkingStep {
                    op: WorkingOp::Sum,
                    inputs: growth_line.iter().map(|rupees| money(*rupees)).collect(),
                    result: money(growth_total),
                },
            ],
        },
    };
    // Six past months (April to September 2026), then the months ahead.
    let months: Vec<AnswerMonth> = (4..=12)
        .map(|m| month(2026, m))
        .chain((1..=12).map(|m| month(2027, m)))
        .take(usize::try_from(6 + ahead).unwrap())
        .collect();
    let past: Vec<Cell> = [36_000, 39_000, 42_000, 45_000, 48_000, 51_000]
        .iter()
        .map(|rupees| money(*rupees))
        .collect();
    let with_past_gap = |line: &[i64]| -> Vec<Cell> {
        std::iter::repeat_n(Cell::Empty, 6)
            .chain(line.iter().map(|rupees| money(*rupees)))
            .collect()
    };
    let actual: Vec<Cell> = past
        .into_iter()
        .chain(std::iter::repeat_n(
            Cell::Empty,
            usize::try_from(ahead).unwrap(),
        ))
        .collect();
    let first = month(2026, 10);
    let last = *months.last().unwrap();
    let mut parts = vec![
        text("اگر کمیشن اسی رفتار سے چلتا رہا تو "),
        AnswerPart::Month {
            year: 2026,
            month: 10,
        },
        text(" سے "),
        AnswerPart::Month {
            year: last.year,
            month: last.month,
        },
        text(" تک تقریباً "),
        figure(money(pace_total), false),
        text(" بنے گا۔ اگر ہر مہینہ "),
        AnswerPart::Figure {
            cell: money(3_000),
            dir: Some(FigureDirection::Up),
            checked: false,
            working: None,
        },
        text(" رہا تو "),
        figure(money(growth_total), false),
        text("۔"),
    ];
    if with_target {
        parts = vec![
            text("ہدف "),
            figure(money(300_000), false),
            text(" اسی رفتار سے "),
            AnswerPart::Month {
                year: 2027,
                month: 4,
            },
            text(" میں پورا ہو گا اور اسی اضافے سے "),
            AnswerPart::Month {
                year: 2027,
                month: 2,
            },
            text(" میں۔"),
        ];
    }
    let mut a = answer(parts);
    a.charts = vec![AnswerChart {
        result_id: "e1".to_owned(),
        measure_id: "assistant-measure-commission-all".to_owned(),
        months,
        series: vec![
            ChartSeries {
                kind: ChartSeriesKind::Actual,
                values: actual,
            },
            ChartSeries {
                kind: ChartSeriesKind::SamePace,
                values: with_past_gap(&pace_line),
            },
            ChartSeries {
                kind: ChartSeriesKind::SameGrowth,
                values: with_past_gap(&growth_line),
            },
        ],
        is_estimate: true,
    }];
    a.estimate = Some(AnswerEstimate {
        result_id: "e1".to_owned(),
        measure_id: "assistant-measure-commission-all".to_owned(),
        full_months: 6,
        first,
        last,
        target: with_target.then(|| money(300_000)),
        methods: vec![pace, growth],
        year: None,
    });
    let mut r = reply(if with_target { 5 } else { 4 }, AskOutcome::Estimate);
    r.answer = Some(a);
    r
}

/// The working step that adds up the books' months of E2E-59: April to September, 261,000.
fn books_sum_step() -> WorkingStep {
    WorkingStep {
        op: WorkingOp::Sum,
        inputs: [36_000, 39_000, 42_000, 45_000, 48_000, 51_000]
            .iter()
            .map(|rupees| money(*rupees))
            .collect(),
        result: money(261_000),
    }
}

/// The books' months of 2026 before October in E2E-59: April to September, 261,000.
fn books_so_far() -> EstimateYear {
    EstimateYear {
        so_far: money(261_000),
        months: 6,
        from: Some(month(2026, 4)),
        to: Some(month(2026, 9)),
    }
}

/// To the end of the year (plan 52 item 5, Umar 02-11-2026, today 10-10-26): "saal ke aakhir tak
/// commission kitna". The whole year is the books' 261,000 plus the months left (October to
/// December): same pace 48,000 × 3 = 144,000, whole year 405,000; same growth 54,000 + 57,000 +
/// 60,000 = 171,000, whole year 432,000. October is counted as a whole estimated month.
fn year_end_reply() -> AskReply {
    let step = |op: WorkingOp, inputs: Vec<Cell>, result: Cell| WorkingStep { op, inputs, result };
    let pace = EstimateMethod {
        kind: EstimateMethodKind::SamePace,
        months_used: 3,
        used_from: month(2026, 7),
        used_to: month(2026, 9),
        per_month: Some(money(48_000)),
        step: None,
        step_dir: None,
        total: money(144_000),
        target_month: None,
        factor: None,
        year_total: Some(money(405_000)),
        working: AnswerWorking {
            title_id: "assistant-working-same-pace".to_owned(),
            steps: vec![
                step(
                    WorkingOp::Average,
                    vec![money(45_000), money(48_000), money(51_000)],
                    money(48_000),
                ),
                step(
                    WorkingOp::Times,
                    vec![money(48_000), Cell::Number { value: 3 }],
                    money(144_000),
                ),
                books_sum_step(),
                step(
                    WorkingOp::Sum,
                    vec![money(261_000), money(144_000)],
                    money(405_000),
                ),
            ],
        },
    };
    let growth = EstimateMethod {
        kind: EstimateMethodKind::SameGrowth,
        months_used: 6,
        used_from: month(2026, 4),
        used_to: month(2026, 9),
        per_month: None,
        step: Some(money(3_000)),
        step_dir: Some(FigureDirection::Up),
        total: money(171_000),
        target_month: None,
        factor: None,
        year_total: Some(money(432_000)),
        working: AnswerWorking {
            title_id: "assistant-working-same-growth".to_owned(),
            steps: vec![
                step(
                    WorkingOp::Difference,
                    vec![money(51_000), money(36_000)],
                    money(15_000),
                ),
                step(
                    WorkingOp::DividedBy,
                    vec![money(15_000), Cell::Number { value: 5 }],
                    money(3_000),
                ),
                step(
                    WorkingOp::Sum,
                    vec![money(54_000), money(57_000), money(60_000)],
                    money(171_000),
                ),
                books_sum_step(),
                step(
                    WorkingOp::Sum,
                    vec![money(261_000), money(171_000)],
                    money(432_000),
                ),
            ],
        },
    };
    let months: Vec<AnswerMonth> = (4..=12).map(|m| month(2026, m)).collect();
    let past: Vec<Cell> = [36_000, 39_000, 42_000, 45_000, 48_000, 51_000]
        .iter()
        .map(|rupees| money(*rupees))
        .collect();
    let gap = |line: &[i64]| -> Vec<Cell> {
        std::iter::repeat_n(Cell::Empty, 6)
            .chain(line.iter().map(|rupees| money(*rupees)))
            .collect()
    };
    let actual: Vec<Cell> = past
        .into_iter()
        .chain(std::iter::repeat_n(Cell::Empty, 3))
        .collect();
    let mut a = answer(vec![
        AnswerPart::Month {
            year: 2026,
            month: 4,
        },
        text(" سے "),
        AnswerPart::Month {
            year: 2026,
            month: 9,
        },
        text(" تک کتابوں میں "),
        figure(money(261_000), false),
        text(" ہے۔ "),
        AnswerPart::Month {
            year: 2026,
            month: 10,
        },
        text(" سے "),
        AnswerPart::Month {
            year: 2026,
            month: 12,
        },
        text(" تک اسی رفتار سے "),
        figure(money(144_000), false),
        text(" اور پورے سال کا کل تقریباً "),
        figure(money(405_000), false),
        text(" بنے گا، اور اسی اضافے سے "),
        figure(money(432_000), false),
        text("۔"),
    ]);
    a.charts = vec![AnswerChart {
        result_id: "e1".to_owned(),
        measure_id: "assistant-measure-commission-all".to_owned(),
        months,
        series: vec![
            ChartSeries {
                kind: ChartSeriesKind::Actual,
                values: actual,
            },
            ChartSeries {
                kind: ChartSeriesKind::SamePace,
                values: gap(&[48_000, 48_000, 48_000]),
            },
            ChartSeries {
                kind: ChartSeriesKind::SameGrowth,
                values: gap(&[54_000, 57_000, 60_000]),
            },
        ],
        is_estimate: true,
    }];
    a.estimate = Some(AnswerEstimate {
        result_id: "e1".to_owned(),
        measure_id: "assistant-measure-commission-all".to_owned(),
        full_months: 6,
        first: month(2026, 10),
        last: month(2026, 12),
        target: None,
        methods: vec![pace, growth],
        year: Some(books_so_far()),
    });
    let mut r = reply(5, AskOutcome::Estimate);
    r.answer = Some(a);
    r
}

/// A what-if (plan 52 item 5, today 10-10-26): "is raftar se saal ke aakhir tak commission kitna,
/// aur agar raftar double ho jaye to". Until the end of the year is 3 months (October to December):
/// same pace 48,000 a month, 144,000; the pace doubled 96,000 a month, 288,000. The Owner's word
/// "double" is never in the model's text: the app shows the factor, 200.00%. To the end of the year
/// the answer is the whole year: the books' 261,000 (April to September) plus the months left, so
/// 405,000 by the same pace and 549,000 by the doubled one.
fn what_if_reply() -> AskReply {
    let ahead: i64 = 3;
    let pace_total: i64 = 48_000 * ahead;
    let doubled_total: i64 = 96_000 * ahead;
    let step = |op: WorkingOp, inputs: Vec<Cell>, result: Cell| WorkingStep { op, inputs, result };
    let average = step(
        WorkingOp::Average,
        vec![money(45_000), money(48_000), money(51_000)],
        money(48_000),
    );
    let pace = EstimateMethod {
        kind: EstimateMethodKind::SamePace,
        months_used: 3,
        used_from: month(2026, 7),
        used_to: month(2026, 9),
        per_month: Some(money(48_000)),
        step: None,
        step_dir: None,
        total: money(pace_total),
        target_month: None,
        factor: None,
        year_total: Some(money(405_000)),
        working: AnswerWorking {
            title_id: "assistant-working-same-pace".to_owned(),
            steps: vec![
                average.clone(),
                step(
                    WorkingOp::Times,
                    vec![money(48_000), Cell::Number { value: ahead }],
                    money(pace_total),
                ),
                books_sum_step(),
                step(
                    WorkingOp::Sum,
                    vec![money(261_000), money(pace_total)],
                    money(405_000),
                ),
            ],
        },
    };
    let what_if = EstimateMethod {
        kind: EstimateMethodKind::WhatIfPace,
        months_used: 3,
        used_from: month(2026, 7),
        used_to: month(2026, 9),
        per_month: Some(money(96_000)),
        step: None,
        step_dir: None,
        total: money(doubled_total),
        target_month: None,
        factor: Some(Cell::Percent { hundredths: 20_000 }),
        year_total: Some(money(549_000)),
        working: AnswerWorking {
            title_id: "assistant-working-what-if-pace".to_owned(),
            steps: vec![
                average,
                step(
                    WorkingOp::Times,
                    vec![money(48_000), Cell::Percent { hundredths: 20_000 }],
                    money(96_000),
                ),
                step(
                    WorkingOp::Times,
                    vec![money(96_000), Cell::Number { value: ahead }],
                    money(doubled_total),
                ),
                books_sum_step(),
                step(
                    WorkingOp::Sum,
                    vec![money(261_000), money(doubled_total)],
                    money(549_000),
                ),
            ],
        },
    };
    let months: Vec<AnswerMonth> = (4..=12).map(|m| month(2026, m)).collect();
    let past: Vec<Cell> = [36_000, 39_000, 42_000, 45_000, 48_000, 51_000]
        .iter()
        .map(|rupees| money(*rupees))
        .collect();
    let gap = |line: &[i64]| -> Vec<Cell> {
        std::iter::repeat_n(Cell::Empty, 6)
            .chain(line.iter().map(|rupees| money(*rupees)))
            .collect()
    };
    let actual: Vec<Cell> = past
        .into_iter()
        .chain(std::iter::repeat_n(Cell::Empty, 3))
        .collect();
    let mut a = answer(vec![
        text("پورے سال کا کل اسی رفتار سے تقریباً "),
        figure(money(405_000), false),
        text(" بنے گا۔ اگر آنے والے مہینوں کی رفتار "),
        AnswerPart::Figure {
            cell: Cell::Percent { hundredths: 20_000 },
            dir: None,
            checked: false,
            working: None,
        },
        text(" ہو تو "),
        figure(money(549_000), false),
        text("۔"),
    ]);
    a.charts = vec![AnswerChart {
        result_id: "e1".to_owned(),
        measure_id: "assistant-measure-commission-all".to_owned(),
        months,
        series: vec![
            ChartSeries {
                kind: ChartSeriesKind::Actual,
                values: actual,
            },
            ChartSeries {
                kind: ChartSeriesKind::SamePace,
                values: gap(&[48_000, 48_000, 48_000]),
            },
            ChartSeries {
                kind: ChartSeriesKind::WhatIfPace,
                values: gap(&[96_000, 96_000, 96_000]),
            },
        ],
        is_estimate: true,
    }];
    a.estimate = Some(AnswerEstimate {
        result_id: "e1".to_owned(),
        measure_id: "assistant-measure-commission-all".to_owned(),
        full_months: 6,
        first: month(2026, 10),
        last: month(2026, 12),
        target: None,
        methods: vec![pace, what_if],
        year: Some(books_so_far()),
    });
    let mut r = reply(6, AskOutcome::Estimate);
    r.answer = Some(a);
    r
}

/// E2E-63 (today 15-11-26): advice from R20, under the advice band.
fn advice() -> AskReply {
    let mut working = AnswerWorking {
        title_id: "assistant-working-sum".to_owned(),
        steps: Vec::new(),
    };
    working.steps.push(WorkingStep {
        op: WorkingOp::Sum,
        inputs: vec![dr(18_684), dr(2_880)],
        result: dr(21_564),
    });
    let mut a = answer(vec![
        text("پہلے "),
        AnswerPart::Name { name: arshad() },
        text(" سے وصولی کریں: ان کے ذمے "),
        figure(dr(18_684), true),
        text(" باقی ہیں اور ان کا بل "),
        figure(Cell::Number { value: 36 }, false),
        text(" دن پرانا ہے۔ پھر "),
        AnswerPart::Name { name: akram() },
        text(" سے: ان کے ذمے "),
        figure(dr(2_880), true),
        text(" ہیں۔ دونوں مل کر "),
        AnswerPart::Figure {
            cell: dr(21_564),
            dir: None,
            checked: true,
            working: Some(working),
        },
        text(" بنتے ہیں۔"),
    ]);
    a.advice = true;
    a.tables = vec![r20_table()];
    a.sources = vec![AnswerSource::Report {
        result_id: "r2".to_owned(),
        label_id: "report-r20-title".to_owned(),
        id: ReportId::R20,
        filters: SourceReportFilters {
            as_of: "2026-11-15".to_owned(),
            ..SourceReportFilters::default()
        },
    }];
    a.next = vec!["اکرم ٹول ٹیکس کا کھاتہ".to_owned()];
    a.read_at = "2026-11-15T09:30:00Z".to_owned();
    let mut r = reply(6, AskOutcome::Advice);
    r.answer = Some(a);
    r
}

/// General advice (plan 52 item 11, PRD FR-7.88 v1.19): "sale kaise barhayen" when the books have
/// too little to answer. The band "عام مشورہ" shows instead of the advice band; the answer has no
/// figure of its own; `advice` is true too.
fn general_advice() -> AskReply {
    let mut a = answer(vec![text(
        "ریٹ مناسب اور مستقل رکھیں۔\n• پرانے بقایا کی وصولی ہر ہفتے کریں۔\n• جن گاہکوں کا بل پرانا ہو انہیں فون کریں۔\n• بیوپاریوں کی ادائیگی وقت پر کریں۔\n• ہر بکری اسی دن درج کریں۔",
    )]);
    a.advice = true;
    a.general_advice = true;
    let mut r = reply(8, AskOutcome::Advice);
    r.answer = Some(a);
    r
}

/// E82: "akram" fits two accounts.
fn which_one() -> AskReply {
    let mut r = reply(7, AskOutcome::WhichOne);
    r.options = vec![
        AnswerOption {
            key: "a20".to_owned(),
            name: akram(),
            city: Some(Names {
                ur: "کوٹ رادھا کشن".to_owned(),
                en: "Kot Radha Kishan".to_owned(),
            }),
            balance: Some(dr(2_880)),
            checked: true,
            dead: false,
        },
        AnswerOption {
            key: "a22".to_owned(),
            name: akram_k(),
            city: None,
            balance: Some(Cell::Balance {
                paisa: rs(0),
                side: BalanceSide::Zero,
            }),
            checked: true,
            dead: false,
        },
    ];
    r
}

/// The name is not found: the three closest names.
fn not_found() -> AskReply {
    let mut r = reply(8, AskOutcome::NotFound);
    r.options = vec![
        AnswerOption {
            key: "a20".to_owned(),
            name: akram(),
            city: None,
            balance: Some(dr(2_880)),
            checked: true,
            dead: false,
        },
        AnswerOption {
            key: "a22".to_owned(),
            name: akram_k(),
            city: None,
            balance: None,
            checked: false,
            dead: false,
        },
    ];
    r
}

/// "akram ki 5000 wasooli likh do": the assistant only tells (E2E-53, FR-7.79).
fn change_refused() -> AskReply {
    let mut r = reply(9, AskOutcome::ChangeRefused);
    r.entry_route = Some(EntryRoute {
        screen: EntryScreen::Wasooli,
        party: Some(record(&akram(), 20)),
    });
    r
}

/// An answer to a question that also asks for a change (PRD E98): the figure is answered and the
/// panel adds the change refused line and its button, with Akram's record.
fn answered_route() -> AskReply {
    let mut r = answered();
    r.entry_route = Some(EntryRoute {
        screen: EntryScreen::Wasooli,
        party: Some(record(&akram(), 20)),
    });
    r
}

/// A help answer (PRD FR-7.89, plan 51 B3): the guide's own words as text parts, and a button for
/// the screen it is about. A main screen opens as it is, so the party is null.
fn help_route() -> AskReply {
    let mut r = reply(21, AskOutcome::Answered);
    r.answer = Some(answer(vec![
        AnswerPart::Text {
            text: "A market trip is a day trip: a person is sent to a bigger market to buy goods for some partners.".to_owned(),
        },
        AnswerPart::Text {
            text: "\n• Open Market Trips and make a new trip.".to_owned(),
        },
    ]));
    r.entry_route = Some(EntryRoute {
        screen: EntryScreen::MarketTrips,
        party: None,
    });
    r
}

/// Advice to a question that also asks for a change (PRD E98): the same route.
fn advice_route() -> AskReply {
    let mut r = advice();
    r.entry_route = Some(EntryRoute {
        screen: EntryScreen::Wasooli,
        party: Some(record(&akram(), 20)),
    });
    r
}

/// The question is outside the books; the nearest source is a button.
fn outside_books() -> AskReply {
    let mut a = answer(Vec::new());
    a.sources = vec![AnswerSource::Report {
        result_id: "r1".to_owned(),
        label_id: "report-r20-title".to_owned(),
        id: ReportId::R20,
        filters: SourceReportFilters::default(),
    }];
    let mut r = reply(10, AskOutcome::OutsideBooks);
    r.answer = Some(a);
    r
}

/// The answer failed its checks twice: the tool results as plain tables (E2E-60).
fn checks_failed() -> AskReply {
    let mut a = answer(Vec::new());
    a.tables = vec![r13_table()];
    a.sources = vec![card_source("r2", 20, akram()), r13_source()];
    let mut r = reply(11, AskOutcome::ChecksFailed);
    r.answer = Some(a);
    r
}

/// A limit stopped the question before it read anything to show (plan 51 B2): the panel says the
/// books could not be read in the time allowed, not "I did not understand".
fn stopped_calls() -> AskReply {
    let mut r = reply(19, AskOutcome::NotUnderstood);
    r.stop_reason = Some(StopReason::Calls);
    r
}

/// A limit stopped the question after it read a table: the table is shown as it is (plan 51 B2).
fn stopped_tokens_with_table() -> AskReply {
    let mut a = answer(Vec::new());
    a.tables = vec![r13_table()];
    a.sources = vec![card_source("r2", 20, akram()), r13_source()];
    let mut r = reply(20, AskOutcome::NotUnderstood);
    r.stop_reason = Some(StopReason::Tokens);
    r.answer = Some(a);
    r
}

/// A figure's two places differ (FR-7.76): the party card says 2,880, R13 says 2,900.
fn records_differ() -> AskReply {
    let mut a = answer(Vec::new());
    a.sources = vec![card_source("r2", 20, akram()), r13_source()];
    a.differences = vec![FigureDifference {
        figure_id: "assistant-figure-party-balance".to_owned(),
        name: Some(akram()),
        places: vec![
            PlaceValue {
                place_id: "assistant-place-party-card".to_owned(),
                cell: dr(2_880),
            },
            PlaceValue {
                place_id: "assistant-place-r13-closing".to_owned(),
                cell: dr(2_900),
            },
        ],
    }];
    let mut r = reply(12, AskOutcome::RecordsDiffer);
    r.answer = Some(a);
    r
}

/// A partner figure is needed and the section is locked (E2E-50).
fn locked() -> AskReply {
    let mut r = reply(13, AskOutcome::Locked);
    r.locked = true;
    r
}

fn limit_reached() -> AskReply {
    let mut r = reply(20, AskOutcome::LimitReached);
    r.count_today = DAILY_LIMIT;
    r
}

fn offline() -> AskReply {
    let mut r = reply(14, AskOutcome::Offline);
    r.failure = Some(AssistantFailure {
        kind: FailureKind::Offline,
        http_status: None,
    });
    r
}

/// Azure has no endpoint for the model: HTTP 404 (Section 0.2 of D6).
fn error_404() -> AskReply {
    let mut r = reply(15, AskOutcome::Error);
    r.failure = Some(AssistantFailure {
        kind: FailureKind::NoEndpoint,
        http_status: Some(404),
    });
    r
}

fn status(enabled: bool) -> AssistantStatus {
    AssistantStatus {
        enabled,
        key_saved: enabled,
        key_last4: enabled.then(|| "ab12".to_owned()),
        model: DEFAULT_MODEL.to_owned(),
        default_model: DEFAULT_MODEL.to_owned(),
        count_today: if enabled { 9 } else { 0 },
        limit: DAILY_LIMIT,
    }
}

fn log_page() -> AssistantLogPage {
    let tool = |name: &str, source: Option<&str>| LogTool {
        tool: name.to_owned(),
        source: source.map(str::to_owned),
    };
    let base = |id: i64, source: LogSource, outcome: AskOutcome, counted: bool| AssistantLogRow {
        id,
        asked_at: "2026-10-10T13:12:00Z".to_owned(),
        asked_day: "2026-10-10".to_owned(),
        question: String::new(),
        source,
        outcome,
        counted,
        parent_id: None,
        tools: Vec::new(),
        model: None,
        model_calls: None,
        tokens_in: None,
        tokens_cached: None,
        tokens_out: None,
        tokens_reasoning: None,
        total_millis: None,
        model_millis: None,
        tool_millis: None,
        can_rerun: false,
    };
    let mut model_row = base(1, LogSource::Model, AskOutcome::WhichOne, true);
    model_row.question = "akram ka hisab".to_owned();
    model_row.tools = vec![tool("find", None)];
    model_row.model = Some(DEFAULT_MODEL.to_owned());
    model_row.model_calls = Some(1);
    model_row.tokens_in = Some(9_200);
    model_row.tokens_cached = Some(7_600);
    model_row.tokens_out = Some(180);
    model_row.tokens_reasoning = Some(60);
    model_row.total_millis = Some(6_400);
    model_row.model_millis = Some(6_350);
    model_row.tool_millis = Some(4);
    model_row.can_rerun = true;
    let mut chip_row = base(2, LogSource::Chip, AskOutcome::Answered, false);
    chip_row.question = "cash_in_hand".to_owned();
    chip_row.tools = vec![tool("tiles", None)];
    chip_row.total_millis = Some(12);
    chip_row.can_rerun = true;
    let mut pick_row = base(3, LogSource::Continue, AskOutcome::Answered, false);
    pick_row.question = "akram ka hisab".to_owned();
    pick_row.parent_id = Some(1);
    pick_row.tools = vec![tool("party_card", None), tool("report", Some("R13"))];
    pick_row.model = Some(DEFAULT_MODEL.to_owned());
    pick_row.model_calls = Some(2);
    pick_row.tokens_in = Some(17_000);
    pick_row.tokens_cached = Some(15_200);
    pick_row.tokens_out = Some(900);
    pick_row.tokens_reasoning = Some(300);
    pick_row.can_rerun = true;
    AssistantLogPage {
        rows: vec![pick_row, chip_row, model_row],
        offset: 0,
        limit: 100,
        total_count: 3,
    }
}

// ---- the test -------------------------------------------------------------------------------

#[test]
fn contract_fixtures_are_the_contract() {
    let ask = |text: &str| json!({ "text": text, "conversation_id": null });
    let ask_request = |name: &str, request: &Value| {
        // The request is read back into the real input type, so it can never be wrong.
        let parsed: AssistantAskInput = serde_json::from_value(request.clone()).unwrap();
        assert!(!parsed.text.is_empty(), "{name}");
    };

    let asks: Vec<(&str, Value, AskReply)> = vec![
        (
            "assistant_reply_answered",
            ask("ikram ke total bakaya kitne hai"),
            answered(),
        ),
        (
            "assistant_reply_answered_sum",
            ask("akram aur arshad dono ka mila kar kitna hai"),
            answered_sum(),
        ),
        (
            "assistant_reply_answered_chart",
            ask("is mahine ka commission pichle mahine se zyada hai"),
            answered_chart(),
        ),
        (
            "assistant_reply_estimate",
            ask("agar commission isi raftar se chalta raha to agle 6 mahine mein kitna hoga"),
            estimate_reply(false),
        ),
        (
            "assistant_reply_estimate_target",
            ask("3 lakh ka target kab tak pura hoga"),
            estimate_reply(true),
        ),
        (
            "assistant_reply_estimate_year_end",
            ask("saal ke aakhir tak commission kitna hoga"),
            year_end_reply(),
        ),
        (
            "assistant_reply_estimate_what_if",
            ask("is raftar se saal ke aakhir tak commission kitna, aur agar raftar double ho to"),
            what_if_reply(),
        ),
        (
            "assistant_reply_general_advice",
            ask("sale kaise barhayen"),
            general_advice(),
        ),
        (
            "assistant_reply_advice",
            ask("kis se pehle wasooli karun"),
            advice(),
        ),
        (
            "assistant_reply_which_one",
            ask("akram ka hisab"),
            which_one(),
        ),
        (
            "assistant_reply_not_found",
            ask("akrm ka hisab"),
            not_found(),
        ),
        (
            "assistant_reply_need_dates",
            ask("pichle teen hafte ki bikri"),
            reply(16, AskOutcome::NeedDates),
        ),
        (
            "assistant_reply_need_party",
            ask("kitna lena hai"),
            reply(17, AskOutcome::NeedParty),
        ),
        (
            "assistant_reply_outside_books",
            ask("kal lahore mein seb ka rate kya hoga"),
            outside_books(),
        ),
        (
            "assistant_reply_change_refused",
            ask("akram ki 5000 wasooli likh do"),
            change_refused(),
        ),
        (
            "assistant_reply_answered_route",
            ask("akram ka bakaya batao aur 5000 ki wasooli likh do"),
            answered_route(),
        ),
        (
            "assistant_reply_help_route",
            ask("market safar kya hai"),
            help_route(),
        ),
        (
            "assistant_reply_advice_route",
            ask("kis se pehle wasooli karun aur akram ki wasooli likh do"),
            advice_route(),
        ),
        (
            "assistant_reply_not_understood",
            ask("xyz abc"),
            reply(18, AskOutcome::NotUnderstood),
        ),
        (
            "assistant_reply_not_understood_calls",
            ask("apni sale ko ziyada kaise karain ?"),
            stopped_calls(),
        ),
        (
            "assistant_reply_not_understood_tokens_table",
            ask("ikram ke total bakaya kitne hai"),
            stopped_tokens_with_table(),
        ),
        (
            "assistant_reply_checks_failed",
            ask("ikram ke total bakaya kitne hai"),
            checks_failed(),
        ),
        (
            "assistant_reply_records_differ",
            ask("ikram ke total bakaya kitne hai"),
            records_differ(),
        ),
        (
            "assistant_reply_locked",
            ask("partner ki kiya total hain abhi tak"),
            locked(),
        ),
        (
            "assistant_reply_limit_reached",
            ask("aaj ki bikri aur commission"),
            limit_reached(),
        ),
        (
            "assistant_reply_offline",
            ask("aaj ki bikri aur commission"),
            offline(),
        ),
        (
            "assistant_reply_error_404",
            ask("aaj ki bikri aur commission"),
            error_404(),
        ),
    ];
    for (name, request, reply) in &asks {
        ask_request(name, request);
        pair(name, request, reply);
    }

    // A chip, its reply, and a chip that needs a party.
    let chip = json!({ "chip": "cash_in_hand", "party_id": null, "conversation_id": null });
    let _: AssistantQuickInput = serde_json::from_value(chip.clone()).unwrap();
    let mut cash = reply(19, AskOutcome::Answered);
    cash.answer = Some(answer(vec![
        text("موجود روکڑ "),
        figure(money(58_484), true),
        text(" ہے۔"),
    ]));
    pair("assistant_quick_cash_in_hand", &chip, &cash);

    // The continuation after "کون سا؟", and after the partner code.
    let pick = json!({ "question_id": 7, "choice": { "type": "match", "key": "a20" } });
    let _: AssistantPickInput = serde_json::from_value(pick.clone()).unwrap();
    pair("assistant_pick_match", &pick, &answered());
    let pick_code = json!({ "question_id": 13, "choice": { "type": "partner_code" } });
    let _: AssistantPickInput = serde_json::from_value(pick_code.clone()).unwrap();
    pair("assistant_pick_partner_code", &pick_code, &answered());
    let rerun = json!({ "question_id": 1 });
    let _: AssistantRerunInput = serde_json::from_value(rerun.clone()).unwrap();
    pair("assistant_rerun", &rerun, &checks_failed());

    // Status, settings, key, test, log, conversation and the step event.
    reply_only("assistant_status_on", &status(true));
    reply_only("assistant_status_off", &status(false));
    let settings = json!({ "enabled": true, "model": DEFAULT_MODEL });
    let _: AssistantSettingsSetInput = serde_json::from_value(settings.clone()).unwrap();
    pair("assistant_settings_set", &settings, &status(true));
    // The key in the request file is a placeholder word, never a real key.
    let key_save = json!({ "key": "placeholder-not-a-key", "password": "placeholder" });
    let _: AssistantKeySaveInput = serde_json::from_value(key_save.clone()).unwrap();
    pair("assistant_key_save", &key_save, &status(true));
    let key_remove = json!({ "password": "placeholder" });
    let _: AssistantKeyRemoveInput = serde_json::from_value(key_remove.clone()).unwrap();
    pair("assistant_key_remove", &key_remove, &status(false));
    reply_only(
        "assistant_test_worked",
        &AssistantTestResult {
            worked: true,
            millis: 2_340,
            model: DEFAULT_MODEL.to_owned(),
            failure: None,
            count_today: 10,
            limit: DAILY_LIMIT,
        },
    );
    reply_only(
        "assistant_test_no_endpoint",
        &AssistantTestResult {
            worked: false,
            millis: 410,
            model: DEFAULT_MODEL.to_owned(),
            failure: Some(AssistantFailure {
                kind: FailureKind::NoEndpoint,
                http_status: Some(404),
            }),
            count_today: 10,
            limit: DAILY_LIMIT,
        },
    );
    let list = json!({ "offset": 0, "limit": 100 });
    let _: AssistantLogListInput = serde_json::from_value(list.clone()).unwrap();
    pair("assistant_log_list_first_page", &list, &log_page());
    let started = json!({ "conversation_id": "c-1" });
    let _: AssistantNewConversationInput = serde_json::from_value(started.clone()).unwrap();
    let _: AssistantNewConversationInput =
        serde_json::from_value(json!({ "conversation_id": null })).unwrap();
    pair(
        "assistant_new_conversation",
        &started,
        &ConversationStarted {
            conversation_id: "c-2".to_owned(),
        },
    );
    reply_only(
        "assistant_step_reading_report",
        &AssistantStep {
            id: "assistant-step-reading-report".to_owned(),
            args: vec![
                MessageArg {
                    name: "report".to_owned(),
                    value: ArgValue::Text {
                        text: "R13".to_owned(),
                    },
                },
                MessageArg {
                    name: "party".to_owned(),
                    value: ArgValue::Names {
                        names: Names {
                            ur: "اکرم ٹول ٹیکس".to_owned(),
                            en: "Akram Toll Tax".to_owned(),
                        },
                    },
                },
            ],
        },
    );
}

/// The shape rules the screens rely on, checked on the fixtures' source values.
#[test]
fn the_replies_keep_the_contract_rules() {
    // Every reply names today's count against the fixed limit of 20.
    for r in [answered(), which_one(), locked(), offline(), error_404()] {
        assert_eq!(r.limit, 20);
        assert!(r.count_today <= r.limit);
    }
    // At most 7 rows in a table, at most 5 options, at most 3 next questions.
    for r in [
        answered(),
        advice(),
        checks_failed(),
        stopped_tokens_with_table(),
    ] {
        let a = r.answer.unwrap();
        for table in &a.tables {
            assert!(table.rows.len() <= mandi_assistant::limits::MAX_TABLE_ROWS);
            for row in &table.rows {
                assert_eq!(row.row.cells.len(), table.columns.len());
            }
            assert_eq!(table.totals.len(), table.columns.len());
        }
        assert!(a.next.len() <= mandi_assistant::limits::MAX_NEXT);
    }
    assert!(which_one().options.len() <= mandi_assistant::limits::MAX_OPTIONS);
    // The estimate band shows only when an estimate is there; advice only for advice.
    assert!(estimate_reply(false).answer.unwrap().estimate.is_some());
    assert!(answered().answer.unwrap().estimate.is_none());
    assert!(advice().answer.unwrap().advice);
    assert!(!answered().answer.unwrap().advice);
    // E2E-59 and E2E-63 numbers.
    let e = estimate_reply(false).answer.unwrap().estimate.unwrap();
    assert_eq!(e.methods.len(), 2);
    assert_eq!(e.methods[0].total, money(288_000));
    assert_eq!(e.methods[1].total, money(369_000));
    assert_eq!((e.first, e.last), (month(2026, 10), month(2027, 3)));
    let t = estimate_reply(true).answer.unwrap().estimate.unwrap();
    assert_eq!((t.first, t.last), (month(2026, 10), month(2027, 9)));
    assert_eq!(t.methods[0].target_month, Some(month(2027, 4)));
    assert_eq!(t.methods[1].target_month, Some(month(2027, 2)));
    // The worked day: 2,880 + 18,684 = 21,564.
    assert_eq!(
        rs(2_880).as_paisa() + rs(18_684).as_paisa(),
        rs(21_564).as_paisa()
    );
    // A key is never printed: Debug hides the key and the password.
    let input = AssistantKeySaveInput {
        key: "placeholder-not-a-key".to_owned(),
        password: "placeholder".to_owned(),
    };
    let shown = format!("{input:?}");
    assert!(!shown.contains("placeholder"), "{shown}");
}

/// The filters of a source carry whole records and turn back into the engine's own filters
/// (lead decision, plan 48 T1).
#[test]
fn a_source_carries_whole_records_and_turns_back_into_ids() {
    let filters = SourceReportFilters {
        party: Some(SourceParty {
            record: record(&akram(), 20),
            person_key: Some("shahid|0300".to_owned()),
        }),
        seller: Some(record(&arshad(), 21)),
        item: Some(AnswerRecord {
            id: 5,
            code: Some("0005".to_owned()),
            name_ur: "کیلا".to_owned(),
            name_en: "Banana".to_owned(),
            tag: None,
        }),
        bikri: Some(mandi_assistant::source::SourceBikri { id: 77, number: 1 }),
        guests_only: true,
        ..SourceReportFilters::default()
    };
    let engine = filters.to_filters();
    let party = engine.party.unwrap();
    assert_eq!(party.account_id, 20);
    assert_eq!(party.person_key.as_deref(), Some("shahid|0300"));
    assert_eq!(engine.seller_id, Some(21));
    assert_eq!(engine.item_id, Some(5));
    assert_eq!(engine.city_id, None);
    assert_eq!(engine.bikri_id, Some(77));
    assert_eq!(engine.guests_only, Some(true));
    // The JSON of a source holds the whole record, with both names and the tag.
    let json = serde_json::to_value(&filters).unwrap();
    assert_eq!(json["party"]["record"]["name_en"], "Akram Toll Tax");
    assert_eq!(json["party"]["record"]["code"], "0020");
    assert_eq!(json["seller"]["id"], 21);
    assert_eq!(json["item"]["name_ur"], "کیلا");
    assert!(json["party"]["record"]["tag"].is_null());

    let view = SourceAnalyticsFilters {
        buyer: Some(SourceParty {
            record: record(&akram_k(), 22),
            person_key: None,
        }),
        trip: Some(AnswerRecord {
            id: 3,
            code: None,
            name_ur: "سفر".to_owned(),
            name_en: "Trip".to_owned(),
            tag: None,
        }),
        ..SourceAnalyticsFilters::default()
    };
    let engine = view.to_filters();
    assert_eq!(engine.buyer.unwrap().account_id, 22);
    assert_eq!(engine.trip_id, Some(3));
    assert_eq!(engine.item_id, None);
    let json = serde_json::to_value(&view).unwrap();
    assert_eq!(json["buyer"]["record"]["tag"], "K");
}
