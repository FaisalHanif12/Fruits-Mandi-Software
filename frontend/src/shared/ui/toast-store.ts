// The queue of short messages at the bottom end corner (PRD FR-7.7). Any screen calls
// showToast; the one Toaster in the app shell shows them.

import type { ReactNode } from 'react';
import { create } from 'zustand';
import type { Shortcut } from '../keyboard';

export type ToastTone = 'success' | 'error';

/** A button inside a toast, for example "واپس لیں Alt+Z" after a save (PRD FR-7.47). */
export interface ToastAction {
  /** The words of the button. */
  label: ReactNode;
  /** The same words as plain text, for screen readers (Radix asks for it). */
  altText: string;
  /** A key that does the same while the toast is on screen. Shown on the button. */
  shortcut?: Shortcut;
  onAction: () => void;
}

export interface ToastOptions {
  action?: ToastAction;
  /** How long the toast stays, in milliseconds. Default: TOAST_DURATION_MS. */
  durationMs?: number;
}

export interface ToastItem extends ToastOptions {
  id: number;
  tone: ToastTone;
  /** Words from Fluent: a <Localized> element or a string from l10n.getString. */
  message: ReactNode;
}

interface ToastState {
  toasts: readonly ToastItem[];
  show: (tone: ToastTone, message: ReactNode, options?: ToastOptions) => void;
  dismiss: (id: number) => void;
}

let nextToastId = 1;

export const useToastStore = create<ToastState>()((set) => ({
  toasts: [],
  show: (tone, message, options = {}) => {
    const id = nextToastId;
    nextToastId += 1;
    set((state) => ({ toasts: [...state.toasts, { id, tone, message, ...options }] }));
  },
  dismiss: (id) => {
    set((state) => ({ toasts: state.toasts.filter((toast) => toast.id !== id) }));
  },
}));

/** Shows a message that disappears by itself, for example "بکری 1 محفوظ ہو گئی". */
export function showToast(tone: ToastTone, message: ReactNode, options?: ToastOptions): void {
  useToastStore.getState().show(tone, message, options);
}
