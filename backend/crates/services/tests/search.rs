//! The search index and the picker commands (plan 06, acceptance D2-A6, NFR-SPD-6, PRD FR-7.40,
//! FR-7.41): both scripts, small mistakes, the order of the results, recent first, an index that
//! is built again with the same answers, and speed over 20,000 records.
#![cfg(test)]

mod common;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use std::time::Instant;

use common::*;
use mandi_services::core::masters::{CityInput, MasterInput, MasterKind};
use mandi_services::core::search::{SearchHit, SearchScope};
use mandi_services::core::text;
use mandi_services::core::translit::WordList;
use mandi_services::testing::TestEnv;
use mandi_services::{ServiceError, auth, masters, search};
use tempfile::TempDir;

fn ready() -> (TempDir, TestEnv, auth::Actor) {
    let (dir, env) = env();
    set_up(&env);
    login_owner(&env);
    let owner = auth::require(&env.ctx).unwrap();
    (dir, env, owner)
}

/// Adds a city, confirming whatever spelling the check asks about (the tests here are about
/// search, not about the spelling check).
fn add(env: &TestEnv, owner: &auth::Actor, code: &str, ur: &str, en: &str) -> i64 {
    let mut input = CityInput {
        id: None,
        version: None,
        code: code.into(),
        name_ur: ur.into(),
        name_en: en.into(),
        confirmed_rules_words: Vec::new(),
    };
    loop {
        match masters::save(&env.ctx, owner, &MasterInput::City(input.clone())) {
            Ok(row) => return row.id(),
            Err(ServiceError::Rejected(items))
                if items.iter().all(|i| {
                    i.code == mandi_services::core::error::ErrorCode::ErrSpellingUnconfirmed
                }) =>
            {
                for item in items {
                    if let Some(mandi_services::core::error::ErrorValue::Text(words)) =
                        item.values.get("words")
                    {
                        input
                            .confirmed_rules_words
                            .extend(words.split(' ').map(str::to_owned));
                    }
                }
            }
            Err(other) => panic!("{other:?}"),
        }
    }
}

fn names(hits: &[SearchHit]) -> Vec<String> {
    hits.iter().map(|h| h.name_en.clone()).collect()
}

fn find(env: &TestEnv, owner: &auth::Actor, text: &str) -> Vec<SearchHit> {
    search::search(&env.ctx, owner, SearchScope::City, text, 20).unwrap()
}

#[test]
fn a_mistyped_roman_word_and_look_alike_urdu_letters_find_the_record() {
    let (_dir, env, owner) = ready();
    add(&env, &owner, "", "اکرم ٹول ٹیکس", "Akram Toll Tax");
    add(&env, &owner, "", "ارشد ملتانی", "Arshad Multani");
    add(&env, &owner, "", "لاہور", "Lahore");
    // One mistake in 4 letters, a letter more, a letter less, the phonetic key, either script.
    for typed in [
        "akrm",
        "akram",
        "akrim",
        "akrem toll",
        "AKRAM",
        "  akram  ",
        "toll akram",
    ] {
        assert_eq!(
            names(&find(&env, &owner, typed))
                .first()
                .map(String::as_str),
            Some("Akram Toll Tax"),
            "{typed}"
        );
    }
    for typed in ["اکرم", "اكرم", "اکرام", "اکْرَم", "اک\u{200c}رم", "ٹول اکرم"]
    {
        assert_eq!(
            names(&find(&env, &owner, typed))
                .first()
                .map(String::as_str),
            Some("Akram Toll Tax"),
            "{typed}"
        );
    }
    // Arshad typed without its vowel (the plan example `arshd`).
    assert_eq!(
        names(&find(&env, &owner, "arshd"))
            .first()
            .map(String::as_str),
        Some("Arshad Multani")
    );
    // Nothing near: nothing. Empty text is the whole list (plan 32 R7): the three cities, by code.
    assert!(find(&env, &owner, "zzzzzzz").is_empty());
    assert_eq!(find(&env, &owner, "").len(), 3);
    assert_eq!(find(&env, &owner, "   ").len(), 3);
}

