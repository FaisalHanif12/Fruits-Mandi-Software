//! Plan 54 S3 / plan 55 Section 2: the reads the app does before the first model call
//! (`plan_reads`). Each question is asked of the real books (the PRD worked day, today 10-10-2026,
//! and the rich test year, today 05-11-2026), understood, and planned. The expected calls are the
//! exact ones the model would have sent: the tool and every non-null field of its input. A key is
//! written `$akram` and is replaced by the real key of the record before the comparison.
//!
//! A second test runs every planned read through the REAL tools, one after the other as the loop
//! would, and checks none of them is refused: the input is strict, so a wrong field or a key that
//! was not shown by a `find` before it would fail here.
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

use std::collections::HashMap;

use mandi_assistant::results::{ToolResult, Words, fit};
use mandi_assistant::tools::{Env, State, Stop, call};
use mandi_assistant::understand::{Intent, PlannedRead, Reader, plan_reads, understand};
use mandi_reports::dashboard::BackupFacts;
use mandi_services::core::accounts::AccountType;
use mandi_services::core::auth::Language;
use mandi_services::core::dates::BusinessDate;
use mandi_services::core::masters::{AddaInput, CityInput, MasterInput};
use mandi_services::domain::figures::question_numbers;
use serde_json::Value;

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

fn reader<'a>(w: &'a world::World, today: BusinessDate) -> Reader<'a> {
    Reader {
        ctx: &w.env.ctx,
        actor: &w.owner,
        language: Language::En,
        today,
        partner_locked: false,
    }
}

fn on(y: i32, m: u32, d: u32) -> BusinessDate {
    BusinessDate::from_ymd(y, m, d).expect("a date")
}

/// A read as one short line: the tool and each field that is set, in the order of its schema.
fn render(read: &PlannedRead) -> String {
    let Value::Object(fields) = &read.input else {
        return format!("{}{{?}}", read.tool);
    };
    let mut parts = Vec::new();
    for (name, value) in fields {
        let text = match value {
            Value::Null => continue,
            Value::String(s) => s.clone(),
            Value::Array(rows) => rows
                .iter()
                .map(|r| {
                    format!(
                        "{}:{}",
                        r.get("column").and_then(Value::as_str).unwrap_or("?"),
                        r.get("order").and_then(Value::as_str).unwrap_or("?")
                    )
                })
                .collect::<Vec<_>>()
                .join("+"),
            other => other.to_string(),
        };
        parts.push(format!("{name}:{text}"));
    }
    format!("{}{{{}}}", read.tool, parts.join(","))
}

fn render_all(reads: &[PlannedRead]) -> String {
    reads.iter().map(render).collect::<Vec<_>>().join(" | ")
}

/// One line for each question of the file in `READS` (one question a line). A tool for the person
/// who builds, not a check: `READS=/path cargo test -p mandi-assistant --test understand_reads --
/// --ignored --nocapture dump`. `WORLD=year` asks the thin year.
#[test]
#[ignore = "a tool for the person who builds, not a check"]
fn dump() {
    let Ok(file) = std::env::var("READS") else {
        return;
    };
    let text = std::fs::read_to_string(file).expect("the file");
    let world = std::env::var("WORLD").unwrap_or_default();
    let year_world = world == "year" || world == "full";
    let (w, today);
    let y;
    if year_world {
        y = if world == "full" {
            year::seed::build()
        } else {
            year::seed::build_thin()
        };
        w = &y.w;
        today = on(2026, 11, 5);
    } else {
        let b = books();
        let b = Box::leak(Box::new(b));
        w = b;
        today = on(2026, 10, 10);
    }
    for question in text.lines().filter(|l| !l.trim().is_empty()) {
        let u = understand(&reader(w, today), question);
        let plan = plan_reads(&u).map_or_else(|| "NONE".to_owned(), |r| render_all(&r));
        println!("{question:55} => {plan}");
    }
}

// ---- the keys of the records, so the expected reads can name them ------------------------------------

/// The keys of the records the cases name, as the understanding finds them: `$akram` is the key of
/// the buyer Akram. They are read by asking, never typed here, so a change of ids changes nothing.
fn keys(w: &world::World, today: BusinessDate) -> HashMap<&'static str, String> {
    let r = reader(w, today);
    let mut out = HashMap::new();
    for (name, question) in [
        ("akram", "akram ka bakaya"),
        ("arshad", "arshad ka bakaya"),
        ("faisal", "faisal ka bakaya"),
        ("rahim", "rahim ko dena hai"),
        ("karim", "karim ko dena hai"),
        ("partner_a", "partner a ki kul raqam"),
        ("partner_b", "partner b ki kul raqam"),
    ] {
        let u = understand(&r, question);
        let hit = u
            .parties
            .first()
            .and_then(|p| p.outcome.hit())
            .unwrap_or_else(|| panic!("{name} is not in the books"));
        out.insert(name, hit.key.clone());
    }
    for (name, question) in [
        ("kela", "kele ka commission"),
        ("saib", "saib ka commission"),
    ] {
        let u = understand(&r, question);
        let hit = u
            .items
            .first()
            .and_then(|i| i.outcome.hit())
            .unwrap_or_else(|| panic!("{name} is not in the books"));
        out.insert(name, hit.key.clone());
    }
    for (name, question) in [
        ("lahore", "lahore ki ugrahi"),
        ("swat", "swat adda ki bikri pichle mahine"),
    ] {
        let u = understand(&r, question);
        let hit = u
            .records
            .first()
            .and_then(|x| x.outcome.hit())
            .unwrap_or_else(|| panic!("{name} is not in the books"));
        out.insert(name, hit.key.clone());
    }
    out
}

/// `$akram` and the like replaced by the real keys.
fn with_keys(expected: &str, keys: &HashMap<&'static str, String>) -> String {
    let mut text = expected.to_owned();
    // The longest names first, so `$partner_a` is not cut at `$partner`.
    let mut names: Vec<&&str> = keys.keys().collect();
    names.sort_by_key(|n| std::cmp::Reverse(n.len()));
    for name in names {
        text = text.replace(&format!("${name}"), &keys[*name]);
    }
    text
}

/// Runs the cases of a table: the question, and the reads it must plan (`NONE` for none).
fn check_table(
    w: &world::World,
    today: BusinessDate,
    keys: &HashMap<&'static str, String>,
    cases: &[(&str, &str)],
) -> Vec<String> {
    let mut wrong = Vec::new();
    for (question, expected) in cases {
        let u = understand(&reader(w, today), question);
        let got = plan_reads(&u).map_or_else(|| "NONE".to_owned(), |r| render_all(&r));
        let want = with_keys(expected, keys);
        if got != want {
            wrong.push(format!("{question:?}\n   got    {got}\n   wanted {want}"));
        }
    }
    wrong
}

/// Runs the reads of a plan through the real tools, in order, as the loop would, and says which
/// were refused. A `find` first shows the keys the later reads name.
fn run_plan(
    w: &world::World,
    today: BusinessDate,
    question: &str,
    reads: &[PlannedRead],
) -> Vec<String> {
    run_plan_state(w, today, question, reads).0
}

/// [`run_plan`], and the state with every result that was read.
fn run_plan_state(
    w: &world::World,
    today: BusinessDate,
    question: &str,
    reads: &[PlannedRead],
) -> (Vec<String>, State) {
    let words = Words::new().expect("the words");
    let numbers = question_numbers(question);
    let mut state = State::default();
    let mut refused = Vec::new();
    for read in reads {
        let env = Env {
            ctx: &w.env.ctx,
            actor: &w.owner,
            words: &words,
            language: Language::En,
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
            Err(Stop::Error(error)) => {
                refused.push(format!("{question:?}: {} refused: {error:?}", render(read)));
            }
            Err(Stop::Locked) => refused.push(format!("{question:?}: {} locked", render(read))),
        }
    }
    (refused, state)
}

// ---- the cases -----------------------------------------------------------------------------------------

