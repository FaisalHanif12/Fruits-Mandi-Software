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
        SaleMeasure::Gross => "gross",
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
            } else if measure == SaleMeasure::Gross
                && matches!(column, "purchase_value" | "seller_amount")
            {
                // A buyer's own purchase and the seller's amount are no other column of the gross sale.
                Kind::Other
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

/// The references of `column` that this question's reads hold: the totals of a result first.
fn cells_of(state: &State, column: &str) -> Vec<String> {
    let mut out = Vec::new();
    for result in state
        .results
        .iter()
        .filter(|r| r.turn == state.turn && matches!(r.tool, "analytics" | "report"))
    {
        if matches!(
            result.node.get(&["totals", column]),
            Some(Node::Val(Val::Money(_)))
        ) {
            out.push(format!("{}.totals.{column}", result.id));
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
        let column = r.path.rsplit('.').next().unwrap_or_default();
        // The gross asked beside the net or the bills ("khaam bikri aur gahakon ke bill"): only the `gross`
        // column is that figure. The register R2 `value` is the buyers' goods value, which holds a rate
        // difference (live check of 21-11-2026, M18).
        let gross_asked_too = ask.gross_too && column == "gross";
        let wrong = match kind {
            Kind::Other | Kind::Right => false,
            // Another column of the sale: wrong for every measure (the net is no bill, the bill no net, the
            // gross neither), unless the question asks that column too.
            Kind::Sale => !gross_asked_too,
            Kind::Commission => ask.measure == SaleMeasure::Bills && !ask.commission_too,
            Kind::Balance => ask.measure == SaleMeasure::Bills && !ask.balance_too,
        };
        if !wrong {
            continue;
        }
        // The column the cell should have been: a goods value cited beside the gross asked too is the gross.
        let target = if ask.gross_too && column == "value" {
            "gross"
        } else {
            right
        };
        // The same row of the same result holds the right column: only the cited cell changes.
        if let Some((id, row, _)) = split(&r.path)
            && kind == Kind::Sale
            && money_at(state, id, row, target)
        {
            let fixed = format!("{id}.{row}.{target}");
            if !repairs.iter().any(|(wrong, _)| *wrong == r.path) {
                repairs.push((r.path.clone(), fixed));
            }
            continue;
        }
        let mut failure = Failure::measure(&r.path, None);
        failure.instead = cells_of(state, target);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::results::{Money, ToolResult};
    use crate::understand::asked_sale_measure;
    use mandi_services::core::money::Paisa;
    use serde_json::json;

    fn money(rupees: i64) -> Node {
        Node::val(Val::Money(Money::plain(
            Paisa::from_rupees(rupees).unwrap(),
        )))
    }

    fn result(id: &str, tool: &'static str, totals: Vec<(&str, Node)>) -> ToolResult {
        ToolResult {
            id: id.to_owned(),
            tool,
            input: json!({}),
            node: Node::obj(vec![("totals", Node::obj(totals))]),
            shown: None,
            source: None,
            partner_parts_locked: false,
            partner_data: false,
            turn: 0,
            working: None,
        }
    }

    /// The live books of M18: the register R2 holds the buyers' goods value 9,155,400 (it carries a 600 rate
    /// difference) and the bills 9,723,115; view A holds the gross 9,154,800.
    fn live_m18() -> State {
        State {
            results: vec![
                result(
                    "r1",
                    "report",
                    vec![
                        ("value", money(9_155_400)),
                        ("buyer_total", money(9_723_115)),
                    ],
                ),
                result("r2", "analytics", vec![("gross", money(9_154_800))]),
                result("r3", "analytics", vec![("purchase_value", money(36_000))]),
            ],
            turn: 0,
        }
    }

    fn refused(plan: Measure) -> Vec<Failure> {
        match plan {
            Measure::Refuse(failures) => failures,
            Measure::Fine => panic!("nothing was refused"),
            Measure::Repair(repairs) => panic!("repaired: {repairs:?}"),
        }
    }

    #[test]
    fn the_register_goods_value_is_refused_for_the_gross_asked_beside_the_bills() {
        let ask = asked_sale_measure(
            "pichle mahine ki khaam bikri aur gahakon ke kul bill alag alag batao",
        );
        let state = live_m18();
        // The wrong gross: refused, with the gross of view A to cite instead.
        let failures = refused(plan(
            ask,
            "{{r1.totals.value}} {{r1.totals.buyer_total}}",
            &state,
        ));
        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0].at, "r1.totals.value");
        assert_eq!(failures[0].instead, ["r2.totals.gross"]);
        // The right gross and the bills together: fine.
        assert!(matches!(
            plan(ask, "{{r2.totals.gross}} {{r1.totals.buyer_total}}", &state),
            Measure::Fine
        ));
    }

    #[test]
    fn the_gross_asked_alone_is_the_gross_column_and_not_the_register_value() {
        let ask = asked_sale_measure("ehsan ki khaam bikri kitni hai");
        assert_eq!(ask.map(|a| a.measure), Some(SaleMeasure::Gross));
        let state = live_m18();
        let failures = refused(plan(ask, "{{r1.totals.value}}", &state));
        assert_eq!(failures[0].instead, ["r2.totals.gross"]);
        // A bill or a net cited for the gross is the same fault.
        refused(plan(ask, "{{r1.totals.buyer_total}}", &state));
        // The gross itself, and a buyer's own purchase value, are fine.
        assert!(matches!(
            plan(ask, "{{r2.totals.gross}}", &state),
            Measure::Fine
        ));
        assert!(matches!(
            plan(ask, "{{r3.totals.purchase_value}}", &state),
            Measure::Fine
        ));
    }

    #[test]
    fn a_question_that_names_no_gross_word_keeps_its_old_guard() {
        // "gahakon ke kul bill": the register value is still a wrong column for the bills, and the repair of a
        // row that holds the right column is unchanged.
        let ask = asked_sale_measure("abhi tak gahakon ke kul bill kitne ke bane");
        assert_eq!(ask.map(|a| a.measure), Some(SaleMeasure::Bills));
        match plan(ask, "{{r1.totals.value}}", &live_m18()) {
            Measure::Repair(repairs) => assert_eq!(
                repairs,
                [(
                    "r1.totals.value".to_owned(),
                    "r1.totals.buyer_total".to_owned()
                )]
            ),
            _ => panic!("the goods value of R2 is put right to its bill total"),
        }
        assert!(asked_sale_measure("aaj ki bikri kitni hui").is_none());
    }
}
