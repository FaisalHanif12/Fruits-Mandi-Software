//! The checks an answer must pass before it is shown (PRD FR-7.76, D6-system-prompt Section 4 and 6,
//! Code Standards Q4, Q6).
//!
//! The model writes words and references (`{{r2.balance}}`), never figures. This file checks the
//! words around the references and the references themselves:
//!
//! | Rule | What is refused |
//! |---|---|
//! | `digit` | a digit of any script outside a reference |
//! | `number_word` | a number word of the list or of `number_words.rs` (`دو ہزار`, `اٹھائیس`, `ڈھائی لاکھ`, `lakh`) |
//! | `month` | a month name |
//! | `name` | a name from this question's results, or any account or partner name of the books (`NameBook`), typed outside a reference |
//! | `check_mark` | a typed ✓ |
//! | `reference` | a path that does not exist, or one that does not end on a value |
//! | `earlier_question` | a path into a result of an earlier question |
//! | `side_words` | the words of the other side next to a balance |
//! | `script` | Latin letters in an Urdu answer, Urdu letters in an English one |
//! | `never_word` | a word of the never-use list |
//! | `too_long` | more than 130 words, each filled reference counted as one |
//! | `estimate_kind` | an estimate figure in an answer that is not of kind estimate or advice |
//! | `asked` | the question asks for the commission and the sales, and the answer gives only one of them |
//! | `choose`, `entry`, `show`, `next` | the parts of the answer schema that point at results |
//!
//! A figure whose two places differ is not a failure to rewrite: it is final (`Verdict::Differ`).
//! The word lists are read from `prompt/words.tsv` (Code Standards Q11); the number words add the
//! list of `number_words.rs`, every Urdu number word 1 to 99 and the Roman Urdu ones, which
//! `words.tsv` lacks (lead decision after robustness finding attack_03).
//!
//! Example: "اکرم کے ذمے 2,880 باقی ہیں" fails with `digit` at `2,880`; "{{r1.matches.0.name}} کے
//! ذمے {{r2.balance}} باقی ہیں" passes when `r2.balance` is a بنام balance.

use mandi_services::core::auth::Language;
use mandi_services::core::text;

use crate::limits::{MAX_ANSWER_WORDS, MAX_NEXT, MAX_SHOW, MAX_TABLE_ROWS};
use crate::model_answer::{AnswerKind, ModelAnswer, Piece, ShowAs, pieces};
use crate::prompt::{SYSTEM, WordKind, words};
use crate::results::{NameVal, Node, Shown, Side, Val};
use crate::tools::{Env, Found, State, ToolError, cues, lookup};

/// The columns that are a commission figure, and those that are a sales figure (analytics A to C,
/// reports R2 and R6).
const COMMISSION_COLUMNS: &[&str] = &["total_comm", "seller_comm", "buyer_comm", "commission"];
const SALES_COLUMNS: &[&str] = &["gross", "sales_money", "purchase_value", "seller_amount"];

/// One thing that failed, in the words the model reads for its one rewrite.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Failure {
    /// The rule.
    pub rule: &'static str,
    /// The text or path it is about.
    pub at: String,
    /// For the side words: the reference of the balance.
    pub figure: Option<String>,
}

impl Failure {
    fn new(rule: &'static str, at: &str) -> Self {
        Self {
            rule,
            at: at.to_owned(),
            figure: None,
        }
    }

    /// The object of the `check_failed` message (D6-system-prompt 5.3).
    pub fn to_json(&self) -> serde_json::Value {
        match &self.figure {
            Some(figure) => serde_json::json!({"rule": self.rule, "at": self.at, "figure": figure}),
            None => serde_json::json!({"rule": self.rule, "at": self.at}),
        }
    }
}

/// What the check decided.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verdict {
    /// The answer may be filled and shown.
    Ok,
    /// Fix exactly these points and write the answer again.
    Retry(Vec<Failure>),
    /// A figure was cited whose two places differ: final, no rewrite (PRD FR-7.76 point 2). The
    /// paths are the references to those figures.
    Differ(Vec<String>),
}

/// The names of this question's results, as phrases normalised for matching.
#[derive(Debug, Default)]
pub struct NameSet {
    phrases: Vec<String>,
    /// Every account and partner name of the books, when the check should refuse a name that is
    /// not in this question's results too (see [`NameBook`]).
    book: Option<std::sync::Arc<NameBook>>,
}

/// The most words of a name the book looks for.
const BOOK_MAX_WORDS: usize = 8;

