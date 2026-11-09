//! D7 (plan 59, F3b): the six generic guards of the instant answers, on the rich test year.
//!
//! Each guard is a RULE over the words of a question, never a list of the questions of one run, and each
//! test names the rule and shows both sides of it with the oracle's figures:
//!
//! 1. a COUNT asked ("kitne gahkon", "how many buyers", in any spelling the lexicon takes for the role
//!    word and in all three scripts) never gets an amount;
//! 2. an answer holds as many figures as the question has "how much / how many" cues, or the model answers;
//! 3. ordinary words and date words are never searched as names: they are not "not found", and a day
//!    number no period placed makes the question unsure of its time (a wrong period is a wrong figure);
//! 4. everyday function words (pronouns, postpositions, light verbs, time and manner words) do not switch
//!    off a total; a noun nothing explains still does, and so does a verb that says which way money moved;
//! 5. a state ("have to pay", "bachi hui hai", "is left", "put into the business") is no event;
//! 6. Urdu "کل" before a figure noun of NOW is "total", before a time word or with a past verb it is
//!    yesterday, and when neither is clear the model answers.
#![cfg(test)]

#[path = "../../services/tests/common/mod.rs"]
mod common;
#[path = "support/harness.rs"]
mod harness;
#[path = "support/worked.rs"]
mod worked;
#[allow(dead_code)]
#[path = "../../services/tests/bikri_preview/support.rs"]
mod world;
#[path = "robustness_year/mod.rs"]
mod year;

use harness::{Harness, answer_of, count, figures, money};
use mandi_assistant::ask::{AskOutcome, AskReply};
use mandi_reports::reports::Cell;
use std::collections::BTreeMap;

fn production_harness(y: year::seed::Year) -> Harness {
    let h = Harness::over(y.w);
    h.runtime.set_read_first(true);
    h.runtime.set_local(true);
    h.runtime.set_instant(true);
    h.runtime.set_low_effort(true);
    h
}

fn facts_of(y: &year::seed::Year) -> BTreeMap<String, String> {
    year::oracle::compute(y).facts
}

fn oracle(facts: &BTreeMap<String, String>, key: &str) -> i64 {
    facts
        .get(key)
        .unwrap_or_else(|| panic!("no oracle fact {key}"))
        .parse()
        .unwrap_or_else(|_| panic!("oracle fact {key} is not a number"))
}

/// An oracle balance: its rupees, with the side word the oracle writes after them left off.
fn oracle_balance(facts: &BTreeMap<String, String>, key: &str) -> i64 {
    let text = facts
        .get(key)
        .unwrap_or_else(|| panic!("no oracle fact {key}"));
    text.split_whitespace()
        .next()
        .and_then(|n| n.parse().ok())
        .unwrap_or_else(|| panic!("oracle fact {key} is not a balance: {text}"))
}

fn shown(reply: &AskReply) -> Vec<i64> {
    figures(answer_of(reply))
        .iter()
        .map(|(cell, _)| match cell {
            Cell::Number { .. } => count(cell),
            Cell::Balance { paisa, .. } => paisa.rupees(),
            other => money(other),
        })
        .collect()
}

/// A question the app answers by itself: no request to the model, an answer, and these figures.
fn instant(h: &Harness, question: &str, expected: &[i64]) {
    let (reply, requests) = h.ask(question, None, Vec::new());
    assert!(requests.is_empty(), "{question}: the model was called");
    assert_eq!(reply.outcome, AskOutcome::Answered, "{question}");
    assert_eq!(shown(&reply), expected, "{question}");
}

/// A question the model answers: it is asked (it has no reply here), and the app gave no "not found".
fn goes_to_the_model(h: &Harness, question: &str) {
    let (reply, requests) = h.ask(question, None, Vec::new());
    assert!(!requests.is_empty(), "{question}: the app answered alone");
    assert_ne!(
        reply.outcome,
        AskOutcome::NotFound,
        "{question}: refused as not found"
    );
}

