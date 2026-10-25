//! The spelling check before a named record is saved (PRD FR-7.66, E49, plan 05 Section 4,
//! acceptance D2-A17): a spelling that only the rules made is refused until the user confirms
//! it, confirming learns the pair, and a save that fails learns nothing.
#![cfg(test)]

mod common;

use common::*;
use mandi_services::auth;
use mandi_services::core::error::{ErrorCode, ErrorValue};
use mandi_services::core::translit::{Direction, Kind, Source};
use mandi_services::settings::{self, Firm};
use mandi_services::spelling::{self, NamePair};
use mandi_services::testing::TestEnv;
use mandi_services::{Rejection, ServiceError, translit};
use tempfile::TempDir;

fn ready() -> (TempDir, TestEnv, auth::Actor) {
    let (dir, env) = env();
    set_up(&env);
    login_owner(&env);
    let owner = auth::require(&env.ctx).unwrap();
    (dir, env, owner)
}

fn names<'a>(name_ur: &'a str, name_en: &'a str) -> NamePair<'a> {
    NamePair {
        field_ur: "name_ur",
        name_ur,
        field_en: "name_en",
        name_en,
        kind: Kind::Name,
    }
}

fn rejections(error: ServiceError) -> Vec<Rejection> {
    match error {
        ServiceError::Rejected(items) => items,
        other => panic!("not a rejection: {other:?}"),
    }
}

fn words_of(rejection: &Rejection) -> String {
    match rejection.values.get("words") {
        Some(ErrorValue::Text(words)) => words.clone(),
        other => panic!("no words value: {other:?}"),
    }
}

fn confirm(words: &[&str]) -> Vec<String> {
    words.iter().map(|w| (*w).to_owned()).collect()
}

fn pair_rows(dir: &TempDir) -> i64 {
    raw(dir)
        .query_row("SELECT COUNT(*) FROM translit_pairs", [], |r| r.get(0))
        .unwrap()
}

#[test]
fn names_made_of_listed_words_pass_without_any_confirmation() {
    // E2E-23: tariq mehmood -> طارق محمود. Every word is in the word list.
    let (_dir, env, _owner) = ready();
    let check = spelling::check(&env.ctx, &names("طارق محمود", "Tariq Mehmood"), &[]).unwrap();
    assert!(check.is_empty(), "nothing to learn");
    // An initial and an ampersand are not spellings.
    spelling::check(
        &env.ctx,
        &names("ارشد ملتانی", "M. Arshad Multani & 2"),
        &[],
    )
    .unwrap();
}

#[test]
fn a_rules_only_word_is_refused_on_its_field_until_it_is_confirmed() {
    let (dir, env, _owner) = ready();
    let audit_before = audit_total(&dir);
    let error = spelling::check(&env.ctx, &names("علی قزکس", "Ali Qzxwv"), &[]).unwrap_err();
    let items = rejections(error);
    // Both fields are reported together, each with its words.
    assert_eq!(items.len(), 2);
    assert_eq!(
        (
            items[0].code,
            items[0].field.as_deref(),
            words_of(&items[0])
        ),
        (
            ErrorCode::ErrSpellingUnconfirmed,
            Some("name_ur"),
            "قزکس".to_owned()
        )
    );
    assert_eq!(
        (
            items[1].code,
            items[1].field.as_deref(),
            words_of(&items[1])
        ),
        (
            ErrorCode::ErrSpellingUnconfirmed,
            Some("name_en"),
            "Qzxwv".to_owned()
        )
    );
    // Refused: nothing was written.
    assert_eq!(pair_rows(&dir), 0);
    assert_eq!(audit_total(&dir), audit_before);
}

#[test]
fn only_one_field_is_reported_when_the_other_is_confirmed() {
    let (_dir, env, _owner) = ready();
    let error = spelling::check(
        &env.ctx,
        &names("علی قزکس", "Ali Qzxwv"),
        &confirm(&["Qzxwv"]),
    )
    .unwrap_err();
    let items = rejections(error);
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].field.as_deref(), Some("name_ur"));
    assert_eq!(words_of(&items[0]), "قزکس");
    // Confirming a word that is not in the name changes nothing.
    let error = spelling::check(
        &env.ctx,
        &names("علی قزکس", "Ali Qzxwv"),
        &confirm(&["Other", "قزکس"]),
    )
    .unwrap_err();
    assert_eq!(rejections(error)[0].field.as_deref(), Some("name_en"));
}

