// Text size (PRD FR-7.11, NFR-USE-18): Ctrl + and Ctrl − go up and down in steps of 10, from 80%
// to 150%, and Ctrl 0 goes back to 100%. The whole screen is scaled, so the layout stays aligned
// at every step. It stops at the step where the content would be narrower than 1024 points
// (NFR-USE-15): at 1366 points wide the largest step is 130%, at 1024 it is 100%.
// Saving the choice per user comes with the preferences command (plan 04, prefs_set); until then
// the size lives in this store.

import { useEffect } from 'react';
import { create } from 'zustand';
import { SHORTCUTS, useShortcut } from '../../shared/keyboard';

export const MIN_TEXT_SIZE = 80;
export const DEFAULT_TEXT_SIZE = 100;
export const MAX_TEXT_SIZE = 150;
export const TEXT_SIZE_STEP = 10;
/** The content area is never narrower than this many points (NFR-USE-15). */
export const MIN_CONTENT_WIDTH = 1024;

/** The largest step that still leaves a content area of at least 1024 points. */
export function maxTextSizeFor(windowWidth: number): number {
  const fits =
    Math.floor((windowWidth * 100) / MIN_CONTENT_WIDTH / TEXT_SIZE_STEP) * TEXT_SIZE_STEP;
  return Math.max(DEFAULT_TEXT_SIZE, Math.min(MAX_TEXT_SIZE, fits));
}

/** The size to use: the user's choice, held back by the window width. */
export function effectiveTextSize(chosen: number, windowWidth: number): number {
  return Math.min(chosen, maxTextSizeFor(windowWidth));
}

/** One step bigger or smaller, within 80 to 150, and within what the window allows. */
export function stepTextSize(current: number, direction: 1 | -1, windowWidth: number): number {
  const wanted = current + direction * TEXT_SIZE_STEP;
  return Math.max(MIN_TEXT_SIZE, Math.min(maxTextSizeFor(windowWidth), wanted));
}

interface TextSizeState {
  /** What the user chose. The size in use may be smaller in a narrow window. */
  chosen: number;
  setChosen: (size: number) => void;
}

export const useTextSizeStore = create<TextSizeState>()((set) => ({
  chosen: DEFAULT_TEXT_SIZE,
  setChosen: (size) => {
    set({ chosen: size });
  },
}));

/**
 * Mount once in the shell: answers Ctrl +, Ctrl − and Ctrl 0, and scales the page. Scaling uses
 * the CSS `zoom` of the page, which WebView2 supports, so every size in the layout follows.
 */
export function useTextSize(): void {
  const chosen = useTextSizeStore((state) => state.chosen);
  const setChosen = useTextSizeStore((state) => state.setChosen);
  const width = () => window.innerWidth;
  useShortcut(SHORTCUTS.textBigger, () => {
    setChosen(stepTextSize(effectiveTextSize(chosen, width()), 1, width()));
  });
  useShortcut(SHORTCUTS.textSmaller, () => {
    setChosen(stepTextSize(effectiveTextSize(chosen, width()), -1, width()));
  });
  useShortcut(SHORTCUTS.textNormal, () => {
    setChosen(DEFAULT_TEXT_SIZE);
  });
  useEffect(() => {
    const apply = () => {
      const size = effectiveTextSize(chosen, window.innerWidth);
      document.documentElement.style.zoom = String(size / 100);
    };
    apply();
    window.addEventListener('resize', apply);
    return () => {
      window.removeEventListener('resize', apply);
    };
  }, [chosen]);
}
