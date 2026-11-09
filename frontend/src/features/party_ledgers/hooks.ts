// The two reads of the party ledgers screen as hooks (Code Standards F6): one page at a time, with
// the count and the totals of the whole filter (PRD FR-19.11, NFR-SPD-24). A save elsewhere changes
// a balance, so a list that is opened again reads again (staleTime 0, PRD 6.2).

import { useMemo } from 'react';
import type { LedgerSide } from '../../bindings/LedgerSide';
import { useDebouncedValue, usePagedQuery } from '../../shared/hooks';
import { partyLedgersList, partyLedgersStatement } from './api';

/** A short pause so a burst of key presses in a filter box asks once (Code Standards F35). */
const FILTER_DELAY_MS = 250;

/** Text or nothing: an empty filter is sent as no filter. */
function textOrNull(text: string): string | null {
  const trimmed = text.trim();
  return trimmed === '' ? null : trimmed;
}

/** Level 1: the parties of one side, narrowed by what is typed in the search box. */
export function usePartyLedgersList(side: LedgerSide, search: string) {
  const asked = useMemo(() => ({ side, text: textOrNull(search) }), [side, search]);
  const settled = useDebouncedValue(asked, FILTER_DELAY_MS);
  return usePagedQuery({
    screen: 'party_ledgers.list',
    queryKey: ['party_ledgers', 'list', settled.side, settled.text],
    fetchPage: (offset, limit) =>
      partyLedgersList({ side: settled.side, text: settled.text, offset, limit }),
    rowsIn: (page) => page.rows.length,
    totalOf: (page) => page.count,
    keepPrevious: true,
  });
}

/**
 * Level 2: the statement of one party between two dates, as they are typed (`DD-MM-YYYY`; the
 * backend reads the shortcuts and refuses a bad date under its own box).
 */
export function usePartyStatement(accountId: number, from: string, to: string) {
  const asked = useMemo(
    () => ({ accountId, from: textOrNull(from), to: textOrNull(to) }),
    [accountId, from, to],
  );
  const settled = useDebouncedValue(asked, FILTER_DELAY_MS);
  return usePagedQuery({
    screen: 'party_ledgers.statement',
    queryKey: ['party_ledgers', 'statement', settled.accountId, settled.from, settled.to],
    fetchPage: (offset, limit) =>
      partyLedgersStatement({
        account_id: settled.accountId,
        from: settled.from,
        to: settled.to,
        offset,
        limit,
      }),
    rowsIn: (page) => page.rows.length,
    totalOf: (page) => page.count,
    keepPrevious: true,
  });
}
