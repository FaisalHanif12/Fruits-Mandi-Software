//! Field checks of Code Standards 4.2 and the error shape of 4.3.
#![cfg(test)]

use mandi_core::dates::BusinessDate;
use mandi_core::error::{ErrorCode, ErrorValue, FieldError};
use mandi_core::money::{Bps, Grams, Paisa, Qty};
use mandi_core::validate::{self, AccountCodeInput, FieldErrors};

fn today() -> BusinessDate {
    BusinessDate::from_ymd(2026, 10, 10).unwrap()
}

fn code<T: std::fmt::Debug>(r: Result<T, FieldError>) -> ErrorCode {
    r.unwrap_err().code
}

#[test]
fn money_fields() {
    assert_eq!(
        validate::money("amount", "1,200"),
        Ok(Paisa::from_rupees(1_200).unwrap())
    );
    assert_eq!(
        validate::money("amount", "20+25"),
        Ok(Paisa::from_rupees(45).unwrap())
    );
    assert_eq!(validate::money("amount", "0"), Ok(Paisa::ZERO));
    let e = validate::money("lots[0].cash", "850.50").unwrap_err();
    assert_eq!(
        e,
        FieldError::new(ErrorCode::ErrDecimalsNotAllowed, "lots[0].cash")
    );
    let e = validate::money("amount", "10,00,00,00,001").unwrap_err();
    assert_eq!(e.code, ErrorCode::ErrAmountTooLarge);
    assert_eq!(e.values["max"], ErrorValue::Number(10_000_000_000));
    assert_eq!(code(validate::money("amount", "")), ErrorCode::ErrRequired);
    assert_eq!(validate::money_optional("amount", "  "), Ok(None));
    assert_eq!(
        validate::money_optional("amount", "5"),
        Ok(Some(Paisa::from_rupees(5).unwrap()))
    );
    assert_eq!(
        code(validate::money_optional("amount", "x")),
        ErrorCode::ErrNotANumber
    );
}

#[test]
fn rate_must_be_more_than_zero() {
    assert_eq!(
        validate::rate("rate", "600"),
        Ok(Paisa::from_rupees(600).unwrap())
    );
    assert_eq!(
        code(validate::rate("rate", "0")),
        ErrorCode::ErrMustBeMoreThanZero
    );
    assert_eq!(
        code(validate::rate("rate", "5-5")),
        ErrorCode::ErrMustBeMoreThanZero
    );
    assert_eq!(
        code(validate::rate("rate", "-1")),
        ErrorCode::ErrNegativeNotAllowed
    );
}

#[test]
fn percent_qty_weight() {
    assert_eq!(validate::percent("pct", "6"), Ok(Bps::new(600).unwrap()));
    let e = validate::percent("pct", "2.255").unwrap_err();
    assert_eq!(e.values["max"], ErrorValue::Number(2));
    assert_eq!(validate::qty("qty", "20+25"), Ok(Qty::new(45).unwrap()));
    let e = validate::qty("lots[0].sales[1].qty", "2.5").unwrap_err();
    assert_eq!(
        e,
        FieldError::new(ErrorCode::ErrQtyNotWhole, "lots[0].sales[1].qty")
    );
    let e = validate::qty("qty", "1000001").unwrap_err();
    assert_eq!(e.values["max"], ErrorValue::Number(1_000_000));
    assert_eq!(
        validate::weight("w", "12.345"),
        Ok(Grams::new(12_345).unwrap())
    );
    let e = validate::weight("w", "1.2345").unwrap_err();
    assert_eq!(e.code, ErrorCode::ErrTooManyDecimals);
    assert_eq!(e.values["max"], ErrorValue::Number(3));
    let e = validate::weight("w", "100001").unwrap_err();
    assert_eq!(e.values["max"], ErrorValue::Number(100_000));
    assert_eq!(
        code(validate::weight("w", "0")),
        ErrorCode::ErrMustBeMoreThanZero
    );
}

#[test]
fn dates_and_open_days() {
    let closed = BusinessDate::from_ymd(2026, 10, 9);
    assert_eq!(validate::date("date", "10", today()), Ok(today()));
    assert_eq!(
        code(validate::date("date", "31-02-2026", today())),
        ErrorCode::ErrDateInvalid
    );
    assert_eq!(
        validate::posting_date("date", "10-10-2026", closed, today()),
        Ok(today())
    );
    let e = validate::posting_date("date", "9", closed, today()).unwrap_err();
    assert_eq!(e.code, ErrorCode::ErrDateClosed);
    assert_eq!(
        e.values["closed_up_to"],
        ErrorValue::Text("09-10-2026".to_owned())
    );
    let e = validate::posting_date("date", "11", closed, today()).unwrap_err();
    assert_eq!(e.code, ErrorCode::ErrDateInFuture);
    assert_eq!(e.values["today"], ErrorValue::Text("10-10-2026".to_owned()));
    assert_eq!(
        code(validate::posting_date("date", "x", closed, today())),
        ErrorCode::ErrDateInvalid
    );
}

