//! Search keys, Urdu letter variants, Urdu sort order and script checks
//! (Tech Stack 4.3, NFR-URD-18, NFR-URD-19, NFR-URD-20, PRD FR-7.28, Code Standards B26).
//!
//! Every named record keeps three search keys, built only here:
//! - `search_ur`: the Urdu name with letter variants unified and diacritics removed,
//!   so "اكرم" (Arabic kaf) and "اکرم" (Urdu kaf) give the same key;
//! - `search_en`: the English name in lower case with single spaces;
//! - `search_phonetic`: the English name with vowels and doubled letters removed,
//!   so "arshad" and "arshd" give the same key.

use std::cmp::Ordering;

use icu_collator::options::CollatorOptions;
use icu_collator::{Collator, CollatorBorrowed};
use icu_locale_core::locale;
use thiserror::Error;
use unicode_normalization::UnicodeNormalization;

/// Left-to-right mark (LRM).
const LRM: char = '\u{200E}';

/// True for the zero-width characters that have no direction: ZWSP, ZWNJ, ZWJ, word joiner
/// (U+2060) and BOM. They never reorder text. This is the one list shared by names, search
/// keys and number text (Code Standards G6; robustness finding R02-1).
pub fn is_zero_width(c: char) -> bool {
    matches!(c, '\u{200B}'..='\u{200D}' | '\u{2060}' | '\u{FEFF}')
}

/// True for the invisible direction marks LRM, RLM and the Arabic letter mark (ALM).
pub fn is_direction_mark(c: char) -> bool {
    matches!(c, LRM | '\u{200F}' | '\u{061C}')
}

/// True for the bidi embeddings, overrides and isolates (U+202A to U+202E and U+2066 to
/// U+2069). They change the order in which text is shown, so a pasted "<RLO>12" shows "21"
/// (Trojan Source, CVE-2021-42574). Names and free text refuse them; search keys drop them.
pub fn is_bidi_control(c: char) -> bool {
    matches!(c, '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}')
}

/// True for the characters removed from names: zero-width characters and direction marks.
pub fn is_invisible(c: char) -> bool {
    is_zero_width(c) || is_direction_mark(c)
}

/// True for the characters that never count in a search key: everything in `is_invisible`
/// and the bidi controls, so a pasted query matches the stored key (robustness finding R02-2).
fn is_ignored_in_key(c: char) -> bool {
    is_invisible(c) || is_bidi_control(c)
}

/// True for the invisible marks removed from number, code and phone text before it is read
/// (plan 02 Section 4.6): the zero-width characters and LRM.
///
/// LRM is a strong left-to-right character. It cannot reorder digits inside a left-to-right
/// number field (`NumberTextInput` sets `dir="ltr"`), but it could in right-to-left text,
/// where "500-<LRM>200" shows "200-500". So this rule holds only while number fields stay
/// `dir="ltr"`.
///
/// Marks that can make the screen show another number than the one read are NOT in this
/// list and stay `ERR_NOT_A_NUMBER`: RLM (U+200F) and ALM (U+061C), which are strong
/// right-to-left characters, and the bidi controls.
pub fn is_ignorable_in_number(c: char) -> bool {
    is_zero_width(c) || c == LRM
}

/// True for Arabic-script diacritics and small letters that change no letter of a name:
/// tanween, zabar, zer, pesh, shadd, sukun, maddah and hamza marks, khari zabar
/// (superscript alef) and the small high letters.
pub fn is_urdu_mark(c: char) -> bool {
    matches!(c,
        '\u{0610}'..='\u{061A}'
        | '\u{064B}'..='\u{065F}'
        | '\u{0670}'
        | '\u{06D6}'..='\u{06DC}'
        | '\u{06DF}'..='\u{06E4}'
        | '\u{06E7}'..='\u{06E8}'
        | '\u{06EA}'..='\u{06ED}')
}

/// Unifies Arabic and Urdu letter variants (Tech Stack 4.3).
fn unify_letter(c: char) -> char {
    match c {
        // Arabic yeh and alef maksura -> Urdu farsi yeh.
        '\u{064A}' | '\u{0649}' => '\u{06CC}',
        // Arabic kaf -> Urdu keheh.
        '\u{0643}' => '\u{06A9}',
        // Arabic heh and teh marbuta goal -> heh goal. Do-chashmi heh (U+06BE) stays.
        '\u{0647}' | '\u{06C3}' => '\u{06C1}',
        other => other,
    }
}

