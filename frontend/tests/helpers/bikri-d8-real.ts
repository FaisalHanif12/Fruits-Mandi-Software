// The backend's own answers for Umar's three lines (D8 Bug 1, plan 61, E2E-64): a lot of 200 with
// lines 20 at 1,000, 50 at 900 and 130 at 900. The backend wrote them in its own test
// (backend/crates/services/tests/fixtures/bikri_preview_d8_line3*.json). They are copied here, never
// edited, and read as the generated types. Money in them is paisa, as on the wire.
//   - LINE3: line 3 has its rate. Every figure of D8 Section 3 is in it (gross 182,000, deductions
//     21,920, net 160,080, shared income 23,440 split 9,376 / 7,032 / 7,032).
//   - RATE_EMPTY: line 3 has a buyer and the quantity but no rate. It has no figures, the hint 900, an
//     ERR_REQUIRED on its rate, and the lot counts its 130 as sold (balance 0): the demo screenshot.

import type { BikriDraft } from '../../src/bindings/BikriDraft';
import type { BikriPreview } from '../../src/bindings/BikriPreview';
import line3 from '../fixtures/bikri_preview_d8_line3.json';
import rateEmpty from '../fixtures/bikri_preview_d8_line3_rate_empty.json';
import { withKeysOf } from './bikri-over-qty-real';
import { umarAnswer } from './bikri-umar-data';

// A fixture is the backend's own JSON; a reader takes `unknown`, because the money in a binding is a
// branded type that a plain number is not.
function previewOf(value: unknown): BikriPreview {
  return value as BikriPreview;
}

export const LINE3_PREVIEW: BikriPreview = previewOf(line3);
export const RATE_EMPTY_PREVIEW: BikriPreview = previewOf(rateEmpty);

/**
 * The answer for a draft: the backend's own file once line 3 has its buyer (with or without the rate),
 * and the stand-in for the lines before it, for which the backend wrote no file.
 */
export function realAnswer(draft: BikriDraft): BikriPreview {
  const third = draft.lots[0]?.sales[2];
  if (third?.buyer_account_id == null) return umarAnswer(draft);
  return withKeysOf(third.seller_rate === '' ? RATE_EMPTY_PREVIEW : LINE3_PREVIEW, draft);
}
