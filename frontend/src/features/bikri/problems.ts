// What the preview says is wrong or worth a look, placed where the user can see it: on the cell,
// on the field, and in one list in words (PRD FR-7.6, NFR-USE-12: colour is never the only
// signal). The backend gives each problem a code, a field path such as `lots[0].sales[2].qty`, and
// values (plan 12 contract 4 and 11). This file only sorts them; the words come from Fluent.
//
// A problem that only says "this is empty" is not shown while the user is still typing: a new row
// would turn red before he has reached it. Those come when he tries to save (plan 13 passes
// `showIncomplete`). A problem on a field that has text, and a few that tell him something to do
// (weight needed), show at once.

import type { ErrorCode } from '../../bindings/ErrorCode';
import type { FieldError } from './types';

export type Severity = 'error' | 'warning';

/** Codes that only say a field is empty. Held back until the user saves. */
const INCOMPLETE: ReadonlySet<ErrorCode> = new Set<ErrorCode>([
  'ERR_REQUIRED',
  'ERR_SELLER_REQUIRED',
  'ERR_BUYER_REQUIRED',
  'ERR_PUKHTA_NAME_REQUIRED',
  'ERR_PUKHTA_PHONE_REQUIRED',
  'ERR_NO_LOTS',
  'ERR_NO_SALE_LINES',
]);

/** Where a field path points. */
export type Where =
  | { kind: 'header'; sub: string }
  | { kind: 'lots'; sub: string }
  | { kind: 'lot'; lot: number; sub: string }
  | { kind: 'line'; lot: number; line: number; sub: string };

const LINE_PATH = /^lots\[(\d+)\]\.sales\[(\d+)\](?:\.(.+))?$/;
const LOT_PATH = /^lots\[(\d+)\](?:\.(.+))?$/;
const HEADER_PATH = /^header\.(.+)$/;

/** Reads a field path of the preview: header, the lots, one lot, or one sale line. */
export function whereOf(field: string): Where {
  const line = LINE_PATH.exec(field);
  if (line !== null) {
    return { kind: 'line', lot: Number(line[1]), line: Number(line[2]), sub: line[3] ?? '' };
  }
  const lot = LOT_PATH.exec(field);
  if (lot !== null) return { kind: 'lot', lot: Number(lot[1]), sub: lot[2] ?? '' };
  const header = HEADER_PATH.exec(field);
  if (header !== null) return { kind: 'header', sub: header[1] ?? '' };
  return { kind: 'lots', sub: field };
}

export interface Problem {
  item: FieldError;
  severity: Severity;
  where: Where;
}

/** The problems to show now: errors first, then warnings, in the order the backend gave them. */
export function problemsOf(
  errors: readonly FieldError[],
  warnings: readonly FieldError[],
  showIncomplete: boolean,
): Problem[] {
  const keep = (item: FieldError) => showIncomplete || !INCOMPLETE.has(item.code);
  return [
    ...errors
      .filter(keep)
      .map((item) => ({ item, severity: 'error' as const, where: whereOf(item.field) })),
    ...warnings
      .filter(keep)
      .map((item) => ({ item, severity: 'warning' as const, where: whereOf(item.field) })),
  ];
}

/**
 * Parts of a lot or a line that no field on the screen shows: the ids an edit sends back (plan
 * 16). A problem on one of them, such as `lots[0].lot_id` ("this lot is not part of the entry"),
 * is always told in the list, with the lot and line it is about, never left unseen.
 */
export const SUB_WITHOUT_FIELD: ReadonlySet<string> = new Set(['lot_id', 'sale_id']);

/** The column of the lots grid a lot field belongs to. Fields not listed are in the "more" panel. */
export const LOT_GRID_FIELD: Readonly<Record<string, string>> = {
  seller_account_id: 'seller',
  item_id: 'item',
  packing_id: 'packing',
  qty_received: 'qty',
  freight: 'freight',
  labour: 'labour',
  munshiana: 'munshiana',
  cash_paid: 'cash',
};

/**
 * The fields of a lot that the panel of an own goods lot shows (OwnLotPanel), so the "more" panel
 * does not open for them and the list does not tell them twice (PRD FR-12.26).
 */
export const LOT_OWN_FIELD: ReadonlySet<string> = new Set(['purchase_id']);

/** The column of the sale lines grid a line field belongs to. Others are in the line details. */
export const LINE_GRID_FIELD: Readonly<Record<string, string>> = {
  buyer_account_id: 'buyer',
  qty: 'qty',
  seller_rate: 'rate',
  payment_type: 'payment',
};
