// The پھل commands as hooks (Code Standards F6). A saved, edited or cancelled entry refreshes
// everything that shows what it wrote (PRD 6.2): the fruit list, the lots in باقی مال (the
// balance went down), the sellers' balances and party cards, the partners' totals and statements,
// a garden's history, the search recents and the day.

import { useMutation, useQuery, useQueryClient, type QueryClient } from '@tanstack/react-query';
import { useDebouncedValue, usePagedQuery, usePreviewQuery } from '../../shared/hooks';
import { fruitCancel, fruitEdit, fruitGet, fruitList, fruitPreview, fruitSave } from './api';
import type { FruitCancelInput, FruitInput, FruitListInput, FruitSaveRequest } from './types';

export const FRUIT_KEY = ['fruit'] as const;
/** A pause so a burst of key presses in a filter asks once. */
const FILTER_DELAY_MS = 150;

/** Everything a fruit posting changes, to read again (PRD 6.2). */
const READ_AGAIN: readonly (readonly string[])[] = [
  FRUIT_KEY,
  ['bikri'],
  ['accounts'],
  ['coa'],
  ['partners'],
  ['gardens'],
  ['search'],
  ['day_status'],
];

/** Reads again what a fruit entry, an edit or a cancel changed. */
export function readAgainAfterFruit(queryClient: QueryClient): Promise<unknown> {
  return Promise.all(
    READ_AGAIN.map((queryKey) => queryClient.invalidateQueries({ queryKey: [...queryKey] })),
  );
}

/** What saving would do, live under the form. A null input asks nothing (PRD FR-16.10). */
export function useFruitPreview(input: FruitInput | null) {
  return usePreviewQuery([...FRUIT_KEY, 'preview'], input, fruitPreview);
}

/** What a save gives back that the screen needs: the number of the entry. */
export interface FruitSavedNumber {
  number: number;
}

/** Saves a new entry, or changes a saved one when the request carries its id (PRD FR-16.12). */
export function useSaveFruit() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (request: FruitSaveRequest): Promise<FruitSavedNumber> =>
      request.edit === null
        ? fruitSave(request.entry)
        : fruitEdit({ ...request.edit, entry: request.entry }),
    onSuccess: () => readAgainAfterFruit(queryClient),
  });
}

export function useCancelFruit() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: FruitCancelInput) => fruitCancel(input),
    onSuccess: () => readAgainAfterFruit(queryClient),
  });
}

/** Reads one entry fresh, for the edit form: its version must be the one in the database now. */
export function useReadFruit() {
  return useMutation({ mutationFn: (id: number) => fruitGet({ id }) });
}

/**
 * The entries for the filters, page by page, with the totals over the whole filter (PRD FR-19.11).
 * `fetchNextPage` loads the next 100 rows; the first page carries the count and the totals.
 */
export function useFruitList(filter: Omit<FruitListInput, 'offset' | 'limit' | 'sort'>) {
  const settled = useDebouncedValue(filter, FILTER_DELAY_MS);
  return usePagedQuery({
    screen: 'fruit.list',
    queryKey: [...FRUIT_KEY, 'list', settled],
    fetchPage: (offset, limit, sort) => fruitList({ ...settled, offset, limit, sort }),
    rowsIn: (page) => page.rows.length,
    totalOf: (page) => page.total_count,
    keepPrevious: true,
  });
}

/** One saved entry, for the saved view that its address opens (PRD FR-19.4). Never kept: an edit changes it. */
export function useFruitEntry(id: number | null) {
  return useQuery({
    queryKey: [...FRUIT_KEY, 'one', id],
    queryFn: () => fruitGet({ id: id ?? 0 }),
    enabled: id !== null,
    staleTime: 0,
  });
}
