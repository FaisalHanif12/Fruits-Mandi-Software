//! Figures typed by the model (PRD FR-7.75, FR-7.76, NFR-AI-1, Code Standards Q4).
//!
//! `robustness_assistant.rs` attack 3 has 14 digit forms, Urdu number words, months, a typed ✓,
//! the other side's words, next questions with a figure and a reference to an earlier question.
//! This file adds what it does not have: a digit of every Unicode script, quantity words that are
//! not on the list, letters that look like the words on the list (zero-width, look-alike), a check
//! mark in other shapes, month spellings, and figures put in the other fields of the answer.
//!
//! A case is a final answer the model writes (the same twice: the answer and its one rewrite). It
//! is refused if the outcome is `ChecksFailed` with nothing of the model shown. A case that is
//! shown instead is a typed figure on the Owner's screen.
#![allow(dead_code)]

use std::collections::BTreeSet;

use mandi_assistant::answer::AnswerPart;
use mandi_assistant::ask::{AskOutcome, AskReply};
use mandi_services::core::auth::PrefKey;
use mandi_services::settings;
use serde_json::{Value, json};

use crate::harness::{Harness, answer_of};
use crate::kit::*;

/// One answer the model writes, in a language, and the one thing the app must do with it.
struct Case {
    name: String,
    english: bool,
    text: String,
    next: Vec<String>,
}

fn case(name: &str, english: bool, text: &str) -> Case {
    Case {
        name: name.to_owned(),
        english,
        text: text.to_owned(),
        next: Vec::new(),
    }
}

/// What came of one case.
struct Run {
    name: String,
    outcome: AskOutcome,
    /// Words of the model that reached the screen (empty when refused).
    shown: bool,
    rules: BTreeSet<String>,
}

fn language(h: &Harness, english: bool) {
    settings::set_pref(
        &h.world.env.ctx,
        &h.world.owner,
        PrefKey::Language,
        if english { "en" } else { "ur" },
    )
    .unwrap();
}

fn shown(reply: &AskReply) -> bool {
    let a = answer_of(reply);
    a.parts.iter().any(|p| matches!(p, AnswerPart::Text { .. })) || !a.next.is_empty()
}

fn run_cases(cases: &[Case]) -> Vec<Run> {
    let mut out = Vec::new();
    for chunk in cases.chunks(15) {
        let h = Harness::day();
        for c in chunk {
            language(&h, c.english);
            let content = answer_full(
                "answer",
                &c.text,
                json!([]),
                json!(c.next),
                Value::Null,
                Value::Null,
            );
            let (reply, requests) = ask(
                &h,
                "ikram ke total bakaya kitne hai",
                None,
                akram_then(vec![final_reply(&content), final_reply(&content)]),
            );
            let rules = requests.last().map(failed_rules).unwrap_or_default();
            out.push(Run {
                name: c.name.clone(),
                outcome: reply.outcome,
                shown: shown(&reply),
                rules,
            });
        }
        h.scan_requests();
    }
    out
}

/// The cases that were not refused: the model's words reached the screen.
fn passed(runs: &[Run]) -> Vec<String> {
    runs.iter()
        .filter(|r| r.outcome != AskOutcome::ChecksFailed || r.shown)
        .map(|r| format!("{} -> {:?}", r.name, r.outcome))
        .collect()
}

const WHO: &str = "{{r1.matches.0.name}}";

// ---- every Unicode digit ----------------------------------------------------------------------

/// One character of every 128-wide block of Unicode that has a numeric character: the Nd digits of
/// every script, and the No and Nl forms (circled, fractions, roman numerals ...).
fn numeric_samples() -> Vec<(String, char)> {
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    for code in 0u32..0x2_0000 {
        let Some(c) = char::from_u32(code) else {
            continue;
        };
        if c.is_numeric() && seen.insert(code >> 7) {
            out.push((format!("U+{code:04X}"), c));
        }
    }
    out
}

