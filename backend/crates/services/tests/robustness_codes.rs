//! Codes, names and duplicates under attack (D2 attack list items 1 and 5; PRD FR-10.2, FR-10.3,
//! FR-10.7, FR-7.28, FR-7.63, Code Standards 4.2, E29). Owner: `robustness`.
//!
//! - Accounts made at the same moment never get the same code, and the serials come out as
//!   1, 2, 3 ... with no gap.
//! - Serial 0000 and 9000 to 9999 are refused for normal accounts, whatever way the code is
//!   typed. The series stops at 8999 and no code is ever reused.
//! - Hostile names (mixed scripts, zero-width marks, look-alike letters, huge, emoji, control and
//!   direction marks, SQL, paths, scripts): each is stored as plain text, cleaned, or refused with
//!   nothing written. Nothing is ever run.
//! - Names that differ only in look-alike letters, diacritics, spaces or zero-width marks are
//!   still caught by the duplicate warning.
#![cfg(test)]

mod common;

use std::collections::{BTreeSet, HashSet};
use std::sync::{Arc, Barrier};

use common::*;
use mandi_services::core::accounts::{
    AccountInput, AccountScope, AccountType, AccountsSimilarInput, SearchAccountsInput,
};
use mandi_services::core::error::ErrorCode;
use mandi_services::testing::TestEnv;
use mandi_services::{ServiceError, accounts, auth, search};
use tempfile::TempDir;

fn ready() -> (TempDir, TestEnv, auth::Actor) {
    let (dir, env) = env();
    set_up(&env);
    login_owner(&env);
    let owner = auth::require(&env.ctx).unwrap();
    (dir, env, owner)
}

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

fn count(dir: &TempDir, sql: &str) -> i64 {
    raw(dir).query_row(sql, [], |r| r.get(0)).unwrap()
}

fn rejection_codes(error: &ServiceError) -> Vec<ErrorCode> {
    match error {
        ServiceError::Rejected(items) => items.iter().map(|i| i.code).collect(),
        other => panic!("not a rejection: {other:?}"),
    }
}

#[test]
fn accounts_made_at_the_same_moment_never_share_a_code() {
    let (dir, env, owner) = ready();
    let threads = 8;
    let each = 25;
    let barrier = Arc::new(Barrier::new(threads));
    let handles: Vec<_> = (0..threads)
        .map(|t| {
            let ctx = env.ctx.clone();
            let owner = owner.clone();
            let barrier = barrier.clone();
            std::thread::spawn(move || {
                barrier.wait();
                let mut codes = Vec::new();
                for n in 0..each {
                    // Buyers and sellers at the same time, all different names.
                    let kind = if (t + n) % 2 == 0 {
                        AccountType::Buyer
                    } else {
                        AccountType::Seller
                    };
                    let (ur, en) = (
                        ["اکرم", "ارشد", "شاہد", "ناصر"][(t + n) % 4],
                        ["Akram", "Arshad", "Shahid", "Nasir"][(t + n) % 4],
                    );
                    match accounts::save(&ctx, &owner, &form(kind, ur, en)) {
                        Ok(row) => codes.push(row.code),
                        Err(error) => panic!("thread {t} save {n}: {error:?}"),
                    }
                }
                codes
            })
        })
        .collect();
    let mut all: Vec<String> = Vec::new();
    for handle in handles {
        all.extend(handle.join().unwrap());
    }
    let unique: HashSet<&String> = all.iter().collect();
    assert_eq!(unique.len(), all.len(), "two accounts got the same code");
    assert_eq!(all.len(), threads * each);
    // Serials run 1, 2, 3 ... in each series with no gap and nothing reused.
    for (prefix, len) in [("0304", 8), ("03011", 9)] {
        let serials: BTreeSet<u32> = all
            .iter()
            .filter(|c| c.len() == len && c.starts_with(prefix))
            .map(|c| c[prefix.len()..].parse().unwrap())
            .collect();
        let want: BTreeSet<u32> = (1..=u32::try_from(serials.len()).unwrap()).collect();
        assert_eq!(serials, want, "serials of {prefix} have a gap or a repeat");
    }
    assert_eq!(
        count(
            &dir,
            "SELECT COUNT(*) FROM (SELECT code FROM accounts GROUP BY code HAVING COUNT(*) > 1)"
        ),
        0
    );
    // The audit row of each creation is there, and the chain is intact.
    assert_eq!(
        count(
            &dir,
            "SELECT COUNT(*) FROM audit_log WHERE action = 'create' AND entity = 'account'"
        ),
        i64::try_from(threads * each).unwrap()
    );
    assert_chain_ok(&dir);
}

