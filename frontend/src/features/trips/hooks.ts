// The مارکیٹ سفر commands as hooks (Code Standards F6). A saved trip, entry or cancel refreshes
// everything that shows what it wrote (PRD 6.2): the trips, the purchases, the stock list of باقی
// مال, the party cards, the cash, the day status and the partners' totals, so every screen shows
// the new numbers.

import {
  keepPreviousData,
  useMutation,
  useQuery,
  useQueryClient,
  type QueryClient,
} from '@tanstack/react-query';
import type { SortInput } from '../../bindings/SortInput';
import { useAllPagesQuery, useDebouncedValue, usePagedQuery } from '../../shared/hooks';
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

/** What the list of trips asks the backend, for the page buttons and for a choice list alike. */
function tripsListQuery(settled: Omit<TripsListInput, 'offset' | 'limit' | 'sort'>) {
  return {
    queryKey: [...TRIPS_KEY, 'list', settled],
    fetchPage: (offset: number, limit: number, sort: SortInput | null = null) =>
      tripsList({
        ...settled,
        offset,
        limit,
        sort,
        // Month rows only while the list is in date order.
        group_by_month: settled.group_by_month && (sort === null || sort.column === 'entry_date'),
      }),
    rowsIn: (page: { rows: readonly unknown[] }) => page.rows.length,
    totalOf: (page: { total_count: number }) => page.total_count,
    keepPrevious: true,
  };
}

/**
 * The trips for the filters, one page at a time, with the totals over the whole filter (PRD
 * FR-19.11). The first page carries the count and the totals.
 */
export function useTripsList(
  filter: Omit<TripsListInput, 'offset' | 'limit' | 'sort'>,
  enabled = true,
) {
  const settled = useDebouncedValue(filter, FILTER_DELAY_MS);
  return usePagedQuery({ screen: 'trips.list', ...tripsListQuery(settled), enabled });
}

/** Every trip of the filter, page after page, for a choice list (`useLoadRest`). */
export function useAllTrips(filter: Omit<TripsListInput, 'offset' | 'limit' | 'sort'>) {
  return useAllPagesQuery(tripsListQuery(filter));
}

/** What a trip screen shows: the trip, its purchases, its partners, and the entries page on screen. */
export interface TripView {
  /** The page on screen: the trip, its purchases and each partner's money (always the latest). */
  detail: TripDetail;
  /** The entries of this page, newest first. */
  entries: TripDetail['entries'];
  /** How many entries the trip has in all. */
  entriesCount: number;
}

/** One trip, with its entries one page at a time (PRD FR-19.11). Never kept: a sale changes it. */
export function useTrip(id: number | null) {
  const query = usePagedQuery({
    screen: 'trips.entries',
    queryKey: [...TRIPS_KEY, 'one', id],
    enabled: id !== null,
    fetchPage: (offset, limit) =>
      tripsGet({ id: id ?? 0, entries_offset: offset, entries_limit: limit }),
    rowsIn: (page) => page.entries.length,
    totalOf: (page) => page.entries_count,
  });
  const first = query.data?.pages[0];
  const view: TripView | undefined =
    first === undefined
      ? undefined
      : { detail: first, entries: first.entries, entriesCount: first.entries_count };
  return { ...query, data: view };
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
