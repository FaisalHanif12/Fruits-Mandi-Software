// The backend's own answers for money paid on the spot (plan 65, PRD FR-12.44, E2E-65). The backend wrote
// them in its own test (backend/crates/services/tests/fixtures/bikri_preview_paid_*.json). They are
// copied here, never edited, and read as the generated types. Money in them is paisa, as on the wire.
//   - PAID_D8: Umar's example, the lot of 200 sold in three lines, typed paid 21,360 / 10,000 / nothing:
//     left 0 / 38,100 / 125,060, the summary 194,520 bill, 163,160 left on the accounts, 31,360 paid.
//   - PAID_FIXED_CHUKTA: Event 2, Shahid (a چکتا buyer) buys 10 at 1,200 and nothing is typed in the paid
//     cell: the backend says `paid_fixed` and paid 12,800, left 0.
//   - PAID_OVER_TOTAL: the first line typed as 21,361, one rupee over its 21,360: the line keeps its figures,
//     paid_paisa is 0 and the error `ERR_PAID_EXCEEDS_TOTAL` is on `lots[0].sales[0].paid` with max 21,360.

import type { BikriDraft } from '../../src/bindings/BikriDraft';
import type { BikriPreview } from '../../src/bindings/BikriPreview';
import paidD8 from '../fixtures/bikri_preview_paid_d8.json';
import paidFixedChukta from '../fixtures/bikri_preview_paid_fixed_chukta.json';
import paidOver from '../fixtures/bikri_preview_paid_over_total.json';
import { withKeysOf } from './bikri-over-qty-real';
import { umarAnswer } from './bikri-umar-data';

// A fixture is the backend's own JSON; a reader takes `unknown`, because the money in a binding is a
// branded type that a plain number is not.
function previewOf(value: unknown): BikriPreview {
  return value as BikriPreview;
}

export const PAID_D8_PREVIEW: BikriPreview = previewOf(paidD8);
export const PAID_FIXED_CHUKTA_PREVIEW: BikriPreview = previewOf(paidFixedChukta);
export const PAID_OVER_TOTAL_PREVIEW: BikriPreview = previewOf(paidOver);

/** The text of a typed amount without its grouping, so "21,361" and "21361" are the same words. */
function plain(text: string | null | undefined): string {
  return (text ?? '').replaceAll(',', '');
}

/**
 * The answer for a draft of Umar's lot: the backend's own file once line 3 has its buyer and its rate
 * (the over file when line 1 is typed as 21,361, else the example), and the stand-in for the lines
 * before it, for which the backend wrote no file.
 */
export function paidAnswer(draft: BikriDraft): BikriPreview {
  const third = draft.lots[0]?.sales[2];
  if (third?.buyer_account_id == null || third.seller_rate === '') return umarAnswer(draft);
  const first = plain(draft.lots[0]?.sales[0]?.paid);
  return withKeysOf(first === '21361' ? PAID_OVER_TOTAL_PREVIEW : PAID_D8_PREVIEW, draft);
}

/** The answer of Event 2 for the one line of Karim's lot, with the keys the screen sent. */
export function chuktaAnswer(draft: BikriDraft): BikriPreview {
  return withKeysOf(PAID_FIXED_CHUKTA_PREVIEW, draft);
}
