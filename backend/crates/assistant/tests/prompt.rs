//! The tests of the fixed prompt (D6-system-prompt.md Section 7, P1 to P4, P6 and P7; P5 is in
//! `golden_prompt.rs`, P8 in `prompt_columns.rs` and P9 in `guide.rs`). The files of `prompt/` are
//! the same in every request (the guide is given back by the tool, not sent with it), so they are
//! kept equal to the document, small enough for four calls to fit the input limit, and equal to
//! the catalogue of the app in both directions.
#![cfg(test)]

use std::collections::BTreeSet;
use std::path::PathBuf;

use mandi_assistant::limits::MAX_PREFIX_TOKENS;
use mandi_assistant::model_answer::EntryScreenName;
use mandi_assistant::prompt::{
    ANSWER_SCHEMA_JSON, GUIDE_JSON, SYSTEM, TOOL_NAMES, TOOLS_JSON, WORDS_TSV, WordKind,
    answer_schema, tools, words,
};
use mandi_reports::analytics::catalog as analytics_catalog;
use mandi_reports::reports::catalog as report_catalog;
use serde_json::{Value, json};

fn repo_file(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .join(relative)
}

fn read_repo(relative: &str) -> String {
    std::fs::read_to_string(repo_file(relative)).unwrap_or_else(|e| panic!("{relative}: {e}"))
}

// ---- P1 ---------------------------------------------------------------------------------------

#[test]
fn p1_the_fixed_part_is_at_most_11600_tokens() {
    let bpe = tiktoken_rs::o200k_base().unwrap();
    let tools_min =
        serde_json::to_string(&serde_json::from_str::<Value>(TOOLS_JSON).unwrap()).unwrap();
    let schema_min =
        serde_json::to_string(&serde_json::from_str::<Value>(ANSWER_SCHEMA_JSON).unwrap()).unwrap();
    let total = [SYSTEM, tools_min.as_str(), schema_min.as_str()]
        .iter()
        .map(|part| bpe.encode_with_special_tokens(part).len())
        .sum::<usize>();
    eprintln!("the fixed part is {total} tokens (o200k_base); the limit is {MAX_PREFIX_TOKENS}");
    assert!(
        total <= usize::try_from(MAX_PREFIX_TOKENS).unwrap(),
        "{total} tokens"
    );
}

// ---- P2 ---------------------------------------------------------------------------------------

/// The report ids named in `system.md` (`R13`): an R and digits, not part of a longer word.
fn report_ids_named() -> BTreeSet<String> {
    let chars: Vec<char> = SYSTEM.chars().collect();
    let mut found = BTreeSet::new();
    for (i, c) in chars.iter().enumerate() {
        let before_ok = i == 0 || !chars[i - 1].is_alphanumeric();
        if *c == 'R' && before_ok {
            let digits: String = chars[i + 1..]
                .iter()
                .take_while(|d| d.is_ascii_digit())
                .collect();
            let after = chars.get(i + 1 + digits.len());
            if !digits.is_empty() && !after.is_some_and(|a| a.is_alphanumeric()) {
                found.insert(format!("R{digits}"));
            }
        }
    }
    found
}

/// The view letters of the line "Analytics views (...): A by item | B by seller | ...".
fn view_letters_named() -> BTreeSet<String> {
    let line = SYSTEM
        .lines()
        .skip_while(|l| !l.starts_with("Analytics views"))
        .nth(1)
        .expect("the views line");
    line.split(" | ")
        .filter_map(|entry| {
            let mut chars = entry.trim().chars();
            let letter = chars.next()?;
            (letter.is_ascii_uppercase() && chars.next() == Some(' ')).then(|| letter.to_string())
        })
        .collect()
}

