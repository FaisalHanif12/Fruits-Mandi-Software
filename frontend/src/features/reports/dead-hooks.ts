// R38, the dead amount, as a hook (Code Standards F6): one page at a time, with the count and the
// totals of the whole filter (PRD FR-19.11). A move, a recovery or an undo changes it, and every
// dialog that does that asks all queries to read again.

import { useDebouncedValue, usePagedQuery } from '../../shared/hooks';
import type { DeadAmountInput } from '../../bindings/DeadAmountInput';
import { reportsDeadAmount } from './api';

/** A short pause so a burst of key presses in the search box asks once (Code Standards F35). */
const FILTER_DELAY_MS = 150;

export function useDeadAmount(filter: Pick<DeadAmountInput, 'filter' | 'search'>) {
  const settled = useDebouncedValue(filter, FILTER_DELAY_MS);
  return usePagedQuery({
    screen: 'reports.dead_amount',
    queryKey: ['reports', 'dead_amount', settled],
    fetchPage: (offset, limit) => reportsDeadAmount({ ...settled, offset, limit }),
    rowsIn: (page) => page.rows.length,
    totalOf: (page) => page.row_count,
    keepPrevious: true,
  });
}
