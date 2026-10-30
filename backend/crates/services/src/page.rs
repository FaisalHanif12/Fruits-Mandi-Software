//! Paging for the lists of plan 17 (PRD FR-19.11, Code Standards B31): a page is an offset and a
//! limit, the limit being 100 when the screen sends 0. The totals are always worked out over the
//! whole filter by the caller, never from the page.

/// The page size when the screen sends 0.
pub(crate) const DEFAULT_PAGE: u32 = 100;

/// The rows of one page of a list that is already whole in memory. An offset past the end gives
/// an empty page.
pub(crate) fn slice<T>(rows: Vec<T>, offset: u32, limit: u32) -> Vec<T> {
    let limit = if limit == 0 { DEFAULT_PAGE } else { limit };
    rows.into_iter()
        .skip(usize::try_from(offset).unwrap_or(usize::MAX))
        .take(usize::try_from(limit).unwrap_or(usize::MAX))
        .collect()
}

/// One page over two lists read one after the other (the lots of the stock list, then the trip
/// goods): the page takes from the first list, and what is left of the page from the second. An
/// offset past the first list skips into the second.
pub(crate) fn slice_two<A, B>(
    first: Vec<A>,
    second: Vec<B>,
    offset: u32,
    limit: u32,
) -> (Vec<A>, Vec<B>) {
    let size = if limit == 0 { DEFAULT_PAGE } else { limit };
    let first_len = u32::try_from(first.len()).unwrap_or(u32::MAX);
    let first_page = slice(first, offset, size);
    let taken = u32::try_from(first_page.len()).unwrap_or(u32::MAX);
    let room = size.saturating_sub(taken);
    if room == 0 {
        return (first_page, Vec::new());
    }
    let second_page = slice(second, offset.saturating_sub(first_len), room);
    (first_page, second_page)
}

/// One page over three lists read one after the other (the lots, the trip goods, the garden
/// arrivals of the stock list): the page takes from the first, then what is left of the page from the
/// second, then from the third. An offset past a list skips into the next.
pub(crate) fn slice_three<A, B, C>(
    first: Vec<A>,
    second: Vec<B>,
    third: Vec<C>,
    offset: u32,
    limit: u32,
) -> (Vec<A>, Vec<B>, Vec<C>) {
    let size = if limit == 0 { DEFAULT_PAGE } else { limit };
    let first_len = u32::try_from(first.len()).unwrap_or(u32::MAX);
    let second_len = u32::try_from(second.len()).unwrap_or(u32::MAX);
    let (first_page, second_page) = slice_two(first, second, offset, size);
    let taken = u32::try_from(first_page.len() + second_page.len()).unwrap_or(u32::MAX);
    let room = size.saturating_sub(taken);
    if room == 0 {
        return (first_page, second_page, Vec::new());
    }
    let skip = offset.saturating_sub(first_len.saturating_add(second_len));
    (first_page, second_page, slice(third, skip, room))
}

/// Urdu alphabetical order of two texts (ا ب پ ت ٹ, NFR-URD-19), for the lists that are sorted in
/// memory. The ICU4X collator is made once. If its data were missing, the texts compare as code
/// points.
pub(crate) fn urdu_compare(a: &str, b: &str) -> std::cmp::Ordering {
    use std::sync::OnceLock;
    static COLLATOR: OnceLock<Option<mandi_domain::core::text::UrduCollator>> = OnceLock::new();
    match COLLATOR
        .get_or_init(|| mandi_domain::core::text::UrduCollator::new().ok())
        .as_ref()
    {
        Some(collator) => collator.compare(a, b),
        None => a.cmp(b),
    }
}

/// The limit a database query asks for: 100 when the screen sends 0.
pub(crate) fn limit_of(limit: u32) -> i64 {
    i64::from(if limit == 0 { DEFAULT_PAGE } else { limit })
}

/// Turns the month rows of `mandi_db::listing::month_sums` into month subtotals (plan 38 R4).
/// `figures` names each sum in the same order as the expressions given to the query: the field of
/// the list's totals and whether it is money. The months come oldest first from the database;
/// `newest_first` turns them round for a list that is shown newest first.
pub(crate) fn month_subtotals(
    rows: Vec<mandi_db::listing::MonthSums>,
    figures: &[(&str, bool)],
    newest_first: bool,
) -> Vec<mandi_domain::core::paging::MonthSubtotal> {
    use mandi_domain::core::paging::{MonthFigure, MonthSubtotal};
    let mut months: Vec<MonthSubtotal> = rows
        .into_iter()
        .map(|row| MonthSubtotal {
            month: row.month,
            count: row.count,
            figures: figures
                .iter()
                .zip(row.sums)
                .map(|((key, is_money), value)| MonthFigure {
                    key: (*key).to_owned(),
                    is_money: *is_money,
                    value,
                })
                .collect(),
        })
        .collect();
    if newest_first {
        months.reverse();
    }
    months
}

