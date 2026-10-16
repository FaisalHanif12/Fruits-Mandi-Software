// What the app remembers about the truck number typed (PRD FR-7.49, FR-7.50): the list of known
// numbers, the same-truck-today notice, and the last بکری of the truck. Asked a moment after the
// last key, only when a number is typed. Read only.

import { keepPreviousData, useQuery } from '@tanstack/react-query';
import { useDebouncedValue } from '../../../shared/hooks';
import { assistTruckMemory } from '../api';

/** A short pause so a burst of key presses asks once. */
const TRUCK_DELAY_MS = 150;

export function useTruckMemory(truckNo: string, date: string) {
  const text = truckNo.trim();
  const settled = useDebouncedValue(text, TRUCK_DELAY_MS);
  return useQuery({
    queryKey: ['bikri', 'truck', settled, date],
    queryFn: () => assistTruckMemory(settled, date),
    enabled: settled !== '',
    placeholderData: keepPreviousData,
    staleTime: 0,
    gcTime: 0,
  });
}
