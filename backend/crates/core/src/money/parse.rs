//! Parsing money, percent, weight and quantity text typed by the user
//! (Code Standards M8 and 4.2, money agent 3.1, PRD FR-7.48).
//!
//! Text is read digit by digit into integers. Nothing goes through a float.
//! Urdu digits (۰-۹) and Arabic-Indic digits (٠-٩) are taken as 0-9. In a sum with `+` or
//! `-`, Arabic-Indic digits are refused (see `sum_terms`).
//! Commas are allowed only where they belong: international `1,316,040` or lakh `13,16,040`,
//! and only in money and quantity. The first group never starts with 0 ("0,500" is refused).
//! A weight or a percent takes no commas at all, because "12,345" typed as a decimal comma
//! for 12.345 kg would otherwise be read as 12,345 kg.

use super::{Bps, Grams, Paisa, Qty};
use crate::error::ErrorCode;
use crate::text;

/// The largest money value one field takes: 10,00,00,00,000 rupees (Code Standards 4.2).
pub const MAX_RUPEES: i64 = 10_000_000_000;
/// The largest quantity one field takes (Code Standards 4.2).
pub const MAX_QTY: i64 = 1_000_000;
/// The largest weight one field takes: 100,000 kg (money agent 2).
pub const MAX_WEIGHT_GRAMS: i64 = 100_000_000;

/// Decimals a percent may have ("2.25").
const BPS_DECIMALS: u32 = 2;
/// Decimals a weight in kg may have ("12.345").
const WEIGHT_DECIMALS: u32 = 3;
/// The largest bps (100%).
const MAX_BPS: u64 = 10_000;

/// Whether a number may have a decimal part.
#[derive(Clone, Copy)]
enum Places {
    /// Whole numbers only: any decimal point is `ERR_DECIMALS_NOT_ALLOWED`.
    Whole,
    /// Up to this many decimals, scaled into an integer (2 for percent, 3 for kg).
    Upto(u32),
}

/// Maps Urdu and Arabic digits and signs to ASCII and trims the text. First it removes the
/// invisible marks of `text::is_ignorable_in_number` (zero-width characters, the word joiner
/// and LRM). RLM, ALM and the bidi controls are kept, so they give `ERR_NOT_A_NUMBER`: they
/// could make the screen show another number than the one read (plan 02 Section 4.6).
/// Visible Urdu punctuation such as '،' and '۔' also stays `ERR_NOT_A_NUMBER`.
fn clean(input: &str) -> String {
    let visible: String = input
        .chars()
        .filter(|&c| !text::is_ignorable_in_number(c))
        .collect();
    visible
        .trim()
        .chars()
        .map(|c| match c {
            '\u{06F0}'..='\u{06F9}' => ascii_digit(c, '\u{06F0}'),
            '\u{0660}'..='\u{0669}' => ascii_digit(c, '\u{0660}'),
            '\u{2212}' => '-',
            '\u{066B}' => '.',
            '\u{066C}' => ',',
            other => other,
        })
        .collect()
}

/// Arabic-Indic digits (U+0660 to U+0669) and the Arabic decimal and thousands separators
/// ٫ ٬ (U+066B, U+066C). Unicode gives them the bidi class AN (Arabic Number).
fn is_arabic_indic(c: char) -> bool {
    text::is_arabic_indic_digit(c) || matches!(c, '\u{066B}' | '\u{066C}')
}

/// The ASCII digit for `c`, where `zero` is the code point of 0 in its script.
fn ascii_digit(c: char, zero: char) -> char {
    let offset = u32::from(c).saturating_sub(u32::from(zero));
    char::from_digit(offset, 10).unwrap_or(c)
}

/// True if `s` is one or more ASCII digits.
fn all_digits(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit())
}

