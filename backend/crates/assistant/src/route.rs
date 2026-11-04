//! Pointers to the right source for the questions the live run of 03-11-2026 sent to the wrong one
//! (plan 52 P52-S1, class 1 wrong figures).
//!
//! A report or a view answers one question well and looks like it answers a nearby one. The model
//! picked R29 for the cost of a market trip (R29 holds shop expenses only, so it said "none") and
//! the line count of R2 for the number of بکری. The result now says what it holds and where the
//! other figure is, in its `note`, so the next call goes to the right source. No figure is worked
//! out here.
//!
//! Example: a call of `report` with `{"id":"R2"}` gets the note that `count` is sale lines.

use serde_json::Value;

use crate::ask::{AskedRole, FoundAs};
use crate::results::{Node, Val};
use crate::tools::{State, cues, parse_key};

/// What R2's `count` is and where the number of بکری is (Q152: 122 lines were read as 122 بکری,
/// the books hold 32).
const R2_NOTE: &str = "the count of this register is the number of sale LINES, and one بکری has \
                       several; the number of بکری is the bikri_count in the totals of analytics \
                       view B";

/// What R29 holds and where a trip's or a garden's money is (Q061: no trip cost found in R29,
/// whose heads are shop expenses and partner money only).
const R29_NOTE: &str = "this report holds the shop's expense heads and the money partners took \
                        and returned, nothing of market trips or gardens; a trip's cost and \
                        result are in R33 or analytics view J, a garden's in R36 or analytics \
                        view I";

/// The sentence a result of this call carries about what it holds, if it needs one.
///
/// Example: `scope_note("report", r#"{"id":"R29"}"#)` is the note of R29; a `party_card` has none.
pub fn scope_note(tool: &str, arguments: &str) -> Option<&'static str> {
    if tool != "report" {
        return None;
    }
    let value: Value = serde_json::from_str(arguments).ok()?;
    match value.get("id")?.as_str()? {
        "R2" => Some(R2_NOTE),
        "R29" => Some(R29_NOTE),
        _ => None,
    }
}

/// True when the question asks HOW MANY بکری there were, in a plural word: "kitni bikriyan hui".
/// The singular "kitni bikri" is not here: in the mandi it also asks for the amount of the sales
/// ("aaj ki bikri kitni hui").
///
/// Example: "pichle mahine kitni bikriyan hui" is true; "aaj ki bikri kitni hui" is false.
pub fn bikri_count_asked(question: &str) -> bool {
    cues::has(
        question,
        &[
            "bikriyan",
            "bikriyaan",
            "bikriyon",
            "bikrian",
            "بکریاں",
            "بکریوں",
            "how many sales",
            "number of sales",
            "count of sales",
            "sales count",
        ],
    )
}

// ---- not found, and found in another role (plan 52 Section 8) --------------------------------------

/// The kind of record a `find` was asked for, from its `role`.
///
/// Example: `asked_role_of("any_party")` is `Any`; `"seller"` is `Seller`.
pub fn asked_role_of(word: &str) -> AskedRole {
    match word {
        "seller" => AskedRole::Seller,
        "buyer" => AskedRole::Buyer,
        "item" => AskedRole::Item,
        "partner" => AskedRole::Partner,
        "garden" => AskedRole::Garden,
        "trip" => AskedRole::Trip,
        _ => AskedRole::Any,
    }
}

/// The kind of record the find in `find_id` was asked for.
pub fn asked_role_of_find(state: &State, find_id: &str) -> Option<AskedRole> {
    let word = state.find(find_id)?.input.get("role")?.as_str()?;
    Some(asked_role_of(word))
}

/// True when the search was widened to another role (nothing had the name in the role asked).
pub fn is_widened(state: &State, find_id: &str) -> bool {
    state
        .find(find_id)
        .is_some_and(|r| r.node.get(&["widened"]).is_some())
}

/// The party a find found in another role than the one asked, when there is exactly one: his key,
/// id, role and names. `None` for no match, several, or a find that was not widened and whose match
/// is of the role asked.
///
/// Example: Faisal, a buyer, asked as a seller: `FoundAs { key: "a31", role: Buyer, .. }`.
pub fn found_as_of(state: &State, find_id: &str, asked: Option<AskedRole>) -> Option<FoundAs> {
    let result = state.find(find_id)?;
    let Some(Node::List(matches)) = result.node.get(&["matches"]) else {
        return None;
    };
    let [only] = matches.as_slice() else {
        return None;
    };
    let role = match (only.get(&["type"]), result.node.get(&["widened"])) {
        (Some(Node::Str(t)), _) if t == "buyer" => AskedRole::Buyer,
        (Some(Node::Str(t)), _) if t == "seller" => AskedRole::Seller,
        (_, Some(Node::Str(w))) => asked_role_of(w),
        _ => return None,
    };
    if Some(role) == asked && !is_widened(state, find_id) {
        return None;
    }
    let key = match only.get(&["key"])? {
        Node::Str(key) => key.clone(),
        _ => return None,
    };
    let name = match only.get(&["name"])? {
        Node::Val(Val::Name(name)) => name.answer_name(),
        _ => return None,
    };
    Some(FoundAs {
        id: parse_key(&key)?.id,
        key,
        role,
        name,
    })
}