#[test]
fn the_order_is_code_then_starts_with_then_used_before_then_close() {
    let (_dir, env, owner) = ready();
    let kot = add(&env, &owner, "5", "کوٹ", "Kot");
    let kotli = add(&env, &owner, "6", "کوٹلی", "Kotli");
    let hot = add(&env, &owner, "7", "پوکوٹ", "Pokot"); // contains `kot`, does not start with it
    let koat = add(&env, &owner, "8", "کپٹ", "Kpt"); // one letter off `kot`, sounds different
    let by_code = add(&env, &owner, "100", "سرگودھا", "Sargodha");
    // Typing a code puts the exact code first, then names that start with the digits' text.
    let hits = find(&env, &owner, "100");
    assert_eq!(hits[0].id, by_code);
    let hits = find(&env, &owner, "5");
    assert_eq!(hits[0].id, kot, "exact code first");
    // Names that start with the text come before the rest; nothing used yet.
    let hits = find(&env, &owner, "kot");
    let ids: Vec<i64> = hits.iter().map(|h| h.id).collect();
    assert_eq!(
        &ids[..2],
        &[kot, kotli],
        "starts with: Kot and Kotli (Urdu alphabetical)"
    );
    assert!(ids.contains(&koat), "a close spelling is still offered");
    assert!(
        ids.iter().position(|i| *i == koat).unwrap() > 1,
        "the close spelling comes after the names that start with it"
    );
    // A record chosen before and containing the text comes before a close spelling, after the
    // starts-with ones.
    search::touch(&env.ctx, &owner, SearchScope::City, hot).unwrap();
    let ids: Vec<i64> = find(&env, &owner, "kot").iter().map(|h| h.id).collect();
    let (p_hot, p_koat) = (
        ids.iter().position(|i| *i == hot).unwrap(),
        ids.iter().position(|i| *i == koat).unwrap(),
    );
    assert!(p_hot > 1 && p_hot < p_koat, "{ids:?}");
}

#[test]
fn recent_and_often_chosen_records_come_first_among_equals() {
    let (_dir, env, owner) = ready();
    let a = add(&env, &owner, "", "ملتان", "Multan");
    let b = add(&env, &owner, "", "ملتانی منڈی", "Multani Mandi");
    let c = add(&env, &owner, "", "ملتان روڈ", "Multan Road");
    let order =
        |text: &str| -> Vec<i64> { find(&env, &owner, text).iter().map(|h| h.id).collect() };
    let first = order("multan");
    assert_eq!(first.len(), 3);
    // Choosing b, then c, then c again: c is latest and most often, then b, then the rest.
    for id in [b, c, c] {
        search::touch(&env.ctx, &owner, SearchScope::City, id).unwrap();
    }
    assert_eq!(order("multan"), [c, b, a]);
    // The full list for Alt+Down: the last chosen first, the rest after them.
    let full: Vec<i64> = search::recent(&env.ctx, &owner, SearchScope::City)
        .unwrap()
        .iter()
        .map(|h| h.id)
        .collect();
    assert_eq!(full, [c, b, a]);
}

