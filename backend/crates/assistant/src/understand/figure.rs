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

mod more;

use super::reads::{Line, Planned, What, Who};
use super::{FigureKind, PeriodKind, PeriodRef, Understanding, reads};
use crate::model_answer::{AnswerKind, ModelAnswer, Show, ShowAs};
use crate::results::{Node, Side, ToolResult, Val, Words};

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
    // "kab se", "kis kis ka", "kahan se": a question word asks a date, a person or a place, and a figure alone
    // would leave it out. The model answers (D7, plan 59 C).
    if u.open_ask || u.lower_ask || u.stray_screen_word || u.shop_after_and {
        return None;
    }
    // "gahakon ka bakaya aur byopariyon ko kitna dena hai": both sides are named and one figure would answer
    // for one side only.
    if u.buyer_word && u.seller_word && u.figures.value.len() < 2 {
        return None;
    }
    let planned = reads::plan(u)?;
    if planned.lines.is_empty() {
        return None;
    }
    // "how many sellers and the total": an instant answer must answer EVERY part asked. A count asked and
    // no line that gives a count (only the number of بکری is one) leaves a part out, so the model answers
    // (D7 O3, O4).
    // "How many units" is answered by the quantity (D7, plan 59 S-A7), unless it asks two things.
    let counted = |l: &Line| {
        l.what == What::SaleCount || (l.what == What::Qty && u.qty_asked && !u.how_much_twice)
    };
    if u.asks_count && !planned.lines.iter().any(counted) {
        return None;
    }
    // "kitne bill aur kitne gahak": two counts asked and one line that counts (the number of بکری) leaves a
    // part out (D7, plan 59 F3b).
    if u.count_asks > planned.lines.iter().filter(|l| counted(l)).count() {
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
    // Fewer figures than "how much" and "how many" cues leave a part of the question out ("kitna maal liya,
    // kitne ka, aur kitna commission" asks three). Every instant answer holds one planned line for each cue,
    // or the model answers: the lines are the asked figures, and a line may write more than one sentence (a
    // seller's commission is two), so the sentences are no count of what was asked (D7, plan 59 F3b, C).
    if u.how_much_cues > planned.lines.len() {
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
    // "naseer ka hisab": the balance, and the newest rows of his ledger under it. Two ledgers asked at once
    // are the model's.
    let show = if u.figures.value.as_slice() == [FigureKind::Ledger] {
        let ledgers: Vec<&&ToolResult> = results
            .iter()
            .filter(|r| r.tool == "report" && input_str(r, "id") == Some("R13"))
            .collect();
        match ledgers.as_slice() {
            [one] => vec![Show {
                result: one.id.clone(),
                as_: ShowAs::Table,
                rows: Some(i64::from(reads::LEDGER_ROWS)),
            }],
            _ => return None,
        }
    } else {
        Vec::new()
    };
    Some(FigureAnswer {
        answer: ModelAnswer {
            kind: AnswerKind::Answer,
            text,
            show,
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
        // The role a message takes as its default branch is one it has: the shop's for most, the item's for
        // the average rate.
        let default_role = if sentence.id == "assistant-instant-rate-average" {
            "item"
        } else {
            "shop"
        };
        if words.text_with(language, sentence.id, &probe) == text
            && args.get(at).map(|a| a.1) != Some(default_role)
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
                self.party(key, line.what)
            }
            (Who::Shop | Who::Item(_), What::Stock) => self.stock(&line.who).map(|s| vec![s]),
            (Who::Partner(key), What::PartnerTotal) => self.partners(key.as_deref()),
            (Who::Partner(Some(key)), What::PartnerCapital) => self.capital(key).map(|s| vec![s]),
            // A seller's commission is two figures, each labelled (Umar's ruling of 05-11-2026): the
            // commission charged to him, and the total commission on his goods.
            (Who::Seller(key), What::Commission) => self.seller_commission(key),
            // One named party and one item (D7, plan 59 S-A1, S-A2): the row of the item in view A read
            // for him, and the newest line of the register for the last rate.
            (Who::BuyerItem(buyer, item), What::Commission) => self
                .party_item(
                    "buyer",
                    buyer,
                    item,
                    "assistant-instant-commission-buyer-item",
                    "buyer_comm",
                )
                .map(|s| vec![s]),
            (Who::BuyerItem(buyer, item), What::Laga) => self
                .party_item(
                    "buyer",
                    buyer,
                    item,
                    "assistant-instant-laga-buyer-item",
                    "laga",
                )
                .map(|s| vec![s]),
            (Who::SellerItem(seller, item), What::Commission) => {
                self.seller_item_commission(seller, item)
            }
            (Who::Item(_) | Who::BuyerItem(..), What::LastRate) => {
                self.last_rate(&line.who).map(|s| vec![s])
            }
            (Who::Shop, What::Sale | What::SaleCount | What::Commission | What::Laga)
            | (Who::Item(_), What::Sale | What::Commission | What::Laga)
            | (Who::Seller(_), What::Sale | What::SaleCount)
            | (Who::Buyer(_), What::Sale | What::Commission | What::Laga) => {
                self.flow(&line.who, line.what).map(|s| vec![s])
            }
            // The figures read from the totals of a report or a view (D7, plan 59 F3).
            (Who::Group(group), What::Sale | What::Commission | What::Laga | What::Qty) => {
                self.group(group, line.what).map(|s| vec![s])
            }
            (_, What::Qty) => self.qty(&line.who).map(|s| vec![s]),
            (Who::Item(_) | Who::SellerItem(..), What::AvgRate) => {
                self.avg_rate(&line.who).map(|s| vec![s])
            }
            (Who::Shop | Who::Head(_), What::Expense) => self.expense(&line.who).map(|s| vec![s]),
            (Who::Shop | Who::Collector(_), What::Receipts) => {
                self.receipts(&line.who).map(|s| vec![s])
            }
            (Who::Shop, What::CashClosing) => self.cash_closing().map(|s| vec![s]),
            (Who::Shop, What::CashIn | What::CashOut) => self.cash_flow(line.what).map(|s| vec![s]),
            (Who::Shop, What::TrialDebit | What::TrialCredit) => {
                self.trial_total(line.what).map(|s| vec![s])
            }
            (Who::City(key), What::CityDues) => self.city_dues(key).map(|s| vec![s]),
            (Who::Shop, What::CashSales | What::CreditSales) => {
                self.sale_kind(line.what).map(|s| vec![s])
            }
            (Who::Seller(key), What::NetSale) => self.sale_net(key).map(|s| vec![s]),
            (Who::Shop | Who::Guests | Who::Partner(Some(_)), What::FruitValue) => {
                self.fruit(&line.who).map(|s| vec![s])
            }
            (Who::Shop, What::CapitalIn | What::CapitalOut) => {
                self.capital_flow(line.what).map(|s| vec![s])
            }
            (Who::Shop, _) => self.shop(line.what).map(|s| vec![s]),
            _ => None,
        }
    }

    /// A party's balance, payable or open advance, from his card. A seller with an open advance has two
    /// lines for his balance, each labelled: the ledger balance and the open advance (D7, plan 59 S-A10).
    fn party(&self, key: &str, what: What) -> Option<Vec<Sentence>> {
        let card = self.result("party_card", |r| str_at(r, &["party", "key"]) == Some(key))?;
        // An inactive party, one in the dead amount or one with a warning (over his limit ...) has
        // words of his own in the answer: the model writes it.
        let warned = matches!(card.node.get(&["warnings"]), Some(Node::List(w)) if !w.is_empty());
        if flag(card, "active") == Some(false) || flag(card, "dead") == Some(true) || warned {
            return None;
        }
        // A seller with an open advance has a ledger balance that is not what he is owed (his advance
        // is taken off it, and not at all when it was not deducted, E2E-28): the answer gives his ledger
        // balance and his open advance as two labelled lines, and the Owner reads which he means.
        let advance_open = matches!(
            card.node.get(&["open_advance"]),
            Some(Node::Val(Val::Money(m))) if m.paisa != Paisa::ZERO
        );
        // A seller's payable that is his whole credit balance is shown as the balance, so the answer
        // carries the side word (جمع, Cr) as the ledger does (live run 2, Q600).
        let credit_balance = matches!(
            (card.node.get(&["balance"]), card.node.get(&["payable"])),
            (Some(Node::Val(Val::Money(b))), Some(Node::Val(Val::Money(p))))
                if b.side == Some(Side::Cr) && b.paisa == p.paisa && b.paisa != Paisa::ZERO
        );
        let (kind, path): (&str, &[&str]) = match what {
            What::Payable if credit_balance => ("payable", &["balance"]),
            What::Payable => ("payable", &["payable"]),
            What::Advance => ("advance", &["open_advance"]),
            _ => ("balance", &["balance"]),
        };
        let name = reference(card, &["party", "name"])?;
        // His balance question with an advance open: ONE sentence that says how the three figures relate (the
        // ledger balance already includes the advance, so two bare lines would read as two debts): what is
        // payable now (left out when 0), the open advance, and the ledger balance (D7, plan 59 S-A10, money
        // review: the figures are cells of his card, nothing is computed).
        if what == What::Balance && advance_open {
            let payable_open = matches!(
                card.node.get(&["payable"]),
                Some(Node::Val(Val::Money(m))) if m.paisa != Paisa::ZERO
            );
            let mut args = vec![
                (
                    "kind",
                    if payable_open { "payable" } else { "none" }.to_owned(),
                ),
                ("name", name),
            ];
            if payable_open {
                args.push(("payable", reference(card, &["payable"])?));
            }
            args.push(("advance", reference(card, &["open_advance"])?));
            args.push(("balance", reference(card, &["balance"])?));
            return Some(vec![plain("assistant-instant-seller-advance", args)]);
        }
        Some(vec![plain(
            "assistant-instant-party",
            vec![
                ("kind", kind.to_owned()),
                ("name", name),
                ("figure", reference(card, path)?),
            ],
        )])
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
        // "kitna kamishan kata": the one taken off him, the seller side only (D7, plan 59 F3).
        let only_own = self.u.commission_side == Some(super::CommissionSide::Seller);
        ["seller_comm", "total_comm"]
            .iter()
            .zip(["seller-own", "seller"])
            .take(if only_own { 1 } else { 2 })
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

    /// One figure of a named party's row for one item in view A (the view read with that party and that
    /// item): `column` of the row of the item, or of the totals when the party sold none of it (the
    /// figure is 0). The sentence names the party, the item and the period.
    fn party_item(
        &self,
        field: &str,
        party: &str,
        item: &str,
        id: &'static str,
        column: &str,
    ) -> Option<Sentence> {
        // A figure of a record needs the period the question named (see `flow`).
        if self.u.periods.is_empty() {
            return None;
        }
        let view = self.result("analytics", |r| {
            input_str(r, "view") == Some("A")
                && input_str(r, field) == Some(party)
                && input_str(r, "item") == Some(item)
        })?;
        Some(Sentence {
            id,
            args: vec![
                ("name", self.name_of(party)?),
                ("item", self.name_of(item)?),
                ("figure", row_or_total(view, item, column)?),
            ],
            period: Some(Box::new(self.period()?)),
        })
    }

    /// A seller's commission on one item: the commission charged to him (`seller_comm`) and, when the
    /// question names no side, the total on his goods of that item (`total_comm`), each labelled, from
    /// the one row of view A (Umar's ruling of 05-11-2026, as `seller_commission` does for view B).
    fn seller_item_commission(&self, seller: &str, item: &str) -> Option<Vec<Sentence>> {
        let own = self.party_item(
            "seller",
            seller,
            item,
            "assistant-instant-commission-seller-item-own",
            "seller_comm",
        )?;
        if self.u.commission_side == Some(super::CommissionSide::Seller) {
            return Some(vec![own]);
        }
        let total = self.party_item(
            "seller",
            seller,
            item,
            "assistant-instant-commission-seller-item",
            "total_comm",
        )?;
        Some(vec![own, total])
    }

    /// The last rate of an item (to a buyer, when one is named): the rate and the date of the one row of
    /// the register read newest first. With no sale in the period, the newest sale BEFORE it (the second
    /// read of the plan) gives a sentence that says there was none and then the last rate with its date
    /// (Umar, 08-11-2026). With no sale before either, no sentence (the model says so).
    fn last_rate(&self, who: &Who) -> Option<Sentence> {
        let (buyer, item) = match who {
            Who::Item(item) => (None, item.as_str()),
            Who::BuyerItem(buyer, item) => (Some(buyer.as_str()), item.as_str()),
            _ => return None,
        };
        let first_day = self
            .u
            .periods
            .first()
            .and_then(|p| p.from)
            .map(|d| d.format("%Y-%m-%d").to_string());
        let own = |r: &ToolResult| {
            input_str(r, "id") == Some("R2")
                && input_str(r, "item") == Some(item)
                && input_str(r, "party") == buyer
        };
        // The read of the period itself: its `from` is the period's first day (or none, with no period).
        let primary = self.result("report", |r| {
            own(r) && input_str(r, "from") == first_day.as_deref()
        })?;
        if count_of(primary, &["count"])? >= 1 {
            let args = self.last_rate_args(primary, buyer, item)?;
            return Some(Sentence {
                id: if buyer.is_some() {
                    "assistant-instant-last-rate-buyer-item"
                } else {
                    "assistant-instant-last-rate-item"
                },
                args,
                period: Some(Box::new(self.period()?)),
            });
        }
        // No sale in the period: the read before its first day.
        let day_before = self
            .u
            .periods
            .first()
            .and_then(|p| p.from)
            .and_then(|d| d.pred_opt())
            .map(|d| d.format("%Y-%m-%d").to_string())?;
        let before = self.result("report", |r| {
            own(r)
                && input_str(r, "from").is_none()
                && input_str(r, "to") == Some(day_before.as_str())
        })?;
        if count_of(before, &["count"])? < 1 {
            return None;
        }
        let args = self.last_rate_args(before, buyer, item)?;
        Some(Sentence {
            id: if buyer.is_some() {
                "assistant-instant-last-rate-buyer-item-no-sale"
            } else {
                "assistant-instant-last-rate-item-no-sale"
            },
            args,
            period: Some(Box::new(self.period()?)),
        })
    }

    /// The arguments of a last-rate sentence from the one row of a register read: the buyer's name when one
    /// is named, the item, the rate and the date, each a reference to its cell.
    fn last_rate_args(
        &self,
        r2: &ToolResult,
        buyer: Option<&str>,
        item: &str,
    ) -> Option<Vec<(&'static str, String)>> {
        let Some(Node::Keyed(rows)) = r2.node.get(&["rows"]) else {
            return None;
        };
        let [(key, _)] = rows.as_slice() else {
            return None;
        };
        // The register names the packing only when the lines of the filter were sold in more than one:
        // the rate of an item in one packing is not that of another, and the sentence names none, so
        // the model says which (PRD FR-7.53; money review).
        if r2.node.get(&["rows", key, "packing"]).is_some() {
            return None;
        }
        // The rate CHARGED TO a named buyer ("Akram ko kele ka last rate") is the buyer's rate, which is
        // the register's `rate`. For an item alone, the shop or a seller it is the SELLER's rate (ریٹ
        // بیوپاری, the auction rate), unless the question says the buyer's: when some line of the filter
        // has a rate difference the register carries `seller_rate` too (D7, lead rulings). With no such
        // line the two are the same number, so `rate` is exact.
        let column = if buyer.is_none()
            && !self.u.buyer_rate
            && r2.node.get(&["rows", key, "seller_rate"]).is_some()
        {
            "seller_rate"
        } else {
            "rate"
        };
        let mut args = Vec::new();
        if let Some(buyer) = buyer {
            args.push(("name", self.name_of(buyer)?));
        }
        args.push(("item", self.name_of(item)?));
        args.push(("figure", reference(r2, &["rows", key, column])?));
        args.push(("date", reference(r2, &["rows", key, "date"])?));
        Some(args)
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
