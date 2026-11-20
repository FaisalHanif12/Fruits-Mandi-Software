//! What a view counts (PRD FR-18.10, plan 84): one figure of a group of sale facts, as money or as a
//! count, with checked sums.
//!
//! A [`Figure`] is the value of one measure ([`AnalyticsMeasure`]): the seller amounts, the buyers'
//! bills and the commissions are `Paisa`; the quantity is a plain count, never money. Every sum goes
//! through `Paisa::checked_add` (money agent M7); a count goes through `i64::checked_add`. Two
//! figures of different kinds are never added (a data error, not a silent mix).
//!
//! The figures come only from the stored facts (`facts::Sums`, FR-18.6): the buyer total is the
//! stored buyer value plus the stored buyer commission plus the stored لاگا (C5), never worked out
//! again with today's rules.
//!
//! Example (PRD 18.4, کیلا): gross 29,400; buyer total 29,400 + 1,764 + 400 = 31,564; qty 50; total
//! commission 3,928.

use mandi_core::money::{Paisa, hundredths};

use std::collections::HashMap;

use super::facts::{Group, Sums};
use super::filters::Typed;
use super::model::{AnalyticsColumn, AnalyticsMeasure, AnalyticsSide};
use super::table::col;
use crate::error::ReportsError;
use crate::reports::{Cell, ColumnKind};

/// One figure of a measure: money (whole rupees as paisa) or a count.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Figure {
    /// Money.
    Money(Paisa),
    /// A count (the quantity).
    Count(i64),
}

fn data(what: &str) -> ReportsError {
    ReportsError::Data(what.to_owned())
}

impl Figure {
    /// Nothing, of the kind of the measure.
    pub(crate) fn zero(measure: AnalyticsMeasure) -> Self {
        match measure.kind() {
            ColumnKind::Number => Self::Count(0),
            _ => Self::Money(Paisa::ZERO),
        }
    }

    /// This figure plus another of the same kind, with checked maths.
    ///
    /// # Errors
    /// `ReportsError::Money` or `ReportsError::Data` when the sum does not fit or the kinds differ.
    pub(crate) fn plus(self, other: Self) -> Result<Self, ReportsError> {
        match (self, other) {
            (Self::Money(a), Self::Money(b)) => Ok(Self::Money(a.checked_add(b)?)),
            (Self::Count(a), Self::Count(b)) => a
                .checked_add(b)
                .map(Self::Count)
                .ok_or_else(|| data("a quantity total does not fit")),
            _ => Err(data("a money figure and a count were added")),
        }
    }

    /// This figure less another of the same kind, with checked maths. The result can be below 0 (a
    /// change is not a balance).
    ///
    /// # Errors
    /// As [`Figure::plus`].
    pub(crate) fn minus(self, other: Self) -> Result<Self, ReportsError> {
        match (self, other) {
            (Self::Money(a), Self::Money(b)) => Ok(Self::Money(a.checked_sub(b)?)),
            (Self::Count(a), Self::Count(b)) => a
                .checked_sub(b)
                .map(Self::Count)
                .ok_or_else(|| data("a quantity change does not fit")),
            _ => Err(data("a money figure and a count were subtracted")),
        }
    }

    /// True when nothing is counted.
    pub(crate) fn is_zero(self) -> bool {
        match self {
            Self::Money(paisa) => paisa == Paisa::ZERO,
            Self::Count(value) => value == 0,
        }
    }

    /// True when more than nothing is counted.
    pub(crate) fn is_positive(self) -> bool {
        match self {
            Self::Money(paisa) => paisa > Paisa::ZERO,
            Self::Count(value) => value > 0,
        }
    }

    /// The figure as a cell: money, or a whole number for a count.
    pub(crate) fn cell(self) -> Cell {
        match self {
            Self::Money(paisa) => Cell::Money { paisa },
            Self::Count(value) => Cell::Number { value },
        }
    }

    /// The figure as a cell, or an empty cell when it is 0.
    pub(crate) fn cell_or_empty(self) -> Cell {
        if self.is_zero() {
            Cell::Empty
        } else {
            self.cell()
        }
    }

    /// The figure in a cell, if the cell holds one (money or a whole number).
    pub(crate) fn from_cell(cell: &Cell) -> Option<Self> {
        match cell {
            Cell::Money { paisa } => Some(Self::Money(*paisa)),
            Cell::Number { value } => Some(Self::Count(*value)),
            _ => None,
        }
    }
}

