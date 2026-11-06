//! The instant answer for a simple figure (PRD FR-7.90, plan 55 Section 10).
//!
//! When the app understood a question for sure and it asks for one figure (a party's balance,
//! payable or open advance; a sale, a commission or a laga of the shop, an item, a seller or a buyer
//! for a day or a period; cash in hand; the shop's dues or payables; goods left; a partner's کل رقم),
//! the reads of [`plan_reads`](super::plan_reads) already hold the figure. [`compose_figure`] writes
//! the answer from them with no model: one reviewed sentence for each figure, a Fluent message with
//! its arguments (`assistant-instant-*`), whose figures, names and dates are REFERENCES into the
//! results (`{{r2.balance}}`), never typed.
//!
//! The answer has the shape of the model's answer (a [`ModelAnswer`] of kind `answer`), so the caller
//! runs the very same checks (PRD FR-7.76: the two places of a figure, the side words, the words
//! around each reference) and fills it the same way (the figures with their ticks, the buttons from the
//! sources of the results read). Nothing is added, summed or rounded here (Code Standards Q5).
//!
//! A sentence whose words are missing from the Fluent files, a figure that is not in the reads, a
//! party with a warning (inactive, over his limit, in the dead amount) or two readings that disagree
//! give `None`, and the question goes on to the model.
//!
//! Example (worked day, after Events 1 to 5): "akram ka bakaya kitna hai" reads `find` (r1) and
//! `party_card` (r2); the answer is the sentence `assistant-instant-party` with kind `balance`, name
//! `{{r2.party.name}}` and figure `{{r2.balance}}`: "اکرم ٹول ٹیکس کا بقایا: 2,880 بنام ✓".

use mandi_services::core::auth::Language;
use mandi_services::core::money::Paisa;
use serde_json::Value;

use super::reads::{Line, Planned, What, Who};
use super::{FigureKind, PeriodKind, PeriodRef, Understanding, reads};
use crate::model_answer::{AnswerKind, ModelAnswer};
use crate::results::{Node, ToolResult, Val, Words};

/// The most partner rows one answer lists.
const MAX_PARTNER_LINES: usize = 6;

/// One reviewed sentence: a Fluent message of the panel and its arguments by name. The arguments are
/// plain words (a selector such as `shop`) or references (`{{r2.balance}}`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sentence {
    /// The Fluent message ID, for example `assistant-instant-party`.
    pub id: &'static str,
    /// Its arguments in order, for example `[("kind", "balance"), ("name", "{{r2.party.name}}")]`.
    /// The argument `period` is the words of [`Sentence::period`], already written.
    pub args: Vec<(&'static str, String)>,
    /// For a sale, a commission or a laga: the sentence that names the period
    /// (`assistant-instant-period`), written into the argument `period`.
    pub period: Option<Box<Sentence>>,
}

/// An instant answer: the answer the checks and the filling read, and the sentences it was made of.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FigureAnswer {
    /// Kind `answer`; the text is the sentences with their references; no table, no next question.
    pub answer: ModelAnswer,
    /// The sentences, one for each figure, in the order of the answer.
    pub sentences: Vec<Sentence>,
}

