// The باغات commands as hooks (Code Standards F6). A saved garden, entry, closing or cancel
// refreshes everything that shows what it wrote (PRD 6.2): the gardens, the sale book (a garden
// is offered as a lot seller), the party cards, the cash, the day status and the partners' totals,
// so every screen shows the new numbers.

import {
  useInfiniteQuery,
  useMutation,
  useQuery,
  useQueryClient,
  type QueryClient,
} from '@tanstack/react-query';
import { PAGE_SIZE, useDebouncedValue, usePagedQuery, usePreviewQuery } from '../../shared/hooks';
import {
  gardensArrivalCancel,
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
  gardensEntryPreview,
  gardensEntrySave,
  gardensGet,
  gardensList,
  gardensReopen,
  gardensSummary,
} from './api';
import type {
  ArrivalCancelInput,
  ArrivalInput,
  ArrivalsListInput,
  ArrivalWriteOffInput,
  ArrivalWriteOffUndoInput,
  SellDateInput,
  GardenCloseInput,
  GardenDetail,
  GardenEntryCancelInput,
  GardenEntryInput,
  GardenInput,
  GardenReopenInput,
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

/**
 * The gardens for the filters, page by page, with the totals over the whole filter (PRD FR-19.11).
 * `fetchNextPage` loads the next 100 rows; the first page carries the count and the totals.
 */
export function useGardensList(filter: Omit<GardensListInput, 'offset' | 'limit'>) {
  const settled = useDebouncedValue(filter, FILTER_DELAY_MS);
  return usePagedQuery({
    queryKey: [...GARDENS_KEY, 'list', settled],
    fetchPage: (offset, limit) => gardensList({ ...settled, offset, limit }),
    rowsIn: (page) => page.rows.length,
    totalOf: (page) => page.total_count,
    keepPrevious: true,
  });
}

/** What a garden screen shows: the garden, its partners' money, and all loaded history rows. */
export interface GardenView {
  /** The first page: the garden and each partner's money (always the latest). */
  detail: GardenDetail;
  /** Every history row loaded so far, in the order the backend sends (oldest day first). */
  history: GardenDetail['history'];
  /** How many history rows the garden has in all. */
  historyCount: number;
}

/** One garden, with its history loaded page by page (PRD FR-19.11). Never kept: an entry changes it. */
export function useGarden(id: number | null) {
  return useInfiniteQuery({
    queryKey: [...GARDENS_KEY, 'one', id],
    initialPageParam: 0,
    enabled: id !== null,
    queryFn: ({ pageParam }) =>
      gardensGet({ id: id ?? 0, history_offset: pageParam, history_limit: PAGE_SIZE }),
    getNextPageParam: (last, pages) => {
      const loaded = pages.reduce((sum, page) => sum + page.history.length, 0);
      return loaded < last.history_count && last.history.length > 0 ? loaded : undefined;
    },
    select: (data): GardenView | undefined => {
      const first = data.pages[0];
      if (first === undefined) return undefined;
      return {
        detail: first,
        history: data.pages.flatMap((page) => page.history),
        historyCount: first.history_count,
      };
    },
    staleTime: 0,
  });
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
    mutationFn: (input: GardenEntryCancelInput) => gardensEntryCancel(input),
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
    mutationFn: (input: GardenCloseInput) => gardensClose(input),
    onSuccess: () => readAgainAfterGarden(queryClient),
  });
}

export function useReopenGarden() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: GardenReopenInput) => gardensReopen(input),
    onSuccess: () => readAgainAfterGarden(queryClient),
  });
}

// Goods arrived (PRD FR-12.40). A save, a cancel, a new day to sell or a write-off changes the
// garden's stock, so everything a garden changes is read again, and with it the goods ready for a
// new بکری (the queries that start with `bikri`).

/** The goods arrived entries of one garden, page by page, for the filter (PRD FR-19.11). */
export function useArrivalsList(gardenId: number, state: ArrivalsListInput['state'] = null) {
  return usePagedQuery({
    queryKey: [...GARDENS_KEY, 'arrivals', gardenId, state],
    fetchPage: (offset, limit) =>
      gardensArrivalsList({
        garden_id: gardenId,
        from: null,
        to: null,
        item_id: null,
        state,
        offset,
        limit,
      }),
    rowsIn: (page) => page.rows.length,
    totalOf: (page) => page.total_count,
    keepPrevious: true,
  });
}

export function useSaveGardenArrival() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: ArrivalInput) => gardensArrivalSave(input),
    onSuccess: () => readAgainAfterGarden(queryClient),
  });
}

export function useCancelGardenArrival() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: ArrivalCancelInput) => gardensArrivalCancel(input),
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
    mutationFn: (input: ArrivalWriteOffUndoInput) => gardensArrivalWriteOffUndo(input),
    onSuccess: () => readAgainAfterGarden(queryClient),
  });
}
