//! Name suggestions and learned words through the services (PRD FR-7.22, FR-7.24, FR-7.26,
//! Code Standards B28, plan 05 Section 7).
#![cfg(test)]

mod common;

use common::*;
use mandi_services::core::error::{ErrorCode, ErrorValue};
use mandi_services::core::translit::{Direction, Kind, Suggestion};
use mandi_services::testing::TestEnv;
use mandi_services::translit::{self, NewPair};
use mandi_services::{ServiceError, auth};
use tempfile::TempDir;

fn ready() -> (TempDir, TestEnv, auth::Actor) {
    let (dir, env) = env();
    set_up(&env);
    login_owner(&env);
    let owner = auth::require(&env.ctx).unwrap();
    (dir, env, owner)
}

/// The text of each suggestion.
fn texts(suggestions: Vec<Suggestion>) -> Vec<String> {
    suggestions.into_iter().map(|s| s.text).collect()
}

fn pair(roman: &str, urdu: &str) -> NewPair {
    NewPair {
        roman: roman.to_owned(),
        urdu: urdu.to_owned(),
        kind: Kind::Word,
    }
}

fn rejected(error: ServiceError) -> Vec<(String, ErrorCode)> {
    match error {
        ServiceError::Rejected(items) => items
            .into_iter()
            .map(|i| (i.field.unwrap_or_default(), i.code))
            .collect(),
        other => panic!("not a rejection: {other:?}"),
    }
}

fn pair_rows(dir: &TempDir) -> i64 {
    raw(dir)
        .query_row("SELECT COUNT(*) FROM translit_pairs", [], |r| r.get(0))
        .unwrap()
}

#[test]
fn the_plan_examples_are_suggested_in_both_directions() {
    let (_dir, env, _) = ready();
    let up = |text: &str| texts(translit::suggest(&env.ctx, text, Direction::ToUrdu).unwrap());
    let down = |text: &str| texts(translit::suggest(&env.ctx, text, Direction::ToEnglish).unwrap());
    // E2E-23.
    assert_eq!(up("tariq mehmood")[0], "طارق محمود");
    assert_eq!(down("طارق محمود")[0], "Tariq Mehmood");
    assert_eq!(up("arshad multani")[0], "ارشد ملتانی");
    assert_eq!(down("ارشد ملتانی")[0], "Arshad Multani");
    // NFR section 0.1: two Urdu spellings of Shahid, the usual one first.
    assert_eq!(&up("shahid")[..2], ["شاہد", "شاھد"]);
    assert_eq!(up("")[..], Vec::<String>::new()[..]);
}

#[test]
fn suggesting_writes_nothing_and_refuses_a_text_that_is_too_long() {
    let (dir, env, _) = ready();
    let before = audit_total(&dir);
    let rows = pair_rows(&dir);
    for text in ["arshad", "😀", "ارشد", " "] {
        texts(translit::suggest(&env.ctx, text, Direction::ToUrdu).unwrap());
        texts(translit::suggest(&env.ctx, text, Direction::ToEnglish).unwrap());
    }
    let exactly = "a".repeat(500);
    texts(translit::suggest(&env.ctx, &exactly, Direction::ToUrdu).unwrap());
    let too_long = "a".repeat(501);
    let error = translit::suggest(&env.ctx, &too_long, Direction::ToUrdu).unwrap_err();
    assert_eq!(
        code_and_value(&error, "max"),
        (ErrorCode::ErrTextTooLong, Some(500))
    );
    assert_eq!(
        rejected(error),
        [("text".to_owned(), ErrorCode::ErrTextTooLong)]
    );
    // A huge text is refused, not read.
    let huge = "x ".repeat(2_500_000);
    assert!(translit::suggest(&env.ctx, &huge, Direction::ToEnglish).is_err());
    assert_eq!((audit_total(&dir), pair_rows(&dir)), (before, rows));
}

