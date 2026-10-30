// The one-time people and the register as hooks (Code Standards F6). Nothing is kept for long: a
// save of a بکری invalidates the `['pukhta']` queries (useSaveBikri).

import { keepPreviousData, useQuery } from '@tanstack/react-query';
import type { PukhtaRegisterInput } from '../../bindings/PukhtaRegisterInput';
import type { PukhtaSide } from '../../bindings/PukhtaSide';
import { useDebouncedValue, usePagedQuery } from '../../shared/hooks';
import { pukhtaRegister, searchPukhtaPeople } from './api';

/** A short pause so a burst of key presses asks once. */
const PEOPLE_DELAY_MS = 150;

/**
 * The one-time people that fit the text typed in a name or phone field (PRD FR-7.29). Nothing is
 * asked while the field is empty. The last answer stays while the next one is on its way.
 */
export function usePukhtaPeople(text: string, side: PukhtaSide) {
  const settled = useDebouncedValue(text.trim(), PEOPLE_DELAY_MS);
  return useQuery({
    queryKey: ['pukhta', 'people', side, settled],
    queryFn: () => searchPukhtaPeople(settled, side),
    enabled: settled !== '',
    placeholderData: keepPreviousData,
    staleTime: 0,
    gcTime: 0,
  });
}

const FILTER_DELAY_MS = 150;

/**
 * The register for the filters, page by page, with the count and the totals of the whole filter
 * (PRD FR-13.6, FR-19.11).
 */
export function usePukhtaRegister(
  filter: Omit<PukhtaRegisterInput, 'offset' | 'limit' | 'sort'>,
  enabled = true,
) {
  const settled = useDebouncedValue(filter, FILTER_DELAY_MS);
  return usePagedQuery({
    screen: 'pukhta.register',
    queryKey: ['pukhta', 'register', settled],
    enabled,
    fetchPage: (offset, limit, sort) =>
      pukhtaRegister({
        ...settled,
        offset,
        limit,
        sort,
        // Month rows only while the list is in date order (BikriListInput.group_by_month rule).
        group_by_month: settled.group_by_month && (sort === null || sort.column === 'entry_date'),
      }),
    rowsIn: (page) => page.rows.length,
    totalOf: (page) => page.total_count,
    keepPrevious: true,
  });
}
