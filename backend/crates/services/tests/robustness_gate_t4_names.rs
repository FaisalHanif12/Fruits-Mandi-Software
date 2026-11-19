//! Gate lane T4: Unicode, Urdu and bad text through the service entry points (plan 80; Code
//! Standards 4.2, 4.3, V0, B26; PRD FR-7.28 script checks, FR-7.24, FR-10.2 tag, E24, E31, E35,
//! E38, E42; NFR-URD-18 letter variants). Owner: `robustness`.
//!
//! `core/tests/robustness/` attacks the pure checks (`validate`, `text`). This file goes one level
//! up, through `accounts::save`, `search` and `bikri::preview/save`, because the rules are only
//! worth something if the SAVED row obeys them and a refused save leaves the file alone:
//!
//! 1. A table of names, tags, phones and notes at the edges: 100 and 101 characters, spaces round a
//!    name, mixed scripts, emoji, NUL and bell characters, zero-width joiners, right-to-left marks.
//!    Each is saved clean or refused at its own field, with the code of the rule.
//! 2. Letter variants of NFR-URD-18 (ي ى ك ه ۃ, zabar, ZWJ and ZWNJ): a name saved in one form is
//!    found by the others.
//! 3. A property test over `accounts::save` with random text from a hostile alphabet: a refusal
//!    commits nothing (`PRAGMA data_version` of a second connection does not move), and whatever
//!    is saved obeys the rules of Code Standards 4.2 when read back.
//! 4. A property test over a whole بکری draft: one or two leaves of the draft are replaced by
//!    hostile text or numbers; the preview never fails, a draft with errors is never saved, and a
//!    saved draft gives the numbers of its preview and balanced books.
#![cfg(test)]

mod common;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use common::raw;
use mandi_services::core::accounts::{
    AccountHit, AccountInput, AccountRow, AccountScope, AccountType, SearchAccountsInput,
};
use mandi_services::core::bikri::BikriSaveInput;
use mandi_services::core::error::{ErrorCode, ErrorValue};
use mandi_services::db::rusqlite::Connection;
use mandi_services::{ServiceError, accounts, bikri, search};
use proptest::prelude::*;
use proptest::test_runner::{Config, TestCaseError, TestRunner};
use serde_json::Value;
use support::*;

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
        is_active: true,
        note: String::new(),
        tag: String::new(),
    }
}

/// Saves, confirming the rules-only spellings the app asks about (FR-7.66) like the screen does
/// when the user presses "confirm". Any other refusal is returned.
fn try_save(w: &World, mut input: AccountInput) -> Result<AccountRow, ServiceError> {
    for _ in 0..4 {
        match accounts::save(&w.env.ctx, &w.owner, &input) {
            Err(ServiceError::Rejected(items))
                if !items.is_empty()
                    && items
                        .iter()
                        .all(|i| i.code == ErrorCode::ErrSpellingUnconfirmed) =>
            {
                for item in &items {
                    if let Some(ErrorValue::Text(words)) = item.values.get("words") {
                        input
                            .confirmed_rules_words
                            .extend(words.split(' ').map(str::to_owned));
                    }
                }
            }
            other => return other,
        }
    }
    accounts::save(&w.env.ctx, &w.owner, &input)
}

fn codes(error: &ServiceError) -> Vec<(ErrorCode, String)> {
    match error {
        ServiceError::Rejected(items) => items
            .iter()
            .map(|i| (i.code, i.field.clone().unwrap_or_default()))
            .collect(),
        other => vec![(other.code(), String::new())],
    }
}

struct Probe(Connection);

impl Probe {
    fn version(&self) -> i64 {
        self.0
            .query_row("PRAGMA data_version", [], |r| r.get(0))
            .unwrap()
    }
}

fn invisible(c: char) -> bool {
    c.is_control()
        || matches!(
            c,
            '\u{200b}'..='\u{200f}'
                | '\u{202a}'..='\u{202e}'
                | '\u{2060}'..='\u{2069}'
                | '\u{feff}'
                | '\u{061c}'
        )
}

