// Searching for a picker as hooks (Code Standards F6). The answer to an older text is dropped
// when a newer one has been asked, so a slow answer never shows under the wrong text, and the
// list keeps the last answer while the next one is on its way, so it never flashes empty.

import { keepPreviousData, useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useEffect, useState } from 'react';
import { searchMasters, searchRecent, searchTouch } from './api';
import type { SearchHit, SearchScope } from './types';

/** How many matches a typed search asks for. The list is short on purpose: type more to narrow. */
export const PICKER_RESULT_LIMIT = 50;
/** A short pause so a burst of key presses asks once (NFR-SPD-6 allows 50 ms in all). */
export const PICKER_SEARCH_DELAY_MS = 30;

const SEARCH_KEY = ['search'] as const;

/** `value`, but only after it has stayed the same for `delayMs`. */
function useSettled(value: string, delayMs: number): string {
  const [settled, setSettled] = useState(value);
  useEffect(() => {
    const timer = window.setTimeout(() => {
      setSettled(value);
    }, delayMs);
    return () => {
      window.clearTimeout(timer);
    };
  }, [value, delayMs]);
  return settled;
}

/** What a picker needs to find records: how to search, what is recent, and how to remember a pick. */
export interface PickerSource<H> {
  /** Names the source in the query cache, for example `['search', 'city']`. */
  readonly key: readonly string[];
  /** Records whose code or name matches `text`. */
  search: (text: string, limit: number) => Promise<H[]>;
  /** The last 10 records used, then the rest (PRD FR-7.41). */
  recent: () => Promise<H[]>;
  /** Remembers that a record was chosen, so it comes first next time. */
  touch: (id: number) => Promise<unknown>;
}

/** The source of a master list: city, item, packing and the rest (plan 06). */
export function masterSource(scope: SearchScope): PickerSource<SearchHit> {
  return {
    key: [...SEARCH_KEY, scope],
    search: (text, limit) => searchMasters({ scope, text, limit }),
    recent: () => searchRecent({ scope }),
    touch: (id) => searchTouch({ scope, id }),
  };
}

/**
 * The records a picker shows. Typed text searches; empty text (the user pressed Alt+↓, or
 * emptied the field) gives the recent records first, then the rest (PRD FR-7.41). Nothing is
 * asked while the list is closed.
 */
export function usePickerHits<H>(source: PickerSource<H>, text: string, open: boolean) {
  const typed = useSettled(text.trim(), PICKER_SEARCH_DELAY_MS);
  // Emptying the field must not wait: the recent list is the answer at once.
  const query = text.trim() === '' ? '' : typed;
  return useQuery({
    queryKey: [...source.key, query],
    queryFn: () => (query === '' ? source.recent() : source.search(query, PICKER_RESULT_LIMIT)),
    enabled: open,
    placeholderData: keepPreviousData,
    // The same text can give a different answer after a save, so an answer is never reused.
    staleTime: 0,
    gcTime: 0,
  });
}

/** Tells the backend a record was chosen, so it comes first next time (PRD FR-7.41). */
export function useTouchRecord<H>(source: PickerSource<H>) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: number) => source.touch(id),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: [...source.key] }),
  });
}