/// Every account and partner name of the books as phrases, Urdu and English, built from the
/// search index in memory. An answer may not type one outside a reference, whether or not this
/// question's results hold it: the model could write another party's name in an answer about
/// Akram (plan 52 P52-S1, robustness finding). Whole words, the same normalisation as the check
/// of the results' names, never a part of a word.
///
/// Left out, because they are ordinary words: the seeded system accounts and the expense heads
/// (`exempt`), the glossary words, a one-word name of fewer than 3 letters, and a word of the lists
/// of `words.tsv` (they have their own rules).
#[derive(Debug, Default)]
pub struct NameBook {
    phrases: std::collections::HashSet<String>,
    longest: usize,
}

impl NameBook {
    /// The book of these names (Urdu name, English name, tag).
    pub fn new(names: &[(String, String, Option<String>)], exempt: &[String]) -> Self {
        let lists = words();
        let skip: std::collections::HashSet<String> = glossary_phrases()
            .iter()
            .chain(exempt.iter())
            .map(|p| padded(p).trim().to_lowercase())
            .chain(
                [
                    WordKind::Number,
                    WordKind::Month,
                    WordKind::Never,
                    WordKind::ShopOwes,
                    WordKind::OwesShop,
                ]
                .into_iter()
                .flat_map(|kind| lists.both(kind).map(|w| padded(w).trim().to_lowercase())),
            )
            .collect();
        let mut phrases = std::collections::HashSet::new();
        let mut longest = 1usize;
        for (ur, en, _tag) in names {
            for name in [ur, en] {
                let p = padded(name).trim().to_lowercase();
                let words = p.split(' ').filter(|w| !w.is_empty()).count();
                // A one-letter or two-letter single word is not a name worth matching.
                if words == 0 || (words == 1 && p.chars().count() < 3) || skip.contains(&p) {
                    continue;
                }
                if words <= BOOK_MAX_WORDS {
                    longest = longest.max(words);
                    phrases.insert(p);
                }
            }
        }
        Self { phrases, longest }
    }

    /// The first name of the book in a text (already `padded`): a run of whole words.
    fn find_in(&self, hay: &str) -> Option<String> {
        let words: Vec<String> = hay.split_whitespace().map(str::to_lowercase).collect();
        for start in 0..words.len() {
            for len in (1..=self.longest.min(words.len().saturating_sub(start))).rev() {
                let phrase = words.get(start..start.saturating_add(len))?.join(" ");
                if self.phrases.contains(&phrase) {
                    return Some(phrase);
                }
            }
        }
        None
    }

    /// The first name of the book that a text holds as whole words, or `None`. For a test and for
    /// a tool that wants to know; the check of an answer uses the same search.
    pub fn first_in(&self, text: &str) -> Option<String> {
        self.find_in(&padded(text))
    }

    /// How many phrases the book has.
    pub fn len(&self) -> usize {
        self.phrases.len()
    }

    /// True when the book has none.
    pub fn is_empty(&self) -> bool {
        self.phrases.is_empty()
    }
}

fn is_word_char(c: char) -> bool {
    c.is_alphabetic() || text::is_urdu_mark(c)
}

/// Words only, one space between: punctuation, digits and signs become spaces. Urdu letter
/// variants are made one (ي to ی). Padded with a space at both ends.
pub(crate) fn padded(s: &str) -> String {
    let mut out = String::from(" ");
    let mut last_space = true;
    for c in text::normalize_urdu(s).chars() {
        if is_word_char(c) {
            out.push(c);
            last_space = false;
        } else if !last_space {
            out.push(' ');
            last_space = true;
        }
    }
    if !last_space {
        out.push(' ');
    }
    out
}

/// True when `phrase` (words) is in `haystack` (already padded) as whole words.
fn has_phrase(haystack: &str, phrase: &str, exact_case: bool) -> bool {
    let needle = padded(phrase);
    if needle.trim().is_empty() {
        return false;
    }
    if exact_case {
        haystack.contains(&needle)
    } else {
        haystack.to_lowercase().contains(&needle.to_lowercase())
    }
}