/// Every script's digit, typed in the middle of an Urdu answer, is refused as a digit.
#[test]
fn t5_a_numeric_character_of_every_unicode_block_is_refused() {
    let samples = numeric_samples();
    assert!(samples.len() >= 25, "{} blocks", samples.len());
    let cases: Vec<Case> = samples
        .iter()
        .map(|(name, c)| case(name, false, &format!("{WHO} کے ذمے {c} باقی ہیں۔")))
        .collect();
    let runs = run_cases(&cases);
    let missed: Vec<String> = passed(&runs);
    assert!(
        missed.is_empty(),
        "typed digits reached the screen: {missed:?}"
    );
    let not_digit: Vec<&str> = runs
        .iter()
        .filter(|r| !r.rules.contains("digit"))
        .map(|r| r.name.as_str())
        .collect();
    assert!(
        not_digit.is_empty(),
        "refused, but not as a digit: {not_digit:?}"
    );
}

// ---- words of quantity that are not on the list ------------------------------------------------

fn assert_refused(cases: &[Case]) {
    let runs = run_cases(cases);
    let missed = passed(&runs);
    assert!(
        missed.is_empty(),
        "typed words reached the screen: {missed:?}"
    );
}

fn en_cases(words: &[&str]) -> Vec<Case> {
    words
        .iter()
        .map(|w| {
            case(
                &format!("en: {w}"),
                true,
                &format!("{WHO} owes the shop about {w} rupees."),
            )
        })
        .collect()
}

fn ur_cases(words: &[&str]) -> Vec<Case> {
    words
        .iter()
        .map(|w| {
            case(
                &format!("ur: {w}"),
                false,
                &format!("{WHO} کے ذمے {w} روپے باقی ہیں۔"),
            )
        })
        .collect()
}

/// The quantity words the lists name today are refused, in both languages and in Roman Urdu.
#[test]
fn t5_the_quantity_words_of_the_lists_are_refused() {
    let mut cases = en_cases(&[
        "zero",
        "dozen",
        "triple",
        "quarter",
        "twice",
        "hundreds",
        "thousands",
        "millions",
        "billion",
        "fifteen",
        "nineteen",
        "thirty",
        "forty",
        "fifty",
        "ninety",
        "twenty-one",
        "a hundred",
        "a thousand",
        "a million",
        "half",
        "double",
        "ten percent",
        "several hundred",
        "a few thousand",
        "paanch hazaar",
        "dedh sau",
        "dhai lakh",
        "sawa lakh",
        "do lakh",
        "das hazar",
        "chalis",
        "pachas",
        "sau",
        "teen sau",
        "aath sau assi",
        "hazaar",
        "crore",
        "arab",
    ]);
    cases.extend(ur_cases(&[
        "ڈیڑھ سو",
        "ڈھائی ہزار",
        "سوا لاکھ",
        "پونے دو",
        "آدھا",
        "پونا",
        "درجن",
        "تین سو",
        "پانچ ہزار",
        "کروڑ",
        "ارب",
        "لاکھ",
        "سینکڑوں",
        "ہزاروں",
        "لاکھوں",
        "چالیس",
        "پچاس",
        "اکیس",
        "ایک سو ایک",
    ]));
    assert_refused(&cases);
}

/// Words that mean a number and are on no list: zero and two and three times and a million million
/// (English), the fractions and the dozen and the pair (Roman Urdu), the pair and the ten of days
/// (Urdu). A model that writes "Akram owes nil" or "dedh" has typed a figure. They are list gaps:
/// the digit rule is complete (every Unicode numeric character), the word rules are lists.
#[test]
fn t5_9_the_quantity_words_that_the_lists_miss_are_refused() {
    let mut cases = en_cases(&["nil", "couple", "pair", "quadruple", "thrice", "trillion"]);
    cases.extend(en_cases(&[
        "adha", "pauna", "sawa", "dedh", "dhai", "darjan", "jora", "paune do",
    ]));
    cases.extend(ur_cases(&["جوڑا", "دہائی"]));
    assert_refused(&cases);
}

// ---- letters that look like the words on the list ----------------------------------------------