#[test]
fn p2_the_catalogue_is_named_both_ways() {
    // Reports.
    let known: BTreeSet<String> = report_catalog()
        .iter()
        .map(|e| e.id.code().to_owned())
        .collect();
    assert_eq!(
        report_ids_named(),
        known,
        "report ids named in system.md and the catalogue"
    );
    // Analytics views.
    let views: BTreeSet<String> = analytics_catalog()
        .iter()
        .map(|e| e.id.code().to_owned())
        .collect();
    assert_eq!(view_letters_named(), views, "view letters");
    // The view enum of the tool schema is the same set.
    let from_schema: BTreeSet<String> = tools()
        .iter()
        .find(|t| t["function"]["name"] == "analytics")
        .unwrap()["function"]["parameters"]["properties"]["view"]["enum"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_owned())
        .collect();
    assert_eq!(from_schema, views);
    // The screens of the answer schema are the screens the answer can carry, both ways: the entry
    // screens and the main screens of the help button (plan 51 B3, PRD FR-7.89).
    let screens: Vec<String> =
        answer_schema()["properties"]["entry"]["anyOf"][0]["properties"]["screen"]["enum"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| s.as_str().unwrap().to_owned())
            .collect();
    for screen in &screens {
        serde_json::from_value::<EntryScreenName>(json!(screen))
            .unwrap_or_else(|_| panic!("no route for the screen {screen}"));
    }
    assert_eq!(
        screens.iter().map(String::as_str).collect::<Vec<_>>(),
        EntryScreenName::ALL,
        "the entry.screen enum of the schema and EntryScreenName::ALL, in the same order"
    );
    // The report ids of the `report` tool description are R1 to R38 as well, without the retired R9
    // (PRD v1.32, FR-13.8): 37 reports.
    assert_eq!(known.len(), 37);
}

// ---- P3 ---------------------------------------------------------------------------------------

/// The terms of a glossary file: (id, value) for each `-id = value` line.
fn terms(file: &str) -> Vec<(String, String)> {
    read_repo(file)
        .lines()
        .filter(|l| l.starts_with('-'))
        .filter_map(|l| {
            let (id, value) = l.split_once(" = ")?;
            Some((id.trim().to_owned(), value.trim().to_owned()))
        })
        .collect()
}

/// Glossary terms of the Fluent files that the glossary line of `D6-system-prompt.md` does not hold
/// and need not hold (lead's ruling of 06-11-2026: no prompt tokens for them; the model reaches the
/// things they name through the tools, and the answer shows the app's own words). Each has its
/// reason. The test fails if a term leaves this list without being added to the prompt, or if
/// another term goes missing.
const GLOSSARY_GAPS: [(&str, &str); 12] = [
    (
        "-khate-mein-baqi",
        "the prompt names the columns paid and left; no room in the prefix (11,599 of 11,600)",
    ),
    (
        "-tijarati-nuqsan",
        "the trip's profit or loss is read from R33 and the trip results, never typed by the model",
    ),
    (
        "-sell-date",
        "a field of the sale-date screen, not something a question names",
    ),
    (
        "-maal-aaya",
        "an arrival screen word; arrivals are reached through their lists and results",
    ),
    (
        "-apni-dukan",
        "the source of fruit taken, read from the `source` column of R37",
    ),
    (
        "-dusri-dukan",
        "the source of fruit taken, read from the `source` column of R37",
    ),
    (
        "-mehman",
        "the shop's guests of fruit taken, read from R37 (the for-column and the guests rows)",
    ),
    (
        "-trade-mark",
        "a word of the print header, not something a question asks about",
    ),
    (
        "-darja",
        "the quality of a sale line, read from the report columns and results, never typed",
    ),
    (
        "-darja-awwal",
        "a value of the quality column, shown as the app writes it",
    ),
    (
        "-darja-doyam",
        "a value of the quality column, shown as the app writes it",
    ),
    (
        "-darja-soyam",
        "a value of the quality column, shown as the app writes it",
    ),
];

#[test]
fn p3_every_glossary_term_is_in_the_prompt_with_its_english_word() {
    let line = SYSTEM
        .lines()
        .find(|l| l.starts_with("Glossary ("))
        .expect("the glossary line");
    let ur = terms("frontend/src/i18n/ur/glossary.ftl");
    let en = terms("frontend/src/i18n/en/glossary.ftl");
    assert!(ur.len() > 60);
    let mut missing = Vec::new();
    for (id, ur_value) in &ur {
        // A term built from other terms ({ -x }) is written once, in its parts.
        if ur_value.contains('{') {
            continue;
        }
        let en_value = en
            .iter()
            .find(|(i, _)| i == id)
            .map(|(_, v)| v.split(" (").next().unwrap_or("").to_owned())
            .unwrap_or_default();
        if !line.contains(ur_value.as_str()) {
            missing.push(format!("{id}: {ur_value}"));
        } else if !en_value.is_empty()
            && !en_value.contains('{')
            && !line.contains(en_value.as_str())
        {
            missing.push(format!("{id}: {en_value}"));
        }
    }
    let ids: Vec<&str> = missing
        .iter()
        .map(|m| m.split(':').next().unwrap_or(""))
        .collect();
    let allowed: Vec<&str> = GLOSSARY_GAPS.iter().map(|(id, _)| *id).collect();
    assert_eq!(
        ids, allowed,
        "not in the glossary line of system.md: {missing:#?}"
    );
    assert!(GLOSSARY_GAPS.iter().all(|(_, why)| !why.is_empty()));
}

