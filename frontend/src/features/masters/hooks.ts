// The masters as hooks (Code Standards F6). A save or a change of active state refreshes the lists
// and every picker, so a new record is found at once everywhere (PRD FR-7.10).

import { keepPreviousData, useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useEffect, useState } from 'react';
import { searchMasters } from '../../shared/search';
import { mastersList, mastersNextCode, mastersSave, mastersSetActive } from './api';
import type { MasterDraft, MasterKind } from './types';

const MASTERS_KEY = ['masters'] as const;

/** How many matches the filter box asks the search for. */
const FILTER_LIMIT = 500;
/** A short pause so a burst of key presses asks once. */
const FILTER_DELAY_MS = 150;

/** Every master of a kind. The list always holds the inactive ones too, the screen hides them. */
export function useMasterList(kind: MasterKind) {
  return useQuery({
    queryKey: [...MASTERS_KEY, kind, 'list'],
    queryFn: () => mastersList(kind, true),
  });
}

/** `value`, but only after it has stayed the same for `delayMs`. */
function useSettled(value: string, delayMs: number): string {
  const [settled, setSettled] = useState(value);
  useEffect(() => {
    const timer = window.setTimeout(() => {
      setSettled(value);
    }, delayMs);
    return () => {
      window.clearTimeout(timer);
    };
  }, [value, delayMs]);
  return settled;
}

/**
 * The ids a typed filter keeps, best match first, or null when nothing is typed (keep all). The
 * search is the backend's: both scripts, look-alike letters and small mistakes (PRD FR-7.35,
 * FR-7.40), not a copy of its rules here.
 */
export function useMasterFilter(kind: MasterKind, text: string) {
  const trimmed = text.trim();
  const settled = useSettled(trimmed, FILTER_DELAY_MS);
  const query = useQuery({
    queryKey: [...MASTERS_KEY, kind, 'filter', settled],
    queryFn: () => searchMasters({ scope: kind, text: settled, limit: FILTER_LIMIT }),
    enabled: settled !== '',
    placeholderData: keepPreviousData,
  });
  return {
    /** Null: no filter. */
    ids: trimmed === '' ? null : (query.data?.map((hit) => hit.id) ?? []),
    isPending: trimmed !== '' && (settled !== trimmed || query.isPending),
    isError: trimmed !== '' && query.isError,
    error: query.error,
    refetch: query.refetch,
  };
}

/** The code the next new master of a kind gets, asked each time the form for a new one opens. */
export function useNextCode(kind: MasterKind, enabled: boolean) {
  return useQuery({
    queryKey: [...MASTERS_KEY, kind, 'next-code'],
    queryFn: () => mastersNextCode(kind),
    enabled,
    staleTime: 0,
    gcTime: 0,
  });
}

function useRefreshAfterChange(kind: MasterKind) {
  const queryClient = useQueryClient();
  return async () => {
    await Promise.all([
      queryClient.invalidateQueries({ queryKey: [...MASTERS_KEY, kind] }),
      // Pickers hold searches of this kind (and an item's default packing is a search too).
      queryClient.invalidateQueries({ queryKey: ['search'] }),
    ]);
  };
}

export function useSaveMaster(kind: MasterKind) {
  const refresh = useRefreshAfterChange(kind);
  return useMutation({
    mutationFn: ({
      draft,
      confirmedWords,
    }: {
      draft: MasterDraft;
      confirmedWords: readonly string[];
    }) => mastersSave(kind, draft, confirmedWords),
    onSuccess: refresh,
  });
}

export function useSetMasterActive(kind: MasterKind) {
  const refresh = useRefreshAfterChange(kind);
  return useMutation({
    mutationFn: ({ id, isActive }: { id: number; isActive: boolean }) =>
      mastersSetActive(kind, id, isActive),
    onSuccess: refresh,
  });
}