#[test]
fn a_learned_pair_is_offered_first_from_the_next_word() {
    let (dir, env, owner) = ready();
    let up = |text: &str| texts(translit::suggest(&env.ctx, text, Direction::ToUrdu).unwrap());
    let before = up("xyz");
    assert_ne!(before[0], "زیڈ");
    let row = translit::learn(&env.ctx, &owner, &pair("xyz", "زیڈ")).unwrap();
    assert_eq!(
        (row.roman.as_str(), row.urdu.as_str(), row.use_count),
        ("xyz", "زیڈ", 1)
    );
    assert_eq!(
        row.last_used_at.as_deref(),
        Some("2026-10-10T09:00:00.000Z")
    );
    assert_eq!(up("xyz")[0], "زیڈ");
    assert_eq!(up("arshad xyz")[0], "ارشد زیڈ", "inside a phrase");
    assert_eq!(
        texts(translit::suggest(&env.ctx, "زیڈ", Direction::ToEnglish).unwrap())[0],
        "Xyz"
    );
    // Choosing it again counts one more use and does not make a second row.
    env.clock.advance_seconds(60);
    let again = translit::learn(&env.ctx, &owner, &pair("XYZ", "زیڈ")).unwrap();
    assert_eq!((again.id, again.use_count), (row.id, 2));
    assert_eq!(
        again.last_used_at.as_deref(),
        Some("2026-10-10T09:01:00.000Z")
    );
    assert_eq!(pair_rows(&dir), 1);
    // One audit row per learning, with the old count on the second.
    assert_eq!(audit_count(&dir, "learn"), 2);
    let (old, new): (Option<String>, String) = raw(&dir)
        .query_row(
            "SELECT old_json, new_json FROM audit_log WHERE action = 'learn' ORDER BY id DESC LIMIT 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!(old.as_deref(), Some("{\"use_count\":1}"));
    assert!(
        new.contains("\"use_count\":2") && new.contains("\"roman\":\"xyz\""),
        "{new}"
    );
    assert_chain_ok(&dir);
}

#[test]
fn the_pair_most_used_comes_first_and_a_second_spelling_follows() {
    let (_dir, env, owner) = ready();
    let up = |text: &str| texts(translit::suggest(&env.ctx, text, Direction::ToUrdu).unwrap());
    translit::learn(&env.ctx, &owner, &pair("shahid", "شاھد")).unwrap();
    assert_eq!(&up("shahid")[..2], ["شاھد", "شاہد"]);
    for _ in 0..3 {
        translit::learn(&env.ctx, &owner, &pair("zed", "زیڈ")).unwrap();
    }
    translit::learn(&env.ctx, &owner, &pair("zed", "زڈ")).unwrap();
    assert_eq!(&up("zed")[..2], ["زیڈ", "زڈ"]);
}

#[test]
fn learned_words_survive_a_restart_a_reopened_file_and_a_reload() {
    let (dir, env, owner) = ready();
    translit::learn(&env.ctx, &owner, &pair("xyz", "زیڈ")).unwrap();
    // A new context on the same open file: nothing in memory.
    let fresh = mandi_services::Context::with_parts(
        env.ctx.db().clone(),
        mandi_services::Parts {
            monotonic: std::sync::Arc::new(mandi_services::testing::ManualMonotonic::default()),
            hasher: mandi_services::testing::fast_hasher().unwrap(),
            random: std::sync::Arc::new(mandi_services::testing::CountingRandom::default()),
        },
    );
    assert_eq!(
        texts(translit::suggest(&fresh, "xyz", Direction::ToUrdu).unwrap())[0],
        "زیڈ"
    );
    // The data file closed and opened again (next day of work, or after an update).
    drop(fresh);
    drop(owner);
    drop(env);
    let reopened = TestEnv::new(dir.path()).unwrap();
    assert_eq!(
        texts(translit::suggest(&reopened.ctx, "xyz", Direction::ToUrdu).unwrap())[0],
        "زیڈ"
    );
    // A row put into the table behind the service's back is picked up after a reload.
    raw(&dir)
        .execute(
            "INSERT INTO translit_pairs (roman, urdu, kind, use_count) VALUES ('qqq', 'کیوکیو', 'word', 4)",
            [],
        )
        .unwrap();
    assert_ne!(
        texts(translit::suggest(&reopened.ctx, "qqq", Direction::ToUrdu).unwrap())[0],
        "کیوکیو"
    );
    translit::reload(&reopened.ctx);
    assert_eq!(
        texts(translit::suggest(&reopened.ctx, "qqq", Direction::ToUrdu).unwrap())[0],
        "کیوکیو"
    );
}

