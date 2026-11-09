// The keys of an entry grid, in one place (PRD FR-7.13 to FR-7.20, entry-grid skill table).
// AG Grid asks suppressKeyboardEvent before it handles a key. Returning true means "the grid
// ignores this key". Every key we own is answered here, for both the focused cell and the cell
// editor, so a screen never writes a key handler (Code Standards F27).
//
//   Enter       next Enter stop (enterOrder.ts); finishes the cell being typed in
//   ↑ ↓         move up or down; finish the typing first; ↓ on the last row adds a row
//               (not in a grid with fixed rows)
//   ← →         move to the neighbouring cell as the user sees it; while typing, move the text
//               cursor, and at the edge of the text finish and move (PRD FR-7.14)
//   F2          never starts editing (F2 is "New", PRD 7.3)
//   Esc         while typing, left to the Esc order (escape.ts); the grid does not cancel itself
//   Ctrl+Delete remove the current row (the same way as the remove button, removeRow.tsx)
//   Ctrl+D      copy the cell from the line above (opt in)
//   Ctrl+I      explain a calculated cell (opt in)
//   + and -     in a date cell: next and previous day (PRD FR-7.19)
//   Space       in a choice cell (ادھاری / نقدی): the next word; in a select cell (درجہ): the list opens
//   Alt+↓       in a picker cell: start typing with the full list open (PRD FR-7.16); in a select
//               cell: the list opens
//   Backspace and Delete  on a cell that is not being typed in: the cell is cleared (an editable one),
//               and the key never leaves the screen (PRD FR-7.91); in a select cell the word is taken away
// Any other letter or digit starts typing in an editable cell, as AG Grid does.

import type { SuppressKeyboardEventParams } from 'ag-grid-community';
import { shiftDateText } from '../format';
import {
  SHORTCUTS,
  arrowSide,
  elementDirection,
  matchesShortcut,
  shouldLeaveField,
  type Side,
} from '../keyboard';
import { isNumberLike } from './columnTypes';
import { isEditable, nextEnterStop } from './enterOrder';
import type { EntryStore } from './entryStore';
import { REMOVE_COLUMN_ID } from './removeRow';
import type { CellRef, EntryColumn, EntryRow, GridInterceptors } from './types';

export interface GridKeyEnv {
  store: EntryStore;
  columns: readonly EntryColumn[];
  /** Direction of the grid as drawn: columns run right to left in rtl. */
  direction: 'rtl' | 'ltr';
  /** Ctrl+D copies the cell above. Only the sale-line grids turn it on. */
  copyAbove?: boolean;
  /** Ctrl+I on a calculated cell. */
  onExplain?: (cell: { rowId: string; field: string }) => void;
  /**
   * The rows are fixed (one per account in the opening balances grid): no key adds or removes a
   * row. ↓ on the last row and Enter on the last stop stay where they are.
   */
  fixedRows?: boolean;
  /** Can Ctrl+Delete remove this row? A row that cannot be removed swallows the key. */
  canRemoveRow?: (row: EntryRow) => boolean;
  /**
   * Removes the row the way the remove button does (plan 43): it may ask first what will go, and it puts
   * the focus on the row that takes its place. Without it the key removes the row at once.
   */
  requestRemove?: (rowId: string, field?: string) => void;
  /** What the screen adds to Enter and ↑ (moving between grids and panels). */
  interceptors?: GridInterceptors;
  /** Space or a click on a choice cell: move to the next word. */
  cycleChoice?: (cell: CellRef) => void;
}

type KeyParams = SuppressKeyboardEventParams<EntryRow>;

const NO_MODIFIER = (event: KeyboardEvent) =>
  !event.ctrlKey && !event.altKey && !event.metaKey && !event.shiftKey;

/** The column next to this one on screen: left or right as the user sees it. */
function neighbourField(
  columns: readonly EntryColumn[],
  field: string,
  side: Side,
  direction: 'rtl' | 'ltr',
): string | undefined {
  // Columns are in reading order, so the next column is on the right in ltr and the left in rtl.
  const step = (side === 'right') === (direction === 'ltr') ? 1 : -1;
  const here = columns.findIndex((column) => column.field === field);
  return columns[here + step]?.field;
}