/// The instant answer of a question, from the results of its planned reads, or `None` when the
/// question is not one the app answers by itself (see the module documentation).
///
/// `reads` are the results of [`plan_reads`](super::plan_reads) in the shape the model would see them,
/// with the ids the caller's state gave them.
pub fn compose_figure(
    u: &Understanding,
    reads: &[ToolResult],
    language: Language,
    words: &Words,
) -> Option<FigureAnswer> {
    // "dono ka mila kar kitna hai": the Owner wants the SUM of several things. A list of the parts is
    // an incomplete answer to that, and the app adds nothing (Code Standards Q5): the model has the
    // calculator. One thing's own total ("kul bakaya", "is saal ki kul bikri") is not a sum.
    if u.sum_asked
        && u.parties.len().saturating_add(u.items.len()) > 1
        && !u.figures.value.contains(&FigureKind::PartnerTotal)
    {
        return None;
    }
    let planned = reads::plan(u)?;
    if planned.lines.is_empty() {
        return None;
    }
    let results = results_of(&planned, reads)?;
    let cx = Cx {
        u,
        results: &results,
        all: reads,
    };
    let mut sentences: Vec<Sentence> = Vec::new();
    for line in &planned.lines {
        sentences.extend(cx.sentences_of(line)?);
    }
    if sentences.is_empty() {
        return None;
    }
    // "mila kar", "dono ka": the sum is asked, and a list of the parts has no total.
    if u.sum_strong && sentences.len() > 1 {
        return None;
    }
    // Each sentence is written by the panel's own words; one the files do not hold is no answer.
    let mut lines: Vec<String> = Vec::new();
    for sentence in &mut sentences {
        lines.push(write(words, language, sentence)?);
    }
    let text = if lines.len() == 1 {
        lines.join("")
    } else {
        lines
            .iter()
            .map(|l| format!("• {l}"))
            .collect::<Vec<_>>()
            .join("\n")
    };
    Some(FigureAnswer {
        answer: ModelAnswer {
            kind: AnswerKind::Answer,
            text,
            show: Vec::new(),
            next: Vec::new(),
            choose: None,
            entry: None,
            nearest: None,
        },
        sentences,
    })
}

/// The Fluent message with its arguments, the period first when the sentence has one. `None` when a
/// message is not in the files (`Words` gives the id back for a missing message).
fn write(words: &Words, language: Language, sentence: &mut Sentence) -> Option<String> {
    if let Some(period) = sentence.period.as_deref_mut() {
        let text = write(words, language, period)?;
        sentence.args.retain(|(name, _)| *name != "period");
        sentence.args.push(("period", text));
    }
    let args: Vec<(&str, &str)> = sentence
        .args
        .iter()
        .map(|(name, value)| (*name, value.as_str()))
        .collect();
    let text = words.text_with(language, sentence.id, &args);
    // A role the Fluent file does not have falls to its default line, which drops the name: compare
    // with a role nobody has, and refuse a sentence that reads the same.
    if let Some(at) = args.iter().position(|(name, _)| *name == "role") {
        let mut probe = args.clone();
        if let Some(slot) = probe.get_mut(at) {
            slot.1 = "no-such-role";
        }
        if words.text_with(language, sentence.id, &probe) == text
            && args.get(at).map(|a| a.1) != Some("shop")
        {
            return None;
        }
    }
    (text != sentence.id && !text.trim().is_empty()).then_some(text)
}

// ---- the results of the planned reads ----------------------------------------------------------------------

/// True when `got` holds every field `want` sets, and has none that `want` leaves empty.
fn holds(want: &Value, got: &Value) -> bool {
    match (want, got) {
        // A field the plan left empty must be empty in the read too: a result with an extra filter
        // (an item, a seller, other dates) is not the read that was planned.
        (Value::Object(w), Value::Object(g)) => w.iter().all(|(k, v)| {
            if v.is_null() {
                g.get(k).is_none_or(Value::is_null)
            } else {
                g.get(k).is_some_and(|x| holds(v, x))
            }
        }),
        (w, g) => w == g,
    }
}

/// The result of each planned data read, in the order of the plan. `None` when one has no result: a
/// read the tool refused (a locked partner section, a filter it did not take).
fn results_of<'a>(planned: &Planned, reads: &'a [ToolResult]) -> Option<Vec<&'a ToolResult>> {
    let mut used: Vec<&str> = Vec::new();
    let mut out = Vec::new();
    for read in &planned.reads {
        let found = reads.iter().find(|r| {
            !used.contains(&r.id.as_str()) && r.tool == read.tool && holds(&read.input, &r.input)
        })?;
        used.push(found.id.as_str());
        out.push(found);
    }
    Some(out)
}

// ---- reading a result ----------------------------------------------------------------------------------------

