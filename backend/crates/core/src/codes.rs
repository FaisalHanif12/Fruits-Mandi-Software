//! Codes of sellers, buyers, پختہ کھاتہ, items, packings and staff (plan 32, PRD 1.8).
//!
//! A code is the **type number** (3 digits) followed by a **running number** that starts at 0 and
//! is written with no padding:
//!
//! | Type | Number | First codes |
//! |---|---|---|
//! | Seller (بیوپاری) | `001` | `0010`, `0011` ... `0019`, `00110`, `00111` ... |
//! | Buyer (گاہک) | `002` | `0020`, `0021` ... |
//! | Pukhta (پختہ کھاتہ) | `003` | `0030` (buyer account), `0031` (seller account) |
//! | Item (قسم مال) | `004` | `0040`, `0041` ... |
//! | Packing (پیکنگ) | `005` | `0050`, `0051` ... |
//! | Staff (عملہ) | `006` | `0060`, `0061` ... |
//! | Market trip (مارکیٹ سفر) | `007` | `0070`, `0071` ... (plan 34) |
//! | Garden (باغ) | `008` | `0080`, `0081` ... and the garden's account has the same code |
//!
//! The next code of a type is the largest running number of that type plus one, so a code is never
//! reused. A code has at most 10 digits (`MAX_CODE_DIGITS` of the masters), so the running number has
//! at most 7. The short code of a seller or a buyer is its running number alone (`0` finds `0010`,
//! `11` finds `00111`). The type number is always 3 digits and the running number never starts with
//! `0` (except `0` itself), so a code and a short code can never be mistaken for each other.
//!
//! Pure functions, no database.

use crate::error::{ErrorCode, FieldError};

/// The digits of a type number.
pub const TYPE_DIGITS: usize = 3;
/// The most digits of a code (the limit of every master code).
pub const MAX_CODE_DIGITS: usize = 10;
/// The largest running number: what is left of [`MAX_CODE_DIGITS`] after the type number.
pub const MAX_RUNNING: u64 = 9_999_999;

/// پختہ کھاتہ (خریدار): the one-time buyers' account. Its running number is 0.
pub const PUKHTA_BUYER_CODE: &str = "0030";
/// پختہ کھاتہ (بیوپاری): the one-time sellers' account. Its running number is 1.
pub const PUKHTA_SELLER_CODE: &str = "0031";

/// The kinds of record that have a code of this scheme.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum CodeType {
    /// بیوپاری, type number `001`.
    Seller,
    /// گاہک, type number `002`.
    Buyer,
    /// پختہ کھاتہ, type number `003`. Only the two system accounts have this type.
    Pukhta,
    /// قسم مال, type number `004`.
    Item,
    /// پیکنگ, type number `005`.
    Packing,
    /// عملہ, type number `006`.
    Staff,
    /// مارکیٹ سفر, type number `007` (plan 34): the number of a trip. The first trip is `0070`.
    Trip,
    /// باغ, type number `008` (plan 34): the number of a garden and the code of its account. The
    /// first garden is `0080`.
    Garden,
}

impl CodeType {
    /// Every type.
    pub const ALL: [CodeType; 8] = [
        CodeType::Seller,
        CodeType::Buyer,
        CodeType::Pukhta,
        CodeType::Item,
        CodeType::Packing,
        CodeType::Staff,
        CodeType::Trip,
        CodeType::Garden,
    ];

    /// The three digits at the start of every code of this type.
    ///
    /// ```
    /// use mandi_core::codes::CodeType;
    /// assert_eq!(CodeType::Seller.number(), "001");
    /// assert_eq!(CodeType::Staff.number(), "006");
    /// ```
    pub const fn number(self) -> &'static str {
        match self {
            CodeType::Seller => "001",
            CodeType::Buyer => "002",
            CodeType::Pukhta => "003",
            CodeType::Item => "004",
            CodeType::Packing => "005",
            CodeType::Staff => "006",
            CodeType::Trip => "007",
            CodeType::Garden => "008",
        }
    }

    /// The type that has this type number.
    pub fn from_number(number: &str) -> Option<CodeType> {
        CodeType::ALL.into_iter().find(|t| t.number() == number)
    }
}

/// A code taken apart.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParsedCode {
    /// The type.
    pub code_type: CodeType,
    /// The running number.
    pub running: u64,
}

/// The code of a type and a running number: the type number, then the running number with no
/// padding. `None` if the running number is more than [`MAX_RUNNING`].
///
/// ```
/// use mandi_core::codes::{build_code, CodeType};
/// assert_eq!(build_code(CodeType::Seller, 0).as_deref(), Some("0010"));
/// assert_eq!(build_code(CodeType::Seller, 10).as_deref(), Some("00110"));
/// assert_eq!(build_code(CodeType::Buyer, 1).as_deref(), Some("0021"));
/// assert_eq!(build_code(CodeType::Item, 10_000_000), None);
/// ```
pub fn build_code(code_type: CodeType, running: u64) -> Option<String> {
    (running <= MAX_RUNNING).then(|| format!("{}{running}", code_type.number()))
}