/// The characters that draw nothing: a model told to hide a figure writes it with one of them
/// between its letters, so the Owner reads "thousand" and a check that splits at them sees two
/// pieces. Joiners and word joiners are handled; the rest are listed in `HIDDEN_BY_SPLIT`.
const HIDDEN_HANDLED: &[char] = &['\u{200b}', '\u{200c}', '\u{200d}', '\u{2060}', '\u{feff}'];

/// Invisible characters the check does not see through (finding T5-1).
const HIDDEN_BY_SPLIT: &[char] = &[
    '\u{00ad}', // soft hyphen
    '\u{034f}', // combining grapheme joiner
    '\u{200e}', // left-to-right mark
    '\u{200f}', // right-to-left mark
    '\u{202a}',
    '\u{202b}',
    '\u{202c}',
    '\u{202d}',
    '\u{202e}', // bidi embeddings
    '\u{2061}',
    '\u{2062}',
    '\u{2063}',
    '\u{2064}', // invisible math operators
    '\u{2066}',
    '\u{2067}',
    '\u{2068}',
    '\u{2069}',  // bidi isolates
    '\u{061c}',  // Arabic letter mark
    '\u{180e}',  // Mongolian vowel separator
    '\u{3164}',  // Hangul filler
    '\u{fe0f}',  // variation selector 16
    '\u{fe00}',  // variation selector 1
    '\u{e0020}', // tag space
    '\u{17b4}',  // Khmer inherent vowel
];

/// Every word that names a figure, with the given character hidden inside each of them.
fn hidden_cases(chars: &[char]) -> Vec<Case> {
    let mut cases = Vec::new();
    for (k, z) in chars.iter().enumerate() {
        let n = format!("U+{:04X}", *z as u32);
        cases.push(case(
            &format!("en number {n}"),
            true,
            &format!("{WHO} owes the shop tw{z}o thou{z}sand rupees."),
        ));
        cases.push(case(
            &format!("ur number {n}"),
            false,
            &format!("{WHO} کے ذمے د{z}و ہ{z}زار روپے باقی ہیں۔"),
        ));
        // The month and never-word shapes for the first few characters: the rule is the same.
        if k < 6 {
            cases.push(case(
                &format!("en month {n}"),
                true,
                &format!("{WHO} paid in Octo{z}ber: {{{{r2.balance}}}}."),
            ));
            cases.push(case(
                &format!("en never-word {n}"),
                true,
                &format!("The mo{z}del says {{{{r2.balance}}}}."),
            ));
            cases.push(case(
                &format!("ur month {n}"),
                false,
                &format!("{WHO} کے ذمے {{{{r2.balance}}}} باقی ہیں، اکت{z}وبر کے۔"),
            ));
        }
    }
    cases
}

#[test]
fn t5_a_number_word_with_a_joiner_or_a_word_joiner_inside_is_refused() {
    assert_refused(&hidden_cases(HIDDEN_HANDLED));
}

/// FINDING T5-1: a soft hyphen, a combining grapheme joiner, a direction mark, an invisible math
/// operator, a variation selector or a tag between the letters of "thousand", "October", "model"
/// or the Urdu "ہزار" hides the word from the number-word, month and never-use rules
/// (`check::padded` makes every character that is not a letter a space, so the word falls in two).
/// The Owner reads the word; the check does not. Digits are not affected.
#[test]
fn t5_1_a_word_with_an_invisible_character_inside_is_refused() {
    assert_refused(&hidden_cases(HIDDEN_BY_SPLIT));
}

