//! Checks the workspace shape that a compiler cannot check by itself:
//! the dependency direction (CLAUDE.md rule 6, Code Standards B1), the five
//! Tauri plugins (Tech Stack 2), and that the app crate keeps the workspace lints.
#![cfg(test)]

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::process::Command;

use serde_json::Value;

fn workspace_dir() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap()
}

/// Our crates and the names of our crates each one depends on (normal dependencies).
fn our_dependencies() -> BTreeMap<String, BTreeSet<String>> {
    let output = Command::new(env!("CARGO"))
        .args([
            "metadata",
            "--format-version",
            "1",
            "--no-deps",
            "--offline",
        ])
        .current_dir(workspace_dir())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let metadata: Value = serde_json::from_slice(&output.stdout).unwrap();
    let mut result = BTreeMap::new();
    for package in metadata["packages"].as_array().unwrap() {
        let name = package["name"].as_str().unwrap().to_owned();
        let deps = package["dependencies"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|d| d["kind"].is_null())
            .map(|d| d["name"].as_str().unwrap().to_owned())
            .collect();
        result.insert(name, deps);
    }
    result
}

fn set(names: &[&str]) -> BTreeSet<String> {
    names.iter().map(|n| (*n).to_owned()).collect()
}

#[test]
fn dependency_direction_is_exact() {
    let deps = our_dependencies();
    let ours = |crate_name: &str| -> BTreeSet<String> {
        deps[crate_name]
            .iter()
            .filter(|d| d.starts_with("mandi-"))
            .cloned()
            .collect()
    };
    assert_eq!(deps.len(), 7, "{:?}", deps.keys());
    assert_eq!(
        ours("mandi-app"),
        set(&["mandi-services", "mandi-reports", "mandi-backup"])
    );
    assert_eq!(ours("mandi-services"), set(&["mandi-domain", "mandi-db"]));
    assert_eq!(ours("mandi-reports"), set(&["mandi-db", "mandi-core"]));
    assert_eq!(ours("mandi-backup"), set(&["mandi-db", "mandi-core"]));
    assert_eq!(ours("mandi-domain"), set(&["mandi-core"]));
    assert_eq!(ours("mandi-db"), set(&["mandi-core"]));
    assert_eq!(ours("mandi-core"), set(&[]));
}

#[test]
fn domain_and_core_have_no_database_or_tauri() {
    let deps = our_dependencies();
    for crate_name in ["mandi-core", "mandi-domain"] {
        for dep in &deps[crate_name] {
            assert!(
                !dep.contains("rusqlite") && !dep.starts_with("tauri"),
                "{crate_name} must not depend on {dep}"
            );
        }
    }
}

#[test]
fn only_the_five_tauri_plugins() {
    let deps = our_dependencies();
    let plugins: BTreeSet<String> = deps
        .values()
        .flatten()
        .filter(|d| d.starts_with("tauri-plugin-"))
        .cloned()
        .collect();
    assert_eq!(
        plugins,
        set(&[
            "tauri-plugin-dialog",
            "tauri-plugin-process",
            "tauri-plugin-single-instance",
            "tauri-plugin-updater",
            "tauri-plugin-window-state",
        ])
    );
}

/// The lines of one `[section]` of a TOML file, without comments and blank lines.
fn section(text: &str, header: &str) -> BTreeSet<String> {
    let mut inside = false;
    let mut lines = BTreeSet::new();
    for line in text.lines().map(str::trim) {
        if line.starts_with('[') {
            inside = line == header;
            continue;
        }
        if inside && !line.is_empty() && !line.starts_with('#') {
            lines.insert(line.to_owned());
        }
    }
    lines
}

#[test]
fn app_lints_match_workspace_lints() {
    let root = include_str!("../../Cargo.toml");
    let app = include_str!("../Cargo.toml");
    let workspace_clippy = section(root, "[workspace.lints.clippy]");
    assert!(workspace_clippy.contains("unwrap_used = \"forbid\""));
    assert_eq!(section(app, "[lints.clippy]"), workspace_clippy);
    assert_eq!(
        section(root, "[workspace.lints.rust]"),
        set(&["unsafe_code = \"forbid\""])
    );
    // The one difference: "deny" in the app so src/webview2.rs can allow it.
    assert_eq!(
        section(app, "[lints.rust]"),
        set(&["unsafe_code = \"deny\""])
    );
}

