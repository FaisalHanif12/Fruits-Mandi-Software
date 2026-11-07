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

/// The question asks for a partner's capital (the money he put in), and not for his کل رقم.
///
/// A partner's statement carries his کل رقم (everything he has in the firm). His capital is one
/// part of it, and a figure of its own named `balance`, so a question that did not ask for it must
/// not be shown it: "alif ki kul raqam" was answered with his capital, 0, with a tick (live run 1,
/// Q198, Q199; Q546 to Q548: Usman's capital 200,000 for his کل رقم 2,506,856).
///
/// Example: "be ka sarmaya kitna laga hua hai" is true; "usman ka kitna hai abhi" is false.
pub fn capital_asked(question: &str) -> bool {
    has(
        question,
        &[
            "sarmaya",
            "sarmayah",
            "sarmaiya",
            "sarmaye",
            "punji",
            "paisa lagaya",
            "paise lagaye",
            "capital",
            "investment",
            "investments",
            "invest",
            "invests",
            "invested",
            "put in",
            "سرمایہ",
            "سرمایا",
            "سرمائے",
            "پونجی",
            "سرمایہ کاری",
        ],
    )
}

/// The question asks for the commission.
///
/// Example: "raheem ke mal ka commision is sal kitna he" is true.
pub fn commission_asked(question: &str) -> bool {
    has(
        question,
        &[
            "commission",
            "commision",
            "comision",
            "comm",
            "kamishan",
            "کمیشن",
            "کمشن",
        ],
    )
}

/// The question asks for the sales (the amount), not how many بکری there were.
///
/// Example: "commission and sales on rahim's goods" is true; "pichle mahine kitni bikriyan hui" is
/// false (a count).
pub fn sales_asked(question: &str) -> bool {
    has(
        question,
        &["sale", "sales", "bikri", "bikree", "بکری", "فروخت"],
    ) && !crate::route::bikri_count_asked(question)
}

/// The question asks for the commission AND the sales in one breath: a word for each, joined by
/// "aur", "and" or "or" ("commission aur sale", "sales and commission", "comision aur bikri").
/// Two words of a question about two other things ("sale kaise barhayen, aur kele ka commission
/// kitna hai") are not that: the sales word is a topic there, not a figure asked for.
///
/// Example: "commission and sales on rahim's goods this year" is true; "sale kaise barhayen, aur
/// kele ka commission kitna hai" is false.
pub fn commission_and_sales_asked(question: &str) -> bool {
    const AND: &[&str] = &["aur", "or", "and", "ya", "plus", "اور", "و"];
    const COMMISSION: &[&str] = &[
        "commission",
        "commision",
        "comision",
        "comm",
        "kamishan",
        "کمیشن",
        "کمشن",
    ];
    const SALES: &[&str] = &["sale", "sales", "bikri", "bikree", "بکری", "فروخت"];
    if crate::route::bikri_count_asked(question) {
        return false;
    }
    let spaced = spaced(question);
    let words: Vec<&str> = spaced.split_whitespace().collect();
    let key = |w: &str| text::search_ur(&text::search_en(w));
    let is_one_of = |word: &str, list: &[&str]| list.iter().any(|w| key(w) == word);
    words.iter().enumerate().any(|(at, word)| {
        let other: &[&str] = if is_one_of(word, COMMISSION) {
            SALES
        } else if is_one_of(word, SALES) {
            COMMISSION
        } else {
            return false;
        };
        // The other word comes next, or after one link word.
        let next = words.get(at.saturating_add(1)).copied();
        let after = words.get(at.saturating_add(2)).copied();
        next.is_some_and(|n| is_one_of(n, other))
            || (next.is_some_and(|n| is_one_of(n, AND))
                && after.is_some_and(|n| is_one_of(n, other)))
    })
}