#[test]
fn account_codes() {
    assert_eq!(
        validate::account_code("buyer", "0020"),
        Ok(AccountCodeInput::Full("0020".to_owned()))
    );
    assert_eq!(
        validate::account_code("seller", " 0010 "),
        Ok(AccountCodeInput::Full("0010".to_owned()))
    );
    for (text, serial) in [
        ("0", 0),
        ("1", 1),
        ("7", 7),
        ("9999", 9_999),
        ("1234567", 1_234_567),
        ("۱", 1),
    ] {
        match validate::account_code("buyer", text).unwrap() {
            AccountCodeInput::Short(s) => assert_eq!(s.serial(), serial, "{text}"),
            other => panic!("{text} gave {other:?}"),
        }
    }
    // A full code of the scheme (up to 10 digits) is a full code; a short code has no padding.
    assert_eq!(
        validate::account_code("c", "0029999999"),
        Ok(AccountCodeInput::Full("0029999999".to_owned()))
    );
    for text in [
        "0001",
        "01",
        "00",
        "0304000100",
        "1234567890",
        "12a",
        "-1",
        "1 2",
        "00212345678",
    ] {
        assert_eq!(
            code(validate::account_code("buyer", text)),
            ErrorCode::ErrCodeInvalid,
            "{text}"
        );
    }
    assert_eq!(
        code(validate::account_code("buyer", "")),
        ErrorCode::ErrRequired
    );
}

#[test]
fn urdu_names_fr_7_28() {
    assert_eq!(
        validate::name_ur("name_ur", "  ارشد ملتانی "),
        Ok("ارشد ملتانی".to_owned())
    );
    assert_eq!(
        validate::name_ur("name_ur", "ار\u{200C}شد"),
        Ok("ارشد".to_owned())
    );
    // NFC: alef + hamza above is stored as one letter.
    assert_eq!(
        validate::name_ur("name_ur", "\u{0627}\u{0654}حمد"),
        Ok("\u{0623}حمد".to_owned())
    );
    assert_eq!(
        code(validate::name_ur("name_ur", "Arshad")),
        ErrorCode::ErrNameUrScript
    );
    assert_eq!(
        code(validate::name_ur("name_ur", "ارشد x")),
        ErrorCode::ErrNameUrScript
    );
    assert_eq!(
        code(validate::name_ur("name_ur", "123")),
        ErrorCode::ErrNameUrScript
    );
    assert_eq!(
        code(validate::name_ur("name_ur", " \u{200B} ")),
        ErrorCode::ErrRequired
    );
    assert_eq!(
        code(validate::name_ur("name_ur", "ارشد\tملتانی")),
        ErrorCode::ErrControlChars
    );
    let long = "ا".repeat(101);
    let e = validate::name_ur("name_ur", &long).unwrap_err();
    assert_eq!(e.code, ErrorCode::ErrNameTooLong);
    assert_eq!(e.values["max"], ErrorValue::Number(100));
    assert!(validate::name_ur("name_ur", &"ا".repeat(100)).is_ok());
}

#[test]
fn english_names_fr_7_28() {
    assert_eq!(
        validate::name_en("name_en", " Arshad Multani "),
        Ok("Arshad Multani".to_owned())
    );
    assert_eq!(
        code(validate::name_en("name_en", "ارشد")),
        ErrorCode::ErrNameEnScript
    );
    assert_eq!(
        code(validate::name_en("name_en", "Arshad ا")),
        ErrorCode::ErrNameEnScript
    );
    assert_eq!(
        code(validate::name_en("name_en", "")),
        ErrorCode::ErrRequired
    );
    assert_eq!(
        code(validate::name_en("name_en", "A\nB")),
        ErrorCode::ErrControlChars
    );
}

