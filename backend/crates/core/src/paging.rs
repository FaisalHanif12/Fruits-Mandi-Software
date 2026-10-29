//! What every list command shares for sorting and month headings (plan 38 B2, PRD FR-19.11,
//! FR-7.45, Code Standards B31, NFR-SPD-24).
//!
//! - [`SortInput`]: a column of the list and a direction. The backend sorts the WHOLE filter before
//!   it cuts the page, never only the page. A list opens in its natural order when no sort is sent.
//! - [`MonthSubtotal`]: for a register or ledger in date order, the count and the subtotals of each
//!   month over the WHOLE filter, so a month heading with its subtotal can be drawn above the first
//!   row of that month on any page. The screen never adds rows up (Code Standards F34).
//!
//! Example: `{"column": "gross", "direction": "desc"}` sorts a list by its gross, biggest first.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::{ErrorCode, ErrorValue, FieldError};

/// Which way a column is sorted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum SortDirection {
    /// Smallest first, oldest first, A to Z.
    Asc,
    /// Biggest first, newest first, Z to A.
    Desc,
}

impl SortDirection {
    /// Turns an ordering round for `Desc`. Used by the lists that are built in memory (reports,
    /// analytics, the one-time register): `rows.sort_by(|a, b| direction.apply(a.cmp(b)))`. The
    /// sort is stable, so rows that are equal keep the natural order of the list.
    pub const fn apply(self, ordering: std::cmp::Ordering) -> std::cmp::Ordering {
        match self {
            SortDirection::Asc => ordering,
            SortDirection::Desc => ordering.reverse(),
        }
    }

    /// `ASC` or `DESC`, for SQL.
    pub const fn sql(self) -> &'static str {
        match self {
            SortDirection::Asc => "ASC",
            SortDirection::Desc => "DESC",
        }
    }
}

/// A sort chosen on a column title (plan 38 R4). `column` is the name of a field of the rows of the
/// list, as the answer sends it (for example `entry_date` or `gross`). A column the list cannot
/// sort by is `ERR_OUT_OF_RANGE` on `sort.column`, with the allowed names in the value `allowed`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct SortInput {
    /// The column.
    pub column: String,
    /// The direction.
    pub direction: SortDirection,
}

impl SortInput {
    /// Checks that the column is one of `allowed` (the names of the fields of the row). For the
    /// lists that are built in memory and sorted there.
    ///
    /// # Errors
    /// `ERR_OUT_OF_RANGE` on `sort.column`, with the allowed names in the value `allowed`.
    pub fn check(&self, allowed: &[&str]) -> Result<(), FieldError> {
        if allowed.contains(&self.column.as_str()) {
            Ok(())
        } else {
            Err(FieldError::new(ErrorCode::ErrOutOfRange, "sort.column")
                .with("allowed", ErrorValue::Text(allowed.join(","))))
        }
    }

    /// Finds the column among `allowed` (`(row field, SQL expression)`) and gives the SQL expression
    /// with the direction. The expression comes from the list's own code, never from the user's
    /// text.
    ///
    /// # Errors
    /// `ERR_OUT_OF_RANGE` on `sort.column`.
    pub fn order_by(&self, allowed: &[(&str, &str)]) -> Result<String, FieldError> {
        match allowed.iter().find(|(name, _)| *name == self.column) {
            Some((_, expression)) => Ok(format!("{expression} {}", self.direction.sql())),
            None => Err(
                FieldError::new(ErrorCode::ErrOutOfRange, "sort.column").with(
                    "allowed",
                    ErrorValue::Text(
                        allowed
                            .iter()
                            .map(|(name, _)| *name)
                            .collect::<Vec<_>>()
                            .join(","),
                    ),
                ),
            ),
        }
    }
}

/// One subtotal of a month: the name of the field of the list's totals that it adds up and the sum.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct MonthFigure {
    /// The name of the field of the list's totals, for example `gross`.
    pub key: String,
    /// True for money (the value is in paisa, whole rupees); false for a count such as نگ.
    pub is_money: bool,
    /// The sum over the month.
    #[ts(type = "number")]
    pub value: i64,
}

/// A month of a register or ledger with its count and subtotals, over the WHOLE filter (plan 38
/// R4). Months come in the order of the list, and only when the list is in date order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct MonthSubtotal {
    /// The month, `YYYY-MM`.
    pub month: String,
    /// How many rows the filter has in this month.
    #[ts(type = "number")]
    pub count: i64,
    /// The subtotals of the list's columns for this month.
    pub figures: Vec<MonthFigure>,
}

impl MonthSubtotal {
    /// A money figure.
    pub fn money(key: &str, value: i64) -> MonthFigure {
        MonthFigure {
            key: key.to_owned(),
            is_money: true,
            value,
        }
    }

    /// A count figure.
    pub fn count_of(key: &str, value: i64) -> MonthFigure {
        MonthFigure {
            key: key.to_owned(),
            is_money: false,
            value,
        }
    }
}
