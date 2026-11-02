//! The reads the app does itself after a `find` with exactly one match (plan 52 P52-S1).
//!
//! The prompt asks the model to send `find` and `party_card` in the same reply. The live run of
//! 03-11-2026 showed that it never does: every name question was `find`, then `party_card` or
//! `report` in the next call, so 3 or 4 model calls instead of 2. When the model's reply holds only
//! `find` calls and a find matched exactly one buyer or seller (one search hit, exact or a close
//! spelling, as the model would have taken it), the app reads the party's card
//! itself (and more when the question names it, below), as the model would have. Each result has
//! its own id (`p1`, `p2` ...), so the model's own `r` numbers never move. The model may still call
//! anything it needs.
//!
//! The card is always right: it is the party's balance now. The other reads are for the cues the
//! question names, and only when it names no period and no number, because they are read for the
//! whole books and that is then exactly what is asked:
//!
//! | The question names | A buyer | A seller |
//! |---|---|---|
//! | a ledger (`khata`, `hisab`, `bill`) | R13 | R13 |
//! | what he bought (`khareeda`, `bought`) | R2 with him as `party` | nothing more |
//! | commission (`commission`, `کمیشن`) | view C with him as `buyer` | view B with him as `seller` |
//!
//! Nothing here works out a figure: the tools are the same ones the model calls.
//!
//! Example: "akram ka khata dikhao" with one buyer found as `r1` reads `party_card` (`p1`) and R13
//! (`p2`) at once; "akram kitna dega" reads `p1` only; "akram ka khata is mahine" reads `p1` only,
//! because a month is named and the model asks for it with its period.

use serde_json::{Value, json};

use crate::results::Node;
use crate::tools::State;

/// The roles of a `find` whose single match is a party worth reading at once.
const PARTY_ROLES: [&str; 3] = ["buyer", "seller", "any_party"];

/// How many finds of one reply are read for (a question about two parties).
pub const MAX_FINDS: usize = 2;

/// The words that name a ledger: the question wants the party's entries, not only his balance.
const LEDGER_CUES: &[&str] = &[
    "khata",
    "khaata",
    "khate",
    "khatay",
    "hisab",
    "hisaab",
    "ledger",
    "statement",
    "bill",
    "bills",
    "کھاتہ",
    "کھاتے",
    "کھاتا",
    "حساب",
    "بل",
    "بلز",
    "فرد",
    "لیجر",
];

/// The words that ask what a buyer bought. (`kharidari` is the market trips' word: not here.)
const PURCHASE_CUES: &[&str] = &[
    "khareeda",
    "khareede",
    "khareedi",
    "kharida",
    "kharide",
    "kharidi",
    "khareed",
    "kharid",
    "bought",
    "buy",
    "purchase",
    "purchases",
    "purchased",
    "خریدا",
    "خریدے",
    "خریدی",
    "خرید",
];

/// The words that ask for commission.
const COMMISSION_CUES: &[&str] = &["commission", "comm", "کمیشن", "کمشن", "لاگا"];

/// The words that name a period. A question that names one is for the model to read with the
/// period: the app reads ahead for the whole books only. Any digit counts too (a date, a number of
/// days).
const PERIOD_WORDS: &[&str] = &[
    "aaj",
    "aj",
    "kal",
    "parson",
    "today",
    "yesterday",
    "tomorrow",
    "hafta",
    "hafte",
    "haftay",
    "week",
    "weeks",
    "mahina",
    "mahine",
    "mahinay",
    "month",
    "months",
    "saal",
    "sal",
    "year",
    "years",
    "din",
    "dino",
    "day",
    "days",
    "pichle",
    "pichla",
    "pichli",
    "agle",
    "agla",
    "last",
    "previous",
    "next",
    "since",
    "january",
    "february",
    "march",
    "april",
    "june",
    "july",
    "august",
    "september",
    "october",
    "november",
    "december",
    "jan",
    "feb",
    "aug",
    "sep",
    "sept",
    "oct",
    "nov",
    "dec",
    "آج",
    "کل",
    "ہفتہ",
    "ہفتے",
    "مہینہ",
    "مہینے",
    "سال",
    "دن",
    "پچھلے",
    "پچھلا",
    "پچھلی",
    "اگلے",
    "اگلا",
    "جنوری",
    "فروری",
    "مارچ",
    "اپریل",
    "جون",
    "جولائی",
    "اگست",
    "ستمبر",
    "اکتوبر",
    "نومبر",
    "دسمبر",
];

