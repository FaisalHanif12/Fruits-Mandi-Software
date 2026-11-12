// The way back from the sale book to پچھلی بکری (PRD 12.7, plan 72 P10). A بکری opened from that list
// carries the window it came from in the address (`past`, `pastAll`). The sale book keeps it on every
// address it moves to, and after an edit is saved it goes back to the same window and row.

import { useSearch } from '@tanstack/react-router';
import { useMemo } from 'react';
import { parsePastReturn, type PastReturn } from '../past-window';

/** The window the Owner came from, or undefined when he did not come from پچھلی بکری. */
export function usePastReturn(): PastReturn | undefined {
  const raw: Record<string, unknown> = useSearch({ strict: false });
  const past = raw['past'];
  const pastAll = raw['pastAll'];
  return useMemo(() => parsePastReturn({ past, pastAll }), [past, pastAll]);
}

/** The parts of a sale book address that keep the way back. Nothing when there is none. */
export function keepPastReturn(back: PastReturn | undefined): Partial<PastReturn> {
  return back === undefined
    ? {}
    : { past: back.past, ...(back.pastAll === true ? { pastAll: true as const } : {}) };
}
