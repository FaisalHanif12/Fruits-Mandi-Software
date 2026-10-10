// The queue of short messages at the bottom end corner (PRD FR-7.7). Any screen calls
// showToast; the one Toaster in the app shell shows them.

import type { ReactNode } from 'react';
import { create } from 'zustand';

export type ToastTone = 'success' | 'error';

export interface ToastItem {
  id: number;
  tone: ToastTone;
  /** Words from Fluent: a <Localized> element or a string from l10n.getString. */
  message: ReactNode;
}

interface ToastState {
  toasts: readonly ToastItem[];
  show: (tone: ToastTone, message: ReactNode) => void;
  dismiss: (id: number) => void;
}

let nextToastId = 1;

export const useToastStore = create<ToastState>()((set) => ({
  toasts: [],
  show: (tone, message) => {
    const id = nextToastId;
    nextToastId += 1;
    set((state) => ({ toasts: [...state.toasts, { id, tone, message }] }));
  },
  dismiss: (id) => {
    set((state) => ({ toasts: state.toasts.filter((toast) => toast.id !== id) }));
  },
}));

/** Shows a message that disappears by itself, for example "بکری 1 محفوظ ہو گئی". */
export function showToast(tone: ToastTone, message: ReactNode): void {
  useToastStore.getState().show(tone, message);
}
