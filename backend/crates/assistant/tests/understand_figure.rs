//! Plan 55 Section 10, PRD FR-7.90: the instant answer for a simple figure (`compose_figure`).
//!
//! Each question is asked of the real books (the PRD worked day, today 10-10-2026), understood,
//! planned (`plan_reads`), read through the REAL tools one after the other as the loop would, and
//! composed. The composed answer is then run through the real checks (`Check::run`, PRD FR-7.76) and
//! filled (`fill::fill`), and the test reads the sentence, the figures (each a checked reference
//! into the results) and the buttons from the filled answer. Nothing here calls a model.
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

use mandi_assistant::answer::{Answer, AnswerPart, AnswerSource};
use mandi_assistant::check::{Check, NameSet, Verdict};
use mandi_assistant::fill::fill;
use mandi_assistant::model_answer::{Piece, pieces};
use mandi_assistant::results::{ToolResult, Words, fit, tool_result_json};
use mandi_assistant::tools::{Env, State, Stop, call};
use mandi_assistant::understand::{
    FigureAnswer, Reader, Sentence, compose_figure, plan_reads, understand,
};
use mandi_reports::dashboard::BackupFacts;
use mandi_reports::reports::Cell;
use mandi_services::core::accounts::AccountType;
use mandi_services::core::auth::Language;
use mandi_services::core::dates::BusinessDate;
use mandi_services::core::masters::{AddaInput, CityInput, MasterInput};
use mandi_services::domain::figures::question_numbers;

fn books() -> world::World {
    let w = worked::worked_day();
    let add = |kind, ur: &str, en: &str| {
        world::add_account(&w.env, &w.owner, kind, ur, en, true, "");
    };
    add(AccountType::Buyer, "فیصل", "Faisal");
    worked::expense_head(&w, "بجلی", "Electricity");
    let city = save(&w, |words| {
        MasterInput::City(CityInput {
            id: None,
            version: None,
            code: String::new(),
            name_ur: "لاہور".into(),
            name_en: "Lahore".into(),
            confirmed_rules_words: words.to_vec(),
        })
    });
    save(&w, |words| {
        MasterInput::Adda(AddaInput {
            id: None,
            version: None,
            code: String::new(),
            name_ur: "سوات اڈا".into(),
            name_en: "Swat Adda".into(),
            city_id: Some(city),
            confirmed_rules_words: words.to_vec(),
        })
    });
    w
}

fn save(w: &world::World, input: impl Fn(&[String]) -> MasterInput) -> i64 {
    use mandi_services::core::error::{ErrorCode, ErrorValue};
    let mut words: Vec<String> = Vec::new();
    loop {
        match mandi_services::masters::save(&w.env.ctx, &w.owner, &input(&words)) {
            Ok(row) => return row.id(),
            Err(mandi_services::ServiceError::Rejected(items))
                if items
                    .iter()
                    .all(|i| i.code == ErrorCode::ErrSpellingUnconfirmed) =>
            {
                for item in items {
                    if let Some(ErrorValue::Text(text)) = item.values.get("words") {
                        words.extend(text.split(' ').map(str::to_owned));
                    }
                }
            }
            Err(other) => panic!("{other:?}"),
        }
    }
}

fn on(y: i32, m: u32, d: u32) -> BusinessDate {
    BusinessDate::from_ymd(y, m, d).expect("a date")
}

fn reader<'a>(w: &'a world::World, today: BusinessDate, language: Language) -> Reader<'a> {
    Reader {
        ctx: &w.env.ctx,
        actor: &w.owner,
        language,
        today,
        partner_locked: false,
    }
}

/// Reads the planned reads of a question through the real tools, in order, and gives the state the
/// checks and the filling read. `None` when the question plans no read.
fn read_state(
    w: &world::World,
    today: BusinessDate,
    language: Language,
    question: &str,
) -> Option<(mandi_assistant::understand::Understanding, State)> {
    let u = understand(&reader(w, today, language), question);
    let reads = plan_reads(&u)?;
    let words = Words::new().expect("the words");
    let numbers = question_numbers(question);
    let mut state = State::default();
    for read in &reads {
        let env = Env {
            ctx: &w.env.ctx,
            actor: &w.owner,
            words: &words,
            language,
            today,
            partner_locked: false,
            numbers: &numbers,
            question,
            backup: BackupFacts {
                cloud_on: false,
                cloud_age_hours: None,
            },
        };
        match call(&env, &state, read.tool, &read.input.to_string()) {
            Ok(draft) => {
                let prefix = if draft.tool == "calculate" { 'c' } else { 'r' };
                let id = state.next_id(prefix);
                let mut result = ToolResult {
                    id,
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
                fit(&mut result, &words);
                state.results.push(result);
            }
            Err(Stop::Error(_) | Stop::Locked) => {}
        }
    }
    Some((u, state))
}

/// What an instant answer is, as one line each: the sentences (id without its prefix, the selector
/// arguments, and the period shape), the references of the text in order, the figures of the filled
/// answer (rupees, the side `dr` or `cr`, a tick when checked) and the buttons.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Seen {
    sentences: String,
    refs: String,
    figures: String,
    buttons: String,
}

fn sentence_line(s: &Sentence) -> String {
    let selectors: Vec<String> = s
        .args
        .iter()
        .filter(|(name, value)| *name != "period" && !value.starts_with("{{"))
        .map(|(name, value)| format!("{name}={value}"))
        .collect();
    let period = s
        .period
        .as_deref()
        .and_then(|p| p.args.iter().find(|(name, _)| *name == "shape"))
        .map(|(_, shape)| format!("period={shape}"));
    let all: Vec<String> = selectors.into_iter().chain(period).collect();
    format!(
        "{}[{}]",
        s.id.trim_start_matches("assistant-instant-"),
        all.join(",")
    )
}

fn figure_line(part: &AnswerPart) -> Option<String> {
    let AnswerPart::Figure { cell, checked, .. } = part else {
        return None;
    };
    let tick = if *checked { "✓" } else { "" };
    Some(match cell {
        Cell::Money { paisa } => format!("{}{tick}", paisa.rupees()),
        Cell::Balance { paisa, side } => {
            let side = format!("{side:?}").to_lowercase();
            let side = if side == "zero" { String::new() } else { side };
            format!("{}{side}{tick}", paisa.rupees())
        }
        Cell::Number { value } => format!("n{value}{tick}"),
        other => format!("{other:?}"),
    })
}

fn button_line(source: &AnswerSource) -> String {
    match source {
        AnswerSource::PartyCard { .. } => "party_card".to_owned(),
        AnswerSource::Report { id, .. } => serde_json::to_value(id)
            .ok()
            .and_then(|v| v.as_str().map(str::to_owned))
            .unwrap_or_default(),
        AnswerSource::Analytics { view, .. } => serde_json::to_value(view)
            .ok()
            .and_then(|v| v.as_str().map(|s| format!("view {s}")))
            .unwrap_or_default(),
        AnswerSource::Tiles { .. } => "tiles".to_owned(),
        AnswerSource::Screen { route, .. } => format!("screen {route:?}").to_lowercase(),
    }
}

/// The question through everything the loop does before a model is called: understood, planned,
/// read by the real tools, composed, checked by the real checks and filled. `Err` says where it
/// stopped; `Ok(None)` is a question the app does not answer by itself.
fn instant(
    w: &world::World,
    today: BusinessDate,
    language: Language,
    question: &str,
) -> Result<Option<(FigureAnswer, Answer)>, String> {
    let Some((u, state)) = read_state(w, today, language, question) else {
        return Ok(None);
    };
    let words = Words::new().expect("the words");
    let Some(figure) = compose_figure(&u, &state.results, language, &words) else {
        return Ok(None);
    };
    let numbers = question_numbers(question);
    let env = Env {
        ctx: &w.env.ctx,
        actor: &w.owner,
        words: &words,
        language,
        today,
        partner_locked: false,
        numbers: &numbers,
        question,
        backup: BackupFacts {
            cloud_on: false,
            cloud_age_hours: None,
        },
    };
    let names = NameSet::new(&state, &[]);
    let verdict = Check {
        env: &env,
        state: &state,
        names: &names,
    }
    .run(&figure.answer);
    if verdict != Verdict::Ok {
        return Err(format!(
            "{question}: the checks said {verdict:?} to {:?}",
            figure.answer.text
        ));
    }
    let answer = fill(&env, &state, &figure.answer, "2026-10-10T08:00:00Z");
    Ok(Some((figure, answer)))
}

fn seen_of(figure: &FigureAnswer, answer: &Answer) -> Seen {
    Seen {
        sentences: figure
            .sentences
            .iter()
            .map(sentence_line)
            .collect::<Vec<_>>()
            .join(" + "),
        refs: pieces(&figure.answer.text)
            .into_iter()
            .filter_map(|p| match p {
                Piece::Ref(r) => Some(r.path),
                Piece::Words(_) => None,
            })
            .collect::<Vec<_>>()
            .join(" "),
        figures: answer
            .parts
            .iter()
            .filter_map(figure_line)
            .collect::<Vec<_>>()
            .join(" "),
        buttons: answer
            .sources
            .iter()
            .map(button_line)
            .collect::<Vec<_>>()
            .join(" "),
    }
}

