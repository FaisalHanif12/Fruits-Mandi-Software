//! The dues guard (plan 80, R6-x): an answer to "kul ugrahi" must cite the tile `total_dues`, never
//! the buyers' dues alone.
//!
//! Since PRD v1.44 the shop's کل اگراہی is the buyers' dues plus what sellers owe plus the dead
//! amount plus the open money lent (PRD 9.1, FR-19.9). The tile `buyers_dues` is one part of it. A
//! model that cites `{{r1.buyers_dues.amount}}` for "kul ugrahi" shows a real, checked figure that is
//! short of the total by the money lent and the rest: right figure, wrong thing (NFR-AI-1).
//!
//! When the question asks the total dues ([`crate::understand::asks_total_dues`]) and the answer
//! cites the `buyers_dues.amount` of a tiles result of this question that also holds `total_dues`,
//! without citing the total anywhere, the reference is rewritten to `total_dues.amount` of the same
//! result. Nothing is worked out or changed: only which existing cell is cited. An answer that
//! cites the total as well (a breakdown with its parts) is left as it is.
//!
//! Example: the model cites `{{r1.buyers_dues.amount}}` for "kul ugrahi kitni hai" with 5,000
//! lent: the guard rewrites it to `{{r1.total_dues.amount}}` and the answer shows 26,564, not 21,564.

use crate::model_answer::{Piece, pieces};
use crate::tools::State;

/// The part and the whole, as tile names.
const PART: &str = "buyers_dues";
const WHOLE: &str = "total_dues";
/// The cell of a tile that holds its figure.
const FIGURE: &str = "amount";

/// True when the reference is the figure of the tile `tile` of a tiles result of this question.
fn is_tile_figure(state: &State, path: &str, tile: &str) -> bool {
    let mut parts = path.split('.');
    let (Some(id), Some(name), Some(cell), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return false;
    };
    name == tile
        && cell == FIGURE
        && state
            .find(id)
            .is_some_and(|r| r.turn == state.turn && r.tool == "tiles")
}

/// The repairs for an answer to a question: each (wrong reference, right reference), empty when the
/// question does not ask the total dues, the answer cites the total itself, or nothing cites the part.
pub fn plan(question_asks_total: bool, text: &str, state: &State) -> Vec<(String, String)> {
    if !question_asks_total {
        return Vec::new();
    }
    let cited: Vec<String> = pieces(text)
        .into_iter()
        .filter_map(|piece| match piece {
            Piece::Ref(r) => Some(r.path),
            Piece::Words(_) => None,
        })
        .collect();
    if cited.iter().any(|path| is_tile_figure(state, path, WHOLE)) {
        return Vec::new();
    }
    let mut repairs: Vec<(String, String)> = Vec::new();
    for path in &cited {
        if !is_tile_figure(state, path, PART) {
            continue;
        }
        let Some((id, _)) = path.split_once('.') else {
            continue;
        };
        // The same result must hold the total, so the right cell exists.
        let right = format!("{id}.{WHOLE}.{FIGURE}");
        let holds_total = state
            .find(id)
            .is_some_and(|r| r.node.get(&[WHOLE, FIGURE]).is_some());
        if holds_total && !repairs.iter().any(|(wrong, _)| wrong == path) {
            repairs.push((path.clone(), right));
        }
    }
    repairs
}
