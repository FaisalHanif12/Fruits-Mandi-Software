//! The measure guard (D7, plan 59 F1): an answer to "how much commission was taken from this buyer"
//! must cite the buyer side, never the total of both sides and Laga.
//!
//! Q2062 "pichle mahine Faisal se kele par kitna commission liya" read the right view (view A, Faisal,
//! banana, October) and the model cited its `total_comm`: 15,716, the seller commission 7,586 plus the
//! buyer commission 7,586 plus Laga 544. Commission taken from the buyer is his own column,
//! `buyer_comm`, 7,586. The scope guard checks the party, the item and the dates a read carries, not
//! which COLUMN of its row is cited, so the figure was real, checked, and for the wrong thing.
//!
//! When the app understood that the question asks one side's commission (`Understanding::commission_side`),
//! a cited analytics figure of the other column is not shown as it is:
//!
//! - If the answer cites the `total_comm` of a row (or of the totals), no word of it asks for the whole,
//!   and the same row of the same result has the right column, the reference is rewritten to it by the
//!   app and every check runs again. Nothing is worked out or changed: only which existing cell is
//!   cited.
//! - Anything else is refused with the reference to cite, and the model writes it again once: the
//!   other side's column (a deliberate "بیوپاری کمیشن" must not be turned into another number under the
//!   same words), more than one commission column of a row (a breakdown), a right column that is not in
//!   the result (an empty cell of a side filter), a "total" word beside the figure, and the figures
//!   made from the total (`compare`, `change`, `share`).
//!
//! The total (`total_comm`) stays the figure of a question that names no side. Nothing is added or
//! rounded here (Code Standards Q5).
//!
//! Example: the model cites `{{r4.totals.total_comm}}` for the question above; the guard rewrites it to
//! `{{r4.totals.buyer_comm}}` and the answer shows 7,586.

use crate::check::Failure;
use crate::model_answer::{Piece, pieces};
use crate::results::{Node, Val};
use crate::tools::State;
use crate::understand::CommissionSide;

/// The commission columns of the analytics views.
const COLUMNS: [&str; 3] = ["total_comm", "seller_comm", "buyer_comm"];

/// The last names of the figures a view works out from its total commission: the compared period, the
/// change and the share of all commission. They are no figure of one side.
const FROM_TOTAL: [&str; 5] = ["compare", "change", "difference", "earlier", "share"];

/// Words in the answer that say the whole is meant, so the app never rewrites its figure.
const WHOLE_WORDS: [&str; 9] = [
    "total",
    "kul",
    "کل",
    "ٹوٹل",
    "دونوں",
    "both",
    "combined",
    "overall",
    "dono",
];

/// The column that is the figure of a side.
fn right_column(side: CommissionSide) -> &'static str {
    match side {
        CommissionSide::Buyer => "buyer_comm",
        CommissionSide::Seller => "seller_comm",
    }
}

/// The word the `side` filter of a view takes for a side.
fn right_side_word(side: CommissionSide) -> &'static str {
    match side {
        CommissionSide::Buyer => "buyer",
        CommissionSide::Seller => "seller",
    }
}

/// The `measure` of the estimator that is a side's commission.
fn right_estimate_measure(side: CommissionSide) -> &'static str {
    match side {
        CommissionSide::Buyer => "commission_buyer",
        CommissionSide::Seller => "commission_seller",
    }
}

/// What the guard decided about an answer's text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Measure {
    /// Nothing to change: no commission figure of the wrong column is cited.
    Fine,
    /// Cited figures of the wrong column, each with the reference of the right one in its row: the
    /// references are rewritten (wrong path, right path).
    Repair(Vec<(String, String)>),
    /// The answer cannot be put right by the app: write it again.
    Refuse(Vec<Failure>),
}

/// A reference split into the result id, the path to the row (or `totals`) and the column.
fn split(path: &str) -> Option<(&str, &str, &str)> {
    let (id, rest) = path.split_once('.')?;
    let (row, column) = rest.rsplit_once('.')?;
    Some((id, row, column))
}