/// Letters of another script that look like Latin or Arabic ones, with no real quantity word left
/// in the sentence, so each case stands alone.
#[test]
fn t5_a_number_word_written_with_look_alike_letters_is_refused() {
    let cases = vec![
        // Cyrillic о (U+043E) for the Latin o, Greek ο (U+03BF), fullwidth Latin, a combining mark,
        // mathematical italic: "two" and "thousand" each written without a plain Latin letter.
        case(
            "cyrillic o",
            true,
            &format!("{WHO} owes the shop tw\u{043e} th\u{043e}usand rupees."),
        ),
        case(
            "greek omicron",
            true,
            &format!("{WHO} owes the shop tw\u{03bf} th\u{03bf}usand rupees."),
        ),
        case(
            "fullwidth",
            true,
            &format!(
                "{WHO} owes the shop \u{ff54}\u{ff57}\u{ff4f} \u{ff54}\u{ff48}\u{ff4f}\u{ff55}\u{ff53}\u{ff41}\u{ff4e}\u{ff44} rupees."
            ),
        ),
        case(
            "combining acute",
            true,
            &format!("{WHO} owes the shop tw\u{0301}o th\u{0301}ousand rupees."),
        ),
        case(
            "mathematical italic",
            true,
            &format!(
                "{WHO} owes the shop \u{1d461}\u{1d464}\u{1d45c} \u{1d461}\u{210e}\u{1d45c}\u{1d462}\u{1d460}\u{1d44e}\u{1d45b}\u{1d451} rupees."
            ),
        ),
        // Urdu: the do-chashmi heh for the heh, a tatweel, the Arabic heh, vowel signs, the Arabic
        // kaf for the Urdu kaf.
        case(
            "do-chashmi heh",
            false,
            &format!("{WHO} کے ذمے دو ھزار روپے باقی ہیں۔"),
        ),
        case(
            "tatweel",
            false,
            &format!("{WHO} کے ذمے دو ہـزار روپے باقی ہیں۔"),
        ),
        case(
            "arabic heh",
            false,
            &format!("{WHO} کے ذمے دو هزار روپے باقی ہیں۔"),
        ),
        case(
            "vowel signs",
            false,
            &format!("{WHO} کے ذمے دَو ہَزار روپے باقی ہیں۔"),
        ),
        case(
            "lakh spelled with kaf",
            false,
            &format!("{WHO} کے ذمے دو لاكھ روپے باقی ہیں۔"),
        ),
        case(
            "crore with arabic kaf",
            false,
            &format!("{WHO} کے ذمے ایک كروڑ روپے باقی ہیں۔"),
        ),
    ];
    // Each case is refused by some rule (the Urdu answers by the number words after
    // normalising, the English ones by the digit-free rules or the script rule).
    let runs = run_cases(&cases);
    let missed = passed(&runs);
    // Cyrillic, Greek and mathematical letters in an English answer are not Arabic script and not
    // a listed word: they are reported, not asserted (see the report, T5-1: only the English
    // script rule would catch them).
    let hard: Vec<&String> = missed
        .iter()
        .filter(|m| {
            !m.starts_with("cyrillic")
                && !m.starts_with("greek")
                && !m.starts_with("mathematical")
                && !m.starts_with("fullwidth")
                && !m.starts_with("combining")
        })
        .collect();
    assert!(
        hard.is_empty(),
        "look-alike Urdu letters reached the screen: {hard:?}"
    );
    eprintln!("T5 look-alike Latin letters in an English answer that were shown: {missed:?}");
}

/// FINDING T5-1 (second shape): letters of another script written for Latin ones also hide
/// "two thousand" in an English answer. The English script rule refuses only Arabic-script
/// letters, so Cyrillic, Greek and mathematical letters pass as words of their own.
#[test]
fn t5_1_look_alike_latin_letters_in_an_english_answer_are_refused() {
    let cases = vec![
        case(
            "cyrillic o",
            true,
            &format!("{WHO} owes the shop tw\u{043e} th\u{043e}usand rupees."),
        ),
        case(
            "greek omicron",
            true,
            &format!("{WHO} owes the shop tw\u{03bf} th\u{03bf}usand rupees."),
        ),
        case(
            "fullwidth",
            true,
            &format!(
                "{WHO} owes the shop \u{ff54}\u{ff57}\u{ff4f} \u{ff54}\u{ff48}\u{ff4f}\u{ff55}\u{ff53}\u{ff41}\u{ff4e}\u{ff44} rupees."
            ),
        ),
        case(
            "combining acute",
            true,
            &format!("{WHO} owes the shop tw\u{0301}o th\u{0301}ousand rupees."),
        ),
    ];
    assert_refused(&cases);
}

// ---- the check mark in other shapes -------------------------------------------------------------

