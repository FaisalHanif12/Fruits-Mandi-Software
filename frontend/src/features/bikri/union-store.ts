// The sale lines of every lot as one list, for the "پوری بکری" view (PRD FR-12.1): one grid with a
// lot column, editable like the grid of one lot.
//
// There is no second copy of the data. The stores of the lots stay the only place a line lives.
// This store only shows their rows one after the other, in lot order (`rows` is rebuilt whenever
// anything in the draft changes), and every change made through it is handed to the store of the
// lot that owns the row. So the grid of one lot and this grid always agree. The focus of this
// grid is also told to the lot of the row and selects that lot, so the details of the line, the
// party card and the problems follow the cursor in both views.
//
// A row that is added goes to the lot of the row that has focus (the last lot when none has).

import { create } from 'zustand';
import type { EntryRow, EntryState, EntryStore } from '../../shared/grid';

interface Owner {
  lotKey: string;
  store: EntryStore;
}

export interface UnionDeps {
  /** The lots, in order: their row ids are the lot keys. */
  lotKeys: () => readonly string[];
  /** The store of the sale lines of one lot. */
  salesFor: (lotKey: string) => EntryStore;
  /** Makes a lot the selected one. */
  select: (lotKey: string) => void;
  /** Calls `listener` after any change in the draft. Returns what stops it. */
  onChange: (listener: () => void) => () => void;
}

export function createLinesUnion(deps: UnionDeps): EntryStore {
  const owners = (): Owner[] =>
    deps.lotKeys().map((lotKey) => ({ lotKey, store: deps.salesFor(lotKey) }));
  const collect = (): EntryRow[] => owners().flatMap((owner) => owner.store.getState().rows);
  const ownerOf = (rowId: string): Owner | undefined =>
    owners().find((owner) => owner.store.getState().rows.some((row) => row.rowId === rowId));

  const union = create<EntryState>()((set, get) => ({
    rows: collect(),
    focus: null,
    pendingFocus: null,
    editHint: null,
    addRow: (typed, picked) => {
      const all = owners();
      const focused = get().focus;
      const target = (focused === null ? undefined : ownerOf(focused.rowId)) ?? all.at(-1);
      if (target === undefined) return '';
      return target.store.getState().addRow(typed, picked);
    },
    updateTyped: (rowId, field, text) => {
      ownerOf(rowId)?.store.getState().updateTyped(rowId, field, text);
    },
    updatePicked: (rowId, field, record) => {
      ownerOf(rowId)?.store.getState().updatePicked(rowId, field, record);
    },
    patchRow: (rowId, patch) => {
      ownerOf(rowId)?.store.getState().patchRow(rowId, patch);
    },
    updateTypedWhere: (field, when, text) => {
      for (const owner of owners()) owner.store.getState().updateTypedWhere(field, when, text);
    },
    removeRow: (rowId) => {
      ownerOf(rowId)?.store.getState().removeRow(rowId);
    },
    setFocus: (focus) => {
      set({ focus });
      if (focus === null) return;
      const owner = ownerOf(focus.rowId);
      if (owner === undefined) return;
      owner.store.getState().setFocus(focus);
      deps.select(owner.lotKey);
    },
    setPendingFocus: (pendingFocus) => {
      set({ pendingFocus });
    },
    setEditHint: (editHint) => {
      set({ editHint });
    },
  }));

  // The rows follow the lots. The array changes only when a row did, so a keystroke in the header
  // does not redraw this grid.
  deps.onChange(() => {
    const rows = collect();
    const known = union.getState().rows;
    if (rows.length === known.length && rows.every((row, index) => row === known[index])) return;
    union.setState({ rows });
  });
  return union;
}
