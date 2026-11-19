// The remove button of a row of every entry grid (plan 43, PRD FR-7.3, FR-7.4, NFR-USE-7, NFR-USE-8):
// Ctrl+Delete was the only way to take a line away, and a key is a shortcut, never the only way. Every
// grid made with `EntryGrid` has a small trash button at the end of each row, in Urdu and in English:
// it removes its row with the mouse, with Enter and with Space; it follows `canRemoveRow` and is not
// drawn where a row cannot go; a row that has more under it than itself asks first, in one sentence,
// and the key asks too; focus goes to the same column of the row that took its place; Enter moving
// between cells never stops on it; a grid with fixed rows has none.

import { act, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import type { SuppressKeyboardEventParams } from 'ag-grid-community';
import { LANGUAGES } from '../../src/i18n';
import { createEntryStore, type EntryRow } from '../../src/shared/grid';
import { createGridKeyHandler, type GridKeyEnv } from '../../src/shared/grid/gridKeys';
import { useLanguageStore } from '../../src/shared/i18n';
import { renderGrid, type GridRig } from '../helpers/entry-grid';
import { message, visible } from '../helpers/render';

const ROWS = [
  { buyer: '1', qty: '10', rate: '100' },
  { buyer: '2', qty: '20', rate: '200' },
  { buyer: '3', qty: '30', rate: '300' },
];

function removeButtons(): HTMLElement[] {
  return Array.from(document.querySelectorAll<HTMLElement>('.row-remove-button'));
}

/** The remove button of a row, by its place: the test fails with its words when there is none. */
function buttonAt(index: number): HTMLElement {
  const button = removeButtons()[index];
  if (button === undefined) throw new Error(`no remove button at ${String(index)}`);
  return button;
}

function rowCount(): number {
  return document.querySelectorAll('.ag-row[row-index]').length;
}

/** The buyer typed in each row, top to bottom. */
function buyers(rig: GridRig): (string | undefined)[] {
  return rig.store.getState().rows.map((row) => row.typed['buyer']);
}

describe.each(LANGUAGES)('the remove button of an entry grid (%s)', (language) => {
  const m = (id: string, vars: Record<string, string | number> = {}) => message(language, id, vars);
  // The "back" button also names its key (Esc).
  const backName = (name: string) => visible(name).startsWith(m('common-back'));

  it('is at the end of every row, named "Remove this line" with its key in the tooltip', async () => {
    await renderGrid(language, { rows: ROWS });
    const buttons = removeButtons();
    expect(buttons).toHaveLength(3);
    for (const button of buttons) {
      expect(button).toHaveAccessibleName(m('common-remove-line'));
      expect(button.getAttribute('title')).toBe(`${m('common-remove-line')} (Ctrl+Delete)`);
    }
    // It is the last column of the grid, and the grid is told nothing else about it.
    const heads = Array.from(document.querySelectorAll('.ag-header-cell[col-id]')).map((head) =>
      head.getAttribute('col-id'),
    );
    expect(heads.at(-1)).toBe('__remove');
  });

  it('removes its row with the mouse and leaves the other rows as they were', async () => {
    const rig = await renderGrid(language, { rows: ROWS });
    await rig.user.click(buttonAt(1));
    await waitFor(() => {
      expect(rowCount()).toBe(2);
    });
    expect(buyers(rig)).toEqual(['1', '3']);
    expect(rig.typed(1, 'qty')).toBe('30');
  });

  it('puts the focus on the same column of the row that took its place', async () => {
    const rig = await renderGrid(language, { rows: ROWS });
    await rig.focusCell(1, 'qty');
    await rig.user.click(buttonAt(1));
    await waitFor(() => {
      expect(rig.focused()).toBe('1:qty');
    });
    expect(rig.typed(1, 'buyer')).toBe('3');
  });

  it('puts the focus on the row above when the last row went', async () => {
    const rig = await renderGrid(language, { rows: ROWS });
    await rig.focusCell(2, 'rate');
    await rig.user.click(buttonAt(2));
    await waitFor(() => {
      expect(rig.focused()).toBe('1:rate');
    });
    expect(buyers(rig)).toEqual(['1', '2']);
  });

  it('works with Enter and with Space when it has the focus', async () => {
    const rig = await renderGrid(language, { rows: ROWS });
    removeButtons()[0]?.focus();
    expect(document.activeElement).toBe(removeButtons()[0]);
    await rig.user.keyboard('{Enter}');
    await waitFor(() => {
      expect(rowCount()).toBe(2);
    });
    removeButtons()[0]?.focus();
    await rig.user.keyboard(' ');
    await waitFor(() => {
      expect(rowCount()).toBe(1);
    });
    expect(buyers(rig)).toEqual(['3']);
  });

  it('leaves one empty row when the only row is removed', async () => {
    const rig = await renderGrid(language, { rows: [{ buyer: '1', qty: '10' }] });
    await rig.user.click(buttonAt(0));
    await waitFor(() => {
      expect(rig.typed(0, 'buyer') ?? '').toBe('');
    });
    expect(rowCount()).toBe(1);
  });

  it('is never a stop of Enter: Enter on the last stop of a row goes to the next row, not to the button', async () => {
    const rig = await renderGrid(language, { rows: ROWS });
    await rig.focusCell(0, 'rate');
    await rig.user.keyboard('{Enter}');
    expect(rig.focused()).toBe('1:buyer');
    // And the arrows of a row stay on the cells the user types in.
    await rig.focusCell(1, 'qty');
    await rig.user.keyboard('{Enter}');
    expect(rig.focused()).toBe('1:rate');
    expect(document.activeElement?.classList.contains('row-remove-button')).toBe(false);
  });

  it('is not drawn for a row that canRemoveRow refuses, and the key does nothing there either', async () => {
    const rig = await renderGrid(language, {
      rows: ROWS,
      canRemoveRow: (row) => row.typed['buyer'] !== '2',
    });
    expect(removeButtons()).toHaveLength(2);
    const second = rig.cell(1, '__remove');
    expect(second.querySelector('.row-remove-button')).toBeNull();
    await rig.focusCell(1, 'qty');
    await rig.user.keyboard('{Control>}{Delete}{/Control}');
    expect(rowCount()).toBe(3);
  });

  it('has no button in a grid with fixed rows', async () => {
    await renderGrid(language, { rows: ROWS, fixedRows: true });
    expect(removeButtons()).toHaveLength(0);
    expect(document.querySelector('.ag-header-cell[col-id="__remove"]')).toBeNull();
  });

  it('keeps the rest when the language is switched after a removal', async () => {
    const rig = await renderGrid(language, { rows: ROWS });
    await rig.user.click(buttonAt(0));
    await waitFor(() => {
      expect(rowCount()).toBe(2);
    });
    const other = language === 'ur' ? 'en' : 'ur';
    act(() => {
      useLanguageStore.setState({ language: other });
    });
    await waitFor(() => {
      expect(removeButtons()).toHaveLength(2);
    });
    expect(buyers(rig)).toEqual(['2', '3']);
  });

  describe('a row that has more under it than itself', () => {
    const question = (row: { typed: Record<string, string | undefined> }) =>
      row.typed['buyer'] === '2'
        ? { messageId: 'bikri-lot-remove-question', vars: { count: 3 } }
        : null;

    it('asks first in one sentence, and goes only when the user says so', async () => {
      const rig = await renderGrid(language, { rows: ROWS, removeQuestion: question });
      await rig.user.click(buttonAt(1));
      const dialog = await screen.findByRole('alertdialog');
      expect(visible(dialog.textContent)).toContain(m('bikri-lot-remove-question', { count: 3 }));
      // Nothing has gone while the question is open.
      expect(rowCount()).toBe(3);
      await rig.user.click(
        within(dialog).getByRole('button', { name: m('common-remove-line-confirm') }),
      );
      await waitFor(() => {
        expect(rowCount()).toBe(2);
      });
      expect(buyers(rig)).toEqual(['1', '3']);
    });

    it('keeps the row when the user goes back', async () => {
      const rig = await renderGrid(language, { rows: ROWS, removeQuestion: question });
      await rig.user.click(buttonAt(1));
      const dialog = await screen.findByRole('alertdialog');
      await rig.user.click(within(dialog).getByRole('button', { name: backName }));
      await waitFor(() => {
        expect(screen.queryByRole('alertdialog')).toBeNull();
      });
      expect(rowCount()).toBe(3);
      expect(buyers(rig)).toEqual(['1', '2', '3']);
    });

    it('asks for Ctrl+Delete too, and a row with nothing under it goes without a question', async () => {
      const rig = await renderGrid(language, { rows: ROWS, removeQuestion: question });
      await rig.focusCell(1, 'qty');
      await rig.user.keyboard('{Control>}{Delete}{/Control}');
      const dialog = await screen.findByRole('alertdialog');
      expect(visible(dialog.textContent)).toContain(m('bikri-lot-remove-question', { count: 3 }));
      await rig.user.click(within(dialog).getByRole('button', { name: backName }));
      await waitFor(() => {
        expect(screen.queryByRole('alertdialog')).toBeNull();
      });
      // Row 3 has nothing under it: no question.
      await rig.user.click(buttonAt(2));
      await waitFor(() => {
        expect(rowCount()).toBe(2);
      });
      expect(screen.queryByRole('alertdialog')).toBeNull();
    });
  });
});

// The cell of the remove button is reached with Tab (AG Grid focuses the cell that holds the button), and
// the keys on it are the keys of the button: Enter and Space press it, and so does Ctrl+Delete (plan 43
// R5). The arrows on it are the grid's own (grid-trash-keys.test.tsx): the one that reads on does nothing,
// the other goes back to the last cell of the row, ↑ and ↓ go to the trash cell of the next row, and ↓ on the
// last row's goes to the "+" button. Any letter, and Ctrl+Enter, are left to the grid.
describe('the keys on the cell of the remove button', () => {
  function press(key: string, init: KeyboardEventInit = {}, extra: Partial<GridKeyEnv> = {}) {
    const store = createEntryStore([{ buyer: '1' }, { buyer: '2' }]);
    const requestRemove = vi.fn();
    const setFocusedCell = vi.fn();
    const handler = createGridKeyHandler({
      store,
      columns: [{ field: 'buyer', type: 'text', headerId: 'glossary-bikri' }],
      direction: 'rtl',
      requestRemove,
      ...extra,
    });
    const event = new KeyboardEvent('keydown', { key, cancelable: true, ...init });
    const ignored = handler({
      event,
      node: { rowIndex: 1 },
      column: { getColId: () => '__remove' },
      editing: false,
      api: { setFocusedCell, ensureIndexVisible: vi.fn() },
    } as unknown as SuppressKeyboardEventParams<EntryRow>);
    return {
      ignored,
      event,
      requestRemove,
      setFocusedCell,
      store,
      rowId: store.getState().rows[1]?.rowId,
    };
  }

  it('Enter and Space press the button of that row, and the grid does nothing else with them', () => {
    for (const key of ['Enter', ' ']) {
      const { ignored, event, requestRemove, rowId } = press(key);
      expect(requestRemove).toHaveBeenCalledTimes(1);
      expect(requestRemove).toHaveBeenCalledWith(rowId);
      expect(ignored).toBe(true);
      expect(event.defaultPrevented).toBe(true);
    }
  });

  it('Ctrl+Delete presses it too', () => {
    const { requestRemove, rowId } = press('Delete', { code: 'Delete', ctrlKey: true });
    expect(requestRemove).toHaveBeenCalledWith(rowId);
  });

  it('a letter and Ctrl+Enter are left to the grid', () => {
    for (const [key, init] of [
      ['a', {}],
      ['Enter', { ctrlKey: true }],
    ] as const) {
      const { ignored, requestRemove } = press(key, init);
      expect(ignored).toBe(false);
      expect(requestRemove).not.toHaveBeenCalled();
    }
  });

  it('the arrow that reads on (← in a right-to-left grid) does nothing, the other goes back to the last cell', () => {
    const toEnd = press('ArrowLeft');
    expect(toEnd.ignored).toBe(true);
    expect(toEnd.event.defaultPrevented).toBe(true);
    expect(toEnd.setFocusedCell).not.toHaveBeenCalled();
    const toStart = press('ArrowRight');
    expect(toStart.ignored).toBe(true);
    expect(toStart.setFocusedCell).toHaveBeenCalledWith(1, 'buyer');
  });

  it('↑ goes to the trash cell of the row above, and ↓ on the last row asks the "+" button for the focus', () => {
    const up = press('ArrowUp');
    expect(up.setFocusedCell).toHaveBeenCalledWith(0, '__remove');
    const focusAdd = vi.fn(() => true);
    const down = press('ArrowDown', {}, { focusAdd });
    expect(down.ignored).toBe(true);
    expect(focusAdd).toHaveBeenCalledTimes(1);
    expect(down.setFocusedCell).not.toHaveBeenCalled();
    expect(down.store.getState().rows).toHaveLength(2);
  });
});
