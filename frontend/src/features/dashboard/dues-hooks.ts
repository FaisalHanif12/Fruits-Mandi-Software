// The buyer dues page as a hook (PRD FR-9.4, FR-19.11, Code Standards F6): one page at a time, with the
// count and the total of the whole filter, sorted by the backend (the default is by name). A save
// changes it, and every save asks all queries to read again.

import { useDebouncedValue, usePagedQuery } from '../../shared/hooks';
import { dashboardBuyerDues } from './api';

/** A short pause so a burst of key presses in the search box asks once (Code Standards F35). */
const FILTER_DELAY_MS = 150;

export function useBuyerDues(filter: { search: string }) {
  const settled = useDebouncedValue(filter, FILTER_DELAY_MS);
  return usePagedQuery({
    screen: 'dashboard.buyer_dues',
    queryKey: ['dashboard', 'buyer_dues', settled],
    fetchPage: (offset, limit, sort) =>
      dashboardBuyerDues({
        search: settled.search === '' ? null : settled.search,
        sort,
        offset,
        limit,
      }),
    rowsIn: (page) => page.rows.length,
    totalOf: (page) => page.total_count,
    keepPrevious: true,
  });
}
