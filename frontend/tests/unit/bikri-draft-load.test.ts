// A draft that came from the backend, put into the stores and read back (plan 16): the draft of a
// posting open for edit and the draft an undo gives back. Putting a draft in and building it again
// must give the same draft, with the ids of the saved lot and lines, so an edit that changes
// nothing sends nothing different (PRD FR-12.20). Then the comparison with the loaded text: what
// is changed, added or removed (FR-12.18), and what is not (blank rows, cells the app keeps).

import { describe, expect, it } from 'vitest';
import type { BikriDraft } from '../../src/bindings/BikriDraft';
import { buildDraft } from '../../src/features/bikri/draft';
import { applyBackendDraft, type DraftRecords } from '../../src/features/bikri/draft-load';
import { createBikriDraft } from '../../src/features/bikri/draft-store';
import { captureBaseline, diffEdit } from '../../src/features/bikri/edit-changes';
import { LOT, SALE } from '../../src/features/bikri/types';
import type { PickedRecord } from '../../src/shared/grid';
import { editable } from '../helpers/bikri-plan16';
import { OPEN_FOR_EDIT, UNDO } from '../helpers/bikri-plan16-real';

/** Every id of the Event 1 draft found as a record named after its id. */
const RECORDS: DraftRecords = {
  account: (id) =>
    id === null
      ? null
      : { id, code: `c${String(id)}`, name_ur: `ur${String(id)}`, name_en: `en${String(id)}` },
  master: (scope, id) =>
    id === null
      ? null
      : {
          id,
          code: `${scope}${String(id)}`,
          name_ur: `ur-${scope}-${String(id)}`,
          name_en: `en-${scope}-${String(id)}`,
        },
};

function loaded(draft: BikriDraft = editable().draft) {
  const stores = createBikriDraft('10-10-2026');
  applyBackendDraft(stores, draft, RECORDS);
  return stores;
}

describe('a draft from the backend in the stores', () => {
  it('builds back the same draft, with the ids of the lot and of the lines', () => {
    const draft = editable().draft;
    expect(buildDraft(loaded(draft))).toEqual(draft);
  });

  it('the real draft of the backend (open for edit) goes back unchanged', () => {
    expect(buildDraft(loaded(OPEN_FOR_EDIT.draft))).toEqual(OPEN_FOR_EDIT.draft);
  });

  it('the real draft of an undo (no ids, `null` in the file) builds back to the same values', () => {
    // The file has `lot_id: null` and `sale_id: null`; the screen leaves an id out when it has none.
    const withoutIds = structuredClone(UNDO.draft);
    for (const lot of withoutIds.lots) {
      delete lot.lot_id;
      for (const sale of lot.sales) delete sale.sale_id;
    }
    expect(buildDraft(loaded(UNDO.draft))).toEqual(withoutIds);
  });

  it('keys are the row ids, so the preview lands on the same rows', () => {
    const stores = loaded();
    expect(stores.lots.getState().rows.map((row) => row.rowId)).toEqual(['lot:40']);
    expect(
      stores
        .salesFor('lot:40')
        .getState()
        .rows.map((row) => row.rowId),
    ).toEqual(['sale:101', 'sale:102']);
  });

  it('the cells the app fills once are marked, so nothing is filled again', () => {
    const stores = loaded();
    const lot = stores.lots.getState().rows[0];
    expect(lot?.typed[LOT.sellerFilled]).toBe('1');
    expect(lot?.typed[LOT.memoryFilledFor]).toBe(String(lot?.picked?.[LOT.seller]?.id));
    expect(stores.salesFor('lot:40').getState().rows[0]?.typed[SALE.qtyFilled]).toBe('1');
  });

  it('a typed charge comes back as a typed charge with its method', () => {
    const draft = editable().draft;
    const lot = draft.lots[0];
    if (lot === undefined) throw new Error('no lot');
    lot.labour = { method: 'per_unit', value: '5' };
    lot.munshiana = { method: 'manual', value: '50' };
    const back = buildDraft(loaded(draft)).lots[0];
    expect(back?.labour).toEqual({ method: 'per_unit', value: '5' });
    expect(back?.munshiana).toEqual({ method: 'manual', value: '50' });
  });

  it('a date the backend sends as YYYY-MM-DD is shown as DD-MM-YYYY', () => {
    const draft = editable().draft;
    draft.header.entry_date = '2026-10-10';
    expect(loaded(draft).header.getState().entryDate).toBe('10-10-2026');
  });
});