#[test]
fn p3_the_tile_names_of_the_prompt_are_the_dashboard_words() {
    let dashboard = read_repo("frontend/src/i18n/ur/dashboard.ftl");
    let line = SYSTEM
        .lines()
        .find(|l| l.starts_with("Glossary ("))
        .expect("the glossary line");
    // The tile "پیشگی باقی" is not a glossary term of the prompt (the prompt names the term
    // پیشگی); the buyers' dues tile is. Reported with the glossary gaps above.
    let id = "dashboard-tile-dues";
    let value = dashboard
        .lines()
        .find_map(|l| l.strip_prefix(&format!("{id} = ")))
        .unwrap_or_else(|| panic!("no {id}"));
    assert!(
        line.contains(value.trim()) || SYSTEM.contains(value.trim()),
        "the tile word {value} is not in the prompt"
    );
}

// ---- P4 ---------------------------------------------------------------------------------------

#[test]
fn p4_the_never_use_words_of_the_prompt_are_the_never_rows_of_the_word_list() {
    let line = SYSTEM
        .lines()
        .find_map(|l| l.split_once("Never use: ").map(|(_, rest)| rest))
        .expect("the never-use line");
    let list = line.split_once(".\n").map_or(line, |(a, _)| a);
    let list = list.split(". ").next().unwrap_or(list);
    let from_prompt: BTreeSet<String> = list
        .split(", ")
        .map(|item| {
            item.split(" (say ")
                .next()
                .unwrap_or(item)
                .trim()
                .trim_end_matches('.')
                .to_owned()
        })
        .filter(|w| !w.is_empty())
        .collect();
    let from_words: BTreeSet<String> = words().both(WordKind::Never).cloned().collect();
    assert_eq!(from_prompt, from_words);
}

// ---- P6 ---------------------------------------------------------------------------------------

/// Every object lists every property in `required` and refuses extra ones (strict mode).
fn check_strict(node: &Value, path: &str) {
    match node {
        Value::Object(map) => {
            if let Some(Value::Object(props)) = map.get("properties") {
                assert_eq!(
                    map.get("additionalProperties"),
                    Some(&json!(false)),
                    "{path}: additionalProperties"
                );
                let required: BTreeSet<&str> = map
                    .get("required")
                    .and_then(Value::as_array)
                    .map(|a| a.iter().filter_map(Value::as_str).collect())
                    .unwrap_or_default();
                let all: BTreeSet<&str> = props.keys().map(String::as_str).collect();
                assert_eq!(required, all, "{path}: required lists every property");
            }
            for (k, v) in map {
                check_strict(v, &format!("{path}.{k}"));
            }
        }
        Value::Array(items) => {
            for (i, v) in items.iter().enumerate() {
                check_strict(v, &format!("{path}[{i}]"));
            }
        }
        _ => {}
    }
}

