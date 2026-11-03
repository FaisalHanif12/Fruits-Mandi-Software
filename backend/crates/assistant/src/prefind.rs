//! Finding the party a question names before the first model call (plan 52 P52-S1, item 10).
//!
//! A question about a party ("ikram ke total bakaya kitne hai") costs two model calls at the least:
//! one to ask `find`, one to answer. The app can do the find itself with the very search the
//! pickers use. It does so only when there is no doubt:
//!
//! - the question is a plain lookup: no number, not a how-to, not a follow-up, and no period, tile
//!   or guide was read for it;
//! - one word of the question finds exactly ONE account in the role the question implies, and the
//!   search includes the close spellings, so no other account is within the close-spelling
//!   distance of that word;
//! - no other word of the question finds a different single account (two parties: the model asks).
//!
//! Every doubtful case goes to the model's own `find` as before. When the app did find the party,
//! the answer always shows his name (`run.rs` adds it if the model left it out).
//!
//! Example: "kareem ko kitna dena he" implies a seller (dena); "kareem" finds one seller; the app
//! reads him and his card, and the model's first call can answer.

use mandi_services::core::text;

use crate::answer::{Answer, AnswerName, AnswerPart, AnswerSource};
use crate::help_guard;
use crate::model_answer::{ModelAnswer, Piece, pieces};
use crate::results::Node;
use crate::tools::cues;

/// Words that are not a party's name: the words of the questions this app is asked, in Roman
/// Urdu, Urdu letters and English, and every cue word of the app's own lists. A word of this list
/// is never looked up as a name.
const NOT_A_NAME: &[&str] = &[
    // Roman Urdu
    "ke",
    "ka",
    "ki",
    "ko",
    "se",
    "ne",
    "mein",
    "men",
    "me",
    "par",
    "pe",
    "hai",
    "he",
    "hain",
    "hen",
    "tha",
    "thi",
    "the",
    "hua",
    "hui",
    "hue",
    "ho",
    "hoga",
    "hogi",
    "kitna",
    "kitni",
    "kitne",
    "kya",
    "kaun",
    "kab",
    "kahan",
    "kese",
    "kaise",
    "aur",
    "ya",
    "bhi",
    "sab",
    "total",
    "kul",
    "abhi",
    "tak",
    "dena",
    "dene",
    "lena",
    "lene",
    "diya",
    "diye",
    "liya",
    "liye",
    "bakaya",
    "baqi",
    "baki",
    "raqam",
    "paisa",
    "paise",
    "wasooli",
    "ugrahi",
    "peshgi",
    "adaigi",
    "hisab",
    "hisaab",
    "khata",
    "khate",
    "bill",
    "bills",
    "bikri",
    "maal",
    "mal",
    "commission",
    "kharcha",
    "aaj",
    "kal",
    "uska",
    "unka",
    "iska",
    "inka",
    "mera",
    "meri",
    "mujhe",
    "batao",
    "bataiye",
    "dikhao",
    "dikhaen",
    "likho",
    "wala",
    "wali",
    "kuch",
    "koi",
    "zyada",
    "kam",
    "pehle",
    "baad",
    // English
    "the",
    "and",
    "for",
    "how",
    "much",
    "many",
    "what",
    "does",
    "did",
    "owe",
    "owes",
    "from",
    "with",
    "has",
    "have",
    "his",
    "her",
    "its",
    "our",
    "are",
    "was",
    "were",
    "who",
    "when",
    "where",
    "show",
    "tell",
    "give",
    "all",
    "any",
    "ledger",
    "statement",
    "account",
    "balance",
    "due",
    "dues",
    "paid",
    "pay",
    "bought",
    "buy",
    "sold",
    "sell",
    "sale",
    "sales",
    "cash",
    "total",
    "party",
    // Urdu letters
    "کا",
    "کی",
    "کے",
    "کو",
    "سے",
    "نے",
    "میں",
    "پر",
    "ہے",
    "ہیں",
    "تھا",
    "تھی",
    "کتنا",
    "کتنی",
    "کتنے",
    "کیا",
    "کون",
    "کب",
    "کہاں",
    "اور",
    "یا",
    "بھی",
    "سب",
    "کل",
    "ابھی",
    "تک",
    "دینا",
    "دینے",
    "لینا",
    "لینے",
    "دیا",
    "دیے",
    "بقایا",
    "باقی",
    "رقم",
    "پیسے",
    "وصولی",
    "اگراہی",
    "پیشگی",
    "ادائیگی",
    "حساب",
    "کھاتہ",
    "کھاتے",
    "بل",
    "بکری",
    "مال",
    "کمیشن",
    "خرچہ",
    "آج",
    "اس",
    "ان",
    "ہوا",
    "ہوئی",
];