fn urdu_letter(c: char) -> bool {
    matches!(c, '\u{0600}'..='\u{06ff}' | '\u{0750}'..='\u{077f}' | '\u{fb50}'..='\u{fdff}' | '\u{fe70}'..='\u{feff}')
        && c.is_alphabetic()
}

/// What a stored account must look like, read back (Code Standards 4.2). Empty when it does.
fn rule_breaks(row: &AccountRow) -> Vec<String> {
    let mut bad = Vec::new();
    let (ur, en) = (&row.name_ur, &row.name_en);
    for (label, name) in [("name_ur", ur), ("name_en", en)] {
        let n = name.chars().count();
        if !(1..=100).contains(&n) {
            bad.push(format!("{label} has {n} characters"));
        }
        if name.chars().any(invisible) {
            bad.push(format!(
                "{label} holds a control or invisible character: {name:?}"
            ));
        }
        if name.trim() != name.as_str() {
            bad.push(format!("{label} is not trimmed: {name:?}"));
        }
    }
    if !ur.chars().any(urdu_letter) {
        bad.push(format!("name_ur has no Urdu letter: {ur:?}"));
    }
    if ur.chars().any(|c| c.is_ascii_alphabetic()) {
        bad.push(format!("name_ur has a Latin letter: {ur:?}"));
    }
    if !en.chars().any(|c| c.is_ascii_alphabetic()) {
        bad.push(format!("name_en has no Latin letter: {en:?}"));
    }
    if en.chars().any(urdu_letter) {
        bad.push(format!("name_en has an Urdu letter: {en:?}"));
    }
    if let Some(tag) = &row.tag {
        let ok = (1..=4).contains(&tag.chars().count())
            && tag.chars().all(|c| c.is_ascii_uppercase() || c == '.')
            && tag.chars().any(|c| c.is_ascii_uppercase());
        if !ok {
            bad.push(format!("tag {tag:?} breaks the tag rule"));
        }
    }
    for (label, phone) in [("phone1", &row.phone1), ("phone2", &row.phone2)] {
        if !phone.is_empty()
            && !(phone.len() == 11
                && phone.starts_with("03")
                && phone.bytes().all(|b| b.is_ascii_digit()))
        {
            bad.push(format!("{label} {phone:?} is not 11 digits starting 03"));
        }
    }
    // A note is free text (Code Standards 4.2): up to 500 characters, no control characters, and
    // no bidi override or isolate (R02-3). A zero-width mark in a note is allowed by the rule.
    let bad_note_char =
        |c: char| c.is_control() || matches!(c, '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}');
    if row.note.chars().count() > 500 || row.note.chars().any(bad_note_char) {
        bad.push(format!("note breaks the free text rule: {:?}", row.note));
    }
    bad
}

// ---------------------------------------------------------------------------------------------
// 1. The table
// ---------------------------------------------------------------------------------------------