/// Lint levels that switch `unsafe_code` off or down. `deny` and `forbid` are fine.
const WEAK_LEVELS: [&str; 3] = ["allow", "expect", "warn"];

/// True if the text has `allow(...)`, `expect(...)` or `warn(...)` with
/// `unsafe_code` anywhere in its list. Catches lists (`allow(dead_code, unsafe_code)`),
/// spaces and line breaks, inner and outer attributes, and `cfg_attr(..., allow(...))`,
/// because it looks at every level word, wherever it stands.
fn mentions_unsafe_lint(text: &str) -> bool {
    WEAK_LEVELS.iter().any(|level| {
        level_lists(text, level)
            .iter()
            .any(|list| list_has_unsafe(list))
    })
}

/// The text inside the brackets after each `level(` that is a whole word.
fn level_lists<'a>(text: &'a str, level: &str) -> Vec<&'a str> {
    let mut lists = Vec::new();
    for (at, _) in text.match_indices(level) {
        let before = text[..at].chars().next_back();
        if before.is_some_and(|c| c.is_alphanumeric() || c == '_') {
            continue; // part of a longer word, for example `swallow(`
        }
        let rest = text[at + level.len()..].trim_start();
        let Some(inside) = rest.strip_prefix('(') else {
            continue;
        };
        let mut depth = 1;
        for (i, c) in inside.char_indices() {
            match c {
                '(' => depth += 1,
                ')' => depth -= 1,
                _ => {}
            }
            if depth == 0 {
                lists.push(&inside[..i]);
                break;
            }
        }
    }
    lists
}

fn list_has_unsafe(list: &str) -> bool {
    list.split(',').any(|item| item.trim() == "unsafe_code")
}

#[test]
fn unsafe_lint_check_catches_every_form() {
    // Caught: every weak level, lists, spaces, line breaks, cfg_attr.
    assert!(mentions_unsafe_lint("#[allow(unsafe_code)]"));
    assert!(mentions_unsafe_lint("#![expect(unsafe_code)]"));
    assert!(mentions_unsafe_lint("#[warn(unsafe_code)]"));
    assert!(mentions_unsafe_lint("#[allow(dead_code, unsafe_code)]"));
    assert!(mentions_unsafe_lint(
        "#[allow( unused ,\n    unsafe_code ,)]"
    ));
    assert!(mentions_unsafe_lint(
        "#[expect(unsafe_code, reason = \"x\")]"
    ));
    assert!(mentions_unsafe_lint(
        "#[cfg_attr(windows, allow(unsafe_code))]"
    ));
    assert!(mentions_unsafe_lint(
        "#![cfg_attr(all(test, windows), expect(dead_code, unsafe_code))]"
    ));
    assert!(mentions_unsafe_lint("#[allow (unsafe_code)]"));
    // Not caught: the strong levels, other lints, longer words.
    assert!(!mentions_unsafe_lint("#![forbid(unsafe_code)]"));
    assert!(!mentions_unsafe_lint("#![deny(unsafe_code)]"));
    assert!(!mentions_unsafe_lint("#[allow(dead_code)]"));
    assert!(!mentions_unsafe_lint("#[allow(unsafe_code_extra)]"));
    assert!(!mentions_unsafe_lint("fn swallow(unsafe_code: u8) {}"));
}

/// Every `.rs` file of the app that the compiler builds, except this test file,
/// whose samples above would match on purpose.
fn app_rust_files() -> Vec<std::path::PathBuf> {
    let app = Path::new(env!("CARGO_MANIFEST_DIR"));
    let this_file = app.join("tests").join("workspace_rules.rs");
    let mut files = vec![app.join("build.rs")];
    let mut stack = vec![app.join("src"), app.join("tests")];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|e| e == "rs") && path != this_file {
                files.push(path);
            }
        }
    }
    files
}

#[test]
fn unsafe_is_allowed_only_in_the_webview2_module() {
    let app = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut found: Vec<String> = app_rust_files()
        .iter()
        .filter(|path| mentions_unsafe_lint(&std::fs::read_to_string(path).unwrap()))
        .map(|path| {
            path.strip_prefix(app)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/")
        })
        .collect();
    found.sort();
    assert_eq!(found, vec!["src/webview2.rs".to_owned()]);
}