/// Adds up the month subtotals of a list that is built in memory (plan 38 R4): call [`add`] for
/// each row, in any order, then [`finish`]. A month shows the count of the rows put in it and the
/// sum of each figure. All sums are checked.
///
/// [`add`]: MonthAcc::add
/// [`finish`]: MonthAcc::finish
pub(crate) struct MonthAcc {
    figures: usize,
    months: std::collections::BTreeMap<String, (i64, Vec<i64>)>,
}

impl MonthAcc {
    /// A new accumulator for `figures` sums per month.
    pub(crate) fn new(figures: usize) -> Self {
        Self {
            figures,
            months: std::collections::BTreeMap::new(),
        }
    }

    /// Counts one row of the month of `date` (`YYYY-MM-DD`) and adds `sums` to it. A missing sum
    /// counts as 0.
    pub(crate) fn add(
        &mut self,
        date: &str,
        sums: &[i64],
    ) -> Result<(), crate::error::ServiceError> {
        debug_assert_eq!(sums.len(), self.figures, "one sum for each figure");
        let month = date.get(..7).unwrap_or(date).to_owned();
        let figures = self.figures;
        let (count, totals) = self
            .months
            .entry(month)
            .or_insert_with(|| (0, vec![0; figures]));
        *count = count.checked_add(1).ok_or_else(|| {
            crate::error::ServiceError::reject(mandi_domain::core::error::ErrorCode::ErrOverflow)
        })?;
        for (total, add) in totals.iter_mut().zip(sums) {
            *total = total.checked_add(*add).ok_or_else(|| {
                crate::error::ServiceError::reject(
                    mandi_domain::core::error::ErrorCode::ErrOverflow,
                )
            })?;
        }
        Ok(())
    }

    /// The months, oldest first (newest first when asked), with the named figures: the field of
    /// the list's totals and whether it is money, in the order of the sums given to `add`.
    pub(crate) fn finish(
        self,
        figures: &[(&str, bool)],
        newest_first: bool,
    ) -> Vec<mandi_domain::core::paging::MonthSubtotal> {
        let rows = self
            .months
            .into_iter()
            .map(|(month, (count, sums))| mandi_db::listing::MonthSums { month, count, sums })
            .collect();
        month_subtotals(rows, figures, newest_first)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_limit_of_zero_is_a_page_of_100() {
        let rows: Vec<u32> = (0..250).collect();
        assert_eq!(slice(rows.clone(), 0, 0).len(), 100);
        assert_eq!(slice(rows.clone(), 200, 0).len(), 50);
        assert!(slice(rows.clone(), 300, 0).is_empty());
        assert_eq!(slice(rows, 10, 5), vec![10, 11, 12, 13, 14]);
        assert_eq!(limit_of(0), 100);
        assert_eq!(limit_of(7), 7);
    }

    #[test]
    fn two_lists_are_paged_as_one() {
        // Every page size and offset: the pages together are the first list and then the second,
        // each row once, and a page never holds more than its size.
        for (first_len, second_len) in [(0_u32, 0_u32), (0, 5), (5, 0), (3, 4), (7, 1), (1, 7)] {
            let first: Vec<u32> = (0..first_len).collect();
            let second: Vec<u32> = (100..100 + second_len).collect();
            for size in 1..=9_u32 {
                let (mut got_first, mut got_second) = (Vec::new(), Vec::new());
                let mut offset = 0;
                while offset < first_len + second_len {
                    let (a, b) = slice_two(first.clone(), second.clone(), offset, size);
                    let held = u32::try_from(a.len() + b.len()).unwrap();
                    assert!(held <= size && held > 0, "{first_len} {second_len} {size}");
                    got_first.extend(a);
                    got_second.extend(b);
                    offset += size;
                }
                assert_eq!((got_first, got_second), (first.clone(), second.clone()));
            }
            // Past the end: nothing. A limit of 0 is a page of 100.
            let (a, b) = slice_two(first.clone(), second.clone(), first_len + second_len, 3);
            assert!(a.is_empty() && b.is_empty());
            let (a, b) = slice_two(first.clone(), second.clone(), 0, 0);
            assert_eq!(
                a.len() + b.len(),
                usize::try_from(first_len + second_len).unwrap()
            );
        }
    }
}