/// A case of the table: the fields typed, and what must happen.
struct Case {
    label: &'static str,
    ur: String,
    en: String,
    tag: &'static str,
    phone: &'static str,
    note: String,
    /// `Ok(())` saves; `Err((field, code))` is refused at that field with that code.
    want: Result<(), (&'static str, ErrorCode)>,
}

fn case(
    label: &'static str,
    ur: impl Into<String>,
    en: impl Into<String>,
    want: Result<(), (&'static str, ErrorCode)>,
) -> Case {
    Case {
        label,
        ur: ur.into(),
        en: en.into(),
        tag: "",
        phone: "",
        note: String::new(),
        want,
    }
}

fn cases() -> Vec<Case> {
    let mut c = vec![
        case(
            "100 and 100 characters",
            "ا".repeat(100),
            "a".repeat(100),
            Ok(()),
        ),
        case(
            "100 characters with spaces round them",
            format!("  {}  ", "ا".repeat(100)),
            format!("  {}  ", "a".repeat(100)),
            Ok(()),
        ),
        case(
            "101 Urdu characters",
            "ا".repeat(101),
            "Ali",
            Err(("name_ur", ErrorCode::ErrNameTooLong)),
        ),
        case(
            "101 English characters",
            "علی",
            "a".repeat(101),
            Err(("name_en", ErrorCode::ErrNameTooLong)),
        ),
        case(
            "only spaces in the Urdu name",
            "     ",
            "Ali",
            Err(("name_ur", ErrorCode::ErrRequired)),
        ),
        case(
            "only spaces in the English name",
            "علی",
            "     ",
            Err(("name_en", ErrorCode::ErrRequired)),
        ),
        case(
            "only a zero-width joiner",
            "\u{200d}",
            "Ali",
            Err(("name_ur", ErrorCode::ErrRequired)),
        ),
        case(
            "Latin letters in the Urdu name",
            "Ahmed احمد",
            "Ahmed",
            Err(("name_ur", ErrorCode::ErrNameUrScript)),
        ),
        case(
            "Urdu letters in the English name",
            "احمد",
            "Ahmed احمد",
            Err(("name_en", ErrorCode::ErrNameEnScript)),
        ),
        case(
            "emoji in the Urdu name",
            "احمد😀",
            "Ahmed",
            Err(("name_ur", ErrorCode::ErrNameUrScript)),
        ),
        case(
            "emoji in the English name",
            "احمد",
            "Ahmed😀",
            Err(("name_en", ErrorCode::ErrNameEnScript)),
        ),
        case(
            "NUL in the Urdu name",
            "اح\u{0}مد",
            "Ahmed",
            Err(("name_ur", ErrorCode::ErrControlChars)),
        ),
        case(
            "a bell in the English name",
            "احمد",
            "Ah\u{7}med",
            Err(("name_en", ErrorCode::ErrControlChars)),
        ),
        case(
            "a newline in the English name",
            "احمد",
            "Ah\nmed",
            Err(("name_en", ErrorCode::ErrControlChars)),
        ),
        case(
            "a sign the name does not allow",
            "احمد@",
            "Ahmed",
            Err(("name_ur", ErrorCode::ErrNameUrScript)),
        ),
        case(
            "zero-width space is removed",
            "اح\u{200b}مد",
            "Ahmed",
            Ok(()),
        ),
        case(
            "ZWJ and ZWNJ are removed",
            "اح\u{200d}م\u{200c}د",
            "Ahmed",
            Ok(()),
        ),
        case(
            "digits and the allowed signs",
            "احمد (کراچی) & برادرز / 2",
            "Ahmed (Karachi) & Bros. / 2",
            Ok(()),
        ),
        case(
            "the Arabic letter variants are Urdu letters",
            "سليم موسى هاشم زكوۃ",
            "Saleem",
            Ok(()),
        ),
    ];
    // Two errors at once come back together (Code Standards 4.3).
    c.push(case(
        "both names over 100",
        "ا".repeat(101),
        "a".repeat(101),
        Err(("name_ur", ErrorCode::ErrNameTooLong)),
    ));
    let mut tagged = |label, tag: &'static str, want| {
        let mut x = case(label, "علی", "Ali", want);
        x.tag = tag;
        c.push(x);
    };
    tagged("tag K", "K", Ok(()));
    tagged("tag in lower case", "k", Ok(()));
    tagged("tag of 4 characters", "ABCD", Ok(()));
    tagged("tag of dots and letters", "D.P", Ok(()));
    tagged(
        "tag of 5 characters",
        "ABCDE",
        Err(("tag", ErrorCode::ErrTagInvalid)),
    );
    tagged(
        "tag with a digit",
        "1A",
        Err(("tag", ErrorCode::ErrTagInvalid)),
    );
    tagged("tag in Urdu", "ب", Err(("tag", ErrorCode::ErrTagInvalid)));
    tagged(
        "tag with a space inside",
        "A B",
        Err(("tag", ErrorCode::ErrTagInvalid)),
    );
    tagged(
        "tag of dots only",
        "..",
        Err(("tag", ErrorCode::ErrTagInvalid)),
    );
    tagged(
        "tag with a zero-width space",
        "K\u{200b}",
        Err(("tag", ErrorCode::ErrTagInvalid)),
    );
    tagged(
        "tag with an emoji",
        "K😀",
        Err(("tag", ErrorCode::ErrTagInvalid)),
    );
    let mut phoned = |label, phone: &'static str, want| {
        let mut x = case(label, "علی", "Ali", want);
        x.phone = phone;
        c.push(x);
    };
    phoned("phone 03001234567", "03001234567", Ok(()));
    phoned("phone with a hyphen", "0300-1234567", Ok(()));
    phoned("phone in Urdu digits", "۰۳۰۰۱۲۳۴۵۶۷", Ok(()));
    phoned(
        "phone with a plus",
        "+923001234567",
        Err(("phone1", ErrorCode::ErrPhoneInvalid)),
    );
    phoned(
        "phone too short",
        "1234",
        Err(("phone1", ErrorCode::ErrPhoneInvalid)),
    );
    phoned(
        "phone with a space",
        "0300 1234567",
        Err(("phone1", ErrorCode::ErrPhoneInvalid)),
    );
    phoned(
        "Arabic-Indic digits with a hyphen",
        "٠٣٠٠-١٢٣٤٥٦٧",
        Err(("phone1", ErrorCode::ErrPhoneInvalid)),
    );
    let mut noted = |label, note: String, want| {
        let mut x = case(label, "علی", "Ali", want);
        x.note = note;
        c.push(x);
    };
    noted("note of 500 characters", "ٹ".repeat(500), Ok(()));
    noted(
        "note of 501 characters",
        "ٹ".repeat(501),
        Err(("note", ErrorCode::ErrTextTooLong)),
    );
    noted("note with Urdu and emoji", "ٹھیک ہے 😀 ✓".into(), Ok(()));
    noted(
        "note with a right-to-left override",
        "ab\u{202e}cd".into(),
        Err(("note", ErrorCode::ErrControlChars)),
    );
    noted(
        "note with a NUL",
        "a\u{0}b".into(),
        Err(("note", ErrorCode::ErrControlChars)),
    );
    c
}