/// One line for each question of the file in `SURVEY`, as the case tables write them. A tool for the
/// person who builds, not a check: `SURVEY=/path cargo test -p mandi-assistant --test understand_figure
/// -- --ignored --nocapture survey`. `WORLD=year` asks the thin year; `LANG=ur` the Urdu app.
#[test]
#[ignore = "a tool for the person who builds, not a check"]
fn survey() {
    let Ok(file) = std::env::var("SURVEY") else {
        return;
    };
    let language = if std::env::var("LANG").as_deref() == Ok("ur") {
        Language::Ur
    } else {
        Language::En
    };
    let text = std::fs::read_to_string(file).expect("the file");
    let year_world = matches!(std::env::var("WORLD").as_deref(), Ok("year" | "full"));
    let (w, today);
    let y;
    if year_world {
        y = if std::env::var("WORLD").as_deref() == Ok("full") {
            year::seed::build()
        } else {
            year::seed::build_thin()
        };
        w = &y.w;
        today = on(2026, 11, 5);
    } else {
        let b = Box::leak(Box::new(books()));
        worked::events_1_to_5(b);
        w = b;
        today = on(2026, 10, 10);
    }
    for question in text.lines().filter(|l| !l.trim().is_empty()) {
        match instant(w, today, language, question) {
            Err(e) => println!("{question:48} => ERROR {e}"),
            Ok(None) => println!("{question:48} => NONE"),
            Ok(Some((figure, answer))) => {
                let seen = seen_of(&figure, &answer);
                println!(
                    "{question:48} => {} | {} | {} | {}\n      {}",
                    seen.sentences,
                    seen.refs,
                    seen.figures,
                    seen.buttons,
                    figure.answer.text.replace('\n', " / ")
                );
            }
        }
    }
}

/// One line for each question of the file in `READS`: the results the reads gave, as the model
/// would see them. A tool for the person who builds, not a check:
/// `READS=/path cargo test -p mandi-assistant --test understand_figure -- --ignored --nocapture dump`.
#[test]
#[ignore = "a tool for the person who builds, not a check"]
fn dump() {
    let Ok(file) = std::env::var("READS") else {
        return;
    };
    let text = std::fs::read_to_string(file).expect("the file");
    let w = books();
    worked::events_1_to_5(&w);
    let words = Words::new().expect("the words");
    for question in text.lines().filter(|l| !l.trim().is_empty()) {
        println!("=== {question}");
        match read_state(&w, on(2026, 10, 10), Language::En, question) {
            None => println!("(no reads)"),
            Some((_, state)) => {
                for r in &state.results {
                    let json = tool_result_json(r, &words).to_string();
                    println!("{}", json.chars().take(1800).collect::<String>());
                }
            }
        }
    }
}

// ---- the cases -----------------------------------------------------------------------------------------

