//! Amounts in words (money agent 3.3, PRD FR-19.7, NFR-URD-15, NFR-URD-23).
//! The four table rows in Urdu, English international and English lakh, plus every
//! boundary of plan 02 Section 3.
#![cfg(test)]

use mandi_core::english_words::english_words;
use mandi_core::money::{Grouping, Paisa};
use mandi_core::urdu_words::{WordsError, urdu_words};

fn ur(rupees: i64) -> String {
    urdu_words(Paisa::from_rupees(rupees).unwrap()).unwrap()
}

fn intl(rupees: i64) -> String {
    english_words(Paisa::from_rupees(rupees).unwrap(), Grouping::International).unwrap()
}

fn lakh(rupees: i64) -> String {
    english_words(Paisa::from_rupees(rupees).unwrap(), Grouping::Lakh).unwrap()
}

// ---- The four table rows ----

#[test]
fn words_0() {
    assert_eq!(ur(0), "صفر روپے");
    assert_eq!(intl(0), "Zero rupees only");
    assert_eq!(lakh(0), "Zero rupees only");
}

#[test]
fn words_19_316() {
    assert_eq!(ur(19_316), "انیس ہزار تین سو سولہ روپے");
    assert_eq!(
        intl(19_316),
        "Nineteen thousand three hundred sixteen rupees only"
    );
    assert_eq!(
        lakh(19_316),
        "Nineteen thousand three hundred sixteen rupees only"
    );
}

#[test]
fn words_1_00_000() {
    assert_eq!(ur(100_000), "ایک لاکھ روپے");
    assert_eq!(intl(100_000), "One hundred thousand rupees only");
    assert_eq!(lakh(100_000), "One lakh rupees only");
}

#[test]
fn words_1_316_040() {
    assert_eq!(ur(1_316_040), "تیرہ لاکھ سولہ ہزار چالیس روپے");
    assert_eq!(
        intl(1_316_040),
        "One million three hundred sixteen thousand forty rupees only"
    );
    assert_eq!(
        lakh(1_316_040),
        "Thirteen lakh sixteen thousand forty rupees only"
    );
}

// ---- The printed documents of plan 14 (PRD 4.4, 4.5) ----

#[test]
fn words_12_800_the_pukhta_slip() {
    assert_eq!(ur(12_800), "بارہ ہزار آٹھ سو روپے");
    assert_eq!(intl(12_800), "Twelve thousand eight hundred rupees only");
    assert_eq!(lakh(12_800), "Twelve thousand eight hundred rupees only");
}

#[test]
fn words_of_the_two_buyer_slips() {
    assert_eq!(ur(12_880), "بارہ ہزار آٹھ سو اسی روپے");
    assert_eq!(
        intl(12_880),
        "Twelve thousand eight hundred eighty rupees only"
    );
    assert_eq!(ur(18_684), "اٹھارہ ہزار چھ سو چوراسی روپے");
    assert_eq!(
        intl(18_684),
        "Eighteen thousand six hundred eighty-four rupees only"
    );
    assert_eq!(
        lakh(18_684),
        "Eighteen thousand six hundred eighty-four rupees only"
    );
}

// ---- Boundaries ----

#[test]
fn words_boundary_99() {
    assert_eq!(ur(99), "ننانوے روپے");
    assert_eq!(intl(99), "Ninety-nine rupees only");
    assert_eq!(lakh(99), "Ninety-nine rupees only");
}

#[test]
fn words_boundary_100() {
    assert_eq!(ur(100), "ایک سو روپے");
    assert_eq!(intl(100), "One hundred rupees only");
    assert_eq!(lakh(100), "One hundred rupees only");
}

#[test]
fn words_boundary_999() {
    assert_eq!(ur(999), "نو سو ننانوے روپے");
    assert_eq!(intl(999), "Nine hundred ninety-nine rupees only");
    assert_eq!(lakh(999), "Nine hundred ninety-nine rupees only");
}

#[test]
fn words_boundary_1_000() {
    assert_eq!(ur(1_000), "ایک ہزار روپے");
    assert_eq!(intl(1_000), "One thousand rupees only");
    assert_eq!(lakh(1_000), "One thousand rupees only");
}

#[test]
fn words_boundary_99_999() {
    assert_eq!(ur(99_999), "ننانوے ہزار نو سو ننانوے روپے");
    assert_eq!(
        intl(99_999),
        "Ninety-nine thousand nine hundred ninety-nine rupees only"
    );
    assert_eq!(
        lakh(99_999),
        "Ninety-nine thousand nine hundred ninety-nine rupees only"
    );
}

#[test]
fn words_boundary_9_99_99_999() {
    assert_eq!(
        ur(99_999_999),
        "نو کروڑ ننانوے لاکھ ننانوے ہزار نو سو ننانوے روپے"
    );
    assert_eq!(
        intl(99_999_999),
        "Ninety-nine million nine hundred ninety-nine thousand nine hundred ninety-nine rupees only"
    );
    assert_eq!(
        lakh(99_999_999),
        "Nine crore ninety-nine lakh ninety-nine thousand nine hundred ninety-nine rupees only"
    );
}

#[test]
fn words_boundary_10_00_00_000() {
    assert_eq!(ur(100_000_000), "دس کروڑ روپے");
    assert_eq!(intl(100_000_000), "One hundred million rupees only");
    assert_eq!(lakh(100_000_000), "Ten crore rupees only");
}

#[test]
fn words_maximum_10_00_00_00_000() {
    assert_eq!(ur(10_000_000_000), "دس ارب روپے");
    assert_eq!(intl(10_000_000_000), "Ten billion rupees only");
    assert_eq!(lakh(10_000_000_000), "One thousand crore rupees only");
}

#[test]
fn words_above_maximum_is_an_error() {
    let over = Paisa::from_rupees(10_000_000_001).unwrap();
    assert_eq!(urdu_words(over), Err(WordsError::OutOfRange));
    assert_eq!(
        english_words(over, Grouping::International),
        Err(WordsError::OutOfRange)
    );
    assert_eq!(
        english_words(over, Grouping::Lakh),
        Err(WordsError::OutOfRange)
    );
}

#[test]
fn words_negative_is_an_error() {
    let negative = Paisa::from_rupees(-1).unwrap();
    assert_eq!(urdu_words(negative), Err(WordsError::OutOfRange));
    assert_eq!(
        english_words(negative, Grouping::Lakh),
        Err(WordsError::OutOfRange)
    );
}
