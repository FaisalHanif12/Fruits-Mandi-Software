//! D8 end gate, attack on چکتا buyers (PRD Section 13, FR-7.29, FR-13.1 to FR-13.9, plan 64; Code Standards
//! 4.2, V0). Owner: `robustness`.
//!
//! 1. `hostile_names_and_phones_on_chukta_add_and_on_a_sale_line`: control, zero-width and bidi characters,
//!    wrong script, only spaces, 101 and 100,000 characters, SQL, HTML, path text, the key separator `|`,
//!    emoji, odd phone shapes. Each is refused at its own field with nothing written, or stored clean
//!    (trimmed, no control or format character, within 100 characters) and found again. The same texts are
//!    sent on a بکری line through the buyer 0030.
//! 2. `spelling_variants_are_one_person_and_a_different_phone_is_another`.
//! 3. `eight_threads_add_the_same_person_and_forty_different_ones`.
//! 4. `no_voucher_and_no_opening_and_no_edit_can_put_a_balance_on_0030_or_0031`: receipt, payment, advance,
//!    جنرل with 0030 on either side, an opening balance, a line to the closed account 0031.
//! 5. `a_chukta_name_on_a_line_of_an_ordinary_buyer_leaves_no_trace`: a stray person on akram, on نقدی منڈی.
//! 6. `edits_that_move_a_line_between_chukta_and_ordinary_buyers_keep_the_books_exact`.
//! 7. `the_picker_never_offers_0030_or_0031_and_chukta_people_stay_out_of_every_other_list`.
#![cfg(test)]

mod common;
#[allow(dead_code)]
#[path = "bikri_save/helpers.rs"]
mod helpers;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;
#[allow(dead_code)]
#[path = "bikri_edit/tools.rs"]
mod tools;

use common::{assert_chain_ok, raw};
use helpers::*;
use mandi_services::core::accounts::{
    AccountScope, BalanceSide, HitKind, OpeningLineInput, OpeningsInput, SearchAccountsInput,
};
use mandi_services::core::bikri::{BikriDraft, ChuktaInput};
use mandi_services::core::chukta::{ChuktaAddInput, ChuktaSearchInput};
use mandi_services::core::error::ErrorCode;
use mandi_services::core::general::{GeneralInput, GeneralLineInput};
use mandi_services::core::vouchers::{
    AdvanceInput, PaymentInput, ReceiptInput, RemovedPaymentFields,
};
use mandi_services::{ServiceError, accounts, chukta, ledger, search, vouchers};
use support::*;

fn add_raw(
    w: &World,
    ur: &str,
    en: &str,
    phone: Option<&str>,
) -> Result<mandi_services::core::chukta::ChuktaBuyer, ServiceError> {
    chukta::add(
        &w.env.ctx,
        &w.owner,
        &ChuktaAddInput {
            name_ur: ur.into(),
            name_en: en.into(),
            phone: phone.map(str::to_owned),
        },
    )
}

fn rows(w: &World) -> i64 {
    scalar(w, "SELECT COUNT(*) FROM chukta_buyers")
}

fn audit_rows(w: &World) -> i64 {
    scalar(w, "SELECT COUNT(*) FROM audit_log")
}

fn fields_of(error: &ServiceError) -> Vec<(ErrorCode, String)> {
    match error {
        ServiceError::Rejected(items) => items
            .iter()
            .map(|i| (i.code, i.field.clone().unwrap_or_default()))
            .collect(),
        other => panic!("not a refusal: {other:?}"),
    }
}

/// True if a stored name holds nothing but plain printable characters, trimmed, within 100 characters.
fn is_clean_name(name: &str) -> bool {
    let n = name.chars().count();
    (1..=100).contains(&n)
        && name == name.trim()
        && !name.contains('|')
        && !name.chars().any(|c| {
            c.is_control()
                || matches!(c, '\u{200b}'..='\u{200f}' | '\u{202a}'..='\u{202e}' | '\u{2060}'..='\u{2064}' | '\u{2066}'..='\u{2069}' | '\u{feff}' | '\u{00ad}')
        })
}

