//! A dump of what the app UNDERSTOOD for every question of a bank (plan 59 R2, offline, no model, no key).
//!
//! `run.mjs --offline` says which questions need the model. This test says WHY: for each question it runs the
//! real `understand` and `plan_reads` on the rich year and writes one line with the intent, the `sure` flag, the
//! figures, the doubts, what each name led to, the period, the flags that switch the instant plan off, and
//! whether any read was planned. It changes nothing and writes only the file named by `MANDI_GAP_OUT`.
//!
//! It does nothing unless `MANDI_GAP_BANK` (a bank tsv) and `MANDI_GAP_OUT` (the output tsv) are both set, so a
//! plain `cargo test` skips the slow build of the rich year.
//!
//!     MANDI_GAP_BANK=scripts/ai-eval/realtime-d7.tsv MANDI_GAP_OUT=/some/dir/gap.tsv \
//!       cargo test -j 4 -p mandi-assistant --test robustness_gap -- --nocapture
#![cfg(test)]

#[path = "../../services/tests/common/mod.rs"]
mod common;
#[path = "support/worked.rs"]
mod worked;
#[allow(dead_code)]
#[path = "../../services/tests/bikri_preview/support.rs"]
mod world;
#[path = "robustness_year/mod.rs"]
mod year;

use mandi_assistant::understand::{
    Doubt, Measure, Outcome, Reader, Understanding, plan_reads, understand,
};
use mandi_services::core::auth::Language;
use mandi_services::core::dates::BusinessDate;

/// The question column and the id column of a bank file.
fn rows(path: &str) -> Vec<(String, String)> {
    let text = std::fs::read_to_string(path).expect("the bank file");
    let mut lines = text.lines();
    let head: Vec<&str> = lines.next().expect("a header").split('\t').collect();
    let id = head.iter().position(|h| *h == "id").expect("id");
    let q = head
        .iter()
        .position(|h| *h == "question")
        .expect("question");
    lines
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            let c: Vec<&str> = l.split('\t').collect();
            (c[id].to_owned(), c[q].to_owned())
        })
        .collect()
}

/// The outcome of a name in one word.
fn outcome_word<T>(o: &Outcome<T>) -> &'static str {
    match o {
        Outcome::Found(_) => "found",
        Outcome::FoundAs(_) => "found_as_other_role",
        Outcome::Several(_) => "several",
        Outcome::NotFound => "not_found",
    }
}

/// The flags of an understanding that are set, as words.
fn flags(u: &Understanding) -> Vec<&'static str> {
    let mut out = Vec::new();
    let mut push = |on: bool, name: &'static str| {
        if on {
            out.push(name);
        }
    };
    push(u.two_figures, "two_figures");
    push(u.sum_asked, "sum_asked");
    push(u.sum_strong, "sum_strong");
    push(u.compare, "compare");
    push(u.scoped, "scoped");
    push(u.amount_named, "amount_named");
    push(u.item_group.is_some(), "item_group");
    push(u.packing_filter, "packing_filter");
    push(u.document_number, "document_number");
    push(u.bikri_number.is_some(), "bikri_number");
    push(u.truck.is_some(), "truck");
    push(u.guests, "guests");
    push(u.quality, "quality");
    push(u.capital_flow.is_some(), "capital_flow");
    push(u.garden_goods, "garden_goods");
    push(u.last_one, "last_one");
    push(u.months_asked, "months_asked");
    push(u.oldest, "oldest");
    push(u.happened, "happened");
    push(u.asks_count, "asks_count");
    push(u.buyer_rate, "buyer_rate");
    push(u.qty_asked, "qty_asked");
    push(u.average_asked, "average_asked");
    push(u.pay_asked, "pay_asked");
    push(u.pay_kind.is_some(), "pay_kind");
    push(u.unexplained, "unexplained");
    push(u.role_word, "role_word");
    push(u.how_much_twice, "how_much_twice");
    push(u.value_asked, "value_asked");
    push(u.total_dues, "total_dues");
    push(u.commission_side.is_some(), "commission_side");
    push(u.list.is_some(), "list");
    push(u.other_measure == Measure::Quantity, "measure_quantity");
    push(u.other_measure == Measure::Breakdown, "measure_breakdown");
    push(u.other_measure == Measure::Other, "measure_other");
    out
}

