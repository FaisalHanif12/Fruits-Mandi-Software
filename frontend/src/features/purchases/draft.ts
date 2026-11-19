// The purchase entry as text in the screen's state (Code Standards F39): an empty set of fields,
// the input the backend takes, and the field paths the backend's problems point at. The item lines
// are rows in the lines grid's store (`lines.ts`). Nothing here works out money: every amount is
// the text the user typed (CLAUDE.md rule 1).

import type { EntryPartInput } from '../../bindings/EntryPartInput';
import type { EntryRow } from '../../shared/grid';
import { linesAreFilled, purchaseLineInputs } from './lines';
import type { PurchaseEntryFields, PurchaseEntryInput } from './types';

/** A new purchase entry: everything empty (PRD 12.10). */
export function emptyPurchaseFields(): PurchaseEntryFields {
  return { supplier: '', sellDate: '' };
}

/**
 * The next entry after one was saved: the supplier stays, as the user is likely to buy several
 * lots from one person in one sitting; everything else starts again.
 */
export function nextPurchaseFields(previous: PurchaseEntryFields): PurchaseEntryFields {
  return { ...emptyPurchaseFields(), supplier: previous.supplier };
}

/** The entry counts as filled once anything is typed or chosen, on the fields or on a line. */
export function purchaseIsFilled(fields: PurchaseEntryFields, rows: readonly EntryRow[]): boolean {
  return (
    linesAreFilled(rows) || [fields.supplier, fields.sellDate].some((text) => text.trim() !== '')
  );
}

/** Text or null: an empty field is sent as nothing, so the backend treats it as not typed. */
export function textOrNull(text: string): string | null {
  const trimmed = text.trim();
  return trimmed === '' ? null : trimmed;
}

/** What the screen shares between the types of the trip screen. */
export interface PurchaseCommon {
  tripId: number;
  /** `DD-MM-YYYY` as typed. */
  date: string;
  /** The entry's note: every line carries it. */
  note: string;
  /** One row for every partner of the trip with this entry's source (PRD A54). */
  parts: EntryPartInput[];
}

/**
 * What goes to the backend: the text typed and the ids of what was chosen, with every filled line
 * (PRD FR-12.41). `confirmCash` is the user's answer to "is this entry correct?" for an entry
 * whose shop parts take cash below zero (PRD FR-6.7); the preview ignores it.
 */
export function buildPurchaseEntry(
  fields: PurchaseEntryFields,
  rows: readonly EntryRow[],
  common: PurchaseCommon,
  confirmCash: boolean,
): PurchaseEntryInput {
  return {
    trip_id: common.tripId,
    date: common.date.trim(),
    // Empty: the date to sell is the day of the purchase (PRD FR-12.39).
    sell_date: textOrNull(fields.sellDate),
    supplier: textOrNull(fields.supplier),
    parts: common.parts,
    confirm_cash_below_zero: confirmCash,
    lines: purchaseLineInputs(rows, common.note),
  };
}

/**
 * The fields of the entry form that are not on a line, by the field paths the backend's problems
 * use (PRD 12.10). The trip and the parts have no field of their own on the screen, so their
 * problems are listed in words near the buttons instead.
 */
export const PURCHASE_FIELD_PATHS = ['date', 'supplier', 'sell_date', 'note'] as const;
