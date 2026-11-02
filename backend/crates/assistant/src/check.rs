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
//! | `name` | a name from this question's results typed outside a reference |
//! | `check_mark` | a typed ✓ |
//! | `reference` | a path that does not exist, or one that does not end on a value |
//! | `earlier_question` | a path into a result of an earlier question |
//! | `side_words` | the words of the other side next to a balance |
//! | `script` | Latin letters in an Urdu answer, Urdu letters in an English one |
//! | `never_word` | a word of the never-use list |
//! | `too_long` | more than 130 words, each filled reference counted as one |
//! | `estimate_kind` | an estimate figure in an answer that is not of kind estimate or advice |
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
use crate::results::{Node, Shown, Side, Val};
use crate::tools::{Env, Found, State, ToolError, lookup};

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
}

fn is_word_char(c: char) -> bool {
    c.is_alphabetic() || text::is_urdu_mark(c)
}

/// Words only, one space between: punctuation, digits and signs become spaces. Urdu letter
/// variants are made one (ي to ی). Padded with a space at both ends.
fn padded(s: &str) -> String {
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
        Self { phrases }
    }

    fn find_in(&self, haystack: &str) -> Vec<String> {
        self.phrases
            .iter()
            .filter(|p| haystack.to_lowercase().contains(&p.to_lowercase()))
            .map(|p| p.trim().to_owned())
            .collect()
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

/// True when a path points at an estimate figure (D6-system-prompt Section 4, `kind`).
fn is_estimate_figure(r: &Resolved) -> bool {
    if r.root_tool != "estimate" {
        return false;
    }
    let second = r.path.split('.').nth(1).unwrap_or("");
    matches!(
        second,
        "same_pace" | "same_growth" | "same_season" | "ahead"
    )
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
            if count_words(&items) > MAX_ANSWER_WORDS {
                failures.push(Failure::new(
                    "too_long",
                    &answer.text.chars().take(24).collect::<String>(),
                ));
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
                        let fits = if answer.kind == AnswerKind::WhichOne {
                            n >= 2
                        } else {
                            n == 0
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