/// (question, sentences, references, figures, buttons). Checked by hand against the PRD worked day
/// (Section 4.9, E2E-8, E2E-10, E2E-37): Akram 2,880 and Arshad 18,684 owe, Karim is owed 10,160, cash
/// 58,484, today's sale 41,400 and commission 5,448 (Laga 480), buyers' dues 21,564, partners A, B, C
/// -1,004, 746, 746.
#[rustfmt::skip]
const WORKED_DAY: &[(&str, &str, &str, &str, &str)] = &[
    ("akram ka bakaya kitna hai", "party[kind=balance]", "r2.party.name r2.balance", "2880dr✓", "party_card"),
    // A ledger question: his balance (two places) and the newest ledger rows, with the button of R13 (D7, plan 59 F3).
    ("akram ka khata", "party[kind=balance]", "r2.party.name r2.balance", "2880dr✓", "party_card R13"),
    // Figures that a report or a view holds as a total (D7, plan 59 F3).
    ("kal ki ugrahi", "receipts[role=shop,period=day]", "r1.from r2.totals.amount", "0", "R23"),
    ("kharcha is mahine ka", "expense[kind=shop,period=range]", "r1.from r1.to r2.totals.expense", "3000", "R29"),
    ("aaj kitne nag bike", "qty[role=shop,period=today]", "r2.totals.nag", "n60", "view A"),
    ("akram ka bakaya", "party[kind=balance]", "r2.party.name r2.balance", "2880dr✓", "party_card"),
    ("akram ke zimme kitne paise hain", "party[kind=balance]", "r2.party.name r2.balance", "2880dr✓", "party_card"),
    ("how much does akram owe", "party[kind=balance]", "r2.party.name r2.balance", "2880dr✓", "party_card"),
    ("اکرم کا بقایا کتنا ہے", "party[kind=balance]", "r2.party.name r2.balance", "2880dr✓", "party_card"),
    ("akram ka balance", "party[kind=balance]", "r2.party.name r2.balance", "2880dr✓", "party_card"),
    ("akram se kitna lena hai", "party[kind=balance]", "r2.party.name r2.balance", "2880dr✓", "party_card"),
    ("akram ke dues kitne hain", "party[kind=balance]", "r2.party.name r2.balance", "2880dr✓", "party_card"),
    ("akrm ka bakaya kitna hai", "party[kind=balance]", "r2.party.name r2.balance", "2880dr✓", "party_card"),
    ("balance of akram", "party[kind=balance]", "r2.party.name r2.balance", "2880dr✓", "party_card"),
    ("akram ka bakaya batao", "party[kind=balance]", "r2.party.name r2.balance", "2880dr✓", "party_card"),
    ("arshad ka bakaya kitna hai", "party[kind=balance]", "r2.party.name r2.balance", "18684dr✓", "party_card"),
    ("arshad se kitna lena hai", "party[kind=balance]", "r2.party.name r2.balance", "18684dr✓", "party_card"),
    ("how much does arshad owe", "party[kind=balance]", "r2.party.name r2.balance", "18684dr✓", "party_card"),
    ("ارشد ملتانی کا بقایا", "party[kind=balance]", "r2.party.name r2.balance", "18684dr✓", "party_card"),
    ("arshd ka bakaya", "party[kind=balance]", "r2.party.name r2.balance", "18684dr✓", "party_card"),
    ("faisal ka balance", "party[kind=balance]", "r2.party.name r2.balance", "0✓", "party_card"),
    ("faisal ka bakaya kitna hai", "party[kind=balance]", "r2.party.name r2.balance", "0✓", "party_card"),
    ("karim ko kitna dena hai", "party[kind=balance]", "r2.party.name r2.balance", "10160cr✓", "party_card"),
    ("karim ka bakaya kitna hai", "party[kind=balance]", "r2.party.name r2.balance", "10160cr✓", "party_card"),
    ("how much do we owe karim", "party[kind=balance]", "r2.party.name r2.balance", "10160cr✓", "party_card"),
    ("کریم کو کتنا دینا ہے", "party[kind=balance]", "r2.party.name r2.balance", "10160cr✓", "party_card"),
    ("karim ko kitne paise dene hain", "party[kind=balance]", "r2.party.name r2.balance", "10160cr✓", "party_card"),
    ("karim ka payable kitna hai", "party[kind=payable]", "r2.party.name r2.balance", "10160cr✓", "party_card"),
    ("karim ki adaigi kitni baqi hai", "party[kind=payable]", "r2.party.name r2.balance", "10160cr✓", "party_card"),
    ("payable of karim", "party[kind=payable]", "r2.party.name r2.balance", "10160cr✓", "party_card"),
    ("rahim ki peshgi kitni hai", "party[kind=advance]", "r2.party.name r2.open_advance", "0", "party_card"),
    ("rahim ka advance", "party[kind=advance]", "r2.party.name r2.open_advance", "0", "party_card"),
    ("rahim ka balance", "party[kind=balance]", "r2.party.name r2.balance", "0✓", "party_card"),
    ("rahim ka bakaya kitna hai", "party[kind=balance]", "r2.party.name r2.balance", "0✓", "party_card"),
    ("rahim ko kitna dena hai", "party[kind=balance]", "r2.party.name r2.balance", "0✓", "party_card"),
    ("rahim ka payable kitna hai", "party[kind=payable]", "r2.party.name r2.payable", "0", "party_card"),
    ("aaj ki bikri", "sale[role=shop,period=today]", "r1.today_sales.amount", "41400✓", "tiles"),
    ("aaj ki bikri kitni hui", "sale[role=shop,period=today]", "r1.today_sales.amount", "41400✓", "tiles"),
    ("aaj ka commission", "commission[role=shop,period=today]", "r1.today_commission.amount", "5448✓", "tiles"),
    ("aaj ka commission kitna hai", "commission[role=shop,period=today]", "r1.today_commission.amount", "5448✓", "tiles"),
    ("aaj kitni bikriyan hui", "sale-count[role=shop,period=today]", "r1.today_sales.count", "n2", "tiles"),
    ("kitne bikri hue aaj", "sale-count[role=shop,period=today]", "r1.today_sales.count", "n2", "tiles"),
    ("how many sales today", "sale-count[role=shop,period=today]", "r1.today_sales.count", "n2", "tiles"),
    ("today's sale", "sale[role=shop,period=today]", "r1.today_sales.amount", "41400✓", "tiles"),
    ("sales today", "sale[role=shop,period=today]", "r1.today_sales.amount", "41400✓", "tiles"),
    ("commission today", "commission[role=shop,period=today]", "r1.today_commission.amount", "5448✓", "tiles"),
    ("آج کی بکری", "sale[role=shop,period=today]", "r1.today_sales.amount", "41400✓", "tiles"),
    ("آج کا کمیشن", "commission[role=shop,period=today]", "r1.today_commission.amount", "5448✓", "tiles"),
    ("aaj ka laga", "laga[role=shop,period=today]", "r2.totals.laga", "480", "view A R34"),
    ("aaj ka laga kitna hai", "laga[role=shop,period=today]", "r2.totals.laga", "480", "view A R34"),
    ("kal ki bikri", "sale[role=shop,period=day]", "r1.from r2.totals.gross", "0", "view A R34"),
    ("kal ka commission", "commission[role=shop,period=day]", "r1.from r2.totals.total_comm", "0", "view A R34"),
    ("yesterday's sale", "sale[role=shop,period=day]", "r1.from r2.totals.gross", "0", "view A R34"),
    ("byopariyon ko kitna dena hai", "shop[kind=payable]", "r1.totals.payable", "10160", "R24"),
    ("gahakon se kitna lena hai", "shop[kind=buyers-dues]", "r1.buyers_dues.amount", "21564✓", "tiles R20"),
    // A seller's commission is two labelled lines (Umar's ruling): charged to him, and the total on his goods.
    ("rahim ka commission aaj kitna hai", "commission[role=seller-own,period=today] + commission[role=seller,period=today]", "r2.matches.0.name r3.rows.a18.seller_comm r2.matches.0.name r3.rows.a18.total_comm", "1764 3928", "view B"),
    ("rahim ka commission is mahine", "commission[role=seller-own,period=range] + commission[role=seller,period=range]", "r2.matches.0.name r1.from r1.to r3.rows.a18.seller_comm r2.matches.0.name r1.from r1.to r3.rows.a18.total_comm", "1764 3928", "view B"),
    ("karim ka commission is mahine", "commission[role=seller-own,period=range] + commission[role=seller,period=range]", "r2.matches.0.name r1.from r1.to r3.rows.a19.seller_comm r2.matches.0.name r1.from r1.to r3.rows.a19.total_comm", "720 1520", "view B"),
    ("rahim aur karim ka commission is mahine", "commission[role=seller-own,period=range] + commission[role=seller,period=range] + commission[role=seller-own,period=range] + commission[role=seller,period=range]", "r2.matches.0.name r1.from r1.to r4.rows.a18.seller_comm r2.matches.0.name r1.from r1.to r4.rows.a18.total_comm r3.matches.0.name r1.from r1.to r5.rows.a19.seller_comm r3.matches.0.name r1.from r1.to r5.rows.a19.total_comm", "1764 3928 720 1520", "view B view B"),
    ("rahim ki bikri aur commission is mahine", "sale[role=seller,period=range] + commission[role=seller-own,period=range] + commission[role=seller,period=range]", "r2.matches.0.name r1.from r1.to r3.rows.a18.gross r2.matches.0.name r1.from r1.to r3.rows.a18.seller_comm r2.matches.0.name r1.from r1.to r3.rows.a18.total_comm", "29400 1764 3928", "view B"),
    // A partner's capital now: R31 for him, read in two places (the worked day has none). The button is the
    // partner overview: the capital is not on the statement page of R31, and a button opens only what holds
    // the figure (D331, plan 59 C1).
    ("partner b ka sarmaya", "partner-capital[]", "r1.matches.0.name r2.capital.balance", "0✓", "screen partneroverview"),
    ("partner b ka sarmaya kitna laga hua hai", "partner-capital[]", "r1.matches.0.name r2.capital.balance", "0✓", "screen partneroverview"),
    // Several flows of the same record or of the shop, one line each.
    ("commission aur sale aaj ki", "commission[role=shop,period=today] + sale[role=shop,period=today]", "r1.today_commission.amount r1.today_sales.amount", "5448✓ 41400✓", "tiles"),
    ("aaj ki bikri aur commission", "sale[role=shop,period=today] + commission[role=shop,period=today]", "r1.today_sales.amount r1.today_commission.amount", "41400✓ 5448✓", "tiles"),
    ("sales and commission of today", "sale[role=shop,period=today] + commission[role=shop,period=today]", "r1.today_sales.amount r1.today_commission.amount", "41400✓ 5448✓", "tiles"),
    ("آج کی بکری اور کمیشن", "sale[role=shop,period=today] + commission[role=shop,period=today]", "r1.today_sales.amount r1.today_commission.amount", "41400✓ 5448✓", "tiles"),
    ("kal ki bikri aur commission kitna tha", "sale[role=shop,period=day] + commission[role=shop,period=day]", "r1.from r2.totals.gross r1.from r2.totals.total_comm", "0 0", "view A R34"),
    ("is mahine ab tak kitni bikri aur commission hua", "sale[role=shop,period=range] + commission[role=shop,period=range]", "r1.from r1.to r2.totals.gross r1.from r1.to r2.totals.total_comm", "41400 5448", "view A"),
    ("اس سال کی کل بکری کتنی ہے", "sale[role=shop,period=range]", "r1.from r1.to r2.totals.gross", "41400", "view A"),
    ("kele ka commission aur bikri aaj", "commission[role=item,period=today] + sale[role=item,period=today]", "r2.matches.0.name r3.rows.i1.total_comm r2.matches.0.name r3.rows.i1.gross", "3928 29400", "view A"),
    ("kele ka commission aur bikri is mahine", "commission[role=item,period=range] + sale[role=item,period=range]", "r2.matches.0.name r1.from r1.to r3.rows.i1.total_comm r2.matches.0.name r1.from r1.to r3.rows.i1.gross", "3928 29400", "view A"),
    ("is mahine ki bikri", "sale[role=shop,period=range]", "r1.from r1.to r2.totals.gross", "41400", "view A"),
    ("is mahine ka commission", "commission[role=shop,period=range]", "r1.from r1.to r2.totals.total_comm", "5448", "view A"),
    ("is mahine ka laga", "laga[role=shop,period=range]", "r1.from r1.to r2.totals.laga", "480", "view A"),
    ("is mahine kitni bikriyan hui", "sale-count[role=shop,period=range]", "r1.from r1.to r2.totals.bikri_count", "n2", "view B"),
    ("this month sale", "sale[role=shop,period=range]", "r1.from r1.to r2.totals.gross", "41400", "view A"),
    ("commission this month", "commission[role=shop,period=range]", "r1.from r1.to r2.totals.total_comm", "5448", "view A"),
    ("pichle mahine ki bikri", "sale[role=shop,period=range]", "r1.from r1.to r2.totals.gross", "0", "view A"),
    ("pichle mahine ka commission", "commission[role=shop,period=range]", "r1.from r1.to r2.totals.total_comm", "0", "view A"),
    ("is hafte ki bikri", "sale[role=shop,period=range]", "r1.from r1.to r2.totals.gross", "41400", "view A"),
    ("is hafte ka commission", "commission[role=shop,period=range]", "r1.from r1.to r2.totals.total_comm", "5448", "view A"),
    ("pichle hafte ka commission", "commission[role=shop,period=range]", "r1.from r1.to r2.totals.total_comm", "0", "view A"),
    ("is saal ki bikri", "sale[role=shop,period=range]", "r1.from r1.to r2.totals.gross", "41400", "view A"),
    ("is saal ka commission", "commission[role=shop,period=range]", "r1.from r1.to r2.totals.total_comm", "5448", "view A"),
    ("is saal ka laga", "laga[role=shop,period=range]", "r1.from r1.to r2.totals.laga", "480", "view A"),
    ("ab tak ki bikri", "sale[role=shop,period=range]", "r1.from r1.to r2.totals.gross", "41400", "view A"),
    ("ab tak ka commission", "commission[role=shop,period=range]", "r1.from r1.to r2.totals.total_comm", "5448", "view A"),
    ("october ki bikri", "sale[role=shop,period=range]", "r1.from r1.to r2.totals.gross", "41400", "view A"),
    ("october ka commission", "commission[role=shop,period=range]", "r1.from r1.to r2.totals.total_comm", "5448", "view A"),
    ("1 se 5 october tak ki bikri", "sale[role=shop,period=range]", "r1.from r1.to r2.totals.gross", "0", "view A"),
    ("1 se 5 october tak ka commission", "commission[role=shop,period=range]", "r1.from r1.to r2.totals.total_comm", "0", "view A"),
    ("kele ka commission is mahine", "commission[role=item,period=range]", "r2.matches.0.name r1.from r1.to r3.rows.i1.total_comm", "3928", "view A"),
    ("kele ki bikri is mahine", "sale[role=item,period=range]", "r2.matches.0.name r1.from r1.to r3.rows.i1.gross", "29400", "view A"),
    ("kele ka laga is mahine", "laga[role=item,period=range]", "r2.matches.0.name r1.from r1.to r3.rows.i1.laga", "400", "view A"),
    ("kele ka commission pichle mahine", "commission[role=item,period=range]", "r2.matches.0.name r1.from r1.to r3.totals.total_comm", "0", "view A"),
    ("kele ka commission aaj", "commission[role=item,period=today]", "r2.matches.0.name r3.rows.i1.total_comm", "3928", "view A"),
    ("kele ka commission ab tak", "commission[role=item,period=range]", "r2.matches.0.name r1.from r1.to r3.rows.i1.total_comm", "3928", "view A"),
    ("kelon ka commission is mahine", "commission[role=item,period=range]", "r2.matches.0.name r1.from r1.to r3.rows.i1.total_comm", "3928", "view A"),
    ("banana commission this month", "commission[role=item,period=range]", "r2.matches.0.name r1.from r1.to r3.rows.i1.total_comm", "3928", "view A"),
    ("کیلے کا کمیشن اس مہینے", "commission[role=item,period=range]", "r2.matches.0.name r1.from r1.to r3.rows.i1.total_comm", "3928", "view A"),
    ("saib ka commission is mahine", "commission[role=item,period=range]", "r2.matches.0.name r1.from r1.to r3.rows.i2.total_comm", "1520", "view A"),
    ("saib ki bikri is mahine", "sale[role=item,period=range]", "r2.matches.0.name r1.from r1.to r3.rows.i2.gross", "12000", "view A"),
    ("saib ka laga is mahine", "laga[role=item,period=range]", "r2.matches.0.name r1.from r1.to r3.rows.i2.laga", "80", "view A"),
    ("kele aur saib ka commission is mahine", "commission[role=item,period=range] + commission[role=item,period=range]", "r2.matches.0.name r1.from r1.to r4.rows.i1.total_comm r3.matches.0.name r1.from r1.to r5.rows.i2.total_comm", "3928 1520", "view A view A"),
    ("kele aur saib ki bikri is mahine", "sale[role=item,period=range] + sale[role=item,period=range]", "r2.matches.0.name r1.from r1.to r4.rows.i1.gross r3.matches.0.name r1.from r1.to r5.rows.i2.gross", "29400 12000", "view A view A"),
    ("rahim ki bikri is mahine", "sale[role=seller,period=range]", "r2.matches.0.name r1.from r1.to r3.rows.a18.gross", "29400", "view B"),
    ("rahim ke kitne bikri hue is mahine", "sale-count[role=seller,period=range]", "r2.matches.0.name r1.from r1.to r3.rows.a18.bikri_count", "n1", "view B"),
    ("rahim ki bikri aaj", "sale[role=seller,period=today]", "r2.matches.0.name r3.rows.a18.gross", "29400", "view B"),
    ("rahim ki bikri pichle mahine", "sale[role=seller,period=range]", "r2.matches.0.name r1.from r1.to r3.totals.gross", "0", "view B"),
    ("karim ki bikri is mahine", "sale[role=seller,period=range]", "r2.matches.0.name r1.from r1.to r3.rows.a19.gross", "12000", "view B"),
    ("karim ke kitne bikri hue is mahine", "sale-count[role=seller,period=range]", "r2.matches.0.name r1.from r1.to r3.rows.a19.bikri_count", "n1", "view B"),
    ("rahim aur karim ki bikri is mahine", "sale[role=seller,period=range] + sale[role=seller,period=range]", "r2.matches.0.name r1.from r1.to r4.rows.a18.gross r3.matches.0.name r1.from r1.to r5.rows.a19.gross", "29400 12000", "view B view B"),
    ("akram ki bikri is mahine", "sale[role=buyer,period=range]", "r2.matches.0.name r1.from r1.to r3.rows.a20.purchase_value", "12000", "view C"),
    ("akram ka commission is mahine", "commission[role=buyer,period=range]", "r2.matches.0.name r1.from r1.to r3.rows.a20.buyer_comm", "720", "view C"),
    ("akram ka laga is mahine", "laga[role=buyer,period=range]", "r2.matches.0.name r1.from r1.to r3.rows.a20.laga", "160", "view C"),
    ("akram ki bikri aaj", "sale[role=buyer,period=today]", "r2.matches.0.name r3.rows.a20.purchase_value", "12000", "view C"),
    ("akram ka commission aaj", "commission[role=buyer,period=today]", "r2.matches.0.name r3.rows.a20.buyer_comm", "720", "view C"),
    ("arshad ki bikri is mahine", "sale[role=buyer,period=range]", "r2.matches.0.name r1.from r1.to r3.rows.a21.purchase_value", "17400", "view C"),
    ("arshad ka commission is mahine", "commission[role=buyer,period=range]", "r2.matches.0.name r1.from r1.to r3.rows.a21.buyer_comm", "1044", "view C"),
    ("arshad ka laga is mahine", "laga[role=buyer,period=range]", "r2.matches.0.name r1.from r1.to r3.rows.a21.laga", "240", "view C"),
    ("akram aur arshad ki bikri is mahine", "sale[role=buyer,period=range] + sale[role=buyer,period=range]", "r2.matches.0.name r1.from r1.to r4.rows.a20.purchase_value r3.matches.0.name r1.from r1.to r5.rows.a21.purchase_value", "12000 17400", "view C view C"),
    ("akram aur arshad ka bakaya", "party[kind=balance] + party[kind=balance]", "r3.party.name r3.balance r4.party.name r4.balance", "2880dr✓ 18684dr✓", "party_card party_card"),
    ("akram aur arshad ka commission is mahine", "commission[role=buyer,period=range] + commission[role=buyer,period=range]", "r2.matches.0.name r1.from r1.to r4.rows.a20.buyer_comm r3.matches.0.name r1.from r1.to r5.rows.a21.buyer_comm", "720 1044", "view C view C"),
    ("cash kitna hai", "shop[kind=cash]", "r1.cash_in_hand.amount", "58484✓", "tiles"),
    ("cash in hand", "shop[kind=cash]", "r1.cash_in_hand.amount", "58484✓", "tiles"),
    ("roqar kitni hai", "shop[kind=cash]", "r1.cash_in_hand.amount", "58484✓", "tiles"),
    ("roqar kitna hai", "shop[kind=cash]", "r1.cash_in_hand.amount", "58484✓", "tiles"),
    ("gala kitna hai", "shop[kind=cash]", "r1.cash_in_hand.amount", "58484✓", "tiles"),
    ("how much cash do we have", "shop[kind=cash]", "r1.cash_in_hand.amount", "58484✓", "tiles"),
    ("روکڑ کتنی ہے", "shop[kind=cash]", "r1.cash_in_hand.amount", "58484✓", "tiles"),
    ("kul ugrahi kitni hai", "shop[kind=total-dues]", "r1.total_dues.amount", "21564✓", "tiles R20"),
    ("total dues", "shop[kind=total-dues]", "r1.total_dues.amount", "21564✓", "tiles R20"),
    ("total dues kitne hain", "shop[kind=total-dues]", "r1.total_dues.amount", "21564✓", "tiles R20"),
    ("buyers ke dues kitne hain", "shop[kind=buyers-dues]", "r1.buyers_dues.amount", "21564✓", "tiles R20"),
    ("dues kitne hain", "shop[kind=buyers-dues]", "r1.buyers_dues.amount", "21564✓", "tiles R20"),
    ("gahakon ke kul bakaya", "shop[kind=buyers-dues]", "r1.buyers_dues.amount", "21564✓", "tiles R20"),
    ("ugrahi kitni hai", "shop[kind=buyers-dues]", "r1.buyers_dues.amount", "21564✓", "tiles R20"),
    ("sellers ka payable kitna hai", "shop[kind=payable]", "r1.totals.payable", "10160", "R24"),
    ("beopariyon ko kul kitna dena hai", "shop[kind=payable]", "r1.totals.payable", "10160", "R24"),
    ("payable kitna hai", "shop[kind=payable]", "r1.totals.payable", "10160", "R24"),
    ("adaigi kitni baqi hai", "shop[kind=payable]", "r1.totals.payable", "10160", "R24"),
    ("kitni peshgi baqi hai", "shop[kind=advances]", "r1.advances_open.amount", "0", "tiles"),
    ("doobi raqam kitni hai", "shop[kind=dead]", "r1.dead_amount.amount", "0✓", "tiles"),
    ("dead amount", "shop[kind=dead]", "r1.dead_amount.amount", "0✓", "tiles"),
    ("udhar raqam kitni hai", "shop[kind=lent]", "r1.money_lent_open.amount", "0", "tiles"),
    ("money lent", "shop[kind=lent]", "r1.money_lent_open.amount", "0", "tiles"),
    ("baqi maal kitna hai", "stock[role=shop]", "r2.totals.balance r2.count", "n0 n0", "tiles R10"),
    ("stock kitna hai", "stock[role=shop]", "r2.totals.balance r2.count", "n0 n0", "tiles R10"),
    ("unsold stock", "stock[role=shop]", "r2.totals.balance r2.count", "n0 n0", "tiles R10"),
    ("kele ka stock", "stock[role=item]", "r1.matches.0.name r2.totals.balance r2.count", "n0 n0", "R10"),
    ("partner a ki kul raqam", "partner-total[]", "r1.rows.s1.name r1.rows.s1.total", "-1004✓", "R31"),
    ("partner b ki kul raqam", "partner-total[]", "r1.rows.s2.name r1.rows.s2.total", "746✓", "R31"),
    ("partner c ki kul raqam", "partner-total[]", "r1.rows.s3.name r1.rows.s3.total", "746✓", "R31"),
    ("alif ki kul raqam", "partner-total[]", "r1.rows.s1.name r1.rows.s1.total", "-1004✓", "R31"),
    ("partners ki kul raqam", "partner-total[] + partner-total[] + partner-total[]", "r1.rows.s1.name r1.rows.s1.total r1.rows.s2.name r1.rows.s2.total r1.rows.s3.name r1.rows.s3.total", "-1004✓ 746✓ 746✓", "R31 R31 R31"),
    ("partner a aur partner b ki kul raqam", "partner-total[] + partner-total[]", "r1.rows.s1.name r1.rows.s1.total r1.rows.s2.name r1.rows.s2.total", "-1004✓ 746✓", "R31 R31"),
    ("اکرم کا بقایا", "party[kind=balance]", "r2.party.name r2.balance", "2880dr✓", "party_card"),
    ("ارشد کا بقایا کتنا ہے", "party[kind=balance]", "r2.party.name r2.balance", "18684dr✓", "party_card"),
    ("کریم کی ادائیگی باقی ہے", "party[kind=payable]", "r2.party.name r2.balance", "10160cr✓", "party_card"),
    ("رحیم کی پیشگی کتنی ہے", "party[kind=advance]", "r2.party.name r2.open_advance", "0", "party_card"),
    ("آج کی بکری کتنی ہوئی", "sale[role=shop,period=today]", "r1.today_sales.amount", "41400✓", "tiles"),
    ("اس مہینے کا کمیشن", "commission[role=shop,period=range]", "r1.from r1.to r2.totals.total_comm", "5448", "view A"),
    ("پچھلے مہینے کی بکری", "sale[role=shop,period=range]", "r1.from r1.to r2.totals.gross", "0", "view A"),
    ("روکڑ", "shop[kind=cash]", "r1.cash_in_hand.amount", "58484✓", "tiles"),
    ("کل اگراہی", "shop[kind=total-dues]", "r1.total_dues.amount", "21564✓", "tiles R20"),
    ("باقی مال کتنا ہے", "stock[role=shop]", "r2.totals.balance r2.count", "n0 n0", "tiles R10"),
    ("الف کی کل رقم", "partner-total[]", "r1.rows.s1.name r1.rows.s1.total", "-1004✓", "R31"),
    ("کیلے کی بکری اس مہینے", "sale[role=item,period=range]", "r2.matches.0.name r1.from r1.to r3.rows.i1.gross", "29400", "view A"),
    ("اکرم کی بکری اس مہینے", "sale[role=buyer,period=range]", "r2.matches.0.name r1.from r1.to r3.rows.a20.purchase_value", "12000", "view C"),
    ("رحیم کی بکری اس مہینے", "sale[role=seller,period=range]", "r2.matches.0.name r1.from r1.to r3.rows.a18.gross", "29400", "view B"),
    ("what is akram's balance", "party[kind=balance]", "r2.party.name r2.balance", "2880dr✓", "party_card"),
    ("akram's balance", "party[kind=balance]", "r2.party.name r2.balance", "2880dr✓", "party_card"),
    ("how much commission this month", "commission[role=shop,period=range]", "r1.from r1.to r2.totals.total_comm", "5448", "view A"),
    ("commission on banana this month", "commission[role=item,period=range]", "r2.matches.0.name r1.from r1.to r3.rows.i1.total_comm", "3928", "view A"),
    ("sales of rahim this month", "sale[role=seller,period=range]", "r2.matches.0.name r1.from r1.to r3.rows.a18.gross", "29400", "view B"),
    ("akram ka balance kya hai", "party[kind=balance]", "r2.party.name r2.balance", "2880dr✓", "party_card"),
    ("aaj ka total commission", "commission[role=shop,period=today]", "r1.today_commission.amount", "5448✓", "tiles"),
    ("this month ka commission", "commission[role=shop,period=range]", "r1.from r1.to r2.totals.total_comm", "5448", "view A"),
    ("kela ka commission is mahina", "commission[role=item,period=range]", "r2.matches.0.name r1.from r1.to r3.rows.i1.total_comm", "3928", "view A"),
    ("arshad ka bakaaya", "party[kind=balance]", "r2.party.name r2.balance", "18684dr✓", "party_card"),
    ("aaj ki sale", "sale[role=shop,period=today]", "r1.today_sales.amount", "41400✓", "tiles"),
    ("aaj ki bikri kitni hai", "sale[role=shop,period=today]", "r1.today_sales.amount", "41400✓", "tiles"),
    ("aaj ka kamishan", "commission[role=shop,period=today]", "r1.today_commission.amount", "5448✓", "tiles"),
    ("cash balance", "shop[kind=cash]", "r1.cash_in_hand.amount", "58484✓", "tiles"),
    ("how much payable to sellers", "shop[kind=payable]", "r1.totals.payable", "10160", "R24"),
    ("how much did rahim sell this month", "sale[role=seller,period=range]", "r2.matches.0.name r1.from r1.to r3.rows.a18.gross", "29400", "view B"),
    ("how much commission from akram this month", "commission[role=buyer,period=range]", "r2.matches.0.name r1.from r1.to r3.rows.a20.buyer_comm", "720", "view C"),
    ("kitna commission mila is mahine", "commission[role=shop,period=range]", "r1.from r1.to r2.totals.total_comm", "5448", "view A"),
    ("kitni bikri hui is mahine", "sale[role=shop,period=range]", "r1.from r1.to r2.totals.gross", "41400", "view A"),
    ("is saal ki bikri kitni hui", "sale[role=shop,period=range]", "r1.from r1.to r2.totals.gross", "41400", "view A"),
    ("commission aaj ka", "commission[role=shop,period=today]", "r1.today_commission.amount", "5448✓", "tiles"),
    ("karim ki adaigi", "party[kind=payable]", "r2.party.name r2.balance", "10160cr✓", "party_card"),
    ("rahim ka peshgi", "party[kind=advance]", "r2.party.name r2.open_advance", "0", "party_card"),
    ("partner b ka kul raqam", "partner-total[]", "r1.rows.s2.name r1.rows.s2.total", "746✓", "R31"),
    ("b ki kul raqam", "partner-total[]", "r1.rows.s2.name r1.rows.s2.total", "746✓", "R31"),
    ("kele ki bikri aaj", "sale[role=item,period=today]", "r2.matches.0.name r3.rows.i1.gross", "29400", "view A"),
    ("saib ka commission aaj", "commission[role=item,period=today]", "r2.matches.0.name r3.rows.i2.total_comm", "1520", "view A"),
    ("rahim ke bikri aaj", "sale[role=seller,period=today]", "r2.matches.0.name r3.rows.a18.gross", "29400", "view B"),
    ("karim ki bikri kal", "sale[role=seller,period=day]", "r2.matches.0.name r1.from r3.totals.gross", "0", "view B"),
    ("akram ka bakaya aur arshad ka bakaya", "party[kind=balance] + party[kind=balance]", "r3.party.name r3.balance r4.party.name r4.balance", "2880dr✓ 18684dr✓", "party_card party_card"),
    ("kele aur saib ka laga is mahine", "laga[role=item,period=range] + laga[role=item,period=range]", "r2.matches.0.name r1.from r1.to r4.rows.i1.laga r3.matches.0.name r1.from r1.to r5.rows.i2.laga", "400 80", "view A view A"),
];

