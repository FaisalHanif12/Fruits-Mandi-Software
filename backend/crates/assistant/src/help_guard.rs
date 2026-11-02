//! Help is never a change request (plan 52 item 4, PRD FR-7.79, FR-7.89, E98).
//!
//! The model sometimes ends a "how do I ..." question as a change request: "bikri kaise add karun"
//! has the word "add" and the screen بکری, which is the shape of "add a sale". The live run of
//! 02-11-2026 scored 4 such questions as "I cannot change anything" (Q102 to Q104 and Q110), though
//! the model had read the right topic of the guide.
//!
//! This guard is the app's side of the rule. A question that asks HOW to do something, WHERE to see
//! something or WHAT something is, and carries no number to write, is help. When the model ends
//! such a question as a change request, `run.rs` answers it with the guide of the topic instead:
//! the guide the model read, or the one of the screen the model named. A real change request names
//! an amount ("akram ki 5000 wasooli likh do"), so it still gets the change refused answer.
//!
//! Example: "bikri kaise add karun" is a how-to question with no number: help. "akram ki 5000
//! wasooli kaise likhun" has a number: it stays a change request.

use mandi_services::core::text;
use mandi_services::domain::figures::QuestionNumber;

use crate::model_answer::EntryScreenName;

/// Words that ask how, where or what, in Roman Urdu, Urdu letters and English. Each is one word of
/// the normalised question.
const HOW_WORDS: &[&str] = &[
    // Roman Urdu: how.
    "kaise",
    "kese",
    "kaisay",
    "kaisey",
    "kesay",
    "kesey",
    "tareeka",
    "tareeqa",
    "tareeqay",
    "tareeky",
    "tareeke",
    "tareqa",
    "tarika",
    "tariqa",
    "tarike",
    // Roman Urdu: where.
    "kahan",
    "kahaan",
    "kidhar",
    "kidher",
    // Roman Urdu: what it means.
    "matlab",
    "samjhao",
    "samjhaiye",
    // Urdu letters.
    "کیسے",
    "طریقہ",
    "طریقے",
    "کہاں",
    "مطلب",
    "سمجھائیں",
    "سمجھاؤ",
    // English.
    "how",
    "where",
    "explain",
    "meaning",
];

/// Pairs of words that ask what something is.
const WHAT_PAIRS: &[(&str, &str)] = &[
    ("kya", "hai"),
    ("kya", "hota"),
    ("kya", "hoti"),
    ("kya", "hote"),
    ("kya", "hain"),
    ("what", "is"),
    ("what", "are"),
    ("what", "does"),
    ("کیا", "ہے"),
    ("کیا", "ہوتا"),
    ("کیا", "ہوتی"),
    ("کیا", "ہوتے"),
    ("کیا", "ہیں"),
    ("کس", "طرح"),
];

/// The words of a question: lower case, the letter variants of Urdu made one, everything that is not
/// a letter or a digit taken as a space.
fn words_of(question: &str) -> Vec<String> {
    let normal = text::search_ur(&text::search_en(question));
    normal
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(str::to_owned)
        .collect()
}

/// True when the question asks how, where or what and holds no number to write. "How much" and
/// "how many" ask for a figure, so they are not help.
pub fn is_how_to(question: &str, numbers: &[QuestionNumber]) -> bool {
    if !numbers.is_empty() {
        return false;
    }
    let words = words_of(question);
    let asks_figure = words.windows(2).any(|pair| match pair {
        [first, second] => {
            first == "how" && matches!(second.as_str(), "much" | "many" | "long" | "old")
        }
        _ => false,
    });
    if asks_figure {
        return false;
    }
    let normal = |w: &str| text::search_ur(&text::search_en(w));
    if words
        .iter()
        .any(|w| HOW_WORDS.iter().any(|h| normal(h) == *w))
    {
        return true;
    }
    words.windows(2).any(|pair| match pair {
        [first, second] => WHAT_PAIRS
            .iter()
            .any(|(a, b)| normal(a) == *first && normal(b) == *second),
        _ => false,
    })
}

/// The topic of the guide for a screen of the answer schema (`prompt/guide.json`). A test keeps
/// every name here equal to a real topic.
pub fn topic_of(screen: EntryScreenName) -> &'static str {
    match screen {
        EntryScreenName::Bikri => "bikri_book",
        EntryScreenName::Wasooli => "receipt",
        EntryScreenName::Adaigi => "payment",
        EntryScreenName::Peshgi => "advance",
        EntryScreenName::Kharcha => "shop_expense",
        EntryScreenName::UdharRaqam => "money_lent",
        EntryScreenName::AlagKhata => "separate_khata",
        EntryScreenName::Sarmaya => "partner_capital",
        EntryScreenName::Phal => "fruit_taken",
        EntryScreenName::General => "general_journal",
        EntryScreenName::Account | EntryScreenName::Accounts => "accounts",
        EntryScreenName::Dashboard => "dashboard",
        EntryScreenName::Settings => "settings",
        EntryScreenName::Masters => "masters",
        EntryScreenName::CommissionRules => "commission_rules",
        EntryScreenName::TodaySales => "today_sales",
        EntryScreenName::UnsoldStock => "unsold_stock",
        EntryScreenName::PukhtaKhata => "pukhta_khata",
        EntryScreenName::CollectionList => "collection_list",
        EntryScreenName::CashBook => "cash_book",
        EntryScreenName::Gardens => "gardens",
        EntryScreenName::MarketTrips => "market_trips",
        EntryScreenName::Partners => "partners",
        EntryScreenName::MonthClose => "month_close",
        EntryScreenName::DayClose => "day_close",
        EntryScreenName::Reports => "reports",
        EntryScreenName::DeadAmount => "dead_amount",
        EntryScreenName::Analytics => "analytics",
        EntryScreenName::Backup => "backup",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prompt::guide;
    use mandi_services::domain::figures::question_numbers;

    fn help(q: &str) -> bool {
        is_how_to(q, &question_numbers(q))
    }

    /// The four misses of the live run of 02-11-2026 and the help questions that passed.
    #[test]
    fn how_where_and_what_questions_with_no_number_are_help() {
        for q in [
            "sale entry kese kartay hain",
            "بکری کیسے شامل کروں؟",
            "how do I add a sale?",
            "where do I see a partner's account?",
            "bikri kaise add karun",
            "wasooli kaise likhun",
            "wasuli kese likhu",
            "وصولی کیسے لکھوں؟",
            "how do I write a receipt?",
            "مارکیٹ سفر کیا ہوتا ہے؟",
            "market safar kya hai",
            "what is a pukhta khata",
            "partner ka hisaab kahan dekhun",
            "bikri ka tareeqa batao",
        ] {
            assert!(help(q), "{q}");
        }
    }

    #[test]
    fn a_question_with_a_number_to_write_or_a_figure_to_read_is_not_help() {
        for q in [
            "akram ki 5000 wasooli kaise likhun",
            "akram ki 5 hazar wasooli likh do",
            "how much does akram owe",
            "how many sales today",
            "akram ka rate barha do",
            "delete akram",
            "save karo",
            "ikram ke total bakaya kitne hai",
        ] {
            assert!(!help(q), "{q}");
        }
    }

    #[test]
    fn every_screen_has_a_topic_that_exists_in_the_guide() {
        let topics = &guide().topics;
        for name in EntryScreenName::ALL {
            let screen: EntryScreenName = serde_json::from_value(serde_json::json!(name)).unwrap();
            let topic = topic_of(screen);
            assert!(topics.contains_key(topic), "{name} -> {topic}");
        }
    }
}
