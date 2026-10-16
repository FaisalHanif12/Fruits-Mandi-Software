// The one-time people and the register as hooks (Code Standards F6). Nothing is kept for long: a
// save of a بکری invalidates the `['pukhta']` queries (useSaveBikri).

import { keepPreviousData, useQuery } from '@tanstack/react-query';
import type { PukhtaRegisterInput } from '../../bindings/PukhtaRegisterInput';
import type { PukhtaSide } from '../../bindings/PukhtaSide';
import { useDebouncedValue } from '../../shared/hooks';
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

/** The register for the filters (PRD FR-13.6). */
export function usePukhtaRegister(filter: PukhtaRegisterInput) {
  const settled = useDebouncedValue(filter, FILTER_DELAY_MS);
  return useQuery({
    queryKey: ['pukhta', 'register', settled],
    queryFn: () => pukhtaRegister(settled),
    placeholderData: keepPreviousData,
    staleTime: 0,
    gcTime: 0,
  });
}
