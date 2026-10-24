// A list that loads page by page (Code Standards F34, B31, PRD FR-19.11, NFR-SPD-24): the first page
// comes with the number of rows of the whole filter and the totals over all of them, and the next
// page starts where the loaded rows end. The totals are the backend's; this never adds a row up.
// One piece for every list of every module, so the paging rule is written once (G6).

import { keepPreviousData, useInfiniteQuery } from '@tanstack/react-query';
import { useEffect } from 'react';

/** The size of one page of a list (PRD FR-19.11, B31). */
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

interface PagedQueryOptions<TPage> {
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

export function usePagedQuery<TPage>({
  queryKey,
  fetchPage,
  rowsIn,
  totalOf,
  enabled = true,
  keepPrevious = false,
}: PagedQueryOptions<TPage>) {
  return useInfiniteQuery({
    queryKey,
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
