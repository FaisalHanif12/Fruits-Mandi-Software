// The review of an edit while it is typed (plan 44, PRD FR-7.70): the list of changes and their
// effect, asked 50 ms after the last key and worked out by the backend with the same function as
// the save (Code Standards B5). An answer for an older input never replaces the one for a newer
// input, and the last answer stays on screen while the next one is on its way (F35).

import { keepPreviousData, useQuery } from '@tanstack/react-query';
import type { EditPreview } from '../../bindings/EditPreview';
import type { FieldError } from '../../bindings/FieldError';
import { uiErrorOf } from '../api';
import { useDebouncedValue } from '../hooks';
import { PREVIEW_DELAY_MS } from '../hooks/usePreviewQuery';

/**
 * `name` keeps the answers of one screen apart from another's. `input` is what the backend is asked
 * (the entry as typed); null asks nothing yet. `run` asks the backend.
 */
export function useEditReview<Input>(
  name: string,
  input: Input | null,
  run: (input: Input) => Promise<EditPreview>,
) {
  const settled = useDebouncedValue(input, PREVIEW_DELAY_MS);
  const query = useQuery({
    queryKey: ['edit', 'review', name, settled],
    queryFn: () => {
      if (settled === null) throw new Error('no input');
      return run(settled);
    },
    enabled: settled !== null,
    placeholderData: keepPreviousData,
    staleTime: 0,
    gcTime: 0,
  });
  const errors: readonly FieldError[] = query.data?.errors ?? [];
  return {
    review: query.data?.review,
    /** The mistakes the review found, each at its field. Empty when it can be saved. */
    errors,
    /** The answer is on its way, or it is for an older input. */
    pending: query.isFetching || input !== settled,
    /** The whole review was refused (for example, the goods have moved on). */
    error: query.isError ? uiErrorOf(query.error) : null,
  };
}
