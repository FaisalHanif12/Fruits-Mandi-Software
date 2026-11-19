// A list that loads page by page (Code Standards F34, B31, PRD FR-19.11, NFR-SPD-24, plan 38 R1):
// one page of the filter at a time, with the number of rows of the whole filter and the totals over
// all of them. The totals are the backend's; this never adds a row up. One piece for every list of
// every module, so the paging rule is written once (G6).
//
// `usePagedQuery` is for a list with page buttons: `data.pages` holds the one page on screen, and
// `pager` says where it is and moves it. `useAllPagesQuery` and `useLoadRest` are for a screen that
// needs the whole list for a moment (a choice list of gardens, the opening balances).

import { keepPreviousData, useInfiniteQuery, useQuery } from '@tanstack/react-query';
import { useEffect, useMemo, useState } from 'react';
import type { SortInput } from '../../bindings/SortInput';
import { describePager, usePagerControl } from './usePager';

/** The largest page of any list (PRD FR-19.11, B31); the most a command is asked for at once. */
export const PAGE_SIZE = 100;

/**
 * Where the next page starts, or undefined when every row is loaded. Example: 100 rows loaded of
 * 154 gives 100; 154 of 154 gives undefined; an empty last page stops the loading.
 */
export function nextOffset(
  loaded: number,
  total: number,
  lastPageRows: number,
): number | undefined {
  return loaded < total && lastPageRows > 0 ? loaded : undefined;
}

interface ListQueryOptions<TPage> {
  queryKey: readonly unknown[];
  /** Asks the backend for one page: where it starts and how many rows it may hold. */
  fetchPage: (offset: number, limit: number) => Promise<TPage>;
  /** How many rows one page holds. */
  rowsIn: (page: TPage) => number;
  /** How many rows the whole filter has, as the backend counted them. */
  totalOf: (page: TPage) => number;
  /** False while the list has nothing to ask about (no person chosen yet). */
  enabled?: boolean;
  /** Keep the rows of the last filter on screen while a new filter loads. */
  keepPrevious?: boolean;
}

interface PagedQueryOptions<TPage> extends Omit<ListQueryOptions<TPage>, 'fetchPage'> {
  /**
   * Asks the backend for one page: where it starts, how many rows it may hold, and the order the
   * Owner chose on a column title (null: the list's own order). The backend sorts the whole filter,
   * never only the page (plan 38 R4).
   */
  fetchPage: (offset: number, limit: number, sort: SortInput | null) => Promise<TPage>;
  /**
   * The name of the list, for the rows per page and the page it remembers: `cashbook.takes`
   * (lower-case letters, digits and `_ - .`; with its prefix at most 40 characters).
   */
  screen: string;
  /**
   * The pages are a snapshot of what was on screen when the list opened (the lines behind an
   * analytics number): never read again by themselves, and not kept after the list closes.
   */
  snapshot?: boolean;
}

export function usePagedQuery<TPage>({
  screen,
  queryKey,
  fetchPage,
  rowsIn,
  totalOf,
  enabled = true,
  keepPrevious = false,
  snapshot = false,
}: PagedQueryOptions<TPage>) {
  // The order of the list: a click on a column title (plan 38 R4). A new order is a new filter, so
  // the list goes back to page 1.
  const [sort, setSort] = useState<SortInput | null>(null);
  const keyed = useMemo(() => [...queryKey, { sort }], [queryKey, sort]);
  const filterKey = JSON.stringify(keyed);
  const control = usePagerControl(screen, filterKey);
  const { page, pageSize } = control;
  const query = useQuery({
    queryKey: [...keyed, 'page', pageSize, page],
    queryFn: () => fetchPage((page - 1) * pageSize, pageSize, sort),
    // The saved rows per page is read first, so the first page is asked for once, with its size.
    enabled: enabled && control.ready,
    // Between two pages of one filter the page on screen stays until the next one is here (no
    // flicker). Between two filters it stays only when the screen asks for it.
    placeholderData: (previous, previousQuery) => {
      if (previous === undefined || previousQuery === undefined) return undefined;
      const sameFilter =
        JSON.stringify(previousQuery.queryKey.slice(0, keyed.length)) === filterKey;
      return keepPrevious || sameFilter ? previous : undefined;
    },
    staleTime: snapshot ? Infinity : 0,
    ...(snapshot ? { gcTime: 0 } : {}),
  });

  const current = query.data;
  const total = current === undefined ? 0 : totalOf(current);
  const pager = describePager(control, total, query.isFetching);
  // A page past the end (the last row of the last page was cancelled) goes to the last page.
  const pastTheEnd =
    current !== undefined &&
    !query.isPlaceholderData &&
    rowsIn(current) === 0 &&
    total > 0 &&
    page > pager.pageCount;
  const { setPage } = control;
  const lastPage = pager.pageCount;
  useEffect(() => {
    if (pastTheEnd) setPage(lastPage);
  }, [pastTheEnd, setPage, lastPage]);

  const data = useMemo(() => (current === undefined ? undefined : { pages: [current] }), [current]);
  return {
    /** `pages` holds the one page on screen. */
    data,
    pager,
    /** The order chosen on a column title, and how to change it (null: the list's own order). */
    sort,
    setSort,
    isPending: query.isPending || !control.ready,
    isError: query.isError,
    error: query.error,
    isFetching: query.isFetching,
    isPlaceholderData: query.isPlaceholderData,
    refetch: query.refetch,
  };
}

/** The same list for a screen that needs every row: it reads page after page (`useLoadRest`). */
export function useAllPagesQuery<TPage>({
  queryKey,
  fetchPage,
  rowsIn,
  totalOf,
  enabled = true,
  keepPrevious = false,
}: ListQueryOptions<TPage>) {
  return useInfiniteQuery({
    // Its own key: the pages of the page buttons have another shape under the same filter.
    queryKey: [...queryKey, 'all'],
    initialPageParam: 0,
    enabled,
    queryFn: ({ pageParam }) => fetchPage(pageParam, PAGE_SIZE),
    getNextPageParam: (last, pages) =>
      nextOffset(
        pages.reduce((sum, page) => sum + rowsIn(page), 0),
        totalOf(last),
        rowsIn(last),
      ),
    ...(keepPrevious ? { placeholderData: keepPreviousData } : {}),
    staleTime: 0,
  });
}

/**
 * Loads the pages still to come, one after the other, while `wanted` is true. For a screen that
 * needs the whole list for a moment: a filter that is matched against the rows, or a record that
 * may be on a later page. The list still comes page by page; nothing is asked when `wanted` is
 * false.
 */
export function useLoadRest(
  query: {
    hasNextPage: boolean;
    isFetchingNextPage: boolean;
    fetchNextPage: () => unknown;
  },
  wanted: boolean,
): void {
  const { hasNextPage, isFetchingNextPage, fetchNextPage } = query;
  useEffect(() => {
    if (wanted && hasNextPage && !isFetchingNextPage) void fetchNextPage();
  }, [wanted, hasNextPage, isFetchingNextPage, fetchNextPage]);
}