/// What a party is, for the extra reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// A buyer.
    Buyer,
    /// A seller.
    Seller,
}

/// A party the Owner named that a find matched exactly once.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Party {
    /// The path of the match (`r1.matches.0`).
    pub path: String,
    /// Buyer or seller, as the find says.
    pub kind: Kind,
}

fn words_of(question: &str) -> Vec<String> {
    question
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric() && !('\u{0600}'..='\u{06FF}').contains(&c))
        .filter(|w| !w.is_empty())
        .map(str::to_owned)
        .collect()
}

fn has_any(words: &[String], cues: &[&str]) -> bool {
    words.iter().any(|w| cues.contains(&w.as_str()))
}

/// True when the question names a ledger (`khata`, `hisab`, `bill`, `کھاتہ`, `حساب`, `بل` ...).
pub fn ledger_asked(question: &str) -> bool {
    has_any(&words_of(question), LEDGER_CUES)
}

/// True when the question names a period or holds a digit of any script.
pub fn period_named(question: &str) -> bool {
    question.chars().any(char::is_numeric) || has_any(&words_of(question), PERIOD_WORDS)
}

/// The party of a find that matched exactly one buyer or seller, found as typed or by its Urdu
/// spelling, with nothing more to show. `None` otherwise.
pub fn single_party_match(state: &State, find_id: &str) -> Option<Party> {
    let result = state.find(find_id)?;
    if result.tool != "find" {
        return None;
    }
    let role = result.input.get("role").and_then(Value::as_str)?;
    if !PARTY_ROLES.contains(&role) {
        return None;
    }
    let Some(Node::List(matches)) = result.node.get(&["matches"]) else {
        return None;
    };
    let [only] = matches.as_slice() else {
        return None;
    };
    let more = matches!(result.node.get(&["more"]), Some(Node::Bool(true)));
    // A match found by the spelling rules alone is for the model to confirm with the Owner.
    let by_rules = matches!(result.node.get(&["via"]), Some(Node::Str(s)) if s == "rules");
    let kind = match only.get(&["type"]) {
        Some(Node::Str(t)) if t == "buyer" => Kind::Buyer,
        Some(Node::Str(t)) if t == "seller" => Kind::Seller,
        _ => return None,
    };
    (!more && !by_rules).then(|| Party {
        path: format!("{find_id}.matches.0"),
        kind,
    })
}

/// The tool calls to read ahead for a party: his card always, then what the question names (see the
/// table above). Each is a tool name and its input as the model would send it.
pub fn reads(question: &str, party: &Party) -> Vec<(&'static str, String)> {
    let path = party.path.as_str();
    let mut out = vec![("party_card", json!({"party": path}).to_string())];
    if period_named(question) {
        return out;
    }
    let words = words_of(question);
    if has_any(&words, LEDGER_CUES) {
        out.push(("report", report_input("R13", path)));
    }
    if party.kind == Kind::Buyer && has_any(&words, PURCHASE_CUES) {
        out.push(("report", report_input("R2", path)));
    }
    if has_any(&words, COMMISSION_CUES) {
        out.push(("analytics", commission_input(party)));
    }
    out
}

/// The input of a report for a party: the first 20 rows, no dates.
fn report_input(id: &str, party: &str) -> String {
    json!({
        "id": id, "party": party, "rows": 20,
        "adda": null, "as_of": null, "bikri": null, "city": null, "collector": null,
        "from": null, "garden": null, "group_by": null, "item": null, "partner": null,
        "sort": null, "to": null, "trip": null, "where": null
    })
    .to_string()
}