#[test]
fn names_tags_phones_and_notes_at_their_edges_are_saved_clean_or_refused_at_their_field() {
    let w = world();
    let probe = Probe(raw(&w.dir));
    let mut failures: Vec<String> = Vec::new();
    for case in cases() {
        let mut input = form(AccountType::Buyer, &case.ur, &case.en);
        input.tag = case.tag.into();
        input.phone1 = case.phone.into();
        input.note = case.note.clone();
        let before = probe.version();
        let answer = try_save(&w, input);
        match (&case.want, answer) {
            (Ok(()), Ok(row)) => {
                let bad = rule_breaks(&row);
                if !bad.is_empty() {
                    failures.push(format!(
                        "{}: saved but breaks the rules: {bad:?}",
                        case.label
                    ));
                }
                // The same row comes back from a read.
                match accounts::get(&w.env.ctx, row.id) {
                    Ok(read)
                        if read.name_ur == row.name_ur
                            && read.name_en == row.name_en
                            && read.tag == row.tag => {}
                    other => failures.push(format!("{}: read back differs: {other:?}", case.label)),
                }
            }
            (Ok(()), Err(error)) => {
                failures.push(format!(
                    "{}: should save, was refused: {:?}",
                    case.label,
                    codes(&error)
                ));
            }
            (Err((field, code)), Err(error)) => {
                let got = codes(&error);
                if !got.contains(&(*code, (*field).to_owned())) {
                    failures.push(format!(
                        "{}: wanted {code:?} at {field}, got {got:?}",
                        case.label
                    ));
                }
                if probe.version() != before {
                    failures.push(format!(
                        "{}: refused, but the file changed (half-save)",
                        case.label
                    ));
                }
            }
            (Err(_), Ok(row)) => {
                failures.push(format!("{}: bad input was SAVED as {row:?}", case.label));
            }
        }
    }
    assert!(failures.is_empty(), "\n{}", failures.join("\n"));
}

