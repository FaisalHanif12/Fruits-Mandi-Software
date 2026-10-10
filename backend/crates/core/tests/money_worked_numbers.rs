//! Every worked number of plan 02 Section 3 (money-calculations agent 3.1, 4.2 to 4.4,
//! 5.1 to 5.3). One named test per row. The expected numbers come from the PRD and the
//! money agent and are never changed to fit the code.
#![cfg(test)]

use mandi_core::error::ErrorCode;
use mandi_core::money::{
    Bps, Grams, Grouping, MoneyError, Paisa, Qty, apply_bps, avg_rate, format_paisa, line_value,
    parse_bps, parse_qty, parse_rupees, parse_sum_qty, parse_sum_rupees, parse_weight, per_unit,
    split_by_bps, split_by_qty, weight_value,
};

fn rs(r: i64) -> Paisa {
    Paisa::from_rupees(r).unwrap()
}

fn bps(v: i32) -> Bps {
    Bps::new(v).unwrap()
}

fn qty(v: i64) -> Qty {
    Qty::new(v).unwrap()
}

/// Partners الف (id 1), ب (id 2), ج (id 3) at 40 / 30 / 30 (PRD 4.2).
fn partners_40_30_30() -> [(u32, Bps); 3] {
    [(1, bps(4_000)), (2, bps(3_000)), (3, bps(3_000))]
}

/// Splits whole rupees at 40/30/30 and returns the parts in rupees.
fn split_403030(amount: i64) -> Vec<i64> {
    split_by_bps(rs(amount), &partners_40_30_30())
        .unwrap()
        .into_iter()
        .map(|(_, p)| p.rupees())
        .collect()
}

// ---- 3.1 parse_rupees ----

#[test]
fn parse_rupees_600() {
    assert_eq!(
        parse_rupees("600"),
        Ok(Paisa::try_from_paisa(60_000).unwrap())
    );
}

#[test]
fn parse_rupees_1_200_with_comma() {
    assert_eq!(
        parse_rupees("1,200"),
        Ok(Paisa::try_from_paisa(120_000).unwrap())
    );
}

#[test]
fn parse_rupees_trims_spaces() {
    assert_eq!(parse_rupees("  600 "), Ok(rs(600)));
}

#[test]
fn parse_rupees_urdu_digits() {
    assert_eq!(parse_rupees("۶۰۰"), Ok(rs(600)));
    assert_eq!(parse_rupees("٦٠٠"), Ok(rs(600)));
}

#[test]
fn parse_rupees_empty_is_required() {
    assert_eq!(parse_rupees(""), Err(ErrorCode::ErrRequired));
    assert_eq!(parse_rupees("   "), Err(ErrorCode::ErrRequired));
}

#[test]
fn parse_rupees_decimals_rejected() {
    for text in ["850.50", "850.0", "850.", ".5", "1,200.50"] {
        assert_eq!(
            parse_rupees(text),
            Err(ErrorCode::ErrDecimalsNotAllowed),
            "{text}"
        );
    }
}

#[test]
fn parse_rupees_not_a_number() {
    for text in [
        "abc",
        "1e5",
        "12,34,5x",
        "1.2.3",
        "1 200",
        "+5",
        "-",
        "1,00,0",
        ",100",
        "1,,000",
        "100,",
        "1,00,0.5",
        "1,000,00",
        "1,00,000,000",
        "١٢،٣",
    ] {
        assert_eq!(parse_rupees(text), Err(ErrorCode::ErrNotANumber), "{text}");
    }
}

#[test]
fn parse_rupees_negative_rejected() {
    for text in ["-5", "−5", "-850.50", "-0"] {
        assert_eq!(
            parse_rupees(text),
            Err(ErrorCode::ErrNegativeNotAllowed),
            "{text}"
        );
    }
}

#[test]
fn parse_rupees_maximum_and_above() {
    assert_eq!(parse_rupees("10,00,00,00,000"), Ok(rs(10_000_000_000)));
    assert_eq!(parse_rupees("10,000,000,000"), Ok(rs(10_000_000_000)));
    for text in [
        "10,00,00,00,001",
        "10,000,000,001",
        "99999999999999999999999999999999",
    ] {
        assert_eq!(
            parse_rupees(text),
            Err(ErrorCode::ErrAmountTooLarge),
            "{text}"
        );
    }
}

#[test]
fn parse_rupees_grouping_forms() {
    assert_eq!(parse_rupees("1,00,000"), Ok(rs(100_000)));
    assert_eq!(parse_rupees("13,16,040"), Ok(rs(1_316_040)));
    assert_eq!(parse_rupees("1,316,040"), Ok(rs(1_316_040)));
    assert_eq!(parse_rupees("007"), Ok(rs(7)));
    assert_eq!(parse_rupees("0"), Ok(Paisa::ZERO));
    assert_eq!(parse_rupees("1٬200"), Ok(rs(1_200)));
}

// ---- 3.1 parse_bps ----

#[test]
fn parse_bps_6_percent() {
    assert_eq!(parse_bps("6"), Ok(bps(600)));
}

#[test]
fn parse_bps_2_5_percent() {
    assert_eq!(parse_bps("2.5"), Ok(bps(250)));
}

#[test]
fn parse_bps_2_25_percent() {
    assert_eq!(parse_bps("2.25"), Ok(bps(225)));
}

