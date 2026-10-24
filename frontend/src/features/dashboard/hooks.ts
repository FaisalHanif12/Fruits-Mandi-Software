// The dashboard and the attention list as queries (plan 19). Both are read again after any save
// (see app/refresh-after-save.ts) and the bell is also read once a minute, so a count that changed
// without a save (a draft kept by the auto-save, a day that passed) still shows (NFR-SPD-22).

import { useQuery } from '@tanstack/react-query';
import { assistAttention, dashboardGet } from './api';

export const DASHBOARD_KEY = ['dashboard'] as const;
export const ATTENTION_KEY = ['attention'] as const;

/** How often the bell asks again by itself, in milliseconds. */
export const ATTENTION_POLL_MS = 60_000;

export function useDashboard() {
  return useQuery({ queryKey: DASHBOARD_KEY, queryFn: dashboardGet });
}

/** The attention list. `enabled` is false while nobody can be asked (locked, signed out). */
export function useAttention(enabled = true) {
  return useQuery({
    queryKey: ATTENTION_KEY,
    queryFn: assistAttention,
    enabled,
    refetchInterval: ATTENTION_POLL_MS,
  });
}
