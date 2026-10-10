//! Hostile text in number fields (Code Standards 4.2 and M8, money agent 3.1, PRD E24, E31,
//! E42). Every case gives the exact value or the exact error code, and never panics.
//!
//! Lead decisions 10-10-2026: zero-width marks and LRM (ZWSP, ZWNJ, ZWJ, LRM, BOM) inside a
//! number are removed before parsing. Marks that can make the screen show another number
//! than the one read are refused: RLM (a strong right-to-left character) and the bidi
//! embeddings, overrides and isolates (U+202A-U+202E, U+2066-U+2069). Urdu '،' and
//! '۔' are refused, not read as a comma or a point.

use mandi_core::error::ErrorCode::{
    self, ErrAmountTooLarge as TooLarge, ErrDecimalsNotAllowed as Decimals,
    ErrMustBeMoreThanZero as Zero, ErrNegativeNotAllowed as Negative, ErrNotANumber as Nan,
    ErrPercentOver100 as Over100, ErrQtyNotWhole as NotWhole, ErrQtyTooLarge as QtyTooLarge,
    ErrRequired as Required, ErrTooManyDecimals as TooManyDecimals,
    ErrWeightTooLarge as WeightTooLarge,
};
use mandi_core::money::{
    parse_bps, parse_qty, parse_rupees, parse_sum_qty, parse_sum_rupees, parse_weight,
};
use mandi_core::validate;

use super::{hostile_strings, no_panic};

/// Runs a table of `(input, expected rupees or code)` through a parser.
fn check<T>(
    name: &str,
    parse: impl Fn(&str) -> Result<T, ErrorCode>,
    value: impl Fn(T) -> i64,
    rows: &[(&str, Result<i64, ErrorCode>)],
) {
    let mut wrong = Vec::new();
    for (input, expected) in rows {
        let got = no_panic(input, || parse(input)).map(&value);
        if got != *expected {
            wrong.push(format!(
                "{name}({input:?}) = {got:?}, expected {expected:?}"
            ));
        }
    }
    assert!(wrong.is_empty(), "\n{}", wrong.join("\n"));
}

