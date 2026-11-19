// The backup key between "made" and "the Owner has it on paper" (plan 18, PRD FR-20.10). The key is
// shown once, so while the step is open it lives here, above the screens: a language switch or a
// redraw never loses it and never asks the backend to make a second one (Code Standards F39). It
// is dropped the moment the step is over. It is never written anywhere else.

import { create } from 'zustand';

interface BackupKeyState {
  /** `XXXX-XXXX-XXXX-XXXX-XXXX-XXXX`, or null when none is made yet. */
  key: string | null;
  /** When it was made (UTC), for the printed sheet. */
  createdAt: string | null;
  /** The Owner ticked that he printed it and put it in a safe place. */
  printed: boolean;
  /** What he typed from the sheet to prove he has it. */
  typed: string;
  set: (key: string, createdAt: string) => void;
  setPrinted: (printed: boolean) => void;
  setTyped: (typed: string) => void;
  clear: () => void;
}

export const useBackupKeyStore = create<BackupKeyState>()((set) => ({
  key: null,
  createdAt: null,
  printed: false,
  typed: '',
  set: (key, createdAt) => {
    set({ key, createdAt, printed: false, typed: '' });
  },
  setPrinted: (printed) => {
    set({ printed });
  },
  setTyped: (typed) => {
    set({ typed });
  },
  clear: () => {
    set({ key: null, createdAt: null, printed: false, typed: '' });
  },
}));
