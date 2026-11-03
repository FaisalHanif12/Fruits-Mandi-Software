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
    // What it shows, when it is used, what can be asked or seen, what to do (the live run of
    // 03-11-2026: these were help questions that the pre-read of the guide missed).
    ("kya", "dikhata"),
    ("kya", "dikhati"),
    ("kya", "dikhate"),
    ("kya", "dikhta"),
    ("kya", "milta"),
    ("kya", "milti"),
    ("kab", "use"),
    ("kab", "istemal"),
    ("dekh", "sakte"),
    ("dekh", "sakta"),
    ("what", "can"),
    ("what", "do"),
    ("which", "keyboard"),
    ("which", "shortcuts"),
    ("کیا", "دکھاتی"),
    ("کیا", "دکھاتا"),
    ("کیا", "ملتا"),
    ("کیا", "ملتی"),
    ("کب", "استعمال"),
    ("کیا", "پوچھ"),
];

/// Words of a question that asks for advice, not for the guide: "sale kaise barhayen" is about
/// selling more, not about the sale screen (the live run: Q247 was answered with the guide of the
/// sale book). A question with one of these gets no pre-read of the guide.
const ADVICE_WORDS: &[&str] = &[
    "barhayen",
    "barhana",
    "barhaon",
    "barhao",
    "barhane",
    "bherane",
    "badhayen",
    "badhana",
    "badhao",
    "بڑھائیں",
    "بڑھانے",
    "بڑھاؤں",
    "بڑھاو",
    "increase",
    "raise",
    "improve",
    "grow",
    "faster",
    "jaldi",
    "جلدی",
    "behtar",
    "بہتر",
    "reduce",
];

/// Words a general mandi advice question is made of ("wasooli jaldi kaise karen", "udhar kaise kam
/// karun"): what is improved, the way, and the small words around them.
const GENERAL_ADVICE_VOCABULARY: &[&str] = &[
    // what is improved
    "wasooli",
    "wasuli",
    "ugrahi",
    "recovery",
    "collect",
    "collection",
    "udhar",
    "udhaar",
    "credit",
    "dues",
    "bikri",
    "sale",
    "sales",
    "munafa",
    "nafa",
    "profit",
    "commission",
    "kharcha",
    "kharche",
    "expenses",
    "cost",
    "business",
    "karobar",
    "dukan",
    "shop",
    "وصولی",
    "ادھار",
    "بکری",
    "منافع",
    "کمیشن",
    "خرچہ",
    "کاروبار",
    "دکان",
    // how
    "kaise",
    "kese",
    "kaisay",
    "kis",
    "tarah",
    "tarike",
    "tareeqa",
    "tarika",
    "how",
    "can",
    "to",
    "tips",
    "tip",
    "mashwara",
    "advice",
    "کیسے",
    "طریقہ",
    "مشورہ",
    "کس",
    "طرح",
    // the small words
    "jaldi",
    "tez",
    "fast",
    "faster",
    "quickly",
    "behtar",
    "better",
    "zyada",
    "kam",
    "more",
    "less",
    "karen",
    "karun",
    "kare",
    "karein",
    "karna",
    "kar",
    "sakte",
    "sakta",
    "sakti",
    "hain",
    "hai",
    "ho",
    "mein",
    "main",
    "me",
    "ko",
    "ka",
    "ki",
    "ke",
    "apni",
    "apna",
    "apne",
    "hum",
    "ham",
    "meri",
    "mera",
    "hamari",
    "hamara",
    "aur",
    "se",
    "i",
    "we",
    "do",
    "my",
    "our",
    "the",
    "of",
    "in",
    "get",
    "اپنی",
    "اپنا",
    "ہم",
    "میں",
    "کو",
    "کا",
    "کی",
    "کے",
    "اور",
    "سے",
    "کریں",
    "کروں",
    "جلدی",
    "بہتر",
    "زیادہ",
    "کم",
];