#[test]
fn a_typed_code_in_the_reserved_ranges_is_refused_every_way_it_is_written() {
    let (dir, env, owner) = ready();
    accounts::save(&env.ctx, &owner, &form(AccountType::Buyer, "اکرم", "Akram")).unwrap();
    let before = count(&dir, "SELECT COUNT(*) FROM accounts");
    let audit_before = audit_total(&dir);
    let bad_buyer_codes = [
        "03040000", // serial 0000: reserved (نقدی منڈی)
        "03049000",
        "03049001",
        "03049500",
        "03049998",
        "03049999", // 9000 to 9999 reserved
        "03040001", // taken
        "0304",
        "030400",
        "03040",
        "0304000", // too short to be a code
        "030400001",
        "0304000001", // too long
        "0304000a",
        "0304 0002",
        "0304-0002",
        "03040002 ", // not digits only (a trailing space is trimmed: see below)
        "03050001",
        "03011000",
        "01010099",
        "3040002",          // another series
        "٠٣٠٤٠٠٠٢",         // Arabic-Indic digits
        "０３０４０００２", // full-width digits
        "03040002\u{200B}", // a zero-width mark
        "-03040002",
        "+03040002",
        "03040002.0",
        "0x3040002",
        "3.04e7",
    ];
    for code in bad_buyer_codes {
        let mut input = form(AccountType::Buyer, "ارشد", "Arshad");
        input.code = code.to_owned();
        let fingerprint_rows = count(&dir, "SELECT COUNT(*) FROM accounts");
        let result = accounts::save(&env.ctx, &owner, &input);
        match result {
            Ok(row) => {
                // The only forgiven forms: a typed code that means the next free serial after
                // trimming. It must then be a normal serial, never a reserved one.
                let serial: u32 = row.code[4..].parse().unwrap();
                assert!(
                    (1..=8999).contains(&serial),
                    "typed '{code}' gave the reserved serial {serial} ({})",
                    row.code
                );
                assert!(
                    code.trim() == row.code || code.chars().all(|c| c.is_ascii_digit() || c == ' '),
                    "typed '{code}' was taken as {}",
                    row.code
                );
            }
            Err(error) => {
                assert!(
                    matches!(error, ServiceError::Rejected(_)),
                    "typed '{code}' was refused with an internal error: {error:?}"
                );
                assert_eq!(
                    count(&dir, "SELECT COUNT(*) FROM accounts"),
                    fingerprint_rows,
                    "typed '{code}' was refused but left a row"
                );
            }
        }
    }
    // The reserved serials of the series: a code with serial 0000 or 9000 and up was never
    // created for a user account, whatever was typed.
    assert_eq!(
        count(
            &dir,
            "SELECT COUNT(*) FROM accounts WHERE is_system = 0 AND (serial = 0 OR serial >= 9000)"
        ),
        0
    );
    // Sellers: the same.
    for code in [
        "030110000",
        "030119000",
        "030119001",
        "030119999",
        "030119500",
    ] {
        let mut input = form(AccountType::Seller, "رحیم", "Rahim");
        input.code = code.to_owned();
        let error = accounts::save(&env.ctx, &owner, &input).unwrap_err();
        let codes = rejection_codes(&error);
        assert!(
            codes
                .iter()
                .any(|c| matches!(c, ErrorCode::ErrCodeReserved | ErrorCode::ErrCodeTaken)),
            "seller code {code}: {codes:?}"
        );
    }
    // The next free code is still the right one.
    let next = accounts::next_code(&env.ctx, AccountType::Buyer, None).unwrap();
    let highest: String = raw(&dir)
        .query_row(
            "SELECT MAX(code) FROM accounts WHERE code LIKE '0304____' AND serial BETWEEN 1 AND 8999",
            [],
            |r| r.get(0),
        )
        .unwrap();
    let want = format!("0304{:04}", highest[4..].parse::<u32>().unwrap() + 1);
    assert_eq!(next, want);
    assert!(count(&dir, "SELECT COUNT(*) FROM accounts") >= before);
    assert!(audit_total(&dir) >= audit_before);
    assert_chain_ok(&dir);
}