#[test]
fn parse_bps_too_many_decimals() {
    assert_eq!(parse_bps("2.255"), Err(ErrorCode::ErrTooManyDecimals));
    assert_eq!(parse_bps("2.250"), Err(ErrorCode::ErrTooManyDecimals));
}

#[test]
fn parse_bps_over_100() {
    assert_eq!(parse_bps("101"), Err(ErrorCode::ErrPercentOver100));
    assert_eq!(parse_bps("100.01"), Err(ErrorCode::ErrPercentOver100));
    assert_eq!(
        parse_bps("9".repeat(40).as_str()),
        Err(ErrorCode::ErrPercentOver100)
    );
    assert_eq!(parse_bps("100"), Ok(Bps::FULL));
}

#[test]
fn parse_bps_other_forms() {
    assert_eq!(parse_bps(".5"), Ok(bps(50)));
    assert_eq!(parse_bps("6."), Err(ErrorCode::ErrNotANumber));
    assert_eq!(parse_bps("0"), Ok(bps(0)));
    assert_eq!(parse_bps("۲٫۵"), Ok(bps(250)));
    assert_eq!(parse_bps(""), Err(ErrorCode::ErrRequired));
    assert_eq!(parse_bps("-1"), Err(ErrorCode::ErrNegativeNotAllowed));
    assert_eq!(parse_bps("abc"), Err(ErrorCode::ErrNotANumber));
    assert_eq!(parse_bps("2.x"), Err(ErrorCode::ErrNotANumber));
    assert_eq!(parse_bps("."), Err(ErrorCode::ErrNotANumber));
}

// ---- 3.1 parse_weight ----

#[test]
fn parse_weight_12_345_kg() {
    assert_eq!(parse_weight("12.345"), Ok(Grams::new(12_345).unwrap()));
}

#[test]
fn parse_weight_12_kg() {
    assert_eq!(parse_weight("12"), Ok(Grams::new(12_000).unwrap()));
}

#[test]
fn parse_weight_too_many_decimals() {
    assert_eq!(parse_weight("12.3456"), Err(ErrorCode::ErrTooManyDecimals));
}

#[test]
fn parse_weight_zero() {
    assert_eq!(parse_weight("0"), Err(ErrorCode::ErrMustBeMoreThanZero));
    assert_eq!(parse_weight("0.000"), Err(ErrorCode::ErrMustBeMoreThanZero));
}

#[test]
fn parse_weight_limits() {
    assert_eq!(parse_weight("0.001"), Ok(Grams::new(1).unwrap()));
    assert_eq!(parse_weight("100000"), Ok(Grams::new(100_000_000).unwrap()));
    assert_eq!(
        parse_weight("100000.001"),
        Err(ErrorCode::ErrWeightTooLarge)
    );
    assert_eq!(
        parse_weight("9".repeat(40).as_str()),
        Err(ErrorCode::ErrWeightTooLarge)
    );
    assert_eq!(parse_weight("-1"), Err(ErrorCode::ErrNegativeNotAllowed));
}

// ---- parse_qty (Code Standards 4.2) ----

#[test]
fn parse_qty_cases() {
    assert_eq!(parse_qty("20"), Ok(qty(20)));
    assert_eq!(parse_qty("1,000,000"), Ok(qty(1_000_000)));
    assert_eq!(parse_qty("0"), Err(ErrorCode::ErrMustBeMoreThanZero));
    assert_eq!(parse_qty("1000001"), Err(ErrorCode::ErrQtyTooLarge));
    assert_eq!(
        parse_qty("9".repeat(40).as_str()),
        Err(ErrorCode::ErrQtyTooLarge)
    );
    assert_eq!(parse_qty("2.5"), Err(ErrorCode::ErrQtyNotWhole));
    assert_eq!(parse_qty("20.0"), Err(ErrorCode::ErrQtyNotWhole));
    assert_eq!(parse_qty("20."), Err(ErrorCode::ErrQtyNotWhole));
    assert_eq!(parse_qty("-3"), Err(ErrorCode::ErrNegativeNotAllowed));
    assert_eq!(parse_qty("x"), Err(ErrorCode::ErrNotANumber));
}

// ---- 3.1 parse_sum (PRD FR-7.48) ----

#[test]
fn parse_sum_20_plus_25() {
    assert_eq!(
        parse_sum_rupees("20+25"),
        Ok(Paisa::try_from_paisa(4_500).unwrap())
    );
}

#[test]
fn parse_sum_1_200_plus_300() {
    assert_eq!(
        parse_sum_rupees("1,200+300"),
        Ok(Paisa::try_from_paisa(150_000).unwrap())
    );
}

#[test]
fn parse_sum_500_minus_200() {
    assert_eq!(
        parse_sum_rupees("500-200"),
        Ok(Paisa::try_from_paisa(30_000).unwrap())
    );
}

#[test]
fn parse_sum_10_minus_20_is_negative() {
    assert_eq!(
        parse_sum_rupees("10-20"),
        Err(ErrorCode::ErrNegativeNotAllowed)
    );
}

#[test]
fn parse_sum_bad_forms() {
    for text in [
        "20+", "+20", "2*3", "6/2", "2×3", "6÷2", "-5+1", "2 0+5", "20++5",
    ] {
        assert_eq!(
            parse_sum_rupees(text),
            Err(ErrorCode::ErrNotANumber),
            "{text}"
        );
    }
}

