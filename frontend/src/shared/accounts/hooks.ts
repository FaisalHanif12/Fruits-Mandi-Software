// The accounts and the chart as hooks (Code Standards F6). A save or a change of active state
// refreshes the lists, the chart and every picker, so a new account is found at once everywhere.

import { keepPreviousData, useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useEffect, useState } from 'react';
import {
  accountsGet,
  assistPartyCard,
  searchAccounts,
  searchAccountsRecent,
  searchAccountsTouch,
  searchGlobal,
  accountsList,
  accountsOpeningsPreview,
  accountsOpeningsSave,
  accountsNextCode,
  accountsSave,
  accountsSetActive,
  accountsSimilar,
  coaControlsList,
  coaTree,
} from './api';
import type { PickerSource } from '../search';
import type {
  AccountDraft,
  AccountFilter,
  AccountHit,
  AccountScope,
  AccountType,
  OpeningLineInput,
} from './types';

const ACCOUNTS_KEY = ['accounts'] as const;
const COA_KEY = ['coa'] as const;
/** A short pause so a burst of key presses asks once. */
const SIMILAR_DELAY_MS = 300;

export function useAccountList(filter: AccountFilter) {
  return useQuery({
    queryKey: [...ACCOUNTS_KEY, 'list', filter],
    queryFn: () => accountsList(filter),
    placeholderData: keepPreviousData,
  });
}

/** One account for its form. */
export function useAccount(id: number) {
  return useQuery({ queryKey: [...ACCOUNTS_KEY, 'one', id], queryFn: () => accountsGet(id) });
}

export function useCoaTree() {
  return useQuery({ queryKey: [...COA_KEY, 'tree'], queryFn: coaTree });
}

export function useCoaControls() {
  return useQuery({ queryKey: [...COA_KEY, 'controls'], queryFn: coaControlsList });
}

/** The code the next new account of a type gets. Asked again each time the form for a new one opens. */
export function useAccountNextCode(
  accountType: AccountType,
  controlId: number | null,
  enabled: boolean,
) {
  return useQuery({
    queryKey: [...ACCOUNTS_KEY, 'next-code', accountType, controlId],
    queryFn: () => accountsNextCode(accountType, controlId),
    enabled,
    staleTime: 0,
    gcTime: 0,
  });
}

/**
 * Accounts that look like the one being typed (PRD FR-7.63). Asked after a pause, and only when
 * a name or a phone has been typed.
 */
export function useSimilarAccounts(
  nameUr: string,
  nameEn: string,
  phone: string,
  excludeId: number | null,
) {
  const [settled, setSettled] = useState({ ur: '', en: '', phone: '' });
  useEffect(() => {
    const timer = window.setTimeout(() => {
      setSettled({ ur: nameUr.trim(), en: nameEn.trim(), phone: phone.trim() });
    }, SIMILAR_DELAY_MS);
    return () => {
      window.clearTimeout(timer);
    };
  }, [nameUr, nameEn, phone]);
  const typed = settled.ur !== '' || settled.en !== '' || settled.phone !== '';
  return useQuery({
    queryKey: [...ACCOUNTS_KEY, 'similar', settled.ur, settled.en, settled.phone, excludeId],
    queryFn: () =>
      accountsSimilar({
        name_ur: settled.ur,
        name_en: settled.en,
        phone: settled.phone,
        exclude_id: excludeId,
      }),
    enabled: typed,
    placeholderData: keepPreviousData,
  });
}

function useRefreshAfterChange() {
  const queryClient = useQueryClient();
  return async () => {
    await Promise.all([
      queryClient.invalidateQueries({ queryKey: ACCOUNTS_KEY }),
      queryClient.invalidateQueries({ queryKey: COA_KEY }),
      queryClient.invalidateQueries({ queryKey: ['search'] }),
    ]);
  };
}

export function useSaveAccount() {
  const refresh = useRefreshAfterChange();
  return useMutation({
    mutationFn: ({
      draft,
      confirmedWords,
    }: {
      draft: AccountDraft;
      confirmedWords: readonly string[];
    }) => accountsSave(draft, confirmedWords),
    onSuccess: refresh,
  });
}

export function useSetAccountActive() {
  const refresh = useRefreshAfterChange();
  return useMutation({
    mutationFn: ({ id, isActive }: { id: number; isActive: boolean }) =>
      accountsSetActive(id, isActive),
    onSuccess: refresh,
  });
}

/** Where an account picker gets its records: the accounts of a scope, recent ones first (FR-7.41). */
export function accountSource(
  scope: AccountScope,
  controlId: number | null = null,
): PickerSource<AccountHit> {
  return {
    key: ['search', 'accounts', scope, String(controlId)],
    search: (text, limit) => searchAccounts({ scope, controlId, text, limit }),
    recent: () => searchAccountsRecent(scope, controlId),
    touch: (id) => searchAccountsTouch(scope, controlId, id),
  };
}

/** The party card of an account (PRD FR-7.38). Nothing is asked while there is no account. */
export function usePartyCard(accountId: number | null) {
  return useQuery({
    queryKey: [...ACCOUNTS_KEY, 'card', accountId],
    queryFn: () => assistPartyCard(accountId ?? 0),
    enabled: accountId !== null,
    // A balance changes with every posting, so a card is never reused from the cache.
    staleTime: 0,
  });
}

/** The global search (Ctrl+K). Asked after a short pause, and only when something is typed. */
export function useGlobalSearch(text: string) {
  const [settled, setSettled] = useState('');
  useEffect(() => {
    const timer = window.setTimeout(() => {
      setSettled(text.trim());
    }, SIMILAR_DELAY_MS);
    return () => {
      window.clearTimeout(timer);
    };
  }, [text]);
  return useQuery({
    queryKey: ['search', 'global', settled],
    queryFn: () => searchGlobal(settled),
    enabled: settled !== '',
    placeholderData: keepPreviousData,
    staleTime: 0,
    gcTime: 0,
  });
}

/**
 * The openings grid checked, nothing saved (PRD FR-10.9): each line's problems, the totals of the
 * lines and what ابتدائی فرق would be. The same backend function as the save (Code Standards B5).
 * Never kept: a balance changes with every posting.
 */
export function useOpeningsPreview(lines: readonly OpeningLineInput[]) {
  return useQuery({
    queryKey: [...ACCOUNTS_KEY, 'openings-preview', lines],
    queryFn: () => accountsOpeningsPreview({ lines: [...lines] }),
    placeholderData: keepPreviousData,
    staleTime: 0,
    gcTime: 0,
  });
}

/** Saves the openings grid, all or nothing. Refreshes the lists, the chart and the pickers. */
export function useSaveOpenings() {
  const refresh = useRefreshAfterChange();
  return useMutation({
    mutationFn: (lines: readonly OpeningLineInput[]) => accountsOpeningsSave({ lines: [...lines] }),
    onSuccess: refresh,
  });
}