/// Adds `amount` to `total`, with checked maths.
///
/// # Errors
/// As [`Figure::plus`].
pub(crate) fn add(total: &mut Figure, amount: Figure) -> Result<(), ReportsError> {
    *total = total.plus(amount)?;
    Ok(())
}

/// The measure a matrix or a ranking counts: the one asked for, else the total commission (the
/// classic view).
pub(crate) fn measure_of(typed: &Typed) -> AnalyticsMeasure {
    typed.measure.unwrap_or(AnalyticsMeasure::TotalComm)
}

/// The figure of each id from a query by one id column (the compared period of a matrix or a
/// ranking).
///
/// # Errors
/// `ReportsError::Money` if a sum does not fit.
pub(crate) fn figures_by_id(
    groups: &[Group],
    measure: AnalyticsMeasure,
    side: AnalyticsSide,
) -> Result<HashMap<i64, Figure>, ReportsError> {
    let mut found = HashMap::new();
    for group in groups {
        if let Some(id) = group.int(0) {
            found.insert(id, group.sums.figure(measure, side)?);
        }
    }
    Ok(found)
}

/// The sums of each id from a query by one id column (the compared period of a table whose compare
/// columns are the total commission, when a chart of another measure needs the compared figure).
pub(crate) fn sums_by_id(groups: &[Group]) -> HashMap<i64, Sums> {
    groups
        .iter()
        .filter_map(|group| group.int(0).map(|id| (id, group.sums)))
        .collect()
}

/// The share of a part in a whole as hundredths of a percent, for a count: the one "% of all"
/// formula of `mandi_core::money::hundredths`, with the range checks of `share_hundredths`. `None`
/// when the whole is 0 or less.
///
/// # Errors
/// `ReportsError::Data` for a part below 0 or above the whole.
fn count_share(part: i64, total: i64) -> Result<Option<i64>, ReportsError> {
    if total <= 0 {
        return Ok(None);
    }
    if part < 0 || part > total {
        return Err(data("a share is outside 0 to 100 percent"));
    }
    hundredths(i128::from(part), i128::from(total))
        .map(Some)
        .map_err(|_| data("a share does not fit"))
}

/// A share as a percent cell, or an empty cell when the whole is 0.
///
/// # Errors
/// `ReportsError::Data` for a part below 0 or above the whole, or kinds that differ.
pub(crate) fn share_cell(part: Figure, total: Figure) -> Result<Cell, ReportsError> {
    let hundredths = match (part, total) {
        (Figure::Money(part), Figure::Money(total)) => super::calc::share_hundredths(part, total)?,
        (Figure::Count(part), Figure::Count(total)) => count_share(part, total)?,
        _ => return Err(data("a share of a money figure and a count")),
    };
    Ok(hundredths.map_or(Cell::Empty, |hundredths| Cell::Percent { hundredths }))
}

/// The two cells of a comparison: the compared figure and `current − compared`.
///
/// # Errors
/// `ReportsError::Money` or `ReportsError::Data` if the difference does not fit.
pub(crate) fn compare_figure_cells(
    current: Figure,
    compared: Figure,
) -> Result<Vec<Cell>, ReportsError> {
    Ok(vec![compared.cell(), current.minus(compared)?.cell()])
}

/// The two columns a comparison adds, of the kind of the measure.
pub(crate) fn compare_columns_of(measure: AnalyticsMeasure) -> Vec<AnalyticsColumn> {
    vec![
        col("compare", super::ids::COL_COMPARE, measure.kind()),
        col("change", super::ids::COL_CHANGE, measure.kind()),
    ]
}

/// The Fluent message ID of the heading of a column that holds this measure.
pub(crate) fn title_id(measure: AnalyticsMeasure) -> &'static str {
    match measure {
        AnalyticsMeasure::Gross => super::ids::COL_GROSS,
        AnalyticsMeasure::BuyerTotal => crate::reports::ids::COL_BUYER_TOTAL,
        AnalyticsMeasure::Qty => super::ids::COL_QTY,
        AnalyticsMeasure::TotalComm => super::ids::COL_TOTAL_COMM,
        AnalyticsMeasure::SellerComm => super::ids::COL_SELLER_COMM,
        AnalyticsMeasure::BuyerComm => super::ids::COL_BUYER_COMM,
        AnalyticsMeasure::Laga => super::ids::COL_LAGA,
    }
}