#[test]
fn the_series_stops_at_8999_and_no_code_is_reused() {
    let (dir, env, owner) = ready();
    let conn = raw(&dir);
    let control: i64 = conn
        .query_row(
            "SELECT id FROM account_controls WHERE code = '0304'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    conn.execute_batch("BEGIN").unwrap();
    {
        let mut insert = conn
            .prepare(
                "INSERT INTO accounts (control_id, code, serial, party_type, name_ur, name_en, search_ur, search_en, search_phonetic)
                 VALUES (?1, ?2, ?3, 'buyer', 'گاہک', 'Buyer', 'گاہک', 'buyer', 'byr')",
            )
            .unwrap();
        for serial in 1..=8_998_u32 {
            insert
                .execute(mandi_services::db::rusqlite::params![
                    control,
                    format!("0304{serial:04}"),
                    serial
                ])
                .unwrap();
        }
    }
    conn.execute_batch("COMMIT").unwrap();
    drop(conn);
    // Serial 8999 is the last normal one.
    assert_eq!(
        accounts::next_code(&env.ctx, AccountType::Buyer, None).unwrap(),
        "03048999"
    );
    let last =
        accounts::save(&env.ctx, &owner, &form(AccountType::Buyer, "اکرم", "Akram")).unwrap();
    assert_eq!(last.code, "03048999");
    // Now the series is full: a clear message, nothing written, and the reserved 9000 is never given.
    let rows = count(&dir, "SELECT COUNT(*) FROM accounts");
    let audit = audit_total(&dir);
    for _ in 0..3 {
        let error = accounts::save(
            &env.ctx,
            &owner,
            &form(AccountType::Buyer, "ارشد", "Arshad"),
        )
        .unwrap_err();
        assert_eq!(rejection_codes(&error), vec![ErrorCode::ErrSeriesFull]);
        assert_eq!(
            accounts::next_code(&env.ctx, AccountType::Buyer, None)
                .map_err(|e| rejection_codes(&e)),
            Err(vec![ErrorCode::ErrSeriesFull])
        );
    }
    assert_eq!(count(&dir, "SELECT COUNT(*) FROM accounts"), rows);
    assert_eq!(audit_total(&dir), audit);
    // Typing a free serial inside the range does not help once the code is taken, and the
    // gap-free rule means there is no free normal serial left.
    let mut typed = form(AccountType::Buyer, "ارشد", "Arshad");
    typed.code = "03048999".into();
    assert_eq!(
        rejection_codes(&accounts::save(&env.ctx, &owner, &typed).unwrap_err()),
        vec![ErrorCode::ErrCodeTaken]
    );
    assert_eq!(
        count(
            &dir,
            "SELECT COUNT(*) FROM accounts WHERE code LIKE '03049%' AND is_system = 0"
        ),
        0
    );
}

/// What a hostile name must do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Want {
    /// Stored as given (plain text).
    Kept,
    /// Stored, with these invisible marks removed.
    Cleaned,
    /// Refused with this code on a name field.
    Refused(ErrorCode),
    /// Refused with any rejection.
    RefusedAny,
    /// Either stored or refused, but if stored it holds no control, format or invisible character.
    EitherClean,
    /// Stored as given, or sent back to confirm a spelling made by the rules only (PRD FR-7.66).
    KeptOrSpelling,
}

fn invisible(c: char) -> bool {
    c.is_control()
        || matches!(c as u32,
            0x200B..=0x200F | 0x202A..=0x202E | 0x2060..=0x2064 | 0x2066..=0x206F | 0xFEFF | 0x061C | 0x00AD | 0x180E)
}