/// True when the result is a view or a report of this question's reads: the commission columns are in
/// the analytics views and in report R7 (money review).
fn is_read(result: &crate::results::ToolResult, turn: u32) -> bool {
    result.turn == turn && matches!(result.tool, "analytics" | "report")
}

/// True when the node at `row.column` of the result `id` is a figure of this question's view or report.
fn is_figure(state: &State, id: &str, row: &str, column: &str) -> bool {
    let Some(result) = state.find(id).filter(|r| is_read(r, state.turn)) else {
        return false;
    };
    let mut path: Vec<&str> = row.split('.').collect();
    path.push(column);
    matches!(result.node.get(&path), Some(Node::Val(Val::Money(_))))
}

/// The failures that stand for repairs the app did not (or could not) make: each wrong reference, with
/// the right one to cite.
pub fn repair_failures(repairs: &[(String, String)]) -> Vec<Failure> {
    repairs
        .iter()
        .map(|(wrong, right)| Failure::measure(wrong, Some(right.clone())))
        .collect()
}

/// True when a `calculate` result (`c1`) was made from commission cells of a column that is not the
/// side's: its figure is made of the wrong thing, whatever its own name.
fn calc_of_wrong_column(
    state: &State,
    id: &str,
    right: &str,
    side: CommissionSide,
    laga_asked: bool,
    depth: u8,
) -> bool {
    let Some(result) = state
        .find(id)
        .filter(|r| r.turn == state.turn && r.tool == "calculate")
    else {
        return false;
    };
    let Some(inputs) = result.input.get("inputs").and_then(|v| v.as_array()) else {
        return false;
    };
    inputs.iter().filter_map(|v| v.as_str()).any(|path| {
        // A calculation inside this one (`c1.result`), a few levels deep.
        if let Some((inner, _)) = path.split_once('.')
            && depth < 3
            && calc_of_wrong_column(
                state,
                inner,
                right,
                side,
                laga_asked,
                depth.saturating_add(1),
            )
        {
            return true;
        }
        // A Laga added to a commission when no Laga was asked (`buyer_comm` + `laga`) is no figure of one
        // side's commission, though neither input is the wrong column (money review 8).
        if !laga_asked
            && path.rsplit('.').next() == Some("laga")
            && path
                .split_once('.')
                .is_some_and(|(rid, _)| state.find(rid).is_some_and(|r| is_read(r, state.turn)))
        {
            return true;
        }
        // A wildcard input (`r4.rows.*.total_comm`) names every row: its column is the figure.
        if path.contains(".*")
            && let Some((rid, _)) = path.split_once('.')
            && state.find(rid).is_some_and(|r| is_read(r, state.turn))
            && path
                .rsplit('.')
                .next()
                .is_some_and(|column| COLUMNS.contains(&column) && column != right)
        {
            return true;
        }
        not_a_side_source(state, path, side, laga_asked)
            || split(path).is_some_and(|(rid, row, column)| {
                COLUMNS.contains(&column) && column != right && is_figure(state, rid, row, column)
            })
    })
}

