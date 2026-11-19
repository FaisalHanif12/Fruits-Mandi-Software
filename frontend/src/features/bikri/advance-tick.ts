// The tick "پیشگی اس بکری سے کاٹیں" of a seller on the summary (PRD FR-12.23, plan 78 C3, Umar 17-11-2026): one tick
// for every lot of that seller in this بکری, off when a بکری opens. What each lot sends is `adjustAdvanceOf`
// (`draft.ts`); these two functions are what the summary reads and changes, from the text on the lot rows (Code
// Standards F39). Nothing here works out money.

import type { EntryRow } from '../../shared/grid';
import { tickLotsOf } from './draft';
import { LOT, ON } from './types';

/** Is the seller's tick on? Yes when a lot of his that it covers has it on. */
export function isSellerTicked(rows: readonly EntryRow[], sellerId: number): boolean {
  return tickLotsOf(rows, sellerId).some((row) => row.typed[LOT.adjustAdvance] === ON);
}

/** The tick of a seller made on or off: every lot of his that it covers takes it, so one tick covers them all. */
export function setSellerTick(
  rows: readonly EntryRow[],
  sellerId: number,
  on: boolean,
  update: (rowId: string, field: string, text: string) => void,
): void {
  for (const row of tickLotsOf(rows, sellerId)) {
    update(row.rowId, LOT.adjustAdvance, on ? ON : '0');
  }
}