#[test]
fn hostile_names_are_kept_as_plain_text_cleaned_or_refused_and_never_run() {
    let (dir, env, owner) = ready();
    let long_ur_99 = "اکرم ".repeat(20).trim_end().to_owned();
    let long_en_95 = "Akram ".repeat(16).trim_end().to_owned();
    let too_long_ur = "اکرم ".repeat(21).trim_end().to_owned();
    let too_long_en = "Akram ".repeat(17).trim_end().to_owned();
    let huge_ur = "ا".repeat(1_000_000);
    let huge_en = "A".repeat(1_000_000);
    let cases: Vec<(&str, String, String, Want)> = vec![
        ("plain", "اکرم".into(), "Akram".into(), Want::Kept),
        (
            "father-like punctuation",
            "اکرم (ملتانی) - 2/1 & سنز.".into(),
            "Akram (Multani) - 2/1 & Sons.".into(),
            Want::Kept,
        ),
        (
            "apostrophe",
            "اکرم".into(),
            "O'Brien".into(),
            Want::KeptOrSpelling,
        ),
        (
            "digits in the name",
            "اکرم 786".into(),
            "Akram 786".into(),
            Want::Kept,
        ),
        (
            "99 and 95 characters",
            long_ur_99.clone(),
            long_en_95.clone(),
            Want::Kept,
        ),
        (
            "mixed script in the Urdu name",
            "اکرم Akram".into(),
            "Akram".into(),
            Want::Refused(ErrorCode::ErrNameUrScript),
        ),
        (
            "mixed script in the English name",
            "اکرم".into(),
            "Akram اکرم".into(),
            Want::Refused(ErrorCode::ErrNameEnScript),
        ),
        (
            "English in the Urdu field",
            "Akram".into(),
            "Akram".into(),
            Want::Refused(ErrorCode::ErrNameUrScript),
        ),
        (
            "Urdu in the English field",
            "اکرم".into(),
            "اکرم".into(),
            Want::Refused(ErrorCode::ErrNameEnScript),
        ),
        ("only spaces", "   ".into(), "   ".into(), Want::RefusedAny),
        ("empty", String::new(), String::new(), Want::RefusedAny),
        ("only digits", "123".into(), "456".into(), Want::RefusedAny),
        (
            "only punctuation",
            "...".into(),
            "...".into(),
            Want::RefusedAny,
        ),
        (
            "zero width space in Urdu",
            "اک\u{200B}رم".into(),
            "Akram".into(),
            Want::Cleaned,
        ),
        (
            "zero width joiner and non-joiner",
            "اک\u{200C}ر\u{200D}م".into(),
            "Akram".into(),
            Want::Cleaned,
        ),
        (
            "byte order mark and word joiner",
            "\u{FEFF}اک\u{2060}رم".into(),
            "Akram".into(),
            Want::Cleaned,
        ),
        (
            "zero width in English",
            "اکرم".into(),
            "Ak\u{200B}ra\u{200D}m\u{FEFF}".into(),
            Want::Cleaned,
        ),
        (
            "left to right mark",
            "اکرم\u{200E}".into(),
            "Akram\u{200E}".into(),
            Want::EitherClean,
        ),
        (
            "right to left mark",
            "\u{200F}اکرم".into(),
            "Akram".into(),
            Want::EitherClean,
        ),
        (
            "arabic letter mark",
            "اکرم\u{061C}".into(),
            "Akram".into(),
            Want::EitherClean,
        ),
        (
            "right to left override",
            "اکرم\u{202E}".into(),
            "Akram\u{202E}".into(),
            Want::EitherClean,
        ),
        (
            "isolates",
            "\u{2067}اکرم\u{2069}".into(),
            "\u{2066}Akram\u{2069}".into(),
            Want::EitherClean,
        ),
        (
            "soft hyphen",
            "اکرم".into(),
            "Ak\u{00AD}ram".into(),
            Want::EitherClean,
        ),
        (
            "NUL in the middle",
            "اک\0رم".into(),
            "Ak\0ram".into(),
            Want::RefusedAny,
        ),
        (
            "tab",
            "اکرم\t".into(),
            "Akram\tTrader".into(),
            Want::EitherClean,
        ),
        (
            "new line",
            "اکرم\nملتانی".into(),
            "Akram\nMultani".into(),
            Want::EitherClean,
        ),
        (
            "carriage return",
            "اکرم\r".into(),
            "Akram\r".into(),
            Want::EitherClean,
        ),
        (
            "delete and next line",
            "اکرم\u{7F}".into(),
            "Akram\u{85}".into(),
            Want::EitherClean,
        ),
        (
            "line separator",
            "اکرم\u{2028}ملتانی".into(),
            "Akram\u{2029}Multani".into(),
            Want::EitherClean,
        ),
        (
            "emoji in the Urdu name",
            "اکرم😀".into(),
            "Akram".into(),
            Want::Refused(ErrorCode::ErrNameUrScript),
        ),
        (
            "emoji in the English name",
            "اکرم".into(),
            "Akram 😀".into(),
            Want::Refused(ErrorCode::ErrNameEnScript),
        ),
        ("emoji only", "😀".into(), "😀".into(), Want::RefusedAny),
        (
            "combining marks piled up",
            "ا\u{064E}\u{064E}\u{064E}\u{064E}\u{064E}کرم".into(),
            "Akram".into(),
            Want::EitherClean,
        ),
        (
            "SQL: drop table",
            "اکرم'; DROP TABLE accounts; --".into(),
            "Akram'; DROP TABLE accounts; --".into(),
            Want::RefusedAny,
        ),
        (
            "SQL: or 1=1",
            "اکرم' OR '1'='1".into(),
            "Akram' OR '1'='1".into(),
            Want::RefusedAny,
        ),
        (
            "SQL: union",
            "اکرم".into(),
            "x' UNION SELECT password_hash FROM users --".into(),
            Want::RefusedAny,
        ),
        (
            "SQL: only quotes",
            "اکرم".into(),
            "'''".into(),
            Want::RefusedAny,
        ),
        (
            "path: backslashes",
            "اکرم".into(),
            "..\\..\\windows".into(),
            Want::RefusedAny,
        ),
        (
            "path: slashes",
            "اکرم".into(),
            "../etc/passwd".into(),
            Want::EitherClean,
        ),
        (
            "script tag in English",
            "اکرم".into(),
            "<script>alert(1)</script>".into(),
            Want::RefusedAny,
        ),
        (
            "script tag in Urdu",
            "<script>اکرم</script>".into(),
            "Akram".into(),
            Want::RefusedAny,
        ),
        (
            "html entity",
            "اکرم".into(),
            "Akram &lt;b&gt;".into(),
            Want::EitherClean,
        ),
        (
            "percent and template text",
            "اکرم".into(),
            "Akram %s {{7*7}} ${x}".into(),
            Want::RefusedAny,
        ),
        (
            "100 characters of one letter",
            "ا".repeat(100),
            "A".repeat(100),
            Want::EitherClean,
        ),
        (
            "101 words characters",
            too_long_ur.clone(),
            too_long_en.clone(),
            Want::RefusedAny,
        ),
        (
            "one million characters",
            huge_ur.clone(),
            huge_en.clone(),
            Want::RefusedAny,
        ),
    ];
    let mut kept = 0_i64;
    let mut stored_names: Vec<(String, String)> = Vec::new();
    for (label, ur, en, want) in cases {
        let before_rows = count(&dir, "SELECT COUNT(*) FROM accounts");
        let before_audit = audit_total(&dir);
        let started = std::time::Instant::now();
        let result = accounts::save(&env.ctx, &owner, &form(AccountType::Buyer, &ur, &en));
        let took = started.elapsed();
        assert!(took.as_millis() < 2_000, "{label}: took {took:?}");
        match &result {
            Ok(row) => println!(
                "STORED   {label}: '{}' / '{}'",
                row.name_ur.escape_debug(),
                row.name_en.escape_debug()
            ),
            Err(error) => println!("REFUSED  {label}: {:?}", rejection_codes(error)),
        }
        match (&result, want) {
            (Err(error), Want::KeptOrSpelling) => {
                assert!(
                    rejection_codes(error).contains(&ErrorCode::ErrSpellingUnconfirmed),
                    "{label}: {error:?}"
                );
            }
            (Ok(row), Want::KeptOrSpelling) => {
                assert_eq!(
                    (row.name_ur.as_str(), row.name_en.as_str()),
                    (ur.trim(), en.trim()),
                    "{label}"
                );
                kept += 1;
                stored_names.push((row.name_ur.clone(), row.name_en.clone()));
            }
            (Ok(row), Want::Kept) => {
                assert_eq!(
                    (row.name_ur.as_str(), row.name_en.as_str()),
                    (ur.trim(), en.trim()),
                    "{label}"
                );
                kept += 1;
                stored_names.push((row.name_ur.clone(), row.name_en.clone()));
            }
            (Ok(row), Want::Cleaned) => {
                assert!(
                    !row.name_ur.chars().any(invisible),
                    "{label}: invisible mark kept in the Urdu name"
                );
                assert!(
                    !row.name_en.chars().any(invisible),
                    "{label}: invisible mark kept in the English name"
                );
                assert_eq!(row.name_ur, "اکرم", "{label}");
                assert_eq!(row.name_en, "Akram", "{label}");
                kept += 1;
                stored_names.push((row.name_ur.clone(), row.name_en.clone()));
            }
            (Ok(row), Want::EitherClean) => {
                assert!(
                    !row.name_ur.chars().any(invisible),
                    "{label}: invisible mark kept in '{}'",
                    row.name_ur.escape_debug()
                );
                assert!(
                    !row.name_en.chars().any(invisible),
                    "{label}: invisible mark kept in '{}'",
                    row.name_en.escape_debug()
                );
                kept += 1;
                stored_names.push((row.name_ur.clone(), row.name_en.clone()));
            }
            (Err(error), Want::Refused(code)) => {
                assert!(
                    rejection_codes(error).contains(&code),
                    "{label}: {:?}",
                    rejection_codes(error)
                );
            }
            (Err(error), Want::RefusedAny | Want::EitherClean) => {
                assert!(
                    matches!(error, ServiceError::Rejected(_)),
                    "{label}: internal error {error:?}"
                );
            }
            (Ok(_), Want::Refused(_) | Want::RefusedAny) => {
                panic!("{label}: was stored but must be refused")
            }
            (Err(error), Want::Kept | Want::Cleaned) => {
                panic!("{label}: was refused but must be stored: {error:?}")
            }
        }
        if result.is_err() {
            assert_eq!(
                count(&dir, "SELECT COUNT(*) FROM accounts"),
                before_rows,
                "{label}: refused but left a row"
            );
            assert_eq!(
                audit_total(&dir),
                before_audit,
                "{label}: refused but wrote an audit row"
            );
        }
    }
    // Nothing was run: the tables are all there, intact, and the users table is untouched.
    let tables = count(
        &dir,
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'accounts'",
    );
    assert_eq!(tables, 1);
    assert_eq!(count(&dir, "SELECT COUNT(*) FROM users"), 1);
    assert_eq!(
        count(
            &dir,
            "SELECT COUNT(*) FROM accounts WHERE name_en LIKE '%DROP%' OR name_en LIKE '%UNION%'"
        ),
        0
    );
    let integrity: String = raw(&dir)
        .query_row("PRAGMA integrity_check", [], |r| r.get(0))
        .unwrap();
    assert_eq!(integrity, "ok");
    assert!(kept >= 10);
    // Every stored name finds itself again through the search, in both scripts.
    let actor = auth::require(&env.ctx).unwrap();
    for (ur, en) in &stored_names {
        for text in [ur.as_str(), en.as_str()] {
            let hits = search::search_accounts(
                &env.ctx,
                &actor,
                &SearchAccountsInput {
                    scope: AccountScope::Any,
                    control_id: None,
                    text: text.chars().take(60).collect(),
                    limit: 50,
                },
            )
            .unwrap();
            assert!(
                hits.iter().any(|h| &h.name_ur == ur || &h.name_en == en),
                "the name '{}' / '{}' cannot be found by '{}'",
                ur.escape_debug(),
                en.escape_debug(),
                text.escape_debug()
            );
        }
    }
    assert_chain_ok(&dir);
}

