// The real answers of the backend for a line that sells more than its lot has left (plan 57). The
// backend wrote them in its own test (backend/crates/services/tests/fixtures/
// bikri_preview_over_qty*.json). They are copied here, never edited, and read as the generated
// types. Money in them is paisa, as on the wire. The lot has 50 نگ. The three cases:
//   - OVER: one line of 70 at 1,200. The line has no `calc`, the lot has no totals, and the error
//     points at the quantity with 50 available.
//   - FIXED: the same line with 50. Every figure is back: 60,000, 3,600, 3,600, 400 and 64,000.
//   - TWO_LINES: two lines of 30 (600 and 580). Line 1 stands, line 2 is refused with 20 available.

import type { BikriPreview } from '../../src/bindings/BikriPreview';
import type { BikriPreviewInput } from '../../src/bindings/BikriPreviewInput';
import over from '../fixtures/bikri_preview_over_qty.json';
import overFixed from '../fixtures/bikri_preview_over_qty_fixed.json';
import overFixedRequest from '../fixtures/bikri_preview_over_qty_fixed_request.json';
import overRequest from '../fixtures/bikri_preview_over_qty_request.json';
import overTwoLines from '../fixtures/bikri_preview_over_qty_two_lines.json';
import overTwoLinesRequest from '../fixtures/bikri_preview_over_qty_two_lines_request.json';

// A fixture is the backend's own JSON; a reader takes `unknown`, because the money in a binding is a
// branded type that a plain number is not.
function previewOf(value: unknown): BikriPreview {
  return value as BikriPreview;
}
function requestOf(value: unknown): BikriPreviewInput {
  return value as BikriPreviewInput;
}

export const OVER_PREVIEW: BikriPreview = previewOf(over);
export const OVER_REQUEST: BikriPreviewInput = requestOf(overRequest);
export const OVER_FIXED_PREVIEW: BikriPreview = previewOf(overFixed);
export const OVER_FIXED_REQUEST: BikriPreviewInput = requestOf(overFixedRequest);
export const OVER_TWO_LINES_PREVIEW: BikriPreview = previewOf(overTwoLines);
export const OVER_TWO_LINES_REQUEST: BikriPreviewInput = requestOf(overTwoLinesRequest);

/**
 * A real answer given the keys the screen sent, so a screen whose rows have other keys than the
 * backend's test (`lot-1`, `line-1`) can use it.
 */
export function withKeysOf(
  real: BikriPreview,
  draft: { lots: { key: string; sales: { key: string }[] }[] },
): BikriPreview {
  const preview = structuredClone(real);
  preview.lots.forEach((lot, lotIndex) => {
    lot.key = draft.lots[lotIndex]?.key ?? lot.key;
    lot.lines.forEach((line, lineIndex) => {
      line.key = draft.lots[lotIndex]?.sales[lineIndex]?.key ?? line.key;
    });
  });
  return preview;
}
