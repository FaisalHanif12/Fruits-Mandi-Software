//! The engine takes any text and never panics (Code Standards B28, 9.3; NFR-SPD-20).
#![cfg(test)]

use std::time::{Duration, Instant};

use mandi_core::translit::{
    LearnedPair, MAX_SUGGESTIONS, WordList, suggest_to_english, suggest_to_urdu,
};
use proptest::prelude::*;

fn list() -> &'static WordList {
    WordList::embedded()
}

/// Runs both directions and checks the promises that hold for every input.
fn check(text: &str, learned: &[LearnedPair]) {
    for (name, got) in [
        ("to_urdu", suggest_to_urdu(text, learned, list())),
        ("to_english", suggest_to_english(text, learned, list())),
    ] {
        assert!(
            got.len() <= MAX_SUGGESTIONS,
            "{name}: {} results",
            got.len()
        );
        assert!(got.iter().all(|s| !s.is_empty()), "{name}: an empty result");
        let mut unique = got.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(unique.len(), got.len(), "{name}: the same result twice");
    }
    assert_eq!(
        suggest_to_urdu(text, learned, list()),
        suggest_to_urdu(text, learned, list()),
        "to_urdu is not repeatable"
    );
    assert_eq!(
        suggest_to_english(text, learned, list()),
        suggest_to_english(text, learned, list()),
        "to_english is not repeatable"
    );
}

#[test]
fn awkward_texts_do_not_panic() {
    let long_letters = "a".repeat(10_000);
    let long_urdu = "ا".repeat(10_000);
    let long_words = "ali ".repeat(5_000);
    let cases: Vec<&str> = vec![
        "",
        " ",
        "\t\n\r",
        "a",
        "ء",
        "\0",
        "a\0b",
        "😀",
        "😀😀😀 arshad 😀",
        "arshad😀multani",
        "arshad ارشد multani",
        "ارشد arshad ملتانی",
        "\u{202E}arshad\u{202C}",
        "ar\u{200B}shad",
        "ar\u{200D}shad",
        "a\u{0301}rshad",
        "\u{0301}",
        "ا\u{064E}\u{0650}\u{064F}\u{0651}",
        "\u{0670}",
        "ǅǆǈ",
        "ÀÉÎÕÜ",
        "ß",
        "ﷲ",
        "١٢٣ arshad ٤٥٦",
        "arshad 123 multani 456",
        "!!!???...",
        "((((((((((",
        "'''''",
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "bcdfghjklmnpqrstvwxyzbcdfghjklmnpqrstvwxyz",
        "اااااااااااااااااااااااااااااااااااااااااااااااااااا",
        "ڑھڑھڑھڑھڑھڑھڑھڑھڑھڑھڑھڑھڑھڑھڑھڑھڑھڑھڑھڑھڑھڑھڑھڑھ",
        "sh sh sh sh sh sh sh sh sh sh sh sh sh sh sh sh sh sh sh sh",
        "kh gh ch th ph zh aa ee oo ai au ck",
        &long_letters,
        &long_urdu,
        &long_words,
    ];
    for text in cases {
        check(text, &[]);
    }
}

#[test]
fn a_huge_text_is_cut_short_and_fast() {
    let five_mb: String = "x ".repeat(2_500_000);
    let start = Instant::now();
    check(&five_mb, &[]);
    assert!(
        start.elapsed() < Duration::from_secs(5),
        "5 MB took {:?}",
        start.elapsed()
    );
}

#[test]
fn awkward_learned_pairs_do_not_panic() {
    let pairs = vec![
        LearnedPair {
            roman: String::new(),
            urdu: String::new(),
            use_count: 0,
        },
        LearnedPair {
            roman: "😀".to_owned(),
            urdu: "😀".to_owned(),
            use_count: u32::MAX,
        },
        LearnedPair {
            roman: "ali".to_owned(),
            urdu: "a".repeat(5_000),
            use_count: u32::MAX,
        },
        LearnedPair {
            roman: "x".repeat(5_000),
            urdu: "ایکس".to_owned(),
            use_count: 1,
        },
    ];
    for text in ["ali", "", "x", "ایکس", "😀", &"x".repeat(5_000)] {
        check(text, &pairs);
    }
    // A pile of learned pairs (the Owner may teach it for years).
    let many: Vec<LearnedPair> = (0..20_000)
        .map(|i| LearnedPair {
            roman: format!("word{i}"),
            urdu: "کلمہ".to_owned(),
            use_count: i,
        })
        .collect();
    let start = Instant::now();
    check("ali word19999 multani", &many);
    assert!(start.elapsed() < Duration::from_secs(5));
}

/// NFR-SPD-20: the suggestions for a key press come in under 50 ms. The bound here is loose
/// (a test machine is busy and slow); `robustness` benchmarks the real figure.
#[test]
fn one_key_press_is_fast() {
    let _warm = suggest_to_urdu("ali", &[], list());
    let _warm = suggest_to_english("علی", &[], list());
    let typed = [
        "a",
        "ar",
        "ars",
        "arsh",
        "arsha",
        "arshad",
        "arshad m",
        "arshad mu",
        "arshad mul",
    ];
    let mut slowest = Duration::ZERO;
    for text in typed {
        let start = Instant::now();
        let _ = suggest_to_urdu(text, &[], list());
        slowest = slowest.max(start.elapsed());
    }
    // A word the list does not have: the rules do the work.
    for text in ["zxqvjk", "ghazanfarullah", "mehrabpurwala", "bakhtawarpur"] {
        let start = Instant::now();
        let _ = suggest_to_urdu(text, &[], list());
        slowest = slowest.max(start.elapsed());
    }
    assert!(slowest < Duration::from_millis(500), "slowest {slowest:?}");
    println!("slowest key press in this build: {slowest:?}");
}

/// 300 cases a run; `PROPTEST_CASES=20000` for a long soak.
fn cases() -> u32 {
    std::env::var("PROPTEST_CASES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(300)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(cases()))]

    #[test]
    fn any_text_is_safe(text in "\\PC{0,600}") {
        check(&text, &[]);
    }

    #[test]
    fn english_letters_are_safe(text in "[a-zA-Z ]{0,80}") {
        check(&text, &[]);
    }

    #[test]
    fn urdu_letters_are_safe(text in "[\u{0600}-\u{06FF} ]{0,80}") {
        check(&text, &[]);
    }

    #[test]
    fn mixed_scripts_are_safe(text in "[a-z\u{0600}-\u{06FF}0-9 .\\-😀]{0,80}") {
        check(&text, &[]);
    }

    #[test]
    fn learned_pairs_of_any_kind_are_safe(
        roman in "[a-z]{0,10}",
        urdu in "[\u{0600}-\u{06FF}]{0,10}",
        count in any::<u32>(),
        text in "[a-z ]{0,30}",
    ) {
        let pairs = [LearnedPair { roman, urdu, use_count: count }];
        check(&text, &pairs);
    }
}
