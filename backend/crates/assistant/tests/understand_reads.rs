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
use mandi_assistant::understand::{PlannedRead, Reader, plan_reads, understand};
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
    let year_world = std::env::var("WORLD").as_deref() == Ok("year");
    let (w, today);
    let y;
    if year_world {
        y = year::seed::build_thin();
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
    refused
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
    ("arshad ne kya kya khareeda", "find{role:buyer,text:arshad} | report{id:R2,party:$arshad}"),
    ("arshad ne pichle mahine kya khareeda", "period{kind:last_month} | find{role:buyer,text:arshad} | report{from:2026-09-01,id:R2,party:$arshad,to:2026-09-30}"),
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
    ("kele ka commission pichle mahine", "period{kind:last_month} | find{role:item,text:kele} | analytics{from:2026-09-01,item:$kela,to:2026-09-30,view:A}"),
    ("kele ka commission is saal", "period{kind:this_year} | find{role:item,text:kele} | analytics{from:2026-01-01,item:$kela,to:2026-10-10,view:A}"),
    ("kele ki bikri is mahine", "period{kind:this_month} | find{role:item,text:kele} | analytics{from:2026-10-01,item:$kela,to:2026-10-10,view:A}"),
    // The rate of view A is the average of the whole period. With no period it is the average of
    // all time, no rate for today: the model asks (money review R-d).
    ("kele ka rate", "NONE"),
    ("kele ka bhao", "NONE"),
    ("kele ka rate is mahine", "period{kind:this_month} | find{role:item,text:kele} | analytics{from:2026-10-01,item:$kela,to:2026-10-10,view:A}"),
    ("kele ka commission aaj", "period{kind:today} | find{role:item,text:kele} | analytics{from:2026-10-10,item:$kela,to:2026-10-10,view:A}"),
    ("saib ka commission", "find{role:item,text:saib} | analytics{item:$saib,view:A}"),
    ("kele ka commission rahim ka is mahine", "period{kind:this_month} | find{role:seller,text:rahim} | find{role:item,text:kele} | analytics{from:2026-10-01,item:$kela,seller:$rahim,to:2026-10-10,view:A}"),
    ("kele ka stock", "find{role:item,text:kele} | report{id:R10,item:$kela}"),
    ("پچھلے مہینے کا کمیشن", "period{kind:last_month} | analytics{from:2026-09-01,to:2026-09-30,view:A}"),
    ("کیلے کا کمیشن پچھلے مہینے", "period{kind:last_month} | find{role:item,text:کیلے} | analytics{from:2026-09-01,item:$kela,to:2026-09-30,view:A}"),
    // ---- the shop's sale and commission
    ("aaj ki bikri", "tiles{}"),
    ("aaj ka commission", "tiles{}"),
    ("آج کی بکری", "tiles{}"),
    ("aaj kitni bikriyan hui", "tiles{}"),
    ("is mahine ki bikri", "period{kind:this_month} | analytics{from:2026-10-01,to:2026-10-10,view:A}"),
    ("pichle mahine ka commission", "period{kind:last_month} | analytics{from:2026-09-01,to:2026-09-30,view:A}"),
    ("is saal ka commission", "period{kind:this_year} | analytics{from:2026-01-01,to:2026-10-10,view:A}"),
    ("kal ki bikri", "period{kind:yesterday} | analytics{from:2026-10-09,to:2026-10-09,view:A}"),
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
    // the total over the whole filter (lead ruling of 04-11-2026). One party's stay on his card.
    ("buyers ke dues kitne hain", "report{id:R20,sort:balance:desc}"),
    ("kul ugrahi kitni hai", "report{id:R20,sort:balance:desc}"),
    ("total dues", "report{id:R20,sort:balance:desc}"),
    ("gahakon ke kul bakaya", "report{id:R20,sort:balance:desc}"),
    ("sab gahakon ka bakaya", "report{id:R20,sort:balance:desc}"),
    ("sellers ka payable kitna hai", "report{id:R24,sort:payable:desc}"),
    ("sellers ka kul payable", "report{id:R24,sort:payable:desc}"),
    ("beopariyon ko kul kitna dena hai", "report{id:R24,sort:payable:desc}"),
    // What was GIVEN is an event, not what is open: the tile and the card hold what is open.
    ("kitni peshgi di hui hai", "NONE"),
    ("doobi raqam kitni hai", "tiles{}"),
    ("udhar raqam kitni hai", "tiles{}"),
    ("baqi maal kitna hai", "report{id:R10,sort:days:desc}"),
    ("stock kitna hai", "report{id:R10,sort:days:desc}"),
    // ---- reports of a period: expenses, day summary, day book, receipts, trial balance
    ("kharcha is mahine ka", "period{kind:this_month} | report{from:2026-10-01,id:R29,to:2026-10-10}"),
    ("pichle mahine ka kharcha", "period{kind:last_month} | report{from:2026-09-01,id:R29,to:2026-09-30}"),
    ("aaj ka kharcha", "period{kind:today} | report{from:2026-10-10,id:R29,to:2026-10-10}"),
    ("bijli ka kharcha pichle mahine", "period{kind:last_month} | report{from:2026-09-01,id:R29,to:2026-09-30}"),
    ("aaj ka khulasa", "period{kind:today} | report{from:2026-10-10,id:R34,to:2026-10-10}"),
    ("kal ka khulasa", "period{kind:yesterday} | report{from:2026-10-09,id:R34,to:2026-10-09}"),
    ("trial balance", "report{id:R18}"),
    ("roznamcha aaj ka", "period{kind:today} | report{from:2026-10-10,id:R11,to:2026-10-10}"),
    ("pichle mahine ki wasooli", "period{kind:last_month} | report{from:2026-09-01,id:R23,to:2026-09-30}"),
    ("is mahine kitni wasooli hui", "period{kind:this_month} | report{from:2026-10-01,id:R23,to:2026-10-10}"),
    // ---- partners (R31); the city, the adda
    ("partners ki kul raqam", "report{id:R31}"),
    ("partner b ki kul raqam", "find{role:partner,text:b} | report{id:R31,partner:$partner_b}"),
    ("partner a ki kul raqam", "find{role:partner,text:a} | report{id:R31,partner:$partner_a}"),
    // R31 runs in the kul raqam (E2E-37: b 50,746, his capital 50,000), so his capital is not its figure.
    ("partner b ka sarmaya", "NONE"),
    ("lahore ki ugrahi", "find{role:city,text:lahore} | report{city:$lahore,id:R21}"),
    ("swat adda ki bikri pichle mahine", "period{kind:last_month} | find{role:adda,text:swat adda} | report{adda:$swat,from:2026-09-01,id:R8,to:2026-09-30}"),
    // ---- lists and top N
    ("top 5 buyers", "period{kind:this_year} | analytics{from:2026-01-01,to:2026-10-10,view:H}"),
    ("top 3 sellers pichle mahine", "period{kind:last_month} | analytics{from:2026-09-01,to:2026-09-30,view:H}"),
    ("sab se zyada commission wala item", "period{kind:this_year} | analytics{from:2026-01-01,to:2026-10-10,view:H}"),
    ("sab se kam commission wala item", "period{kind:this_year} | analytics{from:2026-01-01,sort:total_comm:asc,to:2026-10-10,view:A}"),
    ("kis kis ke dues hain", "report{id:R20,sort:balance:desc}"),
    ("sab se zyada bakaya kis ka hai", "NONE"),
    ("sellers ke payable ki list", "report{id:R24,sort:payable:desc}"),
    ("top 5 items by sale this month", "period{kind:this_month} | analytics{from:2026-10-01,rows:5,sort:gross:desc,to:2026-10-10,view:A}"),
    ("kaun sa maal baqi hai", "report{id:R10,sort:days:desc}"),
    // ---- not covered: the model chooses
    ("akram aur arshad ka bakaya", "NONE"),
    ("faisal ka bakaya aur commission", "NONE"),
    ("kele aur saib ka commission", "NONE"),
    ("agle mahine ka commission", "NONE"),
    ("sale kaise barhayen", "NONE"),
    ("wasooli kaise likhun", "NONE"),
    ("akram ki wasooli likh do", "NONE"),
    ("weather kaisa hai", "NONE"),
    ("zubair ka bakaya", "NONE"),
    ("bagh ki bikri", "NONE"),
    ("safar ka hisab", "NONE"),
    ("hello", "NONE"),
    ("pichle mahine aur is mahine ka commission", "NONE"),
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
    let u = understand(&r, "kele ka commission");
    out.insert(
        "kela",
        u.items
            .first()
            .and_then(|i| i.outcome.hit())
            .map(|h| h.key.clone())
            .expect("banana"),
    );
    out
}

