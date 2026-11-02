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

/// The words that point to one topic of the guide: whole words or word pairs of the normalised
/// question, in Roman Urdu, Urdu letters and English. Used only to read the guide for the model
/// before its first call (plan 52 item 7); the model may still ask for another topic.
const TOPIC_CUES: &[(&str, &[&str])] = &[
    (
        "bikri_book",
        &[
            "bikri",
            "بکری",
            "sale",
            "sales",
            "bakri",
            "sale entry",
            "sale book",
        ],
    ),
    (
        "receipt",
        &[
            "wasooli",
            "wasuli",
            "وصولی",
            "receipt",
            "receipts",
            "ugrahi",
        ],
    ),
    ("payment", &["adaigi", "ادائیگی", "payment", "payments"]),
    ("advance", &["peshgi", "پیشگی", "advance"]),
    (
        "shop_expense",
        &["kharcha", "kharch", "خرچہ", "expense", "expenses"],
    ),
    (
        "personal_expense",
        &["zati", "ذاتی", "personal", "drawing", "drawings"],
    ),
    (
        "money_lent",
        &["udhar raqam", "ادھار رقم", "money lent", "loan"],
    ),
    ("partner_capital", &["sarmaya", "سرمایہ", "capital"]),
    (
        "partners",
        &[
            "partner",
            "partners",
            "shirakat",
            "شراکت دار",
            "hissa",
            "حصہ",
        ],
    ),
    ("market_trips", &["safar", "سفر", "trip", "trips"]),
    ("gardens", &["bagh", "باغ", "garden", "gardens"]),
    ("fruit_taken", &["phal", "پھل", "fruit taken"]),
    ("general_journal", &["journal", "جنرل", "general entry"]),
    (
        "commission_rules",
        &[
            "commission rule",
            "commission rules",
            "commission usool",
            "کمیشن اصول",
        ],
    ),
    ("reports", &["report", "reports", "رپورٹ", "رپورٹس"]),
    ("analytics", &["analytics", "tajziyat", "تجزیات"]),
    ("backup", &["backup", "بیک اپ"]),
    ("day_close", &["day close", "din band", "دن بند"]),
    ("month_close", &["month close", "mahina band", "مہینہ بند"]),
    ("dashboard", &["dashboard", "ڈیش بورڈ"]),
    ("settings", &["settings", "tarteebat", "ترتیبات"]),
    ("pukhta_khata", &["pukhta", "pakka khata", "پختہ"]),
    ("cash_book", &["rokar", "روکڑ", "cash book"]),
    ("dead_amount", &["doobi", "dobi", "ڈوبی", "dead amount"]),
    (
        "later_sale",
        &["later sale", "baad mein bikri", "بعد میں بکری"],
    ),
    ("edit_and_cancel", &["edit", "cancel", "mansookh", "منسوخ"]),
    ("unsold_stock", &["baqi maal", "باقی مال", "unsold stock"]),
    ("keys", &["shortcut", "shortcuts", "keyboard", "keys"]),
    ("language_switch", &["language", "zaban", "زبان"]),
    ("search", &["search", "talash", "تلاش"]),
    ("sign_in", &["login", "sign in", "password", "پاس ورڈ"]),
    ("printing", &["print", "printing", "پرنٹ"]),
    ("separate_khata", &["alag khata", "الگ کھاتہ"]),
    (
        "collection_list",
        &["collection list", "ugrahi list", "اگراہی فہرست"],
    ),
];

/// The one topic of the guide a how-to question is about, or `None` when it is not a how-to
/// question or names no topic or more than one. The answer to "bikri kaise add karun" is
/// `bikri_book`. A wrong guess only costs the guide's words in the request: the model reads the
/// topic it was given and may ask for another.
pub fn topic_of_question(question: &str, numbers: &[QuestionNumber]) -> Option<&'static str> {
    if !is_how_to(question, numbers) {
        return None;
    }
    let words = words_of(question);
    let joined = format!(" {} ", words.join(" "));
    let mut found: Vec<&'static str> = Vec::new();
    for (topic, cues) in TOPIC_CUES {
        let hit = cues.iter().any(|cue| {
            let key = text::search_ur(&text::search_en(cue));
            joined.contains(&format!(" {key} "))
        });
        if hit {
            found.push(topic);
        }
    }
    match found.as_slice() {
        [one] => Some(one),
        _ => None,
    }
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
    fn a_how_to_question_names_its_topic_when_exactly_one_fits() {
        let topic = |q: &str| topic_of_question(q, &question_numbers(q));
        assert_eq!(topic("bikri kaise add karun"), Some("bikri_book"));
        assert_eq!(topic("how do I add a sale?"), Some("bikri_book"));
        assert_eq!(topic("wasooli kaise likhun"), Some("receipt"));
        assert_eq!(topic("وصولی کیسے لکھوں؟"), Some("receipt"));
        assert_eq!(topic("market safar kya hai"), Some("market_trips"));
        assert_eq!(topic("backup kaise lun"), Some("backup"));
        assert_eq!(
            topic("where do I see a partner's account?"),
            Some("partners")
        );
        // More than one topic, no topic, or not a how-to question: none.
        assert_eq!(topic("bikri aur wasooli kaise karun"), None);
        assert_eq!(topic("ye kaise karun"), None);
        assert_eq!(topic("akram ki wasooli kitni hai"), None);
        assert_eq!(topic("wasooli 5000 kaise likhun"), None);
    }

    #[test]
    fn every_topic_cue_names_a_real_topic_of_the_guide() {
        let topics = &guide().topics;
        for (topic, cues) in TOPIC_CUES {
            assert!(topics.contains_key(*topic), "{topic}");
            assert!(!cues.is_empty());
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