/// The PRD worked day (today 10-10-2026, the first day of the books 31-08-2026). The question, and
/// the reads it plans, in the order to run them. Fields are written in the order of the rendering
/// (alphabetical, as the JSON is kept).
#[rustfmt::skip]
const WORKED_DAY: &[(&str, &str)] = &[
    // ---- a party's card: balance, dues, payable, advance (now)
    ("akram ka bakaya kitna hai", "find{role:buyer,text:akram} | party_card{party:$akram}"),
    ("akram ke zimme kitne paise hain", "find{role:buyer,text:akram} | party_card{party:$akram}"),
    ("how much does akram owe", "find{role:buyer,text:akram} | party_card{party:$akram}"),
    ("اکرم کا بقایا کتنا ہے", "find{role:buyer,text:اکرم} | party_card{party:$akram}"),
    ("arshad se kitna lena hai", "find{role:buyer,text:arshad} | party_card{party:$arshad}"),
    ("rahim ko kitne paise dene hain", "find{role:seller,text:rahim} | party_card{party:$rahim}"),
    ("karim ko kitna dena hai", "find{role:seller,text:karim} | party_card{party:$karim}"),
    ("karim ka payable kitna hai", "find{role:seller,text:karim} | party_card{party:$karim}"),
    ("rahim ki peshgi kitni hai", "find{role:seller,text:rahim} | party_card{party:$rahim}"),
    ("faisal ka balance", "find{role:buyer,text:faisal} | party_card{party:$faisal}"),
    // ---- a party's khata, statement, purchases, day
    ("akram ka statement", "find{role:buyer,text:akram} | report{id:R14,party:$akram}"),
    ("akram ka fard hisab", "find{role:buyer,text:akram} | report{id:R14,party:$akram}"),
    ("akram ka aaj ka hisab", "find{role:buyer,text:akram} | report{as_of:2026-10-10,id:R35,party:$akram}"),
    ("akram ne aaj kya khareeda", "find{role:buyer,text:akram} | report{as_of:2026-10-10,id:R35,party:$akram}"),
    ("arshad ne kya kya khareeda", "find{role:buyer,text:arshad} | analytics{buyer:$arshad,view:A}"),
    ("arshad ne pichle mahine kya khareeda", "period{kind:last_month} | find{role:buyer,text:arshad} | analytics{buyer:$arshad,from:2026-09-01,to:2026-09-30,view:A}"),
    ("akram ka khata is mahine ka", "period{kind:this_month} | find{role:buyer,text:akram} | report{from:2026-10-01,id:R13,party:$akram,to:2026-10-10}"),
    // ---- a party's commission, sale, count (view B for a seller, C for a buyer)
    ("rahim ki bikri kitni hui", "find{role:seller,text:rahim} | analytics{seller:$rahim,view:B}"),
    ("rahim ki bikri pichle mahine", "period{kind:last_month} | find{role:seller,text:rahim} | analytics{from:2026-09-01,seller:$rahim,to:2026-09-30,view:B}"),
    ("rahim ka commission is mahine", "period{kind:this_month} | find{role:seller,text:rahim} | analytics{from:2026-10-01,seller:$rahim,to:2026-10-10,view:B}"),
    ("rahim ka commission", "find{role:seller,text:rahim} | analytics{seller:$rahim,view:B}"),
    ("akram ka commission", "find{role:buyer,text:akram} | analytics{buyer:$akram,view:C}"),
    ("akram ka laga is saal", "period{kind:this_year} | find{role:buyer,text:akram} | analytics{buyer:$akram,from:2026-01-01,to:2026-10-10,view:C}"),
    ("akram ki bikri", "find{role:buyer,text:akram} | analytics{buyer:$akram,view:C}"),
    ("rahim ke kitne bikri hue pichle mahine", "period{kind:last_month} | find{role:seller,text:rahim} | analytics{from:2026-09-01,seller:$rahim,to:2026-09-30,view:B}"),
    // ---- an item (view A), alone or with a party
    ("kele ka commission pichle mahine", "period{kind:last_month} | find{role:item,text:Banana} | analytics{from:2026-09-01,item:$kela,to:2026-09-30,view:A}"),
    ("kele ka commission is saal", "period{kind:this_year} | find{role:item,text:Banana} | analytics{from:2026-01-01,item:$kela,to:2026-10-10,view:A}"),
    ("kele ki bikri is mahine", "period{kind:this_month} | find{role:item,text:Banana} | analytics{from:2026-10-01,item:$kela,to:2026-10-10,view:A}"),
    // The rate of view A is the average of the whole period. With no period it is the average of
    // all time, no rate for today: the model asks (money review R-d).
    ("kele ka rate", "NONE"),
    ("kele ka bhao", "NONE"),
    ("kele ka rate is mahine", "period{kind:this_month} | find{role:item,text:Banana} | analytics{from:2026-10-01,item:$kela,to:2026-10-10,view:A}"),
    ("kele ka commission aaj", "period{kind:today} | find{role:item,text:Banana} | analytics{from:2026-10-10,item:$kela,to:2026-10-10,view:A}"),
    ("saib ka commission", "find{role:item,text:Apple} | analytics{item:$saib,view:A}"),
    ("kele ka commission rahim ka is mahine", "period{kind:this_month} | find{role:seller,text:rahim} | find{role:item,text:Banana} | analytics{from:2026-10-01,item:$kela,seller:$rahim,to:2026-10-10,view:A}"),
    ("kele ka stock", "find{role:item,text:Banana} | report{id:R10,item:$kela}"),
    ("پچھلے مہینے کا کمیشن", "period{kind:last_month} | analytics{from:2026-09-01,to:2026-09-30,view:A}"),
    ("کیلے کا کمیشن پچھلے مہینے", "period{kind:last_month} | find{role:item,text:Banana} | analytics{from:2026-09-01,item:$kela,to:2026-09-30,view:A}"),
    // ---- the shop's sale and commission
    ("aaj ki bikri", "tiles{}"),
    ("aaj ka commission", "tiles{}"),
    ("آج کی بکری", "tiles{}"),
    ("aaj kitni bikriyan hui", "tiles{}"),
    ("is mahine ki bikri", "period{kind:this_month} | analytics{from:2026-10-01,to:2026-10-10,view:A}"),
    ("pichle mahine ka commission", "period{kind:last_month} | analytics{from:2026-09-01,to:2026-09-30,view:A}"),
    ("is saal ka commission", "period{kind:this_year} | analytics{from:2026-01-01,to:2026-10-10,view:A}"),
    ("kal ki bikri", "period{kind:yesterday} | analytics{from:2026-10-09,to:2026-10-09,view:A} | report{from:2026-10-09,id:R34,to:2026-10-09}"),
    // Several flows of the shop in one question: the same reads as one of them.
    ("kal ki bikri aur commission kitna tha", "period{kind:yesterday} | analytics{from:2026-10-09,to:2026-10-09,view:A} | report{from:2026-10-09,id:R34,to:2026-10-09}"),
    ("is mahine ab tak kitni bikri aur commission hua", "period{kind:this_month} | analytics{from:2026-10-01,to:2026-10-10,view:A}"),
    ("aaj ki bikri aur commission", "tiles{}"),
    // A quantity is read from the view; the amount of the tile is not the nag.
    ("aaj kitne nag bike", "period{kind:today} | analytics{from:2026-10-10,to:2026-10-10,view:A}"),
    ("kele ke kitne nag bike is mahine", "period{kind:this_month} | find{role:item,text:Banana} | analytics{from:2026-10-01,item:i1,to:2026-10-10,view:A}"),
    // Another number than the one the views hold: no reads, the model chooses.
    ("how much seller commission did apple give this month", "period{kind:this_month} | find{role:item,text:Apple} | analytics{from:2026-10-01,item:i2,to:2026-10-10,view:A}"),
    ("partner a ka khalis kitna hai", "NONE"),
    ("pichle hafte ka commission", "period{kind:last_week} | analytics{from:2026-09-28,to:2026-10-04,view:A}"),
    ("october ka commission", "period{from:2026-10-01,kind:range,to:2026-10-10} | analytics{from:2026-10-01,to:2026-10-10,view:A}"),
    ("1 se 5 tak ka commission", "period{from:2026-10-01,kind:range,to:2026-10-05} | analytics{from:2026-10-01,to:2026-10-05,view:A}"),
    // ---- the tiles: cash, dues, payables, advances, dead amount, money lent, goods left
    ("roqar kitni hai", "tiles{}"),
    ("cash kitna hai", "tiles{}"),
    ("how much cash do we have", "tiles{}"),
    ("gala kitna hai", "tiles{}"),
    ("روکڑ کتنی ہے", "tiles{}"),
    // The shop's dues and payables are read from the lists, so the button opens the real list with
    // the total over the whole filter (lead ruling of 04-11-2026). The dues also read the tiles: the
    // answer gives the tile's figure, which is checked against R20. One party's stay on his card.
    ("buyers ke dues kitne hain", "tiles{} | report{id:R20,sort:balance:desc}"),
    ("kul ugrahi kitni hai", "tiles{} | report{id:R20,sort:balance:desc}"),
    ("total dues", "tiles{} | report{id:R20,sort:balance:desc}"),
    ("gahakon ke kul bakaya", "tiles{} | report{id:R20,sort:balance:desc}"),
    ("sab gahakon ka bakaya", "report{id:R20,sort:balance:desc}"),
    ("sellers ka payable kitna hai", "report{id:R24,sort:payable:desc}"),
    ("sellers ka kul payable", "report{id:R24,sort:payable:desc}"),
    ("beopariyon ko kul kitna dena hai", "report{id:R24,sort:payable:desc}"),
    // What was GIVEN is an event, not what is open: the tile and the card hold what is open.
    ("kitni peshgi di hui hai", "NONE"),
    ("doobi raqam kitni hai", "tiles{}"),
    ("udhar raqam kitni hai", "tiles{}"),
    ("baqi maal kitna hai", "tiles{} | report{id:R10,sort:days:desc}"),
    ("stock kitna hai", "tiles{} | report{id:R10,sort:days:desc}"),
    // ---- reports of a period: expenses, day summary, day book, receipts, trial balance
    ("kharcha is mahine ka", "period{kind:this_month} | report{from:2026-10-01,id:R29,to:2026-10-10}"),
    ("pichle mahine ka kharcha", "period{kind:last_month} | report{from:2026-09-01,id:R29,to:2026-09-30}"),
    ("aaj ka kharcha", "period{kind:today} | report{from:2026-10-10,id:R29,to:2026-10-10}"),
    ("bijli ka kharcha pichle mahine", "period{kind:last_month} | report{from:2026-09-01,id:R29,rows:100,to:2026-09-30}"),
    ("aaj ka khulasa", "period{kind:today} | report{from:2026-10-10,id:R34,to:2026-10-10}"),
    ("kal ka khulasa", "period{kind:yesterday} | report{from:2026-10-09,id:R34,to:2026-10-09}"),
    ("trial balance", "report{id:R18}"),
    ("roznamcha aaj ka", "period{kind:today} | report{from:2026-10-10,id:R11,to:2026-10-10}"),
    ("pichle mahine ki wasooli", "period{kind:last_month} | report{from:2026-09-01,id:R23,to:2026-09-30}"),
    ("is mahine kitni wasooli hui", "period{kind:this_month} | report{from:2026-10-01,id:R23,to:2026-10-10}"),
    // ---- partners (R31); the city, the adda
    ("partners ki kul raqam", "report{id:R31}"),
    ("partner b ki kul raqam", "report{id:R31}"),
    ("partner a ki kul raqam", "report{id:R31}"),
    // R31 runs in the kul raqam (E2E-37: b 50,746, his capital 50,000), so his capital is not its figure.
    // His capital now is R31 for him alone (two places); what he put in or took out is the model's.
    ("partner b ka sarmaya", "find{role:partner,text:b} | report{id:R31,partner:s2}"),
    ("partner b ka sarmaya kitna laga hua hai", "find{role:partner,text:b} | report{id:R31,partner:s2}"),
    ("partner b ne sarmaya wapas liya tha ab kitna reh gaya", "NONE"),
    ("partner b ne kitna sarmaya lagaya", "NONE"),
    ("lahore ki ugrahi", "find{role:city,text:Lahore} | report{city:$lahore,id:R21}"),
    ("swat adda ki bikri pichle mahine", "period{kind:last_month} | find{role:adda,text:Swat Adda} | report{adda:$swat,from:2026-09-01,id:R8,to:2026-09-30}"),
    // ---- lists and top N
    ("top 5 buyers", "period{kind:this_year} | analytics{from:2026-01-01,to:2026-10-10,view:H}"),
    ("top 3 sellers pichle mahine", "period{kind:last_month} | analytics{from:2026-09-01,to:2026-09-30,view:H}"),
    ("sab se zyada commission wala item", "period{kind:this_year} | analytics{from:2026-01-01,to:2026-10-10,view:H}"),
    ("sab se kam commission wala item", "period{kind:this_year} | analytics{from:2026-01-01,sort:total_comm:asc,to:2026-10-10,view:A}"),
    ("kis kis ke dues hain", "report{id:R20,sort:balance:desc}"),
    ("sab se zyada bakaya kis ka hai", "NONE"),
    ("sellers ke payable ki list", "report{id:R24,sort:payable:desc}"),
    ("top 5 items by sale this month", "period{kind:this_month} | analytics{from:2026-10-01,rows:5,sort:gross:desc,to:2026-10-10,view:A}"),
    ("kaun sa maal baqi hai", "tiles{} | report{id:R10,sort:days:desc}"),
    // ---- not covered: the model chooses
    ("akram aur arshad ka bakaya", "find{role:buyer,text:akram} | find{role:buyer,text:arshad} | party_card{party:$akram} | party_card{party:$arshad}"),
    ("faisal ka bakaya aur commission", "NONE"),
    ("kele aur saib ka commission", "find{role:item,text:Banana} | find{role:item,text:Apple} | analytics{item:$kela,view:A} | analytics{item:$saib,view:A}"),
    ("agle mahine ka commission", "estimate{ahead:1,measure:commission_all}"),
    ("sale kaise barhayen", "NONE"),
    ("wasooli kaise likhun", "NONE"),
    ("akram ki wasooli likh do", "NONE"),
    ("weather kaisa hai", "NONE"),
    ("zubair ka bakaya", "NONE"),
    ("bagh ki bikri", "NONE"),
    ("safar ka hisab", "NONE"),
    ("hello", "NONE"),
    ("pichle mahine aur is mahine ka commission", "period{kind:this_month} | period{kind:last_month} | analytics{from:2026-10-01,to:2026-10-10,view:A} | analytics{from:2026-09-01,to:2026-09-30,view:A}"),
    ("akram ka bakaya pichle mahine", "NONE"),
    ("akram ki aaj ki bikri ka print", "NONE"),
    ("akram ka phone number", "NONE"),
    ("rate kya hai", "NONE"),
    ("commission kitna hai", "NONE"),
    ("bikri kitni hui", "NONE"),
    ("kharcha kitna hai", "NONE"),
    ("akram ne kele kitne liye pichle mahine", "NONE"),
    ("pichle mahine ka cash book", "period{kind:last_month} | report{from:2026-09-01,id:R12,to:2026-09-30}"),
    // ---- BUG-3: a day with "tak" is the books up to that day (the first day of the books is 31-08-2026)
    ("faisal ki kal tak ki bikri", "period{from:2026-08-31,kind:range,to:2026-10-09} | find{role:buyer,text:faisal} | analytics{buyer:$faisal,from:2026-08-31,to:2026-10-09,view:C}"),
    ("akram ka 30 september tak ka khata", "period{from:2026-08-31,kind:range,to:2026-09-30} | find{role:buyer,text:akram} | report{from:2026-08-31,id:R13,party:$akram,to:2026-09-30}"),
    // ---- BUG-4: a unit of time alone is this one; a time word the app cannot place is the model's
    ("faisal ka mahine ka commission", "period{kind:this_month} | find{role:buyer,text:faisal} | analytics{buyer:$faisal,from:2026-10-01,to:2026-10-10,view:C}"),
    ("faisal ka saal ka commission", "period{kind:this_year} | find{role:buyer,text:faisal} | analytics{buyer:$faisal,from:2026-01-01,to:2026-10-10,view:C}"),
    ("akram ki hafte ki bikri", "period{kind:this_week} | find{role:buyer,text:akram} | analytics{buyer:$akram,from:2026-10-05,to:2026-10-10,view:C}"),
    ("rahim ki pichli bikri", "NONE"),
    ("faisal ka do mahine ka commission", "NONE"),
    // ---- BUG-5: own goods, in the right grammar too
    ("apne maal ki is mahine ki bikri kitni hai", "NONE"),
    ("apna maal is mahine ki bikri", "NONE"),
    ("hamare apne mal ki bikri aaj", "NONE"),
    // ---- R-e: what was given, paid or received is an event
    ("rahim ko kitni peshgi di", "NONE"),
    ("rahim ko kitni adaigi hui", "NONE"),
    ("akram ko kitne paise diye", "NONE"),
    // ---- a payable question about a named party stays a payable question: his card has the field
    ("karim ki adaigi kitni baqi hai", "find{role:seller,text:karim} | party_card{party:$karim}"),
    // A buyer's card holds no advance (a payment to a buyer is cash lent).
    ("akram ki peshgi kitni hai", "NONE"),
    // ---- R-h: a range that looks ahead (its last day has not come) is not sure
    ("1 se 15 october tak ka commission", "NONE"),
    ("1 se 5 october tak ka commission", "period{from:2026-10-01,kind:range,to:2026-10-05} | analytics{from:2026-10-01,to:2026-10-05,view:A}"),
    // ---- R-a: a spelling that differs in vowels only is the same name; a consonant off is not
    ("akrm ka bakaya", "find{role:buyer,text:akrm} | party_card{party:$akram}"),
    ("arshd ka bakaya", "find{role:buyer,text:arshd} | party_card{party:$arshad}"),
];

#[test]
fn the_worked_day_plans_the_reads_the_model_would_have_chosen() {
    let w = books();
    let today = on(2026, 10, 10);
    let keys = keys(&w, today);
    let wrong = check_table(&w, today, &keys, WORKED_DAY);
    assert!(
        wrong.is_empty(),
        "{} of {} wrong:\n{}",
        wrong.len(),
        WORKED_DAY.len(),
        wrong.join("\n")
    );
    assert!(WORKED_DAY.len() >= 100, "{}", WORKED_DAY.len());
}

/// Every planned read is accepted by the real tool, one after the other: the inputs are strict, so a
/// field that is wrong, a key that no `find` before it showed, or a date that is not a date fails
/// here. This is what the loop does before call 1.
#[test]
fn every_planned_read_is_accepted_by_the_real_tools() {
    let w = books();
    let today = on(2026, 10, 10);
    let mut refused = Vec::new();
    let mut planned = 0usize;
    for (question, _) in WORKED_DAY {
        let u = understand(&reader(&w, today), question);
        if let Some(reads) = plan_reads(&u) {
            planned += 1;
            refused.extend(run_plan(&w, today, question, &reads));
        }
    }
    assert!(planned >= 80, "only {planned} planned");
    assert!(refused.is_empty(), "refused:\n{}", refused.join("\n"));
}

// ---- the new tool inputs (plan 55 §11.1, backend3 and backend4) -----------------------------------------

