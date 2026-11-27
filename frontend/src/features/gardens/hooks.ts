// The باغات commands as hooks (Code Standards F6). A saved garden, entry, closing or cancel
// refreshes everything that shows what it wrote (PRD 6.2): the gardens, the sale book (a garden
// is offered as a lot seller), the party cards, the cash, the day status and the partners' totals,
// so every screen shows the new numbers.

import type { GardenReopenRequest } from '../../bindings/GardenReopenRequest';
import type { GardenEntryCancelRequest } from '../../bindings/GardenEntryCancelRequest';
import type { ArrivalWriteOffUndoRequest } from '../../bindings/ArrivalWriteOffUndoRequest';
import type { ArrivalCancelRequest } from '../../bindings/ArrivalCancelRequest';
import { useMutation, useQuery, useQueryClient, type QueryClient } from '@tanstack/react-query';
import type { GardenArrivalEditInput } from '../../bindings/GardenArrivalEditInput';
import type { GardenEntryEditInput } from '../../bindings/GardenEntryEditInput';
import type { GardenHeaderEditInput } from '../../bindings/GardenHeaderEditInput';
import type { SortInput } from '../../bindings/SortInput';
import {
  RECORDS_PAGE_SIZE,
  useAllPagesQuery,
  useDebouncedValue,
  useLoadRest,
  usePagedQuery,
  usePreviewQuery,
} from '../../shared/hooks';
import {
  gardensArrivalCancel,
  gardensArrivalChangeSplit,
  gardensArrivalEntryPreview,
  gardensArrivalEditSave,
  gardensArrivalEntrySave,
  gardensArrivalSave,
  gardensArrivalSetSellDate,
  gardensArrivalWriteOff,
  gardensArrivalWriteOffUndo,
  gardensArrivalsList,
  gardensClose,
  gardensClosePreview,
  gardensCreate,
  gardensCreatePreview,
  gardensDefaults,
  gardensEntryCancel,
  gardensEntryEditSave,
  gardensEntryPreview,
  gardensEntrySave,
  gardensGet,
  gardensHeaderEditSave,
  gardensList,
  gardensReopen,
  gardensSaleMoney,
  gardensSummary,
} from './api';
import type {
  ArrivalEntryInput,
  ArrivalInput,
  ArrivalsListInput,
  ArrivalWriteOffInput,
  SellDateInput,
  SplitInput,
  GardenCloseInput,
  GardenDetail,
  GardenEntryInput,
  GardenInput,
  GardensListInput,
} from './types';

export const GARDENS_KEY = ['gardens'] as const;
/** A pause so a burst of key presses in a filter asks once. */
const FILTER_DELAY_MS = 150;

/** Everything a garden posting changes, to read again (PRD 6.2). */
const READ_AGAIN: readonly (readonly string[])[] = [
  GARDENS_KEY,
  ['bikri'],
  ['accounts'],
  ['coa'],
  ['partners'],
  ['search'],
  ['day_status'],
];

/** Reads again what a garden, an entry, a closing or a cancel changed. */
export function readAgainAfterGarden(queryClient: QueryClient): Promise<unknown> {
  return Promise.all(
    READ_AGAIN.map((queryKey) => queryClient.invalidateQueries({ queryKey: [...queryKey] })),
  );
}

/** The new garden form's start: the partners with a % and a source (PRD A56). */
export function useGardenDefaults() {
  return useQuery({
    queryKey: [...GARDENS_KEY, 'defaults'],
    queryFn: gardensDefaults,
    // The defaults change with every garden made, so a fresh form always asks again.
    staleTime: 0,
    gcTime: 0,
  });
}

/** What making the garden would do, live under the form. A null input asks nothing. */
export function useGardenCreatePreview(input: GardenInput | null) {
  return usePreviewQuery([...GARDENS_KEY, 'create-preview'], input, gardensCreatePreview);
}

export function useCreateGarden() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: GardenInput) => gardensCreate(input),
    onSuccess: () => readAgainAfterGarden(queryClient),
  });
}

