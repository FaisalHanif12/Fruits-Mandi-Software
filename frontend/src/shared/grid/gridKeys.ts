// The keys of an entry grid, in one place (PRD FR-7.13 to FR-7.20, entry-grid skill table).
// AG Grid asks suppressKeyboardEvent before it handles a key. Returning true means "the grid
// ignores this key". Every key we own is answered here, for both the focused cell and the cell
// editor, so a screen never writes a key handler (Code Standards F27).
//
//   Enter       next Enter stop (enterOrder.ts); finishes the cell being typed in
//   ↑ ↓         move up or down; finish the typing first; ↓ on the last row adds a row
//   ← →         move to the neighbouring cell as the user sees it; while typing, move the text
//               cursor, and at the edge of the text finish and move (PRD FR-7.14)
//   F2          never starts editing (F2 is "New", PRD 7.3)
//   Esc         while typing, left to the Esc order (escape.ts); the grid does not cancel itself
//   Ctrl+Delete remove the current row
//   Ctrl+D      copy the cell from the line above (opt in)
//   Ctrl+I      explain a calculated cell (opt in)
//   + and -     in a date cell: next and previous day (PRD FR-7.19)
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
import { nextEnterStop } from './enterOrder';
import type { EntryStore } from './entryStore';
import type { EntryColumn, EntryRow } from './types';

export interface GridKeyEnv {
  store: EntryStore;
  columns: readonly EntryColumn[];
  /** Direction of the grid as drawn: columns run right to left in rtl. */
  direction: 'rtl' | 'ltr';
  /** Ctrl+D copies the cell above. Only the sale-line grids turn it on. */
  copyAbove?: boolean;
  /** Ctrl+I on a calculated cell. */
  onExplain?: (cell: { rowId: string; field: string }) => void;
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
    if (target < 0) return;
    if (target >= store.getState().rows.length) {
      store.getState().addRow();
      store.getState().setPendingFocus({ rowIndex: 'last', field });
      return;
    }
    focusCell(params, target, field);
  };

  const copyFromAbove = (rowIndex: number, field: string) => {
    const { rows, updateTyped } = store.getState();
    const above = rows[rowIndex - 1];
    const here = rows[rowIndex];
    if (above === undefined || here === undefined) return;
    updateTyped(here.rowId, field, above.typed[field] ?? '');
  };

  return (params: KeyParams): boolean => {
    const event = params.event;
    if (event.type !== 'keydown') return false;
    const rowIndex = params.node.rowIndex;
    const field = params.column.getColId();
    const column = columns.find((candidate) => candidate.field === field);
    if (rowIndex === null || column === undefined) return false;
    const { rows } = store.getState();
    const row = rows[rowIndex];
    if (row === undefined) return false;

    // F2 is "New", never "edit".
    if (event.key === 'F2') return true;
    // Esc while typing goes through the Esc order, so the grid must not cancel by itself.
    if (event.key === 'Escape') return params.editing;

    if (matchesShortcut(event, SHORTCUTS.removeRow)) {
      event.preventDefault();
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
      if (column.type === 'calc' && env.onExplain !== undefined) {
        event.preventDefault();
        env.onExplain({ rowId: row.rowId, field });
      }
      return true;
    }

    if (event.key === 'Enter') {
      // Ctrl+Enter is the screen's "save and next": the grid leaves it alone.
      if (!NO_MODIFIER(event)) return false;
      event.preventDefault();
      params.api.stopEditing();
      const step = nextEnterStop(columns, store.getState().rows, { rowIndex, field });
      if (step === null) return true;
      if (step.kind === 'cell') {
        focusCell(params, step.rowIndex, step.field);
      } else {
        store.getState().addRow();
        store.getState().setPendingFocus({ rowIndex: 'last', field: step.field });
      }
      return true;
    }

    if (!NO_MODIFIER(event)) return false;

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
