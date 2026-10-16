//! Property tests, 10,000 cases each (Code Standards 9.3, money agent 11.2 items 1, 2, 5,
//! 6 and 9, plan 02 T2). The case count is set here, not by an environment variable.
#![cfg(test)]

use mandi_core::english_words::english_words;
use mandi_core::money::{
    Bps, Grams, Grouping, MAX_RUPEES, Paisa, Qty, apply_bps, avg_rate, format_paisa, line_value,
    parse_rupees, parse_sum_rupees, split_by_bps, split_by_qty, weight_value,
};
use mandi_core::text::{normalize_urdu, search_en, search_phonetic, search_ur};
use mandi_core::urdu_words::urdu_words;
use proptest::prelude::*;

const CASES: u32 = 10_000;

fn config() -> ProptestConfig {
    ProptestConfig {
        cases: CASES,
        ..ProptestConfig::default()
    }
}

/// 1 to 10 partner shares that total exactly 10,000 bps, with distinct ids in a random
/// order (so the input order is not the id order). Zero shares and ties happen often.
fn shares() -> impl Strategy<Value = Vec<(u32, Bps)>> {
    prop::collection::vec(0i32..=10_000, 0..=9)
        .prop_flat_map(|mut cuts| {
            cuts.sort_unstable();
            let mut bounds = vec![0];
            bounds.extend(cuts);
            bounds.push(10_000);
            let weights: Vec<i32> = bounds.windows(2).map(|w| w[1] - w[0]).collect();
            let ids: Vec<u32> = (1..=u32::try_from(weights.len()).unwrap())
                .map(|i| i * 7)
                .collect();
            (Just(weights), Just(ids).prop_shuffle())
        })
        .prop_map(|(weights, ids)| {
            ids.into_iter()
                .zip(weights)
                .map(|(id, w)| (id, Bps::new(w).unwrap()))
                .collect()
        })
}

/// The money agent's literal algorithm (Section 5.1): floor every part, then the leftover
/// to the largest bps, tie to the lowest id. Used to prove the shortcut in the code.
fn reference_split(amount_rupees: i64, shares: &[(u32, Bps)]) -> Vec<i64> {
    let abs = i128::from(amount_rupees).abs();
    let mut parts: Vec<i128> = shares
        .iter()
        .map(|(_, b)| abs * i128::from(b.get()) / 10_000)
        .collect();
    let leftover = abs - parts.iter().sum::<i128>();
    let winner = shares
        .iter()
        .enumerate()
        .max_by(|(_, (ia, ba)), (_, (ib, bb))| ba.cmp(bb).then(ib.cmp(ia)))
        .map(|(i, _)| i)
        .unwrap();
    parts[winner] += leftover;
    parts
        .into_iter()
        .map(|p| {
            let p = i64::try_from(p).unwrap();
            if amount_rupees < 0 { -p } else { p }
        })
        .collect()
}

fn rupees() -> impl Strategy<Value = i64> {
    -MAX_RUPEES..=MAX_RUPEES
}

