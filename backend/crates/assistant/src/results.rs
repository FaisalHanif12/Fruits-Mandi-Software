//! What the tools give back (D6-system-prompt Section 5, Code Standards Q7).
//!
//! Every tool builds a [`Node`] tree out of typed [`Val`]ues. The tree has two readers:
//!
//! - [`tool_result_json`]: the one function that turns a result into what is **sent to the model**.
//!   It gives the small value objects of Section 5.1 (`{"rs": 2880, "side": "بنام", "checked":
//!   true}`), leaves out every field PRD FR-7.83 keeps back (the builders never put them in), and
//!   keeps a result under [`MAX_RESULT_BYTES`] by dropping rows from the end.
//! - [`Node::get`]: the path a reference points to (`r3.rows.je4518.debit`). The checks and the
//!   filling of the answer read the same tree, so the model can only cite what it was shown.
//!
//! A [`Val`] keeps money as `Paisa`; the JSON shows whole rupees (`Paisa::rupees`). Nothing here
//! adds, multiplies or rounds a figure (Code Standards Q5).

use mandi_reports::export::l10n::L10n;
use mandi_reports::reports::Names;
use mandi_services::core::auth::Language;
use mandi_services::core::money::Paisa;
use mandi_services::domain::figures::Dir;
use serde_json::{Map, Value, json};

use crate::answer::{
    AnswerChart, AnswerEstimate, AnswerName, AnswerSource, AnswerTable, AnswerWorking,
};
use crate::limits::MAX_RESULT_BYTES;

/// The side of a balance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    /// بنام: he owes the shop.
    Dr,
    /// جمع: the shop owes him.
    Cr,
    /// Nothing either way.
    Zero,
}

/// An amount of money with what is known about it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Money {
    /// The amount: never negative with a `side`; signed without one (a loss, a کل رقم).
    pub paisa: Paisa,
    /// The side, for one party's balance and a calculator result over balances.
    pub side: Option<Side>,
    /// `Some(true)` when both places of the figure agree (FR-7.76), `Some(false)` when they
    /// differ, `None` when the figure has no second place.
    pub checked: Option<bool>,
    /// The figure in each place, for the sources that have two: `("party_card", 288000)`.
    pub places: Vec<(&'static str, Paisa)>,
}

impl Money {
    /// An amount with no side and no second place.
    pub fn plain(paisa: Paisa) -> Self {
        Self {
            paisa,
            side: None,
            checked: None,
            places: Vec::new(),
        }
    }

    /// A balance with its side and no second place.
    pub fn balance(paisa: Paisa, side: Side) -> Self {
        Self {
            paisa,
            side: Some(side),
            checked: None,
            places: Vec::new(),
        }
    }
}

/// A record's name in both scripts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NameVal {
    /// Urdu name.
    pub name_ur: String,
    /// English name.
    pub name_en: String,
    /// The tag after the name.
    pub tag: Option<String>,
    /// The code, for example `0020`.
    pub code: Option<String>,
}

impl NameVal {
    /// The name as the answer shows it.
    pub fn answer_name(&self) -> AnswerName {
        AnswerName {
            name_ur: self.name_ur.clone(),
            name_en: self.name_en.clone(),
            tag: self.tag.clone(),
            code: self.code.clone(),
        }
    }
}

/// One value a reference may end on (D6-system-prompt Section 5.1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Val {
    /// Money, a balance, a total.
    Money(Money),
    /// A difference: its size and its direction.
    Diff {
        /// The size.
        size: Paisa,
        /// More, less or the same.
        dir: Dir,
    },
    /// A change in percent: hundredths of a percent and its direction.
    Change {
        /// Hundredths of a percent.
        hundredths: i64,
        /// More, less or the same.
        dir: Dir,
    },
    /// A percent in hundredths.
    Pct(i64),
    /// A count or quantity.
    Count(i64),
    /// A difference of two counts: its size and its direction.
    CountDiff {
        /// The size.
        size: i64,
        /// More, less or the same.
        dir: Dir,
    },
    /// A date, `YYYY-MM-DD`.
    Date(String),
    /// A month of a year.
    Month {
        /// The year.
        year: i32,
        /// The month, 1 to 12.
        month: u32,
    },
    /// A record's name.
    Name(NameVal),
    /// An entry's label (بکری 1, وصولی 4): the kind's Fluent message and the number.
    Doc {
        /// Fluent message ID of the kind.
        type_id: String,
        /// The number.
        number: Option<i64>,
    },
    /// A number read from the Owner's question (`q.n1`): shown plain, never with ✓.
    Question {
        /// The value (rupees for an amount, the count for months).
        number: i64,
    },
    /// A text of the written guide of the app (`app_help`): the model reads it and cites it by
    /// reference, and the app puts the same words into the answer. It is the app's own text, so
    /// the checks that refuse a typed digit or a typed name do not apply to it (plan 51 B3).
    Guide(String),
}