/// True for the text of a running number: digits only, no leading zero (a lone `0` is fine), at
/// most 7 digits. This is the text typed as a short code.
fn is_running_text(text: &str) -> bool {
    !text.is_empty()
        && text.len() <= MAX_CODE_DIGITS.saturating_sub(TYPE_DIGITS)
        && text.bytes().all(|b| b.is_ascii_digit())
        && (text == "0" || !text.starts_with('0'))
}

/// A typed short code as a running number: `"0"` is 0, `"11"` is 11. `None` for anything that is not
/// a running number (letters, a leading zero such as `"011"`, an empty text, more than 7 digits).
///
/// ```
/// use mandi_core::codes::parse_running;
/// assert_eq!(parse_running("0"), Some(0));
/// assert_eq!(parse_running("11"), Some(11));
/// assert_eq!(parse_running("011"), None); // a full code starts with 0, a short code never does
/// assert_eq!(parse_running("1a"), None);
/// ```
pub fn parse_running(typed: &str) -> Option<u64> {
    let text = typed.trim();
    if is_running_text(text) {
        text.parse().ok()
    } else {
        None
    }
}

/// Takes a full code apart: the type number and a running number with no padding. `None` for a
/// code that is not of this scheme (the cash account `01010001`, a city code `12`).
///
/// ```
/// use mandi_core::codes::{parse_code, CodeType};
/// let p = parse_code("00110").unwrap();
/// assert_eq!((p.code_type, p.running), (CodeType::Seller, 10));
/// assert!(parse_code("001").is_none());      // no running number
/// assert!(parse_code("00101").is_none());    // padded: 0 then 1 is not a running number
/// assert!(parse_code("01010001").is_none());
/// ```
pub fn parse_code(code: &str) -> Option<ParsedCode> {
    let text = code.trim();
    let number = text.get(..TYPE_DIGITS)?;
    let rest = text.get(TYPE_DIGITS..)?;
    let code_type = CodeType::from_number(number)?;
    let running = parse_running(rest)?;
    Some(ParsedCode { code_type, running })
}

/// The running number of a code if it is a code of this type.
///
/// ```
/// use mandi_core::codes::{running_of, CodeType};
/// assert_eq!(running_of("0021", CodeType::Buyer), Some(1));
/// assert_eq!(running_of("0021", CodeType::Seller), None);
/// ```
pub fn running_of(code: &str, code_type: CodeType) -> Option<u64> {
    parse_code(code)
        .filter(|p| p.code_type == code_type)
        .map(|p| p.running)
}

/// The code of a trip or a garden from its number in its own series (1, 2, 3 ...): the type number
/// and the number less one, so trip 1 is `0070`, trip 2 `0071`, garden 1 `0080` (plan 34). A trip or
/// garden is never deleted and its number is never reused, so its code never is either. A number
/// below 1 or too big for a code gives the plain number, which a screen shows as it is.
///
/// ```
/// use mandi_core::codes::{number_code, CodeType};
/// assert_eq!(number_code(CodeType::Trip, 1), "0070");
/// assert_eq!(number_code(CodeType::Trip, 11), "00710");
/// assert_eq!(number_code(CodeType::Garden, 1), "0080");
/// assert_eq!(number_code(CodeType::Garden, 0), "0");
/// ```
pub fn number_code(code_type: CodeType, number: i64) -> String {
    number
        .checked_sub(1)
        .and_then(|running| u64::try_from(running).ok())
        .and_then(|running| build_code(code_type, running))
        .unwrap_or_else(|| number.to_string())
}

/// The next running number of a type: 0 when there is no code yet, else the largest plus one.
/// `None` when the largest is already [`MAX_RUNNING`] (the list is full).
///
/// ```
/// use mandi_core::codes::next_running;
/// assert_eq!(next_running(None), Some(0));
/// assert_eq!(next_running(Some(9)), Some(10));
/// assert_eq!(next_running(Some(9_999_999)), None);
/// ```
pub fn next_running(largest: Option<u64>) -> Option<u64> {
    match largest {
        None => Some(0),
        Some(n) => n.checked_add(1).filter(|next| *next <= MAX_RUNNING),
    }
}