#[test]
fn parse_sum_decimal_term() {
    assert_eq!(
        parse_sum_rupees("20.5+1"),
        Err(ErrorCode::ErrDecimalsNotAllowed)
    );
    // The first failing term decides the code.
    assert_eq!(
        parse_sum_rupees("20.5+abc"),
        Err(ErrorCode::ErrDecimalsNotAllowed)
    );
    assert_eq!(parse_sum_rupees("abc+20.5"), Err(ErrorCode::ErrNotANumber));
}

#[test]
fn parse_sum_above_maximum() {
    assert_eq!(
        parse_sum_rupees("10,00,00,00,000+1"),
        Err(ErrorCode::ErrAmountTooLarge)
    );
    assert_eq!(
        parse_sum_rupees("10000000001+0"),
        Err(ErrorCode::ErrAmountTooLarge)
    );
    assert_eq!(
        parse_sum_rupees("10,00,00,00,000+1-1"),
        Ok(rs(10_000_000_000))
    );
}

#[test]
fn parse_sum_spaces_and_single_values() {
    assert_eq!(parse_sum_rupees(" 20 + 25 "), Ok(rs(45)));
    assert_eq!(parse_sum_rupees("10-20+15"), Ok(rs(5)));
    assert_eq!(parse_sum_rupees("600"), Ok(rs(600)));
    assert_eq!(
        parse_sum_rupees("-5"),
        Err(ErrorCode::ErrNegativeNotAllowed)
    );
    assert_eq!(parse_sum_rupees(""), Err(ErrorCode::ErrRequired));
    assert_eq!(parse_sum_rupees("5−2"), Ok(rs(3)));
}

/// Lead decision 10-10-2026: decimals in a quantity get their own code; money keeps
/// ERR_DECIMALS_NOT_ALLOWED (PRD E31, NFR-USE-13).
#[test]
fn qty_decimals_are_qty_not_whole_money_decimals_are_not() {
    assert_eq!(parse_qty("2.5"), Err(ErrorCode::ErrQtyNotWhole));
    assert_eq!(parse_sum_qty("2.5"), Err(ErrorCode::ErrQtyNotWhole));
    assert_eq!(parse_rupees("2.5"), Err(ErrorCode::ErrDecimalsNotAllowed));
    assert_eq!(
        parse_sum_rupees("2.5"),
        Err(ErrorCode::ErrDecimalsNotAllowed)
    );
}

#[test]
fn parse_sum_qty_cases() {
    assert_eq!(parse_sum_qty("20+25"), Ok(qty(45)));
    assert_eq!(parse_sum_qty("20+0"), Ok(qty(20)));
    assert_eq!(parse_sum_qty("20"), Ok(qty(20)));
    assert_eq!(parse_sum_qty("5-5"), Err(ErrorCode::ErrMustBeMoreThanZero));
    assert_eq!(parse_sum_qty("5-6"), Err(ErrorCode::ErrNegativeNotAllowed));
    assert_eq!(parse_sum_qty("1000000+1"), Err(ErrorCode::ErrQtyTooLarge));
    assert_eq!(parse_sum_qty("1000001+0"), Err(ErrorCode::ErrQtyTooLarge));
    assert_eq!(parse_sum_qty("2.5+1"), Err(ErrorCode::ErrQtyNotWhole));
    assert_eq!(parse_sum_qty("20+2.5"), Err(ErrorCode::ErrQtyNotWhole));
    assert_eq!(parse_sum_qty("20+"), Err(ErrorCode::ErrNotANumber));
}

// ---- 3.2 format_paisa ----

#[test]
fn format_paisa_grouping() {
    assert_eq!(
        format_paisa(rs(12_000), Grouping::International).unwrap(),
        "12,000"
    );
    assert_eq!(
        format_paisa(rs(1_316_040), Grouping::International).unwrap(),
        "1,316,040"
    );
    assert_eq!(
        format_paisa(rs(1_316_040), Grouping::Lakh).unwrap(),
        "13,16,040"
    );
}

// ---- 4.2 apply_bps ----

#[test]
fn apply_bps_prd_4_4_first_line_720() {
    assert_eq!(apply_bps(rs(12_000), bps(600)), Ok(rs(720)));
}

#[test]
fn apply_bps_old_video_row_338() {
    assert_eq!(apply_bps(rs(5_630), bps(600)), Ok(rs(338)));
}

#[test]
fn apply_bps_exact_half_rounds_up_338() {
    assert_eq!(apply_bps(rs(5_625), bps(600)), Ok(rs(338)));
}

#[test]
fn apply_bps_just_under_half_337() {
    assert_eq!(apply_bps(rs(5_624), bps(600)), Ok(rs(337)));
}

#[test]
fn apply_bps_buyer_commission_870() {
    assert_eq!(apply_bps(rs(17_400), bps(500)), Ok(rs(870)));
}

#[test]
fn apply_bps_two_and_half_percent_25() {
    assert_eq!(apply_bps(rs(1_000), bps(250)), Ok(rs(25)));
}

#[test]
fn apply_bps_zero_base_0() {
    assert_eq!(apply_bps(Paisa::ZERO, bps(600)), Ok(Paisa::ZERO));
}

#[test]
fn apply_bps_negative_base_is_a_bug() {
    assert_eq!(apply_bps(rs(-1), bps(600)), Err(MoneyError::NegativeBase));
}

