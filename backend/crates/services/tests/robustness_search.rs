//! Search and short codes under attack (D2 attack list item 3; PRD FR-7.38 to FR-7.42, FR-7.35,
//! FR-10.3, Code Standards V0, B27, B29). Owner: `robustness`.
//!
//! - A short code is taken only by `resolve_code` (on `Enter` or leaving the field), and every odd
//!   code gets a clean answer: `1` is the running number 1 and not 11, `0` is the first account,
//!   a number with a leading zero is neither a short code nor a code (plan 32), digits that are
//!   not ASCII digits are a name and not a code, huge text is refused.
//! - Hostile text in a picker (SQL, wildcards, control characters, zero-width marks, emoji, huge
//!   text) never ends in an internal error, never matches everything, is quick, and writes
//!   nothing: search is read-only.
//! - The index follows every committed change (new, renamed, switched off) and never shows an
//!   account that is switched off in a field for new postings.
#![cfg(test)]

mod common;

use std::time::{Duration, Instant};

use common::*;
use mandi_services::core::accounts::{
    AccountInput, AccountScope, AccountType, ResolveCodeInput, ResolveStatus, SearchAccountsInput,
    SearchGlobalInput,
};
use mandi_services::core::error::ErrorCode;
use mandi_services::core::search::SearchScope;
use mandi_services::testing::TestEnv;
use mandi_services::{ServiceError, accounts, auth, search};
use tempfile::TempDir;

const BUYERS: usize = 12;
const SELLERS: usize = 3;
/// A quick answer in a debug build with a small data file. Release is far under this.
const QUICK: Duration = Duration::from_millis(300);

fn form(kind: AccountType, ur: &str, en: &str) -> AccountInput {
    AccountInput {
        id: None,
        version: None,
        account_type: kind,
        control_id: None,
        code: String::new(),
        name_ur: ur.into(),
        name_en: en.into(),
        confirmed_rules_words: Vec::new(),
        father_or_firm_ur: String::new(),
        father_or_firm_en: String::new(),
        city_id: None,
        adda_id: None,
        phone1: String::new(),
        phone2: String::new(),
        address: String::new(),
        linked_account_id: None,
        opening_date: String::new(),
        opening_dr: String::new(),
        opening_cr: String::new(),
        opening_is_advance: false,
        credit_allowed: true,
        credit_limit: String::new(),
        default_marka_id: None,
        collector_staff_id: None,
        collection_weekday: None,
        is_active: true,
        note: String::new(),
        tag: String::new(),
    }
}

struct World {
    dir: TempDir,
    env: TestEnv,
    owner: auth::Actor,
    /// Ids of the buyers 0020 to 00211, in order.
    buyers: Vec<i64>,
    /// Ids of the sellers 0010 to 0012, in order.
    sellers: Vec<i64>,
}

fn world() -> World {
    let (dir, env) = env();
    set_up(&env);
    login_owner(&env);
    let owner = auth::require(&env.ctx).unwrap();
    let ur = ["اکرم", "ارشد", "شاہد", "ناصر"];
    let en = ["Akram", "Arshad", "Shahid", "Nasir"];
    let mut buyers = Vec::new();
    for n in 0..BUYERS {
        let row = accounts::save(
            &env.ctx,
            &owner,
            &form(AccountType::Buyer, ur[n % 4], en[n % 4]),
        )
        .unwrap();
        assert_eq!(row.code, format!("002{n}"));
        buyers.push(row.id);
    }
    let mut sellers = Vec::new();
    for n in 0..SELLERS {
        let row = accounts::save(
            &env.ctx,
            &owner,
            &form(AccountType::Seller, ur[n % 4], en[n % 4]),
        )
        .unwrap();
        assert_eq!(row.code, format!("001{n}"));
        sellers.push(row.id);
    }
    World {
        dir,
        env,
        owner,
        buyers,
        sellers,
    }
}

/// Row counts of every table: "search writes nothing" in one number per table.
fn table_counts(dir: &TempDir) -> Vec<(String, i64)> {
    let conn = raw(dir);
    let names: Vec<String> = {
        let mut st = conn
            .prepare("SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%' ORDER BY name")
            .unwrap();
        st.query_map([], |r| r.get(0))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap()
    };
    names
        .into_iter()
        .map(|n| {
            let c: i64 = conn
                .query_row(&format!("SELECT COUNT(*) FROM \"{n}\""), [], |r| r.get(0))
                .unwrap();
            (n, c)
        })
        .collect()
}