/// The most words of a question that are looked up as names; more is not a plain lookup.
pub const MAX_WORDS: usize = 5;

/// The key of the account a find result holds, when it holds exactly one match, no more to show
/// not a spelling made by the rules alone, and not a match found by widening the role.
pub fn single_key(node: &Node) -> Option<String> {
    let Some(Node::List(matches)) = node.get(&["matches"]) else {
        return None;
    };
    let [only] = matches.as_slice() else {
        return None;
    };
    let more = matches!(node.get(&["more"]), Some(Node::Bool(true)));
    let by_rules = matches!(node.get(&["via"]), Some(Node::Str(s)) if s == "rules");
    // A match found only because the search was widened to another role is not in the role the
    // question implies: never taken.
    let widened = node.get(&["widened"]).is_some();
    match only.get(&["key"]) {
        Some(Node::Str(key)) if !more && !by_rules && !widened => Some(key.clone()),
        _ => None,
    }
}

/// Words that say the party is a buyer (he owes the shop, he bought) or a seller (the shop owes him,
/// he sent goods): the role of the search.
const BUYER_WORDS: &[&str] = &[
    "lena",
    "lene",
    "bakaya",
    "owe",
    "owes",
    "wasooli",
    "ugrahi",
    "bought",
    "khareeda",
    "kharida",
    "لینا",
    "لینے",
    "بقایا",
    "وصولی",
    "اگراہی",
    "خریدا",
];
const SELLER_WORDS: &[&str] = &[
    "dena",
    "dene",
    "payable",
    "adaigi",
    "peshgi",
    "advance",
    "دینا",
    "دینے",
    "ادائیگی",
    "پیشگی",
];

/// The role the question implies: a buyer, a seller, or either.
pub fn role_of(question: &str) -> &'static str {
    let words = words(question);
    let has = |list: &[&str]| words.iter().any(|w| list.contains(&w.as_str()));
    match (has(BUYER_WORDS), has(SELLER_WORDS)) {
        (true, false) => "buyer",
        (false, true) => "seller",
        _ => "any_party",
    }
}

fn words(question: &str) -> Vec<String> {
    text::search_ur(&text::search_en(question))
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(str::to_owned)
        .collect()
}

/// The words of the question that may be a name: three letters at least, no digit, not a word of
/// the lists above or a cue word of the app, each once, in the order of the question.
pub fn name_words(question: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for w in words(question) {
        let known = NOT_A_NAME
            .iter()
            .any(|n| text::search_ur(&text::search_en(n)) == w);
        if w.chars().count() < 3 || w.chars().any(char::is_numeric) || known || out.contains(&w) {
            continue;
        }
        out.push(w);
    }
    out
}

/// True when the question is a plain lookup the app may find the party for: no number, not a
/// how-to, and nothing else is read for it first.
pub fn plain_lookup(question: &str, numbers_empty: bool) -> bool {
    numbers_empty
        && !question.chars().any(char::is_numeric)
        && !help_guard::is_how_to(question, &[])
        && cues::period_kind(question).is_none()
        && !cues::tile_question(question)
}

/// The results the model's answer cites: the first part of every reference in its text, and the
/// results it asks to show. `{{r2.balance}}` cites `r2`.
///
/// Example: a text `"{{r1.matches.0.name}} {{r2.balance}}"` cites `r1` and `r2`.
pub fn cited_ids(content: &str) -> Vec<String> {
    let Ok(model) = ModelAnswer::parse(content) else {
        return Vec::new();
    };
    let mut ids: Vec<String> = Vec::new();
    let texts = std::iter::once(model.text.as_str()).chain(model.next.iter().map(String::as_str));
    for text in texts {
        for piece in pieces(text) {
            if let Piece::Ref(r) = piece {
                let id = r.path.split('.').next().unwrap_or("").to_owned();
                if !id.is_empty() && !ids.contains(&id) {
                    ids.push(id);
                }
            }
        }
    }
    for show in &model.show {
        if !ids.contains(&show.result) {
            ids.push(show.result.clone());
        }
    }
    ids
}

/// The id of the result a source was read from.
fn source_result(source: &AnswerSource) -> &str {
    match source {
        AnswerSource::PartyCard { result_id, .. }
        | AnswerSource::Report { result_id, .. }
        | AnswerSource::Analytics { result_id, .. }
        | AnswerSource::Tiles { result_id } => result_id,
    }
}

