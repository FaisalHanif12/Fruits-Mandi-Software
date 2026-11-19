//! Every command the frontend calls is registered in the backend (plan 78: the print job commands of
//! plans 76 and 77 were called by `api.ts` but never registered, so Print and Save as PDF could not
//! work). The test reads the api files of `frontend/src`, finds each `callCommand<...>('name'` and
//! checks the name is in the handler list of `src/lib.rs` and in the command list of `build.rs`,
//! which makes its permission.
#![cfg(test)]

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk(&path, out);
        } else {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            let is_source = name.ends_with(".ts") || name.ends_with(".tsx");
            let is_test = name.contains(".test.") || name.contains(".spec.");
            if is_source && !is_test {
                out.push(path);
            }
        }
    }
}

/// The names in `callCommand<Type>('name'` and `callCommand('name'`, with the generic skipped.
fn called_commands(text: &str) -> Vec<String> {
    let mut found = Vec::new();
    let bytes = text.as_bytes();
    let mut from = 0;
    while let Some(at) = text.get(from..).and_then(|t| t.find("callCommand")) {
        let mut i = from + at + "callCommand".len();
        from = i;
        while bytes.get(i).is_some_and(u8::is_ascii_whitespace) {
            i += 1;
        }
        if bytes.get(i) == Some(&b'<') {
            let mut depth = 0_usize;
            while let Some(&b) = bytes.get(i) {
                match b {
                    b'<' => depth += 1,
                    b'>' => {
                        depth -= 1;
                        if depth == 0 {
                            i += 1;
                            break;
                        }
                    }
                    _ => {}
                }
                i += 1;
            }
        }
        while bytes.get(i).is_some_and(u8::is_ascii_whitespace) {
            i += 1;
        }
        if bytes.get(i) != Some(&b'(') {
            continue;
        }
        i += 1;
        while bytes.get(i).is_some_and(u8::is_ascii_whitespace) {
            i += 1;
        }
        if bytes.get(i) != Some(&b'\'') {
            continue;
        }
        let start = i + 1;
        if let Some(len) = text.get(start..).and_then(|t| t.find('\'')) {
            let name = &text[start..start + len];
            if name
                .chars()
                .all(|c| c.is_ascii_lowercase() || c == '_' || c.is_ascii_digit())
            {
                found.push(name.to_owned());
            }
        }
    }
    found
}

fn registered_in_lib() -> BTreeSet<String> {
    include_str!("../src/lib.rs")
        .lines()
        .map(str::trim)
        .filter(|l| l.starts_with("commands::"))
        .filter_map(|l| l.trim_end_matches(',').rsplit("::").next())
        .map(str::to_owned)
        .collect()
}

fn registered_in_build() -> BTreeSet<String> {
    include_str!("../build.rs")
        .lines()
        .map(str::trim)
        .filter(|l| l.starts_with('"') && l.ends_with("\","))
        .map(|l| l.trim_matches(|c| c == '"' || c == ',').to_owned())
        .collect()
}

#[test]
fn every_command_the_frontend_calls_is_registered() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../frontend/src");
    let mut files = Vec::new();
    walk(&root, &mut files);
    assert!(!files.is_empty(), "no frontend source found under {root:?}");
    let (lib, build) = (registered_in_lib(), registered_in_build());
    assert!(
        lib.len() > 300,
        "the handler list was not read: {}",
        lib.len()
    );
    let mut called: BTreeSet<String> = BTreeSet::new();
    for file in &files {
        if let Ok(text) = fs::read_to_string(file) {
            called.extend(called_commands(&text));
        }
    }
    assert!(
        called.len() > 300,
        "no commands were found: {}",
        called.len()
    );
    let missing_handler: Vec<&String> = called.iter().filter(|n| !lib.contains(*n)).collect();
    let missing_permission: Vec<&String> = called.iter().filter(|n| !build.contains(*n)).collect();
    assert!(
        missing_handler.is_empty() && missing_permission.is_empty(),
        "called by the frontend but not registered: {missing_handler:?}; without a permission: {missing_permission:?}"
    );
}

#[test]
fn the_scanner_reads_a_call_with_a_generic_and_one_without() {
    let text = "return callCommand<PrintOutcome>('reports_print', { input });\n\
                x = callCommand<\n  Array<A>\n>(\n 'a_b2', {});\n y = callCommand('plain_one');";
    assert_eq!(
        called_commands(text),
        vec![
            "reports_print".to_owned(),
            "a_b2".to_owned(),
            "plain_one".to_owned()
        ]
    );
}