#[test]
fn learning_never_changes_a_saved_name() {
    let (dir, env, owner) = ready();
    let names = |dir: &TempDir| -> String {
        raw(dir)
            .query_row(
                "SELECT group_concat(code || name_ur || name_en || search_ur || search_en || search_phonetic, '|') FROM accounts",
                [],
                |r| r.get(0),
            )
            .unwrap()
    };
    let firm = |env: &TestEnv| mandi_services::settings::get(&env.ctx).unwrap().firm;
    let (names_before, firm_before) = (names(&dir), firm(&env));
    assert!(!names_before.is_empty(), "the seeded accounts are there");
    // Teach it a different spelling of a word the saved names use.
    translit::learn(&env.ctx, &owner, &pair("rahim", "رہیم")).unwrap();
    translit::learn(&env.ctx, &owner, &pair("adnan", "اڈنان")).unwrap();
    translit::remove(&env.ctx, &owner, 1).unwrap();
    assert_eq!(names(&dir), names_before);
    assert_eq!(firm(&env), firm_before);
}

#[test]
fn bad_words_are_refused_with_their_field_and_nothing_is_saved() {
    let (dir, env, owner) = ready();
    let before = audit_total(&dir);
    type Case<'a> = (&'a str, &'a str, Vec<(&'a str, ErrorCode)>);
    let cases: [Case; 14] = [
        ("", "زیڈ", vec![("roman", ErrorCode::ErrRequired)]),
        ("xyz", "", vec![("urdu", ErrorCode::ErrRequired)]),
        (
            "  ",
            "  ",
            vec![
                ("roman", ErrorCode::ErrRequired),
                ("urdu", ErrorCode::ErrRequired),
            ],
        ),
        ("زیڈ", "زیڈ", vec![("roman", ErrorCode::ErrNameEnScript)]),
        ("xyz", "xyz", vec![("urdu", ErrorCode::ErrNameUrScript)]),
        (
            "زیڈ",
            "xyz",
            vec![
                ("roman", ErrorCode::ErrNameEnScript),
                ("urdu", ErrorCode::ErrNameUrScript),
            ],
        ),
        (
            "two words",
            "زیڈ",
            vec![("roman", ErrorCode::ErrNameEnScript)],
        ),
        ("xyz", "دو لفظ", vec![("urdu", ErrorCode::ErrNameUrScript)]),
        ("xy1", "زیڈ", vec![("roman", ErrorCode::ErrNameEnScript)]),
        ("xyz", "زی1", vec![("urdu", ErrorCode::ErrNameUrScript)]),
        ("x-y", "زیڈ", vec![("roman", ErrorCode::ErrNameEnScript)]),
        ("xyz", "زی.ڈ", vec![("urdu", ErrorCode::ErrNameUrScript)]),
        ("x\nyz", "زیڈ", vec![("roman", ErrorCode::ErrControlChars)]),
        (
            "😀",
            "😀",
            vec![
                ("roman", ErrorCode::ErrNameEnScript),
                ("urdu", ErrorCode::ErrNameUrScript),
            ],
        ),
    ];
    for (roman, urdu, want) in cases {
        let error = translit::learn(&env.ctx, &owner, &pair(roman, urdu)).unwrap_err();
        let got = rejected(error);
        let want: Vec<(String, ErrorCode)> =
            want.into_iter().map(|(f, c)| (f.to_owned(), c)).collect();
        assert_eq!(got, want, "{roman:?} / {urdu:?}");
    }
    // Too long: 40 letters are fine, 41 are not, and the message knows the limit.
    let forty = "a".repeat(40);
    let urdu_forty = "ا".repeat(40);
    translit::learn(&env.ctx, &owner, &pair(&forty, &urdu_forty)).unwrap();
    let error =
        translit::learn(&env.ctx, &owner, &pair(&"a".repeat(41), &"ا".repeat(41))).unwrap_err();
    let ServiceError::Rejected(items) = error else {
        panic!()
    };
    assert_eq!(items.len(), 2);
    for item in &items {
        assert_eq!(item.code, ErrorCode::ErrNameTooLong);
        assert_eq!(item.values.get("max"), Some(&ErrorValue::Number(40)));
    }
    assert_eq!(
        audit_total(&dir),
        before + 1,
        "only the one good word was written"
    );
    assert_eq!(pair_rows(&dir), 1);
    assert_chain_ok(&dir);
}

