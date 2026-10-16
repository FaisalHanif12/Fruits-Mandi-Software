// The shared entry grid, key by key, in both layouts (PRD FR-7.13 to FR-7.20, entry-grid skill
// table). In Urdu the grid is drawn right to left, so the first column is on the right and
// ← moves to the next column; in English it is the other way round.

import { act, waitFor } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { LANGUAGES, type Language } from '../../src/i18n';
import { registerEscape } from '../../src/shared/keyboard';
import { useLanguageStore } from '../../src/shared/i18n';
import { renderGrid, type GridRig } from '../helpers/entry-grid';

const RIGHT = '{ArrowRight}';
const LEFT = '{ArrowLeft}';

/** The key that moves to the next column in reading order: → in English, ← in Urdu. */
const forward = (language: Language) => (language === 'ur' ? LEFT : RIGHT);
const backward = (language: Language) => (language === 'ur' ? RIGHT : LEFT);

function rowsCount(): number {
  return document.querySelectorAll('.ag-row[row-index]').length;
}

describe.each(LANGUAGES)('entry grid (%s)', (language) => {
  describe('typing', () => {
    it('any digit starts typing and replaces the cell; Enter keeps it', async () => {
      const rig = await renderGrid(language, { rows: [{ qty: '50' }] });
      await rig.focusCell(0, 'qty');
      await rig.user.keyboard('2');
      expect(rig.isEditing()).toBe(true);
      await rig.user.keyboard('0{Enter}');
      expect(rig.typed(0, 'qty')).toBe('20');
      expect(rig.isEditing()).toBe(false);
    });

    it('Enter and F2 do not start typing; F2 stays free for "New"', async () => {
      const rig = await renderGrid(language, { rows: [{ qty: '50' }] });
      await rig.focusCell(0, 'buyer');
      await rig.user.keyboard('{F2}');
      expect(rig.isEditing()).toBe(false);
      expect(rig.focused()).toBe('0:buyer');
    });

    // The app fills another cell of the row a moment after the user started a cell: the remaining
    // qty arrives with the answer of the preview (PRD FR-7.52). The grid must not close the cell
    // he is typing in. No timing here: the fill is put into the store while the cell is open.
    it('a fill that lands in another cell of the row does not close the cell being typed in', async () => {
      const rig = await renderGrid(language, { rows: [{}] });
      await rig.focusCell(0, 'buyer');
      await rig.user.keyboard('7');
      expect(rig.isEditing()).toBe(true);
      const rowId = rig.store.getState().rows[0]?.rowId ?? '';
      act(() => {
        rig.store.getState().updateTyped(rowId, 'qty', '30');
      });
      await waitFor(() => {
        expect(rig.typed(0, 'qty')).toBe('30');
      });
      expect(rig.isEditing()).toBe(true);
      await rig.user.keyboard('1{Enter}');
      expect(rig.typed(0, 'buyer')).toBe('71');
      expect(rig.typed(0, 'qty')).toBe('30');
    });

    it('a fill that lands in a number cell does not close the number cell being typed in', async () => {
      const rig = await renderGrid(language, { rows: [{}] });
      await rig.focusCell(0, 'rate');
      await rig.user.keyboard('6');
      expect(rig.isEditing()).toBe(true);
      const rowId = rig.store.getState().rows[0]?.rowId ?? '';
      act(() => {
        rig.store.getState().updateTyped(rowId, 'qty', '30');
      });
      await waitFor(() => {
        expect(rig.typed(0, 'qty')).toBe('30');
      });
      expect(rig.isEditing()).toBe(true);
      await rig.user.keyboard('00{Enter}');
      expect(rig.typed(0, 'rate')).toBe('600');
    });

    it('a calculated cell cannot be typed in', async () => {
      const rig = await renderGrid(language, { rows: [{ qty: '50' }] });
      await rig.focusCell(0, 'line');
      await rig.user.keyboard('9');
      expect(rig.isEditing()).toBe(false);
    });
  });

  describe('Enter', () => {
    it('a sale line takes three fields and three Enters: buyer, qty, rate, then the next row', async () => {
      const rig = await renderGrid(language);
      await rig.focusCell(0, 'buyer');
      await rig.user.keyboard('1{Enter}');
      expect(rig.focused()).toBe('0:qty');
      await rig.user.keyboard('20{Enter}');
      expect(rig.focused()).toBe('0:rate');
      await rig.user.keyboard('600{Enter}');
      expect([rig.typed(0, 'buyer'), rig.typed(0, 'qty'), rig.typed(0, 'rate')]).toEqual([
        '1',
        '20',
        '600',
      ]);
      await waitFor(() => {
        expect(rig.focused()).toBe('1:buyer');
      });
      expect(rig.store.getState().rows).toHaveLength(2);
    });

    it('skips cells that are read-only on this row, and the editable cells that are not stops', async () => {
      const rig = await renderGrid(language, { rows: [{}, {}] });
      await rig.focusCell(0, 'rate');
      await rig.user.keyboard('{Enter}');
      // Buyer rate is closed on this row, the note and calculated cells are not stops.
      expect(rig.focused()).toBe('1:buyer');
    });

    it('stops on a cell that is open on this row, and not on the same cell on another row', async () => {
      const rig = await renderGrid(language, { rows: [{ different: 'y' }, {}] });
      await rig.focusCell(0, 'rate');
      await rig.user.keyboard('{Enter}');
      expect(rig.focused()).toBe('0:buyerRate');
      await rig.user.keyboard('{Enter}');
      expect(rig.focused()).toBe('1:buyer');
    });

    it('still stops on an editable cell the app filled, so the user sees it', async () => {
      const rig = await renderGrid(language, { rows: [{ qty: '30' }] });
      await rig.focusCell(0, 'buyer');
      await rig.user.keyboard('1{Enter}');
      expect(rig.focused()).toBe('0:qty');
      // Enter without typing keeps the filled value.
      await rig.user.keyboard('{Enter}');
      expect(rig.focused()).toBe('0:rate');
      expect(rig.typed(0, 'qty')).toBe('30');
    });

    it('finishes the cell and moves on when typing', async () => {
      const rig = await renderGrid(language, { rows: [{}] });
      await rig.focusCell(0, 'buyer');
      await rig.user.keyboard('7');
      expect(rig.isEditing()).toBe(true);
      await rig.user.keyboard('{Enter}');
      expect(rig.isEditing()).toBe(false);
      expect(rig.typed(0, 'buyer')).toBe('7');
      expect(rig.focused()).toBe('0:qty');
    });

    it('Ctrl+Enter is left to the screen (save and next), and does not move', async () => {
      const rig = await renderGrid(language, { rows: [{}] });
      await rig.focusCell(0, 'buyer');
      await rig.user.keyboard('{Control>}{Enter}{/Control}');
      expect(rig.focused()).toBe('0:buyer');
    });
  });

  describe('arrow keys when not typing', () => {
    it('↑ and ↓ move between rows, and ↓ on the last row adds a row', async () => {
      const rig = await renderGrid(language, { rows: [{}, {}] });
      await rig.focusCell(0, 'qty');
      await rig.user.keyboard('{ArrowDown}');
      expect(rig.focused()).toBe('1:qty');
      await rig.user.keyboard('{ArrowUp}');
      expect(rig.focused()).toBe('0:qty');
      await rig.user.keyboard('{ArrowDown}{ArrowDown}');
      await waitFor(() => {
        expect(rig.focused()).toBe('2:qty');
      });
      expect(rig.store.getState().rows).toHaveLength(3);
      expect(rowsCount()).toBe(3);
    });

    it('← and → move to the neighbouring cell as the user sees it', async () => {
      const rig = await renderGrid(language, { rows: [{}] });
      await rig.focusCell(0, 'qty');
      await rig.user.keyboard(forward(language));
      expect(rig.focused()).toBe('0:rate');
      await rig.user.keyboard(backward(language));
      await rig.user.keyboard(backward(language));
      expect(rig.focused()).toBe('0:buyer');
    });

    it('does nothing at the first and last column', async () => {
      const rig = await renderGrid(language, { rows: [{}] });
      await rig.focusCell(0, 'buyer');
      await rig.user.keyboard(backward(language));
      expect(rig.focused()).toBe('0:buyer');
      await rig.focusCell(0, 'line');
      await rig.user.keyboard(forward(language));
      expect(rig.focused()).toBe('0:line');
    });
  });

  describe('arrow keys while typing', () => {
    it('↓ finishes the cell and moves down', async () => {
      const rig = await renderGrid(language, { rows: [{}, {}] });
      await rig.focusCell(0, 'qty');
      await rig.user.keyboard('2{ArrowDown}');
      expect(rig.isEditing()).toBe(false);
      expect(rig.typed(0, 'qty')).toBe('2');
      expect(rig.focused()).toBe('1:qty');
    });

    it('↑ finishes the cell and moves up', async () => {
      const rig = await renderGrid(language, { rows: [{}, {}] });
      await rig.focusCell(1, 'qty');
      await rig.user.keyboard('5{ArrowUp}');
      expect(rig.typed(1, 'qty')).toBe('5');
      expect(rig.focused()).toBe('0:qty');
    });

    it('moves the text cursor first; at the edge of the text it finishes and moves on', async () => {
      const rig = await renderGrid(language, { rows: [{}] });
      await rig.focusCell(0, 'qty');
      // Numbers are written left to right in both layouts, so the right edge is the end.
      await rig.user.keyboard('123');
      await rig.user.keyboard(LEFT);
      expect(rig.isEditing()).toBe(true);
      await rig.user.keyboard(RIGHT);
      expect(rig.isEditing()).toBe(true);
      // The cursor is back at the end: the next → leaves, to the cell on the right of the screen.
      await rig.user.keyboard(RIGHT);
      expect(rig.isEditing()).toBe(false);
      expect(rig.typed(0, 'qty')).toBe('123');
      // On the right of the screen is the next column in English and the previous one in Urdu.
      expect(rig.focused()).toBe(language === 'ur' ? '0:buyer' : '0:rate');
    });

    it('the left edge of a number is its start, in both layouts', async () => {
      const rig = await renderGrid(language, { rows: [{}] });
      await rig.focusCell(0, 'qty');
      await rig.user.keyboard('12{Home}');
      await rig.user.keyboard(LEFT);
      expect(rig.isEditing()).toBe(false);
      expect(rig.typed(0, 'qty')).toBe('12');
      expect(rig.focused()).toBe(language === 'ur' ? '0:rate' : '0:buyer');
    });
  });

  describe('Esc', () => {
    it('while typing, cancels the typing and keeps the old value', async () => {
      const rig = await renderGrid(language, { rows: [{ qty: '50' }] });
      await rig.focusCell(0, 'qty');
      await rig.user.keyboard('999');
      expect(rig.isEditing()).toBe(true);
      await rig.user.keyboard('{Escape}');
      expect(rig.isEditing()).toBe(false);
      expect(rig.typed(0, 'qty')).toBe('50');
    });

    it('while typing, Esc does not leave the screen; the next Esc does (FR-7.18 order)', async () => {
      const leave = vi.fn();
      const remove = registerEscape('leave', leave);
      const rig = await renderGrid(language, { rows: [{ qty: '50' }] });
      await rig.focusCell(0, 'qty');
      await rig.user.keyboard('9{Escape}');
      expect(leave).not.toHaveBeenCalled();
      await rig.user.keyboard('{Escape}');
      expect(leave).toHaveBeenCalledTimes(1);
      remove();
    });
  });

  describe('rows', () => {
    it('Ctrl+Delete removes the current row and focuses the one that took its place', async () => {
      const rig = await renderGrid(language, { rows: [{ qty: '1' }, { qty: '2' }, { qty: '3' }] });
      await rig.focusCell(1, 'qty');
      await rig.user.keyboard('{Control>}{Delete}{/Control}');
      expect(rig.store.getState().rows.map((row) => row.typed['qty'])).toEqual(['1', '3']);
      await waitFor(() => {
        expect(rig.focused()).toBe('1:qty');
      });
    });

    it('Ctrl+Delete on the last row focuses the row above', async () => {
      const rig = await renderGrid(language, { rows: [{ qty: '1' }, { qty: '2' }] });
      await rig.focusCell(1, 'qty');
      await rig.user.keyboard('{Control>}{Delete}{/Control}');
      await waitFor(() => {
        expect(rig.focused()).toBe('0:qty');
      });
    });

    it('Ctrl+Delete on the only row leaves one empty row', async () => {
      const rig = await renderGrid(language, { rows: [{ qty: '1' }] });
      await rig.focusCell(0, 'qty');
      await rig.user.keyboard('{Control>}{Delete}{/Control}');
      expect(rig.store.getState().rows).toHaveLength(1);
      expect(rig.typed(0, 'qty')).toBeUndefined();
    });

    it('Ctrl+Delete while typing removes the row and drops the typing', async () => {
      const rig = await renderGrid(language, { rows: [{ qty: '1' }, { qty: '2' }] });
      await rig.focusCell(0, 'qty');
      await rig.user.keyboard('9{Control>}{Delete}{/Control}');
      expect(rig.store.getState().rows.map((row) => row.typed['qty'])).toEqual(['2']);
      expect(rig.isEditing()).toBe(false);
    });

    it('Ctrl+D copies the cell from the line above when the grid allows it', async () => {
      const rig = await renderGrid(language, {
        rows: [{ rate: '600' }, {}],
        copyAbove: true,
      });
      await rig.focusCell(1, 'rate');
      await rig.user.keyboard('{Control>}d{/Control}');
      expect(rig.typed(1, 'rate')).toBe('600');
    });

    it('Ctrl+D does nothing in a grid that does not allow it, and on the first row', async () => {
      const off = await renderGrid(language, { rows: [{ rate: '600' }, {}] });
      await off.focusCell(1, 'rate');
      await off.user.keyboard('{Control>}d{/Control}');
      expect(off.typed(1, 'rate')).toBeUndefined();
    });

    it('Ctrl+I on a calculated cell asks the screen to explain it, and only there', async () => {
      const onExplain = vi.fn();
      const rig = await renderGrid(language, { rows: [{ qty: '20', rate: '600' }], onExplain });
      await rig.focusCell(0, 'qty');
      await rig.user.keyboard('{Control>}i{/Control}');
      expect(onExplain).not.toHaveBeenCalled();
      await rig.focusCell(0, 'line');
      await rig.user.keyboard('{Control>}i{/Control}');
      expect(onExplain).toHaveBeenCalledWith({
        rowId: rig.store.getState().rows[0]?.rowId,
        field: 'line',
      });
    });
  });

  describe('date cells (FR-7.19)', () => {
    it('+ and - change the day when the cell is not being typed in', async () => {
      const rig = await renderGrid(language, { rows: [{ date: '10-10-2026' }] });
      await rig.focusCell(0, 'date');
      await rig.user.keyboard('+');
      expect(rig.typed(0, 'date')).toBe('11-10-2026');
      await rig.user.keyboard('--');
      expect(rig.typed(0, 'date')).toBe('09-10-2026');
    });
  });

  describe('look', () => {
    it('number cells are marked left to right on the physical right edge; text cells are not', async () => {
      const rig = await renderGrid(language, { rows: [{}] });
      for (const field of ['qty', 'rate', 'buyerRate', 'date', 'line']) {
        expect(rig.cell(0, field), field).toHaveClass('cell-num');
      }
      for (const field of ['buyer', 'different', 'note']) {
        expect(rig.cell(0, field), field).not.toHaveClass('cell-num');
      }
      expect(rig.cell(0, 'line')).toHaveClass('cell-calc');
    });

    it('is drawn in the direction of the language', async () => {
      await renderGrid(language, { rows: [{}] });
      const root = document.querySelector('.entry-grid');
      expect(root).toHaveAttribute('dir', language === 'ur' ? 'rtl' : 'ltr');
      expect(document.querySelector(language === 'ur' ? '.ag-rtl' : '.ag-ltr')).not.toBeNull();
    });

    it('gives the same type the same width on every grid', async () => {
      await renderGrid(language, { rows: [{}] });
      const width = (field: string) =>
        document.querySelector<HTMLElement>(`.ag-header-cell[col-id="${field}"]`)?.style.width;
      expect(width('qty')).toBe('100px');
      expect(width('buyerRate')).toBe(width('line'));
      expect(width('rate')).not.toBe(width('qty'));
    });
  });
});