export function createGridKeyHandler(env: GridKeyEnv) {
  const { store, columns, direction } = env;

  const focusCell = (params: KeyParams, rowIndex: number, field: string) => {
    params.api.setFocusedCell(rowIndex, field);
    params.api.ensureIndexVisible(rowIndex);
  };

  const moveVertically = (params: KeyParams, rowIndex: number, field: string, step: 1 | -1) => {
    const target = rowIndex + step;
    if (target < 0) {
      const row = store.getState().rows[rowIndex];
      if (row !== undefined) env.interceptors?.onAboveFirst?.({ rowId: row.rowId, field });
      return;
    }
    if (target >= store.getState().rows.length) {
      if (env.fixedRows === true) return;
      store.getState().addRow();
      store.getState().setPendingFocus({ rowIndex: 'last', field });
      return;
    }
    focusCell(params, target, field);
  };

  const copyFromAbove = (rowIndex: number, field: string) => {
    const { rows, updateTyped, patchRow } = store.getState();
    const above = rows[rowIndex - 1];
    const here = rows[rowIndex];
    if (above === undefined || here === undefined) return;
    if (columns.find((column) => column.field === field)?.type === 'picker') {
      // A picker cell holds a record: the line above's record is chosen again.
      patchRow(here.rowId, { picked: { [field]: above.picked?.[field] ?? null } });
      return;
    }
    updateTyped(here.rowId, field, above.typed[field] ?? '');
  };

  /**
   * `fromEditor` is true when a picker cell's own editor passes on a key it does not use itself
   * (Enter with the list closed, the arrows). AG Grid also hands every key typed in a popup editor
   * to this handler; for a picker cell those are left alone here, because the editor answers them.
   */
  return (params: KeyParams, fromEditor = false): boolean => {
    const event = params.event;
    if (event.type !== 'keydown') return false;
    const rowIndex = params.node.rowIndex;
    const field = params.column.getColId();
    // The cell of the remove button (plan 43): Enter or Space on it is a press of the button, and Tab
    // reaches it like any cell. It is not one of the grid's columns, so the Enter order never stops there.
    if (field === REMOVE_COLUMN_ID && rowIndex !== null) {
      const removable = store.getState().rows[rowIndex];
      const pressed = (event.key === 'Enter' || event.key === ' ') && NO_MODIFIER(event);
      if (removable === undefined || !(pressed || matchesShortcut(event, SHORTCUTS.removeRow))) {
        return false;
      }
      event.preventDefault();
      env.requestRemove?.(removable.rowId);
      return true;
    }
    const column = columns.find((candidate) => candidate.field === field);
    if (rowIndex === null || column === undefined) return false;
    const { rows } = store.getState();
    const row = rows[rowIndex];
    if (row === undefined) return false;
    // The editor of a picker or select cell answers its own keys; the grid leaves them alone.
    if (params.editing && (column.type === 'picker' || column.type === 'select') && !fromEditor) {
      return true;
    }

    // F2 is "New", never "edit".
    if (event.key === 'F2') return true;
    // Esc while typing goes through the Esc order, so the grid must not cancel by itself.
    if (event.key === 'Escape') return params.editing;

    if (matchesShortcut(event, SHORTCUTS.removeRow)) {
      event.preventDefault();
      if (env.fixedRows === true) return true;
      if (env.canRemoveRow !== undefined && !env.canRemoveRow(row)) return true;
      if (env.requestRemove !== undefined) {
        env.requestRemove(row.rowId, field);
        return true;
      }
      params.api.stopEditing(true);
      const remaining = rows.length - 1;
      store.getState().removeRow(row.rowId);
      store
        .getState()
        .setPendingFocus({ rowIndex: Math.min(rowIndex, Math.max(remaining - 1, 0)), field });
      return true;
    }
    if (matchesShortcut(event, SHORTCUTS.copyAbove)) {
      if (env.copyAbove === true) {
        event.preventDefault();
        params.api.stopEditing(true);
        copyFromAbove(rowIndex, field);
      }
      return true;
    }
    if (matchesShortcut(event, SHORTCUTS.explain)) {
      if ((column.type === 'calc' || column.explain === true) && env.onExplain !== undefined) {
        event.preventDefault();
        env.onExplain({ rowId: row.rowId, field });
      }
      return true;
    }

    // A row the screen holds on one cell: no other cell of the row takes any key, and the cursor does not
    // leave the cell it is held on by Tab or the up and down arrows (D8 Bug 1, B2). Enter and the side
    // arrows are decided below, because a text that was just changed is free to go on.
    const heldField = env.interceptors?.heldField?.(row.rowId) ?? null;
    if (heldField !== null && NO_MODIFIER(event)) {
      const leaves = event.key === 'Tab' || event.key === 'ArrowUp' || event.key === 'ArrowDown';
      if (field !== heldField || leaves) {
        event.preventDefault();
        return true;
      }
    }

    if (event.key === 'Enter') {
      // Ctrl+Enter is the screen's "save and next": the grid leaves it alone.
      if (!NO_MODIFIER(event)) return false;
      event.preventDefault();
      const before = row.typed[field] ?? '';
      params.api.stopEditing();
      // The screen is told whether this Enter changed the text of the cell: a verdict the backend gave
      // on the old text says nothing about the new one (D8 Bug 1, B2).
      const after = store.getState().rows[rowIndex]?.typed[field] ?? '';
      const changed = after !== before;
      if (env.interceptors?.onEnter?.({ rowId: row.rowId, field }, { changed }) === true) {
        return true;
      }
      const step = nextEnterStop(columns, store.getState().rows, { rowIndex, field });
      if (step === null) return true;
      if (step.kind === 'cell') {
        focusCell(params, step.rowIndex, step.field);
      } else if (env.fixedRows !== true) {
        store.getState().addRow();
        store.getState().setPendingFocus({ rowIndex: 'last', field: step.field });
      }
      return true;
    }

    // Alt+↓ on a select cell: its short list opens.
    if (column.type === 'select' && matchesShortcut(event, SHORTCUTS.openList)) {
      if (params.editing) return false;
      event.preventDefault();
      if (isEditable(column, row)) params.api.startEditingCell({ rowIndex, colKey: field });
      return true;
    }

    // Alt+↓ on a picker cell: typing starts with the full list open (PRD FR-7.16).
    if (column.type === 'picker' && matchesShortcut(event, SHORTCUTS.openList)) {
      if (params.editing) return false;
      event.preventDefault();
      store.getState().setEditHint('open-list');
      params.api.startEditingCell({ rowIndex, colKey: field });
      return true;
    }

    if (!NO_MODIFIER(event)) return false;

    // A select cell holds fixed words, so nothing is typed into it. Space opens its list, and any other
    // letter or digit is left alone (not a start of typing). Delete and Backspace clear it, below.
    if (column.type === 'select' && !params.editing) {
      if (event.key === ' ') {
        event.preventDefault();
        if (isEditable(column, row)) params.api.startEditingCell({ rowIndex, colKey: field });
        return true;
      }
      if (event.key.length === 1) return true;
    }

    // Backspace and Delete on a cell that is not being typed in clear it, like a spreadsheet; on a cell
    // that cannot be typed in (a calculated cell, a choice) they do nothing. They never leave the
    // screen (PRD FR-7.91, D8 Bug 2): the key is answered here and not left to the web view.
    if (!params.editing && (event.key === 'Backspace' || event.key === 'Delete')) {
      event.preventDefault();
      if (isEditable(column, row)) {
        if (column.type === 'picker') store.getState().updatePicked(row.rowId, field, null);
        else store.getState().updateTyped(row.rowId, field, '');
      }
      return true;
    }

    // Space on a choice cell that is not being typed in: the next word.
    if (column.type === 'choice' && event.key === ' ' && !params.editing) {
      event.preventDefault();
      if (isEditable(column, row)) env.cycleChoice?.({ rowId: row.rowId, field });
      return true;
    }

    // + and - in a date cell that is not being typed in.
    if (!params.editing && column.type === 'date' && (event.key === '+' || event.key === '-')) {
      event.preventDefault();
      const next = shiftDateText(row.typed[field] ?? '', event.key === '+' ? 1 : -1, new Date());
      store.getState().updateTyped(row.rowId, field, next);
      return true;
    }

    if (event.key === 'ArrowUp' || event.key === 'ArrowDown') {
      event.preventDefault();
      params.api.stopEditing();
      moveVertically(params, rowIndex, field, event.key === 'ArrowDown' ? 1 : -1);
      return true;
    }

    const side = arrowSide(event.key);
    if (side !== null) {
      const target = event.target;
      if (params.editing && target instanceof HTMLInputElement) {
        // The text cursor moves first. At the edge, the arrow leaves the cell (PRD FR-7.14).
        // Number cells are always written left to right, even in the Urdu layout (NFR-USE-19).
        const inputDirection = isNumberLike(column.type) ? 'ltr' : elementDirection(target);
        if (!shouldLeaveField(target, side, inputDirection)) return true;
      }
      // The cell the row is held on is not left by a side arrow either.
      if (heldField !== null) {
        event.preventDefault();
        return true;
      }
      const neighbour = neighbourField(columns, field, side, direction);
      event.preventDefault();
      if (neighbour === undefined) return true;
      params.api.stopEditing();
      focusCell(params, rowIndex, neighbour);
      return true;
    }
    return false;
  };
}