fn resolve(
    w: &World,
    scope: AccountScope,
    text: &str,
) -> Result<mandi_services::core::accounts::ResolvedCode, ServiceError> {
    search::resolve_code(
        &w.env.ctx,
        &ResolveCodeInput {
            scope,
            control_id: None,
            text: text.to_owned(),
        },
    )
}

fn find(
    w: &World,
    scope: AccountScope,
    text: &str,
) -> Result<Vec<mandi_services::core::accounts::AccountHit>, ServiceError> {
    search::search_accounts(
        &w.env.ctx,
        &w.owner,
        &SearchAccountsInput {
            scope,
            control_id: None,
            text: text.to_owned(),
            limit: 20,
            offset: 0,
        },
    )
}

#[test]
fn a_short_code_is_the_running_number_and_one_is_not_eleven() {
    let w = world();
    let before = table_counts(&w.dir);
    // Buyer field: the running number alone, no padding.
    for (typed, code) in [
        ("0", "0020"),
        ("1", "0021"),
        ("10", "00210"),
        ("11", "00211"),
        ("9", "0029"),
        ("  3  ", "0023"),
    ] {
        let found = resolve(&w, AccountScope::Buyer, typed).unwrap();
        assert_eq!(found.status, ResolveStatus::Found, "typed '{typed}'");
        assert_eq!(found.account.unwrap().code, code, "typed '{typed}'");
    }
    // Seller field: the same number is a different account.
    let seller = resolve(&w, AccountScope::Seller, "0").unwrap();
    assert_eq!(seller.status, ResolveStatus::Found);
    assert_eq!(seller.account.unwrap().code, "0010");
    // A field that takes any account: running number 0 is both a seller and a buyer.
    let any = resolve(&w, AccountScope::Any, "0").unwrap();
    assert_eq!(any.status, ResolveStatus::Several);
    let mut codes: Vec<String> = any.candidates.iter().map(|c| c.code.clone()).collect();
    codes.sort();
    assert_eq!(codes, ["0010", "0020"]);
    // Running number 11 exists only as a buyer (3 sellers), so the any field finds it.
    assert_eq!(
        resolve(&w, AccountScope::Any, "11").unwrap().status,
        ResolveStatus::Found
    );
    // A number nobody has yet: unknown, with the code a new account would get.
    let next = resolve(&w, AccountScope::Buyer, "12").unwrap();
    assert_eq!(next.status, ResolveStatus::Unknown);
    assert_eq!(next.suggested_code.as_deref(), Some("00212"));
    // The full code works, in its own series only.
    let full = resolve(&w, AccountScope::Buyer, "0021").unwrap();
    assert_eq!(full.status, ResolveStatus::Found);
    assert_eq!(full.account.unwrap().code, "0021");
    let wrong_series = resolve(&w, AccountScope::Seller, "0021").unwrap();
    assert_ne!(
        wrong_series.status,
        ResolveStatus::Found,
        "a buyer code in a seller field"
    );
    // Taking a code writes nothing.
    assert_eq!(table_counts(&w.dir), before);
}