const MONEY_ROWS: &[(&str, Result<i64, ErrorCode>)] = &[
    // Extremes.
    ("0", Ok(0)),
    ("1", Ok(1)),
    ("10000000000", Ok(10_000_000_000)),
    ("10,00,00,00,000", Ok(10_000_000_000)),
    ("10,000,000,000", Ok(10_000_000_000)),
    ("10000000001", Err(TooLarge)),
    ("9223372036854775807", Err(TooLarge)),
    ("18446744073709551616", Err(TooLarge)),
    ("99999999999999999999999999999999999999999", Err(TooLarge)),
    ("0000000000000000000000000000000000000600", Ok(600)),
    // Decimals (E31).
    ("850.50", Err(Decimals)),
    ("850.0", Err(Decimals)),
    ("850.", Err(Decimals)),
    (".5", Err(Decimals)),
    ("0.0", Err(Decimals)),
    ("1,000.5", Err(Decimals)),
    ("٨٥٠٫٥٠", Err(Decimals)),
    (".", Err(Nan)),
    ("..5", Err(Nan)),
    ("1.2.3", Err(Nan)),
    // Commas in odd places.
    ("1,00,0", Err(Nan)),
    ("850,50", Err(Nan)),
    (",100", Err(Nan)),
    ("100,", Err(Nan)),
    ("1,,000", Err(Nan)),
    ("1,0000", Err(Nan)),
    ("1000,000", Err(Nan)),
    ("1,000,00,000", Err(Nan)),
    ("12,34,5x", Err(Nan)),
    (",", Err(Nan)),
    ("0,500", Err(Nan)),
    ("00,500", Err(Nan)),
    ("01,000", Err(Nan)),
    ("1,00,000", Ok(100_000)),
    ("1,000,000", Ok(1_000_000)),
    ("١٬٢٠٠", Ok(1_200)),
    // Digits of other scripts.
    ("۶۰۰", Ok(600)),
    ("٦٠٠", Ok(600)),
    ("۱,۲۰۰", Ok(1_200)),
    ("６００", Err(Nan)),
    ("𝟔𝟎𝟎", Err(Nan)),
    ("६००", Err(Nan)),
    ("⁶⁰⁰", Err(Nan)),
    ("٦e٥", Err(Nan)),
    // Spaces.
    ("  600 ", Ok(600)),
    ("\t600\n", Ok(600)),
    ("\u{00A0}600\u{00A0}", Ok(600)),
    ("\u{3000}600", Ok(600)),
    ("6 00", Err(Nan)),
    ("1 000", Err(Nan)),
    ("6\u{00A0}00", Err(Nan)),
    // Signs.
    ("-5", Err(Negative)),
    ("\u{2212}5", Err(Negative)),
    ("-0", Err(Negative)),
    ("-850.50", Err(Negative)),
    ("+5", Err(Nan)),
    ("--5", Err(Nan)),
    ("-", Err(Nan)),
    ("- 5", Err(Nan)),
    ("5-", Err(Nan)),
    ("(5)", Err(Nan)),
    ("-abc", Err(Nan)),
    // Empty.
    ("", Err(Required)),
    ("   ", Err(Required)),
    ("\u{200B}", Err(Required)),
    ("\u{200E}\u{200B}", Err(Required)),
    ("\u{200F}\u{200E}", Err(Nan)),
    // Not numbers.
    ("abc", Err(Nan)),
    ("1e5", Err(Nan)),
    ("1E5", Err(Nan)),
    ("0x10", Err(Nan)),
    ("0b101", Err(Nan)),
    ("0o7", Err(Nan)),
    ("1_000", Err(Nan)),
    ("NaN", Err(Nan)),
    ("Infinity", Err(Nan)),
    ("inf", Err(Nan)),
    ("6\u{0000}00", Err(Nan)),
    ("600\n5", Err(Nan)),
    ("6\u{0301}00", Err(Nan)),
    ("' OR 1=1 --", Err(Nan)),
    ("600; DROP TABLE accounts", Err(Nan)),
    // Urdu comma and full stop are refused (lead decision).
    ("1،000", Err(Nan)),
    ("850۔50", Err(Nan)),
    ("600۔", Err(Nan)),
    // Invisible marks are removed (lead decision).
    ("6\u{200B}00", Ok(600)),
    ("\u{200E}600", Ok(600)),
    ("600\u{200E}", Ok(600)),
    ("6\u{200D}0\u{200C}0", Ok(600)),
    ("\u{FEFF}600", Ok(600)),
    ("۶\u{200E}۰۰", Ok(600)),
    // RLM, embeddings, overrides and isolates are refused: "<RLO>12" would show "21".
    ("\u{200F}600", Err(Nan)),
    ("500\u{200F}-200", Err(Nan)),
    ("\u{061C}600", Err(Nan)),
    ("\u{202B}600\u{202C}", Err(Nan)),
    ("\u{202E}12", Err(Nan)),
    ("\u{202E}600\u{202C}", Err(Nan)),
    ("\u{2066}600\u{2069}", Err(Nan)),
    ("\u{2067}1,200\u{2069}", Err(Nan)),
];

/// The word joiner (U+2060) is zero-width like ZWSP and BOM and never reorders text, so the
/// number parser removes it too (finding R02-1, fixed).
const MONEY_ROWS_OTHER_MARKS: &[(&str, Result<i64, ErrorCode>)] =
    &[("6\u{2060}00", Ok(600)), ("\u{2060}600\u{2060}", Ok(600))];

#[test]
fn parse_rupees_hostile_table() {
    check("parse_rupees", parse_rupees, |p| p.rupees(), MONEY_ROWS);
}

#[test]
fn parse_rupees_word_joiner() {
    check(
        "parse_rupees",
        parse_rupees,
        |p| p.rupees(),
        MONEY_ROWS_OTHER_MARKS,
    );
}

#[test]
fn single_value_money_fields_agree_with_parse_rupees() {
    // A field that takes a sum must read a single value exactly as parse_rupees does.
    check(
        "parse_sum_rupees",
        parse_sum_rupees,
        |p| p.rupees(),
        MONEY_ROWS,
    );
}