/// Questions about the app's own state and lists, the truck, the guests' fruit, one voucher type, and
/// a real figure with a forecast: the reads they plan with the new inputs (`app_state`, `find` for the
/// staff, the expense heads and the trucks, R37 with `guests`, R15 with `voucher_type`, `estimate`).
/// None of these has an instant answer: the model writes from the reads.
#[rustfmt::skip]
const NEW_INPUTS: &[(&str, &str)] = &[
    ("rahim par commission kitna lagta hai", "find{role:seller,text:rahim} | app_state{party:a18,what:rules}"),
    ("kele ka laga rate", "find{role:item,text:Banana} | app_state{item:i1,what:rules}"),
    ("aaj ke kaam", "app_state{what:attention}"),
    ("kya baqi hai", "app_state{what:attention}"),
    ("kaunse din band hain", "app_state{what:days}"),
    ("mahina band hua", "app_state{what:days}"),
    ("kitne munshi hain", "find{role:staff,text:munshi}"),
    // Who changed what: the newest rows of the audit log, or those of one party or one بکری.
    ("kisne kya badla", "app_state{what:audit}"),
    ("aakhri changes kya hue", "app_state{what:audit}"),
    ("recent changes", "app_state{what:audit}"),
    ("rahim ka khata kisne badla", "find{role:seller,text:rahim} | app_state{party:a18,what:audit}"),
    ("kisne badla kaise dekhun", "NONE"),
    ("how many staff do we have", "find{role:staff,text:all}"),
    ("kitne expense heads hain", "find{role:expense_head,text:all}"),
    ("kitni packing hain", "find{role:packing,text:all}"),
    ("kitne marka hain", "find{role:marka,text:all}"),
    ("kitne store hain", "find{role:store,text:all}"),
    ("how many narrations are there", "find{role:narration,text:all}"),
    ("LES-1234 ki bikri", "find{role:truck,text:LES-1234}"),
    ("mehmano ka phal", "report{id:R37,partner:guests}"),
    ("mehmano ka phal is mahine", "period{kind:this_month} | report{from:2026-10-01,id:R37,partner:guests,to:2026-10-10}"),
    ("how much fruit did partner A take home", "find{role:partner,text:a} | report{id:R37,partner:s1}"),
    ("sirf wasooli ki list", "report{id:R15,voucher_type:receipt}"),
    ("sirf adaigi ki list is mahine", "period{kind:this_month} | report{from:2026-10-01,id:R15,to:2026-10-10,voucher_type:payment}"),
    // A real figure and a forecast: both parts planned, so the model writes both from what is read.
    ("pichly mahine bijli ka kitna kharcha, aur agle 3 mahine ka estimate", "period{kind:last_month} | find{role:expense_head,text:Electricity} | report{from:2026-09-01,id:R29,rows:100,to:2026-09-30} | estimate{ahead:3,head:eh1,measure:shop_expenses}"),
    ("agle 3 mahine ka commission", "estimate{ahead:3,measure:commission_all}"),
    ("agle 3 mahine ka nuqsan", "estimate{ahead:3,measure:profit}"),
    ("is saal ke aakhir tak sale ka andaza", "estimate{measure:sales,until:year_end}"),
    // A target, a name not found or a figure the estimate has no measure for (the dead amount) stays with the model.
    ("agle 6 mahine mein 500000 commission kab hoga", "NONE"),
    ("agle mahine ka commission", "estimate{ahead:1,measure:commission_all}"),
    // Cash in hand has a measure of its own now (cash_in_hand): the tile and the estimate are read.
    ("agle 3 mahine ka cash", "tiles{} | estimate{ahead:3,measure:cash_in_hand}"),
];

#[test]
fn the_new_tool_inputs_are_planned_and_the_real_tools_accept_them() {
    let w = books();
    let today = on(2026, 10, 10);
    let wrong = check_table(&w, today, &HashMap::new(), NEW_INPUTS);
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
    let mut refused = Vec::new();
    for (question, _) in NEW_INPUTS {
        let u = understand(&reader(&w, today), question);
        if let Some(reads) = plan_reads(&u) {
            refused.extend(run_plan(&w, today, question, &reads));
        }
    }
    assert!(refused.is_empty(), "refused:\n{}", refused.join("\n"));
}

// ---- the rich year (today 05-11-2026) -------------------------------------------------------------------

fn year_keys(w: &world::World, today: BusinessDate) -> HashMap<&'static str, String> {
    let r = reader(w, today);
    let mut out = HashMap::new();
    for (name, question) in [
        ("faisal", "faisal ka bakaya"),
        ("usman_ghani", "usman ghani ka bakaya"),
        ("usman", "partner usman ki kul raqam"),
        ("alif", "alif ki kul raqam"),
        ("rahim", "rahim ko dena hai"),
    ] {
        let u = understand(&r, question);
        let hit = u
            .parties
            .first()
            .and_then(|p| p.outcome.hit())
            .unwrap_or_else(|| panic!("{name} is not in the books"));
        out.insert(name, hit.key.clone());
    }
    for (name, question) in [
        ("kela", "kele ka commission"),
        ("saib", "saib ka commission"),
    ] {
        let u = understand(&r, question);
        out.insert(
            name,
            u.items
                .first()
                .and_then(|i| i.outcome.hit())
                .map(|h| h.key.clone())
                .unwrap_or_else(|| panic!("{name} is not in the books")),
        );
    }
    out
}

#[rustfmt::skip]
const YEAR: &[(&str, &str)] = &[
    ("faisal ka bakaya kitna hai", "find{role:buyer,text:faisal} | party_card{party:$faisal}"),
    ("usman ghani ka bakaya kitna hai", "find{role:buyer,text:usman ghani} | party_card{party:$usman_ghani}"),
    // Usman is a buyer and a partner: with no plain word the app is not sure which.
    ("usman ka bakaya", "NONE"),
    ("partner usman ki kul raqam", "report{id:R31}"),
    ("alif ki kul raqam", "report{id:R31}"),
    ("kele ka commission pichle mahine", "period{kind:last_month} | find{role:item,text:Banana} | analytics{from:2026-10-01,item:$kela,to:2026-10-31,view:A}"),
    ("kele ka commission is saal", "period{kind:this_year} | find{role:item,text:Banana} | analytics{from:2026-01-01,item:$kela,to:2026-11-05,view:A}"),
    ("top 5 buyers is saal", "period{kind:this_year} | analytics{from:2026-01-01,to:2026-11-05,view:H}"),
    ("sab se zyada commission kis item ka", "period{kind:this_year} | analytics{from:2026-01-01,to:2026-11-05,view:H}"),
    ("sab se zyada bikri kis item ki", "period{kind:this_year} | analytics{from:2026-01-01,sort:gross:desc,to:2026-11-05,view:A}"),
    ("aaj ki bikri kitni hui", "tiles{}"),
    ("baqi maal kitna hai", "tiles{} | report{id:R10,sort:days:desc}"),
    ("kharcha pichle mahine ka", "period{kind:last_month} | report{from:2026-10-01,id:R29,to:2026-10-31}"),
    ("pichle saal ka commission", "period{kind:last_year} | analytics{from:2025-01-01,to:2025-12-31,view:A}"),
    ("pichle 3 mahine ka commission", "period{from:2026-08-05,kind:range,to:2026-11-05} | analytics{from:2026-08-05,to:2026-11-05,view:A}"),
    ("rahim ki bikri october mein", "period{from:2026-10-01,kind:range,to:2026-10-31} | find{role:seller,text:rahim} | analytics{from:2026-10-01,seller:$rahim,to:2026-10-31,view:B}"),
    ("cash kitna hai", "tiles{}"),
    ("dues kitne hain", "tiles{} | report{id:R20,sort:balance:desc}"),
    // A garden, a trip or own goods has no filter in the reads: the model chooses.
    ("shahpur garden ka result", "NONE"),
    ("shahpur bagh ki bikri", "NONE"),
    ("14 august ka safar ka result", "NONE"),
    ("kela aur saib ka commission", "find{role:item,text:Banana} | find{role:item,text:Apple} | analytics{item:$kela,view:A} | analytics{item:$saib,view:A}"),
    ("akram ka bakaya aur arshad ka bakaya", "NONE"),
    ("aslam ka udhar kitna hai", "NONE"),
];

#[test]
fn the_rich_year_plans_the_same_way() {
    let y = year::seed::build_thin();
    let today = on(2026, 11, 5);
    let keys = year_keys(&y.w, today);
    let wrong = check_table(&y.w, today, &keys, YEAR);
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
    // The reads are accepted by the real tools in this world too.
    let mut refused = Vec::new();
    for (question, _) in YEAR {
        let u = understand(&reader(&y.w, today), question);
        if let Some(reads) = plan_reads(&u) {
            refused.extend(run_plan(&y.w, today, question, &reads));
        }
    }
    assert!(refused.is_empty(), "{}", refused.join("\n"));
}

// ---- the shape of every read ---------------------------------------------------------------------------

/// The fields of each tool's input, from the schema the model is given (`prompt/tools.json`).
fn schema_fields() -> HashMap<String, Vec<String>> {
    let tools: Value =
        serde_json::from_str(include_str!("../prompt/tools.json")).expect("tools.json");
    let mut out = HashMap::new();
    for tool in tools.as_array().expect("a list") {
        let function = tool.get("function").unwrap_or(tool);
        let name = function["name"].as_str().expect("a name").to_owned();
        let mut fields: Vec<String> = function["parameters"]["properties"]
            .as_object()
            .map(|o| o.keys().cloned().collect())
            .unwrap_or_default();
        fields.sort();
        out.insert(name, fields);
    }
    out
}

/// A planned read has exactly the fields of its tool's schema (every one, `null` when not set), the
/// `period` read comes first, and every `find` comes before the reads that name a record.
#[test]
fn every_read_has_every_field_of_its_schema_and_the_order_is_period_finds_reads() {
    let schema = schema_fields();
    let w = books();
    let today = on(2026, 10, 10);
    let mut seen = 0usize;
    for (question, _) in WORKED_DAY {
        let u = understand(&reader(&w, today), question);
        let Some(reads) = plan_reads(&u) else {
            continue;
        };
        seen += 1;
        let mut rank = 0usize;
        for read in &reads {
            let mut fields: Vec<String> = read
                .input
                .as_object()
                .unwrap_or_else(|| panic!("{question:?}: not an object"))
                .keys()
                .cloned()
                .collect();
            fields.sort();
            assert_eq!(
                Some(&fields),
                schema.get(read.tool),
                "{question:?}: {}",
                render(read)
            );
            let this = match read.tool {
                "period" => 0,
                "find" => 1,
                _ => 2,
            };
            assert!(this >= rank, "{question:?}: {}", render_all(&reads));
            rank = this;
        }
        // One period, or up to four for a comparison or a month by month question.
        assert!(
            reads.iter().filter(|r| r.tool == "period").count() <= 4,
            "{question:?}"
        );
    }
    assert!(seen >= 80, "{seen}");
}

/// The exact input of a few reads, field by field: the shape the model sends.
#[test]
fn a_few_reads_in_full() {
    let w = books();
    let today = on(2026, 10, 10);
    let akram = keys(&w, today)["akram"].clone();
    let plan = |q: &str| plan_reads(&understand(&reader(&w, today), q)).expect("a plan");
    let party = plan("akram ka bakaya kitna hai");
    assert_eq!(party[0].tool, "find");
    assert_eq!(
        party[0].input,
        serde_json::json!({"text": "akram", "role": "buyer"})
    );
    assert_eq!(party[1].tool, "party_card");
    assert_eq!(party[1].input, serde_json::json!({"party": akram}));
    let view = plan("rahim ka commission pichle mahine");
    let last = view.last().expect("a read");
    assert_eq!(last.tool, "analytics");
    let seller = keys(&w, today)["rahim"].clone();
    assert_eq!(
        last.input,
        serde_json::json!({
            "view": "B", "from": "2026-09-01", "to": "2026-09-30", "group_by": null, "item": null,
            "item_group": null, "grade": null, "seller": seller, "buyer": null, "city": null, "adda": null, "garden": null,
            "trip": null, "goods": null, "partner": null, "side": null, "compare": null,
            "sort": null, "rows": null
        })
    );
    let day = plan("aaj ka khulasa");
    assert_eq!(day[0].tool, "period");
    assert_eq!(
        day[0].input,
        serde_json::json!({
            "kind": "today", "month": null, "day": null, "from": null, "to": null, "n": null
        })
    );
    assert_eq!(
        day[1].input,
        serde_json::json!({
            "id": "R34", "from": "2026-10-10", "to": "2026-10-10", "as_of": null, "party": null,
            "item": null, "city": null, "adda": null, "collector": null, "partner": null,
            "trip": null, "garden": null, "bikri": null, "seller": null, "zamindar": null,
            "status": null, "voucher_type": null, "layout": null, "group_by": null,
            "where": null, "sort": null, "rows": null, "item_group": null
        })
    );
    let tiles = plan("cash kitna hai");
    assert_eq!(tiles.len(), 1);
    assert_eq!(tiles[0].input, serde_json::json!({}));
}

/// A question the app is not sure of, or with a locked partner section, plans nothing.
#[test]
fn what_is_not_sure_plans_no_reads() {
    let w = books();
    let today = on(2026, 10, 10);
    let locked = Reader {
        partner_locked: true,
        ..reader(&w, today)
    };
    for question in [
        "partners ki kul raqam",
        "partner b ki kul raqam",
        "alif ki kul raqam",
    ] {
        let u = understand(&locked, question);
        assert!(plan_reads(&u).is_none(), "{question}: {u:?}");
    }
    // The same questions with the section open are planned.
    for question in ["partners ki kul raqam", "partner b ki kul raqam"] {
        assert!(plan_reads(&understand(&reader(&w, today), question)).is_some());
    }
    // Close names, two parties and an empty question plan nothing.
    for question in [
        "",
        "hello",
        "akram",
        // More than three things named, or parties and items together: the model reads them.
        "akram aur arshad aur faisal aur rahim ka bakaya",
        "akram aur arshad ka kele ka commission",
    ] {
        assert!(
            plan_reads(&understand(&reader(&w, today), question)).is_none(),
            "{question:?}"
        );
    }
}

