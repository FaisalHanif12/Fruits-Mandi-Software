// The خریداری commands as hooks (Code Standards F6). A saved purchase, a cancel or a write-off
// refreshes everything that shows what it wrote (PRD 6.2): the trips, the stock list of باقی مال,
// the party cards, the cash and the partners' totals, so every screen shows the new numbers.

import {
  keepPreviousData,
  useMutation,
  useQuery,
  useQueryClient,
  type QueryClient,
} from '@tanstack/react-query';
import { useDebouncedValue } from '../../shared/hooks';
import {
  purchasesCancel,
  purchasesGet,
  purchasesList,
  purchasesPreview,
  purchasesSave,
  purchasesWriteOff,
  purchasesSetSellDate,
  purchasesWriteOffUndo,
} from './api';
import type { SellDateInput } from '../../bindings/SellDateInput';
import type { PurchaseCancelInput, PurchaseInput, PurchasesListInput } from './types';

export const PURCHASES_KEY = ['purchases'] as const;
/** The preview waits this long after the last key press, and the one before it is dropped (F35). */
const PREVIEW_DELAY_MS = 50;
/** A pause so a burst of key presses in a filter asks once. */
const FILTER_DELAY_MS = 150;

/** Everything a posting of a trip's goods changes, to read again (PRD 6.2). */
const READ_AGAIN: readonly (readonly string[])[] = [
  PURCHASES_KEY,
  ['trips'],
  ['bikri'],
  ['accounts'],
  ['coa'],
  ['partners'],
  ['search'],
  ['day_status'],
];

/** Reads again what a purchase, a cancel or a write-off changed. */
export function readAgainAfterPurchase(queryClient: QueryClient): Promise<unknown> {
  return Promise.all(
    READ_AGAIN.map((queryKey) => queryClient.invalidateQueries({ queryKey: [...queryKey] })),
  );
}

/**
 * The preview of the purchase on the form: the cost lines with their workings, each partner's
 * part, the warnings and the cash after. Asked 50 ms after the last key, only once the form is
 * filled, so an empty form asks nothing.
 */
export function usePurchasePreview(input: PurchaseInput | null) {
  const settled = useDebouncedValue(input, PREVIEW_DELAY_MS);
  const query = useQuery({
    queryKey: [...PURCHASES_KEY, 'preview', settled],
    queryFn: () => {
      if (settled === null) throw new Error('no purchase');
      return purchasesPreview(settled);
    },
    enabled: settled !== null,
    placeholderData: keepPreviousData,
    staleTime: 0,
    gcTime: 0,
  });
  // The answer is for what is on screen only when the input has stayed as it was asked about.
  const current = settled === input && !query.isPlaceholderData && query.isSuccess;
  return { query, current };
}

/** The purchases for the filters, with the totals over the whole filter (PRD FR-19.11). */
export function usePurchasesList(filter: PurchasesListInput, enabled = true) {
  const settled = useDebouncedValue(filter, FILTER_DELAY_MS);
  return useQuery({
    queryKey: [...PURCHASES_KEY, 'list', settled],
    queryFn: () => purchasesList(settled),
    enabled,
    placeholderData: keepPreviousData,
    staleTime: 0,
  });
}

/** One purchase with the lots that sold it and its write-offs. Never kept: a sale changes it. */
export function usePurchase(id: number | null) {
  return useQuery({
    queryKey: [...PURCHASES_KEY, 'one', id],
    queryFn: () => purchasesGet(id ?? 0),
    enabled: id !== null,
    staleTime: 0,
  });
}

export function useSavePurchase() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: PurchaseInput) => purchasesSave(input),
    onSuccess: () => readAgainAfterPurchase(queryClient),
  });
}

export function useCancelPurchase() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: PurchaseCancelInput) => purchasesCancel(input),
    onSuccess: () => readAgainAfterPurchase(queryClient),
  });
}

/** Changes the date to sell of a purchase with goods left (PRD FR-12.39). */
export function useSetPurchaseSellDate() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: SellDateInput) => purchasesSetSellDate(input),
    // The goods come into a new بکری from the new day: that list is read again too.
    onSuccess: () => readAgainAfterPurchase(queryClient),
  });
}

export function useWriteOffOwn() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({
      purchaseId,
      qty,
      reason,
    }: {
      purchaseId: number;
      qty: string;
      reason: string;
    }) => purchasesWriteOff(purchaseId, qty, reason),
    onSuccess: () => readAgainAfterPurchase(queryClient),
  });
}

export function useWriteOffOwnUndo() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ moveId, reason }: { moveId: number; reason: string }) =>
      purchasesWriteOffUndo(moveId, reason),
    onSuccess: () => readAgainAfterPurchase(queryClient),
  });
}
