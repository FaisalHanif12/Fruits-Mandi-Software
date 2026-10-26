// The sale lines of all lots as one list (PRD FR-12.1, the whole بکری). The store holds no data of
// its own: it shows the rows of the lots' stores and hands every change to the lot that owns the
// row. These tests use the draft stores directly, with no screen.

import { describe, expect, it } from 'vitest';
import { createBikriDraft, type BikriDraftStores } from '../../src/features/bikri/draft-store';
import type { EntryStore } from '../../src/shared/grid';

function rowIds(store: EntryStore): string[] {
  return store.getState().rows.map((row) => row.rowId);
}

function first(ids: readonly string[]): string {
  const id = ids[0];
  if (id === undefined) throw new Error('no row');
  return id;
}

/** Two lots; the first has two lines and the second one. Returns their keys and the row ids. */
function twoLots() {
  const stores: BikriDraftStores = createBikriDraft('10-10-2026');
  const lotA = first(rowIds(stores.lots));
  const lotB = (() => {
    stores.lots.getState().addRow();
    const ids = rowIds(stores.lots);
    const id = ids[1];
    if (id === undefined) throw new Error('no second lot');
    return id;
  })();
  stores.salesFor(lotA).getState().addRow();
  const linesA = rowIds(stores.salesFor(lotA));
  const linesB = rowIds(stores.salesFor(lotB));
  return { stores, lotA, lotB, linesA, linesB };
}

describe('the sale lines of all lots as one list', () => {
  it('lists the lines of the lots in lot order, and follows lines and lots added later', () => {
    const { stores, lotA, lotB, linesA, linesB } = twoLots();
    expect(rowIds(stores.all)).toEqual([...linesA, ...linesB]);
    const added = stores.salesFor(lotB).getState().addRow();
    expect(rowIds(stores.all)).toEqual([...linesA, ...linesB, added]);
    stores.salesFor(lotA).getState().removeRow(first(linesA));
    expect(rowIds(stores.all)).toEqual([...linesA.slice(1), ...linesB, added]);
  });

  it('hands a typed value to the lot that owns the line, and holds nothing itself', () => {
    const { stores, lotA, lotB, linesA, linesB } = twoLots();
    const inB = first(linesB);
    stores.all.getState().updateTyped(inB, 'qty', '25');
    expect(stores.salesFor(lotB).getState().rows[0]?.typed['qty']).toBe('25');
    expect(
      stores
        .salesFor(lotA)
        .getState()
        .rows.every((row) => row.typed['qty'] === undefined),
    ).toBe(true);
    // The list shows what the lot's store has, row for row.
    expect(stores.all.getState().rows.find((row) => row.rowId === inB)).toBe(
      stores.salesFor(lotB).getState().rows[0],
    );
    expect(linesA).toHaveLength(2);
  });

  it('hands picked records, patches, removals and "set where" to the owners', () => {
    const { stores, lotA, lotB, linesA, linesB } = twoLots();
    const buyer = { id: 19, code: '0020', name_ur: 'اکرم', name_en: 'Akram' };
    const a1 = first(linesA);
    const b1 = first(linesB);
    stores.all.getState().updatePicked(a1, 'buyer', buyer);
    expect(stores.salesFor(lotA).getState().rows[0]?.picked?.['buyer']).toEqual(buyer);
    stores.all.getState().patchRow(b1, { typed: { qty: '7', rate: '600' } });
    expect(stores.salesFor(lotB).getState().rows[0]?.typed).toMatchObject({
      qty: '7',
      rate: '600',
    });
    stores.all.getState().updateTypedWhere('payment', () => true, 'cash');
    for (const lot of [lotA, lotB]) {
      expect(
        stores
          .salesFor(lot)
          .getState()
          .rows.every((row) => row.typed['payment'] === 'cash'),
      ).toBe(true);
    }
    stores.all.getState().removeRow(a1);
    expect(rowIds(stores.salesFor(lotA))).toEqual(linesA.slice(1));
  });

  it('adds a line to the lot of the line that has focus, or to the last lot when none has', () => {
    const { stores, lotA, lotB, linesA } = twoLots();
    const addedToLast = stores.all.getState().addRow();
    expect(rowIds(stores.salesFor(lotB))).toContain(addedToLast);
    stores.all.getState().setFocus({ rowId: first(linesA), field: 'qty' });
    const addedToFirst = stores.all.getState().addRow();
    expect(rowIds(stores.salesFor(lotA))).toContain(addedToFirst);
  });

  it('tells the lot of a focused line and selects it, so the details and the card follow', () => {
    const { stores, lotA, lotB, linesB } = twoLots();
    stores.view.getState().select(lotA);
    const cell = { rowId: first(linesB), field: 'rate' };
    stores.all.getState().setFocus(cell);
    expect(stores.all.getState().focus).toEqual(cell);
    expect(stores.salesFor(lotB).getState().focus).toEqual(cell);
    expect(stores.view.getState().selectedLot).toBe(lotB);
  });

  it('asks the grid that is on screen to focus a line: the lot grid, or the whole grid by position', () => {
    const { stores, lotA, lotB } = twoLots();
    // One lot at a time: the request goes to the lot's own grid.
    stores.focusSaleLine(lotB, 0, 'buyer');
    expect(stores.salesFor(lotB).getState().pendingFocus).toEqual({ rowIndex: 0, field: 'buyer' });
    expect(stores.all.getState().pendingFocus).toBeNull();
    stores.salesFor(lotB).getState().setPendingFocus(null);
    // The whole view: line 0 of lot B comes after the two lines of lot A; "last" is its last line.
    stores.view.getState().setWhole(true);
    stores.focusSaleLine(lotB, 0, 'buyer');
    expect(stores.all.getState().pendingFocus).toEqual({ rowIndex: 2, field: 'buyer' });
    stores.focusSaleLine(lotA, 'last', 'qty');
    expect(stores.all.getState().pendingFocus).toEqual({ rowIndex: 1, field: 'qty' });
    expect(stores.salesFor(lotB).getState().pendingFocus).toBeNull();
  });

  it('does not redraw for a change that is not in a line (a key in the header)', () => {
    const { stores } = twoLots();
    const before = stores.all.getState().rows;
    stores.header.getState().patch({ truckNo: 'LES-1234' });
    expect(stores.all.getState().rows).toBe(before);
  });
});
