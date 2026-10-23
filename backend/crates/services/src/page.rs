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

/// The limit a database query asks for: 100 when the screen sends 0.
pub(crate) fn limit_of(limit: u32) -> i64 {
    i64::from(if limit == 0 { DEFAULT_PAGE } else { limit })
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
}
