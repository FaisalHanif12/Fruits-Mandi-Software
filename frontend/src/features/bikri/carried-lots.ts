// Lots carried over from earlier بکری onto the screen of today's بکری (plan 76 point 1, PRD FR-12.17,
// FR-12.48). A carried lot is a saved lot with goods left: it is shown as a lot row whose cells cannot
// be typed in, and only the new sale lines typed on it are the day's work. The screen keeps the carried
// rows FIRST, in the order the backend lists them (carried lots, then new lots), so the answer of the
// preview and the paths of its problems fall on the right rows. A carried lot is never filled by the app
// and never takes its charges again: the backend decides that, the screen only shows it.

import type { StockRow } from '../../bindings/StockRow';
import type { EntryRow } from '../../shared/grid';
import { isCarriedLot } from './draft';
import type { BikriDraftStores } from './draft-store';
import { LOT, ON, SALE } from './types';

/** The lot row of a lot with goods left: the cells shown, the lot's id and the number of its بکری. */
export function carriedRowOf(stock: StockRow): EntryRow {
  return {
    rowId: globalThis.crypto.randomUUID(),
    typed: {
      [LOT.lotId]: String(stock.lot_id),
      [LOT.carried]: ON,
      [LOT.carriedBikri]: String(stock.bikri_number),
      // The seller and what the lot holds are known: nothing is filled again from his last lot
      // (PRD FR-7.51).
      [LOT.sellerFilled]: ON,
      [LOT.memoryFilledFor]: String(stock.seller.id),
      // What was received is shown; it is never sent (the backend reads the lot).
      [LOT.qty]: String(stock.qty_received),
    },
    picked: {
      [LOT.seller]: stock.seller,
      [LOT.item]: stock.item,
      [LOT.packing]: stock.packing,
    },
  };
}

/** The ids of the lots already on the screen as carried lots, so none is offered twice. */
export function carriedLotIds(stores: BikriDraftStores): Set<number> {
  return new Set(
    stores.lots
      .getState()
      .rows.filter(isCarriedLot)
      .map((row) => Number(row.typed[LOT.lotId])),
  );
}

/**
 * Puts lots with goods left on the screen, after the carried lots already there and before the new lots,
 * and moves the cursor to the first sale line of the first one added. A lot that is on the screen already is
 * left as it is. The one empty lot row that only waits to be typed in is kept after them: it is still the
 * place for a new lot. Returns how many were added.
 */
export function addCarriedLots(stores: BikriDraftStores, stock: readonly StockRow[]): number {
  const on = carriedLotIds(stores);
  const fresh = stock.filter((one) => !on.has(one.lot_id)).map(carriedRowOf);
  if (fresh.length === 0) return 0;
  const rows = stores.lots.getState().rows;
  const carried = rows.filter(isCarriedLot);
  const others = rows.filter((row) => !isCarriedLot(row));
  stores.lots.setState({ rows: [...carried, ...fresh, ...others] });
  const first = fresh[0];
  if (first !== undefined) {
    stores.view.getState().select(first.rowId);
    stores.focusSaleLine(first.rowId, 0, SALE.buyer);
  }
  return fresh.length;
}