/// True when the reference points into a source that has no figure of ONE side's commission, so a
/// commission figure taken from it is the total of both sides and Laga under another name (money
/// review, 4th and 5th). It is the other way round from a list of names: only the views and reports that
/// do hold a side are allowed for a question that asks a side (the views A, B and C, and the reports R1,
/// R6 and R7 by their columns, checked in `plan`); every other source is refused as a whole, so a new
/// total (a tile, the series, a matrix cell, a top list, the day total sheet) can never slip through:
/// - the tiles (`today_commission`);
/// - the views D, E, G, H, I and J (matrix cells, series, peak and lowest, top lists, gardens, trips);
/// - the reports R33, R34 and R36 (trips, the day total sheet, gardens);
/// - for a buyer-side question, the seller's `commission` column of R1 and R6;
/// - an `estimate` of another commission than the side's (`commission_all`, `laga`, the other side).
///
/// The model re-reads what it needs (view A with the party and item) and writes again, once.
fn not_a_side_source(state: &State, path: &str, side: CommissionSide, laga_asked: bool) -> bool {
    let Some((id, rest)) = path.split_once('.') else {
        return false;
    };
    let Some(result) = state.find(id).filter(|r| r.turn == state.turn) else {
        return false;
    };
    let input = |field: &str| result.input.get(field).and_then(|v| v.as_str());
    match result.tool {
        "tiles" => true,
        // An estimate of the commission of both sides, of Laga, or of the other side is not this side's.
        "estimate" => {
            let measure = input("measure");
            // The Laga of the buyers is asked too: its estimate is the figure of that part.
            if measure == Some("laga") && laga_asked && side == CommissionSide::Buyer {
                return false;
            }
            matches!(
                measure,
                Some("commission_all" | "commission_seller" | "commission_buyer" | "laga")
            ) && measure != Some(right_estimate_measure(side))
        }
        // These views take a side filter: for a seller side, read with it, their cells ARE the seller
        // commission (the estimator reads view G that way); read with no side or the other, they hold the
        // total.
        // For a BUYER side the side filter gives the buyer commission PLUS Laga in these views (their
        // figure is `total_comm` of the side), so they stay refused whatever the filter: the model cites
        // the `buyer_comm` column of view A or C or of R7 (money review, 7th). For a seller side the
        // filter leaves the seller commission alone.
        "analytics" => {
            matches!(input("view"), Some("D" | "E" | "G" | "H" | "I" | "J"))
                && (side == CommissionSide::Buyer || input("side") != Some(right_side_word(side)))
        }
        "report" => match input("id") {
            Some("R33" | "R34" | "R36") => true,
            Some("R1" | "R6") => {
                side == CommissionSide::Buyer && rest.rsplit('.').next() == Some("commission")
            }
            _ => false,
        },
        _ => false,
    }
}