#[test]
fn money_sums() {
    check(
        "parse_sum_rupees",
        parse_sum_rupees,
        |p| p.rupees(),
        &[
            ("20+25", Ok(45)),
            ("1,200+300", Ok(1_500)),
            ("500-200", Ok(300)),
            ("20 + 25", Ok(45)),
            ("۲۰+۲۵", Ok(45)),
            ("20\u{2212}5", Ok(15)),
            ("20\u{200E}+\u{200E}25", Ok(45)),
            ("20\u{200F}+\u{200F}25", Err(Nan)),
            ("5-5", Ok(0)),
            ("0+0+0", Ok(0)),
            ("10000000000+0", Ok(10_000_000_000)),
            ("10000000000-1+1", Ok(10_000_000_000)),
            ("10000000000+10000000000-10000000000", Ok(10_000_000_000)),
            ("10000000000+1", Err(TooLarge)),
            ("10000000001-1", Err(TooLarge)),
            ("10-20", Err(Negative)),
            ("0-10000000000", Err(Negative)),
            ("20+", Err(Nan)),
            ("+20", Err(Nan)),
            ("-5+10", Err(Nan)),
            ("20++5", Err(Nan)),
            ("20+-5", Err(Nan)),
            ("2*3", Err(Nan)),
            ("2×3", Err(Nan)),
            ("6/2", Err(Nan)),
            ("6÷2", Err(Nan)),
            ("20.5+1", Err(Decimals)),
            ("20+1e5", Err(Nan)),
            ("20+1,00,0", Err(Nan)),
            ("+", Err(Nan)),
            ("-", Err(Nan)),
            ("+-+-", Err(Nan)),
        ],
    );
}

#[test]
fn money_sum_of_many_terms() {
    // 100,000 terms of 1: 1 lakh rupees. No overflow is reachable: each term is at most
    // 10^10 rupees, so i64 would need about 9 x 10^8 terms.
    let text = "1+".repeat(99_999) + "1";
    assert_eq!(parse_sum_rupees(&text).map(|p| p.rupees()), Ok(100_000));
    let text = "10000000000+".repeat(1_000) + "0";
    assert_eq!(parse_sum_rupees(&text), Err(TooLarge));
    let text = "10000000000+".repeat(1_000) + "0" + &"-10000000000".repeat(1_000);
    assert_eq!(parse_sum_rupees(&text).map(|p| p.rupees()), Ok(0));
}

#[test]
fn rate_and_optional_money() {
    let rate = |t: &str| validate::rate("rate", t).map_err(|e| e.code);
    check(
        "rate",
        rate,
        |p| p.rupees(),
        &[
            ("600", Ok(600)),
            ("0", Err(Zero)),
            ("5-5", Err(Zero)),
            ("000", Err(Zero)),
            ("-600", Err(Negative)),
            ("600.5", Err(Decimals)),
            ("", Err(Required)),
        ],
    );
    assert_eq!(validate::money_optional("m", "  "), Ok(None));
    let e = validate::money("lots[0].freight", "10000000001").unwrap_err();
    assert_eq!(e.field, "lots[0].freight");
    assert_eq!(e.code, TooLarge);
    assert_eq!(
        serde_json::to_value(&e).unwrap()["values"]["max"],
        10_000_000_000_i64
    );
}

#[test]
fn optional_fields_treat_invisible_marks_as_empty() {
    // Finding R02-5 (fixed): a field holding only an invisible mark is empty, not "required".
    for blank in ["\u{200B}", "\u{FEFF}", " \u{200E} "] {
        assert_eq!(validate::money_optional("m", blank), Ok(None), "{blank:?}");
        assert_eq!(validate::phone_optional("p", blank), Ok(None), "{blank:?}");
    }
}

#[test]
fn quantity_fields() {
    let rows: &[(&str, Result<i64, ErrorCode>)] = &[
        ("1", Ok(1)),
        ("1000000", Ok(1_000_000)),
        ("1,000,000", Ok(1_000_000)),
        ("10,00,000", Ok(1_000_000)),
        ("0,500", Err(Nan)),
        ("00,500", Err(Nan)),
        ("۲۰", Ok(20)),
        ("0", Err(Zero)),
        ("000", Err(Zero)),
        ("1000001", Err(QtyTooLarge)),
        ("99999999999999999999999", Err(QtyTooLarge)),
        ("2.5", Err(NotWhole)),
        ("2.", Err(NotWhole)),
        (".5", Err(NotWhole)),
        ("-1", Err(Negative)),
        ("", Err(Required)),
        ("1e3", Err(Nan)),
        ("1,0", Err(Nan)),
        ("２０", Err(Nan)),
    ];
    check("parse_qty", parse_qty, |q| q.get(), rows);
    check("parse_sum_qty", parse_sum_qty, |q| q.get(), rows);
    check(
        "parse_sum_qty",
        parse_sum_qty,
        |q| q.get(),
        &[
            ("20+25", Ok(45)),
            ("0+1", Ok(1)),
            ("5-5", Err(Zero)),
            ("10-20", Err(Negative)),
            ("1000000+1", Err(QtyTooLarge)),
            ("1000001-1", Err(QtyTooLarge)),
            ("1000000+1000000-1000000", Ok(1_000_000)),
            ("2.5+2.5", Err(NotWhole)),
            ("20+", Err(Nan)),
            ("20*2", Err(Nan)),
        ],
    );
}