#[test]
fn phones() {
    // Arabic-Indic digits without a hyphen keep their order and are read.
    assert_eq!(
        validate::phone("phone", "٠٣٠٠١٢٣٤٥٦٧"),
        Ok("03001234567".to_owned())
    );
    // With a hyphen they are refused: bidi would show the two digit runs in another order
    // (lead decision, same rule as sums). Urdu digits and ASCII digits keep their order.
    for text in [
        "٠٣٠٠-١٢٣٤٥٦٧",
        "٠٣٠٠-1234567",
        "0300-١٢٣٤٥٦٧",
        " \u{200E}٠٣٠٠-١٢٣٤٥٦٧",
    ] {
        assert_eq!(
            code(validate::phone("phone", text)),
            ErrorCode::ErrPhoneInvalid,
            "{text}"
        );
        assert_eq!(
            code(validate::phone_optional("phone", text)),
            ErrorCode::ErrPhoneInvalid,
            "{text}"
        );
    }
    assert_eq!(
        validate::phone("phone", "۰۳۰۰-۱۲۳۴۵۶۷"),
        Ok("03001234567".to_owned())
    );
    assert_eq!(
        validate::phone("phone", "03001234567"),
        Ok("03001234567".to_owned())
    );
    assert_eq!(
        validate::phone("phone", "0300-1234567"),
        Ok("03001234567".to_owned())
    );
    assert_eq!(
        validate::phone("phone", "۰۳۰۰۱۲۳۴۵۶۷"),
        Ok("03001234567".to_owned())
    );
    for text in [
        "0300123456",
        "04001234567",
        "030012345678",
        "030-01234567",
        "0300-123456a",
        "0300_1234567",
    ] {
        assert_eq!(
            code(validate::phone("phone", text)),
            ErrorCode::ErrPhoneInvalid,
            "{text}"
        );
    }
    assert_eq!(code(validate::phone("phone", " ")), ErrorCode::ErrRequired);
    assert_eq!(validate::phone_optional("phone", ""), Ok(None));
    assert_eq!(
        validate::phone_optional("phone", "0300-0000000"),
        Ok(Some("03000000000".to_owned()))
    );
}

/// Finding R02-5: the empty check comes after the invisible marks are removed, so an
/// optional field holding only a pasted mark is empty, not `ERR_REQUIRED`.
#[test]
fn optional_fields_with_only_invisible_marks_are_empty() {
    for blank in [
        "\u{200B}",
        "\u{200C}",
        "\u{200D}",
        "\u{200E}",
        "\u{2060}",
        "\u{FEFF}",
        " \u{200E} ",
        "\u{200B}\u{FEFF}\u{2060}",
    ] {
        assert_eq!(validate::money_optional("m", blank), Ok(None), "{blank:?}");
        assert_eq!(validate::phone_optional("p", blank), Ok(None), "{blank:?}");
    }
    // A mark that is refused in numbers (RLM, ALM, bidi controls) is not "empty".
    for mark in ["\u{200F}", "\u{061C}", "\u{202E}", "\u{2066}"] {
        assert_eq!(
            code(validate::money_optional("m", mark)),
            ErrorCode::ErrNotANumber,
            "{mark:?}"
        );
        assert_eq!(
            code(validate::phone_optional("p", mark)),
            ErrorCode::ErrPhoneInvalid,
            "{mark:?}"
        );
    }
    // A required field with only marks is still required.
    assert_eq!(
        code(validate::money("m", "\u{200B}")),
        ErrorCode::ErrRequired
    );
    assert_eq!(
        code(validate::phone("p", "\u{200B}")),
        ErrorCode::ErrRequired
    );
    // Marks around real values do not stop them being read.
    assert_eq!(
        validate::money_optional("m", "\u{200B}1,200\u{200E}"),
        Ok(Some(Paisa::from_rupees(1_200).unwrap()))
    );
    assert_eq!(
        validate::phone_optional("p", "\u{200E}0300-0000000\u{200B}"),
        Ok(Some("03000000000".to_owned()))
    );
}

/// The Arabic-Indic sum rule holds through the field checks too, not only in the parsers.
#[test]
fn fields_refuse_arabic_indic_sums() {
    assert_eq!(
        code(validate::money("m", "١٠٠-٥٠")),
        ErrorCode::ErrNotANumber
    );
    assert_eq!(code(validate::qty("q", "١٠٠+٥٠")), ErrorCode::ErrNotANumber);
    assert_eq!(
        code(validate::money_optional("m", "١٠٠-٥٠")),
        ErrorCode::ErrNotANumber
    );
    // A single value, and a sum in Urdu digits, are still read.
    assert_eq!(
        validate::money("m", "١,٢٠٠"),
        Ok(Paisa::from_rupees(1_200).unwrap())
    );
    assert_eq!(
        validate::money("m", "۱۰۰-۵۰"),
        Ok(Paisa::from_rupees(50).unwrap())
    );
    // The empty check follows Unicode white space, like trim: a no-break space is empty.
    assert_eq!(validate::money_optional("m", "\u{00A0}"), Ok(None));
}