#[test]
fn two_bad_fields_come_back_together_and_nothing_is_saved() {
    // Code Standards 4.3: several errors at once, each at its field.
    let w = world();
    let probe = Probe(raw(&w.dir));
    let mut input = form(AccountType::Seller, &"ا".repeat(101), &"a".repeat(101));
    input.tag = "ABCDE".into();
    input.phone1 = "12".into();
    let before = probe.version();
    let error = try_save(&w, input).unwrap_err();
    let got = codes(&error);
    for want in [
        (ErrorCode::ErrNameTooLong, "name_ur"),
        (ErrorCode::ErrNameTooLong, "name_en"),
        (ErrorCode::ErrTagInvalid, "tag"),
        (ErrorCode::ErrPhoneInvalid, "phone1"),
    ] {
        assert!(
            got.contains(&(want.0, want.1.to_owned())),
            "{want:?} missing from {got:?}"
        );
    }
    assert_eq!(probe.version(), before);
}

// ---------------------------------------------------------------------------------------------
// 2. Letter variants (NFR-URD-18)
// ---------------------------------------------------------------------------------------------

fn find(w: &World, scope: AccountScope, text: &str) -> Vec<AccountHit> {
    search::search_accounts(
        &w.env.ctx,
        &w.owner,
        &SearchAccountsInput {
            include_dead: None,
            scope,
            control_id: None,
            text: text.to_owned(),
            limit: 20,
            offset: 0,
            with_chukta: None,
        },
    )
    .unwrap()
}

#[test]
fn a_name_in_one_letter_form_is_found_by_the_others() {
    let w = world();
    // (the name as typed and saved, English name, the ways it is looked for)
    let table: &[(&str, &str, &[&str])] = &[
        // kaf: Urdu ک, Arabic ك. yeh: Urdu ی, Arabic ي. heh: Urdu ہ, Arabic ه.
        (
            "اکرم",
            "Akramm",
            &["اكرم", "اکَرم", "ا\u{200d}کرم", "ا\u{200c}کرم", "اک"],
        ),
        ("سليم", "Saleemm", &["سلیم", "سليم", "سلی"]),
        ("موسى", "Moosaa", &["موسی", "موسى"]),
        ("هاشم", "Hashimm", &["ہاشم", "هاشم", "ہاش"]),
        ("زكوۃ", "Zakaatt", &["زکوہ", "زكوة", "زکوۃ"]),
    ];
    for (saved, english, queries) in table {
        let row = try_save(&w, form(AccountType::Buyer, saved, english))
            .unwrap_or_else(|e| panic!("{saved}: {:?}", codes(&e)));
        for query in *queries {
            for scope in [AccountScope::Buyer, AccountScope::Any] {
                let hits = find(&w, scope, query);
                assert!(
                    hits.iter().any(|h| h.id == row.id),
                    "{saved:?} saved, {query:?} typed in {scope:?}: not found (got {:?})",
                    hits.iter().map(|h| h.name_ur.clone()).collect::<Vec<_>>()
                );
            }
        }
    }
    // English: case and a near spelling.
    let row = try_save(
        &w,
        form(AccountType::Buyer, "ارشد ملتانی", "Arshad Multani"),
    )
    .unwrap();
    for query in ["ARSHAD", "arshad", "arshd", "multani", "arshad multani"] {
        let hits = find(&w, AccountScope::Buyer, query);
        assert!(hits.iter().any(|h| h.id == row.id), "{query:?}");
    }
}

#[test]
fn the_same_name_with_two_tags_is_two_accounts_and_both_are_found() {
    // E38: the tag tells them apart.
    let w = world();
    let mut a = form(AccountType::Buyer, "اسلم", "Aslam");
    a.tag = "BA".into();
    let plain = form(AccountType::Buyer, "اسلم", "Aslam");
    let with_tag = try_save(&w, a).unwrap();
    let without = try_save(&w, plain).unwrap();
    assert_ne!(with_tag.id, without.id);
    let hits = find(&w, AccountScope::Buyer, "aslam");
    assert!(hits.iter().any(|h| h.id == with_tag.id));
    assert!(hits.iter().any(|h| h.id == without.id));
    let tagged = hits.iter().find(|h| h.id == with_tag.id).unwrap();
    assert_eq!(tagged.tag.as_deref(), Some("BA"));
}

