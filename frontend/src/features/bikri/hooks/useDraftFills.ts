// What the app fills by itself, as visible values the user can change (PRD FR-7.51, FR-7.52,
// FR-12.2, NFR-USE-27, Code Standards F43). Each fill goes into the cell like a typed value and
// is an Enter stop. A fill happens once: the row remembers it, so a cell the user empties is not
// filled again.
//
// - A new lot's seller is the previous lot's seller, or the header زمیندار for the first lot.
// - Choosing a seller fills the item, packing and مارکہ of his last lot, into empty cells.
// - A new sale line's quantity is what the lot has left, from the backend (`remaining_qty`), filled
//   at once, before the buyer is chosen: the line is still blank (no buyer, no rate) until he
//   types, so the backend and the save ignore it. The screen never works it out.
// - The rate is never filled. The last rate is grey text in the cell (FR-7.53).
// - A lot of the firm's own goods (the seller is اپنا مال, PRD FR-12.26) takes its item, packing and
//   quantity from its purchase, not from a seller's memory, and has no نقد: what the row held from
//   an earlier seller is cleared once, so no cell is left that cannot be changed.

import { useEffect } from 'react';
import { isOwnGoods } from '../../../shared/accounts';
import { findMaster, type SearchHit } from '../../../shared/search';
import type { PickedRecord } from '../../../shared/grid';
import { assistSellerMemory } from '../api';
import { isOwnLot } from '../draft';
import type { BikriDraftStores } from '../draft-store';
import { LOT, ON, SALE, type BikriPreview } from '../types';

function pickedOf(hit: SearchHit | null): PickedRecord | null {
  return hit === null
    ? null
    : { id: hit.id, code: hit.code, name_ur: hit.name_ur, name_en: hit.name_en };
}

/** Fills item, packing and مارکہ from the seller's memory into the cells that are still empty. */
async function fillFromSellerMemory(
  stores: BikriDraftStores,
  rowId: string,
  sellerId: number,
): Promise<void> {
  try {
    const memory = await assistSellerMemory(sellerId);
    const [item, packing, marka] = await Promise.all([
      memory.item_id === null ? null : findMaster('item', memory.item_id),
      memory.packing_id === null ? null : findMaster('packing', memory.packing_id),
      memory.marka_id === null ? null : findMaster('marka', memory.marka_id),
    ]);
    const row = stores.lots.getState().rows.find((candidate) => candidate.rowId === rowId);
    // The seller may have been changed while the answer was on its way.
    if (row === undefined || row.picked?.[LOT.seller]?.id !== sellerId) return;
    const picked: Record<string, PickedRecord | null> = {};
    const fills: [string, SearchHit | null][] = [
      [LOT.item, item],
      [LOT.packing, packing],
      [LOT.marka, marka],
    ];
    for (const [field, hit] of fills) {
      if (row.picked[field] === undefined && hit !== null) picked[field] = pickedOf(hit);
    }
    stores.lots.getState().patchRow(rowId, { picked });
  } catch {
    // The memory is a help. If it cannot be read the user fills the cells himself.
  }
}

export function useDraftFills(
  stores: BikriDraftStores,
  preview: BikriPreview | undefined,
  /** True while the answer is for an older draft: nothing is filled from it. */
  stale: boolean,
): void {
  const lotRows = stores.lots((state) => state.rows);
  const zamindar = stores.header((state) => state.zamindar);

  // A lot with no seller yet gets the previous lot's seller, or the زمیندار for the first lot.
  useEffect(() => {
    lotRows.forEach((row, index) => {
      if (row.picked?.[LOT.seller] !== undefined || row.typed[LOT.sellerFilled] === ON) return;
      const candidate = index === 0 ? zamindar : lotRows[index - 1]?.picked?.[LOT.seller];
      // The firm's own goods are sold lot by lot from a purchase, never carried to the next lot.
      if (candidate === null || candidate === undefined || isOwnGoods(candidate)) return;
      stores.lots.getState().patchRow(row.rowId, {
        picked: { [LOT.seller]: candidate },
        typed: { [LOT.sellerFilled]: ON },
      });
    });
  }, [lotRows, zamindar, stores]);

  // A seller was chosen: ask what he sent last time, once for each seller on a row.
  useEffect(() => {
    for (const row of lotRows) {
      const seller = row.picked?.[LOT.seller];
      if (seller === undefined || isOwnLot(row)) continue;
      if (row.typed[LOT.memoryFilledFor] === String(seller.id)) continue;
      stores.lots
        .getState()
        .patchRow(row.rowId, { typed: { [LOT.memoryFilledFor]: String(seller.id) } });
      void fillFromSellerMemory(stores, row.rowId, seller.id);
    }
  }, [lotRows, stores]);

  // A lot became own goods: the item and packing of the seller before him go, the نقد goes. A lot
  // that is not own goods any more drops the purchase it had. A saved lot of own goods (an edit)
  // keeps everything it came with, so an edit that changes nothing sends the draft back as it came.
  useEffect(() => {
    for (const row of lotRows) {
      const purchase = row.typed[LOT.purchase] ?? '';
      if (!isOwnLot(row)) {
        if (purchase !== '') {
          stores.lots.getState().patchRow(row.rowId, { typed: { [LOT.purchase]: '' } });
        }
        continue;
      }
      const typed: Record<string, string> = {};
      const picked: Record<string, PickedRecord | null> = {};
      const saved = (row.typed[LOT.lotId] ?? '') !== '';
      if (!saved && (row.typed[LOT.cash] ?? '') !== '') typed[LOT.cash] = '';
      if (purchase === '' && !saved) {
        if (row.picked?.[LOT.item] !== undefined) picked[LOT.item] = null;
        if (row.picked?.[LOT.packing] !== undefined) picked[LOT.packing] = null;
      }
      if (Object.keys(typed).length > 0 || Object.keys(picked).length > 0) {
        stores.lots.getState().patchRow(row.rowId, { typed, picked });
      }
    }
  }, [lotRows, stores]);

  // A new sale line starts with the quantity the lot has left (PRD FR-7.52). A line with no buyer
  // and no rate is a blank row, so filling its quantity does not make it a sale.
  useEffect(() => {
    if (preview === undefined || stale) return;
    for (const lot of preview.lots) {
      const store = stores.salesFor(lot.key);
      for (const line of lot.lines) {
        const row = store.getState().rows.find((candidate) => candidate.rowId === line.key);
        if (row === undefined) continue;
        const empty = (row.typed[SALE.qty] ?? '') === '';
        if (!empty || row.typed[SALE.qtyFilled] === ON || line.remaining_qty <= 0) continue;
        store.getState().patchRow(row.rowId, {
          typed: { [SALE.qty]: String(line.remaining_qty), [SALE.qtyFilled]: ON },
        });
      }
    }
  }, [preview, stale, stores]);
}
