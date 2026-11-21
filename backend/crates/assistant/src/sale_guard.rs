//! The sale measure guard (plan 82, 21-11-2026): an answer to "safi bikri" must cite the NET column, and an
//! answer to "gahakon ke bill" the BUYER TOTAL, never another column of the same books.
//!
//! The live check answered "ab tak ki safi bikri kitni hui" with the GROSS sale 382,000 (net 332,780), and
//! "safi bikri batao" once with the buyers' bills 407,750. Every figure was real and checked, and each was
//! the wrong thing: the reference check proves a figure is in the books, not that it is the column the
//! question named. The same happened for "gahakon ke bill" (answered with the buyers' balances 294,490) and
//! for one buyer's bills (answered with his balance).
//!
//! When the words of the question name the net sale or the buyers' bills
//! ([`crate::understand::asked_sale_measure`]), every cited money cell of a view, a report or a tile that is
//! another column of the sale is checked:
//!
//! | The question asks | Right column | Refused (a sale figure of another column) |
//! |---|---|---|
//! | net sale (صافی بکری) | `net` | `gross`, `value`, `purchase_value`, `buyer_total`, `deductions`, the sales tiles |
//! | the buyers' bills | `buyer_total` | `gross`, `net`, `value`, `purchase_value`, `deductions`, the commission columns, a balance, the dues tiles |
//!
//! - When the SAME row (or the totals) of the SAME result has the right column, the reference is rewritten to
//!   it by the app: only which existing cell is cited changes, nothing is worked out (Code Standards Q5).
//! - Otherwise the answer is refused with the references of the right column that this question's reads hold
//!   (`cite_instead`), so the model's one rewrite cites them, or reads the report that has the column.
//! - A balance or a commission is not refused when the question asks it too ("mani ka bill aur bakaya").
//!
//! The words come from the question alone, like the dues guard, so the guard works for every question,
//! whether or not the app understood it first.
//!
//! Example: for "ab tak ki safi bikri kitni hui" the model cites `{{r2.totals.gross}}` of R6 (net 332,780 is
//! `{{r2.totals.net}}` of the same result): the reference is rewritten and the answer shows 332,780.

use crate::check::Failure;
use crate::measure_guard::Measure;
use crate::model_answer::{Piece, pieces};
use crate::results::{Node, Val};
use crate::tools::State;
use crate::understand::{SaleAsk, SaleMeasure};

/// The sale columns of the views and reports.
const GROSS_LIKE: [&str; 5] = [
    "gross",
    "value",
    "purchase_value",
    "deductions",
    "seller_amount",
];
const COMMISSION: [&str; 5] = [
    "total_comm",
    "buyer_comm",
    "seller_comm",
    "laga",
    "commission",
];

fn right_column(measure: SaleMeasure) -> &'static str {
    match measure {
        SaleMeasure::Net => "net",
        SaleMeasure::Bills => "buyer_total",
    }
}

/// A reference split into the result id, the path to the row (or `totals`) and the column.
fn split(path: &str) -> Option<(&str, &str, &str)> {
    let (id, rest) = path.split_once('.')?;
    let (row, column) = rest.rsplit_once('.')?;
    Some((id, row, column))
}

/// True when the node at `row.column` of the result `id` of this question is a money cell.
fn money_at(state: &State, id: &str, row: &str, column: &str) -> bool {
    let Some(result) = state
        .find(id)
        .filter(|r| r.turn == state.turn && matches!(r.tool, "analytics" | "report"))
    else {
        return false;
    };
    let mut path: Vec<&str> = row.split('.').collect();
    path.push(column);
    matches!(result.node.get(&path), Some(Node::Val(Val::Money(_))))
}

/// What a cited reference is, for the question's measure.
#[derive(PartialEq, Eq)]
enum Kind {
    /// Not a cell this guard judges (a name, a date, a count, a cell of another source).
    Other,
    /// The right column.
    Right,
    /// Another column of the sale, with the column name.
    Sale,
    /// A commission or a Laga.
    Commission,
    /// A balance or the dues: what a party owes now, not what he was billed.
    Balance,
}