/// The Fluent message ID of the heading of the row total of a matrix (D, E, L): the classic total
/// commission keeps `analytics-col-row-total`; every other measure has its own words, so a print and
/// an Excel sheet say "Total quantity", not "Total commission" (frontend plan 84).
pub(crate) fn row_total_title_id(measure: AnalyticsMeasure) -> &'static str {
    match measure {
        AnalyticsMeasure::TotalComm => super::ids::COL_ROW_TOTAL,
        AnalyticsMeasure::Gross => "analytics-col-row-total-gross",
        AnalyticsMeasure::BuyerTotal => "analytics-col-row-total-buyer-total",
        AnalyticsMeasure::Qty => "analytics-col-row-total-qty",
        AnalyticsMeasure::SellerComm => "analytics-col-row-total-seller-comm",
        AnalyticsMeasure::BuyerComm => "analytics-col-row-total-buyer-comm",
        AnalyticsMeasure::Laga => "analytics-col-row-total-laga",
    }
}

/// The row total column of a matrix: the key of the measure, of its kind.
pub(crate) fn row_total_column(measure: AnalyticsMeasure) -> AnalyticsColumn {
    col(measure.key(), row_total_title_id(measure), measure.kind())
}

/// The column of a measure in a ranking: its key and heading, of its kind.
pub(crate) fn measure_column(measure: AnalyticsMeasure) -> AnalyticsColumn {
    col(measure.key(), title_id(measure), measure.kind())
}