/// The terms of the glossary in the prompt, as phrases: a name that is also a glossary word is not
/// checked (D6-system-prompt Section 4, "Names").
pub fn glossary_phrases() -> Vec<String> {
    let Some(line) = SYSTEM.lines().find(|l| l.starts_with("Glossary (")) else {
        return Vec::new();
    };
    let body = line.split_once(':').map_or(line, |(_, rest)| rest);
    let is_urdu = |t: &str| t.chars().any(|c| ('\u{0600}'..='\u{06FF}').contains(&c));
    let mut out = Vec::new();
    for entry in body.split('|') {
        let mut run = String::new();
        let mut run_urdu = false;
        for token in entry.split_whitespace() {
            if token == "/" || token == "=" {
                if !run.is_empty() {
                    out.push(std::mem::take(&mut run));
                }
                continue;
            }
            let urdu = is_urdu(token);
            if !run.is_empty() && urdu != run_urdu {
                out.push(std::mem::take(&mut run));
            }
            if !run.is_empty() {
                run.push(' ');
            }
            run.push_str(token);
            run_urdu = urdu;
        }
        if !run.is_empty() {
            out.push(run);
        }
    }
    out
}

fn collect_names(node: &Node, out: &mut Vec<(String, String)>) {
    match node {
        Node::Val(Val::Name(n)) => out.push((n.name_ur.clone(), n.name_en.clone())),
        Node::Obj(fields) => fields.iter().for_each(|(_, v)| collect_names(v, out)),
        Node::List(items) => items.iter().for_each(|v| collect_names(v, out)),
        Node::Keyed(rows) => rows.iter().for_each(|(_, v)| collect_names(v, out)),
        _ => {}
    }
}

impl NameSet {
    /// The names of the results of this question, without the glossary words and the names in
    /// `exempt` (the seeded system accounts and standard heads, read from the books).
    pub fn new(state: &State, exempt: &[String]) -> Self {
        let mut found = Vec::new();
        for result in state.results.iter().filter(|r| r.turn == state.turn) {
            collect_names(&result.node, &mut found);
        }
        let skip: Vec<String> = glossary_phrases()
            .iter()
            .chain(exempt.iter())
            .map(|p| padded(p))
            .collect();
        let mut phrases = Vec::new();
        for (ur, en) in found {
            for name in [ur, en] {
                let p = padded(&name);
                // One letter or a bare number is not a name worth matching.
                if p.trim().chars().count() < 2 || skip.contains(&p) || phrases.contains(&p) {
                    continue;
                }
                phrases.push(p);
            }
        }
        Self {
            phrases,
            book: None,
        }
    }

    /// [`NameSet::new`], and with the book of every account and partner name: a name that is not
    /// in this question's results is refused too.
    pub fn with_book(state: &State, exempt: &[String], book: std::sync::Arc<NameBook>) -> Self {
        let mut set = Self::new(state, exempt);
        set.book = Some(book);
        set
    }

    fn find_in(&self, haystack: &str) -> Vec<String> {
        self.phrases
            .iter()
            .filter(|p| haystack.to_lowercase().contains(&p.to_lowercase()))
            .map(|p| p.trim().to_owned())
            .collect()
    }
}

/// A name in the results of this question, with the path a reference to it takes.
struct NamePath {
    path: String,
    name: NameVal,
    /// The `key` of the record (`a20`, `i2`, `s1`) of the object the name stands in, when it has one.
    row_key: Option<String>,
}

fn walk_names(path: &str, node: &Node, row_key: Option<&str>, out: &mut Vec<NamePath>) {
    match node {
        Node::Val(Val::Name(n)) => out.push(NamePath {
            path: path.to_owned(),
            name: n.clone(),
            row_key: row_key.map(str::to_owned),
        }),
        Node::Obj(fields) => {
            let own = fields.iter().find_map(|(k, v)| match (k.as_str(), v) {
                ("key", Node::Str(s)) => Some(s.as_str()),
                _ => None,
            });
            for (k, v) in fields {
                walk_names(&format!("{path}.{k}"), v, own.or(row_key), out);
            }
        }
        Node::Keyed(fields) => {
            for (k, v) in fields {
                walk_names(&format!("{path}.{k}"), v, Some(k.as_str()), out);
            }
        }
        Node::List(items) => {
            for (i, v) in items.iter().enumerate() {
                walk_names(&format!("{path}.{i}"), v, None, out);
            }
        }
        _ => {}
    }
}

/// The words of a text with their place: a run of word characters, as `padded` sees them.
pub(crate) fn word_spans(text: &str) -> Vec<(usize, usize, String)> {
    let mut out = Vec::new();
    let mut begin: Option<usize> = None;
    for (i, c) in text.char_indices() {
        match (is_word_char(c), begin) {
            (true, None) => begin = Some(i),
            (false, Some(b)) => {
                out.push((b, i, text.get(b..i).unwrap_or("")));
                begin = None;
            }
            _ => {}
        }
    }
    if let Some(b) = begin {
        out.push((b, text.len(), text.get(b..).unwrap_or("")));
    }
    out.into_iter()
        .map(|(a, b, w)| (a, b, padded(w).trim().to_lowercase()))
        .collect()
}

