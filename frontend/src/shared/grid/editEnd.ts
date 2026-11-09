// The end of an edit, written to the store whatever the grid decides (entry-grid skill, Code
// Standards F39, D8 plan 61). The store is the one truth of what was typed; the grid only shows it.
// AG Grid writes the text of a finished cell through the column's `valueSetter`, but only when it
// thinks the value changed against what it last saw. If it ever sees "no change" while the store
// holds something else, the text the user typed would be dropped and the cell would show the old
// value again, and typing the same text once more would meet the same answer. So when an edit ends
// with text, the store is made to hold that text, and the cell always ends up as typed.

import type { CellEditingStoppedEvent } from 'ag-grid-community';
import type { EntryStore } from './entryStore';
import { isReadOnlyType, type EntryColumn, type EntryRow } from './types';

/** The column types whose editors write to the store themselves; their text is never a typed value. */
function writesItself(column: EntryColumn): boolean {
  return column.type === 'picker' || column.type === 'select' || column.type === 'choice';
}

/**
 * Makes the store hold the text an edit ended with. An edit that was cancelled (Esc, a removed row, a
 * copy from the line above) has no text (`newValue` is undefined) and is left alone, so a cancel never
 * brings an old value back over a newer one.
 */
export function syncEditedText(
  store: EntryStore,
  columns: readonly EntryColumn[],
  event: Pick<CellEditingStoppedEvent<EntryRow>, 'data' | 'column' | 'newValue'>,
): void {
  const { data } = event;
  const newValue: unknown = event.newValue;
  if (data === undefined || typeof newValue !== 'string') return;
  const field = event.column.getColId();
  const column = columns.find((candidate) => candidate.field === field);
  if (column === undefined || isReadOnlyType(column.type) || writesItself(column)) return;
  const row = store.getState().rows.find((candidate) => candidate.rowId === data.rowId);
  if (row === undefined) return;
  if ((row.typed[field] ?? '') !== newValue)
    store.getState().updateTyped(row.rowId, field, newValue);
}