/// A node of a result tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Node {
    /// Nothing (an empty cell).
    Null,
    /// A text the model may read but cannot cite: ids, filters, column keys, system text.
    Str(String),
    /// A flag the model may read but cannot cite (`rows_cut`).
    Bool(bool),
    /// A value a reference may end on.
    Val(Val),
    /// Named fields.
    Obj(Vec<(String, Node)>),
    /// A list read by place (`matches.0`).
    List(Vec<Node>),
    /// Rows read by their record's stable key (`rows.je4518`).
    Keyed(Vec<(String, Node)>),
}

impl Node {
    /// An object from fields.
    pub fn obj(fields: Vec<(&str, Node)>) -> Node {
        Node::Obj(fields.into_iter().map(|(k, v)| (k.to_owned(), v)).collect())
    }

    /// A value node.
    pub fn val(v: Val) -> Node {
        Node::Val(v)
    }

    /// Follows a path of names, indexes and row keys. `None` when any step does not exist.
    pub fn get(&self, path: &[&str]) -> Option<&Node> {
        let Some((first, rest)) = path.split_first() else {
            return Some(self);
        };
        let next = match self {
            Node::Obj(fields) => fields.iter().find(|(k, _)| k == first).map(|(_, v)| v)?,
            Node::List(items) => items.get(first.parse::<usize>().ok()?)?,
            Node::Keyed(rows) => rows.iter().find(|(k, _)| k == first).map(|(_, v)| v)?,
            Node::Null | Node::Str(_) | Node::Bool(_) | Node::Val(_) => return None,
        };
        next.get(rest)
    }

    /// The value this node is, if it is one.
    pub fn as_val(&self) -> Option<&Val> {
        match self {
            Node::Val(v) => Some(v),
            _ => None,
        }
    }
}

/// The words the model's JSON needs from Fluent: the side words and the labels of entries, in both
/// languages. No sentence is written in Rust (Tech Stack 4.2).
pub struct Words {
    ur: L10n,
    en: L10n,
}

impl Words {
    /// Reads the Fluent files of both languages.
    ///
    /// # Errors
    /// The files could not be read (a bug).
    pub fn new() -> Result<Self, mandi_reports::ReportsError> {
        Ok(Self {
            ur: L10n::new(Language::Ur)?,
            en: L10n::new(Language::En)?,
        })
    }

    /// A Fluent message in a language.
    pub fn text(&self, language: Language, id: &str) -> String {
        match language {
            Language::Ur => self.ur.text(id),
            Language::En => self.en.text(id),
        }
    }

    /// The side word in Urdu: بنام or جمع. Zero has none.
    pub fn side(&self, side: Side) -> Option<String> {
        match side {
            Side::Dr => Some(self.ur.text("report-side-dr")),
            Side::Cr => Some(self.ur.text("report-side-cr")),
            Side::Zero => None,
        }
    }

    /// The label of an entry in both languages: the kind's word and the number ("بکری 1").
    pub fn doc_label(&self, type_id: &str, number: Option<i64>) -> Names {
        let make = |l: &L10n| match number {
            Some(n) => format!("{} {n}", l.text(type_id)),
            None => l.text(type_id),
        };
        Names {
            ur: make(&self.ur),
            en: make(&self.en),
        }
    }
}

fn rupees(p: Paisa) -> Value {
    json!(p.rupees())
}

fn dir_word(dir: Dir) -> &'static str {
    match dir {
        Dir::Up => "up",
        Dir::Down => "down",
        Dir::Same => "same",
    }
}