#[test]
fn leading_zeros_and_odd_digits_get_a_clean_answer() {
    let w = world();
    let before = table_counts(&w.dir);
    // A number with a leading zero is neither a short code nor a full code: no account, no offer.
    for typed in ["00", "000", "01", "0000", "0001", "007", "012", "8999999a"] {
        let found = resolve(&w, AccountScope::Buyer, typed).unwrap();
        assert_ne!(found.status, ResolveStatus::Found, "typed '{typed}'");
        assert_eq!(found.status, ResolveStatus::Unknown, "typed '{typed}'");
        assert!(
            found.account.is_none() && found.suggested_code.is_none(),
            "typed '{typed}' brought an account or an offer: {found:?}"
        );
    }
    // No running number is reserved: a big one nobody has is unknown, and a new account would get
    // that code. More than 7 digits is no number.
    for typed in ["9000", "9001", "9999", "8999999"] {
        let found = resolve(&w, AccountScope::Buyer, typed).unwrap();
        assert_eq!(found.status, ResolveStatus::Unknown, "typed '{typed}'");
        assert!(found.account.is_none(), "typed '{typed}'");
        assert_eq!(
            found.suggested_code,
            Some(format!("002{typed}")),
            "typed '{typed}'"
        );
    }
    let too_long = resolve(&w, AccountScope::Buyer, "89999999").unwrap();
    assert_eq!(too_long.status, ResolveStatus::Unknown);
    assert!(too_long.account.is_none() && too_long.suggested_code.is_none());
    // The system accounts are found by their full code, which is how they are reached.
    for (scope, code) in [
        (AccountScope::Buyer, "03040000"),
        (AccountScope::Buyer, "0030"),
        (AccountScope::Seller, "0031"),
    ] {
        let found = resolve(&w, scope, code).unwrap();
        assert_eq!(found.status, ResolveStatus::Found, "{code}");
    }
    // The pukhta accounts have no short code: `0` and `1` are the first buyers, not 0030.
    assert_eq!(
        resolve(&w, AccountScope::Buyer, "0")
            .unwrap()
            .account
            .unwrap()
            .code,
        "0020"
    );
    // Not ASCII digits means a name, not a code: nothing found, nothing offered.
    for typed in [
        "1a",
        "a1",
        "١٢",   // Arabic-Indic digits
        "۱۲",   // Urdu digits
        "１２", // full-width digits
        "1 2",
        "1,2",
        "-1",
        "+1",
        "1.0",
        "1e1",
        "0x1",
        "٠",
        "one",
        "'; DROP TABLE accounts; --",
    ] {
        let found = resolve(&w, AccountScope::Buyer, typed).unwrap();
        assert_eq!(found.status, ResolveStatus::Unknown, "typed '{typed}'");
        assert!(
            found.account.is_none() && found.suggested_code.is_none(),
            "typed '{typed}'"
        );
    }
    // Empty and blank are a missing field, not an answer.
    for typed in ["", " ", "\t", "\u{200b}"] {
        let result = resolve(&w, AccountScope::Buyer, typed);
        match result {
            Err(error) => assert!(
                error.has_code(ErrorCode::ErrRequired),
                "'{typed}': {error:?}"
            ),
            Ok(found) => assert_eq!(found.status, ResolveStatus::Unknown, "'{typed}'"),
        }
    }
    // Too long: refused with the cap, not read to the end.
    let started = Instant::now();
    for text in ["9".repeat(5_000), "1".repeat(1_000_000)] {
        let error = resolve(&w, AccountScope::Buyer, &text).unwrap_err();
        assert!(error.has_code(ErrorCode::ErrTextTooLong), "{error:?}");
    }
    assert!(
        started.elapsed() < QUICK,
        "a huge text took {:?}",
        started.elapsed()
    );
    // A control field needs its control.
    let error = resolve(&w, AccountScope::Control, "1").unwrap_err();
    assert!(error.has_code(ErrorCode::ErrRequired), "{error:?}");
    assert_eq!(table_counts(&w.dir), before);
}