/// The same, in the rich test year (today 05-11-2026). The numbers of that year are not pinned (the
/// seed belongs to robustness): the sentence, the references, the buttons and which figures carry a
/// tick are.
#[rustfmt::skip]
const YEAR: &[(&str, &str, &str, &str, &str)] = &[
    // H2 (live run 2, Q154): a seller's "mal" is the owner of the figure; the total with Laga is in it.
    ("raheem ke mal ka commision is sal kitna he", "commission[role=seller-own,period=range] + commission[role=seller,period=range]", "r2.matches.0.name r1.from r1.to r3.rows.a18.seller_comm r2.matches.0.name r1.from r1.to r3.rows.a18.total_comm", "581015 1186254", "view B"),
    ("kele ka stock", "stock[role=item]", "r1.matches.0.name r2.totals.balance r2.count", "n87 n9", "R10"),
    ("saib ka stock", "stock[role=item]", "r1.matches.0.name r2.totals.balance r2.count", "n40 n3", "R10"),
    ("cash kitna hai", "shop[kind=cash]", "r1.cash_in_hand.amount", "8272035✓", "tiles"),
    ("kul ugrahi kitni hai", "shop[kind=total-dues]", "r1.total_dues.amount", "9670140✓", "tiles R20"),
    // "kul ugrahi" and "total dues" are the tile کل اگراہی (PRD 9.1, glossary, FR-19.9; D7 O1: PRD v1.31
    // overrules the old ruling); by collector or city they stay the buyers' dues of R20 (live run 2, Q1052,
    // Q1054), never the tile that also holds the dead amount.
    ("agrahi wale ke hisab se kul baqaya kitna hai", "shop[kind=buyers-dues]", "r1.buyers_dues.amount", "9670140✓", "tiles R20"),
    ("total dues by collector", "shop[kind=buyers-dues]", "r1.buyers_dues.amount", "9670140✓", "tiles R20"),
    ("buyers ke dues kitne hain", "shop[kind=buyers-dues]", "r1.buyers_dues.amount", "9670140✓", "tiles R20"),
    ("sellers ka payable kitna hai", "shop[kind=payable]", "r1.totals.payable", "5454508", "R24"),
    ("kitni peshgi baqi hai", "shop[kind=advances]", "r1.advances_open.amount", "30000", "tiles"),
    ("doobi raqam kitni hai", "shop[kind=dead]", "r1.dead_amount.amount", "1426952✓", "tiles"),
    ("udhar raqam kitni hai", "shop[kind=lent]", "r1.money_lent_open.amount", "10000", "tiles"),
    ("aaj ki bikri", "sale[role=shop,period=today]", "r1.today_sales.amount", "183640✓", "tiles"),
    ("aaj ka commission", "commission[role=shop,period=today]", "r1.today_commission.amount", "22822✓", "tiles"),
    ("kal ki bikri", "sale[role=shop,period=day]", "r1.from r2.totals.gross", "335850", "view A R34"),
    // A vegetable has no default laga (PRD 4.2): the answer is the books' own figure, here 0, never 8 a nag.
    ("aloo ka laga is saal", "laga[role=item,period=range]", "r2.matches.0.name r1.from r1.to r3.rows.i6.laga", "0", "view A"),
    ("aloo ka laga is mahine", "laga[role=item,period=range]", "r2.matches.0.name r1.from r1.to r3.rows.i6.laga", "0", "view A"),
    ("is mahine ki bikri", "sale[role=shop,period=range]", "r1.from r1.to r2.totals.gross", "1124200", "view A"),
    ("is mahine ka commission", "commission[role=shop,period=range]", "r1.from r1.to r2.totals.total_comm", "137344", "view A"),
    ("pichle mahine ki bikri", "sale[role=shop,period=range]", "r1.from r1.to r2.totals.gross", "7399580", "view A"),
    ("pichle mahine ka commission", "commission[role=shop,period=range]", "r1.from r1.to r2.totals.total_comm", "905024", "view A"),
    ("pichle mahine ka laga", "laga[role=shop,period=range]", "r1.from r1.to r2.totals.laga", "17080", "view A"),
    ("is saal ka commission", "commission[role=shop,period=range]", "r1.from r1.to r2.totals.total_comm", "8360110", "view A"),
    ("pichle saal ka commission", "commission[role=shop,period=range]", "r1.from r1.to r2.totals.total_comm", "1627500", "view A"),
    ("is hafte ki bikri", "sale[role=shop,period=range]", "r1.from r1.to r2.totals.gross", "1124200", "view A"),
    ("ab tak ka commission", "commission[role=shop,period=range]", "r1.from r1.to r2.totals.total_comm", "9987610", "view A"),
    ("kele ka commission pichle mahine", "commission[role=item,period=range]", "r2.matches.0.name r1.from r1.to r3.rows.i1.total_comm", "261370", "view A"),
    ("kele ki bikri pichle mahine", "sale[role=item,period=range]", "r2.matches.0.name r1.from r1.to r3.rows.i1.gross", "2103820", "view A"),
    ("saib ka commission is saal", "commission[role=item,period=range]", "r2.matches.0.name r1.from r1.to r3.rows.i2.total_comm", "2066114", "view A"),
    ("kele aur saib ka commission pichle mahine", "commission[role=item,period=range] + commission[role=item,period=range]", "r2.matches.0.name r1.from r1.to r4.rows.i1.total_comm r3.matches.0.name r1.from r1.to r5.rows.i2.total_comm", "261370 324348", "view A view A"),
    ("faisal ka bakaya kitna hai", "party[kind=balance]", "r2.party.name r2.balance", "496211dr✓", "party_card"),
    ("usman ghani ka bakaya kitna hai", "party[kind=balance]", "r2.party.name r2.balance", "688551dr✓", "party_card"),
    ("rahim ko kitna dena hai", "party[kind=balance]", "r2.party.name r2.balance", "620667cr✓", "party_card"),
    ("rahim ki bikri pichle mahine", "sale[role=seller,period=range]", "r2.matches.0.name r1.from r1.to r3.rows.a18.gross", "949500", "view B"),
    ("faisal ki bikri pichle mahine", "sale[role=buyer,period=range]", "r2.matches.0.name r1.from r1.to r3.rows.a31.purchase_value", "466350", "view C"),
    ("faisal ka commission is saal", "commission[role=buyer,period=range]", "r2.matches.0.name r1.from r1.to r3.rows.a31.buyer_comm", "280387", "view C"),
    ("partner usman ki kul raqam", "partner-total[]", "r1.rows.s3.name r1.rows.s3.total", "2506856✓", "R31"),
    ("alif ki kul raqam", "partner-total[]", "r1.rows.s1.name r1.rows.s1.total", "2563113✓", "R31"),
    ("partners ki kul raqam", "partner-total[] + partner-total[] + partner-total[]", "r1.rows.s1.name r1.rows.s1.total r1.rows.s2.name r1.rows.s2.total r1.rows.s3.name r1.rows.s3.total", "2563113✓ 2902581✓ 2506856✓", "R31 R31 R31"),
    ("rahim ki peshgi kitni hai", "party[kind=advance]", "r2.party.name r2.open_advance", "0", "party_card"),
    ("rahim ka payable kitna hai", "party[kind=payable]", "r2.party.name r2.balance", "2268537cr✓", "party_card"),
    // Q610 (live run 2): "li hui hai" is the state, so it is the advance still open.
    ("tahir ne kitni peshgi li hui hai", "party[kind=advance]", "r2.party.name r2.open_advance", "30000", "party_card"),
];