/// The audit of one بکری is read with its number; the real tool needs the بکری to exist.
#[test]
fn the_audit_of_one_bikri_is_planned_with_its_number_and_the_tool_accepts_it() {
    let w = worked::worked_day();
    worked::events_1_to_5(&w);
    let today = on(2026, 10, 10);
    let u = understand(&reader(&w, today), "bikri 1 mein kisne kya badla");
    let reads = plan_reads(&u).expect("reads");
    assert_eq!(render_all(&reads), "app_state{bikri:1,what:audit}");
    let refused = run_plan(&w, today, "bikri 1 mein kisne kya badla", &reads);
    assert!(refused.is_empty(), "refused: {refused:?}");
}

// ---- vegetables like fruit, and the group words (Umar: the shop sells vegetables as well as fruit) ----

/// The vegetables the test books get besides مولی (Radish), which the worked day holds: (Urdu,
/// English). The item master decides the names; the lexicon only routes the words to them.
const VEGETABLES: &[(&str, &str)] = &[
    ("آلو", "Potato"),
    ("پیاز", "Onion"),
    ("ٹماٹر", "Tomato"),
    ("ساگ", "Saag"),
    ("گوبھی", "Cauliflower"),
    ("گاجر", "Carrot"),
    ("بھنڈی", "Okra"),
    ("مرچ", "Chilli"),
    ("ادرک", "Ginger"),
    ("لہسن", "Garlic"),
    ("کدو", "Pumpkin"),
    ("پالک", "Spinach"),
];

fn books_with_vegetables() -> world::World {
    let w = books();
    for (ur, en) in VEGETABLES {
        world::add_item(&w.env, &w.owner, ur, en, "vegetable", "unit");
    }
    w
}