/// The input of the view that gives a party's commission over the whole books: view C for a buyer,
/// view B for a seller (PRD FR-18.2).
fn commission_input(party: &Party) -> String {
    let (view, buyer, seller) = match party.kind {
        Kind::Buyer => ("C", Some(party.path.as_str()), None),
        Kind::Seller => ("B", None, Some(party.path.as_str())),
    };
    json!({
        "view": view, "buyer": buyer, "seller": seller,
        "from": null, "to": null, "group_by": null, "item": null, "city": null, "adda": null,
        "garden": null, "trip": null, "goods": null, "partner": null, "side": null,
        "compare": null, "sort": null, "rows": null
    })
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn buyer() -> Party {
        Party {
            path: "r1.matches.0".to_owned(),
            kind: Kind::Buyer,
        }
    }

    fn tools_of(question: &str, party: &Party) -> Vec<String> {
        reads(question, party)
            .into_iter()
            .map(|(tool, args)| {
                let v: Value = serde_json::from_str(&args).unwrap();
                match tool {
                    "report" => format!("report {}", v["id"].as_str().unwrap()),
                    "analytics" => format!("analytics {}", v["view"].as_str().unwrap()),
                    other => other.to_owned(),
                }
            })
            .collect()
    }

    #[test]
    fn a_question_names_a_ledger_by_one_of_its_words() {
        for q in [
            "akram ka khata dikhao",
            "arshad ka hisab kya hai",
            "اکرم کا کھاتہ دکھائیں",
            "رحیم کا حساب",
            "Akram ka Bill kitne ka tha",
            "show the ledger of akram",
        ] {
            assert!(ledger_asked(q), "{q}");
        }
        for q in [
            "how much does akram owe",
            "اکرم سے کتنا لینا ہے",
            "kareem ko kitna dena he",
            "arshad ne abhi tak kuch diya",
            // A word that only contains a cue is not the cue.
            "billing ki baat",
        ] {
            assert!(!ledger_asked(q), "{q}");
        }
    }

    #[test]
    fn a_period_or_a_number_in_the_question_keeps_the_extra_reads_for_the_model() {
        for q in [
            "akram ka khata is mahine",
            "arshad ne aaj kya khareeda",
            "commission pichle saal ka",
            "اکرم کا کھاتہ پچھلے مہینے",
            "akram ka khata 10 din ka",
            "bill of 2026",
            "khata in october",
        ] {
            assert!(period_named(q), "{q}");
            // Only the card is read ahead.
            assert_eq!(tools_of(q, &buyer()), ["party_card"], "{q}");
        }
        for q in [
            "akram ka khata",
            "faisal ne kya khareeda",
            "abhi tak ka hisab",
        ] {
            assert!(!period_named(q), "{q}");
        }
    }

    #[test]
    fn what_is_read_ahead_follows_the_cue_and_the_kind_of_the_party() {
        let seller = Party {
            path: "r1.matches.0".to_owned(),
            kind: Kind::Seller,
        };
        // No cue: the card only.
        assert_eq!(tools_of("akram kitna dega", &buyer()), ["party_card"]);
        // A ledger: R13, for either.
        assert_eq!(
            tools_of("akram ka khata", &buyer()),
            ["party_card", "report R13"]
        );
        assert_eq!(
            tools_of("kareem ka hisab", &seller),
            ["party_card", "report R13"]
        );
        // What a buyer bought: R2 with him as party. A seller bought nothing from us: no R2.
        assert_eq!(
            tools_of("faisal ne kya khareeda", &buyer()),
            ["party_card", "report R2"]
        );
        assert_eq!(tools_of("kareem ne kya khareeda", &seller), ["party_card"]);
        // Commission: view C for a buyer, view B for a seller.
        assert_eq!(
            tools_of("kareem ke maal pe commission", &seller),
            ["party_card", "analytics B"]
        );
        assert_eq!(
            tools_of("faisal se kitna commission mila", &buyer()),
            ["party_card", "analytics C"]
        );
        // The market trips' word is not a purchase cue.
        assert_eq!(tools_of("kharidari kaisi rahi", &buyer()), ["party_card"]);
    }
}
