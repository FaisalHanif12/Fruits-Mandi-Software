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

pub(crate) fn has(question: &str, phrases: &[&str]) -> bool {
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

/// The question asks for one year ahead in words, not digits: "agle ek saal", "next one year". It is
/// 12 months. (A number with months, "agle 12 mahine", is read as a number.)
///
/// Example: "agle ek saal ka commission ka andaza lagao" is true; "agle 6 mahine" is false.
pub fn year_ahead(question: &str) -> bool {
    has(
        question,
        &[
            "agle ek saal",
            "agla ek saal",
            "ek saal ka andaza",
            "ek saal ke andaza",
            "aane wale ek saal",
            "next one year",
            "one year ahead",
            "a year ahead",
            "one year from now",
            "اگلے ایک سال",
            "اگلا ایک سال",
            "ایک سال کا اندازہ",
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
                "doubles",
                "doubled",
                "doubling",
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
            &[
                "half", "halves", "halved", "aadha", "adha", "aadhi", "adhi", "آدھا", "آدھی",
            ],
        ),
        (
            Scenario::Triple,
            &[
                "triple",
                "triples",
                "tripled",
                "tripling",
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

/// The words for "more": zyada, barh, increase, up.
const MORE_WORDS: &[&str] = &[
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
];

/// The words for "less": kam, ghat, decrease, down.
const LESS_WORDS: &[&str] = &[
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
];

/// The question says "more": zyada, barh, increase, up.
pub fn says_more(question: &str) -> bool {
    has(question, MORE_WORDS)
}

/// The question says "less": kam, ghat, decrease, down.
pub fn says_less(question: &str) -> bool {
    has(question, LESS_WORDS)
}

/// A percent more or less that the question asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PercentChange {
    /// "10 percent zyada": the figure with the percent added.
    More,
    /// "10 percent kam": the figure with the percent taken off.
    Less,
}

/// The words that may stand between a percent number and its "more" or "less": the unit and a
/// link word.
const PERCENT_FILLER: &[&str] = &[
    "percent", "fisad", "feesad", "fi", "sad", "pc", "pct", "se", "say", "than", "فیصد", "سے",
];

/// The question asks for a figure "with a percent more or less": a what-if (agar, if), one percent
/// number, and the word for more or less right after it ("10 percent zyada", "10% higher").
/// Refused (`None`) when the percent is a rate or a past fact: "commission 7% hota to kitna
/// zyada milta" has the word too far from the number, and "is saal ka commission pichle saal se
/// 10% zyada hai" is no what-if (money review, plan 52 P52-S1).
///
/// Example: "agar is saal commission 10 percent zyada hota" is `More`; "agar commission 7% hota to
/// kitna zyada milta" and "commission 10% zyada hai" are `None`.
pub fn percent_change(question: &str, percent_numbers: usize) -> Option<PercentChange> {
    if percent_numbers != 1 || !conditional(question) {
        return None;
    }
    let spaced = spaced(question);
    let words: Vec<&str> = spaced.split_whitespace().collect();
    let key = |w: &str| text::search_ur(&text::search_en(w));
    let more: Vec<String> = MORE_WORDS.iter().map(|w| key(w)).collect();
    let less: Vec<String> = LESS_WORDS.iter().map(|w| key(w)).collect();
    let filler: Vec<String> = PERCENT_FILLER.iter().map(|w| key(w)).collect();
    let (mut said_more, mut said_less) = (false, false);
    for (at, word) in words.iter().enumerate() {
        if word.is_empty() || !word.chars().all(|c| c.is_ascii_digit()) {
            continue;
        }
        // The next words: fillers, then one word for more or less.
        for next in words.iter().skip(at.saturating_add(1)).take(3) {
            if more.iter().any(|m| m == next) {
                said_more = true;
                break;
            }
            if less.iter().any(|m| m == next) {
                said_less = true;
                break;
            }
            if !filler.iter().any(|f| f == next) {
                break;
            }
        }
    }
    match (said_more, said_less) {
        (true, false) => Some(PercentChange::More),
        (false, true) => Some(PercentChange::Less),
        _ => None,
    }
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
        // The English forms count: "doubles", "doubled" (plan 52 P52-S1); a longer word does not.
        assert_eq!(scenario("if the pace doubles"), Some(Scenario::Double));
        assert_eq!(scenario("if the pace tripled"), Some(Scenario::Triple));
        assert_eq!(scenario("if the pace halves"), Some(Scenario::Half));
        assert_eq!(
            scenario("if the doublet is sold"),
            None,
            "doublet is not the word"
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

/// The period words the app can read before the first call: the five kinds the live run of
/// 03-11-2026 asked for in 55 of 62 period-first questions (plan 52 P52-S1).
const PERIOD_CUES: &[(&str, &[&str])] = &[
    (
        "this_year",
        &[
            "is saal",
            "is sal",
            "ye saal",
            "yeh saal",
            "this year",
            "اس سال",
        ],
    ),
    (
        "this_month",
        &[
            "is mahine",
            "is mahina",
            "ye mahina",
            "yeh mahina",
            "this month",
            "اس مہینے",
            "اس مہینہ",
        ],
    ),
    (
        "last_month",
        &[
            "pichle mahine",
            "pichla mahina",
            "pichle mahina",
            "last month",
            "previous month",
            "پچھلے مہینے",
            "پچھلا مہینہ",
        ],
    ),
    (
        "last_year",
        &[
            "pichle saal",
            "pichla saal",
            "last year",
            "پچھلے سال",
            "پچھلا سال",
        ],
    ),
    (
        "yesterday",
        &[
            "kal ki",
            "kal ka",
            "kal ke",
            "yesterday",
            "کل کی",
            "کل کا",
            "کل کے",
        ],
    ),
];

/// The one period the question names, when it names exactly one of the kinds above, holds no
/// number and does not say "to the end of the year". `None` for none, for two (a comparison), and
/// for any other period (a month name, a day, "last 3 months", "agle 6 mahine"): the model asks.
pub fn period_kind(question: &str) -> Option<&'static str> {
    if question.chars().any(char::is_numeric) || year_end(question) {
        return None;
    }
    let mut found = PERIOD_CUES
        .iter()
        .filter(|(_, cues)| has(question, cues))
        .map(|(kind, _)| *kind);
    let kind = found.next()?;
    found.next().is_none().then_some(kind)
}

/// The phrases of the questions a tile answers: today's sales, the shop's total dues, what is owed
/// to sellers, the open advances, the dead amount, the cash in hand, today's expenses and the
/// unsold stock (D6-system-prompt, the tiles). Whole phrases, so a party's own question
/// ("ikram ke total bakaya") is never one.
const TILE_PHRASES: &[&str] = &[
    "aaj ki bikri",
    "aj ki bikri",
    "aaj ki khaam bikri",
    "آج کی بکری",
    "آج کی خام بکری",
    "today s sales",
    "todays sales",
    "today s total sales",
    "what were today s",
    "aaj ka kharcha",
    "آج کا خرچہ",
    "kul ugrahi",
    "kul ugrai",
    "kul wasooli",
    "کل وصولی",
    "کل اگراہی",
    "total dues",
    "sab gaahkon ka bakaya",
    "sab gahakon ka bakaya",
    "byopariyon ko kitna",
    "بیوپاریوں کو کتنا",
    "peshgi kitni",
    "پیشگی کتنی",
    "advance kitni",
    "dubi hui raqam",
    "dubi hoi rakam",
    "dobi raqam",
    "ڈوبی رقم",
    "dead amount",
    "cash mein kitna",
    "cash kitna",
    "cash in hand",
    "cash do we have",
    "روکڑ کتنی",
    "موجود روکڑ",
    "gale mein",
    "gale me",
    "baki mal kitna",
    "baqi maal kitna",
    "باقی مال کتنا",
    "unsold stock",
];

/// True when the question is one a tile answers (see [`TILE_PHRASES`]) and holds no number.
pub fn tile_question(question: &str) -> bool {
    !question.chars().any(char::is_numeric) && has(question, TILE_PHRASES)
}

#[cfg(test)]
mod prefetch_tests {
    use super::*;

    #[test]
    fn exactly_one_period_of_the_five_kinds_is_named() {
        assert_eq!(period_kind("is mahine ki bikri"), Some("this_month"));
        assert_eq!(period_kind("اس سال کتنا کمیشن ہوا"), Some("this_year"));
        assert_eq!(
            period_kind("pichle mahine kitni bikriyan hui"),
            Some("last_month")
        );
        assert_eq!(
            period_kind("how much commission last year"),
            Some("last_year")
        );
        assert_eq!(period_kind("کل کی بکری کتنی تھی"), Some("yesterday"));
        // Two periods (a comparison), a number, the end of the year, another period: none.
        assert_eq!(
            period_kind("is mahine ka commission pichle mahine se zyada hai"),
            None
        );
        assert_eq!(period_kind("pichle 3 mahine ka commission"), None);
        assert_eq!(period_kind("is saal ke aakhir tak commission kitna"), None);
        assert_eq!(period_kind("september ka commission"), None);
        assert_eq!(period_kind("agle mahine commission kitna aayega"), None);
        assert_eq!(period_kind("ikram ka bakaya"), None);
    }

    #[test]
    fn a_tile_question_is_a_whole_phrase_and_a_partys_question_is_not() {
        for q in [
            "aaj ki bikri kitni hui",
            "what were today's total sales",
            "kul ugrahi kitni hai",
            "total dues from all buyers",
            "پیشگی کتنی کھلی ہوئی ہے",
            "how much cash do we have in hand",
            "gale me is waqt kitna paisa he",
            "baki mal kitna he",
        ] {
            assert!(tile_question(q), "{q}");
        }
        for q in [
            "ikram ke total bakaya kitne hai",
            "akram ka cash",
            "pichle mahine ki bikri",
            "cash of 5000",
        ] {
            assert!(!tile_question(q), "{q}");
        }
    }
}

#[cfg(test)]
mod percent_change_tests {
    use super::*;

    #[test]
    fn a_what_if_with_one_percent_and_the_word_right_after_it_is_a_percent_change() {
        for q in [
            "agar is saal commission 10 percent zyada hota to kitna hota",
            "if this year's commission had been 10 percent higher, what would it be",
            "agar commission 10% zyada hota to kitna hota",
            "agar sales 15 percent se kam hoti to",
        ] {
            let found = percent_change(q, 1);
            assert!(found.is_some(), "{q}");
        }
        assert_eq!(
            percent_change("agar commission 10 percent zyada hota to kitna", 1),
            Some(PercentChange::More)
        );
        assert_eq!(
            percent_change("agar sales 15 percent kam hoti to kitni hoti", 1),
            Some(PercentChange::Less)
        );
    }

    #[test]
    fn a_rate_a_past_fact_or_two_percents_are_not() {
        // The word is far from the number: 7% is a rate.
        assert_eq!(
            percent_change("agar commission 7% hota to kitna zyada milta", 1),
            None
        );
        // No what-if: a fact, and the reverse question.
        assert_eq!(
            percent_change(
                "is saal ka commission pichle saal se 10% zyada hai to pichle saal kitna tha",
                1
            ),
            None
        );
        // Two percent numbers (or none), and both more and less after numbers.
        assert_eq!(
            percent_change("agar 6% ki jagah 10% zyada hota to", 2),
            None
        );
        assert_eq!(percent_change("agar commission zyada hota to", 0), None);
        assert_eq!(
            percent_change("agar 10 percent zyada aur 5 percent kam hota", 1),
            None
        );
    }
}

#[cfg(test)]
mod year_ahead_tests {
    use super::*;

    #[test]
    fn one_year_in_words_is_twelve_months_ahead_and_other_phrases_are_not() {
        for q in [
            "agle ek saal ka commission ka andaza lagao",
            "اگلے ایک سال کے کمیشن کا اندازہ لگائیں",
            "estimate the commission for the next one year",
            "commission one year ahead",
        ] {
            assert!(year_ahead(q), "{q}");
        }
        for q in [
            "agle 6 mahine ka andaza",
            "is saal ke aakhir tak commission kitna",
            "next year ka commission",
            "pichle saal ka commission",
        ] {
            assert!(!year_ahead(q), "{q}");
        }
    }
}