/// The words of a question whose removal clears `is_set` (a flag or a doubt that stands on the whole question).
/// It tries every single word, then every pair, and gives the words of the first size that works. The words
/// that "explain" the flag are the culprits; removing a word can change how the rest is read, so this is a
/// guide to the wording, not a proof.
fn culprits(
    reader: &Reader<'_>,
    question: &str,
    is_set: &dyn Fn(&Understanding) -> bool,
) -> Vec<String> {
    let words: Vec<&str> = question.split_whitespace().collect();
    let without = |skip: &[usize]| -> String {
        words
            .iter()
            .enumerate()
            .filter(|(i, _)| !skip.contains(i))
            .map(|(_, w)| *w)
            .collect::<Vec<_>>()
            .join(" ")
    };
    let mut found: Vec<String> = Vec::new();
    for (i, word) in words.iter().enumerate() {
        if !is_set(&understand(reader, &without(&[i]))) {
            found.push((*word).to_owned());
        }
    }
    if !found.is_empty() {
        return found;
    }
    for (i, first) in words.iter().enumerate() {
        for (j, second) in words.iter().enumerate().skip(i + 1) {
            if !is_set(&understand(reader, &without(&[i, j]))) {
                found.push(format!("{first}+{second}"));
            }
        }
    }
    found
}

#[test]
fn dump_what_the_app_understood() {
    let (Ok(bank), Ok(out)) = (
        std::env::var("MANDI_GAP_BANK"),
        std::env::var("MANDI_GAP_OUT"),
    ) else {
        return;
    };
    let y = year::seed::build();
    // The year's own date, as `robustness_understand.rs` (the offline run sets the same day).
    let today = BusinessDate::from_ymd(2026, 11, 5).expect("today");
    let reader = Reader {
        ctx: &y.w.env.ctx,
        actor: &y.w.owner,
        language: Language::Ur,
        today,
        partner_locked: false,
    };
    let mut lines = vec![
        "id\tintent\tintent_conf\tsure\tfigures\tdoubts\tparties\titems\trecords\tperiods\tflags\tplanned\treads\tunexplained_words\ttime_words"
            .to_owned(),
    ];
    for (id, question) in rows(&bank) {
        let u = understand(&reader, &question);
        let plan = plan_reads(&u);
        let parties: Vec<String> = u
            .parties
            .iter()
            .map(|p| format!("{}:{}", p.text, outcome_word(&p.outcome)))
            .collect();
        let items: Vec<String> = u
            .items
            .iter()
            .map(|i| format!("{}:{}", i.text, outcome_word(&i.outcome)))
            .collect();
        let records: Vec<String> = u
            .records
            .iter()
            .map(|r| format!("{}:{}", r.text, outcome_word(&r.outcome)))
            .collect();
        let periods: Vec<String> = u
            .periods
            .iter()
            .map(|p| format!("{:?}({})", p.kind, p.words))
            .collect();
        let reads = plan
            .as_ref()
            .map(|reads| {
                reads
                    .iter()
                    .map(|r| {
                        let which = r
                            .input
                            .get("id")
                            .or_else(|| r.input.get("view"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
                        format!(
                            "{}{}{}",
                            r.tool,
                            if which.is_empty() { "" } else { ":" },
                            which
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(",")
            })
            .unwrap_or_default();
        let unexplained = if u.unexplained {
            culprits(&reader, &question, &|x| x.unexplained).join(" ")
        } else {
            String::new()
        };
        let time_words = if u.doubts.contains(&Doubt::TimeWord) {
            culprits(&reader, &question, &|x| x.doubts.contains(&Doubt::TimeWord)).join(" ")
        } else {
            String::new()
        };
        let fields = [
            id,
            format!("{:?}", u.intent.value),
            format!("{:?}", u.intent.confidence),
            u.sure.to_string(),
            format!("{:?}", u.figures.value),
            format!("{:?}", u.doubts),
            parties.join("|"),
            items.join("|"),
            records.join("|"),
            periods.join("|"),
            flags(&u).join(","),
            plan.is_some().to_string(),
            reads,
            unexplained,
            time_words,
        ];
        lines.push(fields.join("\t"));
    }
    std::fs::write(&out, lines.join("\n") + "\n").expect("the output file");
    println!("wrote {} lines to {out}", lines.len() - 1);
}