/// The value of an ASCII digit string. Saturates at `u64::MAX`, which is above every
/// maximum, so a very long number is "too large", never wrong.
fn digits_value(s: &str) -> u64 {
    s.bytes().fold(0u64, |acc, b| {
        acc.saturating_mul(10)
            .saturating_add(u64::from(b.saturating_sub(b'0')))
    })
}

/// Reads the whole-number part, with optional commas in international or lakh grouping.
/// `None` when it is not a valid digit string. When `commas_allowed` is false (weight and
/// percent), any comma makes it `None`. The first group of a grouped number never starts
/// with 0: "0,500" and "00,500" are refused, a lone "0" or "00" is a plain number.
fn grouped_value(s: &str, commas_allowed: bool) -> Option<u64> {
    let groups: Vec<&str> = s.split(',').collect();
    if !groups.iter().all(|g| all_digits(g)) {
        return None;
    }
    if let [first, middle @ .., last] = groups.as_slice() {
        if !commas_allowed || first.starts_with('0') {
            return None;
        }
        let middle_len = |n: usize| middle.iter().all(|g| g.len() == n);
        let international = (1..=3).contains(&first.len()) && middle_len(3);
        let lakh = (1..=2).contains(&first.len()) && middle_len(2);
        if last.len() != 3 || !(international || lakh) {
            return None;
        }
    }
    Some(digits_value(&s.replace(',', "")))
}

/// Parses one number with no sign handling beyond a leading `-`.
/// Returns the value scaled by `10^decimals` for `Places::Upto`.
fn parse_number(text: &str, places: Places) -> Result<u64, ErrorCode> {
    let s = clean(text);
    if s.is_empty() {
        return Err(ErrorCode::ErrRequired);
    }
    let (negative, body) = match s.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, s.as_str()),
    };
    let mut parts = body.split('.');
    let int_text = parts.next().unwrap_or_default();
    let frac_text = parts.next();
    if parts.next().is_some() {
        return Err(ErrorCode::ErrNotANumber);
    }
    let int_value = match int_text {
        "" if frac_text.is_some_and(all_digits) => Some(0),
        _ => grouped_value(int_text, matches!(places, Places::Whole)),
    };
    let frac_ok = match frac_text {
        None => true,
        // "850." is a decimal for money (ERR_DECIMALS_NOT_ALLOWED below); for percent and
        // weight a point with nothing after it is not a number (lead decision 10-10-2026).
        Some("") => matches!(places, Places::Whole) && !int_text.is_empty(),
        Some(f) => all_digits(f),
    };
    let int_value = match int_value {
        Some(v) if frac_ok => v,
        _ => return Err(ErrorCode::ErrNotANumber),
    };
    if negative {
        return Err(ErrorCode::ErrNegativeNotAllowed);
    }
    match (places, frac_text) {
        (_, None) => Ok(scale(int_value, 0, places)),
        (Places::Whole, Some(_)) => Err(ErrorCode::ErrDecimalsNotAllowed),
        (Places::Upto(max), Some(f)) => {
            let used = u32::try_from(f.len()).unwrap_or(u32::MAX);
            if used > max {
                return Err(ErrorCode::ErrTooManyDecimals);
            }
            let frac =
                digits_value(f).saturating_mul(10u64.saturating_pow(max.saturating_sub(used)));
            Ok(scale(int_value, frac, places))
        }
    }
}

/// `int × 10^decimals + frac`, saturating (a saturated value is always "too large").
fn scale(int_value: u64, frac: u64, places: Places) -> u64 {
    let decimals = match places {
        Places::Whole => 0,
        Places::Upto(d) => d,
    };
    int_value
        .saturating_mul(10u64.saturating_pow(decimals))
        .saturating_add(frac)
}