// ---- the checks ----------------------------------------------------------------------------------------

type Case = (
    &'static str,
    &'static str,
    &'static str,
    &'static str,
    &'static str,
);

/// Runs the cases of a table in one language. `pinned` says whether the figures are compared as they
/// are; if not, only their sides and ticks are (the digits are cut).
fn run_table(
    w: &world::World,
    today: BusinessDate,
    language: Language,
    cases: &[Case],
    pinned: bool,
) -> Vec<String> {
    let strip = |text: &str| -> String {
        if pinned {
            text.to_owned()
        } else {
            text.chars().filter(|c| !c.is_ascii_digit()).collect()
        }
    };
    let mut wrong = Vec::new();
    for (question, sentences, refs, figures, buttons) in cases {
        match instant(w, today, language, question) {
            Err(e) => wrong.push(e),
            Ok(None) => wrong.push(format!("{question}: no instant answer")),
            Ok(Some((figure, answer))) => {
                let seen = seen_of(&figure, &answer);
                let want = Seen {
                    sentences: (*sentences).to_owned(),
                    refs: (*refs).to_owned(),
                    figures: strip(figures),
                    buttons: (*buttons).to_owned(),
                };
                let got = Seen {
                    figures: strip(&seen.figures),
                    ..seen
                };
                if got != want {
                    wrong.push(format!(
                        "{question:?}\n   got    {got:?}\n   wanted {want:?}"
                    ));
                }
                // The words are the panel's own: no digit, no argument left unfilled.
                let text = &figure.answer.text;
                let bare: String = pieces(text)
                    .into_iter()
                    .filter_map(|p| match p {
                        Piece::Words(w) => Some(w),
                        Piece::Ref(_) => None,
                    })
                    .collect();
                if bare.chars().any(|c| c.is_ascii_digit() || c == '$')
                    || bare.contains("assistant-")
                {
                    wrong.push(format!("{question:?}: unclean words {bare:?}"));
                }
                // Only a ledger question shows a table: the newest rows of his ledger, one table, at most 7
                // rows (D7, plan 59 F3). No answer has a next question.
                let ledger = question.contains("khata");
                let tables_ok = if ledger {
                    answer.tables.len() == 1
                        && answer.tables.first().is_some_and(|t| t.rows.len() <= 7)
                } else {
                    answer.tables.is_empty()
                };
                if !answer.next.is_empty() || !tables_ok {
                    wrong.push(format!("{question:?}: a table or a next question"));
                }
            }
        }
    }
    wrong
}