/// The reference to a value of a result, when the path ends on one.
fn reference(result: &ToolResult, path: &[&str]) -> Option<String> {
    matches!(result.node.get(path), Some(Node::Val(_))).then(|| {
        let mut text = result.id.clone();
        for step in path {
            text.push('.');
            text.push_str(step);
        }
        format!("{{{{{text}}}}}")
    })
}

/// A money value in whole rupees, or a count, at a path.
fn whole(result: &ToolResult, path: &[&str]) -> Option<i64> {
    match result.node.get(path) {
        Some(Node::Val(Val::Money(m))) => Some(m.paisa.rupees()),
        Some(Node::Val(Val::Count(n))) => Some(*n),
        _ => None,
    }
}

fn flag(result: &ToolResult, key: &str) -> Option<bool> {
    match result.node.get(&[key]) {
        Some(Node::Bool(b)) => Some(*b),
        _ => None,
    }
}

fn count_of(result: &ToolResult, path: &[&str]) -> Option<i64> {
    match result.node.get(path) {
        Some(Node::Val(Val::Count(n))) => Some(*n),
        _ => None,
    }
}

fn str_at<'a>(result: &'a ToolResult, path: &[&str]) -> Option<&'a str> {
    match result.node.get(path) {
        Some(Node::Str(s)) => Some(s.as_str()),
        _ => None,
    }
}

fn input_str<'a>(result: &'a ToolResult, field: &str) -> Option<&'a str> {
    result.input.get(field).and_then(Value::as_str)
}

fn plain(id: &'static str, args: Vec<(&'static str, String)>) -> Sentence {
    Sentence {
        id,
        args,
        period: None,
    }
}

fn is_one_day(p: &PeriodRef) -> bool {
    p.from == Some(p.to)
}

/// The reading context.
struct Cx<'a> {
    u: &'a Understanding,
    /// The result of each planned data read.
    results: &'a [&'a ToolResult],
    /// Every result of the question (the finds hold the names, the period read the dates).
    all: &'a [ToolResult],
}