describe('language switch in the middle of an entry (Code Standards F39)', () => {
  it('keeps every typed value, the typing in the open cell, and the focused cell', async () => {
    const rig: GridRig = await renderGrid('ur', { rows: [{}, {}] });
    await rig.focusCell(0, 'buyer');
    await rig.user.keyboard('1{Enter}20{Enter}600{Enter}');
    await rig.user.keyboard('2{Enter}');
    // Halfway through typing a quantity in the second row.
    await rig.user.keyboard('3');
    expect(rig.isEditing()).toBe(true);

    act(() => {
      useLanguageStore.setState({ language: 'en' });
    });

    await waitFor(() => {
      expect(document.querySelector('.ag-ltr')).not.toBeNull();
    });
    expect(document.querySelector('.entry-grid')).toHaveAttribute('dir', 'ltr');
    const rows = rig.store.getState().rows.map((row) => row.typed);
    expect(rows[0]).toMatchObject({ buyer: '1', qty: '20', rate: '600' });
    expect(rows[1]).toMatchObject({ buyer: '2', qty: '3' });
    // The same cell has focus, and typing carries on there.
    await waitFor(() => {
      expect(rig.focused()).toBe('1:qty');
    });
    await rig.user.keyboard('5{Enter}');
    expect(rig.typed(1, 'qty')).toBe('5');
    // And back.
    act(() => {
      useLanguageStore.setState({ language: 'ur' });
    });
    await waitFor(() => {
      expect(document.querySelector('.ag-rtl')).not.toBeNull();
    });
    expect(rig.store.getState().rows.map((row) => row.typed['qty'])).toEqual(['20', '5']);
  });
});

