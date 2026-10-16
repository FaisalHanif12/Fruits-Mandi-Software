// The rows of an entry grid, kept outside the grid (Code Standards F39, entry-grid skill). The
// grid can be destroyed and rebuilt at any time, for example by a language switch, and nothing
// the user typed is lost, because it lives here. A screen makes one store with createEntryStore.

import { create } from 'zustand';
import type { CellRef, EntryRow, PendingFocus, PickedRecord } from './types';

export interface EntryState {
  rows: EntryRow[];
  /** The cell that has focus, kept so a rebuilt grid can focus it again. */
  focus: CellRef | null;
  /** Adds a row at the end and returns its ID. */
  addRow: (typed?: Record<string, string>, picked?: Record<string, PickedRecord>) => string;
  updateTyped: (rowId: string, field: string, text: string) => void;
  /** Sets the record chosen in a picker cell, or clears it with null. */
  updatePicked: (rowId: string, field: string, record: PickedRecord | null) => void;
  /**
   * Changes several cells of one row in one step: text, and records of picker cells (null clears
   * one). Used when the app fills cells together, for example the item, packing and مارکہ that
   * a seller's memory gives (PRD FR-7.51).
   */
  patchRow: (
    rowId: string,
    patch: {
      typed?: Record<string, string>;
      picked?: Record<string, PickedRecord | null>;
    },
  ) => void;
  /** Sets one field to `text` on every row where `when` is true, in one change. */
  updateTypedWhere: (field: string, when: (row: EntryRow) => boolean, text: string) => void;
  /** Removes a row. The grid always keeps at least one row. */
  removeRow: (rowId: string) => void;
  setFocus: (focus: CellRef | null) => void;
  /** Focus to apply once the rows are drawn, after a key added or removed a row. */
  pendingFocus: PendingFocus | null;
  setPendingFocus: (pending: PendingFocus | null) => void;
  /**
   * What the next cell editor should do besides starting: `open-list` opens the list of a picker
   * cell at once (Alt+↓, PRD FR-7.16). Read and cleared by the editor.
   */
  editHint: 'open-list' | null;
  setEditHint: (hint: 'open-list' | null) => void;
}

function newRowId(): string {
  return globalThis.crypto.randomUUID();
}

function newRow(
  typed: Record<string, string> = {},
  picked?: Record<string, PickedRecord>,
): EntryRow {
  return picked === undefined ? { rowId: newRowId(), typed } : { rowId: newRowId(), typed, picked };
}

/** `picked` with one field set, or taken away when `record` is null. */
function withPicked(
  picked: EntryRow['picked'],
  field: string,
  record: PickedRecord | null,
): Record<string, PickedRecord> {
  const next = { ...picked };
  if (record === null) Reflect.deleteProperty(next, field);
  else next[field] = record;
  return next;
}

export function createEntryStore(initial: readonly Record<string, string>[] = []) {
  return create<EntryState>()((set) => ({
    rows: initial.length > 0 ? initial.map((typed) => newRow(typed)) : [newRow()],
    focus: null,
    pendingFocus: null,
    editHint: null,
    addRow: (typed, picked) => {
      const row = newRow(typed, picked);
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
    updatePicked: (rowId, field, record) => {
      set((state) => ({
        rows: state.rows.map((row) =>
          row.rowId === rowId ? { ...row, picked: withPicked(row.picked, field, record) } : row,
        ),
      }));
    },
    patchRow: (rowId, patch) => {
      set((state) => ({
        rows: state.rows.map((row) => {
          if (row.rowId !== rowId) return row;
          let picked = row.picked;
          for (const [field, record] of Object.entries(patch.picked ?? {})) {
            picked = withPicked(picked, field, record);
          }
          return {
            ...row,
            typed: { ...row.typed, ...patch.typed },
            ...(picked === undefined ? {} : { picked }),
          };
        }),
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
    setEditHint: (editHint) => {
      set({ editHint });
    },
  }));
}

export type EntryStore = ReturnType<typeof createEntryStore>;
