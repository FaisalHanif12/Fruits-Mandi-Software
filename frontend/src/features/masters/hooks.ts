// The masters as hooks (Code Standards F6). A save or a change of active state refreshes the lists
// and every picker, so a new record is found at once everywhere (PRD FR-7.10).

import { keepPreviousData, useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useEffect, useState } from 'react';
import { useAllPagesQuery } from '../../shared/hooks';
import { searchMasters } from '../../shared/search';
import { mastersGetOne, mastersList, mastersNextCode, mastersSave, mastersSetActive } from './api';
import type { MasterDraft, MasterKind } from './types';

const MASTERS_KEY = ['masters'] as const;

/** How many matches the filter box asks the search for. */
const FILTER_LIMIT = 500;
/** A short pause so a burst of key presses asks once. */
const FILTER_DELAY_MS = 150;

/**
 * Every master of a kind, page after page (`useLoadRest`): a master list is short, so the screen
 * filters it and turns its pages in memory (`useClientPager`). Inactive ones are in it only when
 * asked for (PRD FR-19.11).
 */
export function useMasterList(kind: MasterKind, includeInactive: boolean) {
  return useAllPagesQuery({
    queryKey: [...MASTERS_KEY, kind, 'list', includeInactive],
    fetchPage: (offset, limit) => mastersList(kind, includeInactive, offset, limit),
    rowsIn: (page) => page.rows.length,
    totalOf: (page) => page.total_count,
    keepPrevious: true,
  });
}

/**
 * One master of a kind by its id, for its form: one record from the backend, never the pages of the
 * list before it. `record` is undefined while it loads and when there is no such master.
 */
export function useMasterRecord(kind: MasterKind, id: string | undefined) {
  const wanted = id === undefined ? null : Number(id);
  // An address that is not a number names no master: nothing is asked and the form says so.
  const valid = wanted !== null && Number.isInteger(wanted);
  const query = useQuery({
    queryKey: [...MASTERS_KEY, kind, 'one', wanted],
    queryFn: () => mastersGetOne(kind, wanted ?? 0),
    enabled: valid,
    staleTime: 0,
  });
  return {
    record: query.data ?? undefined,
    isPending: valid && query.isPending,
    isError: query.isError,
    error: query.error,
    refetch: query.refetch,
  };
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

/** How many look-alikes the short form of "add it here" shows. */
const SIMILAR_LIMIT = 5;

/**
 * Records of a kind that look like the name being typed (PRD FR-7.63, FR-10.7): the same search the
 * pickers use, so both scripts, look-alike letters and small mistakes count. Asked after a pause,
 * and only when something has been typed.
 */
export function useSimilarMasters(kind: MasterKind, text: string) {
  const settled = useSettled(text.trim(), FILTER_DELAY_MS);
  return useQuery({
    queryKey: [...MASTERS_KEY, kind, 'similar', settled],
    queryFn: () => searchMasters({ scope: kind, text: settled, limit: SIMILAR_LIMIT }),
    enabled: settled !== '',
    placeholderData: keepPreviousData,
  });
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