#[test]
fn guard_1_a_count_asked_never_gets_an_amount_in_any_spelling_of_the_role_word() {
    let y = year::seed::build();
    let f = facts_of(&y);
    let h = production_harness(y);
    // The role word is misspelt in every way the lexicon accepts, in all three scripts: Roman
    // ("gahkon", "gahako", "gaahkon"), Urdu letters ("گاہکوں") and English ("buyres", "bayers",
    // "customres"). "kitne" / "how many" before it asks the NUMBER of buyers, which no figure of dues is.
    for question in [
        "kitne gahkon ke zimme paise baqi hain",
        "kitne gahako ke zimme paise baqi hain",
        "kitne gaahkon ke zimme paise baqi hain",
        "کتنے گاہکوں کے ذمے پیسے باقی ہیں",
        "how many buyres owe money",
        "how many bayers owe us money",
        "how many customres have dues",
        "kitne gahakon se wasooli baqi hai",
        "کتنے بیوپاریوں کو پیسے دینے ہیں",
        "kitne bayopari ko paise dene hain",
    ] {
        goes_to_the_model(&h, question);
    }
    // The AMOUNT of the same words, asked with "kitna" / "how much", is the dues: this is the rule's
    // other side, and why a bare "kitna" reads the amount (the amount of the figure the question names)
    // and "kitne" before a role word reads a count.
    for question in [
        "gahkon ke zimme kitna baqi hai",
        "how much do buyers owe",
        "گاہكوں كے ذمے كتنا باقی ہے",
    ] {
        instant(&h, question, &[oracle(&f, "tiles.buyers_dues")]);
    }
}

#[test]
fn guard_2_an_answer_never_leaves_out_a_part_of_what_was_asked() {
    let y = year::seed::build();
    let f = facts_of(&y);
    let h = production_harness(y);
    // Two or three "how much" cues and a plan for one of them: the model answers, never the one figure.
    for question in [
        "how much was sold and how much commission today",
        "aaj kitni bikri hui aur kitna maal bika aur kitna commission bana",
        "kitna maal liya kitne ka aur kitna commission",
        "how much is the shop expense and how much did the partners take",
    ] {
        goes_to_the_model(&h, question);
    }
    // Two measures in one question go to the model (lead decision D333: PRD FR-7.90 sends two-part questions to
    // the model); with the gate ON they are answered together, each figure its own sentence.
    goes_to_the_model(&h, "aaj kitni bikri hui aur kitna commission bana");
    h.runtime.set_multi_measure(true);
    let (reply, requests) = h.ask(
        "aaj kitni bikri hui aur kitna commission bana",
        None,
        Vec::new(),
    );
    assert!(requests.is_empty(), "both figures are in the plan");
    assert_eq!(
        shown(&reply),
        [
            oracle(&f, "a.today.total.gross"),
            oracle(&f, "a.today.total.total_comm")
        ]
    );
}

#[test]
fn guard_3_ordinary_and_date_words_are_not_names_and_an_unplaced_day_is_not_guessed() {
    let y = year::seed::build();
    let f = facts_of(&y);
    let h = production_harness(y);
    // Pronouns, determiners and common nouns are never searched as a party: no "not found".
    for question in [
        "do we have anything owed by buyers",
        "is there anything left in stock",
        "people owe how much",
        "how many people owe us money",
        "khara maal kitna bika aaj",
    ] {
        goes_to_the_model(&h, question);
    }
    // A date word is a date: "from the 1st of the month to today" is this month so far.
    for question in [
        "یکم سے آج تک کا کمیشن",
        "1 se aaj tak ka commission",
        "from the 1st till today how much commission",
        "how much commission from 1 till now",
    ] {
        instant(&h, question, &[oracle(&f, "a.month.total.total_comm")]);
    }
    // A day number that no period placed is a time the app cannot read: the model answers, the whole
    // month (or the whole book) is never the answer to "the 1st".
    goes_to_the_model(&h, "اس مہینے کی یکم کو بکری کتنی ہوئی");
    goes_to_the_model(&h, "اس مہینے کی 1 تاریخ کو بکری کتنی ہوئی");
}