#[test]
fn percent_fields() {
    check(
        "parse_bps",
        parse_bps,
        |b| i64::from(b.get()),
        &[
            ("0", Ok(0)),
            ("0.01", Ok(1)),
            ("6", Ok(600)),
            ("6.", Err(Nan)),
            (".5", Ok(50)),
            ("100", Ok(10_000)),
            ("100.00", Ok(10_000)),
            ("۲.۵", Ok(250)),
            ("100.01", Err(Over100)),
            ("101", Err(Over100)),
            ("99999999999999999999999", Err(Over100)),
            ("0.001", Err(TooManyDecimals)),
            ("2.255", Err(TooManyDecimals)),
            ("-1", Err(Negative)),
            ("6%", Err(Nan)),
            ("2,5", Err(Nan)),
            // Percent takes no commas at all.
            ("1,000", Err(Nan)),
            ("0,500", Err(Nan)),
            ("00,500", Err(Nan)),
            ("1e2", Err(Nan)),
            ("", Err(Required)),
        ],
    );
}

#[test]
fn weight_fields() {
    check(
        "parse_weight",
        parse_weight,
        |g| g.get(),
        &[
            ("0.001", Ok(1)),
            ("12.345", Ok(12_345)),
            ("100000", Ok(100_000_000)),
            // Weight takes no commas at all.
            ("100,000.000", Err(Nan)),
            ("100000.000", Ok(100_000_000)),
            ("1,000", Err(Nan)),
            ("0,500", Err(Nan)),
            ("00,500", Err(Nan)),
            ("0", Err(Zero)),
            ("0.000", Err(Zero)),
            ("0.0001", Err(TooManyDecimals)),
            ("100000.001", Err(WeightTooLarge)),
            ("99999999999999999999999", Err(WeightTooLarge)),
            ("-1", Err(Negative)),
            ("12,5", Err(Nan)),
            ("", Err(Required)),
        ],
    );
}

#[test]
fn every_parser_survives_every_hostile_string() {
    for s in hostile_strings() {
        let short: String = s.chars().take(20).collect();
        no_panic(&short, || {
            let _ = parse_rupees(&s);
            let _ = parse_sum_rupees(&s);
            let _ = parse_qty(&s);
            let _ = parse_sum_qty(&s);
            let _ = parse_bps(&s);
            let _ = parse_weight(&s);
            let _ = validate::rate("f", &s);
            let _ = validate::percent("f", &s);
            let _ = validate::weight("f", &s);
        });
    }
}

#[test]
fn ten_thousand_digits_is_too_large_not_slow() {
    let start = std::time::Instant::now();
    let nines = "9".repeat(10_000);
    assert_eq!(parse_rupees(&nines), Err(TooLarge));
    assert_eq!(parse_qty(&nines), Err(QtyTooLarge));
    assert_eq!(parse_bps(&nines), Err(Over100));
    assert_eq!(parse_weight(&nines), Err(WeightTooLarge));
    let a_mb = "9".repeat(1_000_000);
    assert_eq!(parse_sum_rupees(&a_mb), Err(TooLarge));
    // Debug build; a release build is far faster. Linear time, not quadratic.
    assert!(
        start.elapsed().as_millis() < 2_000,
        "took {:?}",
        start.elapsed()
    );
}

// ---- Trojan Source in number fields (plan 02 Section 4.6) --------------------------------

/// Marks that can change the order the screen shows digits in: refused wherever they sit.
const REFUSED_MARKS: [char; 12] = [
    '\u{200F}', '\u{061C}', '\u{202A}', '\u{202B}', '\u{202C}', '\u{202D}', '\u{202E}', '\u{2066}',
    '\u{2067}', '\u{2068}', '\u{2069}', '\u{FFFC}',
];

/// Marks that never reorder anything: removed before the number is read.
const REMOVED_MARKS: [char; 6] = [
    '\u{200B}', '\u{200C}', '\u{200D}', '\u{200E}', '\u{2060}', '\u{FEFF}',
];

