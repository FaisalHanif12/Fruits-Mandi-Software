// The tick "پیشگی اس بکری سے کاٹیں" of a seller (PRD FR-12.23, plan 78 C3, Umar 17-11-2026): it is off when a
// بکری opens, one tick covers every lot of that seller, a lot added after the tick takes it too, a saved lot
// keeps the tick it was saved with until the munshi changes it, and a lot of own goods, of a garden, or one
// carried from an earlier بکری is never changed by it. These are the rules on the text of the lot rows (Code
// Standards F39); the backend still takes one tick for each lot, so what each lot sends is checked here.

import { describe, expect, it } from 'vitest';
import { isSellerTicked, setSellerTick } from '../../src/features/bikri/advance-tick';
import { adjustAdvanceOf, lotInput, tickLotsOf } from '../../src/features/bikri/draft';
import { LOT, ON } from '../../src/features/bikri/types';
import type { EntryRow, PickedRecord } from '../../src/shared/grid';

const seller = (id: number, code = `001${String(id)}`): PickedRecord => ({
  id,
  code,
  name_ur: 'رحیم',
  name_en: 'Rahim',
});

/** The row at a place; a test that has none is wrong. */
function at(rows: readonly EntryRow[], index: number): EntryRow {
  const row = rows[index];
  if (row === undefined) throw new Error(`no lot row at ${String(index)}`);
  return row;
}

let next = 0;
/** A lot row of a seller, with the text typed on it. */
function lot(sellerRecord: PickedRecord, typed: Record<string, string> = {}): EntryRow {
  next += 1;
  return { rowId: `lot-${String(next)}`, typed, picked: { [LOT.seller]: sellerRecord } };
}

const RAHIM = seller(1);
const KARIM = seller(2);
const OWN = seller(3, '030119998');
const GARDEN = seller(4, '0081');

/** Applies a tick change the way the screen does: a row's text is changed by its row id. */
function applyTick(rows: EntryRow[], sellerId: number, on: boolean): EntryRow[] {
  const changed = new Map<string, string>();
  setSellerTick(rows, sellerId, on, (rowId, _field, text) => {
    changed.set(rowId, text);
  });
  return rows.map((row) =>
    changed.has(row.rowId)
      ? { ...row, typed: { ...row.typed, [LOT.adjustAdvance]: changed.get(row.rowId) ?? '' } }
      : row,
  );
}

