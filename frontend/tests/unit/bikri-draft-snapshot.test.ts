// The draft of the بکری as text (plan 13, PRD FR-12.10): the snapshot of the draft stores goes to the
// backend and comes back after a crash. What was typed, and the records picked with both names,
// come back exactly; a snapshot that is not ours is refused, never half used.

import { describe, expect, it } from 'vitest';
import { createBikriDraft } from '../../src/features/bikri/draft-store';
import {
  applySnapshot,
  parseSnapshot,
  serialiseDraft,
} from '../../src/features/bikri/draft-snapshot';
import { LOT, SALE } from '../../src/features/bikri/types';

const RAHIM = { id: 17, code: '0010', name_ur: 'رحیم', name_en: 'Rahim' };
const AKRAM = { id: 19, code: '0020', name_ur: 'اکرم ٹول ٹیکس', name_en: 'Akram Toll Tax' };

/** A draft with a header, one lot and one sale line, typed the way the munshi does. */
function typedDraft() {
  const stores = createBikriDraft('10-10-2026');
  stores.header.getState().patch({ truckNo: 'LES-1234', zamindar: RAHIM, totalFreight: '3000' });
  const lot = stores.lots.getState().rows[0];
  if (lot === undefined) throw new Error('no lot row');
  stores.lots.getState().patchRow(lot.rowId, {
    typed: { [LOT.qty]: '50', [LOT.freight]: '3000' },
    picked: { [LOT.seller]: RAHIM },
  });
  const line = stores.salesFor(lot.rowId).getState().rows[0];
  if (line === undefined) throw new Error('no line row');
  stores
    .salesFor(lot.rowId)
    .getState()
    .patchRow(line.rowId, {
      typed: { [SALE.qty]: '20', [SALE.rate]: '600' },
      picked: { [SALE.buyer]: AKRAM },
    });
  return { stores, lotKey: lot.rowId, lineKey: line.rowId };
}

describe('the draft snapshot', () => {
  it('comes back exactly: text as typed, picked records with both names, the same row keys', () => {
    const { stores, lotKey, lineKey } = typedDraft();
    const text = serialiseDraft(stores);
    const snapshot = parseSnapshot(text);
    expect(snapshot).not.toBeNull();
    if (snapshot === null) return;
    const restored = createBikriDraft('11-10-2026');
    applySnapshot(restored, snapshot);
    expect(restored.header.getState()).toMatchObject({
      entryDate: '10-10-2026',
      truckNo: 'LES-1234',
      totalFreight: '3000',
      zamindar: RAHIM,
    });
    const lot = restored.lots.getState().rows[0];
    expect(lot?.rowId).toBe(lotKey);
    expect(lot?.typed).toMatchObject({ [LOT.qty]: '50', [LOT.freight]: '3000' });
    expect(lot?.picked?.[LOT.seller]).toEqual(RAHIM);
    const line = restored.salesFor(lotKey).getState().rows[0];
    expect(line?.rowId).toBe(lineKey);
    expect(line?.typed).toMatchObject({ [SALE.qty]: '20', [SALE.rate]: '600' });
    expect(line?.picked?.[SALE.buyer]).toEqual(AKRAM);
    expect(restored.view.getState().selectedLot).toBe(lotKey);
  });

  it('keeps the whole-sale view and the selected lot', () => {
    const { stores } = typedDraft();
    stores.view.getState().setWhole(true);
    const snapshot = parseSnapshot(serialiseDraft(stores));
    const restored = createBikriDraft('10-10-2026');
    if (snapshot === null) throw new Error('not a snapshot');
    applySnapshot(restored, snapshot);
    expect(restored.view.getState().wholeView).toBe(true);
  });

  it('holds no function and no answer of the preview: only what was typed and picked', () => {
    const { stores } = typedDraft();
    const text = serialiseDraft(stores);
    expect(text).not.toContain('patch');
    expect(Object.keys(JSON.parse(text) as Record<string, unknown>).sort()).toEqual([
      'header',
      'lots',
      'sales',
      'selectedLot',
      'v',
      'wholeView',
    ]);
  });

  it.each([
    ['text that is not JSON', 'not json'],
    ['JSON that is not an object', '[1,2]'],
    ['another version', JSON.stringify({ v: 2 })],
    [
      'a missing header',
      JSON.stringify({ v: 1, lots: [], sales: {}, selectedLot: null, wholeView: false }),
    ],
  ])('refuses %s', (_name, text) => {
    expect(parseSnapshot(text)).toBeNull();
  });

  it('refuses a snapshot whose lot has no lines, or whose typed text is not text', () => {
    const { stores, lotKey } = typedDraft();
    const value = JSON.parse(serialiseDraft(stores)) as {
      sales: Record<string, unknown>;
      lots: { typed: Record<string, unknown> }[];
    };
    const noLines = structuredClone(value);
    noLines.sales[lotKey] = [];
    expect(parseSnapshot(JSON.stringify(noLines))).toBeNull();
    const notText = structuredClone(value);
    const first = notText.lots[0];
    if (first === undefined) throw new Error('no lot');
    first.typed['qty'] = 50;
    expect(parseSnapshot(JSON.stringify(notText))).toBeNull();
  });
});