impl Sums {
    /// This measure of the group (PRD FR-18.10).
    ///
    /// The side filter narrows the commission measures only: the total commission is the part on the
    /// side asked for, the seller commission counts when the side counts the seller, the buyer
    /// commission and لاگا when it counts the buyer; what the sellers were credited, what the
    /// buyers were charged and the quantity are not commissions and never depend on the side.
    ///
    /// # Errors
    /// `ReportsError::Money` if a sum does not fit.
    pub(crate) fn figure(
        &self,
        measure: AnalyticsMeasure,
        side: AnalyticsSide,
    ) -> Result<Figure, ReportsError> {
        let counts_seller = matches!(side, AnalyticsSide::Both | AnalyticsSide::Seller);
        let counts_buyer = matches!(side, AnalyticsSide::Both | AnalyticsSide::Buyer);
        let part = |counted: bool, amount: Paisa| {
            Figure::Money(if counted { amount } else { Paisa::ZERO })
        };
        Ok(match measure {
            AnalyticsMeasure::Gross => Figure::Money(self.gross),
            AnalyticsMeasure::BuyerTotal => Figure::Money(
                self.buyer_value
                    .checked_add(self.buyer_comm)?
                    .checked_add(self.laga)?,
            ),
            AnalyticsMeasure::Qty => Figure::Count(self.qty),
            AnalyticsMeasure::TotalComm => Figure::Money(self.total_comm(side)?),
            AnalyticsMeasure::SellerComm => part(counts_seller, self.seller_comm),
            AnalyticsMeasure::BuyerComm => part(counts_buyer, self.buyer_comm),
            AnalyticsMeasure::Laga => part(counts_buyer, self.laga),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(rupees: i64) -> Paisa {
        Paisa::from_rupees(rupees).unwrap()
    }

    /// کیلا of the worked day (PRD 18.4): 50 نگ, 29,400, 1,764, 1,764, 400.
    fn kela() -> Sums {
        Sums {
            qty: 50,
            gross: p(29_400),
            seller_comm: p(1_764),
            buyer_value: p(29_400),
            buyer_comm: p(1_764),
            laga: p(400),
        }
    }

    #[test]
    fn the_worked_day_figures_of_kela() {
        let s = kela();
        let side = AnalyticsSide::Both;
        let f = |m| s.figure(m, side).unwrap();
        assert_eq!(f(AnalyticsMeasure::Gross), Figure::Money(p(29_400)));
        assert_eq!(f(AnalyticsMeasure::BuyerTotal), Figure::Money(p(31_564)));
        assert_eq!(f(AnalyticsMeasure::Qty), Figure::Count(50));
        assert_eq!(f(AnalyticsMeasure::TotalComm), Figure::Money(p(3_928)));
        assert_eq!(f(AnalyticsMeasure::SellerComm), Figure::Money(p(1_764)));
        assert_eq!(f(AnalyticsMeasure::BuyerComm), Figure::Money(p(1_764)));
        assert_eq!(f(AnalyticsMeasure::Laga), Figure::Money(p(400)));
    }

    #[test]
    fn the_side_narrows_the_commissions_and_nothing_else() {
        let s = kela();
        let seller = AnalyticsSide::Seller;
        let buyer = AnalyticsSide::Buyer;
        let f = |m, side| s.figure(m, side).unwrap();
        // Seller side: only the seller commission counts.
        assert_eq!(
            f(AnalyticsMeasure::TotalComm, seller),
            Figure::Money(p(1_764))
        );
        assert_eq!(
            f(AnalyticsMeasure::SellerComm, seller),
            Figure::Money(p(1_764))
        );
        assert_eq!(
            f(AnalyticsMeasure::BuyerComm, seller),
            Figure::Money(Paisa::ZERO)
        );
        assert_eq!(
            f(AnalyticsMeasure::Laga, seller),
            Figure::Money(Paisa::ZERO)
        );
        // Buyer side: the buyer commission and لاگا.
        assert_eq!(
            f(AnalyticsMeasure::TotalComm, buyer),
            Figure::Money(p(2_164))
        );
        assert_eq!(
            f(AnalyticsMeasure::SellerComm, buyer),
            Figure::Money(Paisa::ZERO)
        );
        // Sales, bills and quantity are not commissions.
        for side in [seller, buyer] {
            assert_eq!(f(AnalyticsMeasure::Gross, side), Figure::Money(p(29_400)));
            assert_eq!(
                f(AnalyticsMeasure::BuyerTotal, side),
                Figure::Money(p(31_564))
            );
            assert_eq!(f(AnalyticsMeasure::Qty, side), Figure::Count(50));
        }
    }

    #[test]
    fn the_row_total_heading_is_the_old_one_for_the_total_commission_and_its_own_for_the_rest() {
        let id = |m| row_total_title_id(m);
        assert_eq!(id(AnalyticsMeasure::TotalComm), "analytics-col-row-total");
        assert_eq!(id(AnalyticsMeasure::Qty), "analytics-col-row-total-qty");
        let c = row_total_column(AnalyticsMeasure::Qty);
        assert_eq!((c.key.as_str(), c.kind), ("qty", ColumnKind::Number));
    }

    #[test]
    fn a_count_is_a_number_cell_and_money_is_a_money_cell() {
        assert_eq!(Figure::Count(50).cell(), Cell::Number { value: 50 });
        assert_eq!(Figure::Money(p(10)).cell(), Cell::Money { paisa: p(10) });
        assert_eq!(Figure::Count(0).cell_or_empty(), Cell::Empty);
        assert_eq!(Figure::Money(Paisa::ZERO).cell_or_empty(), Cell::Empty);
        assert_eq!(AnalyticsMeasure::Qty.kind(), ColumnKind::Number);
        assert_eq!(AnalyticsMeasure::Gross.kind(), ColumnKind::Money);
    }

    #[test]
    fn sums_are_checked_and_never_mix_money_with_a_count() {
        let mut total = Figure::Count(i64::MAX);
        assert!(add(&mut total, Figure::Count(1)).is_err());
        let mut money = Figure::Money(p(1));
        assert!(add(&mut money, Figure::Count(1)).is_err());
        assert!(money.minus(Figure::Count(1)).is_err());
        let mut ok = Figure::Count(40);
        add(&mut ok, Figure::Count(10)).unwrap();
        assert_eq!(ok, Figure::Count(50));
    }

    #[test]
    fn the_compare_cells_are_the_compared_figure_and_the_change() {
        let cells = compare_figure_cells(Figure::Count(60), Figure::Count(75)).unwrap();
        assert_eq!(
            cells,
            vec![Cell::Number { value: 75 }, Cell::Number { value: -15 }]
        );
        let cells = compare_figure_cells(Figure::Money(p(100)), Figure::Money(p(40))).unwrap();
        assert_eq!(
            cells,
            vec![Cell::Money { paisa: p(40) }, Cell::Money { paisa: p(60) }]
        );
    }

    #[test]
    fn the_share_of_a_count_is_the_same_formula_as_money() {
        // 50 of 60 is 83.33%; 10 of 60 is 16.67% (as the money shares of PRD 18.4 round half up).
        assert_eq!(
            share_cell(Figure::Count(50), Figure::Count(60)).unwrap(),
            Cell::Percent { hundredths: 8_333 }
        );
        assert_eq!(
            share_cell(Figure::Count(10), Figure::Count(60)).unwrap(),
            Cell::Percent { hundredths: 1_667 }
        );
        assert_eq!(
            share_cell(Figure::Count(0), Figure::Count(0)).unwrap(),
            Cell::Empty
        );
        assert!(share_cell(Figure::Count(61), Figure::Count(60)).is_err());
    }
}