/// The percent as the model reads it: `"72.10"` (hundredths with two decimals).
fn pct_text(hundredths: i64) -> String {
    let whole = hundredths.checked_div(100).unwrap_or(0);
    let frac = hundredths.checked_rem(100).unwrap_or(0).abs();
    format!("{whole}.{frac:02}")
}

impl Val {
    /// The small object of Section 5.1.
    pub fn to_json(&self, words: &Words) -> Value {
        match self {
            Val::Money(m) => {
                let mut o = Map::new();
                o.insert("rs".to_owned(), rupees(m.paisa));
                if let Some(side) = m.side {
                    o.insert("side".to_owned(), json!(words.side(side)));
                }
                if let Some(checked) = m.checked {
                    o.insert("checked".to_owned(), json!(checked));
                    let places: Map<String, Value> = m
                        .places
                        .iter()
                        .map(|(k, p)| ((*k).to_owned(), rupees(*p)))
                        .collect();
                    o.insert("places".to_owned(), Value::Object(places));
                }
                Value::Object(o)
            }
            Val::Diff { size, dir } => json!({"rs": rupees(*size), "dir": dir_word(*dir)}),
            Val::Change { hundredths, dir } => {
                json!({"pct": pct_text(*hundredths), "dir": dir_word(*dir)})
            }
            Val::Pct(h) => json!({"pct": pct_text(*h)}),
            Val::Count(n) => json!({"n": n}),
            Val::CountDiff { size, dir } => json!({"n": size, "dir": dir_word(*dir)}),
            Val::Date(d) => json!({"date": d}),
            Val::Month { year, month } => json!({"month": format!("{year:04}-{month:02}")}),
            Val::Name(n) => json!({"ur": n.name_ur, "en": n.name_en, "tag": n.tag}),
            Val::Doc { type_id, number } => {
                let label = words.doc_label(type_id, *number);
                json!({"label": {"ur": label.ur, "en": label.en}})
            }
            Val::Question { number } => json!({"n": number}),
            Val::Guide(text) => json!(text),
        }
    }
}

impl Node {
    /// The JSON the model reads. Keyed rows become a list of objects that each carry their `key`.
    pub fn to_json(&self, words: &Words) -> Value {
        match self {
            Node::Null => Value::Null,
            Node::Str(s) => json!(s),
            Node::Bool(b) => json!(b),
            Node::Val(v) => v.to_json(words),
            Node::Obj(fields) => {
                let mut o = Map::new();
                for (k, v) in fields {
                    o.insert(k.clone(), v.to_json(words));
                }
                Value::Object(o)
            }
            Node::List(items) => Value::Array(items.iter().map(|n| n.to_json(words)).collect()),
            Node::Keyed(rows) => Value::Array(
                rows.iter()
                    .map(|(key, row)| match row.to_json(words) {
                        Value::Object(mut o) => {
                            let mut with_key = Map::new();
                            with_key.insert("key".to_owned(), json!(key));
                            with_key.append(&mut o);
                            Value::Object(with_key)
                        }
                        other => json!({"key": key, "value": other}),
                    })
                    .collect(),
            ),
        }
    }
}

/// What a result can show on the screen, copied from the tool's own output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Shown {
    /// A table of a report or a view, with every row kept (at most 100); the answer cuts it to 7.
    Table(AnswerTable),
    /// A table of a view, and the chart of its monthly series.
    TableChart(AnswerTable, Box<AnswerChart>),
    /// A chart of a monthly series, and for an estimate the estimate behind it.
    Chart(Box<AnswerChart>, Option<Box<AnswerEstimate>>),
}

/// One tool result of the question.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolResult {
    /// `r1`, `c1`, `e1`: runs on through the conversation.
    pub id: String,
    /// The tool.
    pub tool: &'static str,
    /// The input as the model sent it, for the log.
    pub input: Value,
    /// What the model reads and cites.
    pub node: Node,
    /// What the screen can draw.
    pub shown: Option<Shown>,
    /// The source for the buttons and the source line.
    pub source: Option<AnswerSource>,
    /// True when the partner parts were left out because the section is locked.
    pub partner_parts_locked: bool,
    /// True when the result holds partner figures (the section was open when it was read): it is
    /// not resent once the section is locked again (PRD FR-7.82, E2E-54).
    pub partner_data: bool,
    /// The question of the conversation it belongs to (0, 1, ...). A reference may only point to a
    /// result of the question being answered (PRD FR-7.76 point 1).
    pub turn: u32,
    /// How a calculator result was worked out (ⓘ).
    pub working: Option<AnswerWorking>,
}