#[test]
fn guard_4_function_words_switch_nothing_off_but_a_noun_or_a_direction_verb_does() {
    let y = year::seed::build();
    let f = facts_of(&y);
    let h = production_harness(y);
    let dues = oracle(&f, "tiles.buyers_dues");
    // Pronouns, postpositions, light verbs and time words: every word is a cue or a function word.
    for question in [
        "hamare gahkon ke zimme kitna baqi rehta hai",
        "ہمارے گاہکوں کے ذمے کتنا نکلتا ہے",
        "we still have how much dues from buyers",
    ] {
        instant(&h, question, &[dues]);
    }
    instant(&h, "us waqt kitna cash hai", &[oracle(&f, "tiles.cash")]);
    instant(&h, "اس وقت کتنا روکڑ ہے", &[oracle(&f, "tiles.cash")]);
    instant(
        &h,
        "how much commission did we earn this month",
        &[oracle(&f, "a.month.total.total_comm")],
    );
    instant(
        &h,
        "is saal kitna commission nikalta hai",
        &[oracle(&f, "a.year.total.total_comm")],
    );
    // A noun nothing explains could be a filter (an expense head, an item, a packing): the model answers.
    goes_to_the_model(&h, "is saal chai ka kharcha kitna hua");
    goes_to_the_model(&h, "how much has been spent on shop rent so far");
    // A verb that says which way money moved is known, but it is never a partner's total.
    for question in [
        "b partner ne kitna liya",
        "b partner ne kitna nikala",
        "how much did partner b take",
    ] {
        goes_to_the_model(&h, question);
    }
    instant(
        &h,
        "partner b ka kitna hai",
        &[oracle(&f, "partner.partner_b.total")],
    );
}

#[test]
fn guard_5_a_state_wording_is_no_event() {
    let y = year::seed::build();
    let f = facts_of(&y);
    let h = production_harness(y);
    let zahid = oracle_balance(&f, "buyer.zahid.balance");
    // What is open now, said with a past participle: a balance, not an event of a period.
    for question in [
        "how much does Zahid Mehmood need to pay",
        "how much is Zahid Mehmood due to pay",
        "how much does Zahid Mehmood have to pay",
        "zahid mehmood ka kitna baqi ho gaya hai",
        "zahid mehmood ka kitna bachi hui hai",
        "how much is left with Zahid Mehmood",
    ] {
        instant(&h, question, &[zahid]);
    }
    instant(
        &h,
        "b partner ne dukaan me kitna sarmaya lagaya hua hai",
        &[oracle(&f, "partner.partner_b.capital")],
    );
    // A word that gives something BACK, or a plain past event, stays an event: the model answers.
    for question in [
        "zahid mehmood ne kitna wapas kiya",
        "zahid mehmood ne kitne paise diye",
        "how much did Zahid Mehmood pay us",
        "ادھار رقم میں کتنی واپس آئی",
    ] {
        goes_to_the_model(&h, question);
    }
}

#[test]
fn guard_6_urdu_kal_is_total_before_a_figure_noun_of_now_and_yesterday_after_a_past_verb() {
    let y = year::seed::build();
    let f = facts_of(&y);
    let h = production_harness(y);
    // "کل" before a figure noun of NOW: "total" (kul), not a day.
    instant(
        &h,
        "کل ادائیگی کتنی ہے",
        &[oracle(&f, "tiles.sellers_payable")],
    );
    instant(&h, "کل اگراہی کتنی ہے", &[oracle(&f, "tiles.total_dues")]);
    // With a past verb or "ki / ka", it is yesterday: the figure of that day.
    instant(
        &h,
        "کل کتنا کمیشن بنا",
        &[oracle(&f, "a.yesterday.total.total_comm")],
    );
    instant(
        &h,
        "kal ka commission",
        &[oracle(&f, "a.yesterday.total.total_comm")],
    );
    // Neither clear: the model answers.
    goes_to_the_model(&h, "کل کتنی");
}

/// The answer shows none of these figures (the app may answer, or the model may; the figure of another
/// question is never the answer).
fn never_shows(h: &Harness, question: &str, wrong: &[i64]) {
    let (reply, requests) = h.ask(question, None, Vec::new());
    if requests.is_empty() && reply.outcome == AskOutcome::Answered {
        let seen = shown(&reply);
        assert!(
            seen.iter().all(|n| !wrong.contains(n)),
            "{question}: the app showed {seen:?}"
        );
    }
}