/// Only the app puts the check mark on a figure (PRD FR-7.76): a typed one makes an unchecked
/// figure look checked. The rule names ✓ ✔ ☑.
#[test]
fn t5_the_three_check_marks_with_or_without_a_variation_selector_are_refused() {
    let marks = [
        ("check mark", "\u{2713}"),
        ("check mark with variation selector", "\u{2713}\u{fe0f}"),
        ("heavy check mark", "\u{2714}"),
        (
            "heavy check mark with variation selector",
            "\u{2714}\u{fe0f}",
        ),
        ("ballot box with check", "\u{2611}"),
        ("ballot box with check, emoji", "\u{2611}\u{fe0f}"),
        ("check mark in square brackets", "[\u{2713}]"),
    ];
    let cases: Vec<Case> = marks
        .iter()
        .map(|(name, m)| {
            case(
                name,
                false,
                &format!("{WHO} کے ذمے {{{{r2.balance}}}} {m} باقی ہیں۔"),
            )
        })
        .collect();
    assert_refused(&cases);
}

/// FINDING T5-4: the white heavy check mark ✅ (the one a language model writes most), the light
/// check mark, the ballot box with a bold check, the square root sign and the squared OK are shapes
/// of "checked" that pass the rule. Beside an unchecked figure they show the Owner a trust mark
/// the app never gave.
#[test]
fn t5_4_other_check_mark_shapes_are_refused() {
    let marks = [
        ("white heavy check mark", "\u{2705}"),
        ("light check mark", "\u{1f5f8}"),
        ("ballot box with bold check", "\u{1f5f9}"),
        ("square root", "\u{221a}"),
        ("OK button", "\u{1f197}"),
    ];
    let cases: Vec<Case> = marks
        .iter()
        .map(|(name, m)| {
            case(
                name,
                false,
                &format!("{WHO} کے ذمے {{{{r2.balance}}}} {m} باقی ہیں۔"),
            )
        })
        .collect();
    assert_refused(&cases);
}

// ---- month names -------------------------------------------------------------------------------

#[test]
fn t5_the_month_names_in_both_languages_are_refused() {
    let en = [
        "January",
        "Jan",
        "February",
        "Feb",
        "March",
        "Mar",
        "April",
        "Apr",
        "May",
        "June",
        "Jun",
        "July",
        "Jul",
        "August",
        "Aug",
        "September",
        "Sept",
        "Sep",
        "October",
        "Oct",
        "November",
        "Nov",
        "December",
        "Dec",
    ];
    let ur = [
        "جنوری",
        "فروری",
        "مارچ",
        "اپریل",
        "مئی",
        "جون",
        "جولائی",
        "اگست",
        "ستمبر",
        "اکتوبر",
        "نومبر",
        "دسمبر",
    ];
    let mut cases: Vec<Case> = en
        .iter()
        .map(|m| {
            case(
                &format!("en {m}"),
                true,
                &format!("{WHO} paid in {m}: {{{{r2.balance}}}}."),
            )
        })
        .collect();
    cases.extend(ur.iter().map(|m| {
        case(
            &format!("ur {m}"),
            false,
            &format!("{WHO} کے ذمے {{{{r2.balance}}}} باقی ہیں، {m} کے۔"),
        )
    }));
    assert_refused(&cases);
}

/// A second spelling of a month that Urdu readers use is not on the list.
#[test]
fn t5_9_the_other_spellings_of_a_month_are_refused() {
    let ur = ["اپرئیل", "اکتبر"];
    let cases: Vec<Case> = ur
        .iter()
        .map(|m| {
            case(
                &format!("ur {m}"),
                false,
                &format!("{WHO} کے ذمے {{{{r2.balance}}}} باقی ہیں، {m} کے۔"),
            )
        })
        .collect();
    assert_refused(&cases);
}

// ---- figures in the other fields of the answer --------------------------------------------------

/// A final answer with the given fields, over the Akram transcript. Gives the reply.
fn field_case(h: &Harness, content: &str) -> AskReply {
    let (reply, _) = ask(
        h,
        "ikram ke total bakaya kitne hai",
        None,
        akram_then(vec![final_reply(content), final_reply(content)]),
    );
    reply
}