/// Cuts a result so it is at most [`MAX_RESULT_BYTES`] of JSON: rows are dropped from the end, the
/// node gets `rows_cut`, and the totals still cover the whole filter. The node kept in the
/// conversation is the cut one, so what the model can cite is exactly what it was shown (a
/// `rows.*` sum over a cut list is then refused).
pub fn fit(result: &mut ToolResult, words: &Words) {
    let mut cut = false;
    while tool_result_json(result, words).to_string().len() > MAX_RESULT_BYTES
        && drop_last_row(&mut result.node)
    {
        cut = true;
    }
    if cut {
        mark_cut(&mut result.node);
    }
}

fn mark_cut(node: &mut Node) {
    if let Node::Obj(fields) = node {
        fields.retain(|(k, _)| k != "rows_cut");
        fields.push(("rows_cut".to_owned(), Node::Bool(true)));
    }
}

/// The one function that turns a result into what is sent to the model (Code Standards Q7): the
/// id and the tool, then the fields of the tree. Call [`fit`] first to keep it under the cap.
pub fn tool_result_json(result: &ToolResult, words: &Words) -> Value {
    let mut out = Map::new();
    out.insert("id".to_owned(), json!(result.id));
    out.insert("tool".to_owned(), json!(result.tool));
    if result.partner_parts_locked {
        out.insert("partner_parts".to_owned(), json!("locked"));
    }
    if let Value::Object(fields) = result.node.to_json(words) {
        for (k, v) in fields {
            out.insert(k, v);
        }
    }
    Value::Object(out)
}