#[test]
fn free_text_fields_keep_markup_as_plain_text_and_refuse_control_characters() {
    let (dir, env, owner) = ready();
    let mut ok = form(AccountType::Buyer, "اکرم", "Akram");
    ok.note = "<script>alert(1)</script> ' OR 1=1 -- ..\\..\\x".into();
    ok.address = "Kot Radha Kishan <b>1</b>".into();
    let row = accounts::save(&env.ctx, &owner, &ok).unwrap();
    let stored_note: String = raw(&dir)
        .query_row("SELECT note FROM accounts WHERE id = ?1", [row.id], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(
        stored_note, ok.note,
        "markup must be stored exactly as typed"
    );
    // 500 characters work, 501 do not.
    let mut at_limit = form(AccountType::Buyer, "ارشد", "Arshad");
    at_limit.note = "x".repeat(500);
    accounts::save(&env.ctx, &owner, &at_limit).unwrap();
    for (label, note) in [
        ("501 characters", "x".repeat(501)),
        ("a million characters", "x".repeat(1_000_000)),
        ("NUL", "a\0b".to_owned()),
        ("override mark", "a\u{202E}b".to_owned()),
        ("escape", "a\u{1b}[31mb".to_owned()),
    ] {
        let rows = count(&dir, "SELECT COUNT(*) FROM accounts");
        let mut input = form(AccountType::Buyer, "شاہد", "Shahid");
        input.note = note;
        let error = accounts::save(&env.ctx, &owner, &input).unwrap_err();
        assert!(
            matches!(error, ServiceError::Rejected(_)),
            "{label}: {error:?}"
        );
        assert_eq!(
            count(&dir, "SELECT COUNT(*) FROM accounts"),
            rows,
            "{label}"
        );
    }
    // Phones: the two good forms and the bad ones.
    for (phone, good) in [
        ("03001234567", true),
        ("0300-1234567", true),
        ("+923001234567", false),
        ("0300 1234567", false),
        ("0300-12345678", false),
        ("1234567", false),
        ("0300-123456a", false),
        ("٠٣٠٠١٢٣٤٥٦٧", true),
        ("٠٣٠٠-١٢٣٤٥٦٧", false),
        ("03001234567'; DROP TABLE accounts;--", false),
    ] {
        let mut input = form(AccountType::Buyer, "ناصر", "Nasir");
        input.phone1 = phone.into();
        let result = accounts::save(&env.ctx, &owner, &input);
        assert_eq!(result.is_ok(), good, "phone '{phone}': {result:?}");
        if let Ok(row) = result {
            assert_eq!(
                Some(row.phone1.len()),
                Some(11),
                "phone '{phone}' is stored as 11 digits"
            );
        }
    }
}

#[test]
fn names_that_differ_only_in_look_alike_letters_are_still_caught() {
    let (_dir, env, owner) = ready();
    let first = accounts::save(
        &env.ctx,
        &owner,
        &form(AccountType::Buyer, "اکرم ملتانی", "Akram Multani"),
    )
    .unwrap();
    // Each variant must bring the first account back in the duplicate warning.
    let variants_ur = [
        "اكرم ملتاني",                 // Arabic kaf and yeh
        "اکرم  ملتانی",                // double space
        "اک\u{200B}رم ملتانی",         // zero width space
        "اک\u{200D}رم ملتانی\u{FEFF}", // joiner and byte order mark
        "اَکرم ملتانی",                 // a zabar
        "اکرّم ملتانی",                 // a shadda
        "اکرم ملتانی ",                // trailing space
        "ا\u{0643}رم ملتانی",          // Arabic kaf inside
        "اکرم ملتانى",                 // alef maksura
        "اکرم مُلتانی",                 // a pesh
    ];
    for typed in variants_ur {
        let found = accounts::similar(
            &env.ctx,
            &AccountsSimilarInput {
                name_ur: typed.into(),
                name_en: String::new(),
                phone: String::new(),
                exclude_id: None,
                tag: String::new(),
            },
        )
        .unwrap();
        assert!(
            found.iter().any(|a| a.id == first.id),
            "Urdu variant '{}' did not find the account",
            typed.escape_unicode()
        );
    }
    let variants_en = [
        "AKRAM MULTANI",
        "akram multani",
        "Akram  Multani",
        "  Akram Multani  ",
        "Akram Multan",
        "Akrm Multani",
        "Akram Multanii",
        "Akram Mutlani",
    ];
    for typed in variants_en {
        let found = accounts::similar(
            &env.ctx,
            &AccountsSimilarInput {
                name_ur: String::new(),
                name_en: typed.into(),
                phone: String::new(),
                exclude_id: None,
                tag: String::new(),
            },
        )
        .unwrap();
        assert!(
            found.iter().any(|a| a.id == first.id),
            "English variant '{typed}' did not find the account"
        );
    }
    // The account itself is excluded when it is being edited.
    let own = accounts::similar(
        &env.ctx,
        &AccountsSimilarInput {
            name_ur: "اکرم ملتانی".into(),
            name_en: "Akram Multani".into(),
            phone: String::new(),
            exclude_id: Some(first.id),
            tag: String::new(),
        },
    )
    .unwrap();
    assert!(own.iter().all(|a| a.id != first.id));
    // Something that is not close does not match.
    let far = accounts::similar(
        &env.ctx,
        &AccountsSimilarInput {
            name_ur: "شاہد کریم".into(),
            name_en: "Shahid Karim".into(),
            phone: String::new(),
            exclude_id: None,
            tag: String::new(),
        },
    )
    .unwrap();
    assert!(far.iter().all(|a| a.id != first.id));
}

#[test]
fn the_search_index_rebuilt_from_the_database_gives_the_same_answers() {
    // D2-3: "index rebuilt after a simulated restore gives the same results".
    let (_dir, env, owner) = ready();
    let names = [
        ("اکرم ملتانی", "Akram Multani"),
        ("ارشد ملتانی", "Arshad Multani"),
        ("شاہد کریم", "Shahid Karim"),
        ("ناصر محمود", "Nasir Mehmood"),
        ("طارق محمود", "Tariq Mehmood"),
    ];
    for (n, (ur, en)) in names.iter().enumerate() {
        let mut input = form(
            if n % 2 == 0 {
                AccountType::Buyer
            } else {
                AccountType::Seller
            },
            ur,
            en,
        );
        input.phone1 = format!("0300123456{n}");
        accounts::save(&env.ctx, &owner, &input).unwrap();
    }
    // One account made inactive, one edited, so the live index went through updates.
    let list = accounts::list(
        &env.ctx,
        &mandi_services::core::accounts::AccountsListInput {
            filter: mandi_services::core::accounts::AccountFilter {
                account_type: None,
                text: String::new(),
                include_inactive: true,
            },
            offset: 0,
            limit: 0,
        },
    )
    .unwrap();
    let target = list
        .rows
        .iter()
        .find(|r| r.name_en == "Nasir Mehmood")
        .unwrap()
        .id;
    accounts::set_active(&env.ctx, &owner, target, false).unwrap();
    let actor = auth::require(&env.ctx).unwrap();
    let queries = [
        "akram",
        "akrm",
        "اکرم",
        "mehmood",
        "محمود",
        "multani",
        "ملتان",
        "03040001",
        "1",
        "2",
        "shahid",
        "tarq",
        "nasir",
        "xyz",
        "0300123456",
        "",
    ];
    let ask = |env: &TestEnv| -> Vec<Vec<(i64, String)>> {
        queries
            .iter()
            .flat_map(|q| {
                [AccountScope::Any, AccountScope::Buyer, AccountScope::Seller]
                    .into_iter()
                    .map(move |scope| (q, scope))
            })
            .map(|(q, scope)| {
                search::search_accounts(
                    &env.ctx,
                    &actor,
                    &SearchAccountsInput {
                        scope,
                        control_id: None,
                        text: (*q).into(),
                        limit: 50,
                    },
                )
                .unwrap()
                .into_iter()
                .map(|h| (h.id, h.code))
                .collect()
            })
            .collect()
    };
    let live = ask(&env);
    search::rebuild(&env.ctx).unwrap();
    assert_eq!(
        ask(&env),
        live,
        "a rebuilt index answers differently from the live one"
    );
    env.ctx.invalidate_caches();
    assert_eq!(
        ask(&env),
        live,
        "an index built after the caches were thrown away answers differently"
    );
}