/// The account codes that stand for many people: the two shared پختہ کھاتہ accounts, buyers' 0030
/// and sellers' 0031 since migration 0007 (PRD FR-13.1). A name carrying one of them says no
/// record, so a typed name is never put back as a reference to the shared account.
const PUKHTA_CODES: [&str; 2] = ["0030", "0031"];

/// Which record a name in the results is, or `None` when it does not say (see [`NameSet::put_back`]).
///
/// - An account: its code (`code:0020`). The two پختہ کھاتہ codes are many people: `None`.
/// - An item or a partner in a list of rows keyed `i2` or `s1`, or an item or packing column of a
///   row: the record's own names (`item:سیب|Apple`, `partner:الف|Partner A`), because an item and a
///   partner have one name each in the books.
fn identity_of(entry: &NamePath) -> Option<String> {
    if let Some(code) = entry.name.code.as_deref() {
        return (!PUKHTA_CODES.contains(&code)).then(|| format!("code:{code}"));
    }
    let names = format!("{}|{}", entry.name.name_ur, entry.name.name_en);
    let is_key = |prefix: char| {
        entry.row_key.as_deref().is_some_and(|key| {
            key.strip_prefix(prefix)
                .is_some_and(|rest| !rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit()))
        })
    };
    match entry.path.rsplit('.').next() {
        Some("name") if is_key('i') => Some(format!("item:{names}")),
        Some("name") if is_key('s') => Some(format!("partner:{names}")),
        Some("item" | "packing") => Some(format!("item:{names}")),
        _ => None,
    }
}

/// True when a gap between the words of a name holds nothing but spaces and the punctuation that
/// can stand in a name. A digit, a brace, a ✓ or a sign between the words is not part of the name:
/// putting the name back would swallow it, so that text is left to the rewrite.
fn only_separators(gap: &str) -> bool {
    gap.chars().all(|c| {
        c.is_whitespace() || is_boundary(c) || matches!(c, '-' | '\'' | '/' | '&' | '(' | ')')
    })
}

impl NameSet {
    /// A text with each typed name that the results of this question hold exactly put back as a
    /// reference to it (plan 52 P52-S1): the same whole-word match as the check, and only a name
    /// the results hold as ONE record. A name of two different records (same text, other codes)
    /// is left alone, and so is every name that is not in the results: the check still refuses
    /// them. `None` when nothing was put back.
    ///
    /// Example: "اکرم ٹول ٹیکس کے ذمے {{r2.balance}}" with that name at `r1.matches.0.name` becomes
    /// "{{r1.matches.0.name}} کے ذمے {{r2.balance}}".
    pub fn put_back(&self, state: &State, text: &str) -> Option<String> {
        let mut entries = Vec::new();
        for result in state.results.iter().filter(|r| r.turn == state.turn) {
            walk_names(&result.id, &result.node, None, &mut entries);
        }
        // For each phrase of the check, the one path that holds it, or none when two records do.
        let mut wanted: Vec<(Vec<String>, String)> = Vec::new();
        for phrase in &self.phrases {
            let mut holders = entries.iter().filter(|e| {
                padded(&e.name.name_ur) == *phrase || padded(&e.name.name_en) == *phrase
            });
            let Some(first) = holders.next() else {
                continue;
            };
            // Which record a name is: an account by its code, an item or a partner by its row key
            // (`rows.i2.name`, `rows.s1.name`) or by the item column it stands in. A name that
            // says none (a person built from a report cell, the code of a پختہ کھاتہ account,
            // which stands for many people told apart by phone) is left to the rewrite.
            let Some(who) = identity_of(first) else {
                continue;
            };
            if holders.any(|e| identity_of(e).as_deref() != Some(who.as_str())) {
                continue;
            }
            let words: Vec<String> = phrase
                .trim()
                .to_lowercase()
                .split(' ')
                .map(str::to_owned)
                .collect();
            wanted.push((words, first.path.clone()));
        }
        // The longest name first: "Arshad Multani" before "Arshad".
        wanted.sort_by_key(|(words, _)| std::cmp::Reverse(words.len()));
        if wanted.is_empty() {
            return None;
        }
        let mut changed = false;
        let mut out = String::new();
        for piece in pieces(text) {
            match piece {
                Piece::Ref(r) => out.push_str(text.get(r.start..r.end).unwrap_or("")),
                Piece::Words(w) => {
                    let spans = word_spans(&w);
                    let mut at = 0usize;
                    let mut i = 0usize;
                    while i < spans.len() {
                        let hit = wanted.iter().find(|(words, _)| {
                            spans
                                .get(i..i.saturating_add(words.len()))
                                .is_some_and(|run| {
                                    run.len() == words.len()
                                        && run.iter().zip(words).all(|((_, _, t), want)| t == want)
                                        && run.windows(2).all(|pair| match pair {
                                            [(_, end, _), (start, _, _)] => {
                                                only_separators(w.get(*end..*start).unwrap_or("-"))
                                            }
                                            _ => true,
                                        })
                                })
                        });
                        match hit {
                            Some((words, path)) => {
                                let last = i.saturating_add(words.len()).saturating_sub(1);
                                let (Some(first_span), Some(last_span)) =
                                    (spans.get(i), spans.get(last))
                                else {
                                    break;
                                };
                                out.push_str(w.get(at..first_span.0).unwrap_or(""));
                                out.push_str(&format!("{{{{{path}}}}}"));
                                at = last_span.1;
                                i = last.saturating_add(1);
                                changed = true;
                            }
                            None => i = i.saturating_add(1),
                        }
                    }
                    out.push_str(w.get(at..).unwrap_or(""));
                }
            }
        }
        changed.then_some(out)
    }
}