/// True when the question is a general mandi advice question: how to do better at something the
/// shop does (collect dues faster, sell more, lower credit), in the vocabulary above, with no name
/// of a party or an item and no number. The books have nothing to say to it: the model answers it
/// with no tool, in one call, as general advice (PRD FR-7.88; Q250 and Q251 of the live run 2 read
/// the dues list and answered with ordinary advice instead).
///
/// Example: "wasooli jaldi kaise karen" and "وصولی جلدی کیسے کریں" are general advice;
/// "akram se wasooli kaise karen" names a party and "wasooli kitni hui" asks for a figure.
pub fn general_advice_asked(question: &str, numbers: &[QuestionNumber]) -> bool {
    if !numbers.is_empty() || !is_how_to(question, numbers) {
        return false;
    }
    let normal = |w: &str| text::search_ur(&text::search_en(w));
    let vocabulary: Vec<String> = GENERAL_ADVICE_VOCABULARY
        .iter()
        .chain(ADVICE_WORDS.iter())
        .map(|w| normal(w))
        .collect();
    let words = words_of(question);
    if words.is_empty() || words.len() > 8 || words.iter().any(|w| w.chars().any(char::is_numeric))
    {
        return false;
    }
    // Every word is of the vocabulary: a name, an item or any other word is a question about the
    // books.
    words.iter().all(|w| vocabulary.contains(w))
}

/// Phrases that make a cue mean something else ("the capital of Peru" is not the partners'
/// capital).
const NOT_THE_TOPIC: &[&str] = &["capital of", "capital city"];

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
    // "What is ..." and its kin are weak: "what is each partner's total now" asks for a figure.
    // A word that asks for one (total, now, each, owe, kitna ...) makes it a data question.
    if words
        .iter()
        .any(|w| DATA_WORDS.iter().any(|d| normal(d) == *w))
    {
        return false;
    }
    words.windows(2).any(|pair| match pair {
        [first, second] => WHAT_PAIRS
            .iter()
            .any(|(a, b)| normal(a) == *first && normal(b) == *second),
        _ => false,
    })
}

/// Words that ask for a figure of the books, which turn a "what is ..." question into data.
const DATA_WORDS: &[&str] = &[
    "total", "now", "each", "owe", "owes", "balance", "today", "kitna", "kitni", "kitne", "abhi",
    "کتنا", "کتنی", "کتنے", "ابھی",
];

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
        &[
            "udhar raqam",
            "udhar rakam",
            "ادھار رقم",
            "money lent",
            "loan",
            "staff",
            "staf",
            "عملہ",
        ],
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
    ("gardens", &["bagh", "bag", "باغ", "garden", "gardens"]),
    ("fruit_taken", &["phal", "پھل", "fruit taken"]),
    ("general_journal", &["journal", "جنرل", "general entry"]),
    (
        "commission_rules",
        &[
            "commission rule",
            "commission rules",
            "commission usool",
            "comision rule",
            "کمیشن اصول",
        ],
    ),
    ("reports", &["report", "reports", "رپورٹ", "رپورٹس"]),
    ("analytics", &["analytics", "tajziyat", "تجزیات"]),
    ("backup", &["backup", "بیک اپ"]),
    (
        "day_close",
        &["day close", "close the day", "din band", "دن بند"],
    ),
    ("month_close", &["month close", "mahina band", "مہینہ بند"]),
    ("dashboard", &["dashboard", "ڈیش بورڈ"]),
    ("settings", &["settings", "tarteebat", "ترتیبات"]),
    ("pukhta_khata", &["pukhta", "pakka khata", "پختہ"]),
    ("cash_book", &["rokar", "روکڑ", "cash book"]),
    ("dead_amount", &["doobi", "dobi", "ڈوبی", "dead amount"]),
    (
        "later_sale",
        &[
            "later sale",
            "baad mein bikri",
            "left over",
            "leftover",
            "earlier day",
            "pichle din",
            "بعد میں بکری",
            "پچھلے دن",
        ],
    ),
    (
        "edit_and_cancel",
        &["edit", "cancel", "delete", "mansookh", "منسوخ"],
    ),
    ("unsold_stock", &["baqi maal", "باقی مال", "unsold stock"]),
    (
        "keys",
        &["shortcut", "shortcuts", "keyboard", "keys", "کی بورڈ"],
    ),
    (
        "language_switch",
        &[
            "language",
            "zaban",
            "english mein",
            "urdu mein",
            "زبان",
            "انگریزی میں",
        ],
    ),
    ("search", &["search", "find a party", "talash", "تلاش"]),
    ("sign_in", &["login", "sign in", "password", "پاس ورڈ"]),
    ("printing", &["print", "printing", "پرنٹ"]),
    (
        "separate_khata",
        &["alag khata", "separate khata", "الگ کھاتہ"],
    ),
    (
        "collection_list",
        &[
            "collection list",
            "ugrahi list",
            "ugrai",
            "اگراہی فہرست",
            "اگرائی",
        ],
    ),
    (
        "accounts",
        &[
            "new account",
            "naya account",
            "new buyer",
            "new seller",
            "naya gahak",
            "naya byopari",
            "account banate",
            "account banana",
            "نیا اکاؤنٹ",
            "نیا گاہک",
            "نیا بیوپاری",
        ],
    ),
    (
        "masters",
        &[
            "masters",
            "new item",
            "new city",
            "naya item",
            "naya shehar",
            "item list",
            "city list",
            "نیا آئٹم",
            "نیا شہر",
            "انڈکس",
        ],
    ),
    (
        "today_sales",
        &["aaj ki bikri", "todays sales", "today sales", "آج کی بکری"],
    ),
    (
        "ask_the_app",
        &[
            "sawal wale",
            "ask box",
            "ask the app",
            "سوال والے",
            "سوال کا خانہ",
        ],
    ),
];

