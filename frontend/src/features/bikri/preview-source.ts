// Where a screen of plan 16 asks its preview (PRD FR-12.17, FR-12.18, Code Standards B5): a later
// sale asks `bikri_later_sale_preview`, an edit asks `bikri_edit_preview`. Both use the same hook
// as the sale book (`useBikriPreview`). The answer is put in the shape the grids read:
//
// - A later sale has one lot, so its one `LotPreview` is matched to the one lot row of the screen
//   by place, whatever key the backend gave it.
// - A problem of a later sale on `sales[2].qty` is read as `lots[0].sales[2].qty`, the path every
//   other screen uses, so the problem lands on the same cell and in the same list.
//
// Nothing here changes a number.

import { bikriEditPreview, bikriLaterSalePreview } from './api';
import type { PreviewSource } from './hooks/useBikriPreview';
import type { BikriPreview, FieldError } from './types';

/** The path of a problem of a later sale, in the form of the whole draft. */
function lotZeroPath(field: string): string {
  return field.startsWith('sales[') ? `lots[0].${field}` : field;
}

function withPaths(items: readonly FieldError[]): FieldError[] {
  return items.map((item) => ({ ...item, field: lotZeroPath(item.field) }));
}

/** The answer with its lots under the keys of the lot rows of the screen (by place). */
export function rekeyLots(preview: BikriPreview, lotKeys: readonly string[]): BikriPreview {
  return {
    ...preview,
    lots: preview.lots.map((lot, index) => ({ ...lot, key: lotKeys[index] ?? lot.key })),
    errors: withPaths(preview.errors),
    warnings: withPaths(preview.warnings),
  };
}

/** The preview of new sale lines of one lot (PRD FR-12.17). */
export function laterSaleSource(lotId: number, lotKey: string): PreviewSource {
  return {
    name: `later:${String(lotId)}`,
    // The lot is known, so even a blank line is worth asking about: the answer holds what is left.
    ready: () => true,
    run: async (draft) =>
      rekeyLots(
        await bikriLaterSalePreview(lotId, draft.header.entry_date, draft.lots[0]?.sales ?? []),
        [lotKey],
      ),
  };
}

/** The preview of a posting being edited (PRD FR-12.18). The lots keep the keys they were loaded with. */
export function editSource(entryId: number): PreviewSource {
  return {
    name: `edit:${String(entryId)}`,
    ready: () => true,
    run: (draft) => bikriEditPreview(entryId, draft),
  };
}
