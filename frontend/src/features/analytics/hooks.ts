// The analytics commands as hooks (Code Standards F6).

import {
  keepPreviousData,
  useInfiniteQuery,
  useMutation,
  useQuery,
  useQueryClient,
} from '@tanstack/react-query';
import {
  analyticsCatalog,
  analyticsLines,
  analyticsPrintData,
  analyticsRun,
  analyticsViewsGet,
  analyticsViewsSave,
} from './api';
import type {
  AnalyticsFilters,
  AnalyticsKey,
  AnalyticsPrintInput,
  AnalyticsSavedView,
  AnalyticsView,
} from './types';

/** A page of a view or of its lines (Code Standards B31, NFR-SPD-24). */
export const ANALYTICS_PAGE_SIZE = 100;

/** The catalogue changes only with a new version of the app, so it is read once. */
export function useAnalyticsCatalog() {
  return useQuery({
    queryKey: ['analytics_catalog'],
    queryFn: analyticsCatalog,
    staleTime: Infinity,
  });
}

/**
 * A view, page by page. `filters` null means the view cannot run yet. The totals come with every
 * page and are the same for all of them (they are over the whole filter), so the screen reads them
 * from the last page. While the filters change, the rows already on screen stay until the new ones
 * come (no flicker).
 */
export function useAnalyticsPages(view: AnalyticsView, filters: AnalyticsFilters | null) {
  return useInfiniteQuery({
    queryKey: ['analytics_run', view, filters],
    queryFn: ({ pageParam }) => {
      if (filters === null) throw new Error('no filters');
      return analyticsRun({ view, filters, offset: pageParam, limit: ANALYTICS_PAGE_SIZE });
    },
    enabled: filters !== null,
    initialPageParam: 0,
    getNextPageParam: (last, pages) => {
      const loaded = pages.reduce((count, page) => count + page.rows.length, 0);
      return last.rows.length > 0 && loaded < last.total_count ? loaded : undefined;
    },
    placeholderData: keepPreviousData,
  });
}

export interface LinesRequest {
  view: AnalyticsView;
  filters: AnalyticsFilters;
  rowKey: AnalyticsKey;
  columnKey: AnalyticsKey | null;
}

/** The lines behind one number, page by page. `request` null means none is open. */
export function useAnalyticsLines(request: LinesRequest | null) {
  return useInfiniteQuery({
    queryKey: ['analytics_lines', request],
    queryFn: ({ pageParam }) => {
      if (request === null) throw new Error('no request');
      return analyticsLines({
        view: request.view,
        filters: request.filters,
        row_key: request.rowKey,
        column_key: request.columnKey,
        offset: pageParam,
        limit: ANALYTICS_PAGE_SIZE,
      });
    },
    enabled: request !== null,
    initialPageParam: 0,
    getNextPageParam: (last, pages) => {
      const loaded = pages.reduce((count, page) => count + page.rows.length, 0);
      return last.rows.length > 0 && loaded < last.total_count ? loaded : undefined;
    },
    // The lines of a number are a snapshot of what was on the screen when it was opened.
    staleTime: Infinity,
    gcTime: 0,
  });
}

/** The print data of a view with every row. A print is a snapshot: never read again by itself. */
export function useAnalyticsPrintData(input: AnalyticsPrintInput | null) {
  return useQuery({
    queryKey: ['analytics_print_data', input],
    queryFn: () => {
      if (input === null) throw new Error('no print input');
      return analyticsPrintData(input);
    },
    enabled: input !== null,
    staleTime: Infinity,
    gcTime: 0,
  });
}

const SAVED_VIEWS_KEY = ['analytics_views'] as const;

/** The saved views of this user (PRD FR-18.5). */
export function useSavedViews() {
  return useQuery({ queryKey: SAVED_VIEWS_KEY, queryFn: analyticsViewsGet });
}

/** Replaces the list of saved views; the answer is what the backend kept. */
export function useSaveViews() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (views: AnalyticsSavedView[]) => analyticsViewsSave({ views }),
    onSuccess: (kept) => {
      queryClient.setQueryData(SAVED_VIEWS_KEY, kept);
    },
  });
}
