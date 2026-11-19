// Goods that are loaded into a بکری by hand (PRD FR-12.50, FR-12.26, FR-12.36, plan 78 C1): a trip purchase
// or a garden's goods arrived entry with goods left for our shop. The munshi picks one from its card and it
// becomes a lot (it no longer comes in by itself, plan 33 is replaced). The screen works with this one shape,
// so the way a lot is made does not depend on the kind. The quantity is what the backend says is left: the
// screen counts nothing.

import type { GardenRef } from '../../bindings/GardenRef';
import type { ReadyGoods } from '../../bindings/ReadyGoods';
import type { RecordRef } from '../../bindings/RecordRef';
import type { TripRef } from '../../bindings/TripRef';

/**
 * Where the goods are: a trip purchase (the lot's seller is اپنا مال, FR-12.26) or a garden's
 * goods arrived entry (the lot's seller is the garden, FR-12.36, FR-12.40).
 */
export type ReadyKind = 'trip' | 'garden';

export interface ReadyStock {
  kind: ReadyKind;
  /** The purchase id. Unique within its kind. */
  id: number;
  /** The purchase number (خ-1), the tie-break after the date. */
  number: number;
  /** The day to sell, `YYYY-MM-DD`. */
  sellDate: string;
  item: RecordRef;
  packing: RecordRef;
  /** The goods left. This is the quantity of the lot. */
  qtyLeft: number;
  /** The garden of a garden entry: its account is the seller of the lot. Null for trip goods. */
  garden: GardenRef | null;
  /** The trip of a trip purchase. Null for garden goods. */
  trip: TripRef | null;
}

/**
 * The goods the backend listed as stock rows. The backend lists goods left; the screen still checks it, so a wrong
 * answer never makes a lot: a trip row has its trip, a garden row its garden, and nothing is left means no row.
 */
export function readyStockOf(answer: ReadyGoods): ReadyStock[] {
  return answer.rows.flatMap((row): ReadyStock[] => {
    if (row.qty_left <= 0) return [];
    const base = {
      id: row.id,
      number: row.number,
      sellDate: row.sell_date,
      item: row.item,
      packing: row.packing,
      qtyLeft: row.qty_left,
    };
    if (row.kind === 'trip' && row.trip !== null) {
      return [{ ...base, kind: 'trip', garden: null, trip: row.trip }];
    }
    if (row.kind === 'garden' && row.garden !== null) {
      return [{ ...base, kind: 'garden', garden: row.garden, trip: null }];
    }
    return [];
  });
}

/** Which kind comes first on the same day. A new kind must be given its place here. */
const KIND_ORDER: Record<ReadyKind, number> = { trip: 0, garden: 1 };

/** Oldest day to sell first, then the kind, then the number (plan 33 R8). The backend's order. */
export function byDayThenNumber(a: ReadyStock, b: ReadyStock): number {
  if (a.sellDate !== b.sellDate) return a.sellDate < b.sellDate ? -1 : 1;
  const byKind = KIND_ORDER[a.kind] - KIND_ORDER[b.kind];
  return byKind !== 0 ? byKind : a.number - b.number;
}
