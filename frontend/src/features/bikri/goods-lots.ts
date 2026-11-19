// A lot made from goods that the munshi loaded by hand (PRD FR-12.50, FR-12.26, FR-12.36, FR-12.40, plan 78 C1).
// A trip purchase is sold by اپنا مال and keeps its purchase; a garden's goods arrived entry is sold by the garden (its
// account) and keeps its entry. The visible cells are filled like values typed (NFR-USE-27, F43); the quantity of
// the lot is the goods left the backend gave. The lot is charged and posted like any own lot or garden lot, so the
// commission is never missed.

import { useQuery } from '@tanstack/react-query';
import { accountsResolveCode, pickedOf, type AccountHit } from '../../shared/accounts';
import type { EntryRow } from '../../shared/grid';
import type { BikriDraftStores } from './draft-store';
import { isCarriedLot, isEmptyLotRow } from './draft';
import type { ReadyStock } from './ready-stock';
import { LOT, ON, OWN_GOODS_CODE, SALE } from './types';

/** The اپنا مال account, found by its fixed code (PRD FR-10.4). Null when it is not offered. */
export function useOwnGoodsAccount(enabled: boolean) {
  return useQuery({
    queryKey: ['accounts', 'own-goods-account'],
    enabled,
    queryFn: async (): Promise<AccountHit | null> => {
      const answer = await accountsResolveCode('lot_seller', null, OWN_GOODS_CODE);
      return answer.status === 'found' ? answer.account : null;
    },
  });
}

/** The lot row of a stock record. Null while the account of اپنا مال is not known (a trip purchase needs it). */
export function goodsLotRow(stock: ReadyStock, own: AccountHit | null): EntryRow | null {
  const common = {
    [LOT.qty]: String(stock.qtyLeft),
    // The seller is set, so the screen does not fill it again from the last lot.
    [LOT.sellerFilled]: ON,
  };
  const goods = {
    [LOT.item]: pickedOf(stock.item),
    [LOT.packing]: pickedOf(stock.packing),
  };
  switch (stock.kind) {
    case 'trip':
      return own === null
        ? null
        : {
            rowId: globalThis.crypto.randomUUID(),
            typed: { ...common, [LOT.purchase]: String(stock.id) },
            picked: { [LOT.seller]: pickedOf(own), ...goods },
          };
    case 'garden':
      return stock.garden === null
        ? null
        : {
            rowId: globalThis.crypto.randomUUID(),
            typed: { ...common, [LOT.arrival]: String(stock.id) },
            picked: { [LOT.seller]: pickedOf(stock.garden.account), ...goods },
          };
  }
}

/** The ids of the purchases or arrivals already on the screen as lots of this kind, so none is offered twice. */
export function goodsLotIds(rows: readonly EntryRow[], kind: ReadyStock['kind']): Set<number> {
  const field = kind === 'trip' ? LOT.purchase : LOT.arrival;
  return new Set(
    rows
      .map((row) => row.typed[field] ?? '')
      .filter((text) => text !== '' && text !== '0')
      .map(Number),
  );
}

/**
 * Puts one goods lot on the screen, after the lots carried over and the lots of goods already loaded, and before
 * the one empty lot row that only waits to be typed in (it is kept after it). Moves the cursor to the first sale
 * line of the lot. A lot that is on the screen already is left as it is. Returns true when a lot was added.
 */
export function addGoodsLot(stores: BikriDraftStores, row: EntryRow): boolean {
  const rows = stores.lots.getState().rows;
  const waiting = (candidate: EntryRow) => !isCarriedLot(candidate) && isEmptyLotRow(candidate);
  const placed = rows.filter((candidate) => !waiting(candidate));
  const empty = rows.filter(waiting);
  stores.lots.setState({ rows: [...placed, row, ...empty] });
  stores.view.getState().select(row.rowId);
  stores.focusSaleLine(row.rowId, 0, SALE.buyer);
  return true;
}
