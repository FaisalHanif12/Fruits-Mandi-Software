// The hooks of the screens of the partners area that use the entry pieces of the cashbook module
// (plan 17): the سرمایہ screen and the partner statement. Kept apart from hooks.ts, which the
// module's index exports, so the cashbook module and this one never import each other at load.

import { keepPreviousData, useMutation, useQuery } from '@tanstack/react-query';
import { usePagedQuery } from '../../shared/hooks';
import { useReadAgain } from '../cashbook';
import {
  capitalHistory,
  partnersStatementDay,
  partnersStatementMonth,
  saveCapitalIn,
  saveCapitalOut,
} from './api';
import { PARTNERS_KEY } from './hooks';
import type { CapitalEntry } from './types';

/**
 * One partner's capital history, page by page (PRD 17.4). `range` is the month window of the list (PRD FR-19.11).
 * Never kept: a save changes it.
 */
export function useCapitalHistory(
  partnerId: number | null,
  groupByMonth = false,
  range: { from: string | null; to: string | null } = { from: null, to: null },
  ready = true,
) {
  return usePagedQuery({
    screen: 'partners.capital',
    queryKey: [...PARTNERS_KEY, 'capital', partnerId, groupByMonth, range],
    enabled: partnerId !== null && ready,
    fetchPage: (offset, limit, sort) =>
      capitalHistory({
        partner_id: partnerId ?? 0,
        from: range.from,
        to: range.to,
        offset,
        limit,
        sort,
        group_by_month: groupByMonth && sort === null,
      }),
    rowsIn: (page) => page.rows.length,
    totalOf: (page) => page.total_count,
  });
}

/** Saves a capital move by its type: its own command and posting (PRD FR-7.67). */
export function useSaveCapital() {
  const readAgain = useReadAgain();
  return useMutation({
    mutationFn: (entry: CapitalEntry) =>
      entry.type === 'capital_in' ? saveCapitalIn(entry) : saveCapitalOut(entry),
    onSuccess: readAgain,
  });
}

/**
 * One month of a partner's statement (PRD FR-17.12). `month` is `YYYY-MM`, or null for the current month. The
 * last answer stays on screen while the next month is read. Never kept: every entry changes it.
 */
export function usePartnerStatementMonth(partnerId: number | null, month: string | null) {
  return useQuery({
    queryKey: [...PARTNERS_KEY, 'statement-month', partnerId, month],
    queryFn: () => partnersStatementMonth({ partner_id: partnerId ?? 0, month }),
    enabled: partnerId !== null,
    placeholderData: keepPreviousData,
    staleTime: 0,
  });
}

/** Every movement of one day of the statement, read when its row is opened (`day` is `YYYY-MM-DD`). */
export function usePartnerStatementDay(partnerId: number, day: string, enabled: boolean) {
  return useQuery({
    queryKey: [...PARTNERS_KEY, 'statement-day', partnerId, day],
    queryFn: () => partnersStatementDay({ partner_id: partnerId, date: day }),
    enabled,
    staleTime: 0,
  });
}
