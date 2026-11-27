//! The answer text as the panel would write it, in plain characters, shared by the golden tests
//! that compare a filled answer with its expected text.
#![allow(dead_code)]

use mandi_assistant::answer::{Answer, AnswerPart};
use mandi_assistant::results::Words;
use mandi_reports::reports::Cell;
use mandi_services::core::accounts::BalanceSide;
use mandi_services::core::auth::Language;

fn thousands(n: i64) -> String {
    let digits = n.abs().to_string();
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    if n < 0 { format!("-{out}") } else { out }
}

fn month_name(language: Language, m: i64) -> &'static str {
    const UR: [&str; 12] = [
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
    const EN: [&str; 12] = [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ];
    let names = if language == Language::Ur { UR } else { EN };
    names
        .get(usize::try_from(m - 1).unwrap_or(0))
        .copied()
        .unwrap_or("?")
}

/// The answer text as the panel would write it, in plain characters, for comparing.
pub fn plain_text(answer: &Answer, language: Language, words: &Words) -> String {
    let mut out = String::new();
    for part in &answer.parts {
        match part {
            AnswerPart::Text { text } => out.push_str(text),
            AnswerPart::Figure {
                cell, dir, checked, ..
            } => {
                match cell {
                    Cell::Money { paisa } => out.push_str(&thousands(paisa.rupees())),
                    Cell::Balance { paisa, side } => {
                        out.push_str(&thousands(paisa.rupees()));
                        match side {
                            BalanceSide::Dr => out.push_str(" بنام"),
                            BalanceSide::Cr => out.push_str(" جمع"),
                            BalanceSide::Zero => {}
                        }
                    }
                    Cell::Percent { hundredths } => {
                        out.push_str(&format!("{}.{:02}%", hundredths / 100, hundredths % 100));
                    }
                    Cell::Number { value } => out.push_str(&thousands(*value)),
                    other => panic!("a figure of this kind: {other:?}"),
                }
                if let Some(dir) = dir {
                    out.push_str(match (dir, language) {
                        (mandi_assistant::answer::FigureDirection::Up, Language::Ur) => " زیادہ",
                        (mandi_assistant::answer::FigureDirection::Down, Language::Ur) => " کم",
                        (mandi_assistant::answer::FigureDirection::Same, Language::Ur) => " برابر",
                        (mandi_assistant::answer::FigureDirection::Up, Language::En) => " more",
                        (mandi_assistant::answer::FigureDirection::Down, Language::En) => " less",
                        (mandi_assistant::answer::FigureDirection::Same, Language::En) => {
                            " the same"
                        }
                    });
                }
                if *checked {
                    out.push_str(" ✓");
                }
            }
            AnswerPart::Name { name } => {
                let base = if language == Language::Ur {
                    &name.name_ur
                } else {
                    &name.name_en
                };
                out.push_str(base);
                if let Some(tag) = &name.tag {
                    out.push(' ');
                    out.push_str(tag);
                }
            }
            AnswerPart::Date { iso } => {
                let (y, rest) = iso.split_at(4);
                out.push_str(&format!("{}-{}-{}", &rest[4..6], &rest[1..3], &y[2..]));
            }
            AnswerPart::Month { year, month } => {
                out.push_str(&format!("{} {year}", month_name(language, *month)));
            }
            AnswerPart::Doc { type_id, number } => {
                let label = words.text(language, type_id);
                match number {
                    Some(n) => out.push_str(&format!("{label} {n}")),
                    None => out.push_str(&label),
                }
            }
        }
    }
    out
}