#[test]
fn p6_the_schemas_are_strict_and_the_inputs_round_trip() {
    let defs = tools();
    assert_eq!(defs.len(), 10);
    for (def, name) in defs.iter().zip(TOOL_NAMES) {
        assert_eq!(def["type"], "function");
        assert_eq!(def["function"]["name"], name);
        assert_eq!(def["function"]["strict"], true, "{name} is strict");
        check_strict(&def["function"]["parameters"], name);
    }
    check_strict(answer_schema(), "answer");
    // A sample input for each tool reads through its Rust input type.
    macro_rules! reads {
        ($ty:ty, $json:expr) => {{
            let value: Value = $json;
            serde_json::from_value::<$ty>(value).expect(stringify!($ty));
        }};
    }
    use mandi_assistant::tools::{
        analytics, calc, find, help, party, period, report, state, tiles,
    };
    reads!(find::Input, json!({"text": "ikram", "role": "buyer"}));
    reads!(party::Input, json!({"party": "r1.matches.0"}));
    reads!(
        report::Input,
        json!({"id": "R13", "from": "r2.from", "to": "r2.to", "as_of": null, "party": "r1.matches.0",
               "item": null, "city": null, "adda": null, "collector": null, "partner": null,
               "trip": null, "garden": null, "bikri": null, "group_by": null,
               "where": {"column": "balance", "op": "gt", "value": "q.n1"},
               "sort": [{"column": "balance", "order": "desc"}], "rows": 10})
    );
    reads!(
        analytics::Input,
        json!({"view": "A", "from": "r1.from", "to": "r1.to", "group_by": null, "item": null,
               "seller": null, "buyer": null, "city": null, "adda": null, "garden": null,
               "trip": null, "goods": null, "partner": null, "side": null, "compare": "previous",
               "sort": null, "rows": null})
    );
    reads!(tiles::Input, json!({}));
    reads!(
        period::Input,
        json!({"kind": "last_n_days", "month": null, "day": null, "from": null, "to": null, "n": 10})
    );
    reads!(
        calc::CalcInput,
        json!({"op": "sum", "inputs": ["r1.matches.0.balance", "q.n1"]})
    );
    reads!(
        calc::EstimateInput,
        json!({"measure": "commission_all", "item": null, "seller": null, "buyer": null,
               "garden": null, "trip": null, "ahead": 6, "target": null})
    );
    // The estimate with the fields of the forecast gaps (plan 55 11.1): head, partner and party.
    reads!(
        calc::EstimateInput,
        json!({"measure": "shop_expenses", "item": null, "seller": null, "buyer": null,
               "garden": null, "trip": null, "head": null, "partner": null, "party": null,
               "ahead": 3, "until": null, "what_if": null, "percent": null, "target": null})
    );
    reads!(
        calc::EstimateInput,
        json!({"measure": "party_balance", "item": null, "seller": null, "buyer": null,
               "garden": null, "trip": null, "head": null, "partner": null, "party": "a20",
               "ahead": 3, "until": null, "what_if": null, "percent": null, "target": null})
    );
    // The state of the app: rules, attention, closed days, the audit log (plan 55 11.1, R1 to R4).
    reads!(
        state::Input,
        json!({"what": "rules", "party": null, "item": null, "bikri": null})
    );
    reads!(help::Input, json!({"topic": "bikri_book"}));
    // Every topic of the enum of `app_help` is a topic of the guide, and a topic the guide has is
    // read (the other way round is P9).
    let topic = &defs[8]["function"]["parameters"]["properties"]["topic"]["enum"];
    for t in topic.as_array().unwrap() {
        reads!(help::Input, json!({"topic": t}));
    }
    // A sample answer reads through the answer type.
    let answer = json!({"kind": "answer", "text": "{{r1.matches.0.name}}", "show": [], "next": [],
                        "choose": null, "entry": null, "nearest": null});
    mandi_assistant::model_answer::ModelAnswer::parse(&answer.to_string()).unwrap();
    // A field the tool does not have is refused (strict reading).
    assert!(
        serde_json::from_value::<find::Input>(json!({"text": "x", "role": "buyer", "extra": 1}))
            .is_err()
    );
}

// ---- P7 ---------------------------------------------------------------------------------------

/// The text of the code block that follows the heading `## <number>.` in the prompt document.
fn block_of(document: &str, heading: &str) -> String {
    let mut lines = document.lines().skip_while(|l| !l.starts_with(heading));
    let _ = lines.next();
    let fence_line = lines
        .by_ref()
        .find(|l| l.starts_with("```"))
        .unwrap_or_else(|| panic!("no block after {heading}"));
    let fence: String = fence_line.chars().take_while(|c| *c == '`').collect();
    let mut body = String::new();
    for line in lines {
        if line.trim_end() == fence {
            break;
        }
        body.push_str(line);
        body.push('\n');
    }
    body
}

#[test]
fn p7_the_five_files_are_the_blocks_of_the_document() {
    let document = read_repo("docs/deliverables/D6-system-prompt.md");
    assert_eq!(SYSTEM, block_of(&document, "## 2."), "system.md");
    assert_eq!(TOOLS_JSON, block_of(&document, "## 3."), "tools.json");
    assert_eq!(
        ANSWER_SCHEMA_JSON,
        block_of(&document, "## 4."),
        "answer.schema.json"
    );
    assert_eq!(WORDS_TSV, block_of(&document, "## 6."), "words.tsv");
    assert_eq!(GUIDE_JSON, block_of(&document, "## 9."), "guide.json");
    // The document's own files on disk are the ones embedded (the crate has no other copy).
    for (file, text) in [
        ("system.md", SYSTEM),
        ("tools.json", TOOLS_JSON),
        ("answer.schema.json", ANSWER_SCHEMA_JSON),
        ("words.tsv", WORDS_TSV),
        ("guide.json", GUIDE_JSON),
    ] {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("prompt")
            .join(file);
        assert_eq!(std::fs::read_to_string(path).unwrap(), text, "{file}");
    }
}
