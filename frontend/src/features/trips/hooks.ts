// The مارکیٹ سفر commands as hooks (Code Standards F6). A saved trip, entry or cancel refreshes
// everything that shows what it wrote (PRD 6.2): the trips, the purchases, the stock list of باقی
// مال, the party cards, the cash, the day status and the partners' totals, so every screen shows
// the new numbers.

import {
  keepPreviousData,
  useInfiniteQuery,
  useMutation,
  useQuery,
  useQueryClient,
  type QueryClient,
} from '@tanstack/react-query';
import { PAGE_SIZE, useDebouncedValue, usePagedQuery } from '../../shared/hooks';
import { readAgainAfterPurchase } from '../purchases';
import {
  tripsCostCancel,
  tripsCostPreview,
  tripsCostSave,
  tripsCreate,
  tripsDefaults,
  tripsGet,
  tripsList,
  tripsOtherSaleCancel,
  tripsOtherSalePreview,
  tripsOtherSaleSave,
  tripsSummary,
} from './api';
import type {
  OtherSaleInput,
  TripCostInput,
  TripDetail,
  TripEntryCancelInput,
  TripInput,
  TripsListInput,
} from './types';

export const TRIPS_KEY = ['trips'] as const;
/** The preview waits this long after the last key press, and the one before it is dropped (F35). */
const PREVIEW_DELAY_MS = 50;
/** A pause so a burst of key presses in a filter asks once. */
const FILTER_DELAY_MS = 150;

/** Reads again what a trip entry or a cancel changed. The purchase hooks read the same set. */
export function readAgainAfterTrip(queryClient: QueryClient): Promise<unknown> {
  return readAgainAfterPurchase(queryClient);
}

/** The new trip form's start: the partners with the % and sources of the last trip (PRD A56). */
export function useTripDefaults() {
  return useQuery({
    queryKey: [...TRIPS_KEY, 'defaults'],
    queryFn: tripsDefaults,
    // The defaults change with every trip made, so a fresh form always asks again.
    staleTime: 0,
    gcTime: 0,
  });
}

export function useCreateTrip() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: TripInput) => tripsCreate(input),
    onSuccess: () => readAgainAfterTrip(queryClient),
  });
}

/**
 * The trips for the filters, page by page, with the totals over the whole filter (PRD FR-19.11).
 * `fetchNextPage` loads the next 100 rows; the first page carries the count and the totals.
 */
export function useTripsList(filter: Omit<TripsListInput, 'offset' | 'limit'>) {
  const settled = useDebouncedValue(filter, FILTER_DELAY_MS);
  return usePagedQuery({
    queryKey: [...TRIPS_KEY, 'list', settled],
    fetchPage: (offset, limit) => tripsList({ ...settled, offset, limit }),
    rowsIn: (page) => page.rows.length,
    totalOf: (page) => page.total_count,
    keepPrevious: true,
  });
}

/** What a trip screen shows: the trip, its purchases, its partners, and all loaded entries. */
export interface TripView {
  /** The first page: the trip, its purchases and each partner's money (always the latest). */
  detail: TripDetail;
  /** Every entry loaded so far, newest first. */
  entries: TripDetail['entries'];
  /** How many entries the trip has in all. */
  entriesCount: number;
}

/** One trip, with its history loaded page by page (PRD FR-19.11). Never kept: a sale changes it. */
export function useTrip(id: number | null) {
  return useInfiniteQuery({
    queryKey: [...TRIPS_KEY, 'one', id],
    initialPageParam: 0,
    enabled: id !== null,
    queryFn: ({ pageParam }) =>
      tripsGet({ id: id ?? 0, entries_offset: pageParam, entries_limit: PAGE_SIZE }),
    getNextPageParam: (last, pages) => {
      const loaded = pages.reduce((sum, page) => sum + page.entries.length, 0);
      return loaded < last.entries_count && last.entries.length > 0 ? loaded : undefined;
    },
    select: (data): TripView | undefined => {
      const first = data.pages[0];
      if (first === undefined) return undefined;
      return {
        detail: first,
        entries: data.pages.flatMap((page) => page.entries),
        entriesCount: first.entries_count,
      };
    },
    staleTime: 0,
  });
}

/** The data of the dashboard tile مارکیٹ سفر (PRD 9.1). */
export function useTripsSummary() {
  return useQuery({ queryKey: [...TRIPS_KEY, 'summary'], queryFn: tripsSummary, staleTime: 0 });
}

/** The preview of a trip cost on the form: each partner's part and the spread (C23, C24). */
export function useCostPreview(input: TripCostInput | null) {
  const settled = useDebouncedValue(input, PREVIEW_DELAY_MS);
  const query = useQuery({
    queryKey: [...TRIPS_KEY, 'cost-preview', settled],
    queryFn: () => {
      if (settled === null) throw new Error('no cost');
      return tripsCostPreview(settled);
    },
    enabled: settled !== null,
    placeholderData: keepPreviousData,
    staleTime: 0,
    gcTime: 0,
  });
  const current = settled === input && !query.isPlaceholderData && query.isSuccess;
  return { query, current };
}

/** The preview of a sale in another shop: the cost, the result and each partner's part (C25). */
export function useOtherSalePreview(input: OtherSaleInput | null) {
  const settled = useDebouncedValue(input, PREVIEW_DELAY_MS);
  const query = useQuery({
    queryKey: [...TRIPS_KEY, 'sale-preview', settled],
    queryFn: () => {
      if (settled === null) throw new Error('no sale');
      return tripsOtherSalePreview(settled);
    },
    enabled: settled !== null,
    placeholderData: keepPreviousData,
    staleTime: 0,
    gcTime: 0,
  });
  const current = settled === input && !query.isPlaceholderData && query.isSuccess;
  return { query, current };
}

export function useSaveCost() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: TripCostInput) => tripsCostSave(input),
    onSuccess: () => readAgainAfterTrip(queryClient),
  });
}

export function useSaveOtherSale() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: OtherSaleInput) => tripsOtherSaleSave(input),
    onSuccess: () => readAgainAfterTrip(queryClient),
  });
}

export function useCancelCost() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: TripEntryCancelInput) => tripsCostCancel(input),
    onSuccess: () => readAgainAfterTrip(queryClient),
  });
}

export function useCancelOtherSale() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: TripEntryCancelInput) => tripsOtherSaleCancel(input),
    onSuccess: () => readAgainAfterTrip(queryClient),
  });
}