describe('the seller’s advance tick', () => {
  it('is off when a بکری opens: no lot sends it', () => {
    const rows = [lot(RAHIM), lot(KARIM)];
    expect(isSellerTicked(rows, 1)).toBe(false);
    expect(rows.map((row) => adjustAdvanceOf(row, rows))).toEqual([false, false]);
    expect(lotInput(at(rows, 0), [], false).adjust_advance).toBe(false);
  });

  it('one tick covers every lot of that seller and no lot of another seller', () => {
    const rows = applyTick([lot(RAHIM), lot(KARIM), lot(RAHIM)], 1, true);
    expect(isSellerTicked(rows, 1)).toBe(true);
    expect(isSellerTicked(rows, 2)).toBe(false);
    expect(rows.map((row) => adjustAdvanceOf(row, rows))).toEqual([true, false, true]);
  });

  it('unticking turns it off on every lot of that seller', () => {
    const on = applyTick([lot(RAHIM), lot(RAHIM)], 1, true);
    const off = applyTick(on, 1, false);
    expect(isSellerTicked(off, 1)).toBe(false);
    expect(off.map((row) => adjustAdvanceOf(row, off))).toEqual([false, false]);
  });

  it('a lot added after the tick takes it: the seller is ticked, so the munshi never ticks again', () => {
    const ticked = applyTick([lot(RAHIM)], 1, true);
    const added = [...ticked, lot(RAHIM)];
    expect(adjustAdvanceOf(at(added, 1), added)).toBe(true);
    expect(isSellerTicked(added, 1)).toBe(true);
  });

  it('a lot of own goods, of a garden, or carried from an earlier بکری has no tick of the seller’s to take', () => {
    const own = lot(OWN, { [LOT.adjustAdvance]: ON });
    const garden = lot(GARDEN, { [LOT.adjustAdvance]: ON });
    const carried = lot(RAHIM, { [LOT.carried]: ON, [LOT.lotId]: '7', [LOT.adjustAdvance]: ON });
    const rows = [own, garden, carried];
    expect(rows.map((row) => adjustAdvanceOf(row, rows))).toEqual([false, false, false]);
    expect(tickLotsOf(rows, 1)).toEqual([]);
    // Ticking the seller does not touch the carried lot: it keeps the tick it was saved with.
    const after = applyTick([carried, lot(RAHIM)], 1, true);
    expect(after[0]?.typed[LOT.adjustAdvance]).toBe(ON);
    expect(after[1]?.typed[LOT.adjustAdvance]).toBe(ON);
    const untouched = applyTick(
      [{ ...carried, typed: { ...carried.typed, [LOT.adjustAdvance]: '0' } }, lot(RAHIM)],
      1,
      true,
    );
    expect(untouched[0]?.typed[LOT.adjustAdvance]).toBe('0');
  });

  it('a saved lot keeps the tick it was saved with, and a new lot of the seller takes the tick the summary shows', () => {
    const savedOff = lot(RAHIM, { [LOT.lotId]: '5', [LOT.adjustAdvance]: '0' });
    const savedOn = lot(RAHIM, { [LOT.lotId]: '6', [LOT.adjustAdvance]: ON });
    const fresh = lot(RAHIM);
    const rows = [savedOff, savedOn, fresh];
    // The summary shows the seller ticked because a lot of his has it on, so the new lot is covered too (plan 80
    // R2-4, Q-R2-3). Each saved lot still sends the tick it was saved with, until the munshi changes it.
    expect(isSellerTicked(rows, 1)).toBe(true);
    expect(rows.map((row) => adjustAdvanceOf(row, rows))).toEqual([false, true, true]);
  });

  it('in an edit, a seller whose saved lot is ticked gets a new lot that takes the advance too (R2-4)', () => {
    const saved = lot(RAHIM, { [LOT.lotId]: '5', [LOT.adjustAdvance]: ON });
    const added = lot(RAHIM);
    const rows = [saved, added];
    // What the lot sends is what the summary shows: ticked, so the new lot sends true, not false.
    expect(isSellerTicked(rows, 1)).toBe(true);
    expect(adjustAdvanceOf(added, rows)).toBe(true);
    expect(lotInput(added, [], adjustAdvanceOf(added, rows)).adjust_advance).toBe(true);
    // A seller whose saved lot is not ticked shows no tick, and the new lot sends none.
    const savedOff = lot(RAHIM, { [LOT.lotId]: '6', [LOT.adjustAdvance]: '0' });
    const addedOff = lot(RAHIM);
    const off = [savedOff, addedOff];
    expect(isSellerTicked(off, 1)).toBe(false);
    expect(adjustAdvanceOf(addedOff, off)).toBe(false);
  });

  it('in an edit the tick changes every lot of the seller, saved or new, and the edit sends it', () => {
    const rows = [lot(RAHIM, { [LOT.lotId]: '5', [LOT.adjustAdvance]: '0' }), lot(RAHIM)];
    const on = applyTick(rows, 1, true);
    expect(on.map((row) => adjustAdvanceOf(row, on))).toEqual([true, true]);
    const off = applyTick(on, 1, false);
    expect(off.map((row) => adjustAdvanceOf(row, off))).toEqual([false, false]);
  });

  it('a lot with nobody picked yet sends what is typed on it', () => {
    const empty: EntryRow = { rowId: 'x', typed: { [LOT.adjustAdvance]: ON } };
    expect(adjustAdvanceOf(empty, [empty])).toBe(true);
    const none: EntryRow = { rowId: 'y', typed: {} };
    expect(adjustAdvanceOf(none, [none])).toBe(false);
  });
});

describe('what a lot sends as its tick', () => {
  it('lotInput sends the tick it is given, and always false for own goods and for a garden’s lot', () => {
    const rows = [lot(RAHIM)];
    expect(lotInput(at(rows, 0), [], true).adjust_advance).toBe(true);
    expect(lotInput(at(rows, 0), [], false).adjust_advance).toBe(false);
    const own = lot(OWN);
    expect(lotInput(own, [], true).adjust_advance).toBe(false);
    const garden = lot(GARDEN);
    expect(lotInput(garden, [], true).adjust_advance).toBe(false);
  });

  it('with no tick given, a lot sends what is typed on it: off unless it is ON', () => {
    expect(lotInput(lot(RAHIM), []).adjust_advance).toBe(false);
    expect(lotInput(lot(RAHIM, { [LOT.adjustAdvance]: ON }), []).adjust_advance).toBe(true);
    expect(lotInput(lot(RAHIM, { [LOT.adjustAdvance]: '0' }), []).adjust_advance).toBe(false);
  });
});
