// The row the list should focus when it opens again, for each master: the one just opened or just
// saved. Without it a keyboard user who edits row 40 of 300 would come back to the top. It holds
// an id only, never data.

import { create } from 'zustand';
import type { MasterKind } from './types';

/** The lists that remember a row: the ten masters and the accounts. */
type FocusKind = MasterKind | 'account';

interface MasterFocusState {
  rows: Partial<Record<FocusKind, number>>;
  remember: (kind: FocusKind, id: number) => void;
}

export const useMasterFocus = create<MasterFocusState>()((set) => ({
  rows: {},
  remember: (kind, id) => {
    set((state) => ({ rows: { ...state.rows, [kind]: id } }));
  },
}));