/// The question compares things: farq, difference, compare, versus, "se zyada" or "se kam", more or
/// less "than". A "sab se zyada" (the most) question is a top list and does not compare.
///
/// Example: "kele aur saib ke commission mein farq" is true; "sab se zyada commission kis item ka"
/// is false.
pub fn comparing(question: &str) -> bool {
    let strong = has(
        question,
        &[
            // "by how much", "kitna zyada": the size of a difference is asked (D7 O5).
            "by how much",
            "how much more",
            "how much less",
            "kitna zyada",
            "kitne zyada",
            "kitni zyada",
            "kitna ziada",
            "kitna kam",
            "kitne kam",
            "کتنا زیادہ",
            "کتنے زیادہ",
            "کتنی زیادہ",
            "کتنا کم",
            "کتنے کم",
            "farq",
            "fark",
            "difference",
            "differ",
            "compare",
            "compared",
            "comparison",
            "mawazna",
            "muqabla",
            "muqable",
            "muqabale",
            "versus",
            "vs",
            "فرق",
            "موازنہ",
            "مقابلے",
            "مقابلہ",
        ],
    );
    // "akram se kitna zyada": "se" with a word for more or less within three words after it, and not
    // "sab se zyada" (the most). "more than", "less than" and the like are plain phrases.
    let spaced = spaced(question);
    let words: Vec<&str> = spaced.split_whitespace().collect();
    let key = |w: &str| text::search_ur(&text::search_en(w));
    let se: Vec<String> = ["se", "سے"].iter().map(|w| key(w)).collect();
    let sab: Vec<String> = ["sab", "سب"].iter().map(|w| key(w)).collect();
    let more_less: Vec<String> = [
        "zyada",
        "ziyada",
        "ziada",
        "zayada",
        "zyadah",
        "kam",
        "زیادہ",
        "کم",
    ]
    .iter()
    .map(|w| key(w))
    .collect();
    let se_than = words.iter().enumerate().any(|(at, word)| {
        se.iter().any(|s| s == word)
            // Not "sab se zyada" (the most), and not "kis item se sab se zyada" either: a "se"
            // followed by "sab" starts the top-list phrase.
            && !at
                .checked_sub(1)
                .and_then(|before| words.get(before))
                .is_some_and(|before| sab.iter().any(|s| s == before))
            && !words
                .get(at.saturating_add(1))
                .is_some_and(|next| sab.iter().any(|s| s == next))
            && words
                .iter()
                .skip(at.saturating_add(1))
                .take(3)
                .any(|next| more_less.iter().any(|m| m == next))
    });
    // "more than 5000 wale" is a threshold, not a comparison: the phrase counts only when a word
    // follows that is not a number.
    let than_words = [
        "more", "less", "higher", "lower", "greater", "bigger", "smaller",
    ];
    let than = se_than
        || words.iter().enumerate().any(|(at, word)| {
            than_words.contains(word)
                && words.get(at.saturating_add(1)) == Some(&"than")
                && words
                    .get(at.saturating_add(2))
                    .is_some_and(|next| !next.chars().any(char::is_numeric))
        });
    strong || than
}

/// The question asks for a share or a change in percent: percent, fisad, share, hissa, ratio,
/// "of all", "out of", change, increase, decrease, or it compares. (A percent number of the
/// question is checked beside this, where the numbers are known.)
///
/// Example: "kele ka hissa kitna hai" is true; "kela ka commission" is false.
pub fn share_or_change(question: &str) -> bool {
    comparing(question)
        || has(
            question,
            &[
                "percent",
                "percentage",
                "persent",
                "fisad",
                "feesad",
                "share",
                "hissa",
                "hissah",
                "ratio",
                "change",
                "barh",
                "barhe",
                "ghat",
                "ghate",
                "increase",
                "decrease",
                "of all",
                "out of",
                "mein se",
                "فیصد",
                "پرسنٹ",
                "حصہ",
                "اضافہ",
                "کمی",
                "تبدیلی",
            ],
        )
}

/// The question asks for an average: average, avg, ausat.
///
/// Example: "kele aur saib ke commission ka average" is true.
pub fn average_asked(question: &str) -> bool {
    has(
        question,
        &[
            "average", "avg", "ausat", "ausatan", "osat", "avarage", "averege", "mean", "اوسط",
        ],
    )
}

/// The question asks for ONE figure for several things together: "mila kar", "total", "kul",
/// "together", "combined". Without such a word, "kele or saib ka commission" asks for each item's
/// own figure, and a sum of the two is a figure nobody asked for (Q112 of the live run 2).
///
/// Example: "akram aur arshad dono ka mila kar kitna hai" is true; "kele or saib ka comision aur
/// bikri btao" is false.
pub fn together(question: &str) -> bool {
    !average_asked(question)
        && !asked_apart(question)
        && (has(question, &TOGETHER_STRONG) || has(question, &TOGETHER_WEAK))
}

