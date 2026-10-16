// The day commands as hooks (Code Standards F6). Closing or reopening a day changes what the status
// bar, the dashboard and every entry screen allow, so they are asked again.

import { keepPreviousData, useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useDebouncedValue } from '../../shared/hooks';
import { dayClose, dayClosePreview, dayReopen } from './api';

const DAY_KEY = ['day_status'] as const;
const PREVIEW_KEY = ['day-close-preview'] as const;
/** A short pause so a burst of key presses in the date asks once (Code Standards F35). */
const PREVIEW_DELAY_MS = 50;

/**
 * What closing up to the typed date would do. With no date typed the backend answers for the
 * oldest open day (PRD FR-21.1) and says which date that is, so the screen can show it. When no
 * day is open (today is closed already) there is nothing to ask, so `enabled` is false.
 */
export function useDayClosePreview(upTo: string, enabled = true) {
  const settled = useDebouncedValue(upTo.trim(), PREVIEW_DELAY_MS);
  return useQuery({
    queryKey: [...PREVIEW_KEY, settled],
    queryFn: () => dayClosePreview(settled),
    enabled,
    placeholderData: keepPreviousData,
    staleTime: 0,
    gcTime: 0,
  });
}

function useRefreshAfterChange() {
  const queryClient = useQueryClient();
  return async () => {
    await Promise.all([
      queryClient.invalidateQueries({ queryKey: DAY_KEY }),
      queryClient.invalidateQueries({ queryKey: PREVIEW_KEY }),
    ]);
  };
}

export function useCloseDays() {
  const refresh = useRefreshAfterChange();
  return useMutation({ mutationFn: (upTo: string) => dayClose(upTo), onSuccess: refresh });
}

export function useReopenDay() {
  const refresh = useRefreshAfterChange();
  return useMutation({
    mutationFn: ({ date, reason }: { date: string; reason: string }) => dayReopen(date, reason),
    onSuccess: refresh,
  });
}
