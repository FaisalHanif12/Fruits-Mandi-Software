// The arrows reach the trash cell of a row and the "+" button under the grid (Umar, plan 71 follow-up; PRD
// FR-7.13, FR-7.14, FR-7.3, NFR-USE-7, NFR-USE-8), on the shared entry grid, in Urdu and in English. With the
// keyboard only: the arrow that reads on from the last cell of a row lands on that row's trash cell (← in
// Urdu, → in English) and the other one goes back; Enter, Space and Ctrl+Delete on it remove the row with the
// same rules as the button (a question first when something is under the row); ↑ and ↓ go to the trash cell of
// the row above and below; ↓ on the LAST row's trash cell goes to the "+" button, Enter on it adds a row and
// puts the cursor on its first cell, and ↑ on it goes back; Enter never stops on the trash cell, and ↓ from any
// other cell of the last row still adds a row (FR-7.14). The book's own grids are tested in bikri-add-row.

import { screen, waitFor } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { LANGUAGES } from '../../src/i18n';
import { renderGrid, type GridRig } from '../helpers/entry-grid';
import { message, visible } from '../helpers/render';

const ROWS = [
  { buyer: '1', qty: '10', rate: '100' },
  { buyer: '2', qty: '20', rate: '200' },
  { buyer: '3', qty: '30', rate: '300' },
];

const rowCount = () => document.querySelectorAll('.ag-row[row-index]').length;
const buyers = (rig: GridRig) => rig.store.getState().rows.map((row) => row.typed['buyer']);
const focusedCellId = () =>
  document.querySelector('.ag-cell.ag-cell-focus')?.getAttribute('col-id') ?? null;