/// Money text to whole rupees in paisa (Code Standards M8, money agent 3.1).
///
/// `"1,200"` → `Paisa(120000)`; `"۶۰۰"` → `Paisa(60000)`; `"850.50"` → `ERR_DECIMALS_NOT_ALLOWED`;
/// `"-5"` → `ERR_NEGATIVE_NOT_ALLOWED`; `""` → `ERR_REQUIRED`. Zero is allowed here; a rate
/// must be more than 0, which `validate::rate` checks.
pub fn parse_rupees(text: &str) -> Result<Paisa, ErrorCode> {
    let rupees = parse_number(text, Places::Whole)?;
    rupees_in_range(rupees)
}

/// Checks a rupee value against the maximum and turns it into `Paisa`.
fn rupees_in_range(rupees: u64) -> Result<Paisa, ErrorCode> {
    match i64::try_from(rupees) {
        // r <= 10^10, so r × 100 always fits; the check stays for safety.
        Ok(r) if r <= MAX_RUPEES => r
            .checked_mul(100)
            .map(Paisa)
            .ok_or(ErrorCode::ErrAmountTooLarge),
        _ => Err(ErrorCode::ErrAmountTooLarge),
    }
}

/// Percent text to bps, up to 2 decimals, 0 to 100 (Code Standards 4.2).
///
/// `"6"` → `Bps(600)`; `"2.5"` → `Bps(250)`; `"2.255"` → `ERR_TOO_MANY_DECIMALS`;
/// `"101"` → `ERR_PERCENT_OVER_100`.
pub fn parse_bps(text: &str) -> Result<Bps, ErrorCode> {
    let bps = parse_number(text, Places::Upto(BPS_DECIMALS))?;
    if bps > MAX_BPS {
        return Err(ErrorCode::ErrPercentOver100);
    }
    // Checked against 10,000 above, so it fits i32 and the type's range holds.
    Ok(Bps(i32::try_from(bps).unwrap_or(super::BPS_FULL)))
}

/// Weight text in kg to grams, up to 3 decimals, more than 0 (Code Standards 4.2).
///
/// `"12.345"` → `Grams(12345)`; `"12"` → `Grams(12000)`; `"0"` → `ERR_MUST_BE_MORE_THAN_ZERO`.
pub fn parse_weight(text: &str) -> Result<Grams, ErrorCode> {
    let grams = parse_number(text, Places::Upto(WEIGHT_DECIMALS))?;
    match i64::try_from(grams) {
        Ok(0) => Err(ErrorCode::ErrMustBeMoreThanZero),
        Ok(g) if g <= MAX_WEIGHT_GRAMS => Ok(Grams(g)),
        _ => Err(ErrorCode::ErrWeightTooLarge),
    }
}

/// Quantity text, a whole number from 1 to 1,000,000 (Code Standards 4.2).
///
/// `"20"` → `Qty(20)`; `"0"` → `ERR_MUST_BE_MORE_THAN_ZERO`; `"2.5"` → `ERR_QTY_NOT_WHOLE`.
pub fn parse_qty(text: &str) -> Result<Qty, ErrorCode> {
    let qty = parse_whole_qty(text)?;
    qty_in_range(qty)
}

/// Reads a whole number of نگ. Decimals are `ERR_QTY_NOT_WHOLE`, not the money code
/// (lead decision 10-10-2026, NFR-USE-13: the message names what went wrong).
fn parse_whole_qty(text: &str) -> Result<u64, ErrorCode> {
    match parse_number(text, Places::Whole) {
        Err(ErrorCode::ErrDecimalsNotAllowed) => Err(ErrorCode::ErrQtyNotWhole),
        other => other,
    }
}

/// Checks a quantity against 1 to 1,000,000.
fn qty_in_range(qty: u64) -> Result<Qty, ErrorCode> {
    match i64::try_from(qty) {
        Ok(0) => Err(ErrorCode::ErrMustBeMoreThanZero),
        Ok(q) if q <= MAX_QTY => Ok(Qty(q)),
        _ => Err(ErrorCode::ErrQtyTooLarge),
    }
}

