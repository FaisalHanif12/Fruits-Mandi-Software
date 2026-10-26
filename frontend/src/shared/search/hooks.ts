// Searching for a picker as hooks (Code Standards F6). The answer to an older text is dropped
// when a newer one has been asked, so a slow answer never shows under the wrong text, and the
// list keeps the last answer while the next one is on its way, so it never flashes empty.

import {
  keepPreviousData,
  useInfiniteQuery,
  useMutation,
  useQueryClient,
} from '@tanstack/react-query';
import { useCallback, useEffect, useMemo, useState } from 'react';
import { searchMasters, searchRecent, searchTouch } from './api';
import type { SearchHit, SearchScope } from './types';

/** How many matches a typed search asks for. The list is short on purpose: type more to narrow. */
export const PICKER_RESULT_LIMIT = 50;
/**
 * How many records one page of a picker list holds (plan 32, R7). A picker opened by a click lists
 * every record of its kind; it asks for one page, and the next page when the user scrolls near
 * the end (Code Standards B31, F34).
 */
export const PICKER_PAGE_SIZE = 50;
/** The rows left below the view when the next page is asked for. */
export const PICKER_LOAD_AHEAD_ROWS = 15;
/** A short pause so a burst of key presses asks once (NFR-SPD-6 allows 50 ms in all). */
export const PICKER_SEARCH_DELAY_MS = 30;

const SEARCH_KEY = ['search'] as const;

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

/** What a picker needs to find records: how to search, what is recent, and how to remember a pick. */
export interface PickerSource<H> {
  /** Names the source in the query cache, for example `['search', 'city']`. */
  readonly key: readonly string[];
  /** Records whose code or name matches `text`. */
  search: (text: string, limit: number) => Promise<H[]>;
  /** The last 10 records used, then the rest (PRD FR-7.41). */
  recent: () => Promise<H[]>;
  /**
   * One page of the list for `text` ("" is every record of the kind: recent first, then by code),
   * `limit` records from `offset`. A source that has it is read page by page as the list scrolls
   * (plan 32, R7); one that has not is read once, by `search` and `recent`.
   */
  page?: ((text: string, offset: number, limit: number) => Promise<H[]>) | undefined;
  /** Remembers that a record was chosen, so it comes first next time. */
  touch: (id: number) => Promise<unknown>;
}

/** The source of a master list: city, item, packing and the rest (plan 06). */
export function masterSource(scope: SearchScope): PickerSource<SearchHit> {
  return {
    key: [...SEARCH_KEY, scope],
    search: (text, limit) => searchMasters({ scope, text, limit }),
    recent: () => searchRecent({ scope }),
    // The click-open list and every typed search are read a page at a time (plan 32, R7).
    page: (text, offset, limit) => searchMasters({ scope, text, limit, offset }),
    touch: (id) => searchTouch({ scope, id }),
  };
}

/**
 * The records a picker shows. Typed text searches; empty text (a click, `Alt+↓`, or the field
 * emptied) lists every record of the kind, recent first, then by code (PRD FR-7.41, plan 32 R7).
 * Nothing is asked while the list is closed. A source with pages is read one page at a time:
 * `loadMore` asks for the next one when the user scrolls near the end.
 *
 * `current` says whether the list answers the text in the field right now. It is false while the
 * list is an older answer kept on screen (the search waits a moment after the last key, and keeps
 * the last answer while the next one is on its way), so a key that acts on the list (Enter) can
 * tell a list it may trust from one it must not (a rapid "1" then "2" must not pick the match of
 * "1" while the field shows "12").
 */
export function usePickerHits<H>(source: PickerSource<H>, text: string, open: boolean) {
  const wanted = text.trim();
  const typed = useSettled(wanted, PICKER_SEARCH_DELAY_MS);
  // Emptying the field must not wait: the full list is the answer at once.
  const asked = wanted === '' ? '' : typed;
  const paged = source.page;
  const query = useInfiniteQuery({
    queryKey: [...source.key, asked],
    queryFn: ({ pageParam }) => {
      if (paged !== undefined) return paged(asked, pageParam, PICKER_PAGE_SIZE);
      return asked === '' ? source.recent() : source.search(asked, PICKER_RESULT_LIMIT);
    },
    initialPageParam: 0,
    // A page that is full may have another after it; a short page is the end of the list.
    getNextPageParam: (last, all) =>
      paged === undefined || last.length < PICKER_PAGE_SIZE
        ? undefined
        : all.reduce((count, page) => count + page.length, 0),
    enabled: open,
    placeholderData: keepPreviousData,
    // The same text can give a different answer after a save, so an answer is never reused.
    staleTime: 0,
    gcTime: 0,
  });
  const rows = useMemo(() => query.data?.pages.flat() ?? [], [query.data]);
  // A failed answer is final too: there is nothing more to wait for.
  const current =
    asked === wanted && !query.isPlaceholderData && (query.isSuccess || query.isError);
  const { hasNextPage, isFetchingNextPage, fetchNextPage } = query;
  const loadMore = useCallback(() => {
    if (hasNextPage && !isFetchingNextPage) void fetchNextPage();
  }, [hasNextPage, isFetchingNextPage, fetchNextPage]);
  return {
    rows,
    isPending: query.isPending,
    isError: query.isError,
    error: query.error,
    current,
    hasMore: hasNextPage,
    loadMore,
  };
}

/** Tells the backend a record was chosen, so it comes first next time (PRD FR-7.41). */
export function useTouchRecord<H>(source: PickerSource<H>) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: number) => source.touch(id),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: [...source.key] }),
  });
}

/**
 * Asks for the answer to the whole text now, without the short pause, for an Enter that came
 * before the list caught up with the field. It shares the cache of `usePickerHits`, so a request
 * already on its way is not asked twice.
 */
export function useFetchHits<H>(source: PickerSource<H>) {
  const queryClient = useQueryClient();
  return useCallback(
    (text: string) =>
      queryClient.query({
        // Its own key: the list of a picker is cached as pages, this answer is one array.
        queryKey: [...source.key, 'whole', text],
        queryFn: () => source.search(text, PICKER_RESULT_LIMIT),
        staleTime: 0,
      }),
    [queryClient, source],
  );
}
