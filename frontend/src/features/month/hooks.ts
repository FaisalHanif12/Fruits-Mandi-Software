// The month close commands as hooks (Code Standards F6). A close or a reversal changes the list of
// months, the preview of the month, and what the partner screens show (the capital accounts and the
// note lines of the statement), so those are asked again.

import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { PARTNERS_KEY } from '../partners';
import { monthClosePreview, monthCloseReverse, monthCloseSave, monthList } from './api';

export const MONTH_KEY = ['month'] as const;

/** The months and where each stands. Asked again whenever the screen opens (days close elsewhere). */
export function useMonthList() {
  return useQuery({ queryKey: [...MONTH_KEY, 'list'], queryFn: monthList, staleTime: 0 });
}

/**
 * What closing a month would post. `enabled` is false for a month that is closed already (there
 * is nothing to preview). A refusal is kept as the query's error: the screen says it in words.
 */
export function useMonthPreview(month: string | null, enabled: boolean) {
  return useQuery({
    queryKey: [...MONTH_KEY, 'preview', month],
    queryFn: () => monthClosePreview(month ?? ''),
    enabled: enabled && month !== null,
    staleTime: 0,
    gcTime: 0,
    retry: false,
  });
}

function useRefreshAfterChange() {
  const queryClient = useQueryClient();
  return async () => {
    await Promise.all([
      queryClient.invalidateQueries({ queryKey: MONTH_KEY }),
      queryClient.invalidateQueries({ queryKey: PARTNERS_KEY }),
      queryClient.invalidateQueries({ queryKey: ['accounts'] }),
    ]);
  };
}

export function useCloseMonth() {
  const refresh = useRefreshAfterChange();
  return useMutation({ mutationFn: (month: string) => monthCloseSave(month), onSuccess: refresh });
}

export function useReverseMonth() {
  const refresh = useRefreshAfterChange();
  return useMutation({
    mutationFn: ({ month, reason }: { month: string; reason: string }) =>
      monthCloseReverse(month, reason),
    onSuccess: refresh,
  });
}
