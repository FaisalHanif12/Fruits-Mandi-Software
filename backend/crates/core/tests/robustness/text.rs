//! Search keys, name checks and free text on hostile Unicode (Tech Stack 4.3, NFR-URD-18,
//! Code Standards 4.2, B26, B28). Rules: no panic on any input; keys are stable (building a
//! key twice gives the same key); invisible marks never change a key; hostile names are
//! refused with the right code; injection text is never accepted as a name.

use std::time::Instant;

use mandi_core::error::ErrorCode;
use mandi_core::text::{UrduCollator, normalize_urdu, search_en, search_phonetic, search_ur};
use mandi_core::validate;
use proptest::prelude::*;

use super::{hostile_strings, no_panic};

/// Bidi controls that copy and paste puts around text: embeddings, overrides, isolates.
const BIDI_CONTROLS: [char; 9] = [
    '\u{202A}', '\u{202B}', '\u{202C}', '\u{202D}', '\u{202E}', '\u{2066}', '\u{2067}', '\u{2068}',
    '\u{2069}',
];

#[test]
fn keys_survive_every_hostile_string() {
    let collator = UrduCollator::new().unwrap();
    for s in hostile_strings() {
        let short: String = s.chars().take(20).collect();
        no_panic(&short, || {
            let ur = search_ur(&s);
            let en = search_en(&s);
            let ph = search_phonetic(&s);
            assert!(ph.bytes().all(|b| b.is_ascii_lowercase()), "{short:?}");
            assert!(!ur.starts_with(' ') && !ur.ends_with(' '));
            assert!(!en.contains("  "));
            let _ = collator.compare(&s, "اکرم");
            let _ = validate::name_ur("n", &s);
            let _ = validate::name_en("n", &s);
            let _ = validate::free_text("t", &s);
            let _ = validate::phone("p", &s);
            let _ = validate::account_code("c", &s);
        });
    }
}

#[test]
fn long_inputs_stay_fast() {
    // 100,000 combining marks on one letter: canonical reordering must not be quadratic.
    let zalgo = "a".to_owned() + &"\u{0301}\u{0316}".repeat(50_000);
    let shadda = "ب".to_owned() + &"\u{0651}\u{064E}".repeat(50_000);
    let start = Instant::now();
    let _ = search_en(&zalgo);
    assert_eq!(normalize_urdu(&shadda), "ب");
    let _ = validate::name_ur("n", &shadda);
    // Debug build. Release is many times faster.
    assert!(
        start.elapsed().as_millis() < 3_000,
        "took {:?}",
        start.elapsed()
    );
}

#[test]
fn invisible_marks_never_change_an_urdu_key() {
    let plain = search_ur("اکرم ٹول ٹیکس");
    for mark in [
        '\u{200B}', '\u{200C}', '\u{200D}', '\u{200E}', '\u{200F}', '\u{2060}', '\u{FEFF}',
        '\u{061C}',
    ] {
        let marked = format!("{mark}اکرم{mark} ٹول ٹیکس{mark}");
        assert_eq!(search_ur(&marked), plain, "U+{:04X}", u32::from(mark));
    }
    // Arabic letters, diacritics and spacing still map to the same key.
    assert_eq!(search_ur("  اكرم\u{064E}   ٹول\t ٹيكس "), plain);
}

#[test]
fn bidi_controls_never_change_a_key() {
    let plain_ur = search_ur("اکرم ٹول ٹیکس");
    let plain_en = search_en("Akram Toll Tax");
    for mark in BIDI_CONTROLS {
        let ur = format!("{mark}اکرم ٹول ٹیکس\u{202C}");
        let en = format!("{mark}Akram Toll Tax\u{202C}");
        assert_eq!(search_ur(&ur), plain_ur, "U+{:04X}", u32::from(mark));
        assert_eq!(search_en(&en), plain_en, "U+{:04X}", u32::from(mark));
    }
}