// ---- 4.3 weight_value and per-unit ----

#[test]
fn weight_value_12_345_kg_at_100_is_1_235() {
    assert_eq!(
        weight_value(Grams::new(12_345).unwrap(), rs(100)),
        Ok(rs(1_235))
    );
}

#[test]
fn per_unit_10_times_20_is_200() {
    assert_eq!(per_unit(qty(10), rs(20)), Ok(rs(200)));
}

#[test]
fn line_value_20_times_600_is_12_000() {
    assert_eq!(line_value(qty(20), rs(600)), Ok(rs(12_000)));
    assert_eq!(line_value(qty(20), rs(-1)), Err(MoneyError::NegativeBase));
}

// ---- 4.4 avg_rate ----

#[test]
fn avg_rate_29_400_over_50_is_588() {
    assert_eq!(avg_rate(rs(29_400), qty(50)), Ok(rs(588)));
}

#[test]
fn avg_rate_bad_inputs() {
    assert_eq!(
        avg_rate(rs(29_400), qty(0)),
        Err(MoneyError::DivisionByZero)
    );
    assert_eq!(avg_rate(rs(-1), qty(1)), Err(MoneyError::NegativeBase));
}

// ---- 5.1 partner split (PRD C10) ----

#[test]
fn split_3_948_prd_4_4_d1_a6() {
    assert_eq!(split_403030(3_948), [1_580, 1_184, 1_184]);
}

#[test]
fn split_1_540_prd_4_5() {
    assert_eq!(split_403030(1_540), [616, 462, 462]);
}

#[test]
fn split_3_972_e2e_13() {
    assert_eq!(split_403030(3_972), [1_590, 1_191, 1_191]);
}

#[test]
fn split_4_094_e2e_13b() {
    assert_eq!(split_403030(4_094), [1_638, 1_228, 1_228]);
}

#[test]
fn split_3_560_e2e_16() {
    assert_eq!(split_403030(3_560), [1_424, 1_068, 1_068]);
}

#[test]
fn split_376_e2e_16b() {
    assert_eq!(split_403030(376), [152, 112, 112]);
}

#[test]
fn split_thirds_1_000() {
    let shares = [(1u32, bps(3_333)), (2, bps(3_333)), (3, bps(3_334))];
    let parts: Vec<i64> = split_by_bps(rs(1_000), &shares)
        .unwrap()
        .into_iter()
        .map(|(_, p)| p.rupees())
        .collect();
    assert_eq!(parts, [333, 333, 334]);
}

#[test]
fn split_equal_shares_tie_1_001() {
    // Ids given out of order: the leftover still goes to the lower id (created first).
    let shares = [(7u32, bps(5_000)), (3, bps(5_000))];
    let parts = split_by_bps(rs(1_001), &shares).unwrap();
    assert_eq!(parts, [(7, rs(500)), (3, rs(501))]);
}

#[test]
fn split_single_partner_777() {
    let parts = split_by_bps(rs(777), &[(1u32, Bps::FULL)]).unwrap();
    assert_eq!(parts, [(1, rs(777))]);
}

#[test]
fn split_zero() {
    assert_eq!(split_403030(0), [0, 0, 0]);
}

#[test]
fn split_shares_not_100_e9() {
    let shares = [(1u32, bps(4_000)), (2, bps(3_000)), (3, bps(2_999))];
    assert_eq!(
        split_by_bps(rs(100), &shares),
        Err(MoneyError::SharesNot100)
    );
    assert_eq!(MoneyError::SharesNot100.code(), ErrorCode::ErrSharesNot100);
}

// ---- 5.2 negative amounts (PRD C11) ----

#[test]
fn split_expense_3_000() {
    assert_eq!(split_403030(-3_000), [-1_200, -900, -900]);
}

#[test]
fn split_expense_3_384_e2e_20() {
    assert_eq!(split_403030(-3_384), [-1_354, -1_015, -1_015]);
}

#[test]
fn split_negative_income_minus_100() {
    assert_eq!(split_403030(-100), [-40, -30, -30]);
}

// ---- 5.3 split_by_qty (PRD C17) ----

#[test]
fn split_by_qty_1_000_over_30_and_20() {
    let parts = split_by_qty(rs(1_000), &[(1u32, qty(30)), (2, qty(20))]).unwrap();
    assert_eq!(parts, [(1, rs(600)), (2, rs(400))]);
}

#[test]
fn split_by_qty_1_000_over_1_1_1() {
    let parts = split_by_qty(rs(1_000), &[(1u32, qty(1)), (2, qty(1)), (3, qty(1))]).unwrap();
    assert_eq!(parts, [(1, rs(334)), (2, rs(333)), (3, rs(333))]);
}

#[test]
fn split_by_qty_zero_total_is_a_bug() {
    assert_eq!(
        split_by_qty(rs(1_000), &[(1u32, qty(0))]),
        Err(MoneyError::DivisionByZero)
    );
}

// ---- Largest values (money agent 11.4) ----

#[test]
fn largest_allowed_values_do_not_overflow() {
    let max_rate = rs(10_000_000_000);
    let line = line_value(qty(1_000_000), max_rate).unwrap();
    assert_eq!(line.rupees(), 10_000_000_000_000_000);
    assert_eq!(apply_bps(line, Bps::FULL), Ok(line));
    let by_weight = weight_value(Grams::new(100_000_000).unwrap(), max_rate).unwrap();
    assert_eq!(by_weight.rupees(), 1_000_000_000_000_000);
    // Ten such lines no longer fit: an error, never a wrap.
    let total = (0..10).try_fold(Paisa::ZERO, |acc, _| acc.checked_add(line));
    assert_eq!(total, Err(MoneyError::Overflow));
}

