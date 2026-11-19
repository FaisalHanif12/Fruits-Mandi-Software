// ادھاری تفصیل (R5, PRD R5 v1.43, plan 80 Q-R5-2) as a hook (PRD FR-19.11, Code Standards F6): one page of the
// day's buyers at a time, with the totals of the whole day. The backend sorts by name; a save changes it, and
// every save asks all queries to read again.

import { usePagedQuery } from '../../shared/hooks';
import { reportsAdhariSheet } from './api';

/** The filter of the sheet: its day (`YYYY-MM-DD`, empty is today). */
export interface AdhariFilter {
  date: string;
}

export function useAdhariSheet(filter: AdhariFilter) {
  return usePagedQuery({
    screen: 'reports.adhari_sheet',
    queryKey: ['reports', 'adhari_sheet', filter],
    // The sheet has no search and no sort of its own: its rows are the day's buyers by name.
    fetchPage: (offset, limit) => reportsAdhariSheet({ date: filter.date, offset, limit }),
    rowsIn: (page) => page.rows.length,
    totalOf: (page) => page.total_count,
    keepPrevious: true,
  });
}