const GOOD_UR: &str = "شاہد";
const GOOD_EN: &str = "Shahid";

/// (ur, en, phone, must refuse, field expected when refused)
fn hostile() -> Vec<(String, String, Option<String>, bool, &'static str)> {
    let long_ur = "ش".repeat(100_000);
    let long_en = "S".repeat(100_000);
    let mut v: Vec<(String, String, Option<String>, bool, &'static str)> = Vec::new();
    let ur = |t: &str, must: bool| (t.to_owned(), GOOD_EN.to_owned(), None, must, "name_ur");
    let en = |t: &str, must: bool| (GOOD_UR.to_owned(), t.to_owned(), None, must, "name_en");
    let ph = |t: &str, must: bool| {
        (
            GOOD_UR.to_owned(),
            GOOD_EN.to_owned(),
            Some(t.to_owned()),
            must,
            "phone",
        )
    };
    for (t, must) in [
        ("", true),
        ("   ", true),
        ("\t\n", true),
        ("شاہد\u{0}", true),
        ("شا\u{0}ہد", true),
        ("شاہد\u{7}", true),
        ("شاہد\u{1b}[31m", true),
        ("Shahid", true),
        ("شاہد Shahid", true),
        ("123", true),
        ("شاہد|03000000000", true),
        ("شاہد'; DROP TABLE chukta_buyers; --", true),
        ("<script>alert(1)</script>", true),
        ("..\\..\\windows", true),
        ("😀شاہد", true),
        ("{{7*7}}", true),
        ("${jndi:ldap://x}", true),
        (&"ش".repeat(101), true),
        (&"ش".repeat(100), false),
        (&"ش".repeat(100_000), true),
        ("  شاہد  ", false),
        ("شا\u{200b}ہد", false),
        ("شاہد\u{200d}", false),
        ("شاہد\u{feff}", false),
        ("\u{202e}شاہد", false),
        ("شاہد\u{200f}", false),
        ("شاہد\u{202c}", false),
        ("شاہد خان", false),
        ("شاہد  خان", false),
        ("شَاہِد", false),
        ("شاہد (ٹول ٹیکس)", false),
    ] {
        v.push(ur(t, must));
    }
    for (t, must) in [
        ("", true),
        ("   ", true),
        ("Shahid\u{0}", true),
        ("Sha\u{0}hid", true),
        ("Shahid\u{7}", true),
        ("شاہد", true),
        ("Shahid شاہد", true),
        ("123", true),
        ("Shahid|1", true),
        ("Shahid'; DROP TABLE chukta_buyers; --", true),
        ("<script>alert(1)</script>", true),
        ("..\\..\\windows", true),
        ("😀Shahid", true),
        ("\u{ff33}hahid", true),
        (&"S".repeat(101), true),
        (&"S".repeat(100), false),
        (&"S".repeat(100_000), true),
        ("  Shahid  ", false),
        ("Sha\u{200b}hid", false),
        ("Shahid\u{200d}", false),
        ("\u{202e}Shahid", false),
        ("O'Brien", false),
        ("Shahid & Sons", false),
        ("Shahid.K (Toll)", false),
        ("Shahid Mehmood", false),
    ] {
        v.push(en(t, must));
    }
    for (t, must) in [
        ("0300-0000000", false),
        ("03000000000", false),
        ("  03000000000  ", false),
        ("0300 0000000", false),
        ("+923000000000", false),
        ("0300000000", true),
        ("030000000000", true),
        ("abc", true),
        ("0300-000000a", true),
        ("03000000000\u{0}", true),
        ("0300-0000000; DROP TABLE x", true),
        ("٠٣٠٠٠٠٠٠٠٠٠٠", false),
        (
            "\u{ff10}\u{ff13}\u{ff10}\u{ff10}\u{ff10}\u{ff10}\u{ff10}\u{ff10}\u{ff10}\u{ff10}\u{ff10}",
            false,
        ),
        ("1", true),
        (&"0".repeat(10_000), true),
        ("03", true),
        ("0300-00000000000000", true),
    ] {
        v.push(ph(t, must));
    }
    v.push((long_ur, GOOD_EN.to_owned(), None, true, "name_ur"));
    v.push((GOOD_UR.to_owned(), long_en, None, true, "name_en"));
    v
}

#[test]
fn hostile_names_and_phones_on_chukta_add_and_on_a_sale_line() {
    let w = world();
    let mut stored = 0;
    for (ur, en, phone, must_refuse, field) in hostile() {
        let shown = |t: &str| -> String { t.chars().take(30).collect() };
        let what = format!(
            "ur {:?} en {:?} phone {:?}",
            shown(&ur),
            shown(&en),
            phone.as_deref().map(shown)
        );
        let before = (rows(&w), audit_rows(&w));
        let started = std::time::Instant::now();
        let result = add_raw(&w, &ur, &en, phone.as_deref());
        assert!(
            started.elapsed().as_millis() < 3_000,
            "{what}: took {:?}",
            started.elapsed()
        );
        match result {
            Err(e) => {
                assert!(
                    matches!(e, ServiceError::Rejected(_)),
                    "{what}: not a clean refusal: {e:?}"
                );
                let f = fields_of(&e);
                assert!(
                    f.iter().any(|(_, fld)| fld == field),
                    "{what}: refused at {f:?}, expected a refusal at {field}"
                );
                assert!(
                    f.iter()
                        .all(|(_, fld)| ["name_ur", "name_en", "phone"].contains(&fld.as_str())),
                    "{what}: {f:?}"
                );
                assert_eq!(
                    (rows(&w), audit_rows(&w)),
                    before,
                    "{what}: a refusal wrote something"
                );
            }
            Ok(buyer) => {
                assert!(!must_refuse, "{what}: accepted, but must be refused");
                assert!(
                    is_clean_name(&buyer.name_ur),
                    "{what}: stored name_ur {:?}",
                    buyer.name_ur
                );
                assert!(
                    is_clean_name(&buyer.name_en),
                    "{what}: stored name_en {:?}",
                    buyer.name_en
                );
                if let Some(p) = &buyer.phone {
                    assert!(
                        p.len() == 11
                            && p.starts_with("03")
                            && p.chars().all(|c| c.is_ascii_digit()),
                        "{what}: stored phone {p:?}"
                    );
                }
                // He is found again by his own names, and adding him again gives the same row.
                let found = chukta::search(
                    &w.env.ctx,
                    &ChuktaSearchInput {
                        text: buyer.name_en.clone(),
                    },
                )
                .unwrap();
                assert!(
                    found.iter().any(|b| b.id == buyer.id),
                    "{what}: not found by name"
                );
                let again = add_raw(&w, &ur, &en, phone.as_deref()).unwrap();
                assert_eq!(again.id, buyer.id, "{what}: a second add made another row");
                stored += 1;
            }
        }
    }
    assert!(stored >= 15, "only {stored} texts were stored");
    let integrity: String = raw(&w.dir)
        .query_row("PRAGMA integrity_check", [], |r| r.get(0))
        .unwrap();
    assert_eq!(integrity, "ok");
    // No two rows share a key, no key holds a second separator, every key is the one of its name and phone.
    let keys: Vec<(String, String, Option<String>)> = raw(&w.dir)
        .prepare("SELECT person_key, name_ur, phone FROM chukta_buyers")
        .unwrap()
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    let mut seen = std::collections::HashSet::new();
    for (key, name_ur, phone) in &keys {
        assert!(seen.insert(key.clone()), "duplicate key {key:?}");
        assert_eq!(key.matches('|').count(), 1, "{key:?}");
        assert_eq!(
            key,
            &mandi_services::core::text::person_key(name_ur, phone.as_deref().unwrap_or("")),
            "key does not match name and phone"
        );
    }
    assert_chain_ok(&w.dir);

    // The same texts on a بکری line through the buyer 0030: refused at the chukta fields with nothing
    // saved, or saved clean and in the list.
    let w = world();
    for (ur, en, phone, must_refuse, field) in hostile() {
        let what = format!(
            "line: ur {:?} en {:?} phone {:?}",
            ur.chars().take(20).collect::<String>(),
            en.chars().take(20).collect::<String>(),
            phone
                .as_deref()
                .map(|p| p.chars().take(20).collect::<String>())
        );
        let mut s = sale("s-1", w.chukta_khata, "10", "1200");
        s.chukta = Some(ChuktaInput {
            name_ur: ur.clone(),
            name_en: en.clone(),
            phone: phone.clone().unwrap_or_default(),
        });
        let d = draft(vec![with_sales(
            bare_lot(&w, "lot-1", w.karim, w.apple, "10"),
            vec![s],
        )]);
        let before = snapshot(&raw(&w.dir));
        let chukta_before = rows(&w);
        let p = preview(&w, &d);
        match save(&w, &d) {
            Ok(saved) => {
                assert!(!must_refuse, "{what}: saved, but must be refused");
                assert!(p.errors.is_empty(), "{what}: {:?}", p.errors);
                let (name_ur, name_en, ph, key): (String, String, Option<String>, String) =
                    raw(&w.dir)
                        .query_row(
                            "SELECT chukta_name_ur, chukta_name_en, chukta_phone, chukta_person_key
                         FROM bikri_sales WHERE bikri_id = ?1",
                            [saved.bikri_id],
                            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
                        )
                        .unwrap();
                assert!(is_clean_name(&name_ur), "{what}: line name_ur {name_ur:?}");
                assert!(is_clean_name(&name_en), "{what}: line name_en {name_en:?}");
                // A line with no phone stores an empty text, not NULL (the list of buyers uses NULL).
                if let Some(p) = ph.as_deref().filter(|p| !p.is_empty()) {
                    assert!(
                        p.len() == 11 && p.chars().all(|c| c.is_ascii_digit()),
                        "{what}: {p:?}"
                    );
                }
                assert_eq!(key.matches('|').count(), 1, "{what}: key {key:?}");
                let in_list: i64 = scalar(
                    &w,
                    &format!(
                        "SELECT COUNT(*) FROM chukta_buyers WHERE person_key = '{}'",
                        key.replace('\'', "''")
                    ),
                );
                assert_eq!(
                    in_list, 1,
                    "{what}: the person of the line is not in the list"
                );
            }
            Err(e) => {
                assert!(matches!(e, ServiceError::Rejected(_)), "{what}: {e:?}");
                let f = fields_of(&e);
                assert!(
                    f.iter()
                        .all(|(_, fld)| fld.starts_with("lots[0].sales[0].chukta")),
                    "{what}: a refusal at the wrong field: {f:?} (expected {field})"
                );
                assert!(
                    !p.errors.is_empty(),
                    "{what}: refused by the save, clean in the preview"
                );
                assert_eq!(
                    snapshot(&raw(&w.dir)),
                    before,
                    "{what}: a refused save wrote something"
                );
                assert_eq!(rows(&w), chukta_before, "{what}: the list changed");
            }
        }
    }
    assert_eq!(tools::trial_difference(&w), 0);
    let (side, _) = balance(&w, w.chukta_khata);
    assert_eq!(side, BalanceSide::Zero, "0030 holds a balance");
    assert_chain_ok(&w.dir);
}

#[test]
fn spelling_variants_are_one_person_and_a_different_phone_is_another() {
    let w = world();
    let base = add_raw(&w, "علی خان", "Ali Khan", Some("0300-1234567"))
        .unwrap()
        .id;
    for (ur, phone) in [
        ("علي خان", "03001234567"),         // Arabic yeh
        ("علی  خان", "0300-1234567"),       // double space
        ("عَلی خان", "0300-1234567"),        // a zabar
        ("علی\u{200b} خان", "03001234567"), // zero-width space
        (" علی خان ", "  03001234567 "),
        ("علی خان", "\t0300-1234567\n"),
    ] {
        let again = add_raw(&w, ur, "Ali Khan", Some(phone)).unwrap();
        assert_eq!(again.id, base, "{ur:?} {phone:?} is the same person");
    }
    assert_eq!(rows(&w), 1);
    // Another phone, no phone, another Urdu name: other people. The English name does not make one.
    let other_phone = add_raw(&w, "علی خان", "Ali Khan", Some("0300-7654321"))
        .unwrap()
        .id;
    let no_phone = add_raw(&w, "علی خان", "Ali Khan", None).unwrap().id;
    let other_name = add_raw(&w, "علی احمد", "Ali Khan", Some("0300-1234567"))
        .unwrap()
        .id;
    let same_with_new_english = add_raw(&w, "علی خان", "Aly Khan", Some("0300-1234567")).unwrap();
    assert_eq!(same_with_new_english.id, base);
    assert_eq!(
        same_with_new_english.name_en, "Ali Khan",
        "the list keeps the first spelling"
    );
    let ids = [base, other_phone, no_phone, other_name];
    let set: std::collections::HashSet<_> = ids.iter().collect();
    assert_eq!(set.len(), 4);
    assert_eq!(rows(&w), 4);
    // Do-chashmi heh is another letter: a different person, never merged by accident.
    let dochashmi = add_raw(&w, "علی  خان\u{06be}", "Ali Khanh", Some("0300-1234567"))
        .unwrap()
        .id;
    assert!(!ids.contains(&dochashmi));
}

#[test]
fn eight_threads_add_the_same_person_and_forty_different_ones() {
    let w = world();
    let barrier = std::sync::Barrier::new(8);
    let ids: Vec<i64> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..8)
            .map(|_| {
                scope.spawn(|| {
                    barrier.wait();
                    add_raw(&w, "شاہد", "Shahid", Some("0300-0000000"))
                        .unwrap()
                        .id
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });
    assert!(ids.iter().all(|i| *i == ids[0]), "{ids:?}");
    assert_eq!(rows(&w), 1);
    assert_eq!(
        scalar(
            &w,
            "SELECT COUNT(*) FROM audit_log WHERE entity = 'chukta_buyer' AND action = 'create'"
        ),
        1,
        "one audit row for one new person"
    );
    // Forty different people, five in each of eight threads. English names without digits, so the
    // search by name is not mixed with the search by phone digits.
    let names = [
        "احمد",
        "بلال",
        "جمال",
        "دانش",
        "رضا",
        "سعید",
        "طارق",
        "عمران",
    ];
    let english = [
        "Ahmad", "Bilal", "Jamal", "Danish", "Raza", "Saeed", "Tariq", "Imran",
    ];
    let ones = ["ایک", "دو", "تین", "چار", "پانچ"];
    let words = ["Alpha", "Bravo", "Charlie", "Delta", "Echo"];
    std::thread::scope(|scope| {
        for t in 0..8 {
            let w = &w;
            scope.spawn(move || {
                for k in 0..5 {
                    let en = format!("{} {}", english[t], words[k]);
                    let ur = format!("{} {}", names[t], ones[k]);
                    add_raw(w, &ur, &en, Some(&format!("0300-00000{t}{k}"))).unwrap();
                }
            });
        }
    });
    assert_eq!(rows(&w), 41);
    for (t, name) in english.iter().enumerate() {
        let found = chukta::search(
            &w.env.ctx,
            &ChuktaSearchInput {
                text: format!("{name} "),
            },
        )
        .unwrap();
        assert_eq!(found.len(), 5, "person {t}: {found:?}");
    }
    assert_chain_ok(&w.dir);
}

#[test]
fn no_voucher_and_no_opening_and_no_edit_can_put_a_balance_on_0030_or_0031() {
    let w = world();
    // One real چکتا sale, so 0030 has lines at all.
    let mut s = sale("s-1", w.chukta_khata, "10", "1200");
    s.chukta = person("شاہد", "Shahid", "0300-0000000");
    save_ok(
        &w,
        &draft(vec![with_sales(
            bare_lot(&w, "lot-1", w.karim, w.apple, "10"),
            vec![s],
        )]),
    );
    let old_closed = id_of(&w.env, "0031");
    let cash_before = tools::cash(&w);
    for account in [w.chukta_khata, old_closed] {
        let before = tools::fingerprint(&raw(&w.dir));
        let tag = format!("account {account}");
        let receipt = vouchers::save_receipt(
            &w.env.ctx,
            &w.owner,
            &ReceiptInput {
                date: "10-10-2026".into(),
                account_id: account,
                amount: "1000".into(),
                removed: RemovedPaymentFields::default(),
                collector_staff_id: None,
                narration: String::new(),
            },
        );
        assert!(receipt.is_err(), "{tag}: a receipt was taken");
        let payment = vouchers::save_payment(
            &w.env.ctx,
            &w.owner,
            &PaymentInput {
                date: "10-10-2026".into(),
                account_id: account,
                amount: "1000".into(),
                removed: RemovedPaymentFields::default(),
                reason: "cash".into(),
                narration: String::new(),
                confirm_cash_below_zero: true,
            },
        );
        assert!(payment.is_err(), "{tag}: a payment was taken");
        let advance = vouchers::save_advance(
            &w.env.ctx,
            &w.owner,
            &AdvanceInput {
                confirm_cash_below_zero: true,
                date: "10-10-2026".into(),
                seller_account_id: account,
                amount: "1000".into(),
                removed: RemovedPaymentFields::default(),
                expected_goods: String::new(),
                narration: String::new(),
            },
        );
        assert!(advance.is_err(), "{tag}: an advance was given");
        let cash = id_of(&w.env, "01010001");
        for (dr_acc, cr_acc) in [(account, cash), (cash, account)] {
            let general = vouchers::general_save(
                &w.env.ctx,
                &w.owner,
                &GeneralInput {
                    date: "10-10-2026".into(),
                    narration: None,
                    lines: vec![
                        GeneralLineInput {
                            account_id: Some(dr_acc),
                            dr: "1000".into(),
                            cr: String::new(),
                            narration: None,
                        },
                        GeneralLineInput {
                            account_id: Some(cr_acc),
                            dr: String::new(),
                            cr: "1000".into(),
                            narration: None,
                        },
                    ],
                    confirm_cash_below_zero: true,
                },
            );
            assert!(
                general.is_err(),
                "{tag}: a جنرل line was taken ({dr_acc} / {cr_acc})"
            );
        }
        // An opening balance on it.
        let row = accounts::get(&w.env.ctx, account).unwrap();
        let opening = accounts::openings_save(
            &w.env.ctx,
            &w.owner,
            &OpeningsInput {
                lines: vec![OpeningLineInput {
                    account_id: account,
                    version: row.version,
                    opening_date: "10-10-2026".into(),
                    opening_dr: "500".into(),
                    opening_cr: String::new(),
                    opening_is_advance: false,
                }],
            },
        );
        assert!(opening.is_err(), "{tag}: an opening balance was taken");
        assert_eq!(
            tools::fingerprint(&raw(&w.dir)),
            before,
            "{tag}: a refusal wrote something"
        );
    }
    // A بکری line to the closed account 0031 with or without a person is refused too.
    for chukta_input in [None, person("بشیر", "Bashir", "0300-2222222")] {
        let mut s = sale("s-1", old_closed, "10", "1200");
        s.chukta = chukta_input;
        let d = draft(vec![with_sales(
            bare_lot(&w, "lot-1", w.karim, w.apple, "10"),
            vec![s],
        )]);
        let before = snapshot(&raw(&w.dir));
        assert!(save(&w, &d).is_err(), "a line to 0031 was saved");
        assert_eq!(snapshot(&raw(&w.dir)), before);
    }
    for account in [w.chukta_khata, old_closed] {
        let b = ledger::account_balance(&w.env.ctx, account).unwrap();
        assert_eq!(
            b.side,
            BalanceSide::Zero,
            "account {account} holds a balance"
        );
    }
    assert_eq!(tools::cash(&w), cash_before);
    assert_eq!(tools::trial_difference(&w), 0);
    assert_chain_ok(&w.dir);
}

#[test]
fn a_chukta_name_on_a_line_of_an_ordinary_buyer_leaves_no_trace() {
    let w = world();
    let list_before = rows(&w);
    for buyer in [w.akram, w.naqdi] {
        let mut s = sale("s-1", buyer, "10", "1200");
        s.chukta = person("طارق", "Tariq", "0300-5555555");
        s.paid = Some("1".into());
        let d = draft(vec![with_sales(
            bare_lot(&w, "lot-1", w.karim, w.apple, "10"),
            vec![s],
        )]);
        let saved = save_ok(&w, &d);
        let stray: i64 = scalar(
            &w,
            &format!(
                "SELECT (SELECT COUNT(*) FROM bikri_sales WHERE bikri_id = {id} AND (chukta_person_key IS NOT NULL OR chukta_name_ur IS NOT NULL OR chukta_name_en IS NOT NULL OR chukta_phone IS NOT NULL))
                      + (SELECT COUNT(*) FROM journal_lines WHERE entry_id = {entry} AND chukta_person_key IS NOT NULL)
                      + (SELECT COUNT(*) FROM analytics_facts WHERE bikri_id = {id} AND chukta_person_key IS NOT NULL)",
                id = saved.bikri_id,
                entry = saved.entry_id
            ),
        );
        assert_eq!(stray, 0, "a stray person was stored for buyer {buyer}");
    }
    assert_eq!(
        rows(&w),
        list_before,
        "the list of چکتا buyers grew from a stray name"
    );
    assert_eq!(tools::trial_difference(&w), 0);
    // A line on 0030 with no person at all is refused at the name, with nothing saved.
    let s = sale("s-1", w.chukta_khata, "10", "1200");
    let d = draft(vec![with_sales(
        bare_lot(&w, "lot-1", w.karim, w.apple, "10"),
        vec![s],
    )]);
    let before = snapshot(&raw(&w.dir));
    let e = save(&w, &d).unwrap_err();
    assert!(
        fields_of(&e)
            .iter()
            .any(|(c, f)| *c == ErrorCode::ErrChuktaNameRequired
                && f == "lots[0].sales[0].chukta.name_ur"),
        "{e:?}"
    );
    assert_eq!(snapshot(&raw(&w.dir)), before);
}

#[test]
fn edits_that_move_a_line_between_chukta_and_ordinary_buyers_keep_the_books_exact() {
    let w = world();
    let mut s = sale("s-1", w.chukta_khata, "10", "1200");
    s.chukta = person("شاہد", "Shahid", "0300-0000000");
    let saved = save_ok(
        &w,
        &draft(vec![with_sales(
            bare_lot(&w, "lot-1", w.karim, w.apple, "10"),
            vec![s],
        )]),
    );
    let mut entry = saved.entry_id;
    let steps: [(&str, i64, Option<ChuktaInput>); 6] = [
        ("to akram", w.akram, None),
        (
            "to a person",
            w.chukta_khata,
            person("ناصر", "Nasir", "0300-1111111"),
        ),
        (
            "to another person",
            w.chukta_khata,
            person("ناصر", "Nasir", "0300-2222222"),
        ),
        ("to naqdi", w.naqdi, None),
        ("to arshad", w.arshad, None),
        (
            "back to shahid",
            w.chukta_khata,
            person("شاہد", "Shahid", "0300-0000000"),
        ),
    ];
    for (name, buyer, chukta_input) in steps {
        let opened = tools::open(&w, saved.bikri_id, entry);
        let mut d: BikriDraft = opened.draft.clone();
        d.lots[0].sales[0].buyer_account_id = Some(buyer);
        d.lots[0].sales[0].chukta = chukta_input;
        let edited = tools::edit(&w, &opened, &d);
        entry = edited.entry_id;
        assert_eq!(tools::trial_difference(&w), 0, "{name}");
        assert_chain_ok(&w.dir);
        // 0030 holds no balance whatever the line did; a person key sits on 0030 lines only.
        let (side, _) = balance(&w, w.chukta_khata);
        assert_eq!(side, BalanceSide::Zero, "{name}: 0030 holds a balance");
        let wrong: i64 = scalar(
            &w,
            "SELECT COUNT(*) FROM journal_lines l JOIN accounts a ON a.id = l.account_id
             WHERE l.chukta_person_key IS NOT NULL AND a.code <> '0030'",
        );
        assert_eq!(wrong, 0, "{name}: a person key on an ordinary account");
        let wrong_sales: i64 = scalar(
            &w,
            "SELECT COUNT(*) FROM bikri_sales s JOIN accounts a ON a.id = s.buyer_account_id
             WHERE s.is_active = 1 AND ((a.code = '0030') <> (s.chukta_person_key IS NOT NULL))",
        );
        assert_eq!(wrong_sales, 0, "{name}: person key and buyer 0030 disagree");
        // Every ordinary buyer owes what is left on his live lines.
        for b in [w.akram, w.arshad, w.naqdi] {
            let left: i64 = scalar(
                &w,
                &format!(
                    "SELECT COALESCE(SUM(buyer_total_paisa - paid_paisa), 0) FROM bikri_sales
                     WHERE buyer_account_id = {b} AND is_active = 1"
                ),
            );
            let (side, amount) = balance(&w, b);
            let signed = if side == BalanceSide::Cr {
                -amount
            } else {
                amount
            };
            assert_eq!(signed * 100, left, "{name}: account {b}");
        }
    }
}

#[test]
fn the_picker_never_offers_0030_or_0031_and_chukta_people_stay_out_of_every_other_list() {
    let w = world();
    for (ur, en, ph) in [
        ("شاہد", "Shahid", "0300-0000000"),
        ("ناصر", "Nasir", "0300-1111111"),
    ] {
        let mut s = sale("s-1", w.chukta_khata, "5", "1000");
        s.chukta = person(ur, en, ph);
        save_ok(
            &w,
            &draft(vec![with_sales(
                bare_lot(&w, "lot-1", w.karim, w.apple, "5"),
                vec![s],
            )]),
        );
    }
    let old_closed = id_of(&w.env, "0031");
    let ask = |scope: AccountScope, text: &str, flag: Option<bool>, dead: Option<bool>| {
        search::search_accounts(
            &w.env.ctx,
            &w.owner,
            &SearchAccountsInput {
                scope,
                control_id: None,
                text: text.into(),
                limit: 50,
                offset: 0,
                include_dead: dead,
                with_chukta: flag,
            },
        )
        .unwrap()
    };
    for scope in [AccountScope::Buyer, AccountScope::Seller, AccountScope::Any] {
        for flag in [None, Some(false), Some(true)] {
            for dead in [None, Some(true)] {
                for text in [
                    "",
                    "0030",
                    "0031",
                    "chukta",
                    "old closed",
                    "shahid",
                    "nasir",
                    "شاہد",
                    "پرانا",
                    "03000000000",
                ] {
                    let hits = ask(scope, text, flag, dead);
                    for h in &hits {
                        let ok_person = h.kind == HitKind::Chukta
                            && scope == AccountScope::Buyer
                            && flag == Some(true);
                        assert!(
                            ok_person || (h.id != w.chukta_khata && h.id != old_closed),
                            "{scope:?} {flag:?} {dead:?} {text:?}: offered {h:?}"
                        );
                    }
                }
            }
        }
    }
    // People appear once each in the buyer field of a line, marked چکتا.
    let hits = ask(AccountScope::Buyer, "", Some(true), None);
    let people: Vec<_> = hits.iter().filter(|h| h.kind == HitKind::Chukta).collect();
    assert_eq!(people.len(), 2);
}