/// What a reference ends on.
struct Resolved {
    node: Node,
    root_tool: &'static str,
    path: String,
}

/// Follows a reference. `Err` carries the failure to report.
fn resolve(env: &Env<'_>, state: &State, path: &str) -> Result<Resolved, Failure> {
    let found = lookup(env, state, path, "reference").map_err(|e: ToolError| {
        if e.code == "ERR_EARLIER_QUESTION" {
            Failure::new("earlier_question", path)
        } else {
            Failure::new("reference", path)
        }
    })?;
    let node = match &found {
        Found::Ref(n) => (*n).clone(),
        Found::Owned(n) => n.clone(),
    };
    if !matches!(node, Node::Val(_)) {
        return Err(Failure::new("reference", path));
    }
    let first = path.split('.').next().unwrap_or("");
    let result = state.find(first);
    let root_tool = result.map_or("question", |r| r.tool);
    Ok(Resolved {
        node,
        root_tool,
        path: path.to_owned(),
    })
}

fn is_money_figure(node: &Node) -> bool {
    matches!(
        node,
        Node::Val(Val::Money(_) | Val::Diff { .. } | Val::Change { .. })
    )
}

/// The parts of an `estimate` result that are estimated figures. `now` and `history` are book
/// figures and are not in the list.
fn is_estimate_part(second: &str) -> bool {
    matches!(
        second,
        "same_pace" | "same_growth" | "same_season" | "what_if_pace" | "ahead" | "year"
    )
}

/// True when a path points at an estimate figure (D6-system-prompt Section 4, `kind`).
fn is_estimate_figure(r: &Resolved) -> bool {
    r.root_tool == "estimate" && is_estimate_part(r.path.split('.').nth(1).unwrap_or(""))
}

/// True when a path starts in an `estimate` result and goes into one of its estimated parts, also
/// with a `*` row (`e1.ahead.*.same_pace`). `calculate` refuses such a path: an estimate must
/// never come back as a book figure (PRD FR-7.76 point 4).
pub(crate) fn is_estimate_path(state: &State, path: &str) -> bool {
    let mut parts = path.split('.');
    let first = parts.next().unwrap_or("");
    state.find(first).is_some_and(|r| r.tool == "estimate")
        && parts.next().is_some_and(is_estimate_part)
}

fn is_boundary(c: char) -> bool {
    matches!(
        c,
        '۔' | '،' | '؟' | '.' | ',' | ';' | ':' | '!' | '?' | '\n' | '•' | '؛'
    )
}

/// What the text of one reference or a field holds.
enum Item {
    Words(String),
    /// A reference: its path, what it ends on (`None` when it did not resolve), and whether it
    /// is an estimate figure.
    Ref {
        path: String,
        node: Option<Node>,
        estimate: bool,
    },
}

fn items_of(env: &Env<'_>, state: &State, text_in: &str) -> (Vec<Item>, Vec<Failure>) {
    let mut failures = Vec::new();
    let mut items = Vec::new();
    for piece in pieces(text_in) {
        match piece {
            Piece::Words(w) => items.push(Item::Words(w)),
            Piece::Ref(r) => match resolve(env, state, &r.path) {
                Ok(resolved) => {
                    let estimate = is_estimate_figure(&resolved);
                    items.push(Item::Ref {
                        path: r.path,
                        node: Some(resolved.node),
                        estimate,
                    });
                }
                Err(f) => {
                    failures.push(f);
                    items.push(Item::Ref {
                        path: r.path,
                        node: None,
                        estimate: false,
                    });
                }
            },
        }
    }
    (items, failures)
}