/// The question says all of them together in words that cannot mean one thing's own total:
/// "mila kar", "dono ka", "sab ka", "combined". A sum of every row or every match needs this: a
/// bare "total" can be one item's total ("kele ka total commission").
///
/// Example: "sab items ka commission mila kar" is true; "kele ka total commission" is false.
pub fn together_strong(question: &str) -> bool {
    !average_asked(question) && !asked_apart(question) && has(question, &TOGETHER_STRONG)
}

/// The question asks for each one on its own: "alag alag", "separately", "har ek".
fn asked_apart(question: &str) -> bool {
    has(
        question,
        &[
            "alag alag",
            "separately",
            "each one",
            "each of",
            "har ek",
            "ek ek",
            "الگ الگ",
            "ہر ایک",
        ],
    )
}

const TOGETHER_STRONG: [&str; 26] = [
    "mila kar",
    "mila ke",
    "milakar",
    "milake",
    "combined",
    "together",
    "dono ka",
    "donon ka",
    "dono ki",
    "dono ke",
    "donon ki",
    "donon ke",
    "jama karo",
    "jor kar",
    "jod kar",
    "sab ka",
    "sab ko mila kar",
    "دونوں کا",
    "دونوں کی",
    "دونوں کے",
    "ملا کر",
    "ملا کے",
    "سب کا",
    "جوڑ کر",
    "جمع کرو",
    "کل ملا کر",
];

/// Words for a total that fit one thing's own total too ("kele ka commission in total", "overall",
/// "both sides"): two named things with one of them are one sum; every row of a table with one of
/// them is not.
const TOGETHER_WEAK: [&str; 12] = [
    "total",
    "kul",
    "sum",
    "in total",
    "in all",
    "altogether",
    "overall",
    "both",
    "plus",
    "add",
    "ٹوٹل",
    "میزان",
];

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