#[test]
fn words_are_cleaned_before_they_are_saved() {
    let (_dir, env, owner) = ready();
    // Upper case and spaces around; a zero-width joiner inside the Urdu.
    let row = translit::learn(&env.ctx, &owner, &pair("  ARSHAD ", "ار\u{200D}شد")).unwrap();
    assert_eq!((row.roman.as_str(), row.urdu.as_str()), ("arshad", "ارشد"));
    // A word with a mark (khari zabar) is a word.
    let row = translit::learn(&env.ctx, &owner, &pair("mustafa", "مصطفیٰ")).unwrap();
    assert_eq!(row.urdu, "مصطفیٰ");
    // The kind is kept.
    let kinds = [Kind::Name, Kind::Place, Kind::Item, Kind::Word];
    for (i, kind) in kinds.into_iter().enumerate() {
        let new = NewPair {
            roman: format!("kind{}", "a".repeat(i + 1)),
            urdu: "کند".to_owned(),
            kind,
        };
        let row = translit::learn(&env.ctx, &owner, &new).unwrap();
        assert_eq!(row.kind, kind);
    }
}

#[test]
fn the_owner_lists_and_removes_pairs_and_it_is_audited() {
    let (dir, env, owner) = ready();
    for (roman, urdu, times) in [("aaa", "ااا", 1), ("bbb", "ببب", 3), ("ccc", "ححح", 2)] {
        for _ in 0..times {
            translit::learn(&env.ctx, &owner, &pair(roman, urdu)).unwrap();
        }
    }
    let rows = translit::list(&env.ctx).unwrap();
    assert_eq!(
        rows.iter()
            .map(|r| (r.roman.as_str(), r.use_count))
            .collect::<Vec<_>>(),
        [("bbb", 3), ("ccc", 2), ("aaa", 1)]
    );
    // Removing it stops it being offered, at once, and writes an audit row.
    let bbb = rows[0].id;
    assert_eq!(
        texts(translit::suggest(&env.ctx, "bbb", Direction::ToUrdu).unwrap())[0],
        "ببب"
    );
    translit::remove(&env.ctx, &owner, bbb).unwrap();
    assert_ne!(
        texts(translit::suggest(&env.ctx, "bbb", Direction::ToUrdu).unwrap())[0],
        "ببب"
    );
    assert_eq!(pair_rows(&dir), 2);
    assert_eq!(audit_count(&dir, "remove"), 1);
    let old: String = raw(&dir)
        .query_row(
            "SELECT old_json FROM audit_log WHERE action = 'remove'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert!(
        old.contains("\"roman\":\"bbb\"") && old.contains("\"use_count\":3"),
        "{old}"
    );
    // Unknown id.
    let error = translit::remove(&env.ctx, &owner, 9_999).unwrap_err();
    assert_eq!(rejected(error), [("id".to_owned(), ErrorCode::ErrNotFound)]);
    assert_eq!(
        translit::remove(&env.ctx, &owner, bbb).unwrap_err().code(),
        ErrorCode::ErrNotFound
    );
    assert_eq!(
        audit_count(&dir, "remove"),
        1,
        "a failed removal writes nothing"
    );
    // The same word can be learned again after it was removed.
    let again = translit::learn(&env.ctx, &owner, &pair("bbb", "ببب")).unwrap();
    assert_eq!(again.use_count, 1);
    assert_chain_ok(&dir);
}

#[test]
fn two_at_once_are_both_counted() {
    let (dir, env, owner) = ready();
    let ctx = env.ctx.clone();
    let threads: Vec<_> = (0..4)
        .map(|_| {
            let ctx = ctx.clone();
            let owner = owner.clone();
            std::thread::spawn(move || {
                translit::learn(&ctx, &owner, &pair("xyz", "زیڈ"))
                    .unwrap()
                    .use_count
            })
        })
        .collect();
    let mut counts: Vec<u32> = threads.into_iter().map(|t| t.join().unwrap()).collect();
    counts.sort_unstable();
    assert_eq!(counts, [1, 2, 3, 4]);
    assert_eq!(pair_rows(&dir), 1);
    translit::reload(&env.ctx);
    assert_eq!(
        texts(translit::suggest(&env.ctx, "xyz", Direction::ToUrdu).unwrap())[0],
        "زیڈ"
    );
}

#[test]
fn hostile_text_never_breaks_a_service_call() {
    let (_dir, env, _) = ready();
    for text in [
        "\0",
        "😀😀😀",
        "ar\u{200B}shad",
        "\u{202E}arshad",
        "arshad ارشد",
        &"a".repeat(500),
        &"ا".repeat(500),
    ] {
        for direction in [Direction::ToUrdu, Direction::ToEnglish] {
            let got = texts(translit::suggest(&env.ctx, text, direction).unwrap());
            assert!(got.len() <= 3);
        }
    }
}