fn report(wrong: &[String], total: usize) {
    assert!(
        wrong.is_empty(),
        "{} of {total} wrong:\n{}",
        wrong.len(),
        wrong.join("\n")
    );
}

/// Every figure question of the worked day, in Roman Urdu, Urdu letters, English and a mix, with
/// misspellings: the app answers itself with the reviewed sentence, the figure as a reference with its
/// tick where it has a second place, and the buttons of the sources it read. The Urdu app and the
/// English app give the same sentence, references, figures and buttons.
#[test]
fn the_worked_day_answers_by_itself_in_both_languages() {
    let w = books();
    worked::events_1_to_5(&w);
    let today = on(2026, 10, 10);
    assert!(WORKED_DAY.len() >= 150, "{}", WORKED_DAY.len());
    for language in [Language::En, Language::Ur] {
        let wrong = run_table(&w, today, language, WORKED_DAY, true);
        report(&wrong, WORKED_DAY.len());
    }
}

/// The same in the rich test year: stock, many items and partners, big numbers. The figures are the
/// books' own (their ticks are checked); the sentence, references and buttons are pinned.
#[test]
fn the_rich_year_answers_by_itself() {
    let y = year::seed::build();
    let today = on(2026, 11, 5);
    for language in [Language::En, Language::Ur] {
        let wrong = run_table(&y.w, today, language, YEAR, false);
        report(&wrong, YEAR.len());
    }
}