describe.each(LANGUAGES)('the arrows and the trash cell of a grid (%s)', (language) => {
  const m = (id: string, vars: Record<string, string | number> = {}) => message(language, id, vars);
  // In Urdu the text reads on to the left, so ← is "toward the end"; in English it is →.
  const toEnd = language === 'ur' ? '{ArrowLeft}' : '{ArrowRight}';
  const toStart = language === 'ur' ? '{ArrowRight}' : '{ArrowLeft}';

  it('the arrow that reads on from the last cell reaches the trash cell, and the other one goes back', async () => {
    const rig = await renderGrid(language, { rows: ROWS });
    await rig.focusCell(0, 'line');
    await rig.user.keyboard(toEnd);
    expect(rig.focused()).toBe('0:__remove');
    await rig.user.keyboard(toStart);
    expect(rig.focused()).toBe('0:line');
  });

  it('the arrow that reads on does nothing more once the cursor is on the trash cell', async () => {
    const rig = await renderGrid(language, { rows: ROWS });
    await rig.focusCell(1, 'line');
    await rig.user.keyboard(toEnd);
    await rig.user.keyboard(toEnd);
    expect(rig.focused()).toBe('1:__remove');
    expect(rowCount()).toBe(3);
  });

  it('shows the focus outline on the trash cell, and the grid keeps the cell the user typed in', async () => {
    const rig = await renderGrid(language, { rows: ROWS });
    await rig.focusCell(0, 'qty');
    // From the quantity, one arrow for each cell to the end of the row, and one more for the trash cell.
    for (let presses = 0; presses < 12 && focusedCellId() !== '__remove'; presses += 1) {
      await rig.user.keyboard(toEnd);
    }
    expect(focusedCellId()).toBe('__remove');
    // The store keeps the last cell the user typed in, not the trash cell (a language switch focuses that).
    expect(rig.store.getState().focus?.field).not.toBe('__remove');
  });

  it('Enter on the trash cell removes the row, as a click on the button does', async () => {
    const rig = await renderGrid(language, { rows: ROWS });
    await rig.focusCell(1, 'line');
    await rig.user.keyboard(toEnd);
    expect(rig.focused()).toBe('1:__remove');
    await rig.user.keyboard('{Enter}');
    await waitFor(() => {
      expect(rowCount()).toBe(2);
    });
    expect(buyers(rig)).toEqual(['1', '3']);
  });

  it('Space and Ctrl+Delete on the trash cell remove the row too', async () => {
    const rig = await renderGrid(language, { rows: ROWS });
    await rig.focusCell(0, 'line');
    await rig.user.keyboard(toEnd);
    await rig.user.keyboard(' ');
    await waitFor(() => {
      expect(rowCount()).toBe(2);
    });
    await rig.focusCell(0, 'line');
    await rig.user.keyboard(toEnd);
    expect(rig.focused()).toBe('0:__remove');
    await rig.user.keyboard('{Control>}{Delete}{/Control}');
    await waitFor(() => {
      expect(rowCount()).toBe(1);
    });
    expect(buyers(rig)).toEqual(['3']);
  });

  it('asks first when something is under the row, exactly as the button does', async () => {
    const rig = await renderGrid(language, {
      rows: ROWS,
      removeQuestion: (row) =>
        row.typed['buyer'] === '2'
          ? { messageId: 'bikri-lot-remove-question', vars: { count: 3 } }
          : null,
    });
    await rig.focusCell(1, 'line');
    await rig.user.keyboard(toEnd);
    await rig.user.keyboard('{Enter}');
    const box = await screen.findByRole('alertdialog');
    expect(visible(box.textContent)).toContain(m('bikri-lot-remove-question', { count: 3 }));
    expect(rowCount()).toBe(3);
    await rig.user.keyboard('{Escape}');
    await waitFor(() => {
      expect(screen.queryByRole('alertdialog')).toBeNull();
    });
    expect(buyers(rig)).toEqual(['1', '2', '3']);
  });

  it('↓ and ↑ on the trash cell go to the trash cell of the row below and above', async () => {
    const rig = await renderGrid(language, { rows: ROWS });
    await rig.focusCell(0, 'line');
    await rig.user.keyboard(toEnd);
    await rig.user.keyboard('{ArrowDown}');
    expect(rig.focused()).toBe('1:__remove');
    await rig.user.keyboard('{ArrowDown}');
    expect(rig.focused()).toBe('2:__remove');
    await rig.user.keyboard('{ArrowUp}');
    expect(rig.focused()).toBe('1:__remove');
    await rig.user.keyboard(toStart);
    expect(rig.focused()).toBe('1:line');
  });

  it('a row without a trash button is not a stop: the arrow stops at its last cell, and ↓ from above lands on it', async () => {
    const rig = await renderGrid(language, {
      rows: ROWS,
      canRemoveRow: (row) => row.typed['buyer'] !== '2',
    });
    await rig.focusCell(1, 'line');
    await rig.user.keyboard(toEnd);
    expect(rig.focused()).toBe('1:line');
    await rig.focusCell(0, 'line');
    await rig.user.keyboard(toEnd);
    expect(rig.focused()).toBe('0:__remove');
    await rig.user.keyboard('{ArrowDown}');
    expect(rig.focused()).toBe('1:line');
  });

  it('a grid with fixed rows has no trash cell: the arrow stays on its last cell', async () => {
    const rig = await renderGrid(language, { rows: ROWS, fixedRows: true });
    await rig.focusCell(0, 'line');
    await rig.user.keyboard(toEnd);
    expect(rig.focused()).toBe('0:line');
  });

  it('Enter never stops on the trash cell, and ↓ from any other cell of the last row still adds a row', async () => {
    const rig = await renderGrid(language, { rows: ROWS });
    await rig.focusCell(2, 'rate');
    await rig.user.keyboard('{Enter}');
    // The last stop of the last row: Enter adds a row and goes to its first stop, never to the trash.
    await waitFor(() => {
      expect(rowCount()).toBe(4);
    });
    expect(rig.focused()).toBe('3:buyer');
    await rig.focusCell(3, 'qty');
    await rig.user.keyboard('{ArrowDown}');
    await waitFor(() => {
      expect(rowCount()).toBe(5);
    });
    expect(rig.focused()).toBe('4:qty');
  });

  it('without a "+" button, ↓ on the last row’s trash cell adds a row like ↓ on any other cell', async () => {
    const rig = await renderGrid(language, { rows: ROWS });
    await rig.focusCell(2, 'line');
    await rig.user.keyboard(toEnd);
    expect(rig.focused()).toBe('2:__remove');
    await rig.user.keyboard('{ArrowDown}');
    await waitFor(() => {
      expect(rowCount()).toBe(4);
    });
  });

  describe('with a "+" button under the grid', () => {
    const plus = () =>
      screen.getByRole('button', { name: (name) => visible(name) === m('bikri-sales-add-row') });

    it('↓ on the last row’s trash cell goes to the "+", Enter adds a row and the cursor lands on its first cell, ↑ comes back', async () => {
      const rig = await renderGrid(language, { rows: ROWS, withAddButton: true });
      await rig.focusCell(2, 'line');
      await rig.user.keyboard(toEnd);
      expect(rig.focused()).toBe('2:__remove');
      await rig.user.keyboard('{ArrowDown}');
      expect(plus()).toHaveFocus();
      // Nothing was added by the arrow itself.
      expect(rowCount()).toBe(3);
      await rig.user.keyboard('{ArrowUp}');
      expect(rig.focused()).toBe('2:__remove');
      await rig.user.keyboard('{ArrowDown}');
      expect(plus()).toHaveFocus();
      await rig.user.keyboard('{Enter}');
      await waitFor(() => {
        expect(rowCount()).toBe(4);
      });
      await waitFor(() => {
        expect(rig.focused()).toBe('3:buyer');
      });
    });

    it('↓ from the trash cell of a row that is not the last goes to the next row, not to the "+"', async () => {
      const rig = await renderGrid(language, { rows: ROWS, withAddButton: true });
      await rig.focusCell(0, 'line');
      await rig.user.keyboard(toEnd);
      await rig.user.keyboard('{ArrowDown}');
      expect(rig.focused()).toBe('1:__remove');
      expect(plus()).not.toHaveFocus();
    });

    it('↓ from any other cell of the last row still adds a row and does not go to the "+"', async () => {
      const rig = await renderGrid(language, { rows: ROWS, withAddButton: true });
      await rig.focusCell(2, 'qty');
      await rig.user.keyboard('{ArrowDown}');
      await waitFor(() => {
        expect(rowCount()).toBe(4);
      });
      expect(plus()).not.toHaveFocus();
    });

    it('Space on the "+" adds a row too, and the other arrows are left to the button', async () => {
      const rig = await renderGrid(language, { rows: ROWS, withAddButton: true });
      plus().focus();
      await rig.user.keyboard('{ArrowDown}');
      expect(plus()).toHaveFocus();
      await rig.user.keyboard(' ');
      await waitFor(() => {
        expect(rowCount()).toBe(4);
      });
    });
  });
});