/// The role a verb of the question gives to the one it is about: "becha" (sold) a seller,
/// "khareeda" (bought) a buyer. `None` for no such verb or both. The ambiguous words (bakaya,
/// dena, lena) are not here: they fit a buyer and a seller alike.
///
/// Example: "Faisal ne kya becha" is `Seller`; "Faisal ne aaj kya khareeda" is `Buyer`.
pub fn trade_role(question: &str) -> Option<AskedRole> {
    let sold = cues::has(
        question,
        &[
            "becha",
            "beche",
            "bechi",
            "bechna",
            "bechta",
            "bechte",
            "bika",
            "bike",
            "sold",
            "sell",
            "sells",
            "selling",
            "بیچا",
            "بیچے",
            "بیچی",
            "بیچنا",
            "بکا",
            "بکے",
        ],
    );
    let bought = cues::has(
        question,
        &[
            "khareeda",
            "khareede",
            "khareedi",
            "khareedna",
            "kharida",
            "kharide",
            "kharidi",
            "bought",
            "buy",
            "buys",
            "purchased",
            "خریدا",
            "خریدے",
            "خریدی",
            "خریدنا",
        ],
    );
    // "Faisal ko becha" (sold to him) and "Faisal se khareeda" (bought from him) are the other
    // way round: the verb says nothing about the role of the one named.
    let sold_to = cues::has(question, &["ko", "کو", "sold to", "sell to", "sells to"]);
    let bought_from = cues::has(
        question,
        &["se", "سے", "bought from", "buy from", "buys from"],
    );
    match (sold && !sold_to, bought && !bought_from) {
        (true, false) => Some(AskedRole::Seller),
        (false, true) => Some(AskedRole::Buyer),
        _ => None,
    }
}

/// The role the question's verb asks for, when a find found only parties of the other role: "Faisal
/// ne kya becha" and the only Faisal is a buyer. Then his purchases must not be read as sales: the
/// question ends as not found, with `found_as` (plan 52 Section 8).
pub fn wrong_role_found(question: &str, state: &State, find_id: &str) -> Option<AskedRole> {
    let implied = trade_role(question)?;
    let result = state.find(find_id)?;
    let Some(Node::List(matches)) = result.node.get(&["matches"]) else {
        return None;
    };
    // Only a search made as the verb's role, or as any party, can be the wrong one: a model that
    // searched as the other role on purpose ("Faisal ko becha": a buyer) is right.
    let searched_as = match result.node.get(&["role"]) {
        Some(Node::Str(role)) => role.as_str(),
        _ => "",
    };
    let wanted = match implied {
        AskedRole::Seller => "seller",
        _ => "buyer",
    };
    if searched_as != wanted && searched_as != "any_party" {
        return None;
    }
    let other = match implied {
        AskedRole::Seller => "buyer",
        _ => "seller",
    };
    let all_other = !matches.is_empty()
        && matches
            .iter()
            .all(|m| matches!(m.get(&["type"]), Some(Node::Str(t)) if t == other));
    all_other.then_some(implied)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_verb_gives_the_role_of_the_one_named_unless_the_party_is_the_other_side_of_it() {
        assert_eq!(trade_role("faisal ne kya becha"), Some(AskedRole::Seller));
        assert_eq!(
            trade_role("faisal ne aaj kya khareeda"),
            Some(AskedRole::Buyer)
        );
        assert_eq!(trade_role("what did faisal sell"), Some(AskedRole::Seller));
        // Sold TO him, bought FROM him, both verbs, or none: no role.
        assert_eq!(trade_role("faisal ko kitna becha"), None);
        assert_eq!(trade_role("faisal se kitna khareeda"), None);
        assert_eq!(trade_role("faisal ne becha aur khareeda"), None);
        assert_eq!(trade_role("faisal ka bakaya"), None);
    }

    #[test]
    fn r2_and_r29_say_what_they_hold_and_nothing_else_does() {
        assert_eq!(
            scope_note("report", r#"{"id":"R2","from":null}"#),
            Some(R2_NOTE)
        );
        assert_eq!(scope_note("report", r#"{"id":"R29"}"#), Some(R29_NOTE));
        assert_eq!(scope_note("report", r#"{"id":"R33"}"#), None);
        assert_eq!(scope_note("analytics", r#"{"view":"B"}"#), None);
        assert_eq!(scope_note("report", "not json"), None);
        assert_eq!(scope_note("report", r#"{"rows":5}"#), None);
    }

    #[test]
    fn only_a_plural_word_asks_for_the_number_of_bikri() {
        for q in [
            "pichle mahine kitni bikriyan hui",
            "اس مہینے کتنی بکریاں ہوئیں",
            "how many sales were there last month",
            "number of sales this year",
        ] {
            assert!(bikri_count_asked(q), "{q}");
        }
        for q in [
            "aaj ki bikri kitni hui",
            "آج کی خام بکری کتنی ہے",
            "is saal ki kul bikri aur commission kitni hui",
            "how much did we sell",
        ] {
            assert!(!bikri_count_asked(q), "{q}");
        }
    }
}