#[test]
fn money_review_a_word_that_changes_a_figure_is_never_explained_away() {
    // Money review of 08-11-2026, one question for each way a word made known could put the figure of
    // another question on the screen. Each goes to the model; none gets the figure of a plain balance, the
    // cash in hand, the month's whole sale or a partner's total.
    let y = year::seed::build();
    let f = facts_of(&y);
    let h = production_harness(y);
    for question in [
        // A direction of money that happened: cash that came in or went out, not the cash in hand.
        "آج گلے سے کتنا کیش نکالا",
        "zahid mehmood se kitna bakaya chukaya",
        // Words that exclude, place or give a habit: another question than the month's whole figure.
        "kele ke alawa is mahine kitni bikri hui",
        "is mahine bahar kitni bikri hui",
        "is mahine usually kitna commission banta hai",
        // A partner's earnings, takings and receipts are not what he has.
        "how much did partner b earn",
        "how much has partner b made",
        "partner b ko kitna milta hai",
        "partner b kitna le gaya",
        "partner b kitna nikalta hai",
        // "ho gayi" beside a collection or a payment is an event, not a balance.
        "zahid mehmood se kitni wasooli ho gayi",
        "rahim ko kitni adaigi ho gayi",
        "kitni adaigi ho gayi",
        // What he put in is not his capital balance.
        "how much did partner B put into the business",
        "how much did partner B invest in the business",
        // A bare "کل" with a past verb is yesterday's, and the doubt stands.
        "کل روکڑ کتنی تھی",
        // A day that no period took.
        "اس مہینے کی 5 کی بکری کتنی ہوئی",
        "this month on the 5th how much was sold",
        // Two buyers and two measures: not one figure each.
        "zahid mehmood aur rahim ne kitna maal liya aur kitne ka",
    ] {
        goes_to_the_model(&h, question);
    }
    // A town or a name that a made verb form looked like is searched as a name, never skipped.
    never_shows(
        &h,
        "manga ke gahkon ke zimme kitna",
        &[oracle(&f, "tiles.buyers_dues")],
    );
    // The first of the month in other words: this month so far.
    for question in [
        "pehli tarikh se aaj tak ka commission",
        "from the 1st of the month to today how much commission",
        "from the first till today how much commission",
    ] {
        instant(&h, question, &[oracle(&f, "a.month.total.total_comm")]);
    }
    // What stays a state: the verbs of being and having, and "ho gaya hai" beside what is still owed.
    let zahid = oracle_balance(&f, "buyer.zahid.balance");
    instant(&h, "how much does zahid mehmood owe", &[zahid]);
    instant(&h, "zahid mehmood ka kitna baqi ho gaya hai", &[zahid]);
    for question in [
        "partner b ke paas kitna hai",
        "how much does partner b have",
    ] {
        instant(&h, question, &[oracle(&f, "partner.partner_b.total")]);
    }
}

/// The number of rows of every table of the database, by table name.
fn row_counts(dir: &tempfile::TempDir) -> Vec<(String, i64)> {
    let conn = common::raw(dir);
    let mut names: Vec<String> = conn
        .prepare("SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%' ORDER BY name")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    names.sort();
    names
        .into_iter()
        .map(|name| {
            let n: i64 = conn
                .query_row(&format!("SELECT COUNT(*) FROM \"{name}\""), [], |r| {
                    r.get(0)
                })
                .unwrap();
            (name, n)
        })
        .collect()
}

#[test]
fn pre_reads_fill_the_first_request_and_keep_the_privacy_and_read_only_rules() {
    // Plan 59 D, Q7 and NFR-AI-6: the reads the app makes ahead of the model's first call for a question it
    // cannot answer whole are the tools themselves (`tool_result_json`, one read snapshot). On the worked day,
    // رحیم has a phone, an address and a note, and a چکتا buyer has a phone: none of them is in a request,
    // the first request holds tool results (the pre-reads), and nothing is written but the question log.
    let w = worked::worked_day();
    worked::events_1_to_5(&w);
    let h = Harness::over(w);
    h.runtime.set_read_first(true);
    h.runtime.set_local(true);
    h.runtime.set_instant(true);
    h.runtime.set_low_effort(true);
    let before = row_counts(&h.world.dir);
    let questions = [
        // A seller and one day: his day sheet is read ahead.
        "rahim ne aaj kitna nakad liya aur kitna baqi hai",
        // A buyer and one day.
        "akram ne aaj kitna nakad diya aur kitna udhaar raha",
        // Cash sales of an item, by name of the register.
        "aaj kele ki naqdi bikri kitni hui, kitne lines aur kitne ki",
    ];
    let mut with_reads = 0;
    for question in questions {
        let (_, requests) = h.ask(question, None, Vec::new());
        assert!(!requests.is_empty(), "{question}: the model was not asked");
        let tools = requests
            .first()
            .and_then(|r| r["messages"].as_array())
            .map_or(0, |m| m.iter().filter(|x| x["role"] == "tool").count());
        if tools > 0 {
            with_reads += 1;
        }
    }
    assert!(
        with_reads >= 2,
        "the first request of at least two of them holds the pre-reads, got {with_reads}"
    );
    let after = row_counts(&h.world.dir);
    for ((name, was), (_, now)) in before.iter().zip(after.iter()) {
        let expected = if name == "assistant_questions" {
            was + i64::try_from(questions.len()).unwrap()
        } else {
            *was
        };
        assert_eq!(*now, expected, "table {name}");
    }
    h.scan_requests();
}

