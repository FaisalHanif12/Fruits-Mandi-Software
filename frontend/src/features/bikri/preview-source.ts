// Where a screen asks its preview (PRD FR-12.17, FR-12.18, FR-12.48, Code Standards B5): an edit asks
// `bikri_edit_preview`, a screen with lots carried over from earlier بکری asks `bikri_mixed_preview`
// (plan 76 point 1). Both use the same hook as the sale book (`useBikriPreview`). The answer is put in
// the shape the grids read: its lots are keyed by place to the lot rows of the screen, and a problem on
// `carried[0].sales[2].qty` is read as `lots[0].sales[2].qty`, the path every other screen uses, so the
// problem lands on the same cell and in the same list.
//
// Nothing here changes a number.

import { bikriEditPreview, bikriMixedPreview } from './api';
import type { PreviewSource } from './hooks/useBikriPreview';
import type { BikriDraftStores } from './draft-store';
import type { FieldError } from './types';

/** The preview of a posting being edited (PRD FR-12.18). The lots keep the keys they were loaded with. */
export function editSource(entryId: number): PreviewSource {
  return {
    name: `edit:${String(entryId)}`,
    ready: () => true,
    run: (draft) => bikriEditPreview(entryId, draft),
  };
}

/**
 * The path of a problem of a mixed screen in the form of the whole draft: `carried[i]` is the lot at
 * place i, and `new_lots[k]` the lot at place `carried + k`, because the screen keeps the carried lots
 * first, as the backend lists them (plan 76 point 1). Every other path is left as it is.
 */
export function mixedPath(field: string, carriedCount: number): string {
  const carried = /^carried\[(\d+)\](.*)$/.exec(field);
  if (carried !== null) return `lots[${carried[1] ?? '0'}]${carried[2] ?? ''}`;
  const added = /^new_lots\[(\d+)\](.*)$/.exec(field);
  if (added !== null) return `lots[${String(carriedCount + Number(added[1]))}]${added[2] ?? ''}`;
  return field;
}

/**
 * The preview of the carried lots and the new lots together. The answer for the whole screen has the
 * shapes of `bikri_preview`, so every grid, panel and note reads it as it reads any other. Its lots come
 * keyed by place (carried first, then the new ones), so they are rekeyed to the rows of the screen.
 */
export function mixedSource(stores: BikriDraftStores): PreviewSource {
  return {
    name: 'mixed',
    // A carried lot is known, so even a blank line is worth asking about: the answer holds what is left.
    ready: (draft) => (draft.carried ?? []).length > 0,
    run: async (draft) => {
      const carriedCount = (draft.carried ?? []).length;
      const answer = await bikriMixedPreview({
        date: draft.header.entry_date,
        carried: draft.carried ?? [],
        new_lots: draft.lots,
      });
      const keys = stores.lots.getState().rows.map((row) => row.rowId);
      const place = (items: readonly FieldError[]) =>
        items.map((item) => ({ ...item, field: mixedPath(item.field, carriedCount) }));
      const whole = answer.preview;
      return {
        ...whole,
        lots: whole.lots.map((lot, index) => ({ ...lot, key: keys[index] ?? lot.key })),
        errors: place(whole.errors),
        warnings: place(whole.warnings),
      };
    },
  };
}
