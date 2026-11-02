//! What the Owner SAID, where a figure needs his word for it (plan 52 item 5, money review).
//!
//! The model never types a number, and a scenario is not a number it may invent: "by the end of the
//! year", "if the pace doubles" and "if it goes up 10%" are only worked out when the question says
//! so. This module reads the words, in Roman Urdu, Urdu letters and English, as whole words of the
//! normalised question. A cue that is not there, or two that disagree, is refused by the tool and
//! the model asks the Owner.
//!
//! Example: "is raftar se saal ke aakhir tak commission kitna" has the year-end cue;
//! "agar raftar double ho jaye to" has the double cue; "10% zyada" has the more cue.

use mandi_services::core::text;

/// A scenario word: how the pace is scaled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scenario {
    /// The pace doubles.
    Double,
    /// The pace halves.
    Half,
    /// The pace triples.
    Triple,
}

/// The question as normalised whole words, with a single space between them and at both ends, so
/// a phrase is found only as whole words.
fn spaced(question: &str) -> String {
    let normal = text::search_ur(&text::search_en(question));
    let words: Vec<&str> = normal
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .collect();
    format!(" {} ", words.join(" "))
}

fn has(question: &str, phrases: &[&str]) -> bool {
    let spaced = spaced(question);
    phrases.iter().any(|p| {
        let key = text::search_ur(&text::search_en(p));
        spaced.contains(&format!(" {key} "))
    })
}

/// The question says "to the end of the year".
pub fn year_end(question: &str) -> bool {
    has(
        question,
        &[
            "saal ke aakhir",
            "saal ke akhir",
            "saal ke end",
            "saal ka aakhir",
            "saal ka akhir",
            "saal khatam",
            "saal ke khatam",
            "sal ke aakhir",
            "sal ke akhir",
            "december tak",
            "year end",
            "end of the year",
            "end of year",
            "end of this year",
            "by the year end",
            "سال کے آخر",
            "سال کا آخر",
            "سال کے اختتام",
            "سال ختم",
            "دسمبر تک",
        ],
    )
}

/// The question asks "what if": agar, if, suppose, فرض (money review: a past fact such as "commission
/// doubled last year" or "up to December" is not a scenario).
pub fn conditional(question: &str) -> bool {
    has(
        question,
        &[
            "agar",
            "agr",
            "magar agar",
            "if",
            "suppose",
            "farz",
            "farz karo",
            "what if",
            "اگر",
            "فرض",
            "فرض کریں",
        ],
    )
}

/// The scenario words of the question. More than one kind in one question is returned as `None`
/// by [`scenario`], so the tool refuses and asks.
fn scenarios(question: &str) -> Vec<Scenario> {
    let mut found = Vec::new();
    let table: [(Scenario, &[&str]); 3] = [
        (
            Scenario::Double,
            &[
                "double",
                "dugna",
                "dugni",
                "dogna",
                "dogni",
                "doguna",
                "duguna",
                "twice",
                "do guna",
                "دگنا",
                "دگنی",
                "دوگنا",
                "دوگنی",
                "دو گنا",
            ],
        ),
        (
            Scenario::Half,
            &["half", "aadha", "adha", "aadhi", "adhi", "آدھا", "آدھی"],
        ),
        (
            Scenario::Triple,
            &[
                "triple",
                "tigna",
                "tigni",
                "thrice",
                "teen guna",
                "تگنا",
                "تگنی",
                "تین گنا",
            ],
        ),
    ];
    for (scenario, words) in table {
        if has(question, words) {
            found.push(scenario);
        }
    }
    found
}

/// The one scenario word the question holds, or `None` for none or for several kinds.
pub fn scenario(question: &str) -> Option<Scenario> {
    match scenarios(question).as_slice() {
        [one] => Some(*one),
        _ => None,
    }
}

/// True when the question holds more than one kind of scenario word.
pub fn scenarios_clash(question: &str) -> bool {
    scenarios(question).len() > 1
}

/// The question says "more": zyada, barh, increase, up.
pub fn says_more(question: &str) -> bool {
    has(
        question,
        &[
            "zyada",
            "ziyada",
            "zyadah",
            "barh",
            "barhe",
            "barhta",
            "barhay",
            "barhna",
            "badh",
            "increase",
            "increases",
            "more",
            "higher",
            "up",
            "اضافہ",
            "زیادہ",
            "بڑھ",
            "بڑھے",
            "بڑھنا",
        ],
    )
}

/// The question says "less": kam, ghat, decrease, down.
pub fn says_less(question: &str) -> bool {
    has(
        question,
        &[
            "kam",
            "kami",
            "ghat",
            "ghate",
            "ghatay",
            "ghatna",
            "decrease",
            "decreases",
            "less",
            "lower",
            "down",
            "drop",
            "کم",
            "کمی",
            "گھٹ",
            "گھٹے",
            "گھٹنا",
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_end_of_the_year_is_found_in_every_script() {
        for q in [
            "is raftar se saal ke aakhir tak commission kitna hoga",
            "commission by the end of the year",
            "what will sales be by the end of this year",
            "december tak kitna",
            "اس رفتار سے سال کے آخر تک کمیشن کتنا",
        ] {
            assert!(year_end(q), "{q}");
        }
        for q in [
            "agle 6 mahine",
            "is saal ka commission",
            "year over year",
            "pichla saal",
        ] {
            assert!(!year_end(q), "{q}");
        }
    }

    #[test]
    fn a_scenario_word_is_found_as_a_whole_word_and_two_kinds_clash() {
        assert_eq!(
            scenario("agar raftar double ho jaye to"),
            Some(Scenario::Double)
        );
        assert_eq!(
            scenario("if the pace doubles"),
            None,
            "doubles is not the word"
        );
        assert_eq!(scenario("agar raftar dugni ho"), Some(Scenario::Double));
        assert_eq!(scenario("رفتار دگنی ہو تو"), Some(Scenario::Double));
        assert_eq!(scenario("agar raftar aadhi reh jaye"), Some(Scenario::Half));
        assert_eq!(scenario("if the pace is half"), Some(Scenario::Half));
        assert_eq!(scenario("agar raftar teen guna ho"), Some(Scenario::Triple));
        assert_eq!(scenario("agar raftar 10% zyada ho"), None);
        assert!(scenarios_clash("double ya half"));
        assert_eq!(scenario("double ya half"), None);
    }

    #[test]
    fn a_scenario_needs_an_if() {
        assert!(conditional("agar raftar double ho jaye to"));
        assert!(conditional("what if the pace doubles"));
        assert!(conditional("اگر رفتار دگنی ہو تو"));
        assert!(!conditional(
            "is saal commission double hua, agle 6 mahine?"
        ));
        assert!(!conditional("commission pichle saal se 10% zyada hai"));
    }

    #[test]
    fn more_and_less_words() {
        assert!(says_more("agar raftar 10% zyada ho jaye"));
        assert!(says_more("if the pace goes up 10%"));
        assert!(says_less("agar raftar 20% kam ho jaye"));
        assert!(says_less("if the pace goes down by 20%"));
        assert!(!says_more("agar raftar 20% kam ho jaye"));
        assert!(!says_less("agar raftar 10% zyada ho jaye"));
    }
}
