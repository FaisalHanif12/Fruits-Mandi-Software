// What the user has typed on the partner setup, kept above the screens (Code Standards F39): the
// share of each partner as text, and the day the shares start. The setup page and the first-run
// wizard share it, so a language switch, a dialog or a step change loses nothing. A partner with
// no entry shows the share he has saved. Everything is text, as typed (CLAUDE.md rule 1).

import { create } from 'zustand';

interface SetupState {
  /** The share text typed, by partner id. */
  shareEdits: Readonly<Record<number, string>>;
  /** The "applies from" date as typed (DD-MM-YYYY), or null while the default (today) shows. */
  fromEdit: string | null;
  setShare: (partnerId: number, text: string) => void;
  setFrom: (text: string) => void;
  /** After a save, or when the user leaves for good. */
  clear: () => void;
}

export const usePartnerSetupStore = create<SetupState>()((set) => ({
  shareEdits: {},
  fromEdit: null,
  setShare: (partnerId, text) => {
    set((state) => ({ shareEdits: { ...state.shareEdits, [partnerId]: text } }));
  },
  setFrom: (text) => {
    set({ fromEdit: text });
  },
  clear: () => {
    set({ shareEdits: {}, fromEdit: null });
  },
}));
