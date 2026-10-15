//! No `f32` or `f64` in the money path (Code Standards M3, plan 02 acceptance 4, end-gate
//! review 16).
//!
//! `core::translit` scores spelling candidates with `f64`. That is the one allowed exception: it
//! never touches money, and it only ranks suggestions. This test keeps the exception in its
//! place. The money, validation, date, text and amount-in-words code has no float type, and no
//! file outside `translit` has one. `translit` does not use the money types either.
#![cfg(test)]

use std::fs;
use std::path::{Path, PathBuf};

/// Source files and folders (under `src/`) that hold or read money, input checks, dates, search
/// keys and amounts in words. None of them may name a float type.
const NO_FLOAT_PATHS: &[&str] = &[
    "money",
    "validate.rs",
    "dates.rs",
    "text.rs",
    "urdu_words.rs",
    "english_words.rs",
];

fn src() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

fn rust_files(path: &Path, out: &mut Vec<PathBuf>) {
    if path.is_dir() {
        let mut entries: Vec<PathBuf> = fs::read_dir(path)
            .unwrap()
            .map(|e| e.unwrap().path())
            .collect();
        entries.sort();
        for entry in entries {
            rust_files(&entry, out);
        }
    } else if path.extension().is_some_and(|e| e == "rs") {
        out.push(path.to_owned());
    }
}

/// The text without `//` comments (a doc line may talk about floats).
fn without_comments(text: &str) -> String {
    text.lines()
        .map(|line| line.split("//").next().unwrap_or(""))
        .collect::<Vec<_>>()
        .join("\n")
}

/// The code of a file without `//` comments.
fn code_of(path: &Path) -> String {
    without_comments(&fs::read_to_string(path).unwrap())
}

/// True if `text` holds `word` as a whole identifier.
fn has_word(text: &str, word: &str) -> bool {
    let is_ident = |c: char| c.is_alphanumeric() || c == '_';
    text.match_indices(word).any(|(at, _)| {
        let before = text[..at].chars().next_back();
        let after = text[at + word.len()..].chars().next();
        !before.is_some_and(is_ident) && !after.is_some_and(is_ident)
    })
}

fn has_float_type(text: &str) -> bool {
    has_word(text, "f32") || has_word(text, "f64")
}

#[test]
fn the_money_and_input_code_has_no_float_type() {
    let mut checked = 0;
    for name in NO_FLOAT_PATHS {
        let mut files = Vec::new();
        let path = src().join(name);
        assert!(path.exists(), "{name} is not under src/");
        rust_files(&path, &mut files);
        for file in files {
            checked += 1;
            assert!(
                !has_float_type(&code_of(&file)),
                "{} names f32 or f64 (Code Standards M3)",
                file.display()
            );
        }
    }
    assert!(checked >= 9, "only {checked} files were checked");
}

#[test]
fn only_the_translit_scoring_may_use_floats() {
    let mut files = Vec::new();
    rust_files(&src(), &mut files);
    let translit = src().join("translit");
    for file in files {
        if file.starts_with(&translit) {
            continue;
        }
        assert!(
            !has_float_type(&code_of(&file)),
            "{} names f32 or f64: floats are allowed in core::translit only",
            file.display()
        );
    }
}

#[test]
fn translit_does_not_touch_the_money_types() {
    // The exception holds only while the floats cannot reach an amount.
    let mut files = Vec::new();
    rust_files(&src().join("translit"), &mut files);
    for file in files {
        let code = code_of(&file);
        for word in ["Paisa", "Bps", "Qty", "Grams", "round_to_rupee"] {
            assert!(
                !has_word(&code, word),
                "{} uses {word}: translit must stay apart from money",
                file.display()
            );
        }
        assert!(
            !code.contains("money::"),
            "{} imports money",
            file.display()
        );
    }
}

#[test]
fn the_guard_sees_a_float_when_there_is_one() {
    // The scanner itself: a type, a cast, a word that only contains it, and a comment.
    assert!(has_float_type("let x: f64 = 1.0;"));
    assert!(has_float_type("n as f32"));
    assert!(has_float_type("Vec<f64>"));
    assert!(!has_float_type("let buf64 = 1; let f640 = 2;"));
    assert!(!has_float_type(&without_comments(
        "// an f64 in a comment\nlet a = 1;"
    )));
}