#[test]
fn confirming_accepts_the_save_and_learns_the_pair_in_the_same_transaction() {
    let (dir, env, owner) = ready();
    let confirmed = confirm(&["قزکس", "Qzxwv"]);
    let check = spelling::check(&env.ctx, &names("علی قزکس", "Ali Qzxwv"), &confirmed).unwrap();
    assert_eq!(check.pairs(), [("qzxwv", "قزکس")]);
    // The save's transaction: learn, then commit.
    let learned = env
        .ctx
        .db()
        .write(|tx| check.learn(&env.ctx, &owner, tx))
        .unwrap();
    assert_eq!(learned.len(), 1);
    // Not in memory until the commit is reported.
    learned.after_commit(&env.ctx);
    assert_eq!(pair_rows(&dir), 1);
    let (roman, urdu, kind, uses): (String, String, String, i64) = raw(&dir)
        .query_row(
            "SELECT roman, urdu, kind, use_count FROM translit_pairs",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .unwrap();
    assert_eq!(
        (roman.as_str(), urdu.as_str(), kind.as_str(), uses),
        ("qzxwv", "قزکس", "name", 1)
    );
    assert_eq!(audit_count(&dir, "learn"), 1);
    assert_chain_ok(&dir);
    // From now on the word is known: no confirmation is asked, and the suggestion says learned.
    spelling::check(&env.ctx, &names("علی قزکس", "Ali Qzxwv"), &[]).unwrap();
    let got = translit::suggest(&env.ctx, "qzxwv", Direction::ToUrdu).unwrap();
    assert_eq!(
        (got[0].text.as_str(), got[0].source),
        ("قزکس", Source::Learned)
    );
}

#[test]
fn a_learned_word_survives_a_reopened_database() {
    let (dir, env, owner) = ready();
    let check = spelling::check(
        &env.ctx,
        &names("علی قزکس", "Ali Qzxwv"),
        &confirm(&["قزکس", "Qzxwv"]),
    )
    .unwrap();
    let learned = env
        .ctx
        .db()
        .write(|tx| check.learn(&env.ctx, &owner, tx))
        .unwrap();
    learned.after_commit(&env.ctx);
    drop(owner);
    drop(env);
    let reopened = TestEnv::new(dir.path()).unwrap();
    spelling::check(&reopened.ctx, &names("علی قزکس", "Ali Qzxwv"), &[]).unwrap();
}

#[test]
fn a_save_that_fails_after_the_check_learns_nothing() {
    let (dir, env, owner) = ready();
    let check = spelling::check(
        &env.ctx,
        &names("علی قزکس", "Ali Qzxwv"),
        &confirm(&["قزکس", "Qzxwv"]),
    )
    .unwrap();
    let audit_before = audit_total(&dir);
    // The save learns the pair and then fails for another reason: the transaction rolls back.
    let failed: Result<(), ServiceError> = env.ctx.db().write(|tx| {
        let _learned = check.learn(&env.ctx, &owner, tx)?;
        Err(ServiceError::reject(ErrorCode::ErrDateClosed))
    });
    assert!(failed.is_err());
    assert_eq!(pair_rows(&dir), 0, "no pair in the table");
    assert_eq!(audit_total(&dir), audit_before, "no audit row");
    // And not in memory either: the word is still unknown.
    assert!(spelling::check(&env.ctx, &names("علی قزکس", "Ali Qzxwv"), &[]).is_err());
    let got = translit::suggest(&env.ctx, "qzxwv", Direction::ToUrdu).unwrap();
    assert_ne!(got[0].source, Source::Learned);
}

#[test]
fn names_with_a_different_number_of_words_are_accepted_but_teach_nothing() {
    let (dir, env, owner) = ready();
    // Two Urdu words against three English words: no safe pairing.
    let check = spelling::check(
        &env.ctx,
        &names("علی قزکس", "Ali Qzxwv Wvbnm"),
        &confirm(&["قزکس", "Qzxwv", "Wvbnm"]),
    )
    .unwrap();
    assert!(check.is_empty());
    let learned = env
        .ctx
        .db()
        .write(|tx| check.learn(&env.ctx, &owner, tx))
        .unwrap();
    assert!(learned.is_empty());
    assert_eq!(pair_rows(&dir), 0);
}

#[test]
fn confirming_a_second_time_counts_one_more_use_not_a_second_row() {
    let (dir, env, owner) = ready();
    let confirmed = confirm(&["قزکس", "Qzxwv"]);
    for _ in 0..2 {
        // The word is known after the first time, so the second check has nothing to learn.
        let check = spelling::check(&env.ctx, &names("علی قزکس", "Ali Qzxwv"), &confirmed).unwrap();
        env.ctx
            .db()
            .write(|tx| check.learn(&env.ctx, &owner, tx))
            .unwrap()
            .after_commit(&env.ctx);
    }
    assert_eq!(pair_rows(&dir), 1);
}

#[test]
fn a_word_the_table_cannot_learn_is_skipped_and_the_save_goes_through() {
    // A confirmed word longer than a learned word may be (40 letters) cannot be learned. The
    // user has looked at it, so the save is not refused.
    let (dir, env, owner) = ready();
    let long_en = "Abcdefghijklmnopqrstuvwxyzabcdefghijklmnopq";
    let long_ur = "ابجدهوزحطیکلمنسعفصقرشتثخذضظغ".repeat(2);
    let name_ur = format!("علی {long_ur}");
    let name_en = format!("Ali {long_en}");
    let check = spelling::check(
        &env.ctx,
        &names(&name_ur, &name_en),
        &confirm(&[long_en, &long_ur]),
    )
    .unwrap();
    let learned = env
        .ctx
        .db()
        .write(|tx| check.learn(&env.ctx, &owner, tx))
        .unwrap();
    assert!(learned.is_empty());
    assert_eq!(pair_rows(&dir), 0);
}

#[test]
fn hostile_names_never_break_the_check() {
    let (_dir, env, _owner) = ready();
    let long = "a".repeat(5_000);
    for (ur, en) in [
        ("", ""),
        (" ", "\t"),
        ("😀", "😀"),
        ("a\0b", "ا\0ب"),
        ("علی".repeat(1_000).as_str(), long.as_str()),
    ] {
        // Whatever the result, no panic.
        let _ = spelling::check(&env.ctx, &names(ur, en), &[]);
    }
}

// ---------------------------------------------------------------------------------------------
// The firm and mandi names (end-gate review 3, PRD FR-7.24, FR-7.66).
// ---------------------------------------------------------------------------------------------

fn firm_with(name_ur: &str, name_en: &str, mandi_ur: &str, mandi_en: &str) -> Firm {
    Firm {
        name_ur: name_ur.into(),
        name_en: name_en.into(),
        mandi_ur: mandi_ur.into(),
        mandi_en: mandi_en.into(),
        address: "Kot Radha Kishan".into(),
        phone: "0300-1234567".into(),
    }
}

fn learned_pairs(dir: &TempDir) -> Vec<(String, String, String)> {
    let conn = raw(dir);
    let mut statement = conn
        .prepare("SELECT roman, urdu, kind FROM translit_pairs ORDER BY roman")
        .unwrap();
    statement
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
        .unwrap()
        .map(Result::unwrap)
        .collect()
}

#[test]
fn a_firm_or_mandi_name_with_a_rules_only_word_is_refused_until_confirmed() {
    let (dir, env, owner) = ready();
    let before = settings::get(&env.ctx).unwrap().firm;
    let audit_before = audit_total(&dir);
    // Both names have a word only the rules could have made: all four fields come back at once.
    let firm = firm_with("علی قزکس", "Ali Qzxwv", "قزکس منڈی", "Wvbnm Mandi");
    let items = rejections(settings::set_firm(&env.ctx, &owner, &firm).unwrap_err());
    let got: Vec<(&str, String)> = items
        .iter()
        .map(|i| (i.field.as_deref().unwrap_or(""), words_of(i)))
        .collect();
    assert_eq!(
        got,
        [
            ("name_ur", "قزکس".to_owned()),
            ("name_en", "Qzxwv".to_owned()),
            ("mandi_ur", "قزکس".to_owned()),
            ("mandi_en", "Wvbnm".to_owned()),
        ]
    );
    assert!(
        items
            .iter()
            .all(|i| i.code == ErrorCode::ErrSpellingUnconfirmed)
    );
    // Refused: the firm, the pairs and the audit log are as they were.
    assert_eq!(settings::get(&env.ctx).unwrap().firm, before);
    assert_eq!(pair_rows(&dir), 0);
    assert_eq!(audit_total(&dir), audit_before);
}

#[test]
fn confirming_saves_the_firm_and_learns_a_name_pair_and_a_place_pair() {
    let (dir, env, owner) = ready();
    let firm = firm_with("علی قزکس", "Ali Qzxwv", "قزکس منڈی", "Qzxwv Mandi");
    let saved = settings::set_firm_confirmed(&env.ctx, &owner, &firm, &confirm(&["قزکس", "Qzxwv"]))
        .unwrap();
    assert_eq!(saved.name_en, "Ali Qzxwv");
    assert_eq!(settings::get(&env.ctx).unwrap().firm, saved);
    // The firm name teaches a person-like pair; the same pair from the mandi name is one use
    // more, not a second row.
    assert_eq!(
        learned_pairs(&dir),
        [("qzxwv".to_owned(), "قزکس".to_owned(), "name".to_owned())]
    );
    let uses: i64 = raw(&dir)
        .query_row("SELECT use_count FROM translit_pairs", [], |r| r.get(0))
        .unwrap();
    assert_eq!(uses, 2);
    assert_eq!(audit_count(&dir, "learn"), 2);
    assert_chain_ok(&dir);
    // Known now: the plain save works with nothing confirmed.
    settings::set_firm(&env.ctx, &owner, &firm).unwrap();
}

#[test]
fn a_contact_name_with_a_rules_only_word_is_refused_until_confirmed() {
    // Plan 17e: the contact persons of the print header are names too (PRD FR-7.66).
    use mandi_services::core::firm::FirmContact;
    use mandi_services::settings::FirmPrintInfo;
    let (dir, env, owner) = ready();
    let firm = settings::get(&env.ctx).unwrap().firm;
    let info = FirmPrintInfo {
        firm_contacts: vec![FirmContact {
            name_ur: "علی قزکس".into(),
            name_en: "Ali Qzxwv".into(),
            phone: "0300-1234567".into(),
        }],
        trade_mark: "RH".into(),
    };
    let audit_before = audit_total(&dir);
    let items = rejections(
        settings::set_firm_with_print_info(&env.ctx, &owner, &firm, &info, &[]).unwrap_err(),
    );
    let got: Vec<(&str, String)> = items
        .iter()
        .map(|i| (i.field.as_deref().unwrap_or(""), words_of(i)))
        .collect();
    assert_eq!(
        got,
        [
            ("firm_contacts[0].name_ur", "قزکس".to_owned()),
            ("firm_contacts[0].name_en", "Qzxwv".to_owned()),
        ]
    );
    assert_eq!(audit_total(&dir), audit_before, "nothing was written");
    assert_eq!(pair_rows(&dir), 0);
    // Confirmed: it saves, and the pair is learned.
    let (_, saved) = settings::set_firm_with_print_info(
        &env.ctx,
        &owner,
        &firm,
        &info,
        &confirm(&["قزکس", "Qzxwv"]),
    )
    .unwrap();
    assert_eq!(saved.firm_contacts[0].name_en, "Ali Qzxwv");
    assert_eq!(
        learned_pairs(&dir),
        [("qzxwv".to_owned(), "قزکس".to_owned(), "name".to_owned())]
    );
}

#[test]
fn a_mandi_word_is_learned_as_a_place() {
    let (dir, env, owner) = ready();
    let firm = firm_with(
        "ماہر عدنان اینڈ برادرز",
        "Maher Adnan and Brothers",
        "قزکس منڈی",
        "Qzxwv Mandi",
    );
    settings::set_firm_confirmed(&env.ctx, &owner, &firm, &confirm(&["قزکس", "Qzxwv"])).unwrap();
    assert_eq!(
        learned_pairs(&dir),
        [("qzxwv".to_owned(), "قزکس".to_owned(), "place".to_owned())]
    );
}

#[test]
fn the_first_run_wizard_checks_the_spelling_of_the_firm_names_too() {
    let (dir, env) = env();
    let mut input = first_run_input();
    input.firm = firm_with("علی قزکس", "Ali Qzxwv", "نئی سبزی منڈی", "New Sabzi Mandi");
    // A bad user name and a rules-only word come back together.
    input.owner_username = "no".into();
    let items = rejections(auth::first_run(&env.ctx, &input).unwrap_err());
    let got: Vec<(&str, ErrorCode)> = items
        .iter()
        .map(|i| (i.field.as_deref().unwrap_or(""), i.code))
        .collect();
    assert!(got.contains(&("owner_username", ErrorCode::ErrUsernameInvalid)));
    assert!(got.contains(&("firm_name_ur", ErrorCode::ErrSpellingUnconfirmed)));
    assert!(got.contains(&("firm_name_en", ErrorCode::ErrSpellingUnconfirmed)));
    input.owner_username = "Owner".into();

    // Nothing is made: no user, no pair, no audit row.
    let refused = rejections(auth::first_run(&env.ctx, &input).unwrap_err());
    assert_eq!(refused.len(), 2);
    assert!(auth::status(&env.ctx).unwrap().first_run);
    assert_eq!(pair_rows(&dir), 0);
    assert_eq!(audit_total(&dir), 0);

    // Confirmed: the wizard goes through and the pair is learned, by the Owner just made.
    let code = auth::first_run_confirmed(&env.ctx, &input, &confirm(&["قزکس", "Qzxwv"])).unwrap();
    assert_eq!(code.len(), 19);
    assert!(!auth::status(&env.ctx).unwrap().first_run);
    assert_eq!(
        learned_pairs(&dir),
        [("qzxwv".to_owned(), "قزکس".to_owned(), "name".to_owned())]
    );
    let by: Option<i64> = raw(&dir)
        .query_row(
            "SELECT user_id FROM audit_log WHERE action = 'learn'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(by, Some(1));
    assert_chain_ok(&dir);
}
