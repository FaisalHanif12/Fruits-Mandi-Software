// The voucher commands as hooks (Code Standards F6). A saved advance refreshes the lists, the
// chart, the party cards and the advances of the seller, so every screen shows the new balance.

import { keepPreviousData, useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useMemo } from 'react';
import { useDebouncedValue } from '../../shared/hooks';
import { assistAdvances, vouchersPreviewAdvances, vouchersSaveAdvance } from './api';
import { hasSeller } from './draft';
import type { AdvanceDraft, ReadyAdvance } from './types';

const VOUCHERS_KEY = ['vouchers'] as const;
/** The preview waits this long after the last key press, and the one before it is dropped (F35). */
const PREVIEW_DELAY_MS = 50;

/** A seller's open advances and last advance (PRD FR-7.60). Never kept: they change with a save. */
export function useAdvances(sellerId: number | null) {
  return useQuery({
    queryKey: [...VOUCHERS_KEY, 'advances', sellerId],
    queryFn: () => assistAdvances(sellerId ?? 0),
    enabled: sellerId !== null,
    staleTime: 0,
  });
}

/**
 * The preview of the vouchers on screen: the balance before and after each, what is wrong with
 * each, and the total. Asked 50 ms after the last key, only for vouchers that have a seller and
 * an amount, so an empty form asks nothing.
 */
export function useAdvancePreview(drafts: readonly AdvanceDraft[]) {
  const ready = useMemo(
    () => drafts.filter(hasSeller).filter((draft) => draft.amount.trim() !== ''),
    [drafts],
  );
  const settled = useDebouncedValue(ready, PREVIEW_DELAY_MS);
  return {
    settled,
    query: useQuery({
      queryKey: [...VOUCHERS_KEY, 'preview', settled],
      queryFn: () => vouchersPreviewAdvances(settled),
      enabled: settled.length > 0,
      placeholderData: keepPreviousData,
      staleTime: 0,
      gcTime: 0,
    }),
  };
}

export function useSaveAdvance() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (draft: ReadyAdvance) => vouchersSaveAdvance(draft),
    onSuccess: async () => {
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: VOUCHERS_KEY }),
        queryClient.invalidateQueries({ queryKey: ['accounts'] }),
        queryClient.invalidateQueries({ queryKey: ['coa'] }),
      ]);
    },
  });
}
