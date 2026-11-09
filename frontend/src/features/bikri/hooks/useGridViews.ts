// What the two grids show from the preview, worked out again only when something it depends on changed
// (D8 plan 61, NFR-SCL-3, NFR-SPD-5). With 1,000 sale lines, building the text of every cell is real
// work, and the page draws again on every move of the cursor. Here its inputs are arguments, so the
// answer is the same object until the preview, the problems, the lines or the language change.

import { useMemo } from 'react';
import type { EntryRow } from '../../../shared/grid';
import type { Grouping } from '../../../shared/format';
import type { BikriDraftStores } from '../draft-store';
import { buildGridViews } from '../grid-views';
import type { Problem } from '../problems';
import type { BikriPreview } from '../types';

interface GridViewsInput {
  stores: BikriDraftStores;
  preview: BikriPreview | undefined;
  problems: readonly Problem[];
  /** The lots, and the lines of every lot one after the other in the order of the lots. */
  lotRows: readonly EntryRow[];
  allRows: readonly EntryRow[];
  /** The selected lot, and whether the whole sale is shown. */
  lotKey: string;
  whole: boolean;
  grouping: Grouping;
  say: (id: string, vars?: Record<string, string>) => string;
  /** The name of a lot's seller and item, for the lot column of the whole-sale view. */
  lotWho: (row: EntryRow | undefined) => string;
  errorText: (item: Problem['item']) => string;
}

export function useGridViews(input: GridViewsInput) {
  const { stores, preview, problems, lotRows, allRows, lotKey, whole, grouping } = input;
  const { say, lotWho, errorText } = input;
  // The keys of the lines of each lot. `allRows` is a new list whenever a line of any lot changed.
  const lineKeysByLot = useMemo(() => {
    const counts = lotRows.map((row) => stores.salesFor(row.rowId).getState().rows.length);
    return counts.map((count, index) => {
      const start = counts.slice(0, index).reduce((sum, rows) => sum + rows, 0);
      return allRows.slice(start, start + count).map((line) => line.rowId);
    });
  }, [allRows, lotRows, stores]);
  const views = useMemo(
    () =>
      buildGridViews({
        preview,
        problems,
        lotKeys: lotRows.map((row) => row.rowId),
        selectedLot: lotKey,
        lineKeysByLot,
        showAll: whole,
        lotLabel: (index) => {
          const who = lotWho(lotRows[index]);
          return who === ''
            ? say('bikri-all-lot-no-seller', { lot: String(index + 1) })
            : say('bikri-all-lot', { lot: String(index + 1), who });
        },
        grouping,
        say,
        words: (problem) => errorText(problem.item),
      }),
    [preview, problems, lotRows, lotKey, lineKeysByLot, whole, grouping, say, lotWho, errorText],
  );
  return { views, lineKeysByLot };
}
