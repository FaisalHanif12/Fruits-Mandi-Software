// The reads of the بکری screens as hooks (Code Standards F6): one saved بکری, the lists of پچھلی بکری (today
// and the window of earlier days), and the answer of "find". Nothing is kept for long: a save, an edit or a cancel changes
// them all, and a save invalidates the `['bikri']` queries (useSaveBikri).

import { keepPreviousData, useQuery } from '@tanstack/react-query';
import { useDebouncedValue, usePagedQuery } from '../../../shared/hooks';
import { bikriCancelPreview, bikriFind, bikriGet, bikriList, bikriStockList } from '../api';
import type { BikriFindInput } from '../../../bindings/BikriFindInput';
import type { StockListInput } from '../../../bindings/StockListInput';
import type { BikriListInput } from '../../../bindings/BikriListInput';

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

/** The filter of a list as the screen builds it, without the page and the order. */
type ListFilter = Omit<BikriListInput, 'offset' | 'limit' | 'sort'>;

/**
 * The filter a list asks with. A burst of key presses in a filter asks once (the pause of
 * `FILTER_DELAY_MS`); but before the list knows any day (the backend's today is on its way) there is
 * nothing to wait for and nothing to ask: the settled filter then holds no day, and asking with it
 * would be a call with no day at all (every بکری of every day, or a window the backend refuses). So a
 * list is asked only with a filter that has its days, and the first one is taken at once, with no
 * pause (plan 72: found on the development page, where such a call hung the stand-in).
 */
function askedFilter(
  live: ListFilter,
  settled: ListFilter,
  hasDays: (filter: ListFilter) => boolean,
) {
  const asked = hasDays(settled) ? settled : live;
  return { asked, enabled: hasDays(asked) };
}

const hasDay = (filter: ListFilter) => filter.date !== null;
const hasWindow = (filter: ListFilter) => filter.from !== null && filter.to !== null;

/**
 * Today's section of پچھلی بکری: the list of بکری of one day, page by page, with the count and the totals
 * of the whole filter (PRD 12.7, FR-19.11). The last answer stays while the next comes.
 */
export function useBikriList(filter: ListFilter) {
  const settled = useDebouncedValue(filter, FILTER_DELAY_MS);
  const { asked, enabled } = askedFilter(filter, settled, hasDay);
  return usePagedQuery({
    screen: 'bikri.today',
    queryKey: ['bikri', 'list', asked],
    enabled,
    fetchPage: (offset, limit, sort) => bikriList({ ...asked, offset, limit, sort }),
    rowsIn: (page) => page.rows.length,
    totalOf: (page) => page.total_count,
    keepPrevious: true,
  });
}

/**
 * The window of earlier days of پچھلی بکری (PRD 12.7, FR-12.46 v1.39, plan 75 7.7): one entry for each day with
 * its count and خام بکری, and the totals of the whole filter. It asks for a page of one row: the rows of a
 * day come from the day itself, only when the Owner opens it (`useBikriList` with that day). A save, an
 * edit or a cancel invalidates the `['bikri']` queries, so a way back from an edit asks again (P10).
 */
export function useBikriWindow(filter: ListFilter) {
  const settled = useDebouncedValue(filter, FILTER_DELAY_MS);
  const { asked, enabled } = askedFilter(filter, settled, hasWindow);
  return useQuery({
    queryKey: ['bikri', 'list', 'window', asked],
    queryFn: () => bikriList({ ...asked, offset: 0, limit: 1, sort: null }),
    enabled,
    placeholderData: keepPreviousData,
    staleTime: 0,
  });
}

/** True when a find has at least one filter, so an empty find asks nothing. */
export function hasFindFilter(input: BikriFindInput): boolean {
  return (
    input.number !== null ||
    (input.truck_no ?? '').trim() !== '' ||
    input.seller_id !== null ||
    (input.date ?? '').trim() !== '' ||
    input.item_id !== null ||
    input.buyer_account_id !== null ||
    (input.buyer_text ?? '').trim() !== ''
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
export function useStockList(
  filter: Omit<StockListInput, 'offset' | 'limit' | 'sort' | 'search'> & {
    search?: string | null;
  },
  enabled = true,
) {
  const settled = useDebouncedValue(filter, FILTER_DELAY_MS);
  return usePagedQuery({
    screen: 'bikri.stock',
    queryKey: ['bikri', 'stock', settled],
    enabled,
    fetchPage: (offset, limit, sort) =>
      bikriStockList({ ...settled, search: settled.search ?? null, offset, limit, sort }),
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
