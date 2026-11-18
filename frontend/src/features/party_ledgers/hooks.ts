// The two reads of the party ledgers screen as hooks (Code Standards F6): one page at a time, with
// the count and the totals of the whole filter (PRD FR-19.11, NFR-SPD-24). A save elsewhere changes
// a balance, so a list that is opened again reads again (staleTime 0, PRD 6.2).

import { keepPreviousData, useQuery } from '@tanstack/react-query';
import { useMemo } from 'react';
import type { LedgerSide } from '../../bindings/LedgerSide';
import { isoToDisplayDate } from '../../shared/format';
import { useDebouncedValue, usePagedQuery } from '../../shared/hooks';
import { partyLedgersList, partyLedgersStatementRows } from './api';

/** The most rows one read may ask for (Code Standards B31); 30 days never come near it. */
const WINDOW_LIMIT = 100;

/**
 * A window read this moment ago is not read again: "go to a date" brings the window's own page and
 * totals with its answer (PRD FR-12.47), the screen puts it in the cache and shows it with no second call.
 */
const WINDOW_FRESH_MS = 2_000;

/** The cache key of the 30 days of one party's statement. */
export function partyWindowKey(
  accountId: number,
  window: { from: string; to: string } | null,
  history: boolean,
) {
  return [
    'party_ledgers',
    'window',
    accountId,
    window?.from ?? null,
    window?.to ?? null,
    history,
  ] as const;
}

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
 * The 30 days of one party's statement, one row for each entry, oldest first, as one read: the rows of the
 * window with the balance before it, what it charged and what was paid, and the balance at its end, all the
 * backend's (PRD FR-12.47, FR-19.16, plan 78 B1 to B3). It is one page, because 30 days hold at most 100 entries
 * of one party. Days are `YYYY-MM-DD`; the backend is sent them as typed dates. The window last shown stays on
 * screen while the next one is on its way. `history` shows the entries an edit or a cancel replaced too.
 */
export function usePartyWindow(
  accountId: number,
  window: { from: string; to: string } | null,
  history: boolean,
) {
  return useQuery({
    queryKey: partyWindowKey(accountId, window, history),
    queryFn: () =>
      partyLedgersStatementRows({
        account_id: accountId,
        ...(window === null
          ? {}
          : { from: isoToDisplayDate(window.from), to: isoToDisplayDate(window.to) }),
        include_history: history,
        newest_first: false,
        offset: 0,
        limit: WINDOW_LIMIT,
      }),
    enabled: window !== null,
    placeholderData: keepPreviousData,
    // A save elsewhere changes a balance (PRD 6.2): a window opened again reads again.
    staleTime: WINDOW_FRESH_MS,
  });
}

/**
 * The first and the last date of level 2 as typed (`DD-MM-YYYY`), once the Owner has paused: a burst of
 * key presses in a date box asks the backend once (Code Standards F35).
 */
export function useSettledDates(from: string, to: string) {
  const typed = useMemo(() => ({ from, to }), [from, to]);
  return useDebouncedValue(typed, FILTER_DELAY_MS);
}

/**
 * Level 2 as a range: the statement of one party between two dates, as they are typed (`DD-MM-YYYY`; the
 * backend reads the shortcuts and refuses a bad date under its own box), one row for each entry, page by page
 * with its own balance brought forward and closing balance. The dates are already settled (`useSettledDates`).
 * With `enabled` off nothing is asked (the screen shows its 30-day window instead).
 */
export function usePartyStatement(
  accountId: number,
  from: string,
  to: string,
  history: boolean,
  enabled = true,
) {
  const dates = useMemo(
    () => ({ accountId, from: textOrNull(from), to: textOrNull(to) }),
    [accountId, from, to],
  );
  return usePagedQuery({
    screen: 'party_ledgers.statement',
    queryKey: ['party_ledgers', 'statement', dates.accountId, dates.from, dates.to, history],
    fetchPage: (offset, limit) =>
      partyLedgersStatementRows({
        account_id: dates.accountId,
        ...(dates.from === null ? {} : { from: dates.from }),
        ...(dates.to === null ? {} : { to: dates.to }),
        include_history: history,
        newest_first: false,
        offset,
        limit,
      }),
    rowsIn: (page) => page.rows.length,
    totalOf: (page) => page.count,
    enabled,
    keepPrevious: true,
  });
}
