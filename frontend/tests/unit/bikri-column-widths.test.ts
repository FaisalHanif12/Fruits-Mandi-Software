// Nothing scrolls sideways, not even inside a grid (NFR-USE-15, Code Standards F20). Each grid of
// the sale book asks for a least width per column (`minWidth`, or the least of its type). The sum
// of those must fit the room the grid has in the smallest window of each layout step, with a
// classic 17 px scrollbar on the screen and the 2 px border of the grid. The rooms are measured in
// a browser (the grid of the lines is the width of the page less the menus):
//   compact (1024 px): 694 px, normal (1366 px): 876 px, wide (1920 px): 948 px.

import { describe, expect, it } from 'vitest';
import { allLinesColumns, lotColumns, saleColumns } from '../../src/features/bikri/columns';
import { FIT_MIN_WIDTHS, type EntryColumn } from '../../src/shared/grid';
import type { LayoutStep } from '../../src/shared/hooks';

const SCROLLBAR = 17;
const BORDER = 2;
/** The width of the grid in the smallest window of each step, without a scrollbar. */
const ROOM: Record<LayoutStep, number> = { compact: 694, normal: 876, wide: 948 };

function leastWidth(columns: readonly EntryColumn[]): number {
  return columns.reduce((sum, column) => sum + (column.minWidth ?? FIT_MIN_WIDTHS[column.type]), 0);
}

const STEPS: readonly LayoutStep[] = ['compact', 'normal', 'wide'];

describe.each(STEPS)('the grids of the sale book in a %s window', (step) => {
  const room = ROOM[step] - SCROLLBAR - BORDER;

  it('the sale lines of one lot fit without a sideways scroll', () => {
    expect(leastWidth(saleColumns(step))).toBeLessThanOrEqual(room);
  });

  it('the lots fit', () => {
    expect(leastWidth(lotColumns(step))).toBeLessThanOrEqual(room);
  });

  it('the lines of all lots, with their lot column, fit', () => {
    expect(leastWidth(allLinesColumns(step))).toBeLessThanOrEqual(room);
  });

  it('the whole-sale view starts with the lot column and keeps the buyer, qty and rate stops', () => {
    const fields = allLinesColumns(step).map((column) => column.field);
    expect(fields.slice(0, 4)).toEqual(['lot', 'buyer', 'qty', 'rate']);
  });
});

describe('the payment type is not a column of the grid (it is in the details of the line)', () => {
  it.each(STEPS)(
    '%s: no payment column in the grid of one lot or in the whole-sale view',
    (step) => {
      expect(saleColumns(step).some((column) => column.field === 'payment')).toBe(false);
      expect(allLinesColumns(step).some((column) => column.field === 'payment')).toBe(false);
    },
  );
});

describe('the quality column (درجہ) is right after the rate and never an Enter stop', () => {
  it.each(STEPS)('%s: grid of one lot', (step) => {
    const columns = saleColumns(step);
    const at = columns.findIndex((column) => column.field === 'rate');
    expect(columns[at + 1]?.field).toBe('grade');
    expect(columns[at + 1]?.type).toBe('select');
    expect(columns[at + 1]?.enterStop).toBe(false);
  });

  it.each([
    ['compact', false],
    ['normal', true],
    ['wide', true],
  ] as const)('%s: in the whole-sale view: %s', (step, kept) => {
    expect(allLinesColumns(step).some((column) => column.field === 'grade')).toBe(kept);
  });
});