proptest! {
    #![proptest_config(config())]

    /// Item 1: Σ parts == amount; parts ≥ 0 for positive amounts, ≤ 0 for negative;
    /// output order = input order; same result as the literal algorithm.
    #[test]
    fn split_by_bps_adds_back_exactly(amount in rupees(), shares in shares()) {
        let paisa = Paisa::from_rupees(amount).unwrap();
        let parts = split_by_bps(paisa, &shares).unwrap();
        let sum: i64 = parts.iter().map(|(_, p)| p.rupees()).sum();
        prop_assert_eq!(sum, amount);
        for ((id, part), (input_id, _)) in parts.iter().zip(&shares) {
            prop_assert_eq!(id, input_id);
            if amount >= 0 { prop_assert!(part.rupees() >= 0) } else { prop_assert!(part.rupees() <= 0) }
        }
        let got: Vec<i64> = parts.iter().map(|(_, p)| p.rupees()).collect();
        prop_assert_eq!(got, reference_split(amount, &shares));
    }

    /// Item 2: Σ parts == total for any quantities.
    #[test]
    fn split_by_qty_adds_back_exactly(
        amount in rupees(),
        qtys in prop::collection::vec(0i64..=1_000_000, 1..=50),
    ) {
        prop_assume!(qtys.iter().sum::<i64>() > 0);
        let parts: Vec<(usize, Qty)> = qtys.iter().enumerate().map(|(i, &q)| (i, Qty::new(q).unwrap())).collect();
        let result = split_by_qty(Paisa::from_rupees(amount).unwrap(), &parts).unwrap();
        let sum: i64 = result.iter().map(|(_, p)| p.rupees()).sum();
        prop_assert_eq!(sum, amount);
    }

    /// Item 5: parse(format(x)) == x for every whole-rupee x in range, both groupings.
    #[test]
    fn parse_of_format_round_trips(amount in 0..=MAX_RUPEES) {
        let paisa = Paisa::from_rupees(amount).unwrap();
        for grouping in [Grouping::International, Grouping::Lakh] {
            let text = format_paisa(paisa, grouping).unwrap();
            prop_assert_eq!(parse_rupees(&text), Ok(paisa));
            prop_assert_eq!(parse_sum_rupees(&text), Ok(paisa));
        }
    }

    /// Item 6: a percent charge is never more than its base for bps ≤ 10,000, never
    /// negative, and never goes down when the base grows.
    #[test]
    fn percent_charge_bounded_and_monotonic(
        base in 0..=MAX_RUPEES * 1_000_000,
        extra in 0..=MAX_RUPEES,
        bps in 0i32..=10_000,
    ) {
        let bps = Bps::new(bps).unwrap();
        let small = Paisa::from_rupees(base).unwrap();
        let big = Paisa::from_rupees(base + extra).unwrap();
        let charge_small = apply_bps(small, bps).unwrap();
        let charge_big = apply_bps(big, bps).unwrap();
        prop_assert!(charge_small >= Paisa::ZERO);
        prop_assert!(charge_small <= small);
        prop_assert!(charge_small <= charge_big);
    }

    /// Item 9: every money value made by core is a whole rupee (a multiple of 100 paisa).
    #[test]
    fn every_result_is_whole_rupees(
        rate in 0..=MAX_RUPEES,
        qty in 0i64..=1_000_000,
        grams in 0i64..=100_000_000,
        bps in 0i32..=10_000,
        text in "\\PC{0,20}",
    ) {
        let rate = Paisa::from_rupees(rate).unwrap();
        let q = Qty::new(qty).unwrap();
        let values = [
            line_value(q, rate).ok(),
            weight_value(Grams::new(grams).unwrap(), rate).ok(),
            apply_bps(rate, Bps::new(bps).unwrap()).ok(),
            avg_rate(rate, q).ok(),
            parse_rupees(&text).ok(),
            parse_sum_rupees(&text).ok(),
        ];
        for v in values.into_iter().flatten() {
            prop_assert_eq!(v.as_paisa() % 100, 0);
        }
    }

    /// Words never panic and give text for every amount in range, all three styles.
    #[test]
    fn words_cover_the_whole_range(amount in 0..=MAX_RUPEES) {
        let paisa = Paisa::from_rupees(amount).unwrap();
        prop_assert!(!urdu_words(paisa).unwrap().is_empty());
        prop_assert!(!english_words(paisa, Grouping::International).unwrap().is_empty());
        prop_assert!(!english_words(paisa, Grouping::Lakh).unwrap().is_empty());
    }

    /// Words are an error, never wrong text, outside the range.
    #[test]
    fn words_refuse_out_of_range(
        amount in prop_oneof![MAX_RUPEES + 1..=i64::MAX / 100, i64::MIN / 100..=-1],
    ) {
        let paisa = Paisa::from_rupees(amount).unwrap();
        prop_assert!(urdu_words(paisa).is_err());
        prop_assert!(english_words(paisa, Grouping::International).is_err());
        prop_assert!(english_words(paisa, Grouping::Lakh).is_err());
    }

    /// normalize_urdu and the search keys never panic on any Unicode text, and are stable.
    #[test]
    fn search_keys_never_panic(text in "\\PC*", any in any::<String>()) {
        for s in [&text, &any] {
            let n = normalize_urdu(s);
            prop_assert_eq!(normalize_urdu(&n), n.clone());
            prop_assert_eq!(search_ur(s), n);
            let e = search_en(s);
            prop_assert_eq!(search_en(&e), e);
            let p = search_phonetic(s);
            prop_assert!(p.bytes().all(|b| b.is_ascii_lowercase()));
        }
    }

    /// Every parser gives a value or a code for any text; it never panics.
    #[test]
    fn parsers_never_panic(text in "\\PC*", any in any::<String>()) {
        for s in [&text, &any] {
            let _ = parse_rupees(s);
            let _ = parse_sum_rupees(s);
            let _ = mandi_core::money::parse_bps(s);
            let _ = mandi_core::money::parse_weight(s);
            let _ = mandi_core::money::parse_qty(s);
            let _ = mandi_core::money::parse_sum_qty(s);
        }
    }
}

/// Asserts that the three search keys of `s` are stable: a key built from a key is the
/// same key (robustness finding R02-4).
fn assert_keys_stable(s: &str) {
    let ur = search_ur(s);
    assert_eq!(search_ur(&ur), ur, "search_ur of {s:?}");
    let en = search_en(s);
    assert_eq!(search_en(&en), en, "search_en of {s:?}");
    let ph = search_phonetic(s);
    assert_eq!(search_phonetic(&ph), ph, "search_phonetic of {s:?}");
}

/// Random text finds most problems; this checks every Unicode character on its own, then
/// every base letter with two marks after it (Latin accents, Arabic marks, zero-width
/// characters, direction marks, bidi controls). Removing a mark can put two characters side
/// by side that compose, and lower-casing can make marks that need reordering; both only
/// show with a base and marks together.
#[test]
fn keys_are_stable_for_every_char_and_mark_pair() {
    for c in (0..=u32::from(char::MAX)).filter_map(char::from_u32) {
        assert_keys_stable(&c.to_string());
    }
    let bases = ['e', 'i', 'A', '\u{0130}', 'ا', 'و', 'ي', 'ه', 'ہ', 'ك', 'ب'];
    let marks: Vec<char> = ('\u{0300}'..='\u{0315}')
        .chain('\u{0610}'..='\u{061F}')
        .chain('\u{064B}'..='\u{0657}')
        .chain(['\u{065F}', '\u{0670}', '\u{06D6}', '\u{06E7}', '\u{06ED}'])
        .chain('\u{200B}'..='\u{200F}')
        .chain(['\u{2060}', '\u{FEFF}', '\u{0130}', '\u{1E9E}', ' '])
        .chain('\u{202A}'..='\u{202E}')
        .chain('\u{2066}'..='\u{2069}')
        .collect();
    for base in bases {
        for &m1 in &marks {
            for &m2 in &marks {
                assert_keys_stable(&format!("{base}{m1}{m2}"));
            }
        }
    }
}

#[test]
fn every_property_runs_10_000_cases() {
    assert_eq!(config().cases, 10_000);
}