/// The question says "next month" (one month ahead) in words, with no number: "agle mahine", "next
/// month", "اگلے مہینے". It is 1 month, as "agle ek saal" is 12 (the plural "agle mahino mein" is not
/// it: that is "the coming months", 6). A number with months ("agle 3 mahine") is read as a number.
///
/// Example: "saag agle mahine kitna bikega" is true; "agle mahino mein kitna" and "agle 3 mahine" are
/// false.
pub fn next_month(question: &str) -> bool {
    has(
        question,
        &[
            "agle mahine",
            "agla mahina",
            "agle mahine tak",
            "aglay mahine",
            "next month",
            "اگلے مہینے",
            "اگلے ماہ",
            "اگلا مہینہ",
            "اگلا ماہ",
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
    fn an_average_question_is_not_a_together_question() {
        for q in [
            "kele aur saib dono ka average",
            "sab ka average commission",
            "mila kar average",
            "dono ka ausatan kitna",
        ] {
            assert!(average_asked(q), "{q}");
            assert!(!together(q), "{q}");
        }
        for q in [
            "dono ki mila kar",
            "kele aur saib dono ke commission",
            "both banana and apple",
            "jod kar batao",
        ] {
            assert!(together(q), "{q}");
        }
        assert!(!together("kele aur saib ka commission"));
    }

    #[test]
    fn a_top_list_or_a_threshold_is_not_a_comparison() {
        for q in [
            "kis item se sab se zyada commission aya",
            "sab se zyada bikne wala item",
            "5000 se more than 5000 wale buyers",
            "more than 5000 wale buyers",
        ] {
            assert!(!comparing(q), "{q}");
        }
        for q in [
            "akram se kitna zyada hai arshad",
            "banana is more than apple",
            "kela saib se ziada hai",
            "kele ka saib se farq",
        ] {
            assert!(comparing(q), "{q}");
        }
        // A plain amount question is not a share.
        assert!(!share_or_change("kele ka commission kitna banta hai"));
    }

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
mod together_tests {
    use super::*;

    #[test]
    fn commission_and_sales_are_asked_when_the_two_words_are_joined() {
        for q in [
            "rahim ke mal ka commission aur sale kitna he",
            "commission and sales on rahim's goods this year",
            "bikri aur commission btao",
            "sales commission this month",
            "کمیشن اور بکری بتائیں",
        ] {
            assert!(commission_and_sales_asked(q), "{q}");
        }
        for q in [
            "sale kaise barhayen, aur kele ka commission kitna hai",
            "rahim ke mal ka commission kitna he",
            "is saal ki bikri kitni hui",
            "rahim ki kitni bikriyan hui aur commission",
            "commission pichle mahine kitna tha, bikri ki baat nahi",
        ] {
            assert!(!commission_and_sales_asked(q), "{q}");
        }
    }

    #[test]
    fn only_a_word_for_one_figure_makes_a_sum_wanted() {
        for q in [
            "akram aur arshad dono ka mila kar kitna hai",
            "kele or saib ka total commission",
            "what is the combined commission of banana and apple",
            "کیلے اور سیب کا ٹوٹل کمیشن",
        ] {
            assert!(together(q), "{q}");
        }
        for q in [
            "kele or saib ka is sal ka comision aur bikri btao",
            "commission of banana and apple this year",
            "کیلے اور سیب کا کمیشن بتائیں",
            // کل is also tomorrow or yesterday, جمع also the credit side: not "together" alone.
            "کل کا کمیشن اور جمع کا کمیشن",
        ] {
            assert!(!together(q), "{q}");
        }
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

/// True when the question asks HOW MANY of something, not how much money: "the number of bills", "how
/// many sellers", "kitne bayopariyon ko", "کتنے بل". An instant answer that holds no count leaves such
/// a part out, so the question goes to the model (D7 O3, O4). "Kitne paise", "kitne ka" and "how many
/// rupees" ask an amount and are not a count.
///
/// Example: "today's sales, the number of bills and the commission" is true; "kitne paise dene hain" is
/// false.
pub fn asks_count(question: &str) -> bool {
    // "number of", "count of", "how many": a count, unless money or months follow.
    const ANY: [&str; 9] = [
        "number of",
        "count of",
        "how many",
        "how many times",
        "kitni baar",
        "kitni bar",
        "kitni dafa",
        "کتنی بار",
        "کتنی دفعہ",
    ];
    const MONEY: [&str; 11] = [
        "rupees", "rupee", "money", "paise", "paisa", "rs", "months", "month", "روپے", "پیسے",
        "رقم",
    ];
    // "kitne", "kitni": a count only before a word for things that are counted ("kitne bayopariyon",
    // "kitni bikriyan"); "kitni bikri" and "kitne paise" ask an amount.
    const HEADS: [&str; 5] = ["kitne", "kitni", "kitnay", "کتنے", "کتنی"];
    const COUNTED: [&str; 45] = [
        "bill",
        "bills",
        "bil",
        "bikriyan",
        "bikriyon",
        "bikrian",
        "بل",
        "بلز",
        "بکریاں",
        "بکریوں",
        "bayopari",
        "bayopariyon",
        "byopari",
        "byopariyon",
        "beopari",
        "beopariyon",
        "gahak",
        "gahakon",
        "buyers",
        "sellers",
        "customers",
        "parties",
        "party",
        "log",
        "logon",
        "partners",
        "items",
        "lots",
        "lot",
        "trucks",
        "entries",
        "گاہک",
        "گاہکوں",
        "بیوپاریوں",
        "لوگوں",
        "لوگ",
        "logo",
        "people",
        "persons",
        "person",
        "accounts",
        "کھاتے",
        "کھاتوں",
        "khate",
        "khaton",
    ];
    let spaced = spaced(question);
    let words: Vec<&str> = spaced.split_whitespace().collect();
    let key = |w: &str| text::search_ur(&text::search_en(w));
    let money: Vec<String> = MONEY.iter().map(|w| key(w)).collect();
    let counted: Vec<String> = COUNTED.iter().map(|w| key(w)).collect();
    let heads: Vec<String> = HEADS.iter().map(|w| key(w)).collect();
    let any_head = ANY.iter().any(|head| {
        let head: Vec<String> = head.split(' ').map(key).collect();
        words.windows(head.len()).enumerate().any(|(at, window)| {
            window.iter().zip(&head).all(|(w, h)| *w == h.as_str())
                && words
                    .get(at.saturating_add(head.len()))
                    .is_some_and(|next| !money.iter().any(|m| m == next))
        })
    });
    any_head
        || words.iter().enumerate().any(|(at, word)| {
            heads.iter().any(|h| h == word)
                && words
                    .get(at.saturating_add(1))
                    .is_some_and(|next| counted.iter().any(|c| c == next))
        })
}