/// The tools whose results the first request of a question carries (the pre-reads the app made).
fn first_request_tools(requests: &[serde_json::Value]) -> Vec<String> {
    requests
        .first()
        .and_then(|r| r["messages"].as_array())
        .map(|messages| {
            messages
                .iter()
                .filter(|m| m["role"] == "tool")
                .filter_map(|m| {
                    let body: serde_json::Value =
                        serde_json::from_str(m["content"].as_str()?).ok()?;
                    body["tool"].as_str().map(str::to_owned)
                })
                .collect()
        })
        .unwrap_or_default()
}

#[test]
fn money_review_pass_two_a_pre_read_never_puts_another_questions_figure_before_the_model() {
    // The scope guard does not see the pre-reads, so they stop wherever the plan stops. Each of these
    // questions would have had a whole-shop tile or a party's card read ahead for a question with a filter,
    // a payment type, an event or a capital flow the read lacks.
    let y = year::seed::build();
    let f = facts_of(&y);
    let h = production_harness(y);
    let tools_of = |question: &str| {
        let (_, requests) = h.ask(question, None, Vec::new());
        assert!(!requests.is_empty(), "{question}: the model was not asked");
        first_request_tools(&requests)
    };
    for question in [
        "aaj sabzi ki udhaar bikri aur commission kitna hua",
        "aaj kele ki naqdi bikri aur commission kitna hua",
        "is saal kitna sarmaya lagaya aur kitni bikri hui",
    ] {
        assert!(
            !tools_of(question).iter().any(|t| t == "tiles"),
            "{question}: the shop's tile was read ahead"
        );
    }
    // An unknown word that could be a filter, and an event ("li"), read no whole figure.
    // (A head word the books do not have is read by the plan itself, and the scope guard watches that read.)
    let unknown = tools_of("aaj commission aur kharcha zzqxv kitna hua");
    assert!(
        !unknown
            .iter()
            .any(|t| matches!(t.as_str(), "report" | "analytics" | "tiles" | "party_card")),
        "an unknown word: no figure is read ahead, got {unknown:?}"
    );
    assert!(
        !tools_of("rahim ne is mahine kitni peshgi li")
            .iter()
            .any(|t| t == "party_card"),
        "an event: his open advance is not read ahead"
    );
    // A buyers, sellers or partner word inside a trial balance question: the whole trial balance is not it.
    for question in [
        "trial balance me gahakon ka banaam kitna hai",
        "trial balance me byopariyon ka jama kitna hai",
        "ٹرائل بیلنس میں گاہکوں کا بنام کتنا ہے",
        // "kal" before the trial balance is yesterday's (or "kul" mistyped): the model's.
        "kal trial balance ka banaam kitna hai",
        "کل ٹرائل بیلنس کا بنام کتنا ہے",
    ] {
        never_shows(
            &h,
            question,
            &[
                oracle(&f, "r18.total.debit"),
                oracle(&f, "r18.total.credit"),
            ],
        );
        goes_to_the_model(&h, question);
    }
    // The dues of the buyers of a city are no seller's figure.
    for question in [
        "kasur ke byopari ka kul bakaya kitna hai",
        "قصور کے بیوپاری کا کل بقایا کتنا ہے",
    ] {
        never_shows(&h, question, &[oracle(&f, "d7.city.kasur.dues")]);
        goes_to_the_model(&h, question);
    }
    h.scan_requests();
}
