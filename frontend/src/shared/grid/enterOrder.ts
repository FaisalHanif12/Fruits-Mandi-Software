// The Enter rule of PRD FR-7.14 (entry-grid skill): Enter moves to the next editable cell in
// reading order, skips cells that are read-only on this row, and still stops on an editable cell
// that the app filled, so the user sees and confirms it. On the last stop of a row it goes to the
// first stop of the next row, and makes that row if there is none. Pure, so every case is tested
// without a grid. Reading order is the order of the columns, which is right to left in Urdu
// because the grid itself is drawn right to left.

import { isReadOnlyType, type EntryColumn, type EntryRow } from './types';

/** Can this cell be typed in on this row? A calculated cell never can. */
export function isEditable(column: EntryColumn, row: EntryRow): boolean {
  if (isReadOnlyType(column.type)) return false;
  return column.editable?.(row) ?? true;
}

/** Does Enter stop at this column, when the cell is editable? */
export function isEnterStop(column: EntryColumn): boolean {
  return column.enterStop ?? !isReadOnlyType(column.type);
}

export type EnterStep =
  { kind: 'cell'; rowIndex: number; field: string } | { kind: 'new-row'; field: string };

const BLANK_ROW: EntryRow = { rowId: '', typed: {} };

function firstStop(columns: readonly EntryColumn[], row: EntryRow): string | null {
  const column = columns.find((candidate) => isEnterStop(candidate) && isEditable(candidate, row));
  return column?.field ?? null;
}

/**
 * Where Enter goes from the cell at `from`. Null when there is no stop at all (a grid with no
 * editable column).
 */
export function nextEnterStop(
  columns: readonly EntryColumn[],
  rows: readonly EntryRow[],
  from: { rowIndex: number; field: string },
): EnterStep | null {
  const here = columns.findIndex((column) => column.field === from.field);
  const current = rows[from.rowIndex];
  if (current !== undefined) {
    const after = columns
      .slice(here + 1)
      .find((column) => isEnterStop(column) && isEditable(column, current));
    if (after !== undefined) return { kind: 'cell', rowIndex: from.rowIndex, field: after.field };
  }
  for (let rowIndex = from.rowIndex + 1; rowIndex < rows.length; rowIndex += 1) {
    const row = rows[rowIndex];
    const field = row === undefined ? null : firstStop(columns, row);
    if (field !== null) return { kind: 'cell', rowIndex, field };
  }
  const field = firstStop(columns, BLANK_ROW);
  return field === null ? null : { kind: 'new-row', field };
}
