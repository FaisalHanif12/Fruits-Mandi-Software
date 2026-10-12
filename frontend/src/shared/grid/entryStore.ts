// The rows of an entry grid, kept outside the grid (Code Standards F39, entry-grid skill). The
// grid can be destroyed and rebuilt at any time, for example by a language switch, and nothing
// the user typed is lost, because it lives here. A screen makes one store with createEntryStore.

import { create } from 'zustand';
import type { CellRef, EntryRow, PendingFocus } from './types';

export interface EntryState {
  rows: EntryRow[];
  /** The cell that has focus, kept so a rebuilt grid can focus it again. */
  focus: CellRef | null;
  /** Adds an empty row at the end and returns its ID. */
  addRow: (typed?: Record<string, string>) => string;
  updateTyped: (rowId: string, field: string, text: string) => void;
  /** Sets one field to `text` on every row where `when` is true, in one change. */
  updateTypedWhere: (field: string, when: (row: EntryRow) => boolean, text: string) => void;
  /** Removes a row. The grid always keeps at least one row. */
  removeRow: (rowId: string) => void;
  setFocus: (focus: CellRef | null) => void;
  /** Focus to apply once the rows are drawn, after a key added or removed a row. */
  pendingFocus: PendingFocus | null;
  setPendingFocus: (pending: PendingFocus | null) => void;
}

function newRowId(): string {
  return globalThis.crypto.randomUUID();
}

function newRow(typed: Record<string, string> = {}): EntryRow {
  return { rowId: newRowId(), typed };
}

export function createEntryStore(initial: readonly Record<string, string>[] = []) {
  return create<EntryState>()((set) => ({
    rows: initial.length > 0 ? initial.map((typed) => newRow(typed)) : [newRow()],
    focus: null,
    pendingFocus: null,
    addRow: (typed) => {
      const row = newRow(typed);
      set((state) => ({ rows: [...state.rows, row] }));
      return row.rowId;
    },
    updateTyped: (rowId, field, text) => {
      set((state) => ({
        rows: state.rows.map((row) =>
          row.rowId === rowId ? { ...row, typed: { ...row.typed, [field]: text } } : row,
        ),
      }));
    },
    updateTypedWhere: (field, when, text) => {
      set((state) => ({
        rows: state.rows.map((row) =>
          when(row) ? { ...row, typed: { ...row.typed, [field]: text } } : row,
        ),
      }));
    },
    removeRow: (rowId) => {
      set((state) => {
        const rows = state.rows.filter((row) => row.rowId !== rowId);
        return { rows: rows.length > 0 ? rows : [newRow()] };
      });
    },
    setFocus: (focus) => {
      set({ focus });
    },
    setPendingFocus: (pendingFocus) => {
      set({ pendingFocus });
    },
  }));
}

export type EntryStore = ReturnType<typeof createEntryStore>;
