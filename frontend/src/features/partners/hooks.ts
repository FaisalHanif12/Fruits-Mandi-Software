// The partners commands as hooks (Code Standards F6). A saved partner or share refreshes the
// lists and the overview. A new partner also makes two accounts, so the chart and the account
// lists are read again (PRD 17.1).

import { keepPreviousData, useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useDebouncedValue } from '../../shared/hooks';
import {
  partnersCheckShares,
  partnersCreate,
  partnersGet,
  partnersList,
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
