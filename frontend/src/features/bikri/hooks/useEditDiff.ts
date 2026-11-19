// What the user has changed in a posting open for edit, as a value that is the same object until
// something typed changes (plan 16). The comparison with the loaded text is made once for each
// version of the draft, so the screen and the grids do not do it again on every redraw.

import { useSyncExternalStore } from 'react';
import type { BikriDraftStores } from '../draft-store';
import { diffEdit, type Baseline, type EditDiff } from '../edit-changes';

const known = new WeakMap<
  BikriDraftStores,
  { version: number; baseline: Baseline; diff: EditDiff }
>();

function diffOf(stores: BikriDraftStores, baseline: Baseline): EditDiff {
  const version = stores.version.getState().value;
  const cached = known.get(stores);
  if (cached?.version === version && cached.baseline === baseline) return cached.diff;
  const diff = diffEdit(stores, baseline);
  known.set(stores, { version, baseline, diff });
  return diff;
}

/** The changes against the loaded text. */
export function useEditDiff(stores: BikriDraftStores, baseline: Baseline): EditDiff;
/** The changes against the loaded text, or null on a screen that is not an edit. */
export function useEditDiff(
  stores: BikriDraftStores,
  baseline: Baseline | undefined,
): EditDiff | null;
export function useEditDiff(
  stores: BikriDraftStores,
  baseline: Baseline | undefined,
): EditDiff | null {
  return useSyncExternalStore(stores.version.subscribe, () =>
    baseline === undefined ? null : diffOf(stores, baseline),
  );
}
