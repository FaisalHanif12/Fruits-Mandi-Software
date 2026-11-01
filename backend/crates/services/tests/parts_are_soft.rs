//! Guard for migration 14 (CLAUDE.md rule 4, PRD FR-7.70): `venture_entry_parts` and
//! `purchase_cost_parts` are soft state. An edit retires the old rows (`is_active = 0`) and writes new
//! ones, so every query that READS either table must read only the active rows, or a partner's amount
//! or a trip cost would be counted twice after an edit.
//!
//! The test reads the source of the services, the database layer, the reports and the app, finds every
//! `FROM` and `JOIN` of one of the two tables in a SQL string, and fails unless the same string says
//! `<alias of that table>.is_active = 1` (or `is_active = 1` when the table has no alias). It also
//! fails if any SQL anywhere deletes from them. An INSERT or an UPDATE is not a read and is not checked
//! here (the database triggers hold those: see `crates/db/tests/edit_windows.rs`).
//!
//! Example of what it catches: `SELECT SUM(amount_paisa) FROM venture_entry_parts WHERE entry_id = ?1`
//! after an edit adds up the retired parts and the new ones.
#![cfg(test)]

use std::path::{Path, PathBuf};

const TABLES: [&str; 2] = ["venture_entry_parts", "purchase_cost_parts"];

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

/// The source without whole-line comments, so a quote inside a doc comment cannot open a string.
fn code_only(text: &str) -> String {
    text.lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Every plain string literal of the source (`"..."` with `\"` escapes). SQL is written this way.
fn string_literals(code: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut chars = code.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\'' => {
                // A char literal such as '"' or '\\'; a lifetime has no closing quote soon.
                let mut look = chars.clone();
                let first = look.next();
                let second = look.next();
                let third = look.next();
                match (first, second, third) {
                    (Some('\\'), Some(_), Some('\'')) => {
                        chars.next();
                        chars.next();
                        chars.next();
                    }
                    (Some(_), Some('\''), _) => {
                        chars.next();
                        chars.next();
                    }
                    _ => {}
                }
            }
            '"' => {
                let mut text = String::new();
                while let Some(d) = chars.next() {
                    match d {
                        '\\' => {
                            chars.next();
                        }
                        '"' => break,
                        other => text.push(other),
                    }
                }
                found.push(text);
            }
            _ => {}
        }
    }
    found
}

fn sources() -> Vec<(String, String)> {
    let mut files = Vec::new();
    for folder in [
        "src",
        "../db/src",
        "../reports/src",
        "../backup/src",
        "../../app/src",
    ] {
        walk(
            &Path::new(env!("CARGO_MANIFEST_DIR")).join(folder),
            &mut files,
        );
    }
    files
        .into_iter()
        .map(|path| {
            let text = std::fs::read_to_string(&path).unwrap();
            (path.display().to_string(), code_only(&text))
        })
        .collect()
}

/// Words that can follow a table name in FROM or JOIN without being its alias.
const NOT_AN_ALIAS: [&str; 14] = [
    "ON", "WHERE", "JOIN", "LEFT", "INNER", "ORDER", "GROUP", "LIMIT", "AND", "USING", "SET",
    "HAVING", "UNION", "WINDOW",
];

/// For each `FROM <table>` / `JOIN <table>` of a parts table in the SQL, the text that must be in the
/// statement: `<alias>.is_active = 1`, or `is_active = 1` when the table has no alias. Looking for the
/// alias of the table itself keeps another table's `is_active` in the same statement (`bikri_sales`
/// has one) from satisfying the check.
fn needed_filters(sql: &str) -> Vec<(String, String)> {
    let tokens: Vec<&str> = sql
        .split(|c: char| c.is_whitespace() || c == '(' || c == ')' || c == ',')
        .filter(|t| !t.is_empty())
        .collect();
    let mut needed = Vec::new();
    for (i, token) in tokens.iter().enumerate() {
        if *token != "FROM" && *token != "JOIN" {
            continue;
        }
        let Some(table) = tokens.get(i + 1).filter(|t| TABLES.contains(t)) else {
            continue;
        };
        let alias = tokens
            .get(i + 2)
            .filter(|t| !NOT_AN_ALIAS.contains(&t.to_ascii_uppercase().as_str()))
            .filter(|t| t.chars().all(|c| c.is_ascii_alphanumeric() || c == '_'));
        let filter = match alias {
            Some(alias) => format!("{alias}.is_active = 1"),
            None => "is_active = 1".to_owned(),
        };
        needed.push(((*table).to_owned(), filter));
    }
    needed
}

#[test]
fn every_read_of_the_parts_tables_reads_only_the_active_rows() {
    let mut reads = 0;
    for (file, code) in sources() {
        for literal in string_literals(&code) {
            if !TABLES.iter().any(|t| literal.contains(t)) {
                continue;
            }
            for (table, filter) in needed_filters(&literal) {
                reads += 1;
                assert!(
                    literal.contains(&filter),
                    "{file}: this query reads `{table}` without `{filter}`, so it would count the \
                     rows an edit retired:\n{literal}"
                );
            }
        }
    }
    // The scan really found the queries (a change of style would make it find none and pass).
    assert!(
        reads >= 14,
        "only {reads} reads found: has the scan stopped working?"
    );
}

#[test]
fn nothing_deletes_from_the_parts_tables() {
    for (file, code) in sources() {
        for literal in string_literals(&code) {
            if TABLES.iter().any(|t| literal.contains(t)) {
                assert!(
                    !literal.contains("DELETE"),
                    "{file}: a parts table is soft state (migration 14), never deleted from:\n{literal}"
                );
            }
        }
    }
}
