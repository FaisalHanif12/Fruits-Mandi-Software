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
import { buildDraft, isFilledByUser } from '../draft';
import type { BikriDraft, BikriPreview } from '../types';

/** The preview waits this long after the last key press (NFR-SPD-7). */
export const PREVIEW_DELAY_MS = 50;

/** The draft of each screen, built once for each version of what is typed. */
const built = new WeakMap<BikriDraftStores, { version: number; draft: BikriDraft }>();

/** The version of what is typed that each draft was built from, and a number for each screen. */
const versions = new WeakMap<BikriDraft, number>();
const screens = new WeakMap<BikriDraftStores, number>();
let screenCount = 0;

export function draftOf(stores: BikriDraftStores): BikriDraft {
  const version = stores.version.getState().value;
  const known = built.get(stores);
  if (known?.version === version) return known.draft;
  const draft = buildDraft(stores);
  built.set(stores, { version, draft });
  versions.set(draft, version);
  return draft;
}

/**
 * What names an answer of the preview: the screen and the version of what was typed, never the draft
 * itself. A draft of 1,000 lines is 200 KB, and hashing it for the query key on every render cost more
 * than the rest of the key press (D8 plan 61, NFR-SCL-3). The version goes up only when a row or the
 * header changes, not when the cursor moves, so the same typing is the same key.
 */
function answerKey(stores: BikriDraftStores, draft: BikriDraft): string {
  let screen = screens.get(stores);
  if (screen === undefined) {
    screenCount += 1;
    screen = screenCount;
    screens.set(stores, screen);
  }
  return `${String(screen)}:${String(versions.get(draft) ?? -1)}`;
}

/**
 * Where a screen asks its preview. The sale book asks `bikri_preview`; a later sale and an edit
 * ask their own commands (plan 16) but use the same hook, so the 50 ms wait, the stale answer and
 * the "older answer never replaces a newer one" rule are the same everywhere (Code Standards F35).
 */
export interface PreviewSource {
  /** Part of the query key, so the answers of one screen are never another screen's. */
  name: string;
  /** Asks the backend for the draft as it is. */
  run: (draft: BikriDraft) => Promise<BikriPreview>;
  /** Can the preview be asked yet? Default: the draft has something typed. */
  ready?: (draft: BikriDraft) => boolean;
}

export function useBikriPreview(stores: BikriDraftStores, source?: PreviewSource) {
  const draft = useSyncExternalStore(stores.version.subscribe, () => draftOf(stores));
  const settled = useDebouncedValue(draft, PREVIEW_DELAY_MS);
  // Lots that came in by themselves and were not worked on ask nothing (plan 33, R3).
  const ready = source?.ready ?? ((candidate: BikriDraft) => isFilledByUser(stores, candidate));
  const run = source?.run ?? bikriPreview;
  const query = useQuery({
    queryKey:
      source === undefined
        ? ['bikri', 'preview', answerKey(stores, settled)]
        : ['bikri', 'preview', source.name, answerKey(stores, settled)],
    queryFn: () => run(settled),
    enabled: ready(settled),
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