/// What to do with the commission figures `text` cites, for a question that asks `side`. `laga_asked` is
/// true when the question asks a Laga too: the total of the row then stands for the commission and the
/// Laga together, which is no figure of one side, so it is refused and never rewritten.
pub fn plan(text: &str, state: &State, side: CommissionSide, laga_asked: bool) -> Measure {
    let right = right_column(side);
    let says_whole = pieces(text).iter().any(|p| match p {
        Piece::Words(w) => w
            .split(|c: char| !c.is_alphanumeric())
            .any(|word| WHOLE_WORDS.contains(&word)),
        Piece::Ref(_) => false,
    });
    // A Laga cited beside the commission: `total_comm` then stands for the commission and the Laga
    // together, which is not the figure of one side either (it holds the seller's commission), so it is
    // refused and never rewritten.
    let cites_laga = laga_asked
        || pieces(text).iter().any(|p| match p {
            Piece::Ref(r) => r.path.ends_with(".laga"),
            Piece::Words(_) => false,
        });
    let mut wrong: Vec<(String, String)> = Vec::new();
    let mut rows_with_right: Vec<String> = Vec::new();
    let mut refused: Vec<Failure> = Vec::new();
    let mut columns_by_row: Vec<(String, Vec<&str>)> = Vec::new();
    for piece in pieces(text) {
        let Piece::Ref(r) = piece else { continue };
        if not_a_side_source(state, &r.path, side, laga_asked) {
            refused.push(Failure::measure(&r.path, None));
            continue;
        }
        let Some((id, row, column)) = split(&r.path) else {
            // `c1.result`: a calculation made from the other column.
            if let Some((id, _)) = r.path.split_once('.')
                && calc_of_wrong_column(state, id, right, side, laga_asked, 0)
            {
                refused.push(Failure::measure(&r.path, None));
            }
            continue;
        };
        // A figure worked out from the total of both sides is no figure of one side.
        if FROM_TOTAL.contains(&column) && state.find(id).is_some_and(|x| is_read(x, state.turn)) {
            refused.push(Failure::measure(&r.path, None));
            continue;
        }
        let Some(column) = COLUMNS.iter().find(|c| **c == column) else {
            continue;
        };
        if !is_figure(state, id, row, column) {
            continue;
        }
        let key = format!("{id}.{row}");
        match columns_by_row.iter_mut().find(|(k, _)| *k == key) {
            Some((_, cols)) => {
                if !cols.contains(column) {
                    cols.push(column);
                }
            }
            None => columns_by_row.push((key.clone(), vec![column])),
        }
        if *column == right {
            rows_with_right.push(key);
            continue;
        }
        let fixed = format!("{id}.{row}.{right}");
        if !is_figure(state, id, row, right) {
            refused.push(Failure::measure(&r.path, None));
        } else if *column == "total_comm" && !says_whole && !cites_laga {
            wrong.push((r.path.clone(), fixed));
        } else {
            // The other side's column, or a word that asks for the whole: the model decides.
            refused.push(Failure::measure(&r.path, Some(fixed)));
        }
    }
    // A row cited by two commission columns is a breakdown, or a mix: the app does not choose.
    for (key, cols) in &columns_by_row {
        if cols.len() > 1 && wrong.iter().any(|(w, _)| w.starts_with(key.as_str())) {
            for (path, fixed) in std::mem::take(&mut wrong) {
                if path.starts_with(key.as_str()) {
                    refused.push(Failure::measure(&path, Some(fixed)));
                } else {
                    wrong.push((path, fixed));
                }
            }
        }
    }
    if !refused.is_empty() {
        return Measure::Refuse(refused);
    }
    if wrong.is_empty() {
        Measure::Fine
    } else {
        Measure::Repair(wrong)
    }
}

/// The text with each wrong reference written as the right one. `None` when nothing changed.
pub fn rewrite(text: &str, repairs: &[(String, String)]) -> Option<String> {
    let mut changed = false;
    let mut out = String::new();
    for piece in pieces(text) {
        match piece {
            Piece::Words(w) => out.push_str(&w),
            Piece::Ref(r) => match repairs.iter().find(|(wrong, _)| *wrong == r.path) {
                Some((_, right)) => {
                    out.push_str(&format!("{{{{{right}}}}}"));
                    changed = true;
                }
                None => out.push_str(text.get(r.start..r.end).unwrap_or("")),
            },
        }
    }
    changed.then_some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_reference_is_split_into_result_row_and_column() {
        assert_eq!(
            split("r4.rows.i1.total_comm"),
            Some(("r4", "rows.i1", "total_comm"))
        );
        assert_eq!(
            split("r4.totals.buyer_comm"),
            Some(("r4", "totals", "buyer_comm"))
        );
        assert_eq!(split("r4"), None);
        assert_eq!(split("r4.total_comm"), None);
    }

    #[test]
    fn only_the_cited_reference_is_rewritten_and_nothing_else_changes() {
        let repairs = vec![(
            "r4.totals.total_comm".to_owned(),
            "r4.totals.buyer_comm".to_owned(),
        )];
        assert_eq!(
            rewrite(
                "اس سے {{r4.totals.total_comm}} اور {{r4.totals.laga}}",
                &repairs
            )
            .as_deref(),
            Some("اس سے {{r4.totals.buyer_comm}} اور {{r4.totals.laga}}")
        );
        assert_eq!(rewrite("{{r4.totals.laga}}", &repairs), None);
        assert_eq!(rewrite("no references", &repairs), None);
    }

    #[test]
    fn the_right_column_of_a_side_is_its_own() {
        assert_eq!(right_column(CommissionSide::Buyer), "buyer_comm");
        assert_eq!(right_column(CommissionSide::Seller), "seller_comm");
    }
}
