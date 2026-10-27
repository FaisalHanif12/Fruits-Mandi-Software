// Stock that is ready to sell (PRD FR-12.39, FR-12.40, plan 33). A new بکری brings in a lot for every
// stock record that has goods left and whose date to sell is on or before the بکری's date. Today
// the only kind is a trip purchase; a garden's goods arrived join it (plan 33, R10). The screen
// works with this one shape, so the way lots are made, removed and counted does not depend on the
// kind. The quantity is what the backend says is left: the screen counts nothing.

import type { RecordRef } from '../../bindings/RecordRef';

/** Where the goods are: a trip purchase (the lot's seller is اپنا مال, FR-12.26). */
export type ReadyKind = 'trip';

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
}

/** A stable key of a stock record across kinds, for "this one was removed from this form". */
export function readyKey(stock: Pick<ReadyStock, 'kind' | 'id'>): string {
  return `${stock.kind}:${String(stock.id)}`;
}

/** Which kind comes first on the same day. A new kind must be given its place here. */
const KIND_ORDER: Record<ReadyKind, number> = { trip: 0 };

/** Oldest day to sell first, then the kind, then the number (plan 33 R8). The backend's order. */
export function byDayThenNumber(a: ReadyStock, b: ReadyStock): number {
  if (a.sellDate !== b.sellDate) return a.sellDate < b.sellDate ? -1 : 1;
  const byKind = KIND_ORDER[a.kind] - KIND_ORDER[b.kind];
  return byKind !== 0 ? byKind : a.number - b.number;
}
