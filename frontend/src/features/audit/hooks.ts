// The audit commands as hooks (Code Standards F6).

import { useQuery } from '@tanstack/react-query';
import { usePagedQuery } from '../../shared/hooks';
import { auditList, auditOptions } from './api';
import type { AuditFilters } from './types';

/** The lists behind the filter boxes. They change only when something new is written, so a minute. */
export function useAuditOptions() {
  return useQuery({
    queryKey: ['audit_options'],
    queryFn: auditOptions,
    staleTime: 60_000,
  });
}

/**
 * The log, one page at a time (Code Standards B31, NFR-SPD-24). The total is the backend's count of
 * the whole filter, so the screen never adds up the rows it holds. While the filters change, the
 * rows already on screen stay until the new ones come.
 */
export function useAuditPages(filters: AuditFilters, groupByMonth = false) {
  return usePagedQuery({
    screen: 'audit.log',
    queryKey: ['audit_list', filters, groupByMonth],
    fetchPage: (offset, limit, sort) =>
      auditList({ filters, offset, limit, sort, group_by_month: groupByMonth && sort === null }),
    rowsIn: (page) => page.rows.length,
    totalOf: (page) => page.total_count,
    keepPrevious: true,
  });
}
