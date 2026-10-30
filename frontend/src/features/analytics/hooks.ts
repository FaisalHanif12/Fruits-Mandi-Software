// The analytics commands as hooks (Code Standards F6).

import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import type { PrintAnalyticsViewInput } from '../../bindings/PrintAnalyticsViewInput';
import { usePagedQuery } from '../../shared/hooks';
import { reportsPrintHeader } from '../reports';
import {
  analyticsCatalog,
  analyticsLines,
  analyticsPrintData,
  analyticsPrintView,
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

/** The catalogue changes only with a new version of the app, so it is read once. */
export function useAnalyticsCatalog() {
  return useQuery({
    queryKey: ['analytics_catalog'],
    queryFn: analyticsCatalog,
    staleTime: Infinity,
  });
}

/**
 * A view, one page at a time. `filters` null means the view cannot run yet. The totals come with
 * every page and are the same for all of them (they are over the whole filter), so the screen reads
 * them from the page on screen. While the filters change, the rows already on screen stay until the
 * new ones come (no flicker).
 */
export function useAnalyticsPages(view: AnalyticsView, filters: AnalyticsFilters | null) {
  return usePagedQuery({
    screen: `analytics.${view.toLowerCase()}`,
    queryKey: ['analytics_run', view, filters],
    fetchPage: (offset, limit, sort) => {
      if (filters === null) throw new Error('no filters');
      return analyticsRun({ view, filters, offset, limit, sort });
    },
    rowsIn: (page) => page.rows.length,
    totalOf: (page) => page.total_count,
    enabled: filters !== null,
    keepPrevious: true,
  });
}

export interface LinesRequest {
  view: AnalyticsView;
  filters: AnalyticsFilters;
  rowKey: AnalyticsKey;
  columnKey: AnalyticsKey | null;
}

/** The lines behind one number, one page at a time. `request` null means none is open. */
export function useAnalyticsLines(request: LinesRequest | null) {
  return usePagedQuery({
    screen: 'analytics.lines',
    queryKey: ['analytics_lines', request],
    fetchPage: (offset, limit) => {
      if (request === null) throw new Error('no request');
      return analyticsLines({
        view: request.view,
        filters: request.filters,
        row_key: request.rowKey,
        column_key: request.columnKey,
        offset,
        limit,
      });
    },
    rowsIn: (page) => page.rows.length,
    totalOf: (page) => page.total_count,
    enabled: request !== null,
    // The lines of a number are a snapshot of what was on the screen when it was opened.
    snapshot: true,
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

/** Sends a view to the printer or saves it as a PDF. The caller shows the outcome or the error. */
export function useAnalyticsPrintView() {
  return useMutation({
    mutationFn: (input: PrintAnalyticsViewInput) => analyticsPrintView(input),
  });
}

/**
 * The page the hidden print window draws for a view (PRD FR-18.4): its print data and the shared
 * header and footer, read together so the window says "ready" only when both are there.
 */
export function useAnalyticsPrintBundle(input: AnalyticsPrintInput | null) {
  return useQuery({
    queryKey: ['analytics_print_bundle', input],
    queryFn: async () => {
      if (input === null) throw new Error('no print input');
      const data = await analyticsPrintData(input);
      const frame = await reportsPrintHeader({
        language: input.language,
        title_id: data.title_id,
        facts: data.filters,
      });
      return { language: frame.language, urdu_digits: frame.urdu_digits, data, frame };
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