/** What the list of gardens asks the backend, for the page buttons and for a choice list alike. */
function gardensListQuery(settled: Omit<GardensListInput, 'offset' | 'limit' | 'sort'>) {
  return {
    queryKey: [...GARDENS_KEY, 'list', settled],
    fetchPage: (offset: number, limit: number, sort: SortInput | null = null) =>
      gardensList({
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
 * The gardens for the filters, one page at a time, with the totals over the whole filter (PRD
 * FR-19.11). The first page carries the count and the totals.
 */
export function useGardensList(
  filter: Omit<GardensListInput, 'offset' | 'limit' | 'sort'>,
  enabled = true,
) {
  const settled = useDebouncedValue(filter, FILTER_DELAY_MS);
  // The gardens are records, not days: 30 rows a page (PRD FR-19.11 v1.57, plan 95 item 18b).
  return usePagedQuery({
    screen: 'gardens.list',
    ...gardensListQuery(settled),
    enabled,
    defaultPageSize: RECORDS_PAGE_SIZE,
  });
}

/** Every garden of the filter, page after page, for a choice list (`useLoadRest`). */
export function useAllGardens(filter: Omit<GardensListInput, 'offset' | 'limit' | 'sort'>) {
  return useAllPagesQuery(gardensListQuery(filter));
}

/** What a garden screen shows: the garden, its partners' money, and the history page on screen. */
export interface GardenView {
  /** The page on screen: the garden and each partner's money (always the latest). */
  detail: GardenDetail;
  /** The history rows of this page, in the order the backend sends (oldest day first). */
  history: GardenDetail['history'];
  /** How many history rows the garden has in all. */
  historyCount: number;
}

/** One garden, with its history one page at a time (PRD FR-19.11). Never kept: an entry changes it. */
export function useGarden(id: number | null) {
  const query = usePagedQuery({
    screen: 'gardens.history',
    queryKey: [...GARDENS_KEY, 'one', id],
    enabled: id !== null,
    fetchPage: (offset, limit) =>
      gardensGet({ id: id ?? 0, history_offset: offset, history_limit: limit }),
    rowsIn: (page) => page.history.length,
    totalOf: (page) => page.history_count,
  });
  const first = query.data?.pages[0];
  const view: GardenView | undefined =
    first === undefined
      ? undefined
      : { detail: first, history: first.history, historyCount: first.history_count };
  return { ...query, data: view };
}

/** The data of the dashboard tile باغات (PRD 9.1). */
export function useGardensSummary() {
  return useQuery({ queryKey: [...GARDENS_KEY, 'summary'], queryFn: gardensSummary, staleTime: 0 });
}

/** The preview of an installment, a cost or a sale in another shop: each partner's part (C23). */
export function useGardenEntryPreview(input: GardenEntryInput | null) {
  return usePreviewQuery([...GARDENS_KEY, 'entry-preview'], input, gardensEntryPreview);
}

export function useSaveGardenEntry() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: GardenEntryInput) => gardensEntrySave(input),
    onSuccess: () => readAgainAfterGarden(queryClient),
  });
}

export function useCancelGardenEntry() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: GardenEntryCancelRequest) => gardensEntryCancel(input),
    onSuccess: () => readAgainAfterGarden(queryClient),
  });
}

/** What closing the garden on the day typed would do. A null input asks nothing. */
export function useGardenClosePreview(input: GardenCloseInput | null) {
  return usePreviewQuery([...GARDENS_KEY, 'close-preview'], input, gardensClosePreview);
}

export function useCloseGarden() {
  const queryClient = useQueryClient();
  return useMutation({
    // `confirmOwnerOwes` is the yes of the owner-owes box (PRD FR-12.51); the first try is asked without it.
    mutationFn: ({
      input,
      confirmOwnerOwes,
    }: {
      input: GardenCloseInput;
      confirmOwnerOwes: boolean;
    }) => gardensClose(input, confirmOwnerOwes),
    onSuccess: () => readAgainAfterGarden(queryClient),
  });
}

export function useReopenGarden() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: GardenReopenRequest) => gardensReopen(input),
    onSuccess: () => readAgainAfterGarden(queryClient),
  });
}

// Goods arrived (PRD FR-12.40). A save, a cancel, a new day to sell or a write-off changes the
// garden's stock, so everything a garden changes is read again, and with it the goods ready for a
// new بکری (the queries that start with `bikri`).

/**
 * The list of a garden with many goods arrived entries is in the order the Owner meets them: the newest
 * first (plan 96). It is the list's own order, so the third click on a heading goes back to it.
 */
const NEWEST_FIRST: SortInput = { column: 'entry_date', direction: 'desc', language: null };

interface ArrivalsFilter {
  /** Only arrivals in this state. Null lists all. */
  state?: ArrivalsListInput['state'];
  /** The text of the search box: the number, the item or the other shop. Empty asks for no limit. */
  search?: string;
}

/**
 * The goods arrived entries of one garden, page by page, for the filter (PRD FR-19.11): 30 a page,
 * newest first, narrowed by the search box after a short pause. The first page carries the count and
 * the totals over the whole filter.
 */
export function useArrivalsList(
  gardenId: number,
  { state = null, search = '' }: ArrivalsFilter = {},
) {
  const settled = useDebouncedValue(search.trim(), FILTER_DELAY_MS);
  return usePagedQuery({
    // Each section remembers its own page: the goods not sold yet and the goods sold (FR-12.53).
    screen: state === 'sold_out' ? 'gardens.arrivals_sold' : 'gardens.arrivals',
    queryKey: [...GARDENS_KEY, 'arrivals', gardenId, state, settled],
    fetchPage: (offset, limit, sort) =>
      gardensArrivalsList({
        garden_id: gardenId,
        from: null,
        to: null,
        item_id: null,
        state,
        offset,
        limit,
        show_finished: false,
        search: settled === '' ? null : settled,
        sort: sort ?? NEWEST_FIRST,
        group_by_month: false,
      }),
    rowsIn: (page) => page.rows.length,
    totalOf: (page) => page.total_count,
    keepPrevious: true,
    defaultPageSize: RECORDS_PAGE_SIZE,
  });
}

