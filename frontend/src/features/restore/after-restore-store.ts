// What the Owner is reminded of right after he signs in on a computer that has just been restored
// (plan 56 Section 3, step 6). Kept apart from the restore store, which is cleared when the restore
// pages are left. It holds no key and no books: only the facts the reminders need.

import { create } from 'zustand';

/** The facts the reminders need, from what the restore told. */
export interface AfterRestore {
  /** The assistant was on, so the OpenRouter key must be typed again. */
  assistantWasOn: boolean;
  /** The copy has the cloud copy switched on and this computer has no cloud keys. */
  cloudNeedsKeys: boolean;
}

interface AfterRestoreState {
  /** Set when a restore has just finished and the Owner has not yet seen the reminders. */
  pending: AfterRestore | null;
  remember: (facts: AfterRestore) => void;
  clear: () => void;
}

export const useAfterRestoreStore = create<AfterRestoreState>()((set) => ({
  pending: null,
  remember: (facts) => {
    set({ pending: facts });
  },
  clear: () => {
    set({ pending: null });
  },
}));