#[test]
fn hostile_text_in_a_picker_is_only_text_and_search_writes_nothing() {
    let w = world();
    let before = table_counts(&w.dir);
    let long_ok = "a".repeat(200);
    let hostile: Vec<(&str, String)> = vec![
        ("sql", "' OR 1=1 --".into()),
        ("sql drop", "'; DROP TABLE accounts; --".into()),
        ("percent", "%".into()),
        ("underscore", "_".into()),
        ("percent word", "%kram%".into()),
        ("backslash", "\\".into()),
        ("path", "..\\..\\windows".into()),
        ("script", "<script>alert(1)</script>".into()),
        ("null byte", "ak\0ram".into()),
        ("control", "ak\u{1}\u{7}ram".into()),
        ("tab and newline", "ak\t\nram".into()),
        ("zero width", "ak\u{200b}ra\u{200d}m".into()),
        ("direction marks", "\u{202e}akram\u{202c}".into()),
        ("emoji", "😀😀😀".into()),
        ("combining storm", format!("a{}", "\u{301}".repeat(150))),
        ("arabic look alike", "اكرم".into()),
        ("mixed scripts", "akramاکرم".into()),
        ("regex", ".*".into()),
        ("single quote", "'".into()),
        ("long but allowed", long_ok),
        ("spaces", "          ".into()),
        ("empty", String::new()),
    ];
    let total = i64::try_from(BUYERS + SELLERS).unwrap();
    for scope in [AccountScope::Seller, AccountScope::Buyer, AccountScope::Any] {
        for (label, text) in &hostile {
            let started = Instant::now();
            let result = find(&w, scope, text);
            let took = started.elapsed();
            assert!(took < QUICK, "{label} in {scope:?} took {took:?}");
            match result {
                Ok(hits) => {
                    // A wildcard or a quote must never turn into "everything".
                    if matches!(
                        *label,
                        "percent"
                            | "underscore"
                            | "backslash"
                            | "sql"
                            | "single quote"
                            | "regex"
                            | "script"
                            | "path"
                            | "sql drop"
                    ) {
                        assert!(
                            i64::try_from(hits.len()).unwrap() < total,
                            "{label} in {scope:?} matched every account"
                        );
                    }
                    for hit in &hits {
                        assert!(
                            hit.is_active,
                            "{label}: an account that is switched off is offered"
                        );
                    }
                }
                Err(error) => assert!(
                    error.has_code(ErrorCode::ErrTextTooLong),
                    "{label} in {scope:?}: {error:?}"
                ),
            }
        }
    }
    // The global search takes the same text.
    for (label, text) in &hostile {
        let result = search::global(
            &w.env.ctx,
            &w.owner,
            &SearchGlobalInput {
                text: text.clone(),
                limit: 20,
            },
        );
        if let Err(error) = result {
            assert!(
                error.has_code(ErrorCode::ErrTextTooLong),
                "global {label}: {error:?}"
            );
        }
    }
    // Too long: refused by every account search.
    let huge = "ا".repeat(100_000);
    for scope in [AccountScope::Seller, AccountScope::Buyer, AccountScope::Any] {
        let error = find(&w, scope, &huge).unwrap_err();
        assert!(
            error.has_code(ErrorCode::ErrTextTooLong),
            "{scope:?}: {error:?}"
        );
    }
    // Nothing was written, and all accounts are still there.
    assert_eq!(table_counts(&w.dir), before);
    let normal: i64 = raw(&w.dir)
        .query_row(
            "SELECT COUNT(*) FROM accounts WHERE is_system = 0",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(normal, total);
}

#[test]
fn the_picker_lists_for_masters_refuse_the_same_huge_text_as_the_account_pickers() {
    // Code Standards V0: every text from a screen is untrusted, whatever picker it comes from.
    let w = world();
    let huge = "x".repeat(100_000);
    for scope in [
        SearchScope::City,
        SearchScope::Item,
        SearchScope::Packing,
        SearchScope::Adda,
        SearchScope::Staff,
        SearchScope::Seller,
        SearchScope::Buyer,
    ] {
        let started = Instant::now();
        let result = search::search(&w.env.ctx, &w.owner, scope, &huge, 20);
        let took = started.elapsed();
        match result {
            Err(error) => assert!(
                error.has_code(ErrorCode::ErrTextTooLong),
                "{scope:?}: {error:?}"
            ),
            Ok(_) => panic!(
                "{scope:?}: a text of 100,000 characters was searched ({took:?}) instead of refused \
                 with ERR_TEXT_TOO_LONG, as the account picker does"
            ),
        }
    }
}

#[test]
fn the_index_follows_every_committed_change() {
    let w = world();
    let names = |scope: AccountScope, text: &str| -> Vec<String> {
        find(&w, scope, text)
            .unwrap()
            .into_iter()
            .map(|h| h.code)
            .collect()
    };
    // New: found at once.
    let tariq = accounts::save(
        &w.env.ctx,
        &w.owner,
        &form(AccountType::Buyer, "طارق", "Tariq"),
    )
    .unwrap();
    assert!(names(AccountScope::Buyer, "tariq").contains(&tariq.code));
    assert!(names(AccountScope::Buyer, "طارق").contains(&tariq.code));
    assert_eq!(tariq.code, "00212");
    assert_eq!(
        resolve(&w, AccountScope::Buyer, "12").unwrap().status,
        ResolveStatus::Found
    );
    // Switched off: not offered for new postings, and a code says so.
    accounts::set_active(&w.env.ctx, &w.owner, tariq.id, false).unwrap();
    assert!(!names(AccountScope::Buyer, "tariq").contains(&tariq.code));
    assert!(!names(AccountScope::Any, "12").contains(&tariq.code));
    let off = resolve(&w, AccountScope::Buyer, "12").unwrap();
    assert_eq!(off.status, ResolveStatus::Inactive);
    // Switched on again: back.
    accounts::set_active(&w.env.ctx, &w.owner, tariq.id, true).unwrap();
    assert!(names(AccountScope::Buyer, "tariq").contains(&tariq.code));
    // A search that is only a spelling mistake still finds him (FR-7.40).
    assert!(names(AccountScope::Buyer, "tarq").contains(&tariq.code));
    // The balances on the party list always come from the books, not from the index.
    let balances = search::balances(&w.env.ctx, &[tariq.id, w.buyers[0]]).unwrap();
    assert_eq!(balances.len(), 2);
    // A rebuild from the database gives the same answers (B27).
    let before = names(AccountScope::Any, "a");
    search::rebuild(&w.env.ctx).unwrap();
    assert_eq!(names(AccountScope::Any, "a"), before);
    let _ = (&w.sellers, &w.buyers);
}