/// Checks a code typed on a form for a record of `code_type`: it must be a full code of that type.
/// Returns the running number.
///
/// - `ERR_CODE_INVALID` if it is not a full code of this type (wrong type number, padded running
///   number, letters, too long). Example: `0021` on a seller form.
///
/// ```
/// use mandi_core::codes::{check_typed_code, CodeType};
/// assert_eq!(check_typed_code("code", "0013", CodeType::Seller).unwrap(), 3);
/// assert!(check_typed_code("code", "0023", CodeType::Seller).is_err());
/// ```
///
/// # Errors
/// `ERR_CODE_INVALID` on `field`.
pub fn check_typed_code(field: &str, typed: &str, code_type: CodeType) -> Result<u64, FieldError> {
    running_of(typed.trim(), code_type)
        .ok_or_else(|| FieldError::new(ErrorCode::ErrCodeInvalid, field))
}

/// The short code of a seller or a buyer code: its running number. `None` for every other code,
/// and for پختہ کھاتہ (`0030`, `0031`), which has none (PRD 1.8 R5).
///
/// ```
/// use mandi_core::codes::short_code_of_party;
/// assert_eq!(short_code_of_party("0010").as_deref(), Some("0"));
/// assert_eq!(short_code_of_party("00111").as_deref(), Some("11"));
/// assert_eq!(short_code_of_party("0021").as_deref(), Some("1"));
/// assert_eq!(short_code_of_party("0030"), None);
/// assert_eq!(short_code_of_party("01010001"), None);
/// ```
pub fn short_code_of_party(code: &str) -> Option<String> {
    parse_code(code)
        .filter(|p| matches!(p.code_type, CodeType::Seller | CodeType::Buyer))
        .map(|p| p.running.to_string())
}