/// The shop's goods left is answered only when the dashboard tile and R10 agree. The rich year has
/// trip purchases and garden arrivals in R10, which the tile (the lots of our shop) does not count:
/// the two differ, so the question goes to the model (money review 2, finding 3).
#[test]
fn goods_left_with_trip_goods_in_the_books_goes_to_the_model() {
    let y = year::seed::build();
    let today = on(2026, 11, 5);
    for question in [
        "baqi maal kitna hai",
        "stock kitna hai",
        "how much unsold stock is left",
    ] {
        match instant(&y.w, today, Language::En, question) {
            Ok(None) => {}
            other => panic!("{question}: {other:?}"),
        }
    }
}

/// What is not a simple figure is the model's: two figures, a comparison, a forecast, a list, a
/// print, a khata, advice, a change request, a made-up name, an item's rate, what was given, a word
/// for a garden or own goods, a buyer's advance and a question with no period for a figure of a day.
#[test]
fn what_is_not_a_simple_figure_goes_to_the_model() {
    let w = books();
    worked::events_1_to_5(&w);
    let today = on(2026, 10, 10);
    let mut instant_ones = Vec::new();
    for question in [
        // The quality of a sale line (plan 58): the figure of the item adds every quality, so it is
        // no answer for one of them.
        "awwal seb aaj kitne ka bika",
        "doyam kele ki aaj ki bikri",
        "kele ki soyam bikri is mahine",
        "اول سیب کی آج کی بکری",
        "first quality apple sales today",
        "A grade banana sales this month",
        "aaj commission bikri ka kitne fisad",
        "akram aur arshad ka jama kitna hua",
        "aaj kele ki kitni bikri hui",
        "kis se pehle wasooli karon",
        "how much was sold of vegitables last week",
        "is mahine ka commission pichle mahine se kitna zyada hai",
        "akram ka bakaya aur commission",
        "aaj aur kal ki bikri",
        "pichle mahine aur is mahine ka commission",
        "is mahine ka commission pichle mahine se kitna zyada hai",
        "faisal ki aaj ki bikri ka print",
        "akram ki 5000 wasooli likh do",
        "sale kaise barhayen",
        "agle mahine ka commission",
        "mausam kaisa hai",
        "akram ka bakaya pichle mahine",
        "akram ne kya kya khareeda",
        "top 5 buyers",
        "kis kis ke dues hain",
        "bagh ka commission is mahine",
        "safar ki bikri",
        "apne maal ki bikri aaj",
        "rahim ko kitni peshgi di",
        "rahim ko kitni adaigi hui",
        "kele ka rate",
        "kele ka bhao",
        "zubair ka bakaya",
        "akram ki peshgi kitni hai",
        "bikri kitni hui",
        "commission kitna hai",
        "aaj ka khulasa",
        "hello",
        "akram aur kele ka commission",
        "pichli bikri",
        "faisal ka mahine mein commission",
        "rahim ka commission kitna hai",
        "rahim ka seller commission aaj",
        "byopariyon se kitna lena hai",
        "gahakon ko kitna dena hai",
        "kul kitna dena hai",
        "aaj kitna cash aya",
        "rahim ko adaigi ki gayi",
        "rahim ki adaigi kitni ki hai",
        "partner a ka khalis kitna hai",
        "partner a ka total raqam kitna hai",
        "خالص کتنا ہے الف کا",
        "aaj kitna maal bika",
        "naqdi bikri aaj",
        "udhari bikri aaj",
        "safi bikri aaj",
        "commission kitne percent hai",
        "بکری کی تعداد",
        "bikri ki tadad aaj",
        "buyers ka commission aaj",
        "sellers ka commission is mahine",
        "udhar kitna hai",
        "rahim ki peshgi wapsi kitni hai",
        "udhar raqam ki wapsi kitni hai",
        "partner b ki sarmaya wapsi kitni hai",
        "rahim ko kitni peshgi bheji",
        "rahim se kitni peshgi aayi",
        "how much has partner b invested",
        "partner a aur partner b ki kul raqam mila kar kitni hai",
        "sab partners ki kul raqam mila kar",
        "kal ki bikri aur commission mila kar",
        "aaj ka cash in kitna hai",
        "akram ne is mahine kis kis din kitna commission diya",
        "akram ka bakaya aur arshad ka bakaya dono ka mila kar kitna hai",
        "akram aur arshad ka bakaya mila kar",
        "total of akram and arshad together",
        "kele or saib ka commission mila kar is mahine",
        "rahim ne kitni peshgi li",
        "how much advance did rahim take",
        "rahim ko kitni adaigi ki",
        "akram se kitni wasooli ki hai",
        "kitni wasooli ki hai",
        "akram se kitne paise liye",
        "how much money did akram give",
        "commission from buyers on banana this month",
        "kele par gahak ka commission is mahine",
        "کل بکری کتنی ہوئی",
        "aaj kitna laga",
        "akram ka commission aur laga is mahine",
        "aaj ka sale commission aur laga",
        "how much seller commission did apple give this month",
        "seller commission aaj",
        "buyer laga aaj",
        "how much fruit did partner A take home",
        "how many boxes of banana were sold today",
        "کل کمیشن کتنا ہے",
        "کل بکری کتنی ہے",
        "kele ka commission",
        "saib ka commission",
        "apple ki bikri",
        "rahim ki bikri",
        "akram ki bikri",
        "akram ka commission",
        "akram ka laga",
    ] {
        match instant(&w, today, Language::En, question) {
            Ok(None) => {}
            other => instant_ones.push(format!("{question}: {other:?}")),
        }
    }
    assert!(
        instant_ones.is_empty(),
        "answered by the app:\n{}",
        instant_ones.join("\n")
    );
}

/// A figure that is not in the reads is no answer: with any one result of the plan missing (a read the
/// tool refused), nothing is composed. Every read of the plan is needed, so the answer never rests on
/// a figure the app did not read.
#[test]
fn a_missing_read_gives_no_answer() {
    let w = books();
    worked::events_1_to_5(&w);
    let today = on(2026, 10, 10);
    let words = Words::new().expect("the words");
    let mut tried = 0usize;
    for (at, (question, ..)) in WORKED_DAY.iter().enumerate() {
        if at % 3 != 0 {
            continue;
        }
        let Some((u, state)) = read_state(&w, today, Language::En, question) else {
            panic!("{question}: no reads");
        };
        assert!(
            compose_figure(&u, &state.results, Language::En, &words).is_some(),
            "{question}"
        );
        for skip in 0..state.results.len() {
            let fewer: Vec<ToolResult> = state
                .results
                .iter()
                .enumerate()
                .filter(|(i, _)| *i != skip)
                .map(|(_, r)| r.clone())
                .collect();
            tried += 1;
            assert!(
                compose_figure(&u, &fewer, Language::En, &words).is_none(),
                "{question}: still answered without {}",
                state.results[skip].tool
            );
        }
    }
    assert!(tried >= 100, "{tried}");
}

/// A figure that is not the one the plan read is no answer either: the result of an other record
/// (a card of another party) does not stand in for the one asked. Two guards, each tried alone: the
/// result must have been read with the input the plan gave, and its own record must be the one named.
#[test]
fn a_result_of_another_record_does_not_stand_in() {
    let w = books();
    worked::events_1_to_5(&w);
    let today = on(2026, 10, 10);
    let words = Words::new().expect("the words");
    let (u, state) =
        read_state(&w, today, Language::En, "akram ka bakaya kitna hai").expect("reads");
    let (_, other) =
        read_state(&w, today, Language::En, "arshad ka bakaya kitna hai").expect("reads");
    let theirs = other
        .results
        .iter()
        .find(|r| r.tool == "party_card")
        .expect("a card");
    // Arshad's card, read for Arshad, under Akram's question: the input is not the plan's.
    let mut whole = state.results.clone();
    if let Some(card) = whole.iter_mut().find(|r| r.tool == "party_card") {
        card.node = theirs.node.clone();
        card.input = theirs.input.clone();
    }
    assert!(compose_figure(&u, &whole, Language::En, &words).is_none());
    // A card read for Akram whose contents are another party's: its own record is not the one named.
    let mut contents = state.results.clone();
    if let Some(card) = contents.iter_mut().find(|r| r.tool == "party_card") {
        card.node = theirs.node.clone();
    }
    assert!(compose_figure(&u, &contents, Language::En, &words).is_none());
}

fn set_field(result: &mut ToolResult, key: &str, node: mandi_assistant::results::Node) {
    if let mandi_assistant::results::Node::Obj(fields) = &mut result.node {
        fields.retain(|(k, _)| k != key);
        fields.push((key.to_owned(), node));
    }
}