impl Cx<'_> {
    /// The data result of a tool for which `ok` holds.
    fn result(&self, tool: &str, ok: impl Fn(&ToolResult) -> bool) -> Option<&ToolResult> {
        self.results
            .iter()
            .copied()
            .find(|r| r.tool == tool && ok(r))
    }

    fn report(&self, id: &str) -> Option<&ToolResult> {
        self.result("report", |r| input_str(r, "id") == Some(id))
    }

    fn tiles(&self) -> Option<&ToolResult> {
        self.result("tiles", |_| true)
    }

    /// A view read for one record (`field` is `item`, `seller` or `buyer`), or for none of them.
    fn analytics(&self, view: &str, field: &str, key: Option<&str>) -> Option<&ToolResult> {
        self.result("analytics", |r| {
            input_str(r, "view") == Some(view)
                && match key {
                    Some(k) => input_str(r, field) == Some(k),
                    None => ["item", "seller", "buyer"]
                        .iter()
                        .all(|f| input_str(r, f).is_none()),
                }
        })
    }

    /// The name of a record the `find` of this question showed, as a reference.
    fn name_of(&self, key: &str) -> Option<String> {
        self.all.iter().filter(|r| r.tool == "find").find_map(|r| {
            let Some(Node::List(matches)) = r.node.get(&["matches"]) else {
                return None;
            };
            matches.iter().enumerate().find_map(|(at, m)| {
                (matches!(m.get(&["key"]), Some(Node::Str(k)) if k == key))
                    .then(|| reference(r, &["matches", &at.to_string(), "name"]))
                    .flatten()
            })
        })
    }

    /// The sentence that names the period: `today`, a `day`, a `range`, or `all` when the question
    /// named none (the books so far).
    fn period(&self) -> Option<Sentence> {
        let shape = |shape: &str, args: Vec<(&'static str, String)>| {
            let mut all = vec![("shape", shape.to_owned())];
            all.extend(args);
            plain("assistant-instant-period", all)
        };
        let Some(p) = self.u.periods.first() else {
            return Some(shape("all", Vec::new()));
        };
        if p.kind == PeriodKind::Today {
            return Some(shape("today", Vec::new()));
        }
        let read = self.all.iter().find(|r| r.tool == "period")?;
        let from = reference(read, &["from"])?;
        let to = reference(read, &["to"])?;
        Some(if is_one_day(p) {
            shape("day", vec![("day", from)])
        } else {
            shape("range", vec![("from", from), ("to", to)])
        })
    }

    /// The sentences of one line of the plan. `None` when the figure is not in the reads.
    fn sentences_of(&self, line: &Line) -> Option<Vec<Sentence>> {
        match (&line.who, line.what) {
            (Who::Party(key), What::Balance | What::Payable | What::Advance) => {
                self.party(key, line.what).map(|s| vec![s])
            }
            (Who::Shop | Who::Item(_), What::Stock) => self.stock(&line.who).map(|s| vec![s]),
            (Who::Partner(key), What::PartnerTotal) => self.partners(key.as_deref()),
            (Who::Partner(Some(key)), What::PartnerCapital) => self.capital(key).map(|s| vec![s]),
            // A seller's commission is two figures, each labelled (Umar's ruling of 05-11-2026): the
            // commission charged to him, and the total commission on his goods.
            (Who::Seller(key), What::Commission) => self.seller_commission(key),
            (Who::Shop, What::Sale | What::SaleCount | What::Commission | What::Laga)
            | (Who::Item(_), What::Sale | What::Commission | What::Laga)
            | (Who::Seller(_), What::Sale | What::SaleCount)
            | (Who::Buyer(_), What::Sale | What::Commission | What::Laga) => {
                self.flow(&line.who, line.what).map(|s| vec![s])
            }
            (Who::Shop, _) => self.shop(line.what).map(|s| vec![s]),
            _ => None,
        }
    }

    /// A party's balance, payable or open advance, from his card.
    fn party(&self, key: &str, what: What) -> Option<Sentence> {
        let card = self.result("party_card", |r| str_at(r, &["party", "key"]) == Some(key))?;
        // An inactive party, one in the dead amount or one with a warning (over his limit ...) has
        // words of his own in the answer: the model writes it.
        let warned = matches!(card.node.get(&["warnings"]), Some(Node::List(w)) if !w.is_empty());
        if flag(card, "active") == Some(false) || flag(card, "dead") == Some(true) || warned {
            return None;
        }
        // A seller with an open advance has a ledger balance that is not what he is owed (his advance
        // is taken off it, and not at all when it was not deducted, E2E-28): "his balance" is the
        // model's, and it says which of the two it means.
        if what == What::Balance
            && matches!(
                card.node.get(&["open_advance"]),
                Some(Node::Val(Val::Money(m))) if m.paisa != Paisa::ZERO
            )
        {
            return None;
        }
        let (kind, path): (&str, &[&str]) = match what {
            What::Payable => ("payable", &["payable"]),
            What::Advance => ("advance", &["open_advance"]),
            _ => ("balance", &["balance"]),
        };
        Some(plain(
            "assistant-instant-party",
            vec![
                ("kind", kind.to_owned()),
                ("name", reference(card, &["party", "name"])?),
                ("figure", reference(card, path)?),
            ],
        ))
    }

    /// The shop's figures of the tiles and of R24.
    fn shop(&self, what: What) -> Option<Sentence> {
        let (kind, source, path): (&str, Option<&ToolResult>, &[&str]) = match what {
            What::Cash => ("cash", self.tiles(), &["cash_in_hand", "amount"]),
            What::BuyersDues => ("buyers-dues", self.tiles(), &["buyers_dues", "amount"]),
            What::TotalDues => ("total-dues", self.tiles(), &["total_dues", "amount"]),
            What::AdvancesOpen => ("advances", self.tiles(), &["advances_open", "amount"]),
            What::DeadAmount => ("dead", self.tiles(), &["dead_amount", "amount"]),
            What::MoneyLent => ("lent", self.tiles(), &["money_lent_open", "amount"]),
            What::ShopPayable => ("payable", self.report("R24"), &["totals", "payable"]),
            _ => return None,
        };
        Some(plain(
            "assistant-instant-shop",
            vec![
                ("kind", kind.to_owned()),
                ("figure", reference(source?, path)?),
            ],
        ))
    }

    /// Goods left: the nag and the lots over the whole filter of R10. R10 also lists trip purchases
    /// and garden arrivals (PRD FR-12.25), which belong to the trip's and the garden's partners: the
    /// shop's answer is given only when the dashboard tile of the lots agrees with it, and an item's
    /// only when none of its goods is for another shop.
    fn stock(&self, who: &Who) -> Option<Sentence> {
        let item = match who {
            Who::Item(key) => Some(key.as_str()),
            _ => None,
        };
        let r10 = self.result("report", |r| {
            input_str(r, "id") == Some("R10") && input_str(r, "item") == item
        })?;
        match item {
            None => {
                let tile = self.tiles()?;
                if whole(r10, &["totals", "balance"])? != whole(tile, &["stock", "qty"])?
                    || whole(r10, &["count"])? != whole(tile, &["stock", "lots"])?
                {
                    return None;
                }
            }
            Some(_) => {
                if whole(r10, &["totals", "other_left"])? != 0 {
                    return None;
                }
            }
        }
        let mut args = vec![(
            "role",
            if item.is_some() { "item" } else { "shop" }.to_owned(),
        )];
        if let Some(key) = item {
            args.push(("name", self.name_of(key)?));
        }
        args.push(("qty", reference(r10, &["totals", "balance"])?));
        args.push(("lots", reference(r10, &["count"])?));
        Some(plain("assistant-instant-stock", args))
    }

    /// A partner's kul raqam, or every partner's, from the rows of R31.
    fn partners(&self, key: Option<&str>) -> Option<Vec<Sentence>> {
        let r31 = self.report("R31")?;
        let Some(Node::Keyed(rows)) = r31.node.get(&["rows"]) else {
            return None;
        };
        let keys: Vec<&str> = match key {
            Some(k) => vec![k],
            None => rows.iter().map(|(k, _)| k.as_str()).collect(),
        };
        if keys.is_empty() || keys.len() > MAX_PARTNER_LINES {
            return None;
        }
        keys.into_iter()
            .map(|k| {
                Some(plain(
                    "assistant-instant-partner-total",
                    vec![
                        ("name", reference(r31, &["rows", k, "name"])?),
                        ("figure", reference(r31, &["rows", k, "total"])?),
                    ],
                ))
            })
            .collect()
    }

    /// A partner's capital now: the balance of the capital part of R31 for him, read in two places
    /// (the capital history and the partner overview).
    fn capital(&self, key: &str) -> Option<Sentence> {
        let r31 = self.result("report", |r| {
            input_str(r, "id") == Some("R31") && input_str(r, "partner") == Some(key)
        })?;
        // A minus capital is a debit balance (more taken back than put in): the model says so.
        if whole(r31, &["capital", "balance"])? < 0 {
            return None;
        }
        Some(plain(
            "assistant-instant-partner-capital",
            vec![
                ("name", self.name_of(key)?),
                ("figure", reference(r31, &["capital", "balance"])?),
            ],
        ))
    }

    /// A seller's commission, in two lines: the commission charged to him (the seller side of view B)
    /// and the total commission on his goods (both sides and Laga, `total_comm`). Both come from the
    /// one read of view B with him, in the period the question named.
    fn seller_commission(&self, key: &str) -> Option<Vec<Sentence>> {
        if self.u.periods.is_empty() {
            return None;
        }
        let view = self.analytics("B", "seller", Some(key))?;
        let name = self.name_of(key)?;
        ["seller_comm", "total_comm"]
            .iter()
            .zip(["seller-own", "seller"])
            .map(|(column, role)| {
                Some(Sentence {
                    id: "assistant-instant-commission",
                    args: vec![
                        ("role", role.to_owned()),
                        ("name", name.clone()),
                        ("figure", row_or_total(view, key, column)?),
                    ],
                    period: Some(Box::new(self.period()?)),
                })
            })
            .collect()
    }

    /// A sale, a count, a commission or a laga of the shop, an item, a seller or a buyer.
    fn flow(&self, who: &Who, what: What) -> Option<Sentence> {
        // A record's figure needs the period the question named: with none, "so far" would be the
        // app's guess.
        if !matches!(who, Who::Shop) && self.u.periods.is_empty() {
            return None;
        }
        let (id, column) = match what {
            What::Sale => ("assistant-instant-sale", "gross"),
            What::SaleCount => ("assistant-instant-sale-count", "bikri_count"),
            What::Commission => ("assistant-instant-commission", "total_comm"),
            What::Laga => ("assistant-instant-laga", "laga"),
            _ => return None,
        };
        let (role, key, figure) = match who {
            Who::Shop => ("shop", None, self.shop_flow(what)?),
            Who::Item(key) => (
                "item",
                Some(key.as_str()),
                row_or_total(self.analytics("A", "item", Some(key))?, key, column)?,
            ),
            Who::Seller(key) => (
                "seller",
                Some(key.as_str()),
                row_or_total(self.analytics("B", "seller", Some(key))?, key, column)?,
            ),
            Who::Buyer(key) => {
                // A buyer's view has its own columns for the same figures.
                let column = match what {
                    What::Sale => "purchase_value",
                    What::Commission => "buyer_comm",
                    _ => column,
                };
                (
                    "buyer",
                    Some(key.as_str()),
                    row_or_total(self.analytics("C", "buyer", Some(key))?, key, column)?,
                )
            }
            _ => return None,
        };
        let mut args = vec![("role", role.to_owned())];
        if let Some(key) = key {
            args.push(("name", self.name_of(key)?));
        }
        args.push(("figure", figure));
        Some(Sentence {
            id,
            args,
            period: Some(Box::new(self.period()?)),
        })
    }

    /// The shop's figure: the tile for today, else the totals of view A (view B for the count).
    fn shop_flow(&self, what: What) -> Option<String> {
        let today = self
            .u
            .periods
            .first()
            .is_some_and(|p| p.kind == PeriodKind::Today);
        if today && what != What::Laga {
            let path: &[&str] = match what {
                What::Sale => &["today_sales", "amount"],
                What::SaleCount => &["today_sales", "count"],
                _ => &["today_commission", "amount"],
            };
            return reference(self.tiles()?, path);
        }
        let (view, column) = match what {
            What::Sale => ("A", "gross"),
            What::SaleCount => ("B", "bikri_count"),
            What::Commission => ("A", "total_comm"),
            _ => ("A", "laga"),
        };
        let read = self.analytics(view, "", None)?;
        // One day: the day summary holds the same figure; two readings that differ give no answer.
        if self.u.periods.first().is_some_and(is_one_day) {
            let (row, cell) = match what {
                What::Sale => ("x1", "amount"),
                What::SaleCount => ("x1", "count"),
                What::Commission => ("x6", "amount"),
                _ => ("x5", "amount"),
            };
            let sheet = self.report("R34")?;
            let here = whole(read, &["totals", column])?;
            let there = whole(sheet, &["rows", row, cell])?;
            if here != there {
                return None;
            }
        }
        reference(read, &["totals", column])
    }
}

/// The figure of a record in a view read for that record alone: its row when the view has one, else
/// the totals (the view is empty, so every figure is 0). More than one row is no answer.
fn row_or_total(view: &ToolResult, key: &str, column: &str) -> Option<String> {
    if count_of(view, &["count"]).is_none_or(|n| n > 1) {
        return None;
    }
    match view.node.get(&["rows"]) {
        Some(Node::Keyed(rows)) if rows.is_empty() => reference(view, &["totals", column]),
        Some(Node::List(rows)) if rows.is_empty() => reference(view, &["totals", column]),
        Some(Node::Keyed(_)) => reference(view, &["rows", key, column]),
        _ => None,
    }
}
