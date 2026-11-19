// The filters of the audit log and the entry that is open, kept above the screen (Code Standards
// F39): a language switch builds the grid again and loses nothing. A small store, like the other
// screens' stores (Tech Stack 3).

import { create } from 'zustand';
import { EMPTY_AUDIT_FILTERS, type AuditFilterState } from './filters';
import type { AuditRow } from './types';

interface AuditStore {
  filters: AuditFilterState;
  /** True once the filters the Owner last used have been put back (PRD FR-7.45): done once per run. */
  hydrated: boolean;
  /** The entry whose changed fields are shown, or null for the list. */
  opened: AuditRow | null;
  /** The row that had focus in the list, so it is focused again when the grid is built again. */
  focusRowId: string | null;
  change: (part: Partial<AuditFilterState>) => void;
  clear: () => void;
  /** Puts back the filters kept from last time (null: none kept), once. */
  hydrate: (filters: AuditFilterState | null) => void;
  open: (row: AuditRow | null) => void;
  setFocusRowId: (id: string) => void;
  reset: () => void;
}

export const useAuditStore = create<AuditStore>((set) => ({
  filters: EMPTY_AUDIT_FILTERS,
  hydrated: false,
  opened: null,
  focusRowId: null,
  change: (part) => {
    set((now) => ({ filters: { ...now.filters, ...part } }));
  },
  clear: () => {
    set({ filters: EMPTY_AUDIT_FILTERS });
  },
  hydrate: (filters) => {
    set((now) =>
      now.hydrated ? now : { hydrated: true, ...(filters === null ? {} : { filters }) },
    );
  },
  open: (opened) => {
    set({ opened });
  },
  setFocusRowId: (focusRowId) => {
    set({ focusRowId });
  },
  reset: () => {
    set({ filters: EMPTY_AUDIT_FILTERS, hydrated: false, opened: null, focusRowId: null });
  },
}));