#[test]
fn overflow_returns_errors() {
    let big = Paisa::try_from_paisa(i64::MAX - i64::MAX % 100).unwrap();
    assert_eq!(big.checked_add(rs(1)), Err(MoneyError::Overflow));
    let small = Paisa::try_from_paisa(i64::MIN - i64::MIN % 100).unwrap();
    assert_eq!(small.checked_sub(rs(1)), Err(MoneyError::Overflow));
    assert_eq!(Paisa::from_rupees(i64::MAX), Err(MoneyError::Overflow));
    assert_eq!(line_value(qty(i64::MAX), rs(2)), Err(MoneyError::Overflow));
    assert_eq!(
        weight_value(Grams::new(i64::MAX).unwrap(), big),
        Err(MoneyError::Overflow)
    );
    assert_eq!(apply_bps(big, Bps::FULL), Ok(big));
}

#[test]
fn types_reject_values_out_of_range() {
    assert_eq!(Paisa::try_from_paisa(150), Err(MoneyError::NotWholeRupee));
    assert_eq!(Bps::new(10_001), Err(MoneyError::BpsOutOfRange));
    assert_eq!(Bps::new(-1), Err(MoneyError::BpsOutOfRange));
    assert_eq!(Qty::new(-1), Err(MoneyError::NegativeValue));
    assert_eq!(Grams::new(-1), Err(MoneyError::NegativeValue));
    assert_eq!(Bps::FULL.get(), 10_000);
    assert_eq!(Qty::new(5).unwrap().get(), 5);
    assert_eq!(Grams::new(5).unwrap().get(), 5);
    assert!(rs(-5).is_negative());
    assert_eq!(rs(5).checked_neg(), Ok(rs(-5)));
    assert_eq!(rs(5).checked_sub(rs(7)), Ok(rs(-2)));
    assert_eq!(i64::from(rs(5)), 500);
}

#[test]
fn paisa_is_a_json_integer() {
    assert_eq!(serde_json::to_string(&rs(12_000)).unwrap(), "1200000");
    assert_eq!(
        serde_json::from_str::<Paisa>("1200000").unwrap(),
        rs(12_000)
    );
    assert!(serde_json::from_str::<Paisa>("150").is_err());
    assert!(serde_json::from_str::<Paisa>("1.5").is_err());
    assert_eq!(Paisa::try_from(500i64), Ok(rs(5)));
}

#[test]
fn split_by_qty_tie_goes_to_lowest_lot_number() {
    // Lots given out of order: on a tie in نگ the lowest lot number takes the leftover.
    let parts = split_by_qty(rs(1_001), &[(4u32, qty(10)), (2, qty(10))]).unwrap();
    assert_eq!(parts, [(4, rs(500)), (2, rs(501))]);
}

// ---- Lead decisions 10-10-2026 on number text ----

/// A point with nothing after it: money is a decimal, percent and weight are not numbers.
#[test]
fn trailing_point_is_refused_everywhere() {
    assert_eq!(parse_rupees("850."), Err(ErrorCode::ErrDecimalsNotAllowed));
    assert_eq!(parse_bps("6."), Err(ErrorCode::ErrNotANumber));
    assert_eq!(parse_weight("12."), Err(ErrorCode::ErrNotANumber));
    assert_eq!(parse_qty("20."), Err(ErrorCode::ErrQtyNotWhole));
}

/// Zero-width marks, the word joiner, LRM and BOM are removed before a number is read
/// (plan 02 Section 4.6, robustness finding R02-1).
#[test]
fn invisible_marks_are_stripped_from_numbers() {
    let marks = [
        '\u{200B}', '\u{200C}', '\u{200D}', '\u{200E}', '\u{2060}', '\u{FEFF}',
    ];
    for m in marks {
        let at = format!("U+{:04X}", u32::from(m));
        assert_eq!(
            parse_rupees(&format!("{m}1,{m}200{m}")),
            Ok(rs(1_200)),
            "{at}"
        );
        assert_eq!(
            parse_sum_rupees(&format!("20{m}+{m}25")),
            Ok(rs(45)),
            "{at}"
        );
        assert_eq!(parse_bps(&format!("{m}2.5{m}")), Ok(bps(250)), "{at}");
        assert_eq!(
            parse_weight(&format!("{m}12.345")),
            Ok(Grams::new(12_345).unwrap()),
            "{at}"
        );
        assert_eq!(parse_qty(&format!("2{m}0")), Ok(qty(20)), "{at}");
        assert_eq!(parse_sum_qty(&format!("{m}20+25")), Ok(qty(45)), "{at}");
    }
    // Only marks: nothing was typed.
    assert_eq!(
        parse_rupees("\u{200E}\u{FEFF}"),
        Err(ErrorCode::ErrRequired)
    );
    // The Arabic letter mark is not on the list and is still refused.
    assert_eq!(parse_rupees("1\u{061C}200"), Err(ErrorCode::ErrNotANumber));
}

