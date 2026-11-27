// Plan 97 item 2 (PRD FR-12.52, FR-7.14, Code Standards F27): a screen can refuse a new row, and every key that makes one
// asks it, in `shared/grid`, so no screen writes its own key handler. The sale book refuses a new line on a sold out lot.
// With the keyboard only, on the shared entry grid, in the Urdu and the English layout:
// - `↓` on a cell of the last row adds no row and the cursor stays;
// - `Enter` on the last Enter stop adds no row and the cursor stays;
// - `↓` on the trash cell of the last row (a grid with no "+" button) adds no row;
// - the screen is told which cell the key was pressed on, so it can say why;
// - when the screen allows it, each key adds the row as before (nothing else changed).

import { waitFor } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { LANGUAGES } from '../../src/i18n';
import type { GridInterceptors } from '../../src/shared/grid';
import { renderGrid, type GridRig } from '../helpers/entry-grid';

const ROWS = [
  { buyer: '1', qty: '10', rate: '100' },
  { buyer: '2', qty: '20', rate: '200' },
];

const rowCount = () => document.querySelectorAll('.ag-row[row-index]').length;
const lastRowId = (rig: GridRig) => rig.store.getState().rows.at(-1)?.rowId;

describe.each(LANGUAGES)('a screen that refuses a new row (%s)', (language) => {
  // In Urdu the text reads on to the left, so ← is "toward the end"; in English it is →.
  const toEnd = language === 'ur' ? '{ArrowLeft}' : '{ArrowRight}';

  it('↓ on the last row adds no row, keeps the cursor, and tells the screen the cell', async () => {
    const onNewRow = vi.fn<NonNullable<GridInterceptors['onNewRow']>>(() => true);
    const rig = await renderGrid(language, { rows: ROWS, interceptors: { onNewRow } });
    await rig.focusCell(1, 'qty');
    await rig.user.keyboard('{ArrowDown}');
    expect(rowCount()).toBe(2);
    expect(rig.focused()).toBe('1:qty');
    expect(onNewRow).toHaveBeenCalledTimes(1);
    expect(onNewRow).toHaveBeenCalledWith({ rowId: lastRowId(rig), field: 'qty' });
    // Again: asked again, still nothing added.
    await rig.user.keyboard('{ArrowDown}');
    expect(onNewRow).toHaveBeenCalledTimes(2);
    expect(rowCount()).toBe(2);
  });

  it('↓ on a row that is not the last one still moves down, and does not ask', async () => {
    const onNewRow = vi.fn(() => true);
    const rig = await renderGrid(language, { rows: ROWS, interceptors: { onNewRow } });
    await rig.focusCell(0, 'qty');
    await rig.user.keyboard('{ArrowDown}');
    expect(rig.focused()).toBe('1:qty');
    expect(onNewRow).not.toHaveBeenCalled();
  });

  it('Enter on the last Enter stop of the last row adds no row and keeps the cursor', async () => {
    const onNewRow = vi.fn<NonNullable<GridInterceptors['onNewRow']>>(() => true);
    const rig = await renderGrid(language, { rows: ROWS, interceptors: { onNewRow } });
    // Buyer, quantity and rate are the stops: Enter on the rate of the last row would make the next row.
    await rig.focusCell(1, 'rate');
    await rig.user.keyboard('{Enter}');
    expect(rowCount()).toBe(2);
    expect(onNewRow).toHaveBeenCalledTimes(1);
    expect(onNewRow).toHaveBeenCalledWith({ rowId: lastRowId(rig), field: 'rate' });
    // Enter on a stop that is not the last one still goes on to the next cell.
    await rig.focusCell(1, 'buyer');
    await rig.user.keyboard('{Enter}');
    expect(rig.focused()).toBe('1:qty');
    expect(onNewRow).toHaveBeenCalledTimes(1);
  });

  it('↓ on the trash cell of the last row, with no "+" button under the grid, adds no row', async () => {
    const onNewRow = vi.fn<NonNullable<GridInterceptors['onNewRow']>>(() => true);
    const rig = await renderGrid(language, { rows: ROWS, interceptors: { onNewRow } });
    await rig.focusCell(1, 'line');
    await rig.user.keyboard(toEnd);
    await waitFor(() => {
      expect(rig.focused()).toBe('1:__remove');
    });
    await rig.user.keyboard('{ArrowDown}');
    expect(rowCount()).toBe(2);
    expect(onNewRow).toHaveBeenCalledTimes(1);
  });

  it('when the screen allows it, ↓ and Enter add the row as before', async () => {
    const onNewRow = vi.fn(() => false);
    const rig = await renderGrid(language, { rows: ROWS, interceptors: { onNewRow } });
    await rig.focusCell(1, 'qty');
    await rig.user.keyboard('{ArrowDown}');
    await waitFor(() => {
      expect(rowCount()).toBe(3);
    });
    expect(rig.focused()).toBe('2:qty');
    await rig.focusCell(2, 'rate');
    await rig.user.keyboard('{Enter}');
    await waitFor(() => {
      expect(rowCount()).toBe(4);
    });
    expect(onNewRow).toHaveBeenCalledTimes(2);
  });
});