/** The sales of a garden that the money list asks for (PRD FR-12.53). */
export interface SaleMoneyFilter {
  /** The days, `DD-MM-YYYY`, or null for no limit (every month). */
  from: string | null;
  to: string | null;
  /** Only the sales of the goods of this goods arrived entry, or null for every sale. */
  arrivalId: number | null;
  search?: string;
}

/**
 * Where the garden's sale money is, a page at a time, with the totals over the whole filter. It is
 * kept under the gardens' key, so every save of a garden reads it again; a screen that opens it asks
 * afresh, so a receipt taken since is in "collected since".
 */
export function useSaleMoney(
  gardenId: number,
  { from, to, arrivalId, search = '' }: SaleMoneyFilter,
  screen: string,
  enabled = true,
) {
  const settled = useDebouncedValue(search.trim(), FILTER_DELAY_MS);
  return usePagedQuery({
    screen,
    queryKey: [...GARDENS_KEY, 'sale-money', gardenId, from, to, arrivalId, settled],
    fetchPage: (offset, limit) =>
      gardensSaleMoney({
        garden_id: gardenId,
        from,
        to,
        arrival_id: arrivalId,
        search: settled === '' ? null : settled,
        offset,
        limit,
      }),
    rowsIn: (page) => page.rows.length,
    totalOf: (page) => page.total_count,
    enabled,
    keepPrevious: true,
  });
}

/**
 * Every goods arrived entry of one garden that is still listed, page after page, for the choice
 * list of a sale in another shop (plan 90): the form needs all the lines to offer the right ones,
 * not only the page the list below is on.
 */
export function useAllArrivals(gardenId: number) {
  const query = useAllPagesQuery({
    queryKey: [...GARDENS_KEY, 'arrivals-all', gardenId],
    fetchPage: (offset, limit) =>
      gardensArrivalsList({
        garden_id: gardenId,
        from: null,
        to: null,
        item_id: null,
        state: null,
        offset,
        limit,
        show_finished: false,
        search: null,
        sort: null,
        group_by_month: false,
      }),
    rowsIn: (page) => page.rows.length,
    totalOf: (page) => page.total_count,
  });
  useLoadRest(query, true);
  return query;
}

export function useSaveGardenArrival() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: ArrivalInput) => gardensArrivalSave(input),
    onSuccess: () => readAgainAfterGarden(queryClient),
  });
}

/** The check of the goods arrived entry on the form: the mistakes of each line (PRD FR-12.41). */
export function useArrivalEntryPreview(input: ArrivalEntryInput | null) {
  return usePreviewQuery(
    [...GARDENS_KEY, 'arrival-entry-preview'],
    input,
    gardensArrivalEntryPreview,
  );
}

/** Saves a goods arrived entry with its lines in one transaction (PRD FR-12.41). */
export function useSaveArrivalEntry() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: ArrivalEntryInput) => gardensArrivalEntrySave(input),
    onSuccess: () => readAgainAfterGarden(queryClient),
  });
}

/** Changes the split of an arrival with goods left (PRD FR-12.41). */
export function useChangeArrivalSplit() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: SplitInput) => gardensArrivalChangeSplit(input),
    onSuccess: () => readAgainAfterGarden(queryClient),
  });
}

export function useCancelGardenArrival() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: ArrivalCancelRequest) => gardensArrivalCancel(input),
    onSuccess: () => readAgainAfterGarden(queryClient),
  });
}

export function useSetArrivalSellDate() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: SellDateInput) => gardensArrivalSetSellDate(input),
    onSuccess: () => readAgainAfterGarden(queryClient),
  });
}

export function useWriteOffArrival() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: ArrivalWriteOffInput) => gardensArrivalWriteOff(input),
    onSuccess: () => readAgainAfterGarden(queryClient),
  });
}

export function useWriteOffArrivalUndo() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: ArrivalWriteOffUndoRequest) => gardensArrivalWriteOffUndo(input),
    onSuccess: () => readAgainAfterGarden(queryClient),
  });
}

/** Saves the edit of a garden entry and reads again what it changed (plan 44). */
export function useSaveGardenEntryEdit() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: GardenEntryEditInput) => gardensEntryEditSave(input),
    onSuccess: () => readAgainAfterGarden(queryClient),
  });
}

/** Saves the edit of a garden's goods arrived. */
export function useSaveArrivalEdit() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: GardenArrivalEditInput) => gardensArrivalEditSave(input),
    onSuccess: () => readAgainAfterGarden(queryClient),
  });
}

/** Saves the edit of the header of a garden. */
export function useSaveGardenHeaderEdit() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: GardenHeaderEditInput) => gardensHeaderEditSave(input),
    onSuccess: () => readAgainAfterGarden(queryClient),
  });
}