describe('what the user changed against the loaded text', () => {
  const salesOf = (stores: ReturnType<typeof loaded>) => stores.salesFor('lot:40');

  it('nothing is changed right after loading', () => {
    const stores = loaded();
    expect(diffEdit(stores, captureBaseline(stores)).unchanged).toBe(true);
  });

  it('a cell with other text is changed; the old text is kept for the list', () => {
    const stores = loaded();
    const baseline = captureBaseline(stores);
    salesOf(stores).getState().updateTyped('sale:101', SALE.rate, '610');
    const diff = diffEdit(stores, baseline);
    expect(diff.unchanged).toBe(false);
    expect(diff.items).toHaveLength(1);
    expect(diff.items[0]).toMatchObject({
      kind: 'changed',
      scope: 'line',
      lotNo: 1,
      lineNo: 1,
      field: SALE.rate,
      before: '600',
      after: '610',
    });
    expect(diff.saleCells['sale:101']?.[SALE.rate]).toEqual({ text: '600', record: null });
  });

  it('typing the old text back, or spaces around it, is no change', () => {
    const stores = loaded();
    const baseline = captureBaseline(stores);
    salesOf(stores).getState().updateTyped('sale:101', SALE.rate, '610');
    salesOf(stores).getState().updateTyped('sale:101', SALE.rate, ' 600 ');
    expect(diffEdit(stores, baseline).unchanged).toBe(true);
  });

  it('another record in a picker is a change, told by the record', () => {
    const stores = loaded();
    const baseline = captureBaseline(stores);
    const other: PickedRecord = { id: 99, code: 'x', name_ur: 'ur99', name_en: 'en99' };
    salesOf(stores).getState().updatePicked('sale:101', SALE.buyer, other);
    const diff = diffEdit(stores, baseline);
    expect(diff.items[0]).toMatchObject({ field: SALE.buyer });
    expect(diff.items[0]?.beforeRecord?.id).toBe(19);
    expect(diff.items[0]?.afterRecord?.id).toBe(99);
  });

  it('a line removed is listed; a line added is listed; a blank new row is not a change', () => {
    const stores = loaded();
    const baseline = captureBaseline(stores);
    salesOf(stores).getState().removeRow('sale:102');
    expect(diffEdit(stores, baseline).items).toMatchObject([
      { kind: 'removed', scope: 'line', lotNo: 1, lineNo: 2, field: null },
    ]);
    // A blank row: the grid makes one when Enter leaves the last line, and the app fills its qty.
    const blank = salesOf(stores)
      .getState()
      .addRow({ [SALE.qty]: '5', [SALE.qtyFilled]: '1' });
    expect(diffEdit(stores, baseline).items).toHaveLength(1);
    salesOf(stores).getState().updateTyped(blank, SALE.rate, '560');
    const added = diffEdit(stores, baseline).items.filter((item) => item.kind === 'added');
    expect(added.map((item) => item.field).sort()).toEqual([SALE.qty, SALE.rate].sort());
  });

  it('the header and the lot are compared too', () => {
    const stores = loaded();
    const baseline = captureBaseline(stores);
    stores.header.getState().patch({ truckNo: 'LHR-77' });
    stores.lots.getState().updateTyped('lot:40', LOT.freight, '3,500');
    const items = diffEdit(stores, baseline).items;
    expect(items.map((item) => `${item.scope}:${item.field ?? ''}`)).toEqual([
      'header:truckNo',
      `lot:${LOT.freight}`,
    ]);
  });
});