/// The words of the other side inside the window of one balance, if any.
fn side_failure(items: &[Item], at: usize, side: Side, figure: &str) -> Option<Failure> {
    let lists = words();
    let other = match side {
        Side::Dr => WordKind::ShopOwes,
        Side::Cr => WordKind::OwesShop,
        Side::Zero => return None,
    };
    // Backwards from the figure.
    let mut before = String::new();
    for item in items.get(..at)?.iter().rev() {
        match item {
            Item::Words(w) => {
                let cut = w.rfind(is_boundary).map_or(0, |i| {
                    i.saturating_add(w[i..].chars().next().map_or(1, char::len_utf8))
                });
                before = format!("{} {before}", w.get(cut..).unwrap_or(""));
                if cut > 0 {
                    break;
                }
            }
            Item::Ref {
                node: Some(node), ..
            } if is_money_figure(node) => break,
            Item::Ref { .. } => {}
        }
    }
    let mut after = String::new();
    for item in items.get(at.saturating_add(1)..)?.iter() {
        match item {
            Item::Words(w) => {
                let end = w.find(is_boundary).unwrap_or(w.len());
                after = format!("{after} {}", w.get(..end).unwrap_or(""));
                if end < w.len() {
                    break;
                }
            }
            Item::Ref {
                node: Some(node), ..
            } if is_money_figure(node) => break,
            Item::Ref { .. } => {}
        }
    }
    let window = padded(&format!("{before} {after}"));
    lists
        .both(other)
        .find(|w| has_phrase(&window, w, false))
        .map(|w| Failure {
            rule: "side_words",
            at: w.clone(),
            figure: Some(figure.to_owned()),
        })
}

fn side_of_node(node: &Node) -> Option<Side> {
    match node {
        Node::Val(Val::Money(m)) => m.side,
        _ => None,
    }
}

fn wrong_script(language: Language, s: &str) -> Option<String> {
    let bad = |c: char| match language {
        Language::Ur => c.is_ascii_alphabetic(),
        Language::En => {
            ('\u{0600}'..='\u{06FF}').contains(&c)
                || ('\u{0750}'..='\u{077F}').contains(&c)
                || ('\u{FB50}'..='\u{FDFF}').contains(&c)
                || ('\u{FE70}'..='\u{FEFF}').contains(&c)
        }
    };
    s.chars().find(|c| bad(*c)).map(|c| c.to_string())
}

/// The checks of the words alone (`text` or one `next` question): digits, number words, months,
/// names, ✓, script, never-use words.
fn word_failures(env: &Env<'_>, names: &NameSet, items: &[Item], out: &mut Vec<Failure>) {
    let lists = words();
    let joined: String = items
        .iter()
        .map(|i| match i {
            Item::Words(w) => w.as_str(),
            Item::Ref { .. } => " ",
        })
        .collect::<Vec<_>>()
        .join(" ");
    if let Some(d) = joined.chars().find(|c| c.is_numeric()) {
        out.push(Failure::new("digit", &d.to_string()));
    }
    if joined.contains(['✓', '✔', '☑']) {
        out.push(Failure::new("check_mark", "✓"));
    }
    if joined.contains("{{") || joined.contains("}}") {
        out.push(Failure::new("reference", "{{"));
    }
    if let Some(c) = wrong_script(env.language, &joined) {
        out.push(Failure::new("script", &c));
    }
    let hay = padded(&joined);
    // The number words of `words.tsv` and the ones `number_words` adds (1 to 99 in Urdu and Roman
    // Urdu, which the list lacks, robustness finding attack_03).
    if let Some(w) = lists
        .both(WordKind::Number)
        .map(String::as_str)
        .chain(crate::number_words::all())
        .find(|w| has_phrase(&hay, w, false))
    {
        out.push(Failure::new("number_word", w));
    }
    if let Some(w) = lists
        .both(WordKind::Month)
        .find(|w| has_phrase(&hay, w, true))
    {
        out.push(Failure::new("month", w));
    }
    if let Some(w) = lists
        .both(WordKind::Never)
        .find(|w| has_phrase(&hay, w, false))
    {
        out.push(Failure::new("never_word", w));
    }
    if let Some(name) = names.find_in(&hay).into_iter().next() {
        out.push(Failure::new("name", &name));
    } else if let Some(name) = names.book.as_ref().and_then(|b| b.find_in(&hay)) {
        // A name of the books that this question's results do not hold: the app cannot put it
        // back (the results do not say which record it is), so it is for the rewrite.
        out.push(Failure::new("name", &name));
    }
}

