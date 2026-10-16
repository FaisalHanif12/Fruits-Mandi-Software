//! Amounts in words at every boundary and out of range (PRD FR-19.7, NFR-URD-15, NFR-URD-23,
//! money agent 3.3). Rules: text for 0 to 10,00,00,00,000; an error outside, never wrong
//! words; well formed (single spaces, the right ending, "zero" only for 0); and no two
//! amounts ever get the same words.

use std::collections::HashSet;

use mandi_core::english_words::english_words;
use mandi_core::money::{Grouping, MAX_RUPEES, Paisa};
use mandi_core::urdu_words::{WordsError, urdu_words};

use super::{max_paisa, min_paisa, no_panic, rs};

/// Every power of ten and its neighbours up to the maximum, plus the place boundaries
/// of both groupings (99, 100, 999, 1,000, 99,999, 1,00,000, 9,99,99,999, 10,00,00,000 ...).
fn boundaries() -> Vec<i64> {
    let mut out = vec![
        0, 1, 2, 11, 19, 20, 21, 99, 101, 110, 111, 119, 120, 999, 1_001,
    ];
    let mut p: i64 = 10;
    while p <= MAX_RUPEES {
        out.extend([p - 1, p, p + 1]);
        out.extend([2 * p - 1, 2 * p]);
        p *= 10;
    }
    // 99,99,999, 9,99,99,999, 99,99,99,999 and 1,00,00,00,001 in lakh grouping.
    out.extend([
        9_999_999,
        99_999_999,
        999_999_999,
        1_000_000_001,
        MAX_RUPEES - 1,
    ]);
    out.retain(|n| (0..=MAX_RUPEES).contains(n));
    out.sort_unstable();
    out.dedup();
    out
}

fn assert_well_formed(n: i64, text: &str, ending: &str, zero: &str) {
    assert!(!text.is_empty(), "{n}: empty");
    assert_eq!(text.trim(), text, "{n}: spaces at the ends: {text:?}");
    assert!(!text.contains("  "), "{n}: double space: {text:?}");
    assert!(text.ends_with(ending), "{n}: ending: {text:?}");
    assert_eq!(
        text.to_lowercase().contains(zero),
        n == 0,
        "{n}: zero word: {text:?}"
    );
}

#[test]
fn words_at_every_boundary() {
    for n in boundaries() {
        let amount = rs(n);
        let ur = no_panic("urdu_words", || urdu_words(amount)).unwrap();
        let ending = if n == 1 { "روپیہ" } else { "روپے" };
        assert_well_formed(n, &ur, ending, "صفر");
        for g in [Grouping::International, Grouping::Lakh] {
            let en = no_panic("english_words", || english_words(amount, g)).unwrap();
            let ending = if n == 1 { "rupee only" } else { "rupees only" };
            assert_well_formed(n, &en, ending, "zero");
            assert!(
                en.starts_with(|c: char| c.is_ascii_uppercase()),
                "{n}: {en}"
            );
            assert!(en.is_ascii(), "{n}: {en}");
        }
    }
}

#[test]
fn the_maximum_in_words() {
    assert_eq!(urdu_words(rs(MAX_RUPEES)).unwrap(), "دس ارب روپے");
    assert_eq!(
        english_words(rs(MAX_RUPEES), Grouping::International).unwrap(),
        "Ten billion rupees only"
    );
    assert_eq!(
        english_words(rs(MAX_RUPEES), Grouping::Lakh).unwrap(),
        "One thousand crore rupees only"
    );
}

#[test]
fn out_of_range_is_an_error_never_words() {
    for amount in [
        rs(MAX_RUPEES + 1),
        rs(-1),
        max_paisa(),
        min_paisa(),
        rs(i64::MAX / 100),
    ] {
        assert_eq!(
            urdu_words(amount),
            Err(WordsError::OutOfRange),
            "{amount:?}"
        );
        for g in [Grouping::International, Grouping::Lakh] {
            assert_eq!(
                english_words(amount, g),
                Err(WordsError::OutOfRange),
                "{amount:?}"
            );
        }
    }
    // A value that is not a whole rupee cannot reach the words functions at all.
    assert!(Paisa::try_from_paisa(150).is_err());
}

#[test]
fn no_two_amounts_share_words() {
    // Every amount from 0 to 2,00,000, and every boundary, has its own words in each form.
    let mut amounts: Vec<i64> = (0..=200_000).collect();
    amounts.extend(boundaries());
    amounts.sort_unstable();
    amounts.dedup();
    let mut ur = HashSet::new();
    let mut intl = HashSet::new();
    let mut lakh = HashSet::new();
    for n in amounts {
        let a = rs(n);
        assert!(
            ur.insert(urdu_words(a).unwrap()),
            "urdu words repeat at {n}"
        );
        assert!(
            intl.insert(english_words(a, Grouping::International).unwrap()),
            "intl at {n}"
        );
        assert!(
            lakh.insert(english_words(a, Grouping::Lakh).unwrap()),
            "lakh at {n}"
        );
    }
}