/// A party with a warning has words of his own in the answer (inactive, over his limit, in the dead
/// amount): the model writes them, so the app does not answer.
#[test]
fn a_party_with_a_warning_goes_to_the_model() {
    use mandi_assistant::results::Node;
    let w = books();
    worked::events_1_to_5(&w);
    let today = on(2026, 10, 10);
    let words = Words::new().expect("the words");
    for question in ["akram ka bakaya kitna hai", "karim ka payable kitna hai"] {
        let (u, state) = read_state(&w, today, Language::En, question).expect("reads");
        assert!(compose_figure(&u, &state.results, Language::En, &words).is_some());
        for (key, node) in [
            ("dead", Node::Bool(true)),
            ("active", Node::Bool(false)),
            (
                "warnings",
                Node::List(vec![Node::Str("over_limit".to_owned())]),
            ),
        ] {
            let mut results = state.results.clone();
            if let Some(card) = results.iter_mut().find(|r| r.tool == "party_card") {
                set_field(card, key, node);
            }
            assert!(
                compose_figure(&u, &results, Language::En, &words).is_none(),
                "{question}: answered with {key} set"
            );
        }
    }
}

/// A figure whose two places differ is still composed: the checks of the loop make it the final
/// "records differ" answer (PRD FR-7.76 point 2), not a wrong figure.
#[test]
fn a_figure_whose_places_differ_is_left_to_the_checks() {
    use mandi_assistant::results::{Money, Node, Val};
    let w = books();
    worked::events_1_to_5(&w);
    let today = on(2026, 10, 10);
    let words = Words::new().expect("the words");
    let (u, mut state) = read_state(&w, today, Language::En, "cash kitna hai").expect("reads");
    let tiles = state
        .results
        .iter_mut()
        .find(|r| r.tool == "tiles")
        .expect("tiles");
    if let Node::Obj(fields) = &mut tiles.node
        && let Some((_, Node::Obj(cash))) = fields.iter_mut().find(|(k, _)| k == "cash_in_hand")
        && let Some((_, amount)) = cash.iter_mut().find(|(k, _)| k == "amount")
        && let Node::Val(Val::Money(m)) = amount
    {
        *m = Money {
            checked: Some(false),
            ..m.clone()
        };
    }
    let figure = compose_figure(&u, &state.results, Language::En, &words).expect("composed");
    let numbers = question_numbers("cash kitna hai");
    let env = Env {
        ctx: &w.env.ctx,
        actor: &w.owner,
        words: &words,
        language: Language::En,
        today,
        partner_locked: false,
        numbers: &numbers,
        question: "cash kitna hai",
        backup: BackupFacts {
            cloud_on: false,
            cloud_age_hours: None,
        },
    };
    let names = NameSet::new(&state, &[]);
    let verdict = Check {
        env: &env,
        state: &state,
        names: &names,
    }
    .run(&figure.answer);
    assert!(matches!(verdict, Verdict::Differ(_)), "{verdict:?}");
}

/// One day has a second place: the day summary (R34) holds the same sale, count, commission and laga
/// as the view of the day. With the two readings apart there is no answer (money review, 05-11-2026).
/// Mutation check: with the cross-check removed from `shop_flow`, the first assert fails.
#[test]
fn a_day_whose_two_readings_differ_gives_no_answer() {
    use mandi_assistant::results::Node;
    let w = books();
    worked::events_1_to_5(&w);
    let today = on(2026, 10, 10);
    let words = Words::new().expect("the words");
    let (u, state) = read_state(&w, today, Language::En, "aaj ka laga").expect("reads");
    assert!(compose_figure(&u, &state.results, Language::En, &words).is_some());
    // R34 holds the gross sale in its first row and the laga in its fifth.
    let sheet_of = |results: &[ToolResult]| -> usize {
        results
            .iter()
            .position(|r| r.node.get(&["rows", "x5", "amount"]).is_some())
            .expect("R34")
    };
    let gross = state
        .results
        .get(sheet_of(&state.results))
        .and_then(|r| r.node.get(&["rows", "x1", "amount"]).cloned())
        .expect("the gross cell");
    let mut apart = state.results.clone();
    let at = sheet_of(&apart);
    let sheet = apart.get_mut(at).expect("R34");
    if let Node::Obj(top) = &mut sheet.node
        && let Some((_, Node::Keyed(rows))) = top.iter_mut().find(|(k, _)| k == "rows")
        && let Some((_, Node::Obj(row))) = rows.iter_mut().find(|(k, _)| k == "x5")
        && let Some((_, amount)) = row.iter_mut().find(|(k, _)| k == "amount")
    {
        *amount = gross;
    } else {
        panic!("the R34 row x5");
    }
    assert!(
        compose_figure(&u, &apart, Language::En, &words).is_none(),
        "the day summary and the view differ, yet an answer was given"
    );
    // Without the day summary there is no second reading either.
    let fewer: Vec<ToolResult> = state
        .results
        .iter()
        .filter(|r| r.node.get(&["rows", "x5", "amount"]).is_none())
        .cloned()
        .collect();
    assert!(compose_figure(&u, &fewer, Language::En, &words).is_none());
}

/// A seller with an open advance: his ledger balance is not what he is owed (the advance is taken off
/// it, or not at all when it was not deducted, E2E-28), so "his balance" is the model's. His payable
/// and his advance are fields of their own and are answered. Mutation check: with the guard removed
/// from `Cx::party`, the first assert fails.
#[test]
fn a_seller_with_an_open_advance_has_no_instant_balance() {
    use mandi_assistant::results::Node;
    let w = books();
    worked::events_1_to_5(&w);
    let today = on(2026, 10, 10);
    let words = Words::new().expect("the words");
    let with_advance = |question: &str| {
        let (u, mut state) = read_state(&w, today, Language::En, question).expect("reads");
        let card = state
            .results
            .iter_mut()
            .find(|r| r.tool == "party_card")
            .expect("the card");
        // Karim has 10,160 payable and no advance: give him an advance of that size.
        let payable = card.node.get(&["payable"]).cloned().expect("payable");
        if let Node::Obj(fields) = &mut card.node
            && let Some((_, slot)) = fields.iter_mut().find(|(k, _)| k == "open_advance")
        {
            *slot = payable;
        } else {
            panic!("the open_advance field");
        }
        compose_figure(&u, &state.results, Language::En, &words)
    };
    assert!(with_advance("karim ka bakaya kitna hai").is_none());
    assert!(with_advance("karim ka payable kitna hai").is_some());
    assert!(with_advance("karim ki peshgi kitni hai").is_some());
}

/// خالص (net, C20) and the capital are not the کل رقم (C22). After E2E-35 ب has put in 50,000: his
/// کل رقم is 50,746 and his capital 50,000 (E2E-37). "khalis" and "net" are another number again
/// (his profit share less the money he took), and no instant answer may show the کل رقم, ticked,
/// for them (backend's money review of 05-11-2026).
#[test]
fn khalis_and_net_are_never_answered_with_the_kul_raqam() {
    let w = books();
    worked::events_1_to_5(&w);
    common::set_today(&w.env, 2026, 10, 11);
    mandi_services::vouchers::save_capital_in(
        &w.env.ctx,
        &w.owner,
        &mandi_services::core::capital::CapitalInput {
            date: "11-10-2026".into(),
            partner_id: Some(w.partners[1]),
            amount: "50,000".into(),
            narration: None,
            confirm_cash_below_zero: false,
        },
    )
    .expect("capital in");
    let today = on(2026, 10, 11);
    // His کل رقم is answered, with its tick.
    let (figure, answer) = instant(&w, today, Language::En, "partner b ki kul raqam kitni hai")
        .expect("composed")
        .expect("an instant answer");
    let seen = seen_of(&figure, &answer);
    assert_eq!(seen.figures, "50746✓", "{seen:?}");
    // His capital is the 50,000 he put in, which is another number.
    let (figure, answer) = instant(&w, today, Language::En, "partner b ka sarmaya kitna hai")
        .expect("composed")
        .expect("an instant answer");
    assert_eq!(seen_of(&figure, &answer).figures, "50000✓");
    // The other numbers are the model's, with no reads made for them.
    for question in [
        "partner b ka khalis kitna hai",
        "partner b ka net kitna hai",
        "partner b ka khalis",
        "پارٹنر ب کا خالص کتنا ہے",
        "partner b ka total raqam kitna hai",
    ] {
        let u = understand(&reader(&w, today, Language::En), question);
        assert!(plan_reads(&u).is_none(), "{question}: reads were planned");
        match instant(&w, today, Language::En, question) {
            Ok(None) => {}
            other => panic!("{question}: {other:?}"),
        }
    }
}

/// An item that IS in the books but has no sales is not "no such item": the figure is 0 for the month
/// (and for the year), through view A with the item, as for any item.
#[test]
fn an_item_in_the_books_with_no_sales_gives_zero() {
    let w = books();
    worked::events_1_to_5(&w);
    world::add_item(
        &w.env,
        &w.owner,
        "شملہ مرچ",
        "Capsicum",
        "vegetable",
        "unit",
    );
    let today = on(2026, 10, 10);
    for question in [
        "shimla mirch ki is mahine ki sale batao",
        "simla mirch ki is mahine ki bikri",
        "capsicum ka commission is mahine",
    ] {
        let (figure, answer) = instant(&w, today, Language::En, question)
            .expect("composed")
            .unwrap_or_else(|| panic!("{question}: no instant answer"));
        let seen = seen_of(&figure, &answer);
        assert_eq!(seen.figures, "0", "{question}: {seen:?}");
        assert!(seen.sentences.contains("role=item"), "{question}: {seen:?}");
    }
}
