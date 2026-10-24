//! The Accounts group of the catalogue (PRD 19.2): the daily journal R11, the party ledger R13,
//! the party statement R14, the account index R16, the head balances R17, the trial balance R18
//! and the opening trial R19.
//!
//! Every number is read from posted journal lines (PRD FR-6.1, FR-19.5). A total is a `SUM()` of
//! stored `INTEGER` paisa columns, or a checked sum in Rust of such sums; a balance is the
//! difference of two such sums, worked out in Rust (money agent Section 9). Nothing is worked out
//! again from a rule.

mod balances;
mod cash;
mod journal;
mod ledger;
mod ledger_items;

pub(super) use balances::{run_r16, run_r17, run_r18, run_r19};
pub(super) use cash::run as run_r12;
pub(super) use journal::run_r11;
#[doc(hidden)]
pub use journal::{PAGE_SQL as R11_PAGE_SQL, TOTALS_SQL as R11_TOTALS_SQL};
#[doc(hidden)]
pub use ledger::{PAGE_SQL as R13_PAGE_SQL, TOTALS_SQL as R13_TOTALS_SQL};
pub(super) use ledger::{run_r13, run_r14};

/// Which rows a run returns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PageSpec {
    /// Where the page starts.
    pub(crate) offset: i64,
    /// How many rows. `None` returns every row (a print).
    pub(crate) limit: Option<i64>,
}

impl PageSpec {
    /// The SQL `LIMIT` value: `-1` is no limit in SQLite.
    pub(crate) fn sql_limit(self) -> i64 {
        self.limit.unwrap_or(-1)
    }

    /// The `limit` the page reports: 0 for every row.
    pub(crate) fn reported_limit(self) -> i64 {
        self.limit.unwrap_or(0)
    }

    /// The part of an already built list that belongs to this page.
    pub(crate) fn slice<T: Clone>(self, all: &[T]) -> Vec<T> {
        let start = usize::try_from(self.offset)
            .unwrap_or(usize::MAX)
            .min(all.len());
        let end = match self.limit {
            Some(limit) => start
                .saturating_add(usize::try_from(limit).unwrap_or(usize::MAX))
                .min(all.len()),
            None => all.len(),
        };
        all.get(start..end).map(<[T]>::to_vec).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::PageSpec;

    #[test]
    fn pages_cut_a_list_without_losing_or_repeating_a_row() {
        let all: Vec<i64> = (0..23).collect();
        let mut joined = Vec::new();
        let mut offset = 0;
        loop {
            let page = PageSpec {
                offset,
                limit: Some(10),
            }
            .slice(&all);
            if page.is_empty() {
                break;
            }
            joined.extend(page);
            offset += 10;
        }
        assert_eq!(joined, all);
    }

    #[test]
    fn a_page_past_the_end_is_empty_and_no_limit_is_every_row() {
        let all = [1, 2, 3];
        let past = PageSpec {
            offset: 99,
            limit: Some(5),
        };
        assert!(past.slice(&all).is_empty());
        let every = PageSpec {
            offset: 0,
            limit: None,
        };
        assert_eq!(every.slice(&all), vec![1, 2, 3]);
        assert_eq!(every.sql_limit(), -1);
        assert_eq!(every.reported_limit(), 0);
        let huge = PageSpec {
            offset: i64::MAX,
            limit: Some(i64::MAX),
        };
        assert!(huge.slice(&all).is_empty());
    }
}