/// Drops the last row of the biggest row list of a result tree. False when there is none.
fn drop_last_row(node: &mut Node) -> bool {
    let Node::Obj(fields) = node else {
        return false;
    };
    // Rows are the field called `rows` (or `matches` / `ahead`); drop from there first.
    for name in ["rows", "matches", "ahead"] {
        let Some((_, list)) = fields.iter_mut().find(|(k, _)| k == name) else {
            continue;
        };
        let dropped = match list {
            Node::Keyed(rows) => rows.pop().is_some(),
            Node::List(items) => items.pop().is_some(),
            _ => false,
        };
        if dropped {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rs(r: i64) -> Paisa {
        Paisa::from_rupees(r).unwrap()
    }

    #[test]
    fn a_path_follows_names_places_and_row_keys() {
        let tree = Node::obj(vec![
            (
                "balance",
                Node::val(Val::Money(Money::balance(rs(2_880), Side::Dr))),
            ),
            (
                "matches",
                Node::List(vec![Node::obj(vec![(
                    "code",
                    Node::Str("0020".to_owned()),
                )])]),
            ),
            (
                "rows",
                Node::Keyed(vec![(
                    "je4518".to_owned(),
                    Node::obj(vec![("debit", Node::val(Val::Count(1)))]),
                )]),
            ),
        ]);
        assert!(tree.get(&["balance"]).and_then(Node::as_val).is_some());
        assert!(tree.get(&["matches", "0", "code"]).is_some());
        assert!(tree.get(&["rows", "je4518", "debit"]).is_some());
        assert!(tree.get(&["rows", "je9", "debit"]).is_none());
        assert!(tree.get(&["matches", "1"]).is_none());
        assert!(tree.get(&["balance", "x"]).is_none());
        assert_eq!(tree.get(&[]), Some(&tree));
    }

    #[test]
    fn money_shows_whole_rupees_its_side_and_both_places() {
        let words = Words::new().unwrap();
        let mut m = Money::balance(rs(2_880), Side::Dr);
        m.checked = Some(true);
        m.places = vec![("party_card", rs(2_880)), ("R13", rs(2_880))];
        let json = Val::Money(m).to_json(&words);
        assert_eq!(json["rs"], 2_880);
        assert_eq!(json["side"], "بنام");
        assert_eq!(json["checked"], true);
        assert_eq!(json["places"]["R13"], 2_880);
        // No side word for zero; a plain amount has no side at all.
        let zero = Val::Money(Money::balance(rs(0), Side::Zero)).to_json(&words);
        assert!(zero["side"].is_null());
        let plain = Val::Money(Money::plain(rs(-1_004))).to_json(&words);
        assert_eq!(plain, json!({"rs": -1_004}));
    }

    #[test]
    fn differences_changes_percents_dates_months_and_labels() {
        let words = Words::new().unwrap();
        assert_eq!(
            Val::Diff {
                size: rs(3_000),
                dir: Dir::Up
            }
            .to_json(&words),
            json!({"rs": 3_000, "dir": "up"})
        );
        assert_eq!(
            Val::Change {
                hundredths: 625,
                dir: Dir::Down
            }
            .to_json(&words),
            json!({"pct": "6.25", "dir": "down"})
        );
        assert_eq!(Val::Pct(7_210).to_json(&words), json!({"pct": "72.10"}));
        assert_eq!(Val::Pct(5).to_json(&words), json!({"pct": "0.05"}));
        assert_eq!(Val::Count(2).to_json(&words), json!({"n": 2}));
        assert_eq!(
            Val::Date("2026-10-10".into()).to_json(&words),
            json!({"date": "2026-10-10"})
        );
        assert_eq!(
            Val::Month {
                year: 2026,
                month: 9
            }
            .to_json(&words),
            json!({"month": "2026-09"})
        );
        let label = Val::Doc {
            type_id: "report-doc-bikri-first".into(),
            number: Some(1),
        }
        .to_json(&words);
        assert_eq!(label["label"]["ur"], "بکری 1");
        assert_eq!(label["label"]["en"], "Sale 1");
    }

    #[test]
    fn keyed_rows_become_a_list_that_carries_each_key() {
        let words = Words::new().unwrap();
        let node = Node::obj(vec![(
            "rows",
            Node::Keyed(vec![(
                "a20".to_owned(),
                Node::obj(vec![("n", Node::val(Val::Count(3)))]),
            )]),
        )]);
        assert_eq!(
            node.to_json(&words),
            json!({"rows": [{"key": "a20", "n": {"n": 3}}]})
        );
    }

    #[test]
    fn a_big_result_loses_rows_from_the_end_and_says_so() {
        let words = Words::new().unwrap();
        let rows: Vec<(String, Node)> = (0..2_000)
            .map(|i| {
                (
                    format!("a{i}"),
                    Node::obj(vec![("text", Node::Str("x".repeat(40)))]),
                )
            })
            .collect();
        let result = ToolResult {
            id: "r1".to_owned(),
            tool: "report",
            input: Value::Null,
            node: Node::obj(vec![
                ("rows", Node::Keyed(rows)),
                ("totals", Node::val(Val::Count(2_000))),
            ]),
            shown: None,
            source: None,
            partner_parts_locked: false,
            partner_data: false,
            turn: 0,
            working: None,
        };
        let mut result = result;
        fit(&mut result, &words);
        let json = tool_result_json(&result, &words);
        assert!(serde_json::to_string(&json).unwrap().len() <= MAX_RESULT_BYTES);
        assert_eq!(json["rows_cut"], true);
        // The totals still cover the whole filter.
        assert_eq!(json["totals"], json!({"n": 2_000}));
        let kept = json["rows"].as_array().unwrap().len();
        assert!(kept > 0 && kept < 2_000);
        assert_eq!(json["id"], "r1");
    }

    #[test]
    fn a_locked_partner_result_says_so() {
        let words = Words::new().unwrap();
        let result = ToolResult {
            id: "r2".to_owned(),
            tool: "report",
            input: Value::Null,
            node: Node::obj(vec![]),
            shown: None,
            source: None,
            partner_parts_locked: true,
            partner_data: false,
            turn: 0,
            working: None,
        };
        assert_eq!(tool_result_json(&result, &words)["partner_parts"], "locked");
    }
}