/// Finding R02-1: the word joiner (U+2060) is zero-width like ZWSP and BOM, so a number
/// pasted with it is read. It is one shared list with the name cleaner (`text::is_invisible`).
#[test]
fn word_joiner_is_stripped_from_numbers_r02_1() {
    assert_eq!(parse_rupees("6\u{2060}00"), Ok(rs(600)));
    assert_eq!(parse_rupees("\u{2060}600\u{2060}"), Ok(rs(600)));
    assert_eq!(parse_sum_rupees("500\u{2060}-\u{2060}200"), Ok(rs(300)));
    assert_eq!(parse_sum_qty("20\u{2060}+25"), Ok(qty(45)));
    // Only a word joiner: nothing was typed.
    assert_eq!(parse_rupees("\u{2060}"), Err(ErrorCode::ErrRequired));
}

/// RLM and the bidi embeddings, overrides and isolates can show digits in another order
/// than the one read, so they are refused, never removed (plan 02 Section 4.6).
#[test]
fn bidi_controls_in_numbers_are_refused() {
    let controls = [
        '\u{200F}', '\u{202A}', '\u{202B}', '\u{202C}', '\u{202D}', '\u{202E}', '\u{2066}',
        '\u{2067}', '\u{2068}', '\u{2069}',
    ];
    for m in controls {
        let at = format!("U+{:04X}", u32::from(m));
        let nan = Some(ErrorCode::ErrNotANumber);
        assert_eq!(parse_rupees(&format!("{m}1,200")).err(), nan, "{at}");
        assert_eq!(parse_rupees(&format!("1,200{m}")).err(), nan, "{at}");
        assert_eq!(parse_sum_rupees(&format!("500-{m}200")).err(), nan, "{at}");
        assert_eq!(parse_bps(&format!("{m}2.5")).err(), nan, "{at}");
        assert_eq!(parse_weight(&format!("{m}12.345")).err(), nan, "{at}");
        assert_eq!(parse_qty(&format!("{m}20")).err(), nan, "{at}");
        assert_eq!(parse_sum_qty(&format!("{m}20+25")).err(), nan, "{at}");
    }
}

/// RLM is a strong right-to-left character, so it is refused, not removed (review
/// finding 1, plan 02 Section 4.6).
#[test]
fn rlm_in_numbers_is_refused() {
    // "500<RLM>-200" could show reordered while it reads as 300.
    assert_eq!(
        parse_sum_rupees("500\u{200F}-200"),
        Err(ErrorCode::ErrNotANumber)
    );
    assert_eq!(
        parse_sum_qty("500\u{200F}-200"),
        Err(ErrorCode::ErrNotANumber)
    );
    // "<RLM>12".
    assert_eq!(parse_rupees("\u{200F}12"), Err(ErrorCode::ErrNotANumber));
    assert_eq!(parse_qty("\u{200F}12"), Err(ErrorCode::ErrNotANumber));
    assert_eq!(parse_bps("\u{200F}12"), Err(ErrorCode::ErrNotANumber));
    assert_eq!(parse_weight("\u{200F}12"), Err(ErrorCode::ErrNotANumber));
}

/// The three Trojan Source cases (CVE-2021-42574): each would show a different number
/// from the one saved, so each is refused.
#[test]
fn trojan_source_rlo_cases_are_refused() {
    // "<RLO>12" would show "21".
    assert_eq!(parse_rupees("\u{202E}12"), Err(ErrorCode::ErrNotANumber));
    // "<RLO>1,200" would show "002,1".
    assert_eq!(parse_rupees("\u{202E}1,200"), Err(ErrorCode::ErrNotANumber));
    // "<RLO>20+25" would show "52+02".
    assert_eq!(
        parse_sum_rupees("\u{202E}20+25"),
        Err(ErrorCode::ErrNotANumber)
    );
    assert_eq!(
        parse_sum_qty("\u{202E}20+25"),
        Err(ErrorCode::ErrNotANumber)
    );
}

/// Visible Urdu punctuation stays refused: '،' (U+060C) and '۔' (U+06D4).
#[test]
fn urdu_comma_and_full_stop_are_not_numbers() {
    for text in ["1،200", "850۔50", "۱،۲۰۰"] {
        assert_eq!(parse_rupees(text), Err(ErrorCode::ErrNotANumber), "{text}");
        assert_eq!(
            parse_sum_rupees(text),
            Err(ErrorCode::ErrNotANumber),
            "{text}"
        );
        assert_eq!(parse_qty(text), Err(ErrorCode::ErrNotANumber), "{text}");
        assert_eq!(parse_sum_qty(text), Err(ErrorCode::ErrNotANumber), "{text}");
    }
    assert_eq!(parse_bps("2۔5"), Err(ErrorCode::ErrNotANumber));
    assert_eq!(parse_weight("12۔345"), Err(ErrorCode::ErrNotANumber));
}

// ---- Lead decision 10-10-2026: Arabic-Indic digits in sums (plan 02 Section 4.6) ----

/// Arabic-Indic digits are bidi class AN: "١٠٠-٥٠" reads as 50 but shows as "٥٠-١٠٠",
/// so a sum with them is refused in both sum parsers.
#[test]
fn arabic_indic_sums_are_refused() {
    for text in [
        "١٠٠-٥٠",
        "١٠٠+٥٠",
        "100-٥٠",
        "١٠٠+50",
        "۱۰۰-٥٠",
        "١٬٢٠٠+300",
        "20+٢٫٥",
        "١٠٠\u{2212}٥٠",
        " ١٠٠ - ٥٠ ",
    ] {
        assert_eq!(
            parse_sum_rupees(text),
            Err(ErrorCode::ErrNotANumber),
            "{text}"
        );
        assert_eq!(parse_sum_qty(text), Err(ErrorCode::ErrNotANumber), "{text}");
    }
}