#[test]
fn hostile_names_are_refused_with_the_right_code() {
    let ur = |s: &str| validate::name_ur("name_ur", s).map_err(|e| e.code);
    let en = |s: &str| validate::name_en("name_en", s).map_err(|e| e.code);
    // Injection and path text is never a valid name (it stays plain text in notes).
    for s in [
        "' OR 1=1 --",
        "Robert'); DROP TABLE accounts;--",
        "<script>alert(1)</script>",
        "..\\..\\windows",
        "%s%n",
        "Akram\u{202E}xaT",
        "Akram 🍌",
        "Akram ٹول",
        "Ákram",
    ] {
        assert_eq!(en(s), Err(ErrorCode::ErrNameEnScript), "{s:?}");
    }
    for s in [
        "اکرم; DROP TABLE",
        "اکرم <b>",
        "اکرم akram",
        "اکرم 🍌",
        "\u{202E}اکرم",
    ] {
        assert_eq!(ur(s), Err(ErrorCode::ErrNameUrScript), "{s:?}");
    }
    assert_eq!(ur(""), Err(ErrorCode::ErrRequired));
    assert_eq!(
        ur(" \u{200B}\u{200F}\u{FEFF} "),
        Err(ErrorCode::ErrRequired)
    );
    assert_eq!(en("\u{200D}"), Err(ErrorCode::ErrRequired));
    assert_eq!(ur("اکرم\u{0000}"), Err(ErrorCode::ErrControlChars));
    assert_eq!(en("Akram\tToll"), Err(ErrorCode::ErrControlChars));
    assert_eq!(ur(&"ا".repeat(101)), Err(ErrorCode::ErrNameTooLong));
    assert_eq!(ur(&"ا".repeat(100)), Ok("ا".repeat(100)));
    assert_eq!(en(&"a".repeat(101)), Err(ErrorCode::ErrNameTooLong));
    // Zero-width marks are removed from a saved name.
    assert_eq!(ur("اک\u{200C}رم"), Ok("اکرم".to_owned()));
    assert_eq!(en("\u{FEFF}Akram\u{200B}"), Ok("Akram".to_owned()));
}

#[test]
fn free_text_limits() {
    let t = |s: &str| validate::free_text("note", s).map_err(|e| e.code);
    assert_eq!(t(&"ن".repeat(500)).map(|s| s.chars().count()), Ok(500));
    assert_eq!(t(&"ن".repeat(501)), Err(ErrorCode::ErrTextTooLong));
    assert_eq!(t("line\r\nbreak"), Err(ErrorCode::ErrControlChars));
    assert_eq!(t("bell\u{0007}"), Err(ErrorCode::ErrControlChars));
    // Injection text is kept as plain text: the database layer binds it (Code Standards B14).
    assert_eq!(t("' OR 1=1 --"), Ok("' OR 1=1 --".to_owned()));
    assert_eq!(t("  "), Ok(String::new()));
}

#[test]
fn free_text_drops_bidi_overrides() {
    // A note "رقم 1200 \u{202E}0021" would print "رقم 1200 1200" style reversed digits.
    for mark in BIDI_CONTROLS {
        let note = format!("رقم {mark}0021");
        let kept = validate::free_text("note", &note).unwrap_or_default();
        assert!(!kept.contains(mark), "U+{:04X} kept", u32::from(mark));
    }
}

#[test]
fn codes_and_phones_refuse_hostile_text() {
    let code = |s: &str| validate::account_code("code", s).map_err(|e| e.code);
    for s in [
        "' OR 1=1 --",
        "..\\..",
        "1e3",
        "-1",
        "+1",
        "12345",
        "1234567",
        "1234567890",
        "0x10",
        "１",
    ] {
        assert_eq!(code(s).err(), Some(ErrorCode::ErrCodeInvalid), "{s:?}");
    }
    assert_eq!(code("").err(), Some(ErrorCode::ErrRequired));
    assert!(code("65536").is_err(), "5 digits is not a short code");
    assert_eq!(
        code("۰۳۰۴۰۰۰۱"),
        Ok(validate::AccountCodeInput::Full("03040001".to_owned()))
    );
    let phone = |s: &str| validate::phone("phone", s).map_err(|e| e.code);
    assert_eq!(phone("0300-0000000"), Ok("03000000000".to_owned()));
    assert_eq!(phone("۰۳۰۰۰۰۰۰۰۰۰"), Ok("03000000000".to_owned()));
    for s in [
        "0300 0000000",
        "+923000000000",
        "0300-000000",
        "03000000000\u{0000}",
        "1300-0000000",
        "0300-00000a0",
    ] {
        assert_eq!(phone(s), Err(ErrorCode::ErrPhoneInvalid), "{s:?}");
    }
}