/// How a cited source relates to the found party: `Some(true)` is about him, `Some(false)` is
/// about another party, `None` is about no party (tiles, or a report with no party filter).
fn about_party(source: &AnswerSource, code: &Option<String>) -> Option<bool> {
    match source {
        AnswerSource::PartyCard { party, .. } => Some(&party.code == code),
        AnswerSource::Report { filters, .. } => {
            filters.party.as_ref().map(|p| &p.record.code == code)
        }
        AnswerSource::Analytics { filters, .. } => {
            let seller = filters.seller.as_ref().map(|r| &r.code == code);
            let buyer = filters.buyer.as_ref().map(|p| &p.record.code == code);
            match (seller, buyer) {
                (None, None) => None,
                (a, b) => Some(a.unwrap_or(false) || b.unwrap_or(false)),
            }
        }
        AnswerSource::Tiles { .. } => None,
    }
}

/// Puts the name of the party the app found at the start of an answer that is about him but does
/// not show his name ("اکرم ٹول ٹیکس: ..."), so the Owner always sees which party was read.
///
/// "About him" means the results the answer cites (`cited`) are his and no cited result is another
/// party's. Sources the app only read ahead are not cited and do not count. An answer about
/// someone else, or that already names him, is left as it is.
pub fn ensure_name(answer: &mut Answer, found: &AnswerName, cited: &[String]) {
    let named = answer
        .parts
        .iter()
        .any(|p| matches!(p, AnswerPart::Name { name } if name.code == found.code));
    if named {
        return;
    }
    let relations: Vec<Option<bool>> = answer
        .sources
        .iter()
        .filter(|s| cited.iter().any(|c| c == source_result(s)))
        .map(|s| about_party(s, &found.code))
        .collect();
    let about = relations.contains(&Some(true)) && !relations.contains(&Some(false));
    if !about {
        return;
    }
    answer.parts.insert(
        0,
        AnswerPart::Name {
            name: found.clone(),
        },
    );
    answer.parts.insert(
        1,
        AnswerPart::Text {
            text: ": ".to_owned(),
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::{AnswerRecord, SourceParty};
    use mandi_reports::reports::ReportId;

    #[test]
    fn the_role_follows_the_words_of_the_question() {
        assert_eq!(role_of("ikram ke total bakaya kitne hai"), "buyer");
        assert_eq!(role_of("اکرم سے کتنا لینا ہے"), "buyer");
        assert_eq!(role_of("how much does akram owe"), "buyer");
        assert_eq!(role_of("kareem ko kitna dena he"), "seller");
        assert_eq!(
            role_of("طاہر کو دی ہوئی پیشگی میں سے کتنی کھلی ہے"),
            "seller"
        );
        assert_eq!(role_of("akram ka khata dikhao"), "any_party");
        // Both a buyer's and a seller's word: either.
        assert_eq!(role_of("akram se lena aur dena"), "any_party");
    }

    #[test]
    fn only_a_word_that_may_be_a_name_is_looked_up() {
        assert_eq!(name_words("ikram ke total bakaya kitne hai"), ["ikram"]);
        assert_eq!(
            name_words("how much does Arshad Multani owe"),
            ["arshad", "multani"]
        );
        assert_eq!(name_words("اکرم سے کتنا لینا ہے"), ["اکرم"]);
        // The same word twice once; two letters, digits and cue words never.
        assert_eq!(name_words("akram akram ka khata dikhao"), ["akram"]);
        assert!(name_words("kitna cash hai aaj").is_empty());
        assert!(name_words("akram 123").iter().all(|w| w == "akram"));
    }

    #[test]
    fn a_plain_lookup_has_no_number_no_how_to_no_period_and_no_tile() {
        assert!(plain_lookup("ikram ke total bakaya kitne hai", true));
        assert!(!plain_lookup("ikram ki 5000 wasooli", false));
        assert!(!plain_lookup("akram ka khata kaise dekhun", true));
        assert!(!plain_lookup("akram ka khata is mahine", true));
        assert!(!plain_lookup("aaj ki bikri kitni hui", true));
    }

    fn record(code: &str) -> AnswerRecord {
        AnswerRecord {
            id: 20,
            code: Some(code.to_owned()),
            name_ur: "اکرم ٹول ٹیکس".to_owned(),
            name_en: "Akram Toll Tax".to_owned(),
            tag: None,
        }
    }

    fn found(code: &str) -> AnswerName {
        AnswerName {
            name_ur: "اکرم ٹول ٹیکس".to_owned(),
            name_en: "Akram Toll Tax".to_owned(),
            tag: None,
            code: Some(code.to_owned()),
        }
    }

    fn answer_with(parts: Vec<AnswerPart>, sources: Vec<AnswerSource>) -> Answer {
        Answer {
            parts,
            tables: Vec::new(),
            charts: Vec::new(),
            estimate: None,
            advice: false,
            general_advice: false,
            sources,
            differences: Vec::new(),
            next: Vec::new(),
            read_at: String::new(),
        }
    }

    #[test]
    fn the_found_party_is_named_in_an_answer_about_him_and_only_then() {
        let about_him = AnswerSource::PartyCard {
            result_id: "r2".to_owned(),
            party: record("0020"),
        };
        let text = |t: &str| AnswerPart::Text { text: t.to_owned() };
        // About him and unnamed: his name goes first.
        let mut a = answer_with(vec![text("2,880 باقی")], vec![about_him.clone()]);
        ensure_name(&mut a, &found("0020"), &["r2".to_owned()]);
        assert!(
            matches!(&a.parts[0], AnswerPart::Name { name } if name.code.as_deref() == Some("0020"))
        );
        assert!(matches!(&a.parts[1], AnswerPart::Text { text } if text == ": "));
        assert_eq!(a.parts.len(), 3);
        // Already named: left alone.
        let mut named = answer_with(
            vec![
                AnswerPart::Name {
                    name: found("0020"),
                },
                text("x"),
            ],
            vec![about_him.clone()],
        );
        ensure_name(&mut named, &found("0020"), &["r2".to_owned()]);
        assert_eq!(named.parts.len(), 2);
        // About someone else (the model found another party itself): left alone.
        let other = AnswerSource::PartyCard {
            result_id: "r4".to_owned(),
            party: record("0021"),
        };
        let mut b = answer_with(vec![text("x")], vec![other]);
        ensure_name(&mut b, &found("0020"), &["r4".to_owned()]);
        assert_eq!(b.parts.len(), 1);
        // A report of his ledger counts as about him.
        let report = AnswerSource::Report {
            result_id: "r3".to_owned(),
            label_id: "x".to_owned(),
            id: ReportId::R13,
            filters: crate::source::SourceReportFilters {
                party: Some(SourceParty {
                    record: record("0020"),
                    person_key: None,
                }),
                ..crate::source::SourceReportFilters::default()
            },
        };
        let mut c = answer_with(vec![text("x")], vec![report]);
        ensure_name(&mut c, &found("0020"), &["r3".to_owned()]);
        assert_eq!(c.parts.len(), 3);
    }

    #[test]
    fn a_source_the_app_only_read_ahead_does_not_make_the_answer_about_him() {
        let his = AnswerSource::PartyCard {
            result_id: "r2".to_owned(),
            party: record("0020"),
        };
        let other = AnswerSource::PartyCard {
            result_id: "r4".to_owned(),
            party: record("0021"),
        };
        let text = |t: &str| AnswerPart::Text { text: t.to_owned() };
        // His card was read ahead (r2) but the answer cites the other party's card only.
        let mut a = answer_with(vec![text("x")], vec![his.clone(), other.clone()]);
        ensure_name(&mut a, &found("0020"), &["r3".to_owned(), "r4".to_owned()]);
        assert_eq!(a.parts.len(), 1);
        // The answer cites both parties: it is not about him alone.
        let mut b = answer_with(vec![text("x")], vec![his.clone(), other]);
        ensure_name(&mut b, &found("0020"), &["r2".to_owned(), "r4".to_owned()]);
        assert_eq!(b.parts.len(), 1);
        // The answer cites nothing that is about a party.
        let mut c = answer_with(vec![text("x")], vec![his]);
        ensure_name(&mut c, &found("0020"), &[]);
        assert_eq!(c.parts.len(), 1);
    }

    #[test]
    fn the_cited_results_are_the_references_and_the_shown_results() {
        let content = r#"{"kind":"answer","text":"{{r1.matches.0.name}} {{ r2.balance }} {{r2.rows.x}}","show":[{"result":"r3","as":"table","rows":null}],"next":["{{r5.total}}"],"choose":null,"entry":null,"nearest":null}"#;
        assert_eq!(cited_ids(content), ["r1", "r2", "r5", "r3"]);
        assert!(cited_ids("not json").is_empty());
    }
}
