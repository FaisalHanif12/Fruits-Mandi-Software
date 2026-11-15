// The اگراہی sheet (R5, plan 76 point 2) as a hook (PRD FR-19.11, Code Standards F6): one page of buyers at
// a time for one day, with the count and the totals of the whole filter, sorted by the backend (the
// default is by name). A save changes it, and every save asks all queries to read again.

import { useDebouncedValue, usePagedQuery } from '../../shared/hooks';
import { reportsCollectionSheet } from './api';

/** A short pause so a burst of key presses in the search box asks once (Code Standards F35). */
const FILTER_DELAY_MS = 150;

/** The filter of the sheet: its day (`YYYY-MM-DD`, empty is today) and the search box. */
export interface CollectionFilter {
  date: string;
  search: string;
}

export function useCollectionSheet(filter: CollectionFilter) {
  const settled = useDebouncedValue(filter, FILTER_DELAY_MS);
  return usePagedQuery({
    screen: 'reports.collection_sheet',
    queryKey: ['reports', 'collection_sheet', settled],
    fetchPage: (offset, limit, sort) =>
      reportsCollectionSheet({
        date: settled.date,
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