/// A single Arabic-Indic value is still read (money agent 3.1), in every money and qty parser.
#[test]
fn arabic_indic_single_values_still_accepted() {
    assert_eq!(parse_rupees("١٢٠٠"), Ok(rs(1_200)));
    assert_eq!(parse_rupees("١,٢٠٠"), Ok(rs(1_200)));
    assert_eq!(parse_rupees("١٬٢٠٠"), Ok(rs(1_200)));
    assert_eq!(parse_sum_rupees("١٢٠٠"), Ok(rs(1_200)));
    assert_eq!(parse_sum_rupees("١,٢٠٠"), Ok(rs(1_200)));
    assert_eq!(parse_sum_qty("٢٠"), Ok(qty(20)));
    // A leading minus is not a sum: it stays the negative code.
    assert_eq!(
        parse_sum_rupees("-٥"),
        Err(ErrorCode::ErrNegativeNotAllowed)
    );
}

/// Urdu digits (bidi class EN) keep their order, so sums with them stay accepted.
#[test]
fn urdu_digit_sums_still_accepted() {
    assert_eq!(parse_sum_rupees("۱۰۰-۵۰"), Ok(rs(50)));
    assert_eq!(parse_sum_rupees("۱۰۰+۵۰"), Ok(rs(150)));
    assert_eq!(parse_sum_qty("۱۰۰-۵۰"), Ok(qty(50)));
    assert_eq!(parse_sum_rupees("100-۵۰"), Ok(rs(50)));
}

// ---- Money agent review, round R02: extra cases for the number text rules ----

/// The sum rule reads the raw text, so a zero-width mark next to an Arabic-Indic digit does
/// not hide it. A term that is empty is not a number.
#[test]
fn arabic_indic_sum_edge_cases() {
    for text in ["١٠٠\u{200B}-٥٠", "١٠٠-\u{2060}٥٠", "+١٠", "١٠+", "١٠+٥٠+٣٠"] {
        assert_eq!(
            parse_sum_rupees(text),
            Err(ErrorCode::ErrNotANumber),
            "{text}"
        );
        assert_eq!(parse_sum_qty(text), Err(ErrorCode::ErrNotANumber), "{text}");
    }
}

/// A single Arabic-Indic value is read for percent and weight too, with ٫ as the point.
#[test]
fn arabic_indic_single_values_for_percent_and_weight() {
    assert_eq!(parse_bps("٢٫٥"), Ok(bps(250)));
    assert_eq!(parse_bps("٦"), Ok(bps(600)));
    assert_eq!(parse_weight("١٢٫٣٤٥"), Ok(Grams::new(12_345).unwrap()));
    assert_eq!(parse_qty("٢٠"), Ok(qty(20)));
}

/// A sum is judged by its terms (each is checked like a typed value) and then by its total.
#[test]
fn sum_terms_and_total_are_checked_separately() {
    // The total is what counts, in any order of terms.
    assert_eq!(parse_sum_rupees("5-10+20"), Ok(rs(15)));
    assert_eq!(parse_sum_rupees("10-20+15"), Ok(rs(5)));
    // Terms at the maximum are fine while the total stays inside it.
    assert_eq!(
        parse_sum_rupees("10000000000+10000000000-10000000000"),
        Ok(rs(10_000_000_000))
    );
    // A term above the maximum is refused, even if the total would fit.
    assert_eq!(
        parse_sum_rupees("10000000000+10000000000-15000000000"),
        Err(ErrorCode::ErrAmountTooLarge)
    );
    // Every term fits, but the total does not.
    assert_eq!(
        parse_sum_rupees("10000000000+10000000000-5000000000"),
        Err(ErrorCode::ErrAmountTooLarge)
    );
    // A term of 0 is allowed in a quantity sum; the total must be 1 or more.
    assert_eq!(parse_sum_qty("0+5"), Ok(qty(5)));
    assert_eq!(parse_sum_qty("5-0"), Ok(qty(5)));
    assert_eq!(parse_sum_qty("5-5"), Err(ErrorCode::ErrMustBeMoreThanZero));
    assert_eq!(parse_sum_qty("3-5"), Err(ErrorCode::ErrNegativeNotAllowed));
}

// ---- Lead decision (reviewer finding 2): stricter commas, money first ----