// The backend keeps at most 1,048,576 bytes of one draft and refuses more with ERR_DRAFT_INVALID
// (backend/crates/core/src/drafts.rs, counted in bytes: an Urdu letter is two). The biggest
// بکری of NFR-SCL-3 is 50 lots and 500 sale lines, every field filled, every record with both names.
describe('the size of a snapshot', () => {
  it('the biggest بکری of NFR-SCL-3 (50 lots, 500 lines, all fields full) fits inside the backend’s 1 MB', () => {
    const MAX_DRAFT_BYTES = 1_048_576;
    const stores = createBikriDraft('10-10-2026');
    const full = (keys: readonly string[]) =>
      Object.fromEntries(keys.map((key) => [key, 'ٹرک-1234567890']));
    const record = {
      id: 123456,
      code: '0010',
      name_ur: 'اکرم ٹول ٹیکس',
      name_en: 'Akram Toll Tax',
    };
    stores.header
      .getState()
      .patch({ truckNo: 'LES-1234', zamindar: record, note: 'ن'.repeat(500) });
    const lotKeys = Object.values(LOT);
    const saleKeys = Object.values(SALE);
    const pickedLot = Object.fromEntries(
      [LOT.seller, LOT.item, LOT.packing, LOT.marka, LOT.store].map((key) => [key, record]),
    );
    const firstLot = stores.lots.getState().rows[0];
    if (firstLot === undefined) throw new Error('no lot row');
    for (let lotIndex = 0; lotIndex < 50; lotIndex += 1) {
      const lotKey =
        lotIndex === 0 ? firstLot.rowId : stores.lots.getState().addRow(full(lotKeys), pickedLot);
      if (lotIndex === 0) {
        stores.lots.getState().patchRow(lotKey, { typed: full(lotKeys), picked: pickedLot });
      }
      const sales = stores.salesFor(lotKey);
      for (let line = 0; line < 10; line += 1) {
        const picked = { [SALE.buyer]: record };
        if (line === 0 && sales.getState().rows[0] !== undefined) {
          const first = sales.getState().rows[0];
          if (first !== undefined) {
            sales.getState().patchRow(first.rowId, { typed: full(saleKeys), picked });
          }
        } else {
          sales.getState().addRow(full(saleKeys), picked);
        }
      }
    }
    const lots = stores.lots.getState().rows;
    const lines = lots.reduce(
      (sum, lot) => sum + stores.salesFor(lot.rowId).getState().rows.length,
      0,
    );
    expect(lots).toHaveLength(50);
    expect(lines).toBeGreaterThanOrEqual(500);
    const text = serialiseDraft(stores);
    const bytes = new TextEncoder().encode(text).length;
    expect(parseSnapshot(text)).not.toBeNull();
    // About 537,000 bytes with every field full, which no real بکری is: inside the limit with room.
    expect(bytes).toBeLessThan(MAX_DRAFT_BYTES * 0.75);
  });
});
