// Ready stock comes into a new بکری by itself (PRD FR-12.39, FR-12.40, plan 33). When a new
// بکری opens, every stock record that has goods left and whose date to sell is on or before the
// بکری's date is put in as a lot: for a trip purchase the seller is اپنا مال, the purchase is
// chosen, the item and packing are the purchase's, and the quantity is the goods left. The user
// adds the buyers, the quantities and the rates; the lot is charged and posted like any own lot
// (FR-12.26), so the commission is never missed. The lots are the first lots; a lot of the truck
// comes after them. Oldest date to sell first, then the number.
//
// - Only a new بکری. A saved one, a later sale and an edit never use this hook.
// - The lots are put in before the user has typed a line. Until then they follow the date: if he
//   changes the date of the بکری, the lots of the new date take their place. Once a line is typed
//   on any lot, or a lot of his own is made, nothing is added or taken away by itself.
// - A lot the user removes is not put back in the same form (R3), even if the date changes.
// - A form that holds only these lots is not the user's work: see `isFilledByUser` (draft.ts).
//
// Which stock is ready is the backend's (`useReadyStock`). The frontend counts nothing: the
// quantity of a lot is the stock record's `qtyLeft`.

import { useQuery } from '@tanstack/react-query';
import { useEffect, useMemo } from 'react';
import { accountsResolveCode, pickedOf, type AccountHit } from '../../../shared/accounts';
import type { EntryRow } from '../../../shared/grid';
import type { BikriDraftStores } from '../draft-store';
import { byDayThenNumber, readyKey, type ReadyStock } from '../ready-stock';
import { LOT, ON, OWN_GOODS_CODE, SALE } from '../types';
import { useReadyStock } from './useReadyStock';

/** The date text of a whole date, DD-MM-YYYY, as YYYY-MM-DD, or null while it is not one yet. */
const FULL_DATE = /^(\d{1,2})-(\d{1,2})-(\d{4})$/;

export function isoOfDateText(text: string): string | null {
  const match = FULL_DATE.exec(text.trim());
  if (match === null) return null;
  const day = Number(match[1]);
  const month = Number(match[2]);
  const year = Number(match[3]);
  const real = new Date(Date.UTC(year, month - 1, day));
  if (
    real.getUTCFullYear() !== year ||
    real.getUTCMonth() !== month - 1 ||
    real.getUTCDate() !== day
  ) {
    return null;
  }
  const two = (value: number) => String(value).padStart(2, '0');
  return `${String(year)}-${two(month)}-${two(day)}`;
}

/** The اپنا مال account, found by its fixed code (PRD FR-10.4). Null when it is not offered. */
function useOwnGoodsAccount(enabled: boolean) {
  return useQuery({
    queryKey: ['accounts', 'own-goods-account'],
    enabled,
    queryFn: async (): Promise<AccountHit | null> => {
      const answer = await accountsResolveCode('lot_seller', null, OWN_GOODS_CODE);
      return answer.status === 'found' ? answer.account : null;
    },
  });
}

/** Has nothing been typed on this lot row but what the app filled? */
function isEmptyLotRow(row: EntryRow): boolean {
  if (Object.keys(row.picked ?? {}).length > 0) return false;
  return Object.entries(row.typed).every(
    ([field, text]) => text === '' || field === LOT.sellerFilled || field === LOT.memoryFilledFor,
  );
}

/** Has a line been typed on the lot: a buyer or a rate, or a one-time buyer's name? */
function hasLines(stores: BikriDraftStores, lotKey: string): boolean {
  return stores
    .salesFor(lotKey)
    .getState()
    .rows.some(
      (sale) =>
        sale.picked?.[SALE.buyer] !== undefined ||
        (sale.typed[SALE.rate] ?? '') !== '' ||
        (sale.typed[SALE.pukhtaUr] ?? '') !== '' ||
        (sale.typed[SALE.pukhtaEn] ?? '') !== '',
    );
}

/**
 * True while the form holds nothing of the user's: every lot is one that came in by itself or an
 * empty row, and no lot has a line. Only then do the lots follow the date.
 */