fn hostile_char() -> impl Strategy<Value = char> {
    prop_oneof![
        prop::char::range('\u{0600}', '\u{06FF}'),
        prop::char::range('\u{064B}', '\u{065F}'),
        prop::char::range('\u{0300}', '\u{036F}'),
        prop::char::range('a', 'z'),
        prop::char::range('A', 'Z'),
        prop::sample::select(vec![
            ' ', '\t', '\u{00A0}', '\u{200B}', '\u{200C}', '\u{200D}', '\u{200E}', '\u{200F}',
            '\u{2060}', '\u{FEFF}', '\u{061C}', '\u{202E}', '\u{2066}', 'ي', 'ى', 'ك', 'ه', 'ۃ',
            'ھ', 'é', 'İ', 'ß', '🍌',
        ]),
        any::<char>(),
    ]
}

fn urdu_char() -> impl Strategy<Value = char> {
    prop_oneof![
        prop::char::range('\u{0621}', '\u{064A}'),
        prop::char::range('\u{064B}', '\u{065F}'),
        prop::char::range('\u{0670}', '\u{06D3}'),
        prop::char::range('\u{06F0}', '\u{06F9}'),
        prop::sample::select(vec![
            ' ', '\u{200B}', '\u{200C}', '\u{200D}', '\u{200E}', '\u{200F}', '\u{2060}',
            '\u{FEFF}', '\u{061C}', '.', '-', '(', ')', '&', '/', '\'',
        ]),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 10_000, ..ProptestConfig::default() })]

    // A key built from a key must be the same key. Otherwise the key stored for a name and
    // the key built from a search for the same text can differ.

    /// The real-world case: Urdu letters, Urdu diacritics, the letter variants, spaces and
    /// invisible marks (the only things a saved Urdu name or an Urdu search can hold).
    #[test]
    fn urdu_key_is_stable_on_urdu_text(chars in prop::collection::vec(urdu_char(), 0..40)) {
        let s: String = chars.into_iter().collect();
        let ur = search_ur(&s);
        prop_assert_eq!(search_ur(&ur), ur.clone());
    }

    #[test]
    fn phonetic_key_is_stable(chars in prop::collection::vec(hostile_char(), 0..40)) {
        let s: String = chars.into_iter().collect();
        let ph = search_phonetic(&s);
        prop_assert_eq!(search_phonetic(&ph), ph.clone());
    }
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 10_000, ..ProptestConfig::default() })]

    #[test]
    fn urdu_key_is_stable(chars in prop::collection::vec(hostile_char(), 0..40)) {
        let s: String = chars.into_iter().collect();
        let ur = search_ur(&s);
        prop_assert_eq!(search_ur(&ur), ur.clone());
    }

    #[test]
    fn english_key_is_stable(chars in prop::collection::vec(hostile_char(), 0..40)) {
        let s: String = chars.into_iter().collect();
        let en = search_en(&s);
        prop_assert_eq!(search_en(&en), en.clone());
    }
}

#[test]
fn key_examples_of_r02_4() {
    // Fixed example of finding R02-4 (fixed by backend). 'İ' lower-cases to "i" + U+0307
    // (combining dot, class 230), placed after U+064B (class 27). The key must already be in
    // canonical order, so building a key from a key changes nothing.
    let once = search_en("\u{0130}\u{064B}");
    assert_eq!(search_en(&once), once);
    // Same for the Urdu key with a Latin mark: "e" + U+0657 + U+0300.
    let once = search_ur("e\u{0657}\u{0300}");
    assert_eq!(search_ur(&once), once);
}
