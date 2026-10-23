// The audit commands as hooks (Code Standards F6).

import { keepPreviousData, useInfiniteQuery, useQuery } from '@tanstack/react-query';
import { auditList, auditOptions } from './api';
import type { AuditFilters } from './types';

/** A page of the log (Code Standards B31, NFR-SPD-24). */
export const AUDIT_PAGE_SIZE = 100;

/** The lists behind the filter boxes. They change only when something new is written, so a minute. */
export function useAuditOptions() {
  return useQuery({
    queryKey: ['audit_options'],
    queryFn: auditOptions,
    staleTime: 60_000,
  });
}

/**
 * The log, page by page. The total is the backend's count of the whole filter, so the screen never
 * adds up the rows it holds. While the filters change, the rows already on screen stay until the new
 * ones come.
 */
export function useAuditPages(filters: AuditFilters) {
  return useInfiniteQuery({
    queryKey: ['audit_list', filters],
    queryFn: ({ pageParam }) => auditList({ filters, offset: pageParam, limit: AUDIT_PAGE_SIZE }),
    initialPageParam: 0,
    getNextPageParam: (last, pages) => {
      const loaded = pages.reduce((count, page) => count + page.rows.length, 0);
      return last.rows.length > 0 && loaded < last.total_count ? loaded : undefined;
    },
    placeholderData: keepPreviousData,
  });
}
