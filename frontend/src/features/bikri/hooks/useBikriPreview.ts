// The live preview of the screen (PRD FR-7.5, NFR-SPD-7, Code Standards F35). The draft is built
// from the stores 50 ms after the last key; the call before it is left behind; and an answer for
// an older draft never replaces the one for a newer draft, because each answer is kept under the
// draft it was asked for and the screen reads the one for the draft it has now.
//
// An untouched screen asks nothing. While the next answer is on its way the last one stays on
// screen (`stale` is true), so the numbers never flash empty.

import { keepPreviousData, useQuery } from '@tanstack/react-query';
import { useSyncExternalStore } from 'react';
import { useDebouncedValue } from '../../../shared/hooks';
import { bikriPreview } from '../api';
import type { BikriDraftStores } from '../draft-store';
import { buildDraft, isFilled } from '../draft';
import type { BikriDraft } from '../types';

/** The preview waits this long after the last key press (NFR-SPD-7). */
export const PREVIEW_DELAY_MS = 50;

/** The draft of each screen, built once for each version of what is typed. */
const built = new WeakMap<BikriDraftStores, { version: number; draft: BikriDraft }>();

function draftOf(stores: BikriDraftStores): BikriDraft {
  const version = stores.version.getState().value;
  const known = built.get(stores);
  if (known?.version === version) return known.draft;
  const draft = buildDraft(stores);
  built.set(stores, { version, draft });
  return draft;
}

export function useBikriPreview(stores: BikriDraftStores) {
  const draft = useSyncExternalStore(stores.version.subscribe, () => draftOf(stores));
  const settled = useDebouncedValue(draft, PREVIEW_DELAY_MS);
  const query = useQuery({
    queryKey: ['bikri', 'preview', settled],
    queryFn: () => bikriPreview(settled),
    enabled: isFilled(settled),
    placeholderData: keepPreviousData,
    // The same draft can have another answer after a save elsewhere (a rule, a balance).
    staleTime: 0,
    gcTime: 0,
  });
  return {
    /** The draft as typed now. */
    draft,
    /** The draft the answer is for. */
    settled,
    query,
    /** The answer is for an older draft: it is shown until the new one arrives. */
    stale: query.isPlaceholderData || settled !== draft,
  };
}
