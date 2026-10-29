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
