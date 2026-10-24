// The hooks of the screens of the partners area that use the entry pieces of the cashbook module
// (plan 17): the سرمایہ screen and the partner statement. Kept apart from hooks.ts, which the
// module's index exports, so the cashbook module and this one never import each other at load.

import { useMutation } from '@tanstack/react-query';
import { usePagedQuery } from '../../shared/hooks';
import { useReadAgain } from '../cashbook';
import { capitalHistory, partnersStatement, saveCapitalIn, saveCapitalOut } from './api';
import { PARTNERS_KEY } from './hooks';
import type { CapitalEntry } from './types';

/** One partner's capital history, page by page (PRD 17.4). Never kept: a save changes it. */
export function useCapitalHistory(partnerId: number | null) {
  return usePagedQuery({
    queryKey: [...PARTNERS_KEY, 'capital', partnerId],
    enabled: partnerId !== null,
    fetchPage: (offset, limit) => capitalHistory({ partner_id: partnerId ?? 0, offset, limit }),
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

/** The range of a statement: `DD-MM-YYYY` text, or null for no limit. */
export interface StatementRange {
  from: string | null;
  to: string | null;
}

/** One partner's statement, page by page, with the totals over the whole range (PRD FR-17.9). */
export function usePartnerStatement(partnerId: number | null, range: StatementRange) {
  return usePagedQuery({
    queryKey: [...PARTNERS_KEY, 'statement', partnerId, range],
    enabled: partnerId !== null,
    fetchPage: (offset, limit) =>
      partnersStatement({
        partner_id: partnerId ?? 0,
        from: range.from,
        to: range.to,
        offset,
        limit,
      }),
    rowsIn: (page) => page.lines.length,
    totalOf: (page) => page.total_count,
    keepPrevious: true,
  });
}