/// Splits a simple sum into signed terms (PRD FR-7.48). `None` when the text is a single
/// value (no `+` or `-` after the first character), so the single-value parser decides.
/// A term that is empty (`"20+"`, `"+20"`) gives `ERR_NOT_A_NUMBER`.
///
/// A sum with any Arabic-Indic digit or separator is `ERR_NOT_A_NUMBER` (lead decision,
/// plan 02 Section 4.6). These are bidi class AN, so a `+` or `-` between them takes the
/// right-to-left direction, even in a `dir="ltr"` field: "١٠٠-٥٠" reads as 50 but shows
/// as "٥٠-١٠٠". The screen must never show a different sum from the one read. Urdu digits
/// (۰-۹) are class EN, like 0-9, keep their order and stay accepted.
fn sum_terms(text: &str) -> Result<Option<Vec<(bool, String)>>, ErrorCode> {
    let s = clean(text);
    let is_op = |c: char| c == '+' || c == '-';
    if !s.chars().skip(1).any(is_op) && !s.starts_with('+') {
        return Ok(None);
    }
    if text.chars().any(is_arabic_indic) {
        return Err(ErrorCode::ErrNotANumber);
    }
    let mut terms = Vec::new();
    let mut current = String::new();
    let mut minus = false;
    for c in s.chars() {
        if is_op(c) {
            terms.push((minus, std::mem::take(&mut current)));
            minus = c == '-';
        } else {
            current.push(c);
        }
    }
    terms.push((minus, current));
    if terms.iter().any(|(_, t)| t.trim().is_empty()) {
        return Err(ErrorCode::ErrNotANumber);
    }
    Ok(Some(terms))
}

/// Adds signed terms with checked maths. `Err(too_large)` if the total cannot be held.
fn add_terms(values: &[(bool, i64)], too_large: ErrorCode) -> Result<i64, ErrorCode> {
    values.iter().try_fold(0i64, |acc, &(minus, v)| {
        let next = if minus {
            acc.checked_sub(v)
        } else {
            acc.checked_add(v)
        };
        next.ok_or(too_large)
    })
}

/// Money text that may be a simple sum: `"20+25"` → 45 rupees, `"500-200"` → 300
/// (PRD FR-7.48, money agent 3.1). Each term is checked like a typed value; a total
/// below 0 is `ERR_NEGATIVE_NOT_ALLOWED`; above the maximum, `ERR_AMOUNT_TOO_LARGE`.
pub fn parse_sum_rupees(text: &str) -> Result<Paisa, ErrorCode> {
    let Some(terms) = sum_terms(text)? else {
        return parse_rupees(text);
    };
    let mut values = Vec::with_capacity(terms.len());
    for (minus, term) in &terms {
        values.push((*minus, parse_rupees(term)?.rupees()));
    }
    let total = add_terms(&values, ErrorCode::ErrAmountTooLarge)?;
    match u64::try_from(total) {
        Ok(t) => rupees_in_range(t),
        Err(_) => Err(ErrorCode::ErrNegativeNotAllowed),
    }
}

/// Quantity text that may be a simple sum: `"20+25"` → 45 (PRD FR-7.48). A term may be 0;
/// the total must be 1 to 1,000,000.
pub fn parse_sum_qty(text: &str) -> Result<Qty, ErrorCode> {
    let Some(terms) = sum_terms(text)? else {
        return parse_qty(text);
    };
    let mut values = Vec::with_capacity(terms.len());
    for (minus, term) in &terms {
        let v = parse_whole_qty(term)?;
        match i64::try_from(v) {
            Ok(q) if q <= MAX_QTY => values.push((*minus, q)),
            _ => return Err(ErrorCode::ErrQtyTooLarge),
        }
    }
    let total = add_terms(&values, ErrorCode::ErrQtyTooLarge)?;
    match u64::try_from(total) {
        Ok(t) => qty_in_range(t),
        Err(_) => Err(ErrorCode::ErrNegativeNotAllowed),
    }
}