/// A grouped number never starts with 0: "0,500" is not 500. A lone "0" or "00" still works,
/// and correct groups still work, in money and in quantity.
#[test]
fn comma_groups_never_start_with_zero() {
    for text in [
        "0,500",
        "00,500",
        "01,000",
        "0,50,000",
        "0,000",
        "00,00,000",
    ] {
        assert_eq!(parse_rupees(text), Err(ErrorCode::ErrNotANumber), "{text}");
        assert_eq!(parse_qty(text), Err(ErrorCode::ErrNotANumber), "{text}");
        assert_eq!(
            parse_sum_rupees(text),
            Err(ErrorCode::ErrNotANumber),
            "{text}"
        );
        assert_eq!(parse_sum_qty(text), Err(ErrorCode::ErrNotANumber), "{text}");
    }
    // One term of a sum with a bad group makes the sum a non-number.
    assert_eq!(parse_sum_rupees("0,500+1"), Err(ErrorCode::ErrNotANumber));
    assert_eq!(parse_rupees("0"), Ok(Paisa::ZERO));
    assert_eq!(parse_rupees("00"), Ok(Paisa::ZERO));
    assert_eq!(parse_rupees("1,000"), Ok(rs(1_000)));
    assert_eq!(parse_rupees("10,00,000"), Ok(rs(1_000_000)));
    assert_eq!(parse_rupees("1,316,040"), Ok(rs(1_316_040)));
    assert_eq!(parse_rupees("13,16,040"), Ok(rs(1_316_040)));
    assert_eq!(parse_qty("1,000"), Ok(qty(1_000)));
    assert_eq!(parse_sum_rupees("1,000+500"), Ok(rs(1_500)));
}

/// A weight takes no commas: "12,345" typed for 12.345 kg must not become 12,345 kg.
#[test]
fn weight_takes_no_commas() {
    for text in [
        "12,345", "1,234.5", "12,3", "100,000", "1,000", "12.345,6", ",5",
    ] {
        assert_eq!(parse_weight(text), Err(ErrorCode::ErrNotANumber), "{text}");
    }
    // The same weight without a comma is read as before.
    assert_eq!(parse_weight("12.345"), Ok(Grams::new(12_345).unwrap()));
    assert_eq!(parse_weight("12345"), Ok(Grams::new(12_345_000).unwrap()));
    assert_eq!(parse_weight("100000"), Ok(Grams::new(100_000_000).unwrap()));
}

/// A percent takes no commas either. The syntax fault comes before the 100% limit, so
/// "1,000" is not a number (it was "over 100" before).
#[test]
fn percent_takes_no_commas() {
    for text in ["1,000", "6,5", "2,50", "1,00", "10,00.5"] {
        assert_eq!(parse_bps(text), Err(ErrorCode::ErrNotANumber), "{text}");
    }
    assert_eq!(parse_bps("100"), Ok(bps(10_000)));
    assert_eq!(parse_bps("6"), Ok(bps(600)));
    assert_eq!(parse_bps("2.5"), Ok(bps(250)));
    assert_eq!(parse_bps("101"), Err(ErrorCode::ErrPercentOver100));
}

/// Money agent review of the comma rules: the order of codes and the other digit scripts.
/// Syntax is checked first, then the sign, then decimals and range.
#[test]
fn comma_rules_error_order_and_scripts() {
    let nan = Some(ErrorCode::ErrNotANumber);
    // A bad comma is a syntax fault, so it comes before the sign.
    assert_eq!(parse_weight("-12,345").err(), nan);
    assert_eq!(parse_bps("-1,000").err(), nan);
    assert_eq!(parse_rupees("-0,500").err(), nan);
    assert_eq!(parse_qty("-0,500").err(), nan);
    // A good group with a sign is still the negative code.
    assert_eq!(
        parse_rupees("-1,000"),
        Err(ErrorCode::ErrNegativeNotAllowed)
    );
    assert_eq!(parse_qty("-1,000"), Err(ErrorCode::ErrNegativeNotAllowed));
    // A comma plus a point: weight and percent say "not a number"; money and qty keep
    // their own decimal codes when the group is good.
    assert_eq!(parse_weight("12,345.5").err(), nan);
    assert_eq!(parse_bps("1,000.5").err(), nan);
    assert_eq!(
        parse_rupees("1,000.5"),
        Err(ErrorCode::ErrDecimalsNotAllowed)
    );
    assert_eq!(parse_qty("1,000.5"), Err(ErrorCode::ErrQtyNotWhole));
    // A decimal comma meant as 1.5 is not a quantity or a sum.
    assert_eq!(parse_qty("1,5").err(), nan);
    assert_eq!(parse_sum_qty("1,5+1").err(), nan);
    assert_eq!(parse_sum_rupees("1,5+1").err(), nan);
    assert_eq!(parse_sum_rupees("1,000-0,500").err(), nan);
    // Zero-first groups in Urdu digits, Arabic-Indic digits and with the Arabic separator.
    for text in ["۰,۵۰۰", "٠,٥٠٠", "۰٬۵۰۰", "0٬500"] {
        assert_eq!(parse_rupees(text).err(), nan, "{text}");
        assert_eq!(parse_qty(text).err(), nan, "{text}");
    }
    // The Arabic thousands separator is refused in a weight and a percent too.
    assert_eq!(parse_weight("12٬345").err(), nan);
    assert_eq!(parse_bps("1٬000").err(), nan);
    // The Arabic decimal point is still a point, so this weight is 0.5 kg.
    assert_eq!(parse_weight("0٫500"), Ok(Grams::new(500).unwrap()));
    // Groups that fit neither the international nor the lakh pattern.
    for text in ["10,00,00", "1,0,000", "123,45,678", "850,50", "1,000,00"] {
        assert_eq!(parse_rupees(text).err(), nan, "{text}");
        assert_eq!(parse_qty(text).err(), nan, "{text}");
        assert_eq!(parse_sum_rupees(&format!("{text}+1")).err(), nan, "{text}");
        assert_eq!(parse_sum_qty(&format!("{text}+1")).err(), nan, "{text}");
    }
}
