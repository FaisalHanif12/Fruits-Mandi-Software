// The live preview of an entry form (PRD FR-7.5, FR-16.8, Code Standards F35, NFR-SPD-7): the form
// asks the backend what saving would do, 50 ms after the last key press, and the answer before it
// is dropped. The backend's answer is the same function as the save (Code Standards B5), so the
// screen never works anything out. While a new answer is on its way, the last one stays on screen
// (no flicker), and `current` says whether what is shown is the answer for what is typed now.

import { keepPreviousData, useQuery } from '@tanstack/react-query';
import { useDebouncedValue } from './useDebouncedValue';

/** The preview waits this long after the last key press (F35). */
export const PREVIEW_DELAY_MS = 50;

/**
 * `input` must keep the same object while the form does not change (a memoised value), so
 * `current` can tell an answer for this input from an answer for an older one. A null input asks
 * nothing (an empty form).
 */
export function usePreviewQuery<Input, Answer>(
  key: readonly unknown[],
  input: Input | null,
  ask: (input: Input) => Promise<Answer>,
) {
  const settled = useDebouncedValue(input, PREVIEW_DELAY_MS);
  const query = useQuery({
    queryKey: [...key, settled],
    queryFn: () => {
      if (settled === null) throw new Error('no entry');
      return ask(settled);
    },
    enabled: settled !== null,
    placeholderData: keepPreviousData,
    staleTime: 0,
    gcTime: 0,
  });
  const current = settled === input && !query.isPlaceholderData && query.isSuccess;
  return { settled, query, current };
}