/// Topics that name a screen many questions mention in passing ("the sale", "the partner", "a
/// report"). When a question names one of them and a more specific topic too, the specific one is
/// the topic: "how do I edit or cancel a saved sale" is `edit_and_cancel`, not `bikri_book`.
const GENERIC_TOPICS: &[&str] = &[
    "bikri_book",
    "receipt",
    "payment",
    "advance",
    "shop_expense",
    "partners",
    "reports",
    "general_journal",
    "pukhta_khata",
    "cash_book",
    "dashboard",
    "settings",
    "accounts",
    "masters",
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
    let normal = |w: &str| text::search_ur(&text::search_en(w));
    // A question for advice is not a question about a screen.
    if words
        .iter()
        .any(|w| ADVICE_WORDS.iter().any(|a| normal(a) == *w))
    {
        return None;
    }
    let joined = format!(" {} ", words.join(" "));
    if NOT_THE_TOPIC
        .iter()
        .any(|phrase| joined.contains(&format!(" {} ", normal(phrase))))
    {
        return None;
    }
    let mut found: Vec<&'static str> = Vec::new();
    for (topic, cues) in TOPIC_CUES {
        let hit = cues.iter().any(|cue| {
            let key = normal(cue);
            joined.contains(&format!(" {key} "))
        });
        if hit {
            found.push(topic);
        }
    }
    // A specific topic wins over the generic screens named in passing.
    if found.len() > 1 && found.iter().any(|t| !GENERIC_TOPICS.contains(t)) {
        found.retain(|t| !GENERIC_TOPICS.contains(t));
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

    /// The 52 help questions of the live bank (plan 52 Section 2, category D) and the topic of the
    /// guide each one is about. The pre-read of the guide (item 7) knew 24 of them before the
    /// time analysis of 03-11-2026; the rest cost the model a call of its own to ask for the guide.
    #[test]
    fn the_help_questions_of_the_bank_name_their_topic() {
        let table: &[(&str, &str)] = &[
            ("bikri kaise add karun", "bikri_book"),
            ("sale entry kese kartay hain", "bikri_book"),
            ("بکری کیسے شامل کروں؟", "bikri_book"),
            ("how do I add a sale?", "bikri_book"),
            ("wasooli kaise likhun", "receipt"),
            ("wasuli kese likhu", "receipt"),
            ("وصولی کیسے لکھوں؟", "receipt"),
            ("how do I write a receipt?", "receipt"),
            ("مارکیٹ سفر کیا ہوتا ہے؟", "market_trips"),
            ("where do I see a partner's account?", "partners"),
            (
                "aaj ki bikri wali screen kahan hai aur usme kya dikhta hai",
                "today_sales",
            ),
            ("باقی مال کی اسکرین کیا دکھاتی ہے", "unsold_stock"),
            (
                "how do I sell the goods that were left over from an earlier day",
                "later_sale",
            ),
            ("ugrai ki list kese banti he", "collection_list"),
            ("byopari ko payment kaise karte hain", "payment"),
            ("بیوپاری کو پیشگی کیسے دیتے ہیں", "advance"),
            ("pukhta khata kya hota hai", "pukhta_khata"),
            ("how do I edit or cancel a saved sale", "edit_and_cancel"),
            (
                "bikri ko theek ya cancel kaise karte hain",
                "edit_and_cancel",
            ),
            ("محفوظ بکری کو درست یا منسوخ کیسے کریں", "edit_and_cancel"),
            ("روکڑ بک میں کیا ملتا ہے", "cash_book"),
            ("dukan ka kharcha kaise likhte hain", "shop_expense"),
            ("شراکت دار کا ذاتی خرچہ کیسے لکھتے ہیں", "personal_expense"),
            ("staf ko udhar rakam kese dete hen", "money_lent"),
            ("what is a separate khata", "separate_khata"),
            ("general journal kab use karte hain", "general_journal"),
            (
                "شراکت دار گھر کے لیے جو پھل لے جائے وہ کیسے لکھیں",
                "fruit_taken",
            ),
            ("bag ka hisab kese rakhte hen", "gardens"),
            (
                "how do I record a partner's capital investment",
                "partner_capital",
            ),
            ("analytics screen se kya kya dekh sakte hain", "analytics"),
            ("how do I get a report", "reports"),
            ("what does the dashboard show", "dashboard"),
            ("din band kaise karte hain", "day_close"),
            ("دن بند کیسے کرتے ہیں", "day_close"),
            ("how do I close the day", "day_close"),
            ("مہینہ بند کیسے کرتے ہیں", "month_close"),
            (
                "backup kaise lete hain aur wapas kaise lagate hain",
                "backup",
            ),
            ("where do I change the settings", "settings"),
            ("نیا آئٹم یا شہر کیسے بناتے ہیں", "masters"),
            (
                "naya gahak ya byopari ka account kaise banate hain",
                "accounts",
            ),
            ("how do I add a new buyer or seller account", "accounts"),
            ("how do I find a party quickly", "search"),
            (
                "kisi seller ya item ka comision rule kese lagate hen",
                "commission_rules",
            ),
            ("ڈوبی رقم کیا ہے اور کیسے لگاتے ہیں", "dead_amount"),
            ("english mein kaise badlun", "language_switch"),
            ("which keyboard shortcuts are there", "keys"),
            ("print kaise nikalte hain", "printing"),
            ("اس سوال والے خانے سے کیا پوچھ سکتا ہوں", "ask_the_app"),
            ("I forgot my password, what do I do", "sign_in"),
            ("bikri kaise delete karte hain", "edit_and_cancel"),
            ("مارکیٹ سفر میں خریداری کیسے لکھتے ہیں", "market_trips"),
            ("partner ka khata kahan dekhte hain", "partners"),
        ];
        let mut wrong = Vec::new();
        for (q, want) in table {
            let got = topic_of_question(q, &question_numbers(q));
            if got != Some(*want) {
                wrong.push(format!("{q}: wanted {want}, got {got:?}"));
            }
        }
        assert!(wrong.is_empty(), "{}", wrong.join("\n"));
    }

    /// Advice is not help: "how can I increase sales" is about selling more, not the sale screen.
    #[test]
    fn a_question_for_advice_or_about_something_else_names_no_topic() {
        let topic = |q: &str| topic_of_question(q, &question_numbers(q));
        for q in [
            "sale kaise barhayen",
            "بکری کیسے بڑھائیں",
            "how can I increase sales",
            "sale kese barhaon",
            "wasooli jaldi kaise karen",
            "وصولی جلدی کیسے کریں",
            "sales bherane ke tareeky batao?",
            "بکری بڑھانے کے طریقے بتائیں",
            "what is the capital of Peru",
        ] {
            assert_eq!(topic(q), None, "{q}");
        }
        // The guide still answers the same words when the question is about the screen.
        assert_eq!(topic("bikri kaise add karun"), Some("bikri_book"));
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