// ---------------------------------------------------------------------------------------------
// 3. Property test: accounts::save
// ---------------------------------------------------------------------------------------------

fn alphabet(chars: &'static [char]) -> impl Strategy<Value = char> {
    prop::sample::select(chars)
}

const URDU: &[char] = &[
    'ا', 'ب', 'پ', 'ت', 'ٹ', 'ج', 'چ', 'ح', 'خ', 'د', 'ر', 'ز', 'س', 'ش', 'ک', 'گ', 'ل', 'م', 'ن',
    'و', 'ہ', 'ھ', 'ی', 'ے', 'ء', 'آ',
];
const MARKS: &[char] = &[
    ' ', ' ', ' ', '.', '-', '\'', '(', ')', '/', '&', '1', '0', '9', '۶', '٦', '@', '%', '\u{0}',
    '\u{7}', '\n', '\t', '\u{200b}', '\u{200c}', '\u{200d}', '\u{200e}', '\u{200f}', '\u{202e}',
    '\u{2066}', '\u{feff}', '\u{061c}', '😀', 'é', '\u{301}', '\u{064e}',
];
const VARIANTS: &[char] = &['ي', 'ى', 'ك', 'ه', 'ۃ', 'ة', 'ې', 'ٹ'];
const LATIN: &[char] = &['a', 'b', 'k', 'r', 'A', 'Z', 'm', 'h', 'i', 's', 'z', 'q'];

fn lengths() -> impl Strategy<Value = usize> {
    prop_oneof![0_usize..12, 95_usize..106, 100_usize..130]
}

fn text_strategy() -> impl Strategy<Value = String> {
    lengths().prop_flat_map(|n| {
        let one = prop_oneof![
            6 => alphabet(URDU),
            3 => alphabet(MARKS),
            1 => alphabet(VARIANTS),
            2 => alphabet(LATIN),
        ];
        prop::collection::vec(one, n).prop_map(|v| v.into_iter().collect::<String>())
    })
}

fn english_strategy() -> impl Strategy<Value = String> {
    lengths().prop_flat_map(|n| {
        let one = prop_oneof![
            8 => alphabet(LATIN),
            3 => alphabet(MARKS),
            1 => alphabet(URDU),
        ];
        prop::collection::vec(one, n).prop_map(|v| v.into_iter().collect::<String>())
    })
}

fn short_strategy(chars: &'static [char]) -> impl Strategy<Value = String> {
    prop::collection::vec(alphabet(chars), 0..8).prop_map(|v| v.into_iter().collect::<String>())
}

const TAG_CHARS: &[char] = &['K', 'k', 'A', 'B', '.', '1', ' ', 'ب', '\u{200b}', 'Z'];
const PHONE_CHARS: &[char] = &[
    '0', '3', '1', '2', '4', '5', '-', '۰', '٠', ' ', '+', 'a', '\u{0}',
];

