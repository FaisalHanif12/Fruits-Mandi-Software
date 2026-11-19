//! The button of a party's commission (PRD FR-7.90 v1.42, FR-9.3, plan 77).
//!
//! An answer about one party's commission ("rahim se is mahine kitna commission aur bikri", "akram
//! se kitna commission mila") reads his row of analytics view B (a seller) or view C (a buyer). The
//! commission figures it cites are those of the commission page of that party for the same dates,
//! so the button opens that page, with the side, the party and the dates already chosen, in place
//! of the analytics view. Nothing is read or worked out here: the source is only turned into the
//! one the screen opens.
//!
//! Which answer: one whose text cites a commission column of the result the source was read for
//! (`seller_comm` or `total_comm` of view B, `buyer_comm` of view C, `buyer_comm` or `total_comm` of
//! view A read with a buyer and his side). A sale or a laga alone keeps the analytics button.
//!
//! Example: "rahim se is mahine kitna commission aur bikri" read view B for رحیم (0010) from
//! 2026-10-01 to 2026-10-10 and cites `{{r3.rows.a19.total_comm}}`; the button becomes the
//! commission page of the seller رحیم for those dates. With no dates in the question, `from` and
//! `to` are empty and the page opens over all dates.

use mandi_reports::analytics::{AnalyticsCompare, AnalyticsGoods, AnalyticsSide, AnalyticsView};
use mandi_reports::dashboard::CommissionSide;

use crate::answer::{Answer, AnswerSource};
use crate::model_answer::{Piece, pieces};
use crate::source::SourceParty;

/// The commission columns of a seller's view (B), of a buyer's view (C) and of the view of a buyer's
/// side (A with the buyer and his side, which holds his commission and the total with his Laga).
const SELLER_COLUMNS: [&str; 2] = ["seller_comm", "total_comm"];
const BUYER_COLUMNS: [&str; 1] = ["buyer_comm"];
const BUYER_SIDE_COLUMNS: [&str; 2] = ["buyer_comm", "total_comm"];

/// True when the text cites one of `columns` of the result `result_id` (`r3.rows.a19.total_comm`
/// or `r3.totals.total_comm`).
fn cites(text: &str, result_id: &str, columns: &[&str]) -> bool {
    pieces(text).into_iter().any(|piece| match piece {
        Piece::Ref(r) => {
            let mut path = r.path.split('.');
            path.next() == Some(result_id) && path.next_back().is_some_and(|c| columns.contains(&c))
        }
        Piece::Words(_) => false,
    })
}

/// Turns the analytics button of a party's commission into the commission page of that party.
///
/// Only the view read for the party alone is turned (view B with a seller, view C with a buyer),
/// with both dates or neither, and only when nothing else narrows it: a period shortcut, an item,
/// a group, a quality, a city, an اڈا, a trip, a garden, a kind of goods or a comparison would make
/// the page show other figures than the answer. The page keeps no commission-side choice: it shows
/// the commission charged to the seller and the commission on his goods in two columns.
pub fn apply(answer: &mut Answer, text: &str) {
    for source in &mut answer.sources {
        let AnswerSource::Analytics {
            result_id,
            view,
            filters,
            ..
        } = source
        else {
            continue;
        };
        let dated = filters.from.is_empty() == filters.to.is_empty();
        let alone = filters.period.is_none()
            && dated
            && filters.item.is_none()
            && filters.item_group.is_none()
            && filters.grade.is_none()
            && filters.city.is_none()
            && filters.adda.is_none()
            && filters.partner.is_none()
            && filters.trip.is_none()
            && filters.garden.is_none()
            && filters.goods == AnalyticsGoods::All
            && filters.compare == AnalyticsCompare::Off;
        if !alone {
            continue;
        }
        let page = match view {
            AnalyticsView::B
                if filters.buyer.is_none() && cites(text, result_id, &SELLER_COLUMNS) =>
            {
                filters.seller.clone().map(|record| {
                    (
                        CommissionSide::Seller,
                        SourceParty {
                            record,
                            person_key: None,
                        },
                    )
                })
            }
            AnalyticsView::C
                if filters.seller.is_none() && cites(text, result_id, &BUYER_COLUMNS) =>
            {
                filters
                    .buyer
                    .clone()
                    .map(|party| (CommissionSide::Buyer, party))
            }
            AnalyticsView::A
                if filters.seller.is_none()
                    && filters.side == AnalyticsSide::Buyer
                    && cites(text, result_id, &BUYER_SIDE_COLUMNS) =>
            {
                filters
                    .buyer
                    .clone()
                    .map(|party| (CommissionSide::Buyer, party))
            }
            _ => None,
        };
        if let Some((side, party)) = page {
            *source = AnswerSource::Commission {
                result_id: result_id.clone(),
                side,
                party,
                from: filters.from.clone(),
                to: filters.to.clone(),
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_a_commission_column_of_that_result_counts() {
        let text = "{{r2.matches.0.name}} {{r3.rows.a19.total_comm}} {{r3.rows.a19.gross}}";
        assert!(cites(text, "r3", &SELLER_COLUMNS));
        // Another result's commission does not count for this one.
        assert!(!cites(text, "r4", &SELLER_COLUMNS));
        // A sale alone is no commission.
        assert!(!cites("{{r3.rows.a19.gross}}", "r3", &SELLER_COLUMNS));
        // The totals row of the result counts too.
        assert!(cites("{{r3.totals.seller_comm}}", "r3", &SELLER_COLUMNS));
        // A buyer's commission is its own column; a laga alone does not open the page.
        assert!(cites("{{r3.rows.a21.buyer_comm}}", "r3", &BUYER_COLUMNS));
        assert!(!cites("{{r3.rows.a21.laga}}", "r3", &BUYER_COLUMNS));
        // The totals row of the buyer's side: his commission and the total with his Laga.
        assert!(cites("{{r3.totals.total_comm}}", "r3", &BUYER_SIDE_COLUMNS));
        assert!(!cites("{{r3.totals.laga}}", "r3", &BUYER_SIDE_COLUMNS));
    }
}