/// The fields `choose`, `entry.party`, `nearest.source`, `nearest.party` and `show.result` are
/// strings the model writes. None of them is shown; each is read as a path or a source id. A
/// figure written there is never drawn.
#[test]
fn t5_a_figure_written_in_choose_entry_nearest_or_show_is_never_drawn() {
    let h = Harness::day();
    let cases: Vec<(&str, String)> = vec![
        (
            "entry party",
            answer_full(
                "change_request",
                "",
                json!([]),
                json!([]),
                Value::Null,
                json!({"screen": "wasooli", "party": "r1.matches.0 99,999"}),
            ),
        ),
        (
            "entry party is a figure",
            answer_full(
                "change_request",
                "",
                json!([]),
                json!([]),
                Value::Null,
                json!({"screen": "wasooli", "party": "99999"}),
            ),
        ),
        (
            "nearest source",
            answer_nearest(
                "outside_books",
                "",
                json!({"source": "R13 owes 99,999", "party": "r1.matches.0"}),
            ),
        ),
        (
            "nearest party",
            answer_nearest(
                "outside_books",
                "",
                json!({"source": "party_card", "party": "99,999"}),
            ),
        ),
        (
            "choose",
            answer_full(
                "which_one",
                "",
                json!([]),
                json!([]),
                json!("r1 99,999"),
                Value::Null,
            ),
        ),
        (
            "show result",
            answer_full(
                "answer",
                &format!("{WHO} کے ذمے {{{{r2.balance}}}} باقی ہیں۔"),
                json!([{"result": "r2 99,999", "as": "table", "rows": 7}]),
                json!([]),
                Value::Null,
                Value::Null,
            ),
        ),
    ];
    for (name, content) in cases {
        let reply = field_case(&h, &content);
        let text = serde_json::to_string(&reply).unwrap();
        assert!(
            !text.contains("99,999") && !text.contains("99999"),
            "{name}: a figure of the model's field is in the reply"
        );
    }
    h.scan_requests();
}

/// The model's text of the kinds that have no words of the model (a question it did not
/// understand, one outside the books ...) is never shown, whatever it holds.
#[test]
fn t5_text_in_a_kind_that_has_no_words_of_the_model_is_never_shown() {
    let h = Harness::day();
    for kind in [
        "not_understood",
        "outside_books",
        "need_dates",
        "need_party",
        "which_one",
        "not_found",
        "change_request",
    ] {
        let content = answer_full(
            kind,
            "Akram owes 99,999 rupees, ✓ checked.",
            json!([]),
            json!([]),
            Value::Null,
            Value::Null,
        );
        let reply = field_case(&h, &content);
        let text = serde_json::to_string(&reply).unwrap();
        assert!(
            !text.contains("99,999") && !text.contains("99999"),
            "{kind}: the model's text was shown: {}",
            text.chars().take(300).collect::<String>()
        );
    }
    h.scan_requests();
}

/// `show.rows` at its edges: zero, negative, the largest integer, a float.
#[test]
fn t5_show_rows_at_its_edges_never_draws_more_than_seven_rows() {
    let h = Harness::day();
    let read = tool_reply(&[("report", report_args("R18", json!({"rows": 100})))]);
    for rows in [
        json!(0),
        json!(-1),
        json!(i64::MAX),
        json!(1_000_000),
        json!(7.5),
        json!("7"),
    ] {
        let text = "{{r1.count}} کھاتے ہیں۔";
        let content = answer_full(
            "answer",
            text,
            json!([{"result": "r1", "as": "table", "rows": rows}]),
            json!([]),
            Value::Null,
            Value::Null,
        );
        let (reply, _) = ask(
            &h,
            "trial balance dikhao",
            None,
            vec![read.clone(), final_reply(&content), final_reply(&content)],
        );
        if let Some(shown) = reply.answer.as_ref() {
            for table in &shown.tables {
                assert!(table.rows.len() <= 7, "{rows}: {} rows", table.rows.len());
            }
        }
        assert!(
            matches!(
                reply.outcome,
                AskOutcome::ChecksFailed | AskOutcome::NotUnderstood
            ),
            "{rows}: {:?}",
            reply.outcome
        );
    }
}