/// Orders codes for lists ("by code"): a code of this scheme by type number, then running number
/// as a number (`0019` before `00110`); any other code after them, shorter first, then as text
/// (`12` before `100`). The key is only for sorting.
///
/// ```
/// use mandi_core::codes::code_sort_key;
/// assert!(code_sort_key("0019") < code_sort_key("00110"));
/// assert!(code_sort_key("00110") < code_sort_key("0020"));
/// assert!(code_sort_key("0020") < code_sort_key("01010001"));
/// assert!(code_sort_key("9") < code_sort_key("12"));
/// ```
pub fn code_sort_key(code: &str) -> (u8, &'static str, u64, usize, String) {
    match parse_code(code) {
        Some(p) => (0, p.code_type.number(), p.running, 0, String::new()),
        None => (1, "", 0, code.len(), code.to_owned()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn worked_examples_of_the_plan() {
        // Plan 32 Section 3: first seller 0010, second 0011, eleventh 00110.
        assert_eq!(build_code(CodeType::Seller, 0).as_deref(), Some("0010"));
        assert_eq!(build_code(CodeType::Seller, 1).as_deref(), Some("0011"));
        assert_eq!(build_code(CodeType::Seller, 10).as_deref(), Some("00110"));
        // First buyer, item, packing, staff.
        assert_eq!(build_code(CodeType::Buyer, 0).as_deref(), Some("0020"));
        assert_eq!(build_code(CodeType::Item, 0).as_deref(), Some("0040"));
        assert_eq!(build_code(CodeType::Packing, 0).as_deref(), Some("0050"));
        assert_eq!(build_code(CodeType::Staff, 0).as_deref(), Some("0060"));
        // The two پختہ accounts.
        assert_eq!(
            parse_code(PUKHTA_BUYER_CODE).map(|p| (p.code_type, p.running)),
            Some((CodeType::Pukhta, 0))
        );
        assert_eq!(
            parse_code(PUKHTA_SELLER_CODE).map(|p| (p.code_type, p.running)),
            Some((CodeType::Pukhta, 1))
        );
    }

    #[test]
    fn trips_and_gardens_are_numbered_from_zero_in_their_own_types() {
        // Plan 34 R1, R2: the first trip is 0070, the first garden 0080 (and so is its account).
        assert_eq!(number_code(CodeType::Trip, 1), "0070");
        assert_eq!(number_code(CodeType::Trip, 2), "0071");
        assert_eq!(number_code(CodeType::Trip, 11), "00710");
        assert_eq!(number_code(CodeType::Garden, 1), "0080");
        assert_eq!(number_code(CodeType::Garden, 12), "00811");
        for (code, kind, running) in [
            ("0070", CodeType::Trip, 0),
            ("00710", CodeType::Trip, 10),
            ("0080", CodeType::Garden, 0),
            ("0081", CodeType::Garden, 1),
        ] {
            let p = parse_code(code).unwrap();
            assert_eq!((p.code_type, p.running), (kind, running), "{code}");
        }
        // Never a seller's or a buyer's short code.
        assert_eq!(short_code_of_party("0070"), None);
        assert_eq!(short_code_of_party("0080"), None);
        // A number that is not a number of the series gives the plain number.
        assert_eq!(number_code(CodeType::Trip, 0), "0");
        assert_eq!(number_code(CodeType::Trip, -3), "-3");
    }

    #[test]
    fn short_code_rule() {
        // Seller box: 0 finds 0010, 11 finds 00111.
        assert_eq!(parse_running("0"), Some(0));
        assert_eq!(parse_running("11"), Some(11));
        assert_eq!(running_of("00111", CodeType::Seller), Some(11));
        assert_eq!(running_of("0010", CodeType::Seller), Some(0));
        // A full code is never a short code: it starts with 00.
        assert_eq!(parse_running("0010"), None);
        assert_eq!(parse_running("00111"), None);
        // 8 digits is too long for a running number.
        assert_eq!(parse_running("12345678"), None);
        assert_eq!(parse_running("9999999"), Some(9_999_999));
    }

    #[test]
    fn codes_of_other_schemes_are_not_parsed() {
        for code in [
            "01010001",
            "03040000",
            "030119998",
            "12",
            "",
            "abc",
            "0010x",
            " ",
        ] {
            assert!(parse_code(code).is_none(), "{code}");
        }
        // The type number alone, a padded running number and a bad type number.
        for code in ["001", "00101", "0090", "0000"] {
            assert!(parse_code(code).is_none(), "{code}");
        }
    }

    #[test]
    fn a_full_code_is_at_most_ten_digits() {
        assert_eq!(
            build_code(CodeType::Seller, MAX_RUNNING).map(|c| c.len()),
            Some(MAX_CODE_DIGITS)
        );
        assert!(build_code(CodeType::Seller, MAX_RUNNING.saturating_add(1)).is_none());
    }

    #[test]
    fn typed_code_must_be_of_the_type() {
        assert_eq!(
            check_typed_code("code", " 0013 ", CodeType::Seller).unwrap(),
            3
        );
        for bad in ["0023", "13", "0030", "00103", "", "0013x"] {
            let e = check_typed_code("code", bad, CodeType::Seller).unwrap_err();
            assert_eq!(
                (e.code, e.field.as_str()),
                (ErrorCode::ErrCodeInvalid, "code"),
                "{bad}"
            );
        }
    }

    #[test]
    fn next_running_never_reuses_a_number() {
        assert_eq!(next_running(None), Some(0));
        assert_eq!(next_running(Some(0)), Some(1));
        assert_eq!(next_running(Some(19)), Some(20));
        assert_eq!(next_running(Some(MAX_RUNNING)), None);
    }

    #[test]
    fn sort_key_puts_running_numbers_in_number_order() {
        let mut codes = vec!["00110", "0019", "0011", "0020", "01010001", "0010", "00111"];
        codes.sort_by_key(|c| code_sort_key(c));
        assert_eq!(
            codes,
            vec!["0010", "0011", "0019", "00110", "00111", "0020", "01010001"]
        );
    }

    #[test]
    fn short_code_of_party_only_for_seller_and_buyer() {
        assert_eq!(short_code_of_party("0010").as_deref(), Some("0"));
        assert_eq!(short_code_of_party("0020").as_deref(), Some("0"));
        assert_eq!(short_code_of_party("0030"), None);
        assert_eq!(short_code_of_party("0040"), None);
    }

    proptest! {
        // parse(build(type, n)) == (type, n) for every type and running number.
        #[test]
        fn build_then_parse_gives_the_same(kind in 0usize..CodeType::ALL.len(), n in 0u64..=MAX_RUNNING) {
            let code_type = CodeType::ALL[kind];
            let code = build_code(code_type, n).unwrap();
            let parsed = parse_code(&code).unwrap();
            prop_assert_eq!((parsed.code_type, parsed.running), (code_type, n));
            prop_assert!(code.len() <= MAX_CODE_DIGITS);
        }

        // The short code of a seller or buyer is its running number, and finds it again.
        #[test]
        fn short_code_finds_the_running_number(buyer in any::<bool>(), n in 0u64..=MAX_RUNNING) {
            let code_type = if buyer { CodeType::Buyer } else { CodeType::Seller };
            let code = build_code(code_type, n).unwrap();
            let short = short_code_of_party(&code).unwrap();
            prop_assert_eq!(parse_running(&short), Some(n));
            // And it is never read as a full code.
            prop_assert!(parse_code(&short).is_none());
        }

        // Nothing typed ever panics, and a parsed running number prints back the same.
        #[test]
        fn parsing_any_text_never_panics(text in "\\PC{0,14}") {
            if let Some(p) = parse_code(&text) {
                let rebuilt = build_code(p.code_type, p.running);
                prop_assert_eq!(rebuilt.as_deref(), Some(text.trim()));
            }
            let _ = parse_running(&text);
        }
    }
}
