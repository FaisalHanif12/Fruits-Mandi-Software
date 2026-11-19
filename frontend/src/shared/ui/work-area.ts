// The work area of the shell: the part of the window beside the main menu where a screen is drawn.
// A form that must look like a screen of its own, the Index form a picker's "add" button opens
// (plan 32, R8, decision D1), is drawn over exactly this part, so the main menu stays in view with
// انڈکس marked as the current area.
//
// The shell gives its element here when it is on screen, and the host of the Index form (which sits
// above the shell in the tree) reads it. `indexFormOpen` tells the main menu to mark انڈکس while the
// form is open.

import { create } from 'zustand';

interface WorkAreaState {
  /** The element a full-area form is drawn in, or null when no shell is on screen. */
  slot: HTMLElement | null;
  setSlot: (slot: HTMLElement | null) => void;
  /** True while the Index form of a picker's "add" button is open. */
  indexFormOpen: boolean;
  setIndexFormOpen: (open: boolean) => void;
}

export const useWorkAreaStore = create<WorkAreaState>((set) => ({
  slot: null,
  setSlot: (slot) => {
    set({ slot });
  },
  indexFormOpen: false,
  setIndexFormOpen: (indexFormOpen) => {
    set({ indexFormOpen });
  },
}));