/// Common vegetables are understood in Roman, Urdu and English, with the usual misspellings, as fruit
/// is: the word finds the item of the books.
#[test]
fn common_vegetables_are_understood_in_roman_urdu_and_english_with_misspellings() {
    let w = books_with_vegetables();
    let today = on(2026, 10, 10);
    let words: &[(&str, &str)] = &[
        ("aloo", "Potato"),
        ("alu", "Potato"),
        ("aaloo", "Potato"),
        ("potato", "Potato"),
        ("potatoes", "Potato"),
        ("آلو", "Potato"),
        ("piyaz", "Onion"),
        ("pyaz", "Onion"),
        ("piaz", "Onion"),
        ("pyaaz", "Onion"),
        ("onion", "Onion"),
        ("onions", "Onion"),
        ("پیاز", "Onion"),
        ("tamatar", "Tomato"),
        ("tamater", "Tomato"),
        ("tamatr", "Tomato"),
        ("tomato", "Tomato"),
        ("tomatoes", "Tomato"),
        ("ٹماٹر", "Tomato"),
        ("mooli", "Radish"),
        ("muli", "Radish"),
        ("radish", "Radish"),
        ("مولی", "Radish"),
        ("saag", "Saag"),
        ("sag", "Saag"),
        ("ساگ", "Saag"),
        ("gobi", "Cauliflower"),
        ("gobhi", "Cauliflower"),
        ("گوبھی", "Cauliflower"),
        ("gajar", "Carrot"),
        ("carrots", "Carrot"),
        ("گاجر", "Carrot"),
        ("bhindi", "Okra"),
        ("bhendi", "Okra"),
        ("بھنڈی", "Okra"),
        ("mirch", "Chilli"),
        ("mirchi", "Chilli"),
        ("chilli", "Chilli"),
        ("مرچ", "Chilli"),
        ("adrak", "Ginger"),
        ("ginger", "Ginger"),
        ("ادرک", "Ginger"),
        ("lehsan", "Garlic"),
        ("lahsan", "Garlic"),
        ("lasan", "Garlic"),
        ("لہسن", "Garlic"),
        ("kaddu", "Pumpkin"),
        ("kadu", "Pumpkin"),
        ("pumpkin", "Pumpkin"),
        ("کدو", "Pumpkin"),
        ("palak", "Spinach"),
        ("paalak", "Spinach"),
        ("spinach", "Spinach"),
        ("پالک", "Spinach"),
    ];
    let mut wrong = Vec::new();
    for (word, english) in words {
        let urdu = word.chars().any(|c| ('\u{0600}'..='\u{06FF}').contains(&c));
        let question = if urdu {
            format!("{word} کا کمیشن اس مہینے")
        } else {
            format!("{word} ka commission is mahine")
        };
        let u = understand(&reader(&w, today), &question);
        let got = u.items.first().and_then(|i| match &i.outcome {
            mandi_assistant::understand::Outcome::Found(hit) => Some(hit.name_en.as_str()),
            _ => None,
        });
        if got != Some(*english) {
            wrong.push(format!("{question}: got {got:?}, wanted {english}"));
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// "sabzi" and "phal" are groups of items. No view and no report of the app can filter by a group yet,
/// so the whole shop must never be read and shown for them: a group question has no instant answer,
/// print of the whole shop and, until the tools' schema has the `item_group` filter, no planned read.
/// When the schema has it, view A is read with the group. The rules of a group are read with `app_state`, which
/// takes the word, and a vegetable has no default laga: the answer comes from the rule the books hold.
#[test]
fn a_group_word_is_a_group_and_never_the_whole_shop() {
    let w = books_with_vegetables();
    let today = on(2026, 10, 10);
    let ready = include_str!("../prompt/tools.json").contains("\"item_group\"");
    for (question, group) in [
        ("is mahine sabzi ki bikri kitni hui", "vegetable"),
        ("sabziyon par kitna commission mila is mahine", "vegetable"),
        ("vegetables ka commission is mahine", "vegetable"),
        ("سبزی کی بکری اس مہینے", "vegetable"),
        ("phal ki bikri is mahine", "fruit"),
        ("پھل کا کمیشن اس مہینے", "fruit"),
        ("fruit ka laga is mahine", "fruit"),
    ] {
        let u = understand(&reader(&w, today), question);
        assert_eq!(
            u.item_group
                .map(mandi_assistant::understand::GroupWord::tool_word),
            Some(group),
            "{question}"
        );
        let reads = plan_reads(&u);
        if ready {
            let reads = reads.unwrap_or_else(|| panic!("{question}: no reads"));
            assert!(
                reads
                    .iter()
                    .any(|r| r.tool == "analytics" && r.input["item_group"] == group),
                "{question}: {}",
                render_all(&reads)
            );
        } else {
            assert!(
                reads.is_none(),
                "{question}: the whole shop was read for a group"
            );
        }
        // A print of a group carries the group filter, or has no button: never the whole shop.
        let print = understand(&reader(&w, today), &format!("{question} ka print"));
        if let Some(plan) = &print.report {
            assert_eq!(
                plan.filters
                    .item_group
                    .map(mandi_assistant::understand::GroupWord::tool_word),
                Some(group),
                "{question}: a print of the whole shop for a group"
            );
        }
    }
    // The rules of a group, and of a named vegetable, are read with app_state.
    let wrong = check_table(
        &w,
        today,
        &HashMap::new(),
        &[
            (
                "sabzi ka laga kitna lagta hai",
                "app_state{item:vegetable,what:rules}",
            ),
            (
                "phal ka laga kitna lagta hai",
                "app_state{item:fruit,what:rules}",
            ),
            (
                "سبزی کا لاگا کتنا لگتا ہے",
                "app_state{item:vegetable,what:rules}",
            ),
            (
                "aloo ka laga kitna lagta hai",
                "find{role:item,text:Potato} | app_state{item:i5,what:rules}",
            ),
        ],
    );
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
    // A named item is not a group: fruit or vegetable, its own view A read.
    for question in ["kele ki bikri is mahine", "aloo ki bikri is mahine"] {
        let u = understand(&reader(&w, today), question);
        assert!(u.item_group.is_none(), "{question}");
        assert!(plan_reads(&u).is_some(), "{question}");
    }
}

/// Every question type reads the same for a vegetable as for fruit (Umar: "never assume fruit only"):
/// each template is asked of Banana (کیلا) and of Tomato (ٹماٹر), and the two plans, with the item's
/// name and key made the same, must be equal.
#[test]
fn every_question_type_reads_the_same_for_a_vegetable_as_for_fruit() {
    let w = books_with_vegetables();
    let today = on(2026, 10, 10);
    let key_of = |question: &str| {
        let u = understand(&reader(&w, today), question);
        let hit = u
            .items
            .first()
            .and_then(|i| i.outcome.hit())
            .unwrap_or_else(|| panic!("{question}: no item"));
        (hit.key.clone(), hit.name_en.clone())
    };
    let (banana_key, banana) = key_of("kele ka commission");
    let (tomato_key, tomato) = key_of("tamatar ka commission");
    let templates = [
        "{} ka rate aaj",
        "{} ka rate",
        "{} ka last rate",
        "{} ka commission is mahine",
        "{} ki bikri aaj",
        "{} ka laga is mahine",
        "{} kitne baki hain",
        "{} ka laga kitna lagta hai",
        "{} ka commission pichle mahine se kitna zyada hai",
        "{} kis gahak ne sab se zyada khareeda is mahine",
        "{} kis byopari ka aaya aaj",
        "{} ka print is mahine ka",
        "{} ki bikri aur commission is mahine",
        "{} ki bikri ka print aaj ka",
        "{} ka sab se bara gahak is saal",
        "{} ke sab se bade byopari is mahine",
        "{} ke sab se chote gahak is saal",
    ];
    let render = |question: &str, key: &str, name: &str| {
        let u = understand(&reader(&w, today), question);
        plan_reads(&u)
            .map_or_else(|| "NONE".to_owned(), |r| render_all(&r))
            .replace(key, "KEY")
            .replace(name, "ITEM")
    };
    let mut wrong = Vec::new();
    for template in templates {
        let fruit = render(&template.replace("{}", "kele"), &banana_key, &banana);
        let vegetable = render(&template.replace("{}", "tamatar"), &tomato_key, &tomato);
        if fruit != vegetable {
            wrong.push(format!(
                "{template}\n   fruit     {fruit}\n   vegetable {vegetable}"
            ));
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// An item of two words, and its synonyms and misspellings, finds the item of the books: "shimla
/// mirch", "simla mirch", "shimla mirchi", "capsicum", "bell pepper" and "شملہ مرچ" are Capsicum.
#[test]
fn an_item_of_two_words_and_its_synonyms_find_the_item() {
    let w = books();
    world::add_item(
        &w.env,
        &w.owner,
        "شملہ مرچ",
        "Capsicum",
        "vegetable",
        "unit",
    );
    let today = on(2026, 10, 10);
    let mut wrong = Vec::new();
    for name in [
        "shimla mirch",
        "simla mirch",
        "shimla mirchi",
        "shamla mirch",
        "capsicum",
        "bell pepper",
        "شملہ مرچ",
    ] {
        let question = if name.chars().any(|c| ('\u{0600}'..='\u{06FF}').contains(&c)) {
            format!("{name} کی اس مہینے کی بکری بتاؤ")
        } else {
            format!("{name} ki is mahine ki sale batao")
        };
        let u = understand(&reader(&w, today), &question);
        let got = u
            .items
            .first()
            .and_then(|i| i.outcome.hit())
            .map(|h| h.name_en.as_str());
        if got != Some("Capsicum") {
            wrong.push(format!("{question}: got {got:?}"));
        }
        // Its own view A read for the month, not a read of a chilli.
        let reads = plan_reads(&u).map(|r| render_all(&r)).unwrap_or_default();
        if !reads.contains("find{role:item,text:Capsicum}") {
            wrong.push(format!("{question}: reads {reads}"));
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// The reads of a group of items once the tools take the filter: stock of each vegetable, a group's
/// commission for the year, and the vegetable that sells most. Not before: the group is the model's.
#[test]
fn the_group_reads_once_the_tools_have_the_filter() {
    let ready = include_str!("../prompt/tools.json").contains("\"item_group\"");
    let w = books_with_vegetables();
    let today = on(2026, 10, 10);
    let table: &[(&str, &str)] = &[
        (
            "konsi sabzi kitni baki hai",
            "report{id:R10,item_group:vegetable,layout:item}",
        ),
        (
            "is saal sabzi se kitni commission bani",
            "period{kind:this_year} | analytics{from:2026-01-01,item_group:vegetable,to:2026-10-10,view:A}",
        ),
        (
            "kaun si sabzi sab se zyada bikti hai",
            "period{kind:this_year} | analytics{from:2026-01-01,item_group:vegetable,sort:gross:desc,to:2026-10-10,view:A}",
        ),
        (
            "kaun sa phal sab se zyada bikta hai",
            "period{kind:this_year} | analytics{from:2026-01-01,item_group:fruit,sort:gross:desc,to:2026-10-10,view:A}",
        ),
    ];
    for (question, planned) in table {
        let u = understand(&reader(&w, today), question);
        let got = plan_reads(&u).map(|r| render_all(&r));
        if ready {
            assert_eq!(got.as_deref(), Some(*planned), "{question}");
        } else {
            assert!(
                got.is_none(),
                "{question}: the whole shop was read for a group"
            );
        }
    }
}

/// "Who": a ranking of buyers or sellers for one item, or inside a group, reads the view of buyers (C)
/// or of sellers (B) with the item or the group as a filter (views B and C take `item_group` since PRD
/// v1.29). It never reads the view of items (A), which does not say who, and never the shop's top ten
/// (H), which cannot be sorted. A name that is not in the books, or two items, is the model's.
#[test]
fn a_ranking_of_buyers_or_sellers_for_an_item_or_a_group_reads_the_view_of_that_kind() {
    let w = books_with_vegetables();
    let today = on(2026, 10, 10);
    let key_of = |question: &str| {
        let u = understand(&reader(&w, today), question);
        u.items
            .first()
            .and_then(|i| i.outcome.hit())
            .map(|h| h.key.clone())
            .unwrap_or_else(|| panic!("{question}: no item"))
    };
    let tomato = key_of("tamatar ka commission");
    let banana = key_of("kele ka commission");
    let year = "period{kind:this_year}";
    let month = "period{kind:this_month}";
    let items: Vec<(&str, String)> = vec![
        (
            "tamatar ka sab se bara gahak is saal",
            format!(
                "{year} | find{{role:item,text:Tomato}} | analytics{{from:2026-01-01,item:{tomato},sort:buyer_comm:desc,to:2026-10-10,view:C}}"
            ),
        ),
        (
            "tamatar ke sab se bade byopari is mahine",
            format!(
                "{month} | find{{role:item,text:Tomato}} | analytics{{from:2026-10-01,item:{tomato},sort:total_comm:desc,to:2026-10-10,view:B}}"
            ),
        ),
        (
            "top 5 buyers of tomato this year",
            format!(
                "{year} | find{{role:item,text:Tomato}} | analytics{{from:2026-01-01,item:{tomato},rows:5,sort:buyer_comm:desc,to:2026-10-10,view:C}}"
            ),
        ),
        (
            "kele ke sab se kam gahak",
            format!(
                "{year} | find{{role:item,text:Banana}} | analytics{{from:2026-01-01,item:{banana},sort:buyer_comm:asc,to:2026-10-10,view:C}}"
            ),
        ),
        (
            "ٹماٹر کا سب سے بڑا گاہک اس سال",
            format!(
                "{year} | find{{role:item,text:Tomato}} | analytics{{from:2026-01-01,item:{tomato},sort:buyer_comm:desc,to:2026-10-10,view:C}}"
            ),
        ),
    ];
    let mut wrong = Vec::new();
    let mut check = |question: &str, want: &str| {
        let u = understand(&reader(&w, today), question);
        let got = plan_reads(&u).map_or_else(|| "NONE".to_owned(), |r| render_all(&r));
        if got != want {
            wrong.push(format!("{question:?}\n   got    {got}\n   wanted {want}"));
        }
    };
    for (question, want) in &items {
        check(question, want);
    }
    // Whose, inside a group: the view of buyers or sellers with the group, once the tools take it.
    let ready = include_str!("../prompt/tools.json").contains("\"item_group\"");
    let groups: &[(&str, &str)] = &[
        (
            "sabzi ka sab se bara gahak is saal",
            "period{kind:this_year} | analytics{from:2026-01-01,item_group:vegetable,sort:buyer_comm:desc,to:2026-10-10,view:C}",
        ),
        (
            "sabzi ke top 3 gahak is saal",
            "period{kind:this_year} | analytics{from:2026-01-01,item_group:vegetable,rows:3,sort:buyer_comm:desc,to:2026-10-10,view:C}",
        ),
        (
            "sabzi ka sab se bara byopari is saal",
            "period{kind:this_year} | analytics{from:2026-01-01,item_group:vegetable,sort:total_comm:desc,to:2026-10-10,view:B}",
        ),
        (
            "phal ke sab se bade gahak is saal",
            "period{kind:this_year} | analytics{from:2026-01-01,item_group:fruit,sort:buyer_comm:desc,to:2026-10-10,view:C}",
        ),
        (
            "سبزی کا سب سے بڑا گاہک اس سال",
            "period{kind:this_year} | analytics{from:2026-01-01,item_group:vegetable,sort:buyer_comm:desc,to:2026-10-10,view:C}",
        ),
    ];
    for (question, want) in groups {
        check(question, if ready { want } else { "NONE" });
    }
    // Not in the books, or two items: the model's.
    for question in [
        "shimla mirch ka sab se bara gahak is saal",
        "kele aur tamatar ke sab se bade gahak is saal",
    ] {
        check(question, "NONE");
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
    // The real tools take every one of these reads, as the loop would send them.
    let mut refused = Vec::new();
    for question in items
        .iter()
        .map(|(q, _)| *q)
        .chain(groups.iter().map(|(q, _)| *q))
    {
        let u = understand(&reader(&w, today), question);
        if let Some(reads) = plan_reads(&u) {
            refused.extend(run_plan(&w, today, question, &reads));
        }
    }
    assert!(refused.is_empty(), "refused:\n{}", refused.join("\n"));
}

// ---- the quality of a sale line (plan 58 T3, PRD FR-12.42) -------------------------------------------------

/// A quality word (awwal, doyam, soyam, اول, دوم, سوم, first quality, A grade, darja) reads the figures of
/// that quality: view A with the item, the period, the names and the grade (the tool's `grade` filter). With
/// no quality named ("kis kis darje mein bike") it reads the sale lines (R2), which carry each quality.
/// Never the figure of all the qualities. Roman Urdu, Urdu
/// and English. A print has no quality filter, so it is the model's. No quality word: as before.
#[rustfmt::skip]
const QUALITY: &[(&str, &str)] = &[
    ("awwal seb aaj kitne ka bika", "period{kind:today} | find{role:item,text:Apple} | analytics{from:2026-10-10,grade:first,item:i2,to:2026-10-10,view:A}"),
    ("doyam kele ka aaj ka rate", "period{kind:today} | find{role:item,text:Banana} | analytics{from:2026-10-10,grade:second,item:i1,to:2026-10-10,view:A}"),
    ("soyam kele ki is mahine ki bikri", "period{kind:this_month} | find{role:item,text:Banana} | analytics{from:2026-10-01,grade:third,item:i1,to:2026-10-10,view:A}"),
    ("rahim ke seb kis kis darje mein bike", "find{role:item,text:Apple} | find{role:seller,text:rahim} | report{id:R2,item:i2,rows:100,seller:a18}"),
    ("rahim ke awwal seb aaj kitne ke bike", "period{kind:today} | find{role:item,text:Apple} | find{role:seller,text:rahim} | analytics{from:2026-10-10,grade:first,item:i2,seller:a18,to:2026-10-10,view:A}"),
    ("arshad ne awwal seb kitne ka liya", "find{role:item,text:Apple} | find{role:buyer,text:arshad} | analytics{buyer:a21,grade:first,item:i2,view:A}"),
    ("اول سیب آج کتنے کا بکا", "period{kind:today} | find{role:item,text:Apple} | analytics{from:2026-10-10,grade:first,item:i2,to:2026-10-10,view:A}"),
    ("دوم کیلے کا آج کا ریٹ", "period{kind:today} | find{role:item,text:Banana} | analytics{from:2026-10-10,grade:second,item:i1,to:2026-10-10,view:A}"),
    ("how much first quality apple sold today", "period{kind:today} | find{role:item,text:Apple} | analytics{from:2026-10-10,grade:first,item:i2,to:2026-10-10,view:A}"),
    ("second quality banana rate today", "period{kind:today} | find{role:item,text:Banana} | analytics{from:2026-10-10,grade:second,item:i1,to:2026-10-10,view:A}"),
    ("A grade apple sales this month", "period{kind:this_month} | find{role:item,text:Apple} | analytics{from:2026-10-01,grade:first,item:i2,to:2026-10-10,view:A}"),
    // A print has no quality filter, so it would show every quality: the model's.
    ("awwal seb ka print aaj ka", "NONE"),
    // No quality word: the same question reads as before (view A).
    ("seb aaj kitne ka bika", "period{kind:today} | find{role:item,text:Apple} | analytics{from:2026-10-10,item:i2,to:2026-10-10,view:A}"),
];

#[test]
fn a_quality_word_reads_the_lines_of_the_sales_in_three_languages() {
    let w = books();
    let today = on(2026, 10, 10);
    let wrong = check_table(&w, today, &HashMap::new(), QUALITY);
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
    let mut refused = Vec::new();
    for (question, _) in QUALITY {
        let u = understand(&reader(&w, today), question);
        if let Some(reads) = plan_reads(&u) {
            refused.extend(run_plan(&w, today, question, &reads));
        }
    }
    assert!(refused.is_empty(), "refused:\n{}", refused.join("\n"));
}

// ---- one figure over two periods (PRD FR-18.1, plan 55 §14) ---------------------------------------------

/// "Pichle mahine se kitna zyada": both periods are read before the model starts, as view A for each
/// (the whole shop, one item, a group, one seller), or as one view A read with `compare` when the
/// earlier period is exactly the one the tool works out and the figure is the commission. "Last month"
/// alone means this month so far against last month. Two items are read for each period. Not a balance,
/// periods that overlap, or a period that has not come: the model's.
#[rustfmt::skip]
const COMPARE: &[(&str, &str)] = &[
    ("is mahine ka commission pichle mahine se kitna zyada hai", "period{kind:this_month} | period{kind:last_month} | analytics{from:2026-10-01,to:2026-10-10,view:A} | analytics{from:2026-09-01,to:2026-09-30,view:A}"),
    ("is mahine ki commission pichle mahine se kam hai", "period{kind:this_month} | period{kind:last_month} | analytics{from:2026-10-01,to:2026-10-10,view:A} | analytics{from:2026-09-01,to:2026-09-30,view:A}"),
    ("commission last month se kam hai ya zyada", "period{kind:this_month} | period{kind:last_month} | analytics{from:2026-10-01,to:2026-10-10,view:A} | analytics{from:2026-09-01,to:2026-09-30,view:A}"),
    ("this month commission is how much more than last month", "period{kind:this_month} | period{kind:last_month} | analytics{from:2026-10-01,to:2026-10-10,view:A} | analytics{from:2026-09-01,to:2026-09-30,view:A}"),
    ("اس مہینے کا کمیشن پچھلے مہینے سے کتنا زیادہ ہے", "period{kind:this_month} | period{kind:last_month} | analytics{from:2026-10-01,to:2026-10-10,view:A} | analytics{from:2026-09-01,to:2026-09-30,view:A}"),
    ("is mahine ki bikri aur pichle mahine ki bikri mein farq", "period{kind:this_month} | period{kind:last_month} | analytics{from:2026-10-01,to:2026-10-10,view:A} | analytics{from:2026-09-01,to:2026-09-30,view:A}"),
    ("kele ka commission pichle mahine se kitna zyada hai", "period{kind:this_month} | period{kind:last_month} | find{role:item,text:Banana} | analytics{from:2026-10-01,item:i1,to:2026-10-10,view:A} | analytics{from:2026-09-01,item:i1,to:2026-09-30,view:A}"),
    ("rahim ki bikri pichle mahine se kitni kam hai", "period{kind:this_month} | period{kind:last_month} | find{role:seller,text:rahim} | analytics{from:2026-10-01,seller:a18,to:2026-10-10,view:B} | analytics{from:2026-09-01,seller:a18,to:2026-09-30,view:B}"),
    ("is saal ka commission pichle saal se kitna zyada hai", "period{kind:this_year} | period{kind:last_year} | analytics{from:2026-01-01,to:2026-10-10,view:A} | analytics{from:2025-01-01,to:2025-12-31,view:A}"),
    // The earlier period is exactly the one the tool works out: one read with `compare`.
    ("august ke muqable september ka commission kitna zyada hai", "period{from:2026-09-01,kind:range,to:2026-09-30} | period{from:2026-08-01,kind:range,to:2026-08-31} | analytics{compare:previous,from:2026-09-01,to:2026-09-30,view:A}"),
    // Two items are read for each period. The model's: a balance is no flow, overlapping periods, a
    // period that has not come.
    ("akram ka bakaya pichle mahine se kitna zyada hai", "NONE"),
    ("kele aur tamatar ka commission pichle mahine se kitna zyada hai", "period{kind:this_month} | period{kind:last_month} | find{role:item,text:Banana} | find{role:item,text:Tomato} | analytics{from:2026-10-01,item:i1,to:2026-10-10,view:A} | analytics{from:2026-10-01,item:i7,to:2026-10-10,view:A} | analytics{from:2026-09-01,item:i1,to:2026-09-30,view:A} | analytics{from:2026-09-01,item:i7,to:2026-09-30,view:A}"),
    ("is mahine ka commission is saal se kitna zyada hai", "NONE"),
    ("agle mahine ka commission is mahine se kitna zyada hoga", "NONE"),
];

#[test]
fn one_figure_over_two_periods_reads_both_periods_before_the_model_starts() {
    let w = books_with_vegetables();
    let today = on(2026, 10, 10);
    let wrong = check_table(&w, today, &HashMap::new(), COMPARE);
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
    // A planned comparison is a sure question (the run reads from a sure understanding only).
    for (question, want) in COMPARE {
        if *want != "NONE" {
            let u = understand(&reader(&w, today), question);
            assert!(u.sure, "{question}: not sure, {:?}", u.doubts);
        }
    }
    let mut refused = Vec::new();
    for (question, _) in COMPARE {
        let u = understand(&reader(&w, today), question);
        if let Some(reads) = plan_reads(&u) {
            refused.extend(run_plan(&w, today, question, &reads));
        }
    }
    assert!(refused.is_empty(), "refused:\n{}", refused.join("\n"));
    // A group compares as every question type does, once the tools take the filter.
    let ready = include_str!("../prompt/tools.json").contains("\"item_group\"");
    let u = understand(
        &reader(&w, today),
        "sabzi ka commission pichle mahine se kitna zyada hai",
    );
    let got = plan_reads(&u).map(|r| render_all(&r));
    let want = "period{kind:this_month} | period{kind:last_month} | analytics{from:2026-10-01,item_group:vegetable,to:2026-10-10,view:A} | analytics{from:2026-09-01,item_group:vegetable,to:2026-09-30,view:A}";
    assert_eq!(got.as_deref(), ready.then_some(want));
}

// ---- the last rate of an item sold to one buyer (plan 55 §14) ---------------------------------------------

/// "Akram ko kela aakhri kis rate pe diya": the register of the sales (R2) for the buyer and the item,
/// newest first by date and entry, 5 rows; the first row is the last sale and its `rate` the answer.
/// It needs a "last" word, one buyer, one item and no period. No instant answer.
#[rustfmt::skip]
const LAST_RATE: &[(&str, &str)] = &[
    ("akram ko kela aakhri kis rate pe diya", "find{role:buyer,text:akram} | find{role:item,text:Banana} | report{id:R2,item:i1,party:a20,rows:1,sort:date:desc}"),
    ("akram ko kele ka aakhri rate kya tha", "find{role:buyer,text:akram} | find{role:item,text:Banana} | report{id:R2,item:i1,party:a20,rows:1,sort:date:desc}"),
    ("last rate of banana to akram", "find{role:buyer,text:akram} | find{role:item,text:Banana} | report{id:R2,item:i1,party:a20,rows:1,sort:date:desc}"),
    ("akram ne kela kis rate pe liya last time", "find{role:buyer,text:akram} | find{role:item,text:Banana} | report{id:R2,item:i1,party:a20,rows:1,sort:date:desc}"),
    ("اکرم کو کیلا آخری بار کس ریٹ پر دیا", "find{role:buyer,text:اکرم} | find{role:item,text:Banana} | report{id:R2,item:i1,party:a20,rows:1,sort:date:desc}"),
    // No "last" word, a period, no buyer, or "last sale" (not a rate): the model's.
    ("akram ko kela kis rate pe diya", "NONE"),
    ("akram ko kela pichle mahine kis rate pe diya", "NONE"),
    // An item with no buyer, with or without a period: the newest line of its sales, ONE row (live run 2, C8).
    ("kele ka last rate", "find{role:item,text:Banana} | report{id:R2,item:i1,rows:1,sort:date:desc}"),
    ("what was the last rate of banana today", "period{kind:today} | find{role:item,text:Banana} | report{from:2026-10-10,id:R2,item:i1,rows:1,sort:date:desc,to:2026-10-10}"),
    ("last rate of apple last week", "period{kind:last_week} | find{role:item,text:Apple} | report{from:2026-09-28,id:R2,item:i2,rows:1,sort:date:desc,to:2026-10-04}"),
    // "last week" is a period, not the last one: the average of the week, as ever.
    ("rate of apple last week", "period{kind:last_week} | find{role:item,text:Apple} | analytics{from:2026-09-28,item:i2,to:2026-10-04,view:A}"),
    ("kele ki last sale", "NONE"),
];

#[test]
fn the_last_rate_of_an_item_sold_to_a_buyer_reads_his_newest_lines() {
    let w = books();
    let today = on(2026, 10, 10);
    let wrong = check_table(&w, today, &HashMap::new(), LAST_RATE);
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
    let mut refused = Vec::new();
    for (question, _) in LAST_RATE {
        let u = understand(&reader(&w, today), question);
        if let Some(reads) = plan_reads(&u) {
            refused.extend(run_plan(&w, today, question, &reads));
        }
    }
    assert!(refused.is_empty(), "refused:\n{}", refused.join("\n"));
}

// ---- hidden round 4 (26/42) ------------------------------------------------------------------------------

fn vegetable_key(w: &world::World, today: BusinessDate, question: &str) -> String {
    let u = understand(&reader(w, today), question);
    u.items
        .first()
        .and_then(|i| i.outcome.hit())
        .map(|h| h.key.clone())
        .unwrap_or_else(|| panic!("{question}: no item"))
}

/// Item 1: "How did onion sales go in August, September and October": a figure asked in the past tense, not
/// the how-to of the sale book. View A for the onion, once for each month, before the model starts.
#[test]
fn how_did_a_thing_go_over_several_months_reads_each_month() {
    let w = books_with_vegetables();
    let today = on(2026, 10, 10);
    let onion = vegetable_key(&w, today, "pyaz ka commission");
    let u = understand(
        &reader(&w, today),
        "how did onion sales go in August, September and October",
    );
    assert_eq!(u.intent.value, Intent::Figure, "a how-to, not a figure");
    let want = format!(
        "period{{from:2026-10-01,kind:range,to:2026-10-10}} | period{{from:2026-09-01,kind:range,to:2026-09-30}} | period{{from:2026-08-01,kind:range,to:2026-08-31}} | find{{role:item,text:Onion}} | analytics{{from:2026-10-01,item:{onion},to:2026-10-10,view:A}} | analytics{{from:2026-09-01,item:{onion},to:2026-09-30,view:A}} | analytics{{from:2026-08-01,item:{onion},to:2026-08-31,view:A}}"
    );
    assert!(
        u.sure,
        "the run starts only from a sure understanding: {:?}",
        u.doubts
    );
    let reads = plan_reads(&u).expect("reads");
    assert_eq!(render_all(&reads), want);
    let refused = run_plan(&w, today, "onion months", &reads);
    assert!(refused.is_empty(), "refused: {refused:?}");
    // A real how-to stays a how-to.
    let u = understand(&reader(&w, today), "how do I print a bill");
    assert_eq!(u.intent.value, Intent::HowTo);
}

/// Item 2: "tomato" and "potato" are two letters apart, but each name typed exactly is that item alone, and
/// what Faisal bought of the item is read for him: his lines (R2) with the item, the year's totals.
#[test]
fn an_exact_english_item_name_is_taken_exactly() {
    let w = books_with_vegetables();
    let today = on(2026, 10, 10);
    for (question, name) in [
        ("how much tomato did Faisal buy this year", "Tomato"),
        ("how much potato did Faisal buy this year", "Potato"),
        (
            "is tomato selling more this month than last month",
            "Tomato",
        ),
        (
            "is potato selling more this month than last month",
            "Potato",
        ),
    ] {
        let u = understand(&reader(&w, today), question);
        let [item] = u.items.as_slice() else {
            panic!("{question}: {} items", u.items.len());
        };
        let hit = item
            .outcome
            .hit()
            .unwrap_or_else(|| panic!("{question}: not one item"));
        assert_eq!(hit.name_en, name, "{question}");
        let reads = plan_reads(&u).unwrap_or_else(|| panic!("{question}: no reads"));
        let text = render_all(&reads);
        assert!(
            text.contains(&format!("item:{}", hit.key)),
            "{question}: {text}"
        );
        let refused = run_plan(&w, today, question, &reads);
        assert!(refused.is_empty(), "{question}: {refused:?}");
    }
    let faisal = {
        let u = understand(
            &reader(&w, today),
            "how much tomato did Faisal buy this year",
        );
        render_all(&plan_reads(&u).expect("reads"))
    };
    assert!(
        faisal.contains("report{from:2026-01-01,id:R2,item:"),
        "{faisal}"
    );
}

/// Item 3: The average rate of an item in Urdu: "اوسط" is the word "average", not a name nobody has.
#[test]
fn the_urdu_average_rate_of_an_item_reads_view_a() {
    let w = books();
    let today = on(2026, 10, 10);
    let apple = {
        let u = understand(&reader(&w, today), "seb ka commission");
        u.items
            .first()
            .and_then(|i| i.outcome.hit())
            .map(|h| h.key.clone())
            .expect("apple")
    };
    let want = format!(
        "period{{kind:last_month}} | find{{role:item,text:Apple}} | analytics{{from:2026-09-01,item:{apple},to:2026-09-30,view:A}}"
    );
    for question in [
        "پچھلے مہینے سیب کا اوسط ریٹ کیا رہا",
        "pichle mahine seb ka ausat rate kya raha",
        "what was the average rate of apple last month",
    ] {
        let u = understand(&reader(&w, today), question);
        assert!(!u.has_not_found(), "{question}: a word taken for a name");
        let got = plan_reads(&u).map(|r| render_all(&r));
        assert_eq!(got.as_deref(), Some(want.as_str()), "{question}");
    }
}

/// Item 4: A misspelt Roman Urdu name with the same consonants is the item ("tmatar", "tmatr" for tamatar),
/// as a medium guess; a short or different word is not guessed ("motor" is not "matar").
#[test]
fn a_misspelt_item_name_with_the_same_consonants_is_the_item() {
    let w = books_with_vegetables();
    let today = on(2026, 10, 10);
    for word in ["tamatar", "tmatar", "tmatr", "tamatr", "tamtar", "tomatar"] {
        let question = format!("{word} ka pichle mahine ka comision");
        let u = understand(&reader(&w, today), &question);
        let hit = u
            .items
            .first()
            .and_then(|i| i.outcome.hit())
            .unwrap_or_else(|| panic!("{question}: no item"));
        assert_eq!(hit.name_en, "Tomato", "{question}");
        let reads = plan_reads(&u).unwrap_or_else(|| panic!("{question}: no reads"));
        assert!(
            render_all(&reads).contains("find{role:item,text:Tomato}"),
            "{question}"
        );
    }
    let u = understand(&reader(&w, today), "motor ka pichle mahine ka comision");
    assert!(
        u.items
            .iter()
            .all(|i| i.outcome.hit().is_none_or(|h| h.name_en != "Peas")),
        "motor is not matar"
    );
}

/// Item 5: The two rows that ran out of time get their reads before the model starts: the goods left of
/// each vegetable (R10 with the group), and the sales of an item by month (view D, all dates).
#[test]
fn the_vegetable_stock_list_and_the_months_of_an_item_are_read_first() {
    let w = books_with_vegetables();
    let today = on(2026, 10, 10);
    let radish = vegetable_key(&w, today, "mooli ka commission");
    let onion = vegetable_key(&w, today, "pyaz ka commission");
    let ready = include_str!("../prompt/tools.json").contains("\"item_group\"");
    let stock = "report{id:R10,item_group:vegetable,layout:item}";
    for question in [
        "kaun kaun si sabzi kitni baki padi hai",
        "kaun si sabzi kitni baki hai",
        "konsi sabzi kitni baki hai",
    ] {
        let u = understand(&reader(&w, today), question);
        let got = plan_reads(&u).map(|r| render_all(&r));
        assert_eq!(got.as_deref(), ready.then_some(stock), "{question}");
    }
    for (question, name, key) in [
        ("mooli kitne mahine biki hai", "Radish", &radish),
        ("how many months did onion sell", "Onion", &onion),
    ] {
        let u = understand(&reader(&w, today), question);
        let reads = plan_reads(&u).unwrap_or_else(|| panic!("{question}: no reads"));
        assert_eq!(
            render_all(&reads),
            format!(
                "find{{role:item,text:{name}}} | analytics{{from:2026-08-31,group_by:month,item:{key},to:2026-10-10,view:D}}"
            ),
            "{question}"
        );
        let refused = run_plan(&w, today, question, &reads);
        assert!(refused.is_empty(), "{question}: {refused:?}");
    }
}

/// Hidden round 4, the forecast questions in Urdu letters: "سال کے آخر تک" is the year end (its words
/// are known), "گا" and "bikega" are future suffixes and a sale verb, and a question that holds this
/// year beside the year end keeps both periods (the real part, view A for this year, is read too).
/// "Agle mahine" with no number is one month ahead (`ahead` 1, never the tool's default of 6).
#[test]
fn the_year_end_and_next_month_questions_in_urdu_letters_are_planned() {
    let w = books_with_vegetables();
    let today = on(2026, 10, 10);
    let tomato = vegetable_key(&w, today, "tamatar ka commission");
    let saag = vegetable_key(&w, today, "saag ka commission");
    let year_end = format!(
        "find{{role:item,text:Tomato}} | estimate{{item:{tomato},measure:commission_all,until:year_end}}"
    );
    let both = format!(
        "period{{kind:this_year}} | find{{role:item,text:Tomato}} | analytics{{from:2026-01-01,item:{tomato},to:2026-10-10,view:A}} | estimate{{item:{tomato},measure:commission_all,until:year_end}}"
    );
    let next =
        format!("find{{role:item,text:Saag}} | estimate{{ahead:1,item:{saag},measure:sales}}");
    let sales_both = format!(
        "period{{kind:this_year}} | find{{role:item,text:Saag}} | analytics{{from:2026-01-01,item:{saag},to:2026-10-10,view:A}} | estimate{{ahead:1,item:{saag},measure:sales}}"
    );
    let cases: Vec<(&str, &String)> = vec![
        ("ٹماٹر کا سال کے آخر تک کمیشن", &year_end),
        ("tamatar ka saal ke aakhir tak commission", &year_end),
        (
            "how much will tomato commission be by the end of the year",
            &year_end,
        ),
        (
            "ٹماٹر کا اس سال کا کمیشن کتنا ہے اور سال کے آخر تک کتنا ہو گا",
            &both,
        ),
        (
            "tamatar ka is saal ka commission kitna hai aur saal ke aakhir tak kitna hoga",
            &both,
        ),
        ("tomato commission this year so far and by year end", &both),
        ("ساگ اگلے مہینے کتنا بکے گا", &next),
        ("saag agle mahine kitna bikega", &next),
        ("saag ki sale agle mahine kitni hogi", &next),
        (
            "ساگ کی اس سال کی بکری کتنی ہے اور اگلے مہینے کتنی ہو گی",
            &sales_both,
        ),
        (
            "saag ki is saal ki sale kitni hai aur agle mahine kitni hogi",
            &sales_both,
        ),
    ];
    let mut wrong = Vec::new();
    let mut refused = Vec::new();
    for (question, want) in cases {
        let u = understand(&reader(&w, today), question);
        let got = plan_reads(&u);
        let text = got.as_deref().map(render_all);
        if text.as_deref() != Some(want.as_str()) {
            wrong.push(format!(
                "{question:?}\n   got    {text:?}\n   wanted {want}"
            ));
        }
        if let Some(reads) = got {
            refused.extend(run_plan(&w, today, question, &reads));
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
    assert!(refused.is_empty(), "refused: {refused:?}");
    // "is saal ke aakhir tak": the year belongs to the phrase, so there is no period of its own.
    let u = understand(&reader(&w, today), "is saal ke aakhir tak sale ka andaza");
    assert_eq!(
        plan_reads(&u).as_deref().map(render_all).as_deref(),
        Some("estimate{measure:sales,until:year_end}")
    );
}

// ---- live run 2: a misspelt word is never dropped (C1, C2) and the reads of H2, H4, Q072 ------------------

/// C1: a group word spelt wrong is still the group. "How much was sold of vegitables last week" read the
/// whole shop (704,040 for an expected 0). The group is in the read, whatever the spelling.
#[test]
fn a_group_word_spelt_wrong_is_still_the_group() {
    let w = books_with_vegetables();
    let today = on(2026, 10, 10);
    let ready = include_str!("../prompt/tools.json").contains("\"item_group\"");
    for (question, group) in [
        ("how much was sold of vegitables last week", "vegetable"),
        ("how much was sold of vegetabls last week", "vegetable"),
        ("how much was sold of vegetables last week", "vegetable"),
        ("subzi ki bikri pichle hafte", "vegetable"),
        ("sabziyon ki bikri pichle hafte", "vegetable"),
        ("how much was sold of fruts last week", "fruit"),
        ("phl ki bikri pichle hafte", "fruit"),
    ] {
        let u = understand(&reader(&w, today), question);
        let got = plan_reads(&u).map(|r| render_all(&r));
        match (&got, ready) {
            (Some(text), true) => {
                assert!(
                    text.contains(&format!("item_group:{group}")),
                    "{question}: {text}"
                );
                assert!(
                    text.contains("period{kind:last_week}"),
                    "{question}: {text}"
                );
            }
            (None, false) => {}
            other => panic!("{question}: {other:?}"),
        }
    }
}

/// C2: a period word spelt wrong is a period. "kelaa ka coomision is sall kitna bana" answered for all
/// the books. A time word that is neither read nor known (a four-letter word one letter off "sal") makes
/// the question not sure, so there is no instant figure and no read for the wrong dates.
#[test]
fn a_period_word_spelt_wrong_is_never_dropped() {
    let w = books();
    let today = on(2026, 10, 10);
    for (question, period) in [
        ("kelaa ka coomision is sall kitna bana", "this_year"),
        ("kela ka commission is sall", "this_year"),
        ("kela ka commission is sail", "this_year"),
        ("kela ka commission is saal", "this_year"),
        ("kela ka commission is mahne", "this_month"),
        ("kela ka commission is mhina", "this_month"),
        ("kela ka commission pichle hafty", "last_week"),
        ("kela ka commission pichle hfta", "last_week"),
    ] {
        let u = understand(&reader(&w, today), question);
        let text = plan_reads(&u).map(|r| render_all(&r)).unwrap_or_default();
        assert!(
            text.contains(&format!("period{{kind:{period}}}")),
            "{question}: {text}"
        );
    }
    // A time word nothing reads: not sure, and no read for dates the question did not name.
    let u = understand(&reader(&w, today), "kela ka commission is sakl");
    assert!(!u.sure, "an unread time word left the question sure");
    assert!(plan_reads(&u).is_none());
}

/// H4: "kis se pehle wasooli karon" asks whom to collect from first: the buyers' dues oldest first
/// (R20 sorted by days), never the shop's one total. A named buyer is another question.
#[test]
fn whom_to_collect_from_first_reads_the_dues_oldest_first() {
    let w = books();
    let today = on(2026, 10, 10);
    for question in ["kis se pehle wasooli karon", "kis se pahle wasuli karon"] {
        let u = understand(&reader(&w, today), question);
        let got = plan_reads(&u).map(|r| render_all(&r));
        assert_eq!(
            got.as_deref(),
            Some("report{id:R20,rows:5,sort:days:desc}"),
            "{question}"
        );
        let reads = plan_reads(&u).expect("reads");
        let refused = run_plan(&w, today, question, &reads);
        assert!(refused.is_empty(), "{question}: {refused:?}");
    }
    let u = understand(&reader(&w, today), "akram se pehle wasooli");
    assert_ne!(
        plan_reads(&u).map(|r| render_all(&r)).as_deref(),
        Some("report{id:R20,rows:5,sort:days:desc}")
    );
}

/// Q072: "aaj commission bikri ka kitne fisad" is one flow over another: both are read (today's view A
/// holds the sale and the commission with Laga) and the model divides them, not the commission rule's
/// 6% (the figure test has the negative: no instant figure).
#[test]
fn a_percent_of_one_flow_over_another_reads_both_and_has_no_instant_figure() {
    let w = books();
    let today = on(2026, 10, 10);
    for question in [
        "aaj commission bikri ka kitne fisad",
        "aaj commission bikri ka kitne percent hai",
    ] {
        let u = understand(&reader(&w, today), question);
        let got = plan_reads(&u).map(|r| render_all(&r));
        assert_eq!(
            got.as_deref(),
            Some("period{kind:today} | analytics{from:2026-10-10,to:2026-10-10,view:A}"),
            "{question}"
        );
    }
}

/// G5 (live run 2): "akrm aur arshd ka jama kitna hua" is the sum of two buyers' balances: both cards are
/// read and the model adds them (no instant figure); "aaj kele ki kitni bikri hui" asks the nag and the
/// amount of one item, so view A is read for the model to give both. One buyer's "jama" is the model's.
#[test]
fn the_sum_of_two_parties_and_the_count_of_an_item_are_read_for_the_model() {
    let w = books();
    let today = on(2026, 10, 10);
    for question in [
        "akrm aur arshd ka jama kitna hua",
        "akram aur arshad ka jama kitna hua",
    ] {
        let u = understand(&reader(&w, today), question);
        let reads = plan_reads(&u).unwrap_or_else(|| panic!("{question}: no reads"));
        let text = render_all(&reads);
        assert!(
            text.contains("party_card{party:a20}") && text.contains("party_card{party:a21}"),
            "{question}: {text}"
        );
        let refused = run_plan(&w, today, question, &reads);
        assert!(refused.is_empty(), "{question}: {refused:?}");
    }
    let u = understand(&reader(&w, today), "akram ka jama kitna hai");
    assert!(plan_reads(&u).is_none(), "one buyer's jama is the model's");
    let u = understand(&reader(&w, today), "aaj kele ki kitni bikri hui");
    assert_eq!(
        plan_reads(&u).as_deref().map(render_all).as_deref(),
        Some(
            "period{kind:today} | find{role:item,text:Banana} | analytics{from:2026-10-10,item:i1,to:2026-10-10,view:A}"
        )
    );
}

/// "Mooli kitne mahine biki hai" counts the months of ALL the books: view D from the first day to today.
/// The first run read it with no dates, took the tool's default period and said 1 month for 6. On the
/// rich year (today 05-11-2026) radish sold in 6 months.
#[test]
fn the_months_an_item_sold_in_are_counted_over_all_the_books() {
    let y = year::seed::build();
    let today = on(2026, 11, 5);
    let question = "mooli kitne mahine biki hai";
    let u = understand(&reader(&y.w, today), question);
    let reads = plan_reads(&u).expect("reads");
    let text = render_all(&reads);
    assert!(
        text.contains("analytics{from:") && text.contains("to:2026-11-05,view:D}"),
        "{text}"
    );
    let (refused, state) = run_plan_state(&y.w, today, question, &reads);
    assert!(refused.is_empty(), "refused: {refused:?}");
    let words = Words::new().expect("the words");
    let view = state
        .results
        .iter()
        .find(|r| r.tool == "analytics")
        .map(|r| mandi_assistant::results::tool_result_json(r, &words))
        .expect("the view");
    // The months that hold a figure in the one row of the item: the columns `p_YYYY_MM`.
    let months = view["rows"]
        .as_array()
        .and_then(|rows| rows.first())
        .and_then(|row| row.as_object())
        .map_or(0, |cells| {
            cells
                .iter()
                .filter(|(key, value)| {
                    key.starts_with("p_") && value["rs"].as_i64().is_some_and(|rs| rs != 0)
                })
                .count()
        });
    assert_eq!(months, 6, "{view}");
}

/// The goods left of each vegetable ("kaun kaun si sabzi kitni baki padi hai"): R10 laid out by item with
/// the group, one row for each item, not the lots and not a total (the first run gave totals and advice).
/// On the rich year: potato 130 and tomato 47.
#[test]
fn the_goods_left_of_each_vegetable_are_read_one_row_for_each_item() {
    let y = year::seed::build();
    let today = on(2026, 11, 5);
    let question = "kaun kaun si sabzi kitni baki padi hai";
    let u = understand(&reader(&y.w, today), question);
    let reads = plan_reads(&u).expect("reads");
    assert_eq!(
        render_all(&reads),
        "report{id:R10,item_group:vegetable,layout:item}"
    );
    let (refused, state) = run_plan_state(&y.w, today, question, &reads);
    assert!(refused.is_empty(), "refused: {refused:?}");
    let words = Words::new().expect("the words");
    let stock = state
        .results
        .iter()
        .find(|r| r.tool == "report")
        .map(|r| mandi_assistant::results::tool_result_json(r, &words))
        .expect("the stock");
    let left: Vec<(String, i64)> = stock["rows"]
        .as_array()
        .map(|rows| {
            rows.iter()
                .filter_map(|row| {
                    Some((
                        row["name"]["en"].as_str()?.to_owned(),
                        row["balance"]["n"].as_i64()?,
                    ))
                })
                .collect()
        })
        .unwrap_or_default();
    assert_eq!(
        left,
        [("Potato".to_owned(), 130), ("Tomato".to_owned(), 47)]
    );
}

/// "Total agrahi collector wise batao", "agrahi collector wise", "collector wise dues": the title names the
/// list of dues by collector (R22, as of today), and the figure words beside it only say what it is
/// about. On the rich year its total is the shop's dues, 20,324,367, with a row for each collector.
#[test]
fn a_balance_list_named_by_its_title_is_read_as_that_list() {
    let y = year::seed::build();
    let today = on(2026, 11, 5);
    for question in [
        "total agrahi collector wise batao",
        "agrahi collector wise",
        "collector wise dues",
    ] {
        let u = understand(&reader(&y.w, today), question);
        let reads = plan_reads(&u).unwrap_or_else(|| panic!("{question}: no reads"));
        assert_eq!(render_all(&reads), "report{id:R22}", "{question}");
        let (refused, state) = run_plan_state(&y.w, today, question, &reads);
        assert!(refused.is_empty(), "{question}: {refused:?}");
        let words = Words::new().expect("the words");
        let list = state
            .results
            .iter()
            .find(|r| r.tool == "report")
            .map(|r| mandi_assistant::results::tool_result_json(r, &words).to_string())
            .expect("the list");
        // The total of R22 is the buyers' dues of the tile (the rich year is read from the oracle, so a
        // change of the year's days never leaves this test with a stale number).
        let buyers = year::oracle::compute(&y).facts["tiles.buyers_dues"].clone();
        assert!(list.contains(&buyers), "{question}: the total {buyers}");
        assert!(list.contains("2605353"), "{question}: a collector's row");
    }
}

/// On the rich year (today 05-11-2026) the planned reads hold the books' figures: onion sold 442,130,
/// 395,580 and 608,760 in August, September and October, and potato 111,060 this month so far against
/// 1,894,960 last month.
#[test]
fn the_months_and_the_two_months_compare_hold_the_books_figures_on_the_rich_year() {
    let y = year::seed::build();
    let today = on(2026, 11, 5);
    let words = Words::new().expect("the words");
    let gross_of = |question: &str| -> Vec<String> {
        let u = understand(&reader(&y.w, today), question);
        assert!(
            u.sure,
            "{question}: the run starts only from a sure understanding"
        );
        let reads = plan_reads(&u).unwrap_or_else(|| panic!("{question}: no reads"));
        let (refused, state) = run_plan_state(&y.w, today, question, &reads);
        assert!(refused.is_empty(), "{question}: {refused:?}");
        state
            .results
            .iter()
            .filter(|r| r.tool == "analytics")
            .map(|r| {
                let json = mandi_assistant::results::tool_result_json(r, &words);
                json["totals"]["gross"]["rs"].to_string()
            })
            .collect()
    };
    // The latest month first, as the plan reads them.
    assert_eq!(
        gross_of("how did onion sales go in August, September and October"),
        ["608760", "395580", "442130"]
    );
    assert_eq!(
        gross_of("is potato selling more this month than last month"),
        ["111060", "1894960"]
    );
}

/// A balance now and the months ahead in one question: "cash in hand now and what will it be in 3 months",
/// "rahim ka khata abhi kitna he aur agle mahine tak kitna ho jaye ga". The app reads what it is now (the
/// tile, the party card) and the estimate (cash_in_hand, party_balance with the party, `ahead` 3 or 1), so
/// the model answers from reads that are already in. "In 3 months" is a period of its own.
#[test]
fn a_balance_now_and_the_months_ahead_are_read_together() {
    let y = year::seed::build();
    let today = on(2026, 11, 5);
    let cases = [
        (
            "cash in hand now and what will it be in 3 months",
            "tiles{} | estimate{ahead:3,measure:cash_in_hand}",
        ),
        (
            "cash kitna hai aur agle 3 mahine mein kitna hoga",
            "tiles{} | estimate{ahead:3,measure:cash_in_hand}",
        ),
        (
            "rahim ka khata abhi kitna he aur agle mahine tak kitna ho jaye ga",
            "find{role:seller,text:rahim} | party_card{party:a18} | estimate{ahead:1,measure:party_balance,party:a18}",
        ),
    ];
    for (question, want) in cases {
        let u = understand(&reader(&y.w, today), question);
        let reads = plan_reads(&u).unwrap_or_else(|| panic!("{question}: no reads"));
        assert_eq!(render_all(&reads), want, "{question}");
        let refused = run_plan(&y.w, today, question, &reads);
        assert!(refused.is_empty(), "{question}: {refused:?}");
    }
}

/// "Sab se zyada maal kis cheez ka para hai abhi": the goods left of each item, the most first (R10 laid
/// out by item, sorted by balance, the top 5). "Para hai" with "maal" is the goods left; "cheez" is an item.
#[test]
fn the_item_with_the_most_goods_left_is_read_from_the_stock_by_item() {
    let w = books();
    let today = on(2026, 10, 10);
    for (question, want) in [
        (
            "sab se zyada maal kis cheez ka para hai abhi",
            "report{id:R10,layout:item,rows:5,sort:balance:desc}",
        ),
        (
            "kaun si cheez ka maal sab se zyada baqi hai",
            "report{id:R10,layout:item,rows:5,sort:balance:desc}",
        ),
        (
            "sab se kam maal kis cheez ka baqi hai",
            "report{id:R10,layout:item,rows:5,sort:balance:asc}",
        ),
        ("maal kis cheez ka pada hai", "report{id:R10,layout:item}"),
    ] {
        let u = understand(&reader(&w, today), question);
        let reads = plan_reads(&u).unwrap_or_else(|| panic!("{question}: no reads"));
        assert_eq!(render_all(&reads), want, "{question}");
        let refused = run_plan(&w, today, question, &reads);
        assert!(refused.is_empty(), "{question}: {refused:?}");
    }
}

// ---- live run 2: lists in Urdu, "کل", and the quality of a line ------------------------------------------

/// "پچھلے مہینے کے تمام خرچوں کی فہرست" is the expense report of last month (R29), "ڈوبی رقم میں کس کس کا
/// نام ہے" the dead amount party by party (R38), and the staff names a list of the staff. The words خرچوں,
/// نام, عملے and کام are known words, not a party that was not found (Q882, Q904, Q1433).
#[test]
fn lists_in_urdu_of_the_expenses_the_dead_amount_and_the_staff_are_read() {
    let w = books();
    let today = on(2026, 10, 10);
    for (question, want) in [
        (
            "پچھلے مہینے کے تمام خرچوں کی فہرست",
            "period{kind:last_month} | report{from:2026-09-01,id:R29,to:2026-09-30}",
        ),
        ("ڈوبی رقم میں کس کس کا نام ہے", "report{id:R38}"),
        (
            "سارے عملے کے نام اور ان کا کام دکھائیں",
            "find{role:staff,text:all}",
        ),
    ] {
        let u = understand(&reader(&w, today), question);
        assert!(!u.has_not_found(), "{question}: a word taken for a name");
        let reads = plan_reads(&u).unwrap_or_else(|| panic!("{question}: no reads"));
        assert_eq!(render_all(&reads), want, "{question}");
        let refused = run_plan(&w, today, question, &reads);
        assert!(refused.is_empty(), "{question}: {refused:?}");
    }
}

/// "سبجی کی quantity کل کتنی تھی": "سبجی" is the vegetables, "quantity" the nag, and "کل" with the past "تھی" is
/// yesterday, not the total of all the books (Q1782 read 24,450 for an expected 0). "کل ملا کر" is a total.
#[test]
fn kal_with_a_past_verb_is_yesterday_and_kul_mila_kar_is_a_total() {
    let w = books_with_vegetables();
    let today = on(2026, 10, 10);
    let ready = include_str!("../prompt/tools.json").contains("\"item_group\"");
    let want = "period{kind:yesterday} | analytics{from:2026-10-09,item_group:vegetable,to:2026-10-09,view:A}";
    for question in [
        "سبجی کی quantity کل کتنی تھی",
        "سبزی کی quantity کل کتنی تھی",
    ] {
        let u = understand(&reader(&w, today), question);
        let got = plan_reads(&u).map(|r| render_all(&r));
        assert_eq!(got.as_deref(), ready.then_some(want), "{question}");
    }
    let u = understand(&reader(&w, today), "کل ملا کر سبجی کی بکری");
    let text = plan_reads(&u).map(|r| render_all(&r)).unwrap_or_default();
    assert!(
        !text.contains("period{kind:yesterday}"),
        "a total read as yesterday: {text}"
    );
}

/// The quality a question names, in Roman Urdu, Urdu and English, and its usual misspellings (awal, avval,
/// doem, soym, secnd, sekond). A question about one quality has no instant figure; with the tools'
/// `grade` filter it reads view A with the item, the period and the grade (the figures of that quality
/// only, never of all of them), and until then the sale lines.
#[test]
fn a_quality_word_names_the_grade_and_the_read_carries_it() {
    use mandi_assistant::understand::GradeWord::{First, Second, Third};
    let w = books();
    let today = on(2026, 10, 10);
    let ready = include_str!("../prompt/tools.json").contains("\"grade\"");
    for (question, grade) in [
        ("awwal seb aaj kitne ka bika", First),
        ("awal appel aaj kitne ka bika", First),
        ("avval seb aaj kitne ka bika", First),
        ("اول سیب آج کتنے کا بکا", First),
        ("first quality banana rate today", First),
        ("A grade apple sales this month", First),
        ("doyam kele ka aaj ka rate", Second),
        ("doem seb aaj ki bikri", Second),
        ("secnd apple rate today", Second),
        ("sekond quality seb ka rate aaj", Second),
        ("second apple sales this month", Second),
        ("دوم کیلے کا آج کا ریٹ", Second),
        ("soyam kele ki is mahine ki bikri", Third),
        ("soym kele ka rate aaj", Third),
        ("third quality apple rate today", Third),
    ] {
        let u = understand(&reader(&w, today), question);
        assert!(u.quality, "{question}: no quality");
        assert_eq!(u.grade, Some(grade), "{question}");
        let reads = plan_reads(&u).unwrap_or_else(|| panic!("{question}: no reads"));
        let text = render_all(&reads);
        let word = grade.tool_word();
        if ready {
            assert!(
                text.contains("analytics{") && text.contains(&format!("grade:{word}")),
                "{question}: {text}"
            );
        } else {
            assert!(
                text.contains("report{") && text.contains("id:R2"),
                "{question}: {text}"
            );
        }
        let refused = run_plan(&w, today, question, &reads);
        assert!(refused.is_empty(), "{question}: {refused:?}");
    }
    // All the qualities ("kis darje mein bike"): no one grade, the lines carry each.
    let u = understand(&reader(&w, today), "rahim ke seb kis kis darje mein bike");
    assert!(u.quality && u.grade.is_none());
    assert!(plan_reads(&u).is_some());
}

/// Live run 2: capital taken back is the `capital_out` vouchers (R15), never the drawings (R29), and
/// capital put in is `capital_in` (C9); the gardens' goods sold in our shop are view A with `goods` garden,
/// never the whole shop (C3); a seller or buyer side word before "commission" is the side of the charge,
/// not the role of the person named, so the reads of the buyer and the item are made (C4, C6).
#[test]
fn capital_flow_garden_goods_and_a_side_of_the_commission_are_read_for_what_they_name() {
    let w = books_with_vegetables();
    let today = on(2026, 10, 10);
    let ready = include_str!("../prompt/tools.json").contains("\"item_group\"");
    let _ = ready;
    for (question, want) in [
        (
            "اس سال شراکت داروں نے کتنا سرمایہ واپس لیا",
            "period{kind:this_year} | report{from:2026-01-01,id:R15,to:2026-10-10,voucher_type:capital_out}",
        ),
        (
            "how much capital did partners take back this year",
            "period{kind:this_year} | report{from:2026-01-01,id:R15,to:2026-10-10,voucher_type:capital_out}",
        ),
        (
            "how much capital did partners put in this year",
            "period{kind:this_year} | report{from:2026-01-01,id:R15,to:2026-10-10,voucher_type:capital_in}",
        ),
        (
            "gardens ki is saal ki bikri",
            "period{kind:this_year} | analytics{from:2026-01-01,goods:garden,to:2026-10-10,view:A}",
        ),
    ] {
        let u = understand(&reader(&w, today), question);
        let reads = plan_reads(&u).unwrap_or_else(|| panic!("{question}: no reads"));
        assert_eq!(render_all(&reads), want, "{question}");
        let refused = run_plan(&w, today, question, &reads);
        assert!(refused.is_empty(), "{question}: {refused:?}");
    }
    // The side word is not the role of the name: Akram and Faisal are buyers.
    for question in [
        "seller commission on the banana Akram bought this year",
        "اگست میں اکرم کے آلو پر بیوپاری کا کمیشن",
    ] {
        let u = understand(&reader(&w, today), question);
        assert!(!u.has_not_found(), "{question}");
        let text = plan_reads(&u)
            .map(|r| render_all(&r))
            .unwrap_or_else(|| panic!("{question}: no reads"));
        assert!(text.contains("find{role:buyer,"), "{question}: {text}");
        assert!(text.contains("analytics{buyer:"), "{question}: {text}");
    }
}

/// F1 of the live run: a word that is no name, or a staff member, is never "no such party" (the app
/// ends the question as not found by itself). The partners (plural), the money they took ("personal
/// drawings") and what they put in are read as the shop's flows; a collector is a record, and what he
/// collected is the receipts register for him; a partner is a partner even when "liya" is said; a
/// misspelt Urdu item is the item.
#[test]
fn partners_drawings_a_collector_and_a_misspelt_item_are_read_not_missing() {
    let y = year::seed::build_thin();
    let today = on(2026, 11, 5);
    for (question, want) in [
        (
            "aaj shirakat daaron ne kitna sarmaya lagaya",
            "period{kind:today} | report{from:2026-11-05,id:R15,to:2026-11-05,voucher_type:capital_in}",
        ),
        (
            "pichle hafte shirakat daaron ne kitna sarmaya lagaya",
            "period{kind:last_week} | report{from:2026-10-26,id:R15,to:2026-11-01,voucher_type:capital_in}",
        ),
        (
            "how much did the partners take for personal drawings today",
            "period{kind:today} | report{from:2026-11-05,id:R29,to:2026-11-05}",
        ),
        (
            "kal shirakat daaron ne personal drawings ke liye kitna liya",
            "period{kind:yesterday} | report{from:2026-11-04,id:R29,to:2026-11-04}",
        ),
        (
            "14 October ko shirakat daaron ne zati kharcha ke liye kitna liya",
            "period{from:2026-10-14,kind:range,to:2026-10-14} | report{from:2026-10-14,id:R29,to:2026-10-14}",
        ),
        (
            "Rasheed ne is mahine kitna collection kiya",
            "period{kind:this_month} | find{role:collector,text:Rasheed} | report{collector:st2,from:2026-11-01,id:R23,to:2026-11-05}",
        ),
        (
            "رشید نے اس ہفتے کتنی اگراہی کی",
            "period{kind:this_week} | find{role:collector,text:Rasheed} | report{collector:st2,from:2026-11-02,id:R23,to:2026-11-05}",
        ),
        (
            "Rasheeed ne 14 October ko kitni ugrahi ki",
            "period{from:2026-10-14,kind:range,to:2026-10-14} | find{role:collector,text:Rasheed} | report{collector:st2,from:2026-10-14,id:R23,to:2026-10-14}",
        ),
        (
            "Rasheeed ne pichle saal kitni wasooliyan kin",
            "period{kind:last_year} | find{role:collector,text:Rasheed} | report{collector:st2,from:2025-01-01,id:R23,to:2025-12-31}",
        ),
        (
            "رشید نے پچھلے سال کتنی وصولیاں کیں",
            "period{kind:last_year} | find{role:collector,text:Rasheed} | report{collector:st2,from:2025-01-01,id:R23,to:2025-12-31}",
        ),
        (
            "Alif ne kal zaati kharche ke liye kitna liya",
            "period{kind:yesterday} | find{role:partner,text:alif} | report{from:2026-11-04,id:R29,partner:s1,to:2026-11-04}",
        ),
        (
            "آج صیب کی بکری کتنی ہوئی",
            "period{kind:today} | find{role:item,text:Apple} | analytics{from:2026-11-05,item:i2,to:2026-11-05,view:A}",
        ),
        (
            "پیاض کا اس ہفتے کتنا نگ بکا",
            "period{kind:this_week} | find{role:item,text:Onion} | analytics{from:2026-11-02,item:i9,to:2026-11-05,view:A}",
        ),
    ] {
        let u = understand(&reader(&y.w, today), question);
        assert!(!u.has_not_found(), "{question}: a missing party");
        let reads = plan_reads(&u).unwrap_or_else(|| panic!("{question}: no reads"));
        assert_eq!(render_all(&reads), want, "{question}");
        let refused = run_plan(&y.w, today, question, &reads);
        assert!(refused.is_empty(), "{question}: {refused:?}");
    }
    // A two-letter Urdu word that is also a common word (عام, general) is never taken for the item
    // with a sure answer: the model reads it.
    let u = understand(
        &reader(&y.w, today),
        "عام سے ستمبر میں total کمیشن کتنا بنا",
    );
    assert!(!u.has_not_found());
    assert!(!u.sure, "عام is not sure");
    // The one-time people have no figure of their own: the shop's dues are not theirs.
    for question in [
        "pukhta khata ka kul baqaya kitna hai",
        "pukhta khata ke logon ka kul baqaya kitna hai",
    ] {
        let u = understand(&reader(&y.w, today), question);
        assert!(!u.has_not_found(), "{question}");
        assert!(plan_reads(&u).is_none(), "{question}: the shop's dues");
    }
    // "کل" before a word for the side of a charge is "total", so the day is not sure.
    let u = understand(&reader(&y.w, today), "سبزی کا کل بائر کمیشن کتنا ہے");
    assert!(
        !u.periods
            .iter()
            .any(|p| p.confidence == mandi_assistant::understand::Confidence::High),
        "کل is a day, sure"
    );
}
