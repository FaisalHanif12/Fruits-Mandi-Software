// Test helper: a بکری screen that opens on a draft that is already typed, the way it opens after the
// user chose "finish the last بکری" (plan 13, draft-offer.tsx). Typing a lot and a code into the
// cells of two grids takes many keys, and a busy computer can drop some, so the tests about the
// one-time person start from the draft and type only what they are about.

import { createBikriDraft } from '../../src/features/bikri/draft-store';
import { usePendingRestore } from '../../src/features/bikri/draft-offer';
import { parseSnapshot, serialiseDraft } from '../../src/features/bikri/draft-snapshot';
import { LOT, ON, SALE } from '../../src/features/bikri/types';
import { IDS } from './bikri-data';

const RAHIM = { id: IDS.rahim, code: '030110001', name_ur: 'رحیم', name_en: 'Rahim' };
const KARIM = { id: IDS.karim, code: '030110002', name_ur: 'کریم', name_en: 'Karim' };
const BANANA = { id: IDS.banana, code: '0001', name_ur: 'کیلا', name_en: 'Banana' };
const APPLE = { id: IDS.apple, code: '0002', name_ur: 'سیب', name_en: 'Apple' };
const BOX = { id: IDS.box, code: '0001', name_ur: 'پیٹی', name_en: 'Box' };
const PUKHTA_BUYER = {
  id: IDS.pukhtaBuyer,
  code: '03049999',
  name_ur: 'پختہ کھاتہ (خریدار)',
  name_en: 'Pukhta Khata (Buyer)',
};
const PUKHTA_SELLER = {
  id: IDS.pukhtaSeller,
  code: '030119999',
  name_ur: 'پختہ کھاتہ (بیوپاری)',
  name_en: 'Pukhta Khata (Seller)',
};

interface Seed {
  /** The lot: Rahim's bananas (default) or Karim's apples (Event 2), or a one-time seller. */
  lot?: 'rahim' | 'karim' | 'pukhta-seller';
  /** The first sale line goes to the one-time buyer account. */
  pukhtaBuyer?: boolean;
  /** Text typed on the first line. */
  line?: { qty?: string; rate?: string };
}

/** Makes the next screen the sale book opens as one holding this draft. Call before opening it. */
export function seedBikriDraft(seed: Seed = {}): void {
  const stores = createBikriDraft('10-10-2026');
  const lot = stores.lots.getState().rows[0];
  if (lot === undefined) throw new Error('no lot row');
  const kind = seed.lot ?? 'rahim';
  const seller = kind === 'rahim' ? RAHIM : kind === 'karim' ? KARIM : PUKHTA_SELLER;
  stores.header.getState().patch({
    truckNo: kind === 'rahim' ? 'LES-1234' : 'LHR-0077',
    zamindar: kind === 'pukhta-seller' ? RAHIM : seller,
  });
  stores.lots.getState().patchRow(lot.rowId, {
    typed: {
      [LOT.qty]: kind === 'rahim' ? '50' : '10',
      // Event 2's lot (PRD 4.5): freight 1,000, labour 50 and munshiana 50 typed as amounts.
      ...(kind === 'karim'
        ? { [LOT.freight]: '1000', [LOT.labour]: '50', [LOT.munshiana]: '50' }
        : {}),
      [LOT.sellerFilled]: ON,
      [LOT.memoryFilledFor]: String(seller.id),
    },
    picked: {
      [LOT.seller]: seller,
      [LOT.item]: kind === 'karim' ? APPLE : BANANA,
      [LOT.packing]: BOX,
    },
  });
  const line = stores.salesFor(lot.rowId).getState().rows[0];
  if (line === undefined) throw new Error('no line row');
  stores
    .salesFor(lot.rowId)
    .getState()
    .patchRow(line.rowId, {
      typed: {
        [SALE.qty]: seed.line?.qty ?? '',
        [SALE.rate]: seed.line?.rate ?? '',
        [SALE.qtyFilled]: ON,
      },
      ...(seed.pukhtaBuyer === true ? { picked: { [SALE.buyer]: PUKHTA_BUYER } } : {}),
    });
  const snapshot = parseSnapshot(serialiseDraft(stores));
  if (snapshot === null) throw new Error('the seeded draft is not a snapshot');
  usePendingRestore.getState().offer(snapshot);
}