/// Splits on any whitespace and joins with single spaces (trims as well).
fn single_spaces(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The normalised Urdu form used for search (NFR-URD-18): NFC, letter variants unified,
/// diacritics, zero-width characters and bidi controls removed, spaces collapsed and trimmed.
///
/// `normalize_urdu("اكرم") == normalize_urdu("اکرم")`.
///
/// The key is stable: `normalize_urdu(normalize_urdu(x)) == normalize_urdu(x)` (robustness
/// finding R02-4). The text is joined into letters first (so a letter and its hamza are one
/// letter), marks are removed, and then it is joined again, because removing a mark can put
/// two characters side by side that compose.
pub fn normalize_urdu(s: &str) -> String {
    let unified: String = s
        .nfc()
        .filter(|&c| !is_ignored_in_key(c) && !is_urdu_mark(c))
        .map(unify_letter)
        .nfc()
        .collect();
    single_spaces(&unified)
}

/// Search key for an Urdu name (`search_ur`).
pub fn search_ur(name_ur: &str) -> String {
    normalize_urdu(name_ur)
}

/// Search key for an English name (`search_en`): zero-width characters and bidi controls
/// removed, lower case, NFC, single spaces. "  Akram  Toll Tax " → "akram toll tax".
///
/// The key is stable: `search_en(search_en(x)) == search_en(x)` (robustness finding R02-4).
/// Lower-casing comes before NFC because it can make marks (`İ` gives `i` + U+0307) that
/// NFC must put in order.
pub fn search_en(name_en: &str) -> String {
    let lower: String = name_en
        .chars()
        .filter(|&c| !is_ignored_in_key(c))
        .flat_map(char::to_lowercase)
        .nfc()
        .collect();
    single_spaces(&lower)
}

/// Phonetic key for an English name (`search_phonetic`): lower-case Latin letters only;
/// doubled letters made single; vowels a, e, i, o, u removed except a first letter.
///
/// "Arshad" → "arshd"; "arshd" → "arshd"; "Mehmood" → "mhmd".
pub fn search_phonetic(name_en: &str) -> String {
    let letters: String = name_en
        .chars()
        .filter(char::is_ascii_alphabetic)
        .map(|c| c.to_ascii_lowercase())
        .collect();
    let mut key = String::with_capacity(letters.len());
    for (i, c) in letters.chars().enumerate() {
        let vowel = matches!(c, 'a' | 'e' | 'i' | 'o' | 'u');
        if (i == 0 || !vowel) && !key.ends_with(c) {
            key.push(c);
        }
    }
    key
}

/// The key of a one-time person of the پختہ کھاتہ accounts (PRD FR-13.5): the Urdu name after
/// Urdu normalisation, a `|`, and the phone digits (empty if there is none). Balances, matching
/// and aging are kept per key, so one person's payment never settles another's bill.
///
/// A name never holds `|` (the name check allows only letters, digits, spaces and `. - ' ( ) / &`),
/// so the two parts cannot run into each other. The same name with another phone is another
/// person; the same name and phone with other spelling variants (ي and ی) is the same person.
///
/// Example: `person_key("شاہد", "03000000000")` is `"شاہد|03000000000"`;
/// `person_key(" شاه ", "03000000000")` is the same key as `person_key("شاہ", "03000000000")`.
pub fn person_key(name_ur: &str, phone: &str) -> String {
    let digits: String = phone.chars().filter(char::is_ascii_digit).collect();
    format!("{}|{digits}", normalize_urdu(name_ur))
}

/// True for a letter of the Urdu (Arabic) script.
pub fn is_urdu_letter(c: char) -> bool {
    matches!(c, '\u{0600}'..='\u{06FF}' | '\u{0750}'..='\u{077F}' | '\u{FB50}'..='\u{FDFF}' | '\u{FE70}'..='\u{FEFF}')
        && c.is_alphabetic()
        && !is_urdu_mark(c)
}

/// True for a Latin letter of any kind (`A`, `é`, `ß`).
pub fn is_latin_letter(c: char) -> bool {
    c.is_alphabetic()
        && matches!(c, 'A'..='Z' | 'a'..='z' | '\u{00C0}'..='\u{024F}' | '\u{1E00}'..='\u{1EFF}')
}

/// Punctuation allowed in names in both scripts (Code Standards 4.2).
fn is_name_punctuation(c: char) -> bool {
    matches!(c, ' ' | '.' | '-' | '\'' | '(' | ')' | '/' | '&')
}

/// True for an Arabic-Indic digit ٠-٩ (U+0660 to U+0669). Unicode gives it the bidi class AN
/// (Arabic Number), so a `+` or `-` next to it takes the right-to-left direction and the
/// digit runs show in another order than they are read. Urdu digits ۰-۹ (U+06F0 to U+06F9)
/// are class EN, like 0-9, and keep their order.
pub fn is_arabic_indic_digit(c: char) -> bool {
    matches!(c, '\u{0660}'..='\u{0669}')
}

/// Digits allowed in names: 0-9, Urdu ۰-۹ and Arabic-Indic ٠-٩.
fn is_name_digit(c: char) -> bool {
    c.is_ascii_digit() || matches!(c, '\u{06F0}'..='\u{06F9}') || is_arabic_indic_digit(c)
}

/// True if a cleaned Urdu name uses only Urdu letters, Urdu diacritics, digits, spaces and
/// `. - ' ( ) / &`, and has at least one Urdu letter (PRD FR-7.28).
pub fn is_urdu_name(name: &str) -> bool {
    name.chars().any(is_urdu_letter)
        && name.chars().all(|c| {
            is_urdu_letter(c) || is_urdu_mark(c) || is_name_digit(c) || is_name_punctuation(c)
        })
}

/// True if a cleaned English name uses only the letters A-Z and a-z, digits, spaces and
/// `. - ' ( ) / &`, and has at least one letter (PRD FR-7.28).
pub fn is_english_name(name: &str) -> bool {
    name.chars().any(|c| c.is_ascii_alphabetic())
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || is_name_punctuation(c))
}

