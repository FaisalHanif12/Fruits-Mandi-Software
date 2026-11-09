// The end of an edit writes the store whatever the grid saw as a change (D8 plan 61, entry-grid skill,
// Code Standards F39): the store is the one truth of what was typed. AG Grid writes a finished cell
// through the column's `valueSetter` only when it thinks the value changed; `syncEditedText` makes the
// store hold the editor's text even when it did not. A cancelled edit has no text and is left alone.

import { describe, expect, it } from 'vitest';
import { createEntryStore, syncEditedText, type EntryColumn } from '../../src/shared/grid';

const COLUMNS: readonly EntryColumn[] = [
  { field: 'qty', type: 'qty', headerId: 'glossary-nag' },
  { field: 'rate', type: 'rate', headerId: 'glossary-rate-farq' },
  { field: 'buyer', type: 'picker', headerId: 'glossary-gahak' },
  { field: 'pay', type: 'choice', headerId: 'glossary-udhari' },
  { field: 'total', type: 'calc', headerId: 'glossary-meezan' },
];

/** The part of the grid's event the function reads: the row, the column and the editor's text. */
function endOf(rowId: string, field: string, newValue: unknown) {
  return {
    data: { rowId, typed: {} },
    column: { getColId: () => field },
    newValue,
  } as unknown as Parameters<typeof syncEditedText>[2];
}

describe('syncEditedText', () => {
  it('writes the editor text when the store holds something else, as when the grid saw no change', () => {
    const store = createEntryStore([{ rate: '' }]);
    const rowId = store.getState().rows[0]?.rowId ?? '';
    syncEditedText(store, COLUMNS, endOf(rowId, 'rate', '900'));
    expect(store.getState().rows[0]?.typed['rate']).toBe('900');
  });

  it('keeps the same text as it is: no second write', () => {
    const store = createEntryStore([{ rate: '900' }]);
    const rowId = store.getState().rows[0]?.rowId ?? '';
    let writes = 0;
    store.subscribe(() => {
      writes += 1;
    });
    syncEditedText(store, COLUMNS, endOf(rowId, 'rate', '900'));
    expect(writes).toBe(0);
  });

  it('leaves a cancelled edit alone: it has no text, so an old value never comes back over a newer one', () => {
    const store = createEntryStore([{ rate: '750' }]);
    const rowId = store.getState().rows[0]?.rowId ?? '';
    syncEditedText(store, COLUMNS, endOf(rowId, 'rate', undefined));
    expect(store.getState().rows[0]?.typed['rate']).toBe('750');
  });

  it('does not touch a cell whose editor writes the store itself, or a calculated cell', () => {
    const store = createEntryStore([{ buyer: 'x', pay: 'cash', total: '' }]);
    const rowId = store.getState().rows[0]?.rowId ?? '';
    for (const field of ['buyer', 'pay', 'total']) {
      syncEditedText(store, COLUMNS, endOf(rowId, field, 'typed text'));
    }
    expect(store.getState().rows[0]?.typed).toEqual({ buyer: 'x', pay: 'cash', total: '' });
  });

  it('does nothing for a row that has gone (a removed line)', () => {
    const store = createEntryStore([{ qty: '5' }]);
    syncEditedText(store, COLUMNS, endOf('gone', 'qty', '9'));
    expect(store.getState().rows[0]?.typed['qty']).toBe('5');
  });

  it('writes an empty text too: a cell the user cleared', () => {
    const store = createEntryStore([{ qty: '5' }]);
    const rowId = store.getState().rows[0]?.rowId ?? '';
    syncEditedText(store, COLUMNS, endOf(rowId, 'qty', ''));
    expect(store.getState().rows[0]?.typed['qty']).toBe('');
  });
});
