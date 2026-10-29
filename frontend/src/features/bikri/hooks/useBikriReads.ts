// The reads of the بکری screens as hooks (Code Standards F6): one saved بکری, the list of today's
// بکری, and the answer of "find". Nothing is kept for long: a save, an edit or a cancel changes
// them all, and a save invalidates the `['bikri']` queries (useSaveBikri).

import { keepPreviousData, useQuery } from '@tanstack/react-query';
import { useDebouncedValue, usePagedQuery } from '../../../shared/hooks';
import { bikriCancelPreview, bikriFind, bikriGet, bikriList, bikriStockList } from '../api';
import type { BikriFindInput } from '../../../bindings/BikriFindInput';
import type { BikriListInput } from '../../../bindings/BikriListInput';
import type { StockListInput } from '../../../bindings/StockListInput';

/** A pause so a burst of key presses in a filter asks once. */
const FILTER_DELAY_MS = 150;

/** A saved بکری with the values stored when it was saved (PRD FR-12.13). */
export function useBikriDetail(id: number | null) {
  return useQuery({
    queryKey: ['bikri', 'detail', id],
    queryFn: () => bikriGet(id ?? 0),
    enabled: id !== null,
    staleTime: 0,
  });
}

/**
 * The list of بکری for the filters, page by page, with the count and the totals of the whole filter
 * (PRD FR-12.7, FR-19.11). The last answer stays while the next comes.
 */
export function useBikriList(filter: Omit<BikriListInput, 'offset' | 'limit' | 'sort'>) {
  const settled = useDebouncedValue(filter, FILTER_DELAY_MS);
  return usePagedQuery({
    screen: 'bikri.today',
    queryKey: ['bikri', 'list', settled],
    fetchPage: (offset, limit, sort) => bikriList({ ...settled, offset, limit, sort }),
    rowsIn: (page) => page.rows.length,
    totalOf: (page) => page.total_count,
    keepPrevious: true,
  });
}

/** True when a find has at least one filter, so an empty find asks nothing. */
export function hasFindFilter(input: BikriFindInput): boolean {
  return (
    input.number !== null ||
    (input.truck_no ?? '').trim() !== '' ||
    input.seller_id !== null ||
    (input.date ?? '').trim() !== '' ||
    input.item_id !== null
  );
}

/** The result of "find" (PRD FR-12.13), newest first, at most 50 rows. */
export function useBikriFind(input: BikriFindInput, enabled: boolean) {
  const settled = useDebouncedValue(input, FILTER_DELAY_MS);
  return useQuery({
    queryKey: ['bikri', 'find', settled],
    queryFn: () => bikriFind(settled),
    enabled: enabled && hasFindFilter(settled),
    placeholderData: keepPreviousData,
    staleTime: 0,
    gcTime: 0,
  });
}

/**
 * The lots with goods left for the filters, page by page (PRD FR-12.14, FR-19.11): the lots first,
 * then the firm's own goods, then the gardens' goods, counted together. The first page has the totals of the whole filter.
 * The last answer stays while the next comes.
 */
export function useStockList(filter: Omit<StockListInput, 'offset' | 'limit'>, enabled = true) {
  const settled = useDebouncedValue(filter, FILTER_DELAY_MS);
  return usePagedQuery({
    screen: 'bikri.stock',
    queryKey: ['bikri', 'stock', settled],
    enabled,
    fetchPage: (offset, limit) => bikriStockList({ ...settled, offset, limit }),
    rowsIn: (page) => page.rows.length + page.own_rows.length + page.garden_rows.length,
    totalOf: (page) => page.total_count,
    keepPrevious: true,
  });
}

/** What a cancel would do, for the confirm box (PRD FR-7.8). Asked when the box opens. */
export function useCancelPreview(id: number | null) {
  return useQuery({
    queryKey: ['bikri', 'cancel-preview', id],
    queryFn: () => bikriCancelPreview(id ?? 0),
    enabled: id !== null,
    staleTime: 0,
    gcTime: 0,
  });
}