describe.each(LANGUAGES)('entry grid with fixed rows (%s)', (language) => {
  it('Enter on the last stop of the last row, and ↓ on the last row, add no row', async () => {
    const rig = await renderGrid(language, { rows: [{}, {}], fixedRows: true });
    await rig.focusCell(1, 'rate');
    await rig.user.keyboard('600{Enter}');
    expect(rig.store.getState().rows).toHaveLength(2);
    expect(rig.focused()).toBe('1:rate');
    await rig.user.keyboard('{ArrowDown}');
    expect(rig.store.getState().rows).toHaveLength(2);
    expect(rig.focused()).toBe('1:rate');
  });

  it('Ctrl+Delete removes no row', async () => {
    const rig = await renderGrid(language, { rows: [{ qty: '5' }, { qty: '6' }], fixedRows: true });
    await rig.focusCell(0, 'qty');
    await rig.user.keyboard('{Control>}{Delete}{/Control}');
    expect(rig.store.getState().rows).toHaveLength(2);
  });

  it('Enter still goes on to the next row while there is one', async () => {
    const rig = await renderGrid(language, { rows: [{}, {}], fixedRows: true });
    await rig.focusCell(0, 'rate');
    await rig.user.keyboard('{Enter}');
    await waitFor(() => {
      expect(rig.focused()).toBe('1:buyer');
    });
  });
});

describe.each(LANGUAGES)('entry grid read-only words and cell errors (%s)', (language) => {
  it('a label cell shows the words the screen gave, cannot be typed in and is not an Enter stop', async () => {
    const rig = await renderGrid(language, { rows: [{ buyer: 'Akram' }, {}], withLabel: true });
    expect(rig.cell(0, 'name').textContent).toBe('Akram');
    expect(rig.cell(0, 'name')).toHaveClass('cell-calc');
    await rig.focusCell(0, 'name');
    await rig.user.keyboard('9');
    expect(rig.isEditing()).toBe(false);
    await rig.focusCell(1, 'rate');
    await rig.user.keyboard('{Enter}');
    expect(rig.focused()).not.toBe('1:name');
  });

  it('a cell the backend refused gets the error look, and the others do not', async () => {
    const rig = await renderGrid(language, {
      rows: [{}, {}],
      cellErrors: (ids) => ({ [ids[1] ?? '']: { rate: 'x' } }),
    });
    expect(rig.cell(1, 'rate')).toHaveClass('cell-error');
    expect(rig.cell(1, 'qty')).not.toHaveClass('cell-error');
    expect(rig.cell(0, 'rate')).not.toHaveClass('cell-error');
  });
});
