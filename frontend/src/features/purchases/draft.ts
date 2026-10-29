// The purchase as text in the screen's state (Code Standards F39): an empty set of fields, the
// input the backend takes, and the field paths the backend's problems point at. Nothing here
// works out money: every amount is the text the user typed (CLAUDE.md rule 1).

import type { EntryPartInput } from '../../bindings/EntryPartInput';
import type { PurchaseFields, PurchaseInput } from './types';

/** A new purchase: everything empty (PRD 12.10). */
export function emptyPurchaseFields(): PurchaseFields {
  return {
    supplier: '',
    item: null,
    packing: null,
    qty: '',
    weight: '',
    rate: '',
    freight: '',
    labour: '',
    otherCost: '',
    sellDate: '',
  };
}

/**
 * The next purchase after one was saved: the supplier stays, as the user is likely to buy several
 * lots from one person in one sitting; everything else starts again.
 */
export function nextPurchaseFields(previous: PurchaseFields): PurchaseFields {
  return { ...emptyPurchaseFields(), supplier: previous.supplier };
}

/** A purchase counts as filled once anything is typed or chosen. */
export function purchaseIsFilled(fields: PurchaseFields): boolean {
  return (
    fields.item !== null ||
    fields.packing !== null ||
    [
      fields.supplier,
      fields.qty,
      fields.weight,
      fields.rate,
      fields.freight,
      fields.labour,
      fields.otherCost,
      fields.sellDate,
    ].some((text) => text.trim() !== '')
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
  note: string;
  /** One row for every partner of the trip with this entry's source (PRD A54). */
  parts: EntryPartInput[];
}

/**
 * What goes to the backend: the text typed and the ids of what was chosen. `confirmCash` is the
 * user's answer to "is this entry correct?" for a purchase whose shop parts take cash below zero
 * (PRD FR-6.7); the preview ignores it.
 */
export function buildPurchase(
  fields: PurchaseFields,
  common: PurchaseCommon,
  confirmCash: boolean,
): PurchaseInput {
  return {
    trip_id: common.tripId,
    date: common.date.trim(),
    // Empty: the date to sell is the day of the purchase (PRD FR-12.39).
    sell_date: textOrNull(fields.sellDate),
    supplier: textOrNull(fields.supplier),
    item_id: fields.item?.id ?? null,
    packing_id: fields.packing?.id ?? null,
    qty: fields.qty.trim(),
    weight: textOrNull(fields.weight),
    rate: fields.rate.trim(),
    freight: textOrNull(fields.freight),
    labour: textOrNull(fields.labour),
    other_cost: textOrNull(fields.otherCost),
    note: textOrNull(common.note),
    parts: common.parts,
    confirm_cash_below_zero: confirmCash,
  };
}

/**
 * The fields of the purchase form, by the field paths the backend's problems use (PRD 12.10). The
 * trip and the parts have no field of their own on the screen, so their problems are listed in
 * words near the buttons instead.
 */
export const PURCHASE_FIELD_PATHS = [
  'date',
  'supplier',
  'item_id',
  'packing_id',
  'qty',
  'weight',
  'rate',
  'freight',
  'labour',
  'other_cost',
  'note',
] as const;