/// Finding R02-7: a code pasted with an invisible mark is read like a phone. RLM, ALM and the
/// bidi controls still make it invalid, and a code of only marks is required.
#[test]
fn account_codes_ignore_pasted_invisible_marks() {
    let full = Ok(AccountCodeInput::Full("0020".to_owned()));
    for mark in [
        '\u{200B}', '\u{200C}', '\u{200D}', '\u{200E}', '\u{2060}', '\u{FEFF}',
    ] {
        let text = format!("{mark}00{mark}20{mark}");
        assert_eq!(
            validate::account_code("c", &text),
            full,
            "U+{:04X}",
            u32::from(mark)
        );
    }
    let short = validate::account_code("c", "\u{200E}1\u{200B}");
    assert_eq!(
        short.map(|c| matches!(c, AccountCodeInput::Short(s) if s.serial() == 1)),
        Ok(true)
    );
    for mark in ['\u{200F}', '\u{061C}', '\u{202E}', '\u{2066}'] {
        let text = format!("00{mark}20");
        assert_eq!(
            code(validate::account_code("c", &text)),
            ErrorCode::ErrCodeInvalid,
            "U+{:04X}",
            u32::from(mark)
        );
    }
    assert_eq!(
        code(validate::account_code("c", "\u{200B}\u{2060}")),
        ErrorCode::ErrRequired
    );
}

/// Stricter commas reach the field checks: the weight, percent, money and qty fields.
#[test]
fn fields_follow_the_comma_rules() {
    assert_eq!(
        code(validate::weight("w", "12,345")),
        ErrorCode::ErrNotANumber
    );
    assert_eq!(
        code(validate::weight("w", "-12,345")),
        ErrorCode::ErrNotANumber
    );
    assert_eq!(
        code(validate::percent("p", "1,000")),
        ErrorCode::ErrNotANumber
    );
    assert_eq!(
        code(validate::money("m", "0,500")),
        ErrorCode::ErrNotANumber
    );
    assert_eq!(code(validate::qty("q", "00,500")), ErrorCode::ErrNotANumber);
    assert_eq!(
        validate::money("m", "10,00,000"),
        Ok(Paisa::from_rupees(1_000_000).unwrap())
    );
}

/// Finding R02-3 (Trojan Source, PRD FR-19.6): bidi embeddings, overrides and isolates
/// change the order text is shown in, so a note with one is refused.
#[test]
fn free_text_refuses_bidi_controls() {
    for mark in [
        '\u{202A}', '\u{202B}', '\u{202C}', '\u{202D}', '\u{202E}', '\u{2066}', '\u{2067}',
        '\u{2068}', '\u{2069}',
    ] {
        let note = format!("رقم {mark}0021");
        assert_eq!(
            code(validate::free_text("note", &note)),
            ErrorCode::ErrControlChars,
            "U+{:04X}",
            u32::from(mark)
        );
    }
    // LRM, RLM and a plain mixed-script note are allowed.
    assert!(validate::free_text("note", "رقم 1200 \u{200F}ok").is_ok());
    assert!(validate::free_text("note", "Akram اکرم 12-3").is_ok());
}

#[test]
fn free_text() {
    assert_eq!(validate::free_text("note", "  نوٹ  "), Ok("نوٹ".to_owned()));
    assert_eq!(validate::free_text("note", ""), Ok(String::new()));
    let e = validate::free_text("note", &"x".repeat(501)).unwrap_err();
    assert_eq!(e.code, ErrorCode::ErrTextTooLong);
    assert_eq!(e.values["max"], ErrorValue::Number(500));
    assert!(validate::free_text("note", &"x".repeat(500)).is_ok());
    assert_eq!(
        code(validate::free_text("note", "a\u{0007}b")),
        ErrorCode::ErrControlChars
    );
}

#[test]
fn several_errors_return_together() {
    let mut errors = FieldErrors::default();
    let qty = errors.take(validate::qty("lots[0].sales[0].qty", "abc"));
    let rate = errors.take(validate::rate("lots[0].sales[0].rate", "0"));
    let name = errors.take(validate::name_en("name_en", "Akram"));
    assert!(qty.is_none() && rate.is_none());
    assert_eq!(name.as_deref(), Some("Akram"));
    assert!(!errors.is_empty());
    errors.push(FieldError::new(ErrorCode::ErrSharesNot100, "shares"));
    let all = errors.finish().unwrap_err();
    let codes: Vec<ErrorCode> = all.iter().map(|e| e.code).collect();
    assert_eq!(
        codes,
        [
            ErrorCode::ErrNotANumber,
            ErrorCode::ErrMustBeMoreThanZero,
            ErrorCode::ErrSharesNot100
        ]
    );
    assert_eq!(all[0].field, "lots[0].sales[0].qty");
    assert!(FieldErrors::default().finish().is_ok());
}

#[test]
fn parse_sum_names_of_code_standards() {
    assert_eq!(
        validate::parse_sum("20+25"),
        Ok(Paisa::from_rupees(45).unwrap())
    );
    assert_eq!(validate::parse_sum_qty("20+25"), Ok(Qty::new(45).unwrap()));
}