#[rustfmt::skip]
const YEAR: &[(&str, &str)] = &[
    ("faisal ka bakaya kitna hai", "find{role:buyer,text:faisal} | party_card{party:$faisal}"),
    ("usman ghani ka bakaya kitna hai", "find{role:buyer,text:usman ghani} | party_card{party:$usman_ghani}"),
    // Usman is a buyer and a partner: with no plain word the app is not sure which.
    ("usman ka bakaya", "NONE"),
    ("partner usman ki kul raqam", "find{role:partner,text:usman} | report{id:R31,partner:$usman}"),
    ("alif ki kul raqam", "find{role:partner,text:alif} | report{id:R31,partner:$alif}"),
    ("kele ka commission pichle mahine", "period{kind:last_month} | find{role:item,text:kele} | analytics{from:2026-10-01,item:$kela,to:2026-10-31,view:A}"),
    ("kele ka commission is saal", "period{kind:this_year} | find{role:item,text:kele} | analytics{from:2026-01-01,item:$kela,to:2026-11-05,view:A}"),
    ("top 5 buyers is saal", "period{kind:this_year} | analytics{from:2026-01-01,to:2026-11-05,view:H}"),
    ("sab se zyada commission kis item ka", "period{kind:this_year} | analytics{from:2026-01-01,to:2026-11-05,view:H}"),
    ("sab se zyada bikri kis item ki", "period{kind:this_year} | analytics{from:2026-01-01,sort:gross:desc,to:2026-11-05,view:A}"),
    ("aaj ki bikri kitni hui", "tiles{}"),
    ("baqi maal kitna hai", "report{id:R10,sort:days:desc}"),
    ("kharcha pichle mahine ka", "period{kind:last_month} | report{from:2026-10-01,id:R29,to:2026-10-31}"),
    ("pichle saal ka commission", "period{kind:last_year} | analytics{from:2025-01-01,to:2025-12-31,view:A}"),
    ("pichle 3 mahine ka commission", "period{from:2026-08-05,kind:range,to:2026-11-05} | analytics{from:2026-08-05,to:2026-11-05,view:A}"),
    ("rahim ki bikri october mein", "period{from:2026-10-01,kind:range,to:2026-10-31} | find{role:seller,text:rahim} | analytics{from:2026-10-01,seller:$rahim,to:2026-10-31,view:B}"),
    ("cash kitna hai", "tiles{}"),
    ("dues kitne hain", "report{id:R20,sort:balance:desc}"),
    // A garden, a trip or own goods has no filter in the reads: the model chooses.
    ("shahpur garden ka result", "NONE"),
    ("shahpur bagh ki bikri", "NONE"),
    ("14 august ka safar ka result", "NONE"),
    ("kela aur saib ka commission", "NONE"),
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
        assert!(
            reads.iter().filter(|r| r.tool == "period").count() <= 1,
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
            "seller": seller, "buyer": null, "city": null, "adda": null, "garden": null,
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
            "trip": null, "garden": null, "bikri": null, "group_by": null, "where": null,
            "sort": null, "rows": null
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
    for question in ["", "akram aur arshad ka bakaya", "hello", "akram"] {
        assert!(
            plan_reads(&understand(&reader(&w, today), question)).is_none(),
            "{question:?}"
        );
    }
}
