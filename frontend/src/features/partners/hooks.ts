// The partners commands as hooks (Code Standards F6). A saved partner or share refreshes the
// lists and the overview. A new partner also makes two accounts, so the chart and the account
// lists are read again (PRD 17.1).

import { keepPreviousData, useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useDebouncedValue, usePagedQuery } from '../../shared/hooks';
import {
  partnersCheckShares,
  partnersCreate,
  partnersGet,
  partnersList,
  partnersOpenVentures,
  partnersOverview,
  partnersSetShares,
  partnersUpdate,
} from './api';
import type {
  PartnerCreateInput,
  PartnerUpdateInput,
  PartnersSetSharesInput,
  ShareInput,
} from './types';

export const PARTNERS_KEY = ['partners'] as const;
/** The check waits this long after the last key press, and the one before it is dropped (F35). */
const CHECK_DELAY_MS = 50;

export function usePartnerList() {
  return useQuery({ queryKey: [...PARTNERS_KEY, 'list'], queryFn: partnersList });
}

export function usePartnerOverview() {
  return useQuery({ queryKey: [...PARTNERS_KEY, 'overview'], queryFn: partnersOverview });
}

/**
 * The open gardens and trips one partner is in, read when his compact line is opened (plan 76 point
 * 11), a page at a time for the days they were made in, with the backend's totals over the whole
 * filter (plan 99, PRD FR-17.3, FR-19.11). Asked again whenever the list is opened: what he paid and
 * got back changes with every entry, and every save of a partner posting reads the partners again.
 */
export function usePartnerOpenVentures(
  partnerId: number,
  { from, to }: { from: string | null; to: string | null },
  enabled: boolean,
) {
  return usePagedQuery({
    screen: 'partners.ventures',
    queryKey: [...PARTNERS_KEY, 'open-ventures', partnerId, from, to],
    fetchPage: (offset, limit) =>
      partnersOpenVentures({ partner_id: partnerId, from, to, offset, limit }),
    rowsIn: (page) => page.rows.length,
    totalOf: (page) => page.total_count,
    enabled,
    keepPrevious: true,
  });
}

/**
 * The month of the newest open garden or trip a partner is in, `YYYY-MM`, or null when he is in none:
 * the list opens on it (plan 99, PRD FR-17.3). The backend lists newest first, so its first row is it.
 */
export function usePartnerLatestVentureMonth(partnerId: number) {
  return useQuery({
    queryKey: [...PARTNERS_KEY, 'open-ventures-latest', partnerId],
    queryFn: () =>
      partnersOpenVentures({ partner_id: partnerId, from: null, to: null, offset: 0, limit: 1 }),
    select: (page) => page.rows[0]?.entry_date.slice(0, 7) ?? null,
    staleTime: 0,
  });
}

/** One partner with his share history, for the details dialog. Never kept: shares change. */
export function usePartner(id: number | null) {
  return useQuery({
    queryKey: [...PARTNERS_KEY, 'one', id],
    queryFn: () => partnersGet(id ?? 0),
    enabled: id !== null,
    staleTime: 0,
  });
}

/** What a save of a partner changes elsewhere: the lists, the overview, the chart and accounts. */
function useRefreshAfterPartnerSave() {
  const queryClient = useQueryClient();
  return () =>
    Promise.all([
      queryClient.invalidateQueries({ queryKey: PARTNERS_KEY }),
      queryClient.invalidateQueries({ queryKey: ['accounts'] }),
      queryClient.invalidateQueries({ queryKey: ['coa'] }),
    ]);
}

export function useCreatePartner() {
  const refresh = useRefreshAfterPartnerSave();
  return useMutation({
    mutationFn: (input: PartnerCreateInput) => partnersCreate(input),
    onSuccess: refresh,
  });
}

export function useUpdatePartner() {
  const refresh = useRefreshAfterPartnerSave();
  return useMutation({
    mutationFn: (input: PartnerUpdateInput) => partnersUpdate(input),
    onSuccess: refresh,
  });
}

export function useSetShares() {
  const refresh = useRefreshAfterPartnerSave();
  return useMutation({
    mutationFn: (input: PartnersSetSharesInput) => partnersSetShares(input),
    onSuccess: refresh,
  });
}

/**
 * The backend's reading of the shares on screen: the total, and what is wrong with which row.
 * Asked 50 ms after the last key press. `shares` must keep its identity while nothing changed
 * (build it with useMemo), or the wait would start again on every redraw. `current` is false
 * while the answer on screen belongs to an earlier version of the shares, so the screen never
 * calls an old total correct.
 */
export function useSharesCheck(shares: readonly ShareInput[]) {
  const settled = useDebouncedValue(shares, CHECK_DELAY_MS);
  const query = useQuery({
    queryKey: [...PARTNERS_KEY, 'check', settled],
    queryFn: () => partnersCheckShares(settled),
    enabled: settled.length > 0,
    placeholderData: keepPreviousData,
    staleTime: 0,
    gcTime: 0,
  });
  return { query, current: settled === shares && !query.isFetching };
}
