// Where the guide is (PRD FR-7.12): the number of the step that is open, or null when the guide is
// closed. It lives in a store above the screens, so the guide stays open while the Owner moves from
// screen to screen with it, and a language switch loses nothing (Code Standards F39).

import { create } from 'zustand';
import { GUIDE_STEPS } from './steps';

interface GuideState {
  /** The open step, counted from 0, or null when the guide is closed. */
  step: number | null;
  open: (step: number) => void;
  close: () => void;
}

export const useGuideStore = create<GuideState>((set) => ({
  step: null,
  open: (step) => {
    set({ step: Math.max(0, Math.min(step, GUIDE_STEPS.length - 1)) });
  },
  close: () => {
    set({ step: null });
  },
}));