function isUntouched(stores: BikriDraftStores): boolean {
  const { readyLots } = stores.view.getState();
  return stores.lots
    .getState()
    .rows.every(
      (row) => !hasLines(stores, row.rowId) && (row.rowId in readyLots || isEmptyLotRow(row)),
    );
}

/**
 * The lot row of a stock record: the visible cells, filled like values typed (NFR-USE-27, F43). A
 * trip purchase is sold by اپنا مال and keeps its purchase; a garden's entry is sold by the garden
 * (its account) and keeps its entry. Null while the account of اپنا مال is not known yet.
 */
function lotRowOf(stock: ReadyStock, own: AccountHit | null): EntryRow | null {
  const common = {
    [LOT.qty]: String(stock.qtyLeft),
    // The seller is set, so the screen does not fill it again from the زمیندار.
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

const sameKeys = (a: readonly string[], b: readonly string[]) =>
  a.length === b.length && a.every((key, index) => key === b[index]);

/**
 * Puts the ready lots in the lots of the new بکری, and keeps them in step with the date and with
 * the lots the user removes. Returns how many of them are in the form now (for the note).
 */
export function useReadyLots(stores: BikriDraftStores, enabled: boolean): number {
  const entryDate = stores.header((state) => state.entryDate);
  const lotRows = stores.lots((state) => state.rows);
  const readyLots = stores.view((state) => state.readyLots);
  const iso = enabled ? isoOfDateText(entryDate) : null;
  const own = useOwnGoodsAccount(enabled);
  const ownAccount = own.data ?? null;
  // The screen checks what it was promised (goods left, a day to sell not after the بکری's day),
  // so a wrong answer never makes a lot.
  const answered = useReadyStock(iso);
  const found = useMemo(
    () =>
      answered
        ?.filter((stock) => iso !== null && stock.sellDate <= iso && stock.qtyLeft > 0)
        .sort(byDayThenNumber),
    [answered, iso],
  );

  useEffect(() => {
    if (!enabled) return;
    // A lot of the form that came in by itself and is gone now was removed by the user: it is not
    // put back in this form (R3).
    const { readyLots: known, removedReady } = stores.view.getState();
    const rows = stores.lots.getState().rows;
    let removed = removedReady;
    const kept: Record<string, string> = {};
    for (const [rowId, key] of Object.entries(known)) {
      if (rows.some((row) => row.rowId === rowId)) kept[rowId] = key;
      else removed = [...removed, key];
    }
    if (removed !== removedReady) stores.view.getState().setReadyLots(kept, removed);

    // The lots of the day, while nothing of the user's is on the form.
    if (found === undefined || !isUntouched(stores)) return;
    const wanted = found.filter((stock) => !removed.includes(readyKey(stock)));
    // The account of اپنا مال is asked for once: wait for it while a trip purchase needs it. If it
    // is not offered, trip goods have no lot, and the garden goods still do.
    if (own.isPending && wanted.some((stock) => stock.kind === 'trip')) return;
    const lots = wanted.flatMap((stock) => {
      const row = lotRowOf(stock, ownAccount);
      return row === null ? [] : [{ stock, row }];
    });
    const have = rows.flatMap((row) => (row.rowId in kept ? [kept[row.rowId] ?? ''] : []));
    if (
      sameKeys(
        lots.map((lot) => readyKey(lot.stock)),
        have,
      )
    ) {
      return;
    }
    stores.lots.setState({
      rows:
        lots.length > 0
          ? lots.map((lot) => lot.row)
          : [{ rowId: globalThis.crypto.randomUUID(), typed: {} }],
    });
    stores.view
      .getState()
      .setReadyLots(
        Object.fromEntries(lots.map((lot) => [lot.row.rowId, readyKey(lot.stock)])),
        removed,
      );
  }, [enabled, found, own.isPending, ownAccount, lotRows, stores]);

  return lotRows.filter((row) => row.rowId in readyLots).length;
}
