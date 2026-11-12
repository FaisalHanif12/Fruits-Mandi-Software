// What one بکری may hold (PRD NFR-SCL-3, v1.17): at most 50 lots and 1,000 sale lines over all of its
// lots. The backend keeps the same two numbers (`MAX_LOTS` and `MAX_SALE_LINES` of `mandi_core::bikri`)
// and refuses a draft over them; the screen only stops offering a new row at them, so the Owner is told
// before he types, not by a refusal on save.

import type { BikriDraftStores } from './draft-store';

/** The most lots in one بکری. */
export const MAX_LOTS = 50;
/** The most sale lines in one بکری, over all its lots. */
export const MAX_SALE_LINES = 1_000;

/** True when another lot would be over the limit. */
export function lotLimitReached(stores: BikriDraftStores): boolean {
  return stores.lots.getState().rows.length >= MAX_LOTS;
}

/** How many sale rows the whole draft holds, over every lot (a blank row counts, as it takes room too). */
export function saleRowCount(stores: BikriDraftStores): number {
  let count = 0;
  for (const lot of stores.lots.getState().rows) {
    count += stores.salesFor(lot.rowId).getState().rows.length;
  }
  return count;
}

/** True when another sale line would be over the limit. */
export function lineLimitReached(stores: BikriDraftStores): boolean {
  return saleRowCount(stores) >= MAX_SALE_LINES;
}
