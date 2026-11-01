// The box that asks for the partner code, from anywhere (plan 44, PRD FR-17.11): a locked card, a
// locked table or a refused report opens it with one call, and the shell shows it once. After the
// code is right the box closes and every figure on screen is read again.

import { create } from 'zustand';

interface PartnerCodeDialogState {
  open: boolean;
  show: () => void;
  hide: () => void;
}

export const usePartnerCodeDialog = create<PartnerCodeDialogState>()((set) => ({
  open: false,
  show: () => {
    set({ open: true });
  },
  hide: () => {
    set({ open: false });
  },
}));

/** Opens the partner code box. A plain function, so a screen can pass it as a click handler. */
export function openPartnerCode(): void {
  usePartnerCodeDialog.getState().show();
}