#[test]
fn any_text_in_any_field_of_an_account_is_saved_clean_or_refused_with_nothing_written() {
    let w = world();
    let probe = Probe(raw(&w.dir));
    let mut runner = TestRunner::new(Config {
        cases: 220,
        failure_persistence: None,
        ..Config::default()
    });
    // Half the cases start from a clean value and only one or two fields are hostile, so the
    // property reaches the "saved" side as often as the "refused" side.
    let clean_urdu = prop::collection::vec(
        prop_oneof![9 => alphabet(URDU), 1 => Just(' ')],
        1_usize..=100,
    )
    .prop_map(|v| v.into_iter().collect::<String>());
    let clean_latin = prop::collection::vec(
        prop_oneof![9 => alphabet(LATIN), 1 => Just(' ')],
        1_usize..=100,
    )
    .prop_map(|v| v.into_iter().collect::<String>());
    let strategy = (
        prop_oneof![5 => clean_urdu, 5 => text_strategy()],
        prop_oneof![5 => clean_latin, 5 => english_strategy()],
        prop_oneof![5 => Just(String::new()), 2 => Just("k".to_owned()), 3 => short_strategy(TAG_CHARS)],
        prop_oneof![
            5 => Just(String::new()),
            2 => Just("0300-1234567".to_owned()),
            3 => prop::collection::vec(alphabet(PHONE_CHARS), 0..16).prop_map(|v| v.into_iter().collect::<String>()),
        ],
        prop_oneof![6 => Just(String::new()), 4 => text_strategy()],
        prop_oneof![8 => Just(String::new()), 2 => text_strategy()],
    );
    let saved = std::cell::Cell::new(0_usize);
    let refused = std::cell::Cell::new(0_usize);
    runner
        .run(&strategy, |(ur, en, tag, phone, note, father)| {
            let mut input = form(AccountType::Buyer, &ur, &en);
            input.tag = tag;
            input.phone1 = phone;
            input.note = note;
            input.father_or_firm_ur = father;
            let before = probe.version();
            match try_save(&w, input) {
                Ok(row) => {
                    saved.set(saved.get() + 1);
                    let bad = rule_breaks(&row);
                    prop_assert!(bad.is_empty(), "saved but breaks the rules: {bad:?}");
                    let read = accounts::get(&w.env.ctx, row.id).unwrap();
                    prop_assert_eq!(&read.name_ur, &row.name_ur);
                    prop_assert_eq!(&read.name_en, &row.name_en);
                    // Whatever is saved is found by its own English name and never panics a search.
                    let _ = search::search_accounts(
                        &w.env.ctx,
                        &w.owner,
                        &SearchAccountsInput {
                            include_dead: None,
                            scope: AccountScope::Buyer,
                            control_id: None,
                            text: row.name_en.clone(),
                            limit: 20,
                            offset: 0,
                            with_chukta: None,
                        },
                    )
                    .map_err(|e| TestCaseError::fail(format!("search failed: {e:?}")))?;
                }
                Err(error) => {
                    refused.set(refused.get() + 1);
                    prop_assert!(
                        error.code() != ErrorCode::ErrInternal,
                        "an internal error: {error:?}"
                    );
                    prop_assert_eq!(
                        probe.version(),
                        before,
                        "a refused save committed something"
                    );
                    if let ServiceError::Rejected(items) = &error {
                        prop_assert!(
                            items.iter().all(|i| i.field.is_some()
                                || i.code == ErrorCode::ErrSpellingUnconfirmed),
                            "an error without a field: {items:?}"
                        );
                    }
                }
            }
            Ok(())
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (saved, refused) = (saved.get(), refused.get());
    println!("accounts::save: {saved} saved clean, {refused} refused with nothing written");
    assert!(
        saved > 5 && refused > 5,
        "the strategy must reach both sides"
    );
}

// ---------------------------------------------------------------------------------------------
// 4. Property test: a whole بکری draft
// ---------------------------------------------------------------------------------------------

fn hostile_strings() -> Vec<String> {
    vec![
        "".into(),
        " ".into(),
        "0".into(),
        "-1".into(),
        "850.50".into(),
        "۶۰۰".into(),
        "٦٠٠".into(),
        "1,00,000".into(),
        "10,00,00,00,000".into(),
        "10,00,00,00,001".into(),
        "9223372036854775807".into(),
        "99999999999999999999999".into(),
        "1e5".into(),
        "20+25".into(),
        "10-20".into(),
        "20+".into(),
        "31-02-2026".into(),
        "00-00-0000".into(),
        "10-10-2099".into(),
        "09-10-2026".into(),
        "2026-10-10".into(),
        "' OR 1=1 --".into(),
        "<script>alert(1)</script>".into(),
        "\u{0}".into(),
        "1\u{200b}0".into(),
        "\u{202e}12".into(),
        "😀".into(),
        "ا".repeat(101),
        "x".repeat(5_000),
    ]
}

fn string_paths(value: &Value, here: &mut Vec<String>, out: &mut Vec<Vec<String>>) {
    match value {
        Value::Object(map) => {
            for (key, child) in map {
                here.push(key.clone());
                string_paths(child, here, out);
                here.pop();
            }
        }
        Value::Array(items) => {
            for (index, child) in items.iter().enumerate() {
                here.push(index.to_string());
                string_paths(child, here, out);
                here.pop();
            }
        }
        Value::String(_) | Value::Number(_) | Value::Null => out.push(here.clone()),
        Value::Bool(_) => {}
    }
}

fn set_at(value: &mut Value, path: &[String], new: Value) {
    let Some((first, rest)) = path.split_first() else {
        *value = new;
        return;
    };
    let child = match value {
        Value::Object(map) => map.get_mut(first),
        Value::Array(items) => first.parse::<usize>().ok().and_then(|i| items.get_mut(i)),
        _ => None,
    };
    if let Some(child) = child {
        set_at(child, rest, new);
    }
}

#[test]
fn a_draft_with_hostile_leaves_is_refused_whole_or_saved_exactly_as_previewed() {
    let w = world();
    let probe = Probe(raw(&w.dir));
    let base = serde_json::json!({ "draft": event1(&w), "confirm_cash_below_zero": true });
    let mut paths = Vec::new();
    string_paths(&base["draft"], &mut Vec::new(), &mut paths);
    assert!(paths.len() > 30, "only {} leaves", paths.len());
    let strings = hostile_strings();
    let numbers: Vec<Value> = vec![
        serde_json::json!(0),
        serde_json::json!(-1),
        serde_json::json!(i64::MAX),
        serde_json::json!(i64::MIN),
        serde_json::json!(999_999),
        Value::Null,
    ];
    let mut runner = TestRunner::new(Config {
        cases: 200,
        failure_persistence: None,
        ..Config::default()
    });
    let pick = (
        0..paths.len(),
        0..strings.len(),
        0..numbers.len(),
        any::<bool>(),
    );
    let saved = std::cell::Cell::new(0_usize);
    let refused = std::cell::Cell::new(0_usize);
    runner
        .run(&(pick.clone(), pick), |(a, b)| {
            let mut doc = base.clone();
            for (path, s, n, as_number) in [a, b] {
                let new = if as_number { numbers[n].clone() } else { Value::String(strings[s].clone()) };
                set_at(&mut doc["draft"], &paths[path], new);
            }
            // A value of the wrong type never reaches the service (the IPC layer refuses it).
            let Ok(input) = serde_json::from_value::<BikriSaveInput>(doc) else {
                return Ok(());
            };
            let preview = match bikri::preview(&w.env.ctx, &input.draft) {
                Ok(p) => p,
                Err(error) => {
                    prop_assert!(error.code() != ErrorCode::ErrInternal, "preview: {error:?}");
                    return Ok(());
                }
            };
            let before = probe.version();
            let answer = bikri::save(&w.env.ctx, &w.owner, &input);
            if !preview.errors.is_empty() {
                prop_assert!(answer.is_err(), "a draft with errors was saved: {:?}", preview.errors);
            }
            match answer {
                Ok(saved_draft) => {
                    saved.set(saved.get() + 1);
                    prop_assert_eq!(&saved_draft.preview, &preview, "the save is not the preview (B5)");
                    let (dr, cr): (i64, i64) = probe
                        .0
                        .query_row(
                            "SELECT COALESCE(SUM(dr_paisa),0), COALESCE(SUM(cr_paisa),0) FROM journal_lines",
                            [],
                            |r| Ok((r.get(0)?, r.get(1)?)),
                        )
                        .unwrap();
                    prop_assert_eq!(dr, cr, "the books do not balance");
                }
                Err(error) => {
                    refused.set(refused.get() + 1);
                    prop_assert!(error.code() != ErrorCode::ErrInternal, "save: {error:?}");
                    prop_assert_eq!(probe.version(), before, "a refused save committed something");
                }
            }
            Ok(())
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (saved, refused) = (saved.get(), refused.get());
    println!("bikri draft: {saved} saved as previewed, {refused} refused with nothing written");
    let drafts: i64 = probe
        .0
        .query_row(
            "SELECT COUNT(*) FROM journal_entries WHERE status <> 'posted'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(drafts, 0);
    assert!(refused > 10, "the strategy must reach refusals");
}