/// `text` with `mark` put in front of the `index`-th character (`index` = length: at the end).
fn with_mark_at(text: &str, index: usize, mark: char) -> String {
    let mut out = String::new();
    for (i, c) in text.chars().enumerate() {
        if i == index {
            out.push(mark);
        }
        out.push(c);
    }
    if index >= text.chars().count() {
        out.push(mark);
    }
    out
}

#[test]
fn trojan_source_cases_from_the_lead_list() {
    // "<RLO>12" would show "21" on screen but is read as 12: it must be refused.
    check(
        "parse_sum_rupees",
        parse_sum_rupees,
        |p| p.rupees(),
        &[
            ("\u{202E}12", Err(Nan)),
            ("\u{202E}1,200", Err(Nan)),
            ("\u{202E}20+25", Err(Nan)),
            ("500\u{200F}-200", Err(Nan)),
            ("500-\u{200F}200", Err(Nan)),
            ("\u{200F}", Err(Nan)),
            ("\u{061C}", Err(Nan)),
            // Arabic-Indic digits (bidi class AN) in a sum: shown in another order.
            ("١٠٠-٥٠", Err(Nan)),
            ("١٠٠+٥٠", Err(Nan)),
            ("١٠٠+50", Err(Nan)),
            ("100-٥٠", Err(Nan)),
            ("١,٢٠٠+١", Err(Nan)),
            ("١٠٠\u{2212}٥٠", Err(Nan)),
            ("١٠٠٫٥-١", Err(Nan)),
            // Arabic-Indic digits alone are one value and still accepted.
            ("١,٢٠٠", Ok(1_200)),
            ("١٬٢٠٠", Ok(1_200)),
            ("١٢٠٠", Ok(1_200)),
            ("٦٠٠", Ok(600)),
            // Urdu digits (class EN) keep their order, so a sum of them is accepted.
            ("۱۰۰-۵۰", Ok(50)),
            ("۱۰۰+۵۰", Ok(150)),
            ("۱,۲۰۰+۱", Ok(1_201)),
            ("20+۲۵", Ok(45)),
        ],
    );
    for refused in [
        "\u{202E}12",
        "\u{202E}1,200",
        "\u{202E}20+25",
        "500\u{200F}-200",
    ] {
        assert_eq!(parse_qty(refused), Err(Nan), "parse_qty({refused:?})");
        assert_eq!(
            parse_sum_qty(refused),
            Err(Nan),
            "parse_sum_qty({refused:?})"
        );
    }
    assert_eq!(parse_sum_qty("١٠٠-٥٠"), Err(Nan));
    assert_eq!(parse_sum_qty("١٠٠+٥٠"), Err(Nan));
    assert_eq!(parse_sum_qty("۱۰۰-۵۰").map(|q| q.get()), Ok(50));
    assert_eq!(parse_sum_qty("١٠٠").map(|q| q.get()), Ok(100));
}

#[test]
fn a_reordering_mark_is_refused_in_every_position_of_every_number_field() {
    let mut wrong = Vec::new();
    for base in ["600", "1,200", "۶۰۰", "٦٠٠", "20+25", "500-200", "۱۰۰-۵۰"] {
        let length = base.chars().count();
        for mark in REFUSED_MARKS {
            for index in 0..=length {
                let text = with_mark_at(base, index, mark);
                let results = [
                    (
                        "parse_rupees",
                        parse_rupees(&text).map(|p| p.rupees()).err(),
                    ),
                    (
                        "parse_sum_rupees",
                        parse_sum_rupees(&text).map(|p| p.rupees()).err(),
                    ),
                    ("parse_qty", parse_qty(&text).map(|q| q.get()).err()),
                    ("parse_sum_qty", parse_sum_qty(&text).map(|q| q.get()).err()),
                ];
                for (name, got) in results {
                    // Not everything is a valid base for every parser (a sum in parse_rupees);
                    // the rule is only that the answer is an error, never a number.
                    if got.is_none() {
                        wrong.push(format!("{name}({text:?}) was accepted"));
                    }
                }
            }
        }
    }
    for mark in REFUSED_MARKS {
        for index in 0..=4 {
            let pct = with_mark_at("6.25", index, mark);
            let kg = with_mark_at("12.345", index, mark);
            if parse_bps(&pct).is_ok() {
                wrong.push(format!("parse_bps({pct:?}) was accepted"));
            }
            if parse_weight(&kg).is_ok() {
                wrong.push(format!("parse_weight({kg:?}) was accepted"));
            }
        }
    }
    assert!(wrong.is_empty(), "\n{}", wrong.join("\n"));
}