fn kind_of(state: &State, path: &str, measure: SaleMeasure) -> Kind {
    let right = right_column(measure);
    let Some((id, rest)) = path.split_once('.') else {
        return Kind::Other;
    };
    let Some(result) = state.find(id).filter(|r| r.turn == state.turn) else {
        return Kind::Other;
    };
    match result.tool {
        // The tiles hold the gross sales of a day and the dues: never the net or the bills.
        "tiles" => {
            let tile = rest.split('.').next().unwrap_or_default();
            match tile {
                "today_sales" => Kind::Sale,
                "buyers_dues" | "total_dues" | "sellers_dues" => Kind::Balance,
                _ => Kind::Other,
            }
        }
        // A party's card: his balance, his payable and his advance are what he owes or is owed.
        "party_card" => {
            if matches!(rest, "balance" | "payable" | "advance") {
                Kind::Balance
            } else {
                Kind::Other
            }
        }
        "analytics" | "report" => {
            let column = path.rsplit('.').next().unwrap_or_default();
            if column == right {
                Kind::Right
            } else if GROSS_LIKE.contains(&column) || column == "net" || column == "buyer_total" {
                Kind::Sale
            } else if COMMISSION.contains(&column) {
                Kind::Commission
            } else if column == "balance" {
                Kind::Balance
            } else {
                Kind::Other
            }
        }
        _ => Kind::Other,
    }
}

/// The references of the right column that this question's reads hold: the totals of a result first.
fn right_cells(state: &State, measure: SaleMeasure) -> Vec<String> {
    let right = right_column(measure);
    let mut out = Vec::new();
    for result in state
        .results
        .iter()
        .filter(|r| r.turn == state.turn && matches!(r.tool, "analytics" | "report"))
    {
        if matches!(
            result.node.get(&["totals", right]),
            Some(Node::Val(Val::Money(_)))
        ) {
            out.push(format!("{}.totals.{right}", result.id));
        }
    }
    out.truncate(3);
    out
}

/// What to do with the sale figures `text` cites, for a question that names `ask`.
pub fn plan(ask: Option<SaleAsk>, text: &str, state: &State) -> Measure {
    let Some(ask) = ask else {
        return Measure::Fine;
    };
    let right = right_column(ask.measure);
    let mut repairs: Vec<(String, String)> = Vec::new();
    let mut refused: Vec<Failure> = Vec::new();
    for piece in pieces(text) {
        let Piece::Ref(r) = piece else { continue };
        let kind = kind_of(state, &r.path, ask.measure);
        let wrong = match kind {
            Kind::Other | Kind::Right => false,
            // Another column of the sale: wrong for both measures (the net is no bill, the bill no net),
            // unless the question asks that column too.
            Kind::Sale => !ask.gross_too,
            Kind::Commission => ask.measure == SaleMeasure::Bills && !ask.commission_too,
            Kind::Balance => ask.measure == SaleMeasure::Bills && !ask.balance_too,
        };
        if !wrong {
            continue;
        }
        // The same row of the same result holds the right column: only the cited cell changes.
        if let Some((id, row, _)) = split(&r.path)
            && kind == Kind::Sale
            && money_at(state, id, row, right)
        {
            let fixed = format!("{id}.{row}.{right}");
            if !repairs.iter().any(|(wrong, _)| *wrong == r.path) {
                repairs.push((r.path.clone(), fixed));
            }
            continue;
        }
        let mut failure = Failure::measure(&r.path, None);
        failure.instead = right_cells(state, ask.measure);
        refused.push(failure);
    }
    if !refused.is_empty() {
        return Measure::Refuse(refused);
    }
    if repairs.is_empty() {
        Measure::Fine
    } else {
        Measure::Repair(repairs)
    }
}
