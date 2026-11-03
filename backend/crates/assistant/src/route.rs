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

use crate::tools::cues;

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

#[cfg(test)]
mod tests {
    use super::*;

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