#[test]
fn a_zero_width_mark_never_changes_the_number_read() {
    let mut wrong = Vec::new();
    for base in ["600", "1,200", "۶۰۰", "٦٠٠", "20+25", "500-200", "۱۰۰-۵۰"] {
        let length = base.chars().count();
        for mark in REMOVED_MARKS {
            for index in 0..=length {
                let text = with_mark_at(base, index, mark);
                let (a, b) = (parse_sum_rupees(&text), parse_sum_rupees(base));
                if a != b {
                    wrong.push(format!(
                        "parse_sum_rupees({text:?}) = {a:?}, base gives {b:?}"
                    ));
                }
                let (a, b) = (parse_sum_qty(&text), parse_sum_qty(base));
                if a != b {
                    wrong.push(format!("parse_sum_qty({text:?}) = {a:?}, base gives {b:?}"));
                }
            }
        }
    }
    assert!(wrong.is_empty(), "\n{}", wrong.join("\n"));
}

#[test]
fn only_the_six_zero_width_marks_can_hide_inside_a_number() {
    // Every Unicode scalar value is put in the middle of "600" and of "20+25". A character may
    // be accepted only if it is one of the six removed marks. Anything else that reads as the
    // same number would be an invisible way to change what the screen shows. Visible
    // characters are left out: spaces (a sum may have spaces around `+`) and the zeros '0'
    // and '۰' (they make "20+025", which is 45 and is seen as typed).
    let mut wrong = Vec::new();
    let skipped =
        |c: &char| REMOVED_MARKS.contains(c) || c.is_whitespace() || matches!(c, '0' | '\u{06F0}');
    for c in ('\0'..=char::MAX).filter(|c| !skipped(c)) {
        let money = format!("6{c}00");
        let sum = format!("20+{c}25");
        let got = no_panic("scan", || {
            (
                parse_rupees(&money).map(|p| p.rupees()),
                parse_sum_rupees(&sum).map(|p| p.rupees()),
            )
        });
        if got.0 == Ok(600) {
            wrong.push(format!("6{{U+{:04X}}}00 reads as 600", u32::from(c)));
        }
        if got.1 == Ok(45) {
            wrong.push(format!("20+{{U+{:04X}}}25 reads as 45", u32::from(c)));
        }
    }
    assert!(wrong.is_empty(), "\n{}", wrong.join("\n"));
}

#[test]
fn code_and_phone_fields_refuse_reordering_marks() {
    let code = |s: &str| validate::account_code("code", s).map_err(|e| e.code);
    let phone = |s: &str| validate::phone("phone", s).map_err(|e| e.code);
    for mark in REFUSED_MARKS {
        assert!(
            code(&format!("{mark}03040001")).is_err(),
            "code U+{:04X}",
            u32::from(mark)
        );
        assert!(
            code(&format!("0304{mark}0001")).is_err(),
            "code U+{:04X}",
            u32::from(mark)
        );
        assert!(
            phone(&format!("{mark}03000000000")).is_err(),
            "phone U+{:04X}",
            u32::from(mark)
        );
        assert!(
            phone(&format!("0300{mark}-0000000")).is_err(),
            "phone U+{:04X}",
            u32::from(mark)
        );
    }
    // Pasted zero-width marks around a phone are removed.
    for mark in REMOVED_MARKS {
        assert_eq!(
            phone(&format!("{mark}0300-0000000{mark}")),
            Ok("03000000000".to_owned()),
            "phone U+{:04X}",
            u32::from(mark)
        );
    }
}

#[test]
fn a_pasted_zero_width_mark_in_a_code_is_taken_like_a_phone() {
    // Finding R02-7 (fixed): a code is cleaned like a phone, so a pasted mark is removed.
    let code = |s: &str| validate::account_code("code", s).map_err(|e| e.code);
    for mark in REMOVED_MARKS {
        let got = code(&format!("{mark}03040001{mark}"));
        assert_eq!(
            got,
            Ok(validate::AccountCodeInput::Full("03040001".to_owned())),
            "code U+{:04X}",
            u32::from(mark)
        );
    }
}