/// The Urdu collator could not be built from its built-in data.
#[derive(Debug, Clone, Error)]
#[error("urdu collator data missing: {0}")]
pub struct CollatorError(String);

/// Compares Urdu text in Urdu alphabetical order, ا ب پ ت ٹ ... (NFR-URD-19), with the
/// ICU4X collator for the `ur` locale. Build it once and keep it.
pub struct UrduCollator(CollatorBorrowed<'static>);

impl UrduCollator {
    /// Builds the collator from the data compiled into the app.
    pub fn new() -> Result<UrduCollator, CollatorError> {
        Collator::try_new(locale!("ur").into(), CollatorOptions::default())
            .map(UrduCollator)
            .map_err(|e| CollatorError(e.to_string()))
    }

    /// Urdu alphabetical order of two strings.
    pub fn compare(&self, a: &str, b: &str) -> Ordering {
        self.0.compare(a, b)
    }

    /// A key that sorts like the text does: comparing two keys byte by byte gives the same
    /// answer as [`UrduCollator::compare`] on the two texts. Made once and kept, it makes a
    /// sort of thousands of names cheap (the search index uses it).
    ///
    /// ```
    /// use mandi_core::text::UrduCollator;
    /// let collator = UrduCollator::new().unwrap();
    /// // ا comes before ب comes before پ.
    /// assert!(collator.sort_key("ا") < collator.sort_key("ب"));
    /// assert!(collator.sort_key("ب") < collator.sort_key("پ"));
    /// ```
    pub fn sort_key(&self, s: &str) -> Vec<u8> {
        let mut key = Vec::new();
        // Writing into a Vec cannot fail.
        let Ok(()) = self.0.write_sort_key_to(s, &mut key);
        key
    }

    /// Sorts a list in Urdu alphabetical order.
    pub fn sort<S: AsRef<str>>(&self, items: &mut [S]) {
        items.sort_by(|a, b| self.compare(a.as_ref(), b.as_ref()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arabic_and_urdu_letters_match() {
        assert_eq!(normalize_urdu("اكرم"), normalize_urdu("اکرم"));
        // ي and ى -> ی
        assert_eq!(normalize_urdu("علي"), "علی");
        assert_eq!(normalize_urdu("علىٰ"), "علی");
        // ك -> ک
        assert_eq!(normalize_urdu("ك"), "ک");
        // ه (Arabic heh) -> ہ
        assert_eq!(normalize_urdu("شاه"), "شاہ");
        // ۃ -> ہ
        assert_eq!(normalize_urdu("ۃ"), "ہ");
        // ھ (do-chashmi heh) stays
        assert_eq!(normalize_urdu("بھائی"), "بھائی");
    }

    #[test]
    fn diacritics_and_invisible_marks_go() {
        // zabar, zer, pesh, shadd, sukun, tanween, khari zabar
        assert_eq!(normalize_urdu("مُحَمَّد"), "محمد");
        assert_eq!(normalize_urdu("اِسْلام"), "اسلام");
        assert_eq!(normalize_urdu("فوراً"), "فورا");
        assert_eq!(normalize_urdu("موسیٰ"), "موسی");
        // ZWJ, ZWNJ, ZWSP, BOM
        assert_eq!(
            normalize_urdu("\u{FEFF}ار\u{200C}شد\u{200D}\u{200B}"),
            "ارشد"
        );
        // spaces collapsed and trimmed
        assert_eq!(normalize_urdu("  ارشد \t  ملتانی  "), "ارشد ملتانی");
        // NFC: alef + hamza above becomes one letter and the hamza is kept
        assert_eq!(normalize_urdu("\u{0627}\u{0654}"), "\u{0623}");
    }

    #[test]
    fn a_person_key_joins_the_normalised_urdu_name_and_the_phone_digits() {
        assert_eq!(person_key("شاہد", "03000000000"), "شاہد|03000000000");
        // Letter variants and spaces do not make another person.
        assert_eq!(
            person_key(" شاه ", "03000000000"),
            person_key("شاہ", "03000000000")
        );
        assert_eq!(
            person_key("اكرم  خان", "0300-0000000"),
            "اکرم خان|03000000000"
        );
        // No phone: the key still names the person.
        assert_eq!(person_key("ناصر", ""), "ناصر|");
        // The same name with another phone is another person.
        assert_ne!(person_key("ناصر", ""), person_key("ناصر", "03001111111"));
    }

    #[test]
    fn english_keys() {
        assert_eq!(search_en("  Akram  Toll\u{200B} Tax "), "akram toll tax");
        assert_eq!(search_ur(" اكرم ٹول ٹیکس "), "اکرم ٹول ٹیکس");
        assert_eq!(search_phonetic("Arshad"), "arshd");
        assert_eq!(search_phonetic("arshd"), "arshd");
        assert_eq!(search_phonetic("Akram"), search_phonetic("akrm"));
        assert_eq!(search_phonetic("Mehmood"), "mhmd");
        assert_eq!(search_phonetic("Mehmud"), "mhmd");
        assert_eq!(search_phonetic("Arshad Multani"), "arshdmltn");
        assert_eq!(search_phonetic("Tata"), "t");
        assert_eq!(search_phonetic("Ali 2"), "al");
        assert_eq!(search_phonetic("Oonchi"), "onch");
        assert_eq!(search_phonetic(""), "");
    }

    #[test]
    fn shared_mark_lists() {
        // One list for names, keys and numbers (Code Standards G6).
        for c in ['\u{200B}', '\u{200C}', '\u{200D}', '\u{2060}', '\u{FEFF}'] {
            assert!(is_zero_width(c) && is_invisible(c) && is_ignorable_in_number(c));
        }
        assert!(is_direction_mark('\u{200E}') && is_ignorable_in_number('\u{200E}'));
        // RLM and ALM are removed from names and keys but refused in numbers.
        for c in ['\u{200F}', '\u{061C}'] {
            assert!(is_direction_mark(c) && is_invisible(c) && !is_ignorable_in_number(c));
        }
        // Bidi controls are neither invisible (names refuse them) nor ignorable in numbers.
        for c in ('\u{202A}'..='\u{202E}').chain('\u{2066}'..='\u{2069}') {
            assert!(is_bidi_control(c) && !is_invisible(c) && !is_ignorable_in_number(c));
            assert!(is_ignored_in_key(c));
        }
        for c in ['a', 'ا', ' ', '\u{2065}', '\u{202F}', '\u{206A}'] {
            assert!(!is_zero_width(c) && !is_direction_mark(c) && !is_bidi_control(c));
        }
    }

    /// Finding R02-2: a pasted query with bidi controls matches the stored key.
    #[test]
    fn bidi_controls_never_change_a_key() {
        for c in ('\u{202A}'..='\u{202E}').chain('\u{2066}'..='\u{2069}') {
            assert_eq!(
                search_ur(&format!("{c}اکرم ٹول{c} ٹیکس{c}")),
                "اکرم ٹول ٹیکس"
            );
            assert_eq!(
                search_en(&format!("{c}Akram Toll{c} Tax{c}")),
                "akram toll tax"
            );
        }
        // LRM, RLM and ALM too, and the word joiner.
        for c in ['\u{200E}', '\u{200F}', '\u{061C}', '\u{2060}'] {
            assert_eq!(search_ur(&format!("{c}اکرم{c}")), "اکرم");
            assert_eq!(search_en(&format!("{c}Akram{c}")), "akram");
        }
    }

    /// Finding R02-4: a key built from a key is the same key. These are the cases found.
    #[test]
    fn keys_are_stable_r02_4() {
        // 'İ' lower-cases to "i" + U+0307, which comes after U+064B (class 27) unordered
        // unless NFC runs after lower-casing.
        let once = search_en("\u{0130}\u{064B}");
        assert_eq!(once, "i\u{064B}\u{0307}");
        assert_eq!(search_en(&once), once);
        // Removing U+0657 puts "e" and U+0300 side by side, which compose into "è".
        let once = search_ur("e\u{0657}\u{0300}");
        assert_eq!(once, "\u{00E8}");
        assert_eq!(search_ur(&once), once);
        // A letter and its hamza are one letter before and after: alef + hamza above is أ.
        assert_eq!(normalize_urdu("\u{0627}\u{0654}"), "\u{0623}");
        assert_eq!(normalize_urdu("\u{0623}\u{0654}"), "\u{0623}");
    }

    #[test]
    fn script_checks_fr_7_28() {
        assert!(is_urdu_name("ارشد ملتانی"));
        assert!(is_urdu_name("محمد (ملک) & سنز 2"));
        assert!(is_urdu_name("مُحَمَّد"));
        assert!(!is_urdu_name("Arshad"));
        assert!(!is_urdu_name("ارشد a"));
        assert!(!is_urdu_name("123"));
        assert!(!is_urdu_name("ارشد_"));
        assert!(is_english_name("Arshad Multani"));
        assert!(is_english_name("M. Akram & Sons (Pvt) 2/3-A'"));
        assert!(!is_english_name("ارشد"));
        assert!(!is_english_name("Arshad ارشد"));
        assert!(!is_english_name("José"));
        assert!(!is_english_name("123"));
        assert!(is_latin_letter('é'));
        assert!(!is_latin_letter('ا'));
        assert!(is_urdu_letter('ٹ'));
        assert!(!is_urdu_letter('\u{064E}'));
    }

    #[test]
    fn urdu_sort_order_nfr_urd_19() {
        let collator = UrduCollator::new().unwrap();
        let expected = [
            "اکرم",
            "بشیر",
            "پرویز",
            "تنویر",
            "ٹماٹر",
            "ثاقب",
            "جمیل",
            "چمن",
            "حامد",
            "خالد",
            "دلاور",
            "ڈاکٹر",
            "ذاکر",
            "رحیم",
            "ڑ",
            "زاہد",
            "ژالہ",
            "ساجد",
            "شاہد",
            "صابر",
            "ضمیر",
            "طارق",
            "ظفر",
            "عارف",
            "غفار",
            "فاروق",
            "قاسم",
            "کریم",
            "گلزار",
            "لطیف",
            "مقبول",
            "نوید",
            "وسیم",
            "ہارون",
            "یاسر",
        ];
        let mut shuffled: Vec<&str> = expected.iter().rev().copied().collect();
        shuffled.swap(3, 17);
        shuffled.swap(0, 30);
        // آ sorts with ا (a secondary difference), so it is not in this list.
        collator.sort(&mut shuffled);
        assert_eq!(shuffled, expected);
        assert_eq!(collator.compare("ب", "پ"), Ordering::Less);
        assert_eq!(collator.compare("ت", "ٹ"), Ordering::Less);
    }
}