/// Words of the answer text, each filled reference counted as one (PRD FR-7.76 point 3).
fn count_words(items: &[Item]) -> usize {
    items
        .iter()
        .map(|i| match i {
            Item::Words(w) => w.split_whitespace().count(),
            Item::Ref { .. } => 1,
        })
        .fold(0usize, usize::saturating_add)
}

/// The context the checks need.
pub struct Check<'a> {
    /// The tools' environment: language, numbers, today.
    pub env: &'a Env<'a>,
    /// The results of the conversation.
    pub state: &'a State,
    /// The names of this question's results.
    pub names: &'a NameSet,
}

fn find_result<'s>(state: &'s State, id: &str) -> Option<&'s crate::results::ToolResult> {
    state.find(id).filter(|r| r.turn == state.turn)
}

fn matches_len(node: &Node) -> usize {
    match node.get(&["matches"]) {
        Some(Node::List(items)) => items.len(),
        _ => 0,
    }
}

impl Check<'_> {
    /// Checks a whole answer.
    pub fn run(&self, answer: &ModelAnswer) -> Verdict {
        let mut failures: Vec<Failure> = Vec::new();
        let mut differ: Vec<String> = Vec::new();
        let mut estimate_used = false;
        // text: only for the kinds that have words of the model.
        if answer.kind.has_text() {
            let (items, mut refs) = items_of(self.env, self.state, &answer.text);
            failures.append(&mut refs);
            word_failures(self.env, self.names, &items, &mut failures);
            // General advice names no figure and no name of its own (PRD FR-7.88, v1.19): no
            // reference at all, and nothing shown.
            if answer.kind == AnswerKind::GeneralAdvice {
                if items.iter().any(|i| matches!(i, Item::Ref { .. })) {
                    failures.push(Failure::new("general_advice_reference", "text"));
                }
                if !answer.show.is_empty() {
                    failures.push(Failure::new("general_advice_show", "show"));
                }
            }
            if count_words(&items) > MAX_ANSWER_WORDS {
                failures.push(Failure::new(
                    "too_long",
                    &answer.text.chars().take(24).collect::<String>(),
                ));
            }
            if answer.kind == AnswerKind::Answer {
                self.check_asked(&items, answer, &mut failures);
            }
            for (at, item) in items.iter().enumerate() {
                let Item::Ref {
                    path,
                    node: Some(node),
                    estimate,
                } = item
                else {
                    continue;
                };
                if let Node::Val(Val::Money(m)) = node
                    && m.checked == Some(false)
                {
                    differ.push(path.clone());
                }
                if let Some(side) = side_of_node(node)
                    && let Some(f) = side_failure(&items, at, side, path)
                {
                    failures.push(f);
                }
                estimate_used |= *estimate;
            }
        }
        if estimate_used && !matches!(answer.kind, AnswerKind::Estimate | AnswerKind::Advice) {
            failures.push(Failure::new("estimate_kind", &answer.kind_word()));
        }
        // next: no figure reference, a name only as a reference.
        if answer.next.len() > MAX_NEXT {
            failures.push(Failure::new("next", "more than 3"));
        }
        for question in &answer.next {
            let (items, mut refs) = items_of(self.env, self.state, question);
            failures.append(&mut refs);
            word_failures(self.env, self.names, &items, &mut failures);
            for item in &items {
                if let Item::Ref {
                    node: Some(node), ..
                } = item
                    && !matches!(node, Node::Val(Val::Name(_)))
                {
                    failures.push(Failure::new("next", "a figure in a next question"));
                }
            }
        }
        self.check_show(answer, &mut failures);
        self.check_pointers(answer, &mut failures);
        if !differ.is_empty() {
            differ.dedup();
            return Verdict::Differ(differ);
        }
        failures.dedup();
        if failures.is_empty() {
            Verdict::Ok
        } else {
            Verdict::Retry(failures)
        }
    }

    /// When the question asks for the commission AND the sales, an answer that gives only one of
    /// them is sent back (Q154: commission with no sale, Q155 the reverse in the live run 2). Only
    /// when the books read for this question hold the missing figure, so the model can add it
    /// without a new tool call; an answer that cites a calculation or shows a table is left alone.
    fn check_asked(&self, items: &[Item], answer: &ModelAnswer, failures: &mut Vec<Failure>) {
        let question = self.env.question;
        if !cues::commission_and_sales_asked(question) {
            return;
        }
        let cited: Vec<&str> = items
            .iter()
            .filter_map(|i| match i {
                Item::Ref { path, .. } => Some(path.as_str()),
                Item::Words(_) => None,
            })
            .collect();
        let names_family = |family: &[&str]| {
            cited
                .iter()
                .any(|path| path.split('.').any(|seg| family.contains(&seg)))
        };
        let (commission, sales) = (
            names_family(COMMISSION_COLUMNS),
            names_family(SALES_COLUMNS),
        );
        let cites_a_calculation = cited
            .iter()
            .any(|path| path.starts_with('c') || path.starts_with('e'));
        if commission == sales || cites_a_calculation || !answer.show.is_empty() {
            return;
        }
        let (missing, family) = if commission {
            ("the sales figure (gross) is asked too", SALES_COLUMNS)
        } else {
            (
                "the commission figure (total_comm) is asked too",
                COMMISSION_COLUMNS,
            )
        };
        let held = self
            .state
            .results
            .iter()
            .filter(|r| r.turn == self.state.turn)
            .any(|r| match r.node.get(&["columns"]) {
                Some(Node::List(columns)) => columns
                    .iter()
                    .any(|c| matches!(c, Node::Str(name) if family.contains(&name.as_str()))),
                _ => false,
            });
        if held {
            failures.push(Failure::new("asked", missing));
        }
    }

    fn check_show(&self, answer: &ModelAnswer, failures: &mut Vec<Failure>) {
        if answer.show.len() > MAX_SHOW {
            failures.push(Failure::new("show", "more than 3"));
        }
        for show in &answer.show {
            let Some(result) = find_result(self.state, &show.result) else {
                failures.push(Failure::new("show", &show.result));
                continue;
            };
            let ok = matches!(
                (show.as_, &result.shown),
                (ShowAs::Table, Some(Shown::Table(_) | Shown::TableChart(..)))
                    | (
                        ShowAs::Chart,
                        Some(Shown::Chart(..) | Shown::TableChart(..))
                    )
            );
            if !ok {
                failures.push(Failure::new("show", &show.result));
            }
            if show
                .rows
                .is_some_and(|r| r < 1 || usize::try_from(r).unwrap_or(usize::MAX) > MAX_TABLE_ROWS)
            {
                failures.push(Failure::new("show", "rows"));
            }
        }
    }

    fn check_pointers(&self, answer: &ModelAnswer, failures: &mut Vec<Failure>) {
        match answer.kind {
            AnswerKind::WhichOne | AnswerKind::NotFound => {
                let found = answer
                    .choose
                    .as_deref()
                    .and_then(|id| find_result(self.state, id))
                    .filter(|r| r.tool == "find");
                match found {
                    None => failures.push(Failure::new("choose", "a find result of this question")),
                    Some(r) => {
                        let n = matches_len(&r.node);
                        // One match is a question only for a typed name that a close spelling
                        // found (`f1`, tools/typed.rs): "did you mean this one?" (plan 52).
                        let fits = if answer.kind == AnswerKind::WhichOne {
                            n >= 2 || (n == 1 && r.id.starts_with('f'))
                        } else {
                            // No match at all, or matches of another role than the one asked
                            // (a buyer found for a seller): not found as that role.
                            n == 0 || r.node.get(&["widened"]).is_some()
                        };
                        if !fits {
                            failures.push(Failure::new("choose", &r.id));
                        }
                    }
                }
            }
            AnswerKind::ChangeRequest if answer.entry.is_none() => {
                failures.push(Failure::new("entry", "the screen"));
            }
            _ => {}
        }
        if let Some(entry) = &answer.entry
            && let Some(party) = &entry.party
            && self.party_ref(party).is_none()
        {
            failures.push(Failure::new("entry", party));
        }
        if let Some(nearest) = &answer.nearest
            && let Some(party) = &nearest.party
            && self.party_ref(party).is_none()
        {
            failures.push(Failure::new("nearest", party));
        }
    }

    /// A party reference: a find match of this question (`r1.matches.0`) or a key it showed.
    fn party_ref(&self, path: &str) -> Option<()> {
        crate::tools::record(self.env, self.state, path, "party")
            .ok()
            .map(|_| ())
    }
}

impl ModelAnswer {
    fn kind_word(&self) -> String {
        format!("{:?}", self.kind).to_lowercase()
    }
}