#[test]
fn only_ten_are_kept_as_recent_and_touch_writes_only_the_preference() {
    let (dir, env, owner) = ready();
    let ids: Vec<i64> = (0..12)
        .map(|n| {
            add(
                &env,
                &owner,
                "",
                "لاہور",
                &format!(
                    "Lahore {}",
                    [
                        "One", "Two", "Three", "Four", "Five", "Six", "Seven", "Eight", "Nine",
                        "Ten", "Eleven", "Twelve"
                    ][n]
                ),
            )
        })
        .collect();
    let audit = audit_total(&dir);
    let tables = |dir: &TempDir| -> (i64, i64) {
        let c = raw(dir);
        (
            c.query_row("SELECT COUNT(*) FROM cities", [], |r| r.get(0))
                .unwrap(),
            c.query_row("SELECT COUNT(*) FROM translit_pairs", [], |r| r.get(0))
                .unwrap(),
        )
    };
    let before = tables(&dir);
    for id in &ids {
        search::touch(&env.ctx, &owner, SearchScope::City, *id).unwrap();
    }
    // B29: nothing but the user's preference changed. No audit row either.
    assert_eq!(tables(&dir), before);
    assert_eq!(audit_total(&dir), audit);
    let (recent, counts): (String, String) = {
        let value: String = raw(&dir)
            .query_row(
                "SELECT value_json FROM user_prefs WHERE key = 'recent.city'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        let v: serde_json::Value = serde_json::from_str(&value).unwrap();
        (v["recent"].to_string(), v["counts"].to_string())
    };
    let kept: Vec<i64> = serde_json::from_str(&recent).unwrap();
    assert_eq!(kept.len(), 10);
    assert_eq!(kept[0], ids[11], "the latest first");
    assert!(
        counts.contains(&format!("\"{}\":1", ids[0])),
        "counts keep every record"
    );
    // An id that is not in the list.
    let error = search::touch(&env.ctx, &owner, SearchScope::City, 9999).unwrap_err();
    assert!(error.has_code(mandi_services::core::error::ErrorCode::ErrNotFound));
}

#[test]
fn each_user_list_has_its_own_scope() {
    let (_dir, env, owner) = ready();
    let c = add(&env, &owner, "", "لاہور", "Lahore");
    // Same text in another list finds nothing there.
    let hits = search::search(&env.ctx, &owner, SearchScope::Store, "lahore", 10).unwrap();
    assert!(hits.is_empty());
    let hits = search::search(&env.ctx, &owner, SearchScope::City, "lahore", 10).unwrap();
    assert_eq!(hits[0].id, c);
    assert_eq!(hits[0].kind, SearchScope::City);
}

#[test]
fn a_save_updates_the_index_at_once_and_an_inactive_record_is_not_offered() {
    let (_dir, env, owner) = ready();
    let id = add(&env, &owner, "", "قصور", "Kasur");
    assert_eq!(find(&env, &owner, "kasur")[0].id, id);
    // Made inactive: hidden from the picker, kept in the list.
    masters::set_active(&env.ctx, &owner, MasterKind::City, id, false).unwrap();
    assert!(find(&env, &owner, "kasur").is_empty());
    assert!(
        search::recent(&env.ctx, &owner, SearchScope::City)
            .unwrap()
            .is_empty()
    );
    assert!(
        search::touch(&env.ctx, &owner, SearchScope::City, id).is_err(),
        "an inactive record cannot be chosen"
    );
    masters::set_active(&env.ctx, &owner, MasterKind::City, id, true).unwrap();
    assert_eq!(find(&env, &owner, "kasur")[0].id, id);
    // Renamed: found by the new name, not by the old.
    let edit = CityInput {
        id: Some(id),
        version: Some(masters::list(&env.ctx, MasterKind::City, true).unwrap()[0].version()),
        code: "".into(),
        name_ur: "لاہور".into(),
        name_en: "Lahore".into(),
        confirmed_rules_words: Vec::new(),
    };
    masters::save(&env.ctx, &owner, &MasterInput::City(edit)).unwrap();
    assert_eq!(find(&env, &owner, "lahore")[0].id, id);
    assert!(find(&env, &owner, "kasur").is_empty());
}

#[test]
fn the_index_built_again_gives_the_same_answers_after_a_simulated_restore() {
    let (dir, env, owner) = ready();
    for (ur, en) in [
        ("اکرم ٹول ٹیکس", "Akram Toll Tax"),
        ("ارشد ملتانی", "Arshad Multani"),
        ("ملتان", "Multan"),
        ("کوٹ رادھا کشن", "Kot Radha Kishan"),
    ] {
        add(&env, &owner, "", ur, en);
    }
    let queries = [
        "akrm",
        "arshd",
        "multan",
        "اکرام",
        "kot",
        "radha",
        "100",
        "toll",
    ];
    let answers = |env: &TestEnv, owner: &auth::Actor| -> Vec<Vec<SearchHit>> {
        queries.iter().map(|q| find(env, owner, q)).collect()
    };
    let before = answers(&env, &owner);
    // The same Context, index built again from the database.
    search::rebuild(&env.ctx).unwrap();
    assert_eq!(answers(&env, &owner), before);
    // A new Context on a reopened file (nothing in memory at all), as after a restore.
    drop(env);
    let reopened = TestEnv::new(dir.path()).unwrap();
    login_owner(&reopened);
    let owner = auth::require(&reopened.ctx).unwrap();
    assert_eq!(answers(&reopened, &owner), before);
}

#[test]
fn masters_without_search_columns_are_found() {
    use mandi_services::core::masters::{MarkaInput, NarrationInput};
    let (_dir, env, owner) = ready();
    masters::save(
        &env.ctx,
        &owner,
        &MasterInput::Marka(MarkaInput {
            id: None,
            version: None,
            code: "".into(),
            mark_text: "AR".into(),
            owner_account_id: None,
        }),
    )
    .unwrap();
    masters::save(
        &env.ctx,
        &owner,
        &MasterInput::Narration(NarrationInput {
            id: None,
            version: None,
            text_ur: "بجلی کا بل".into(),
            text_en: "Electricity bill".into(),
            used_in: Vec::new(),
        }),
    )
    .unwrap();
    let s =
        |scope: SearchScope, text: &str| search::search(&env.ctx, &owner, scope, text, 10).unwrap();
    assert_eq!(s(SearchScope::Marka, "ar")[0].name_en, "AR");
    assert_eq!(
        s(SearchScope::Narration, "electricity")[0].name_ur,
        "بجلی کا بل"
    );
    assert_eq!(
        s(SearchScope::Narration, "بجلی")[0].name_en,
        "Electricity bill"
    );
}

#[test]
fn twenty_thousand_records_are_searched_fast_in_both_scripts_with_mistakes() {
    // NFR-SPD-6: under 50 ms on the reference computer in a release build. This test runs in a
    // debug build on any computer, so it only checks a generous bound; `pnpm perf` holds the
    // real number.
    const RECORDS: usize = 20_000;
    const BOUND_MS: u128 = 600;
    let (dir, env, owner) = ready();
    let list = WordList::embedded();
    let words: Vec<(&str, &str)> = list
        .entries()
        .iter()
        .filter(|e| !e.roman.contains(' ') && e.kind == mandi_services::core::translit::Kind::Name)
        .take(3_000)
        .map(|e| (e.roman.as_str(), e.urdu.as_str()))
        .collect();
    {
        let conn = raw(&dir);
        conn.execute_batch("BEGIN").unwrap();
        let mut statement = conn
            .prepare(
                "INSERT INTO cities (code, name_ur, name_en, search_ur, search_en, search_phonetic) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            )
            .unwrap();
        for n in 0..RECORDS {
            let (a, b) = (words[n % words.len()], words[(n * 7 + 13) % words.len()]);
            let en = format!("{} {}", capital(a.0), capital(b.0));
            let ur = format!("{} {}", a.1, b.1);
            statement
                .execute((
                    (n + 1).to_string(),
                    &ur,
                    &en,
                    text::search_ur(&ur),
                    text::search_en(&en),
                    text::search_phonetic(&en),
                ))
                .unwrap();
        }
        drop(statement);
        conn.execute_batch("COMMIT").unwrap();
    }
    let started = Instant::now();
    search::rebuild(&env.ctx).unwrap();
    let build = started.elapsed();
    // The first and a middle record, typed with and without a mistake, in both scripts.
    let probe = |text: &str| {
        let t = Instant::now();
        let hits = search::search(&env.ctx, &owner, SearchScope::City, text, 20).unwrap();
        (hits, t.elapsed())
    };
    let (first_en, first_ur) = {
        let hits = masters::list(&env.ctx, MasterKind::City, false).unwrap();
        match &hits[10_000] {
            mandi_services::core::masters::MasterRow::City(c) => {
                (c.name_en.clone(), c.name_ur.clone())
            }
            _ => panic!("not a city"),
        }
    };
    let en_word = first_en.split(' ').next().unwrap().to_lowercase();
    let mistaken = {
        // Drop one middle letter: one mistake.
        let mut chars: Vec<char> = en_word.chars().collect();
        if chars.len() > 3 {
            chars.remove(2);
        }
        chars.into_iter().collect::<String>()
    };
    let ur_word = first_ur.split(' ').next().unwrap().to_owned();
    let mut slowest = 0;
    for typed in [
        en_word.as_str(),
        mistaken.as_str(),
        "zzzqqq",
        ur_word.as_str(),
        "ab",
        "1",
    ] {
        let (hits, took) = probe(typed);
        slowest = slowest.max(took.as_millis());
        eprintln!("search {typed:?}: {} hits in {took:?}", hits.len());
        if typed == en_word || typed == ur_word {
            assert!(!hits.is_empty(), "{typed}");
        }
        if typed == mistaken {
            assert!(
                !hits.is_empty(),
                "a mistake of one letter still finds something: {typed}"
            );
        }
    }
    eprintln!("index of {RECORDS} records built in {build:?}; slowest search {slowest} ms");
    assert!(
        slowest < BOUND_MS,
        "slowest search took {slowest} ms (bound {BOUND_MS} ms)"
    );
}

fn capital(word: &str) -> String {
    let mut chars = word.chars();
    chars
        .next()
        .map(|c| c.to_uppercase().chain(chars).collect())
        .unwrap_or_default()
}

#[test]
fn a_huge_search_text_is_refused_in_the_master_search_like_in_the_account_search() {
    // End-gate review 19: a paste of thousands of characters is never turned into keys.
    use mandi_services::core::error::{ErrorCode, ErrorValue};
    let (_dir, env, owner) = ready();
    add(&env, &owner, "", "لاہور", "Lahore");
    let at_limit = "a".repeat(search::MAX_QUERY_CHARS);
    search::search(&env.ctx, &owner, SearchScope::City, &at_limit, 10).unwrap();
    for text in ["a".repeat(search::MAX_QUERY_CHARS + 1), "ا".repeat(10_000)] {
        let ServiceError::Rejected(items) =
            search::search(&env.ctx, &owner, SearchScope::City, &text, 10).unwrap_err()
        else {
            panic!("not a rejection")
        };
        assert_eq!(items[0].code, ErrorCode::ErrTextTooLong);
        assert_eq!(items[0].field.as_deref(), Some("text"));
        assert_eq!(items[0].values.get("max"), Some(&ErrorValue::Number(200)));
    }
    // `search_recent` takes only the list, so it has no text to check.
    search::recent(&env.ctx, &owner, SearchScope::City).unwrap();
}

/// PRD 12.3, FR-16.7, E2E-38: the seller field of a lot offers the separate khatas (control 0305)
/// besides the sellers and اپنا مال. No other field offers a khata as a seller or a buyer, and a
/// short code still means a seller serial.
#[test]
fn a_separate_khata_is_offered_as_the_seller_of_a_lot_and_in_no_other_party_field() {
    use mandi_services::core::accounts::{
        AccountScope, ResolveCodeInput, ResolveStatus, SearchAccountsInput,
    };
    use mandi_services::core::error::{ErrorCode, ErrorValue};
    use mandi_services::core::khata::KhataCreateInput;
    let w = support::world();
    let find = |scope: AccountScope, text: &str| -> Vec<String> {
        search::search_accounts(
            &w.env.ctx,
            &w.owner,
            &SearchAccountsInput {
                include_dead: None,
                scope,
                control_id: None,
                text: text.into(),
                limit: 10,
                offset: 0,
            },
        )
        .unwrap()
        .into_iter()
        .map(|hit| hit.name_en)
        .collect()
    };
    // The index is built by this first search, so the khata below reaches it by a save.
    assert_eq!(find(AccountScope::LotSeller, "rahim"), vec!["Rahim"]);
    assert!(find(AccountScope::LotSeller, "family").is_empty());

    let mut words: Vec<String> = Vec::new();
    let khata = loop {
        match mandi_services::khata::create(
            &w.env.ctx,
            &w.owner,
            &KhataCreateInput {
                name_ur: "خاندانی کھاتہ".into(),
                name_en: "Family Khata".into(),
                note: None,
                confirmed_rules_words: words.clone(),
            },
        ) {
            Ok(done) => break done,
            Err(ServiceError::Rejected(items))
                if items
                    .iter()
                    .all(|i| i.code == ErrorCode::ErrSpellingUnconfirmed) =>
            {
                for item in items {
                    if let Some(ErrorValue::Text(t)) = item.values.get("words") {
                        words.extend(t.split(' ').map(str::to_owned));
                    }
                }
            }
            Err(other) => panic!("{other:?}"),
        }
    };
    assert!(
        khata.account.code.starts_with("0305"),
        "{}",
        khata.account.code
    );

    // The seller field of a lot finds it by either name, and still finds sellers and اپنا مال.
    assert_eq!(
        find(AccountScope::LotSeller, "family"),
        vec!["Family Khata"]
    );
    assert_eq!(
        find(AccountScope::LotSeller, "خاندانی"),
        vec!["Family Khata"]
    );
    assert_eq!(find(AccountScope::LotSeller, "rahim"), vec!["Rahim"]);
    assert!(
        !find(AccountScope::LotSeller, "own").is_empty(),
        "اپنا مال is offered in a lot"
    );
    // A seller field, a buyer field and the account lists of other screens do not offer it as a
    // seller or a buyer; a field that takes any account does.
    assert!(find(AccountScope::Seller, "family").is_empty());
    assert!(find(AccountScope::Buyer, "family").is_empty());
    assert_eq!(find(AccountScope::Any, "family"), vec!["Family Khata"]);
    assert!(
        find(AccountScope::Seller, "own").is_empty(),
        "اپنا مال only in a lot"
    );

    // The picker list (Alt+Down) of the lot field has it; the seller field's does not.
    let recent = |scope: AccountScope| -> Vec<String> {
        search::recent_accounts(
            &w.env.ctx,
            &w.owner,
            &mandi_services::core::accounts::AccountsRecentInput {
                include_dead: None,
                scope,
                control_id: None,
            },
        )
        .unwrap()
        .into_iter()
        .map(|hit| hit.name_en)
        .collect()
    };
    assert!(recent(AccountScope::LotSeller).contains(&"Family Khata".to_owned()));
    assert!(!recent(AccountScope::Seller).contains(&"Family Khata".to_owned()));

    // A short code still means a seller's running number: `0` in a lot field is Rahim (0010), never the khata.
    let resolved = search::resolve_code(
        &w.env.ctx,
        &ResolveCodeInput {
            scope: AccountScope::LotSeller,
            control_id: None,
            text: "0".into(),
        },
    )
    .unwrap();
    assert_eq!(resolved.status, ResolveStatus::Found);
    assert_eq!(resolved.account.unwrap().name_en, "Rahim");
    // Its full code works in the lot field (the code is taken as typed).
    let by_code = search::resolve_code(
        &w.env.ctx,
        &ResolveCodeInput {
            scope: AccountScope::LotSeller,
            control_id: None,
            text: khata.account.code.clone(),
        },
    )
    .unwrap();
    assert_eq!(by_code.status, ResolveStatus::Found);
    assert_eq!(by_code.account.unwrap().id, khata.account.id);
}

#[test]
fn the_bank_account_is_offered_in_no_picker_and_no_search() {
    // PRD FR-10.4 (v1.6): all money is روکڑ. بینک 01020001 stays in the chart, but no picker, code,
    // short code or global search offers it, so nothing can be posted to it by mistake.
    use mandi_services::core::accounts::SearchGlobalInput;
    use mandi_services::core::accounts::{AccountScope, GlobalHit, SearchAccountsInput};
    let w = support::world();
    let find = |scope: AccountScope, text: &str| -> Vec<String> {
        search::search_accounts(
            &w.env.ctx,
            &w.owner,
            &SearchAccountsInput {
                include_dead: None,
                scope,
                control_id: None,
                text: text.into(),
                limit: 10,
                offset: 0,
            },
        )
        .unwrap()
        .into_iter()
        .map(|hit| hit.code)
        .collect()
    };
    // The cash account is still found: the exclusion is the bank's alone.
    assert_eq!(find(AccountScope::Any, "01010001"), vec!["01010001"]);
    for scope in [
        AccountScope::Any,
        AccountScope::Seller,
        AccountScope::Buyer,
        AccountScope::LotSeller,
    ] {
        for text in ["01020001", "bank", "بینک", "1"] {
            assert!(
                !find(scope, text).iter().any(|code| code == "01020001"),
                "{text} in {scope:?}"
            );
        }
    }
    let hits = search::global(
        &w.env.ctx,
        &w.owner,
        &SearchGlobalInput {
            text: "bank".into(),
            limit: 20,
        },
    )
    .unwrap();
    assert!(
        hits.iter()
            .all(|hit| !matches!(hit, GlobalHit::Account { hit } if hit.code == "01020001"))
    );
}
