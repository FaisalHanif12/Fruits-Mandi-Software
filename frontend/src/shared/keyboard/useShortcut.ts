// The one way a screen reacts to a shortcut key (Code Standards F27). Screens never add their
// own key listeners.

import { useEffect, useRef } from 'react';
import { heldBackByDialog, useOwnLayer } from './modalLayer';
import { matchesShortcut, type Shortcut } from './shortcuts';

interface UseShortcutOptions {
  /** False while the screen is not the one that should answer the key. Default true. */
  enabled?: boolean;
  /**
   * A CSS selector for the controls that own this key themselves (a list that turns its page with Page
   * Up and Page Down, an open picker list). While the focus is inside one, the shortcut does not answer
   * and does not stop the key, so that control keeps it. Default: it answers wherever the focus is.
   */
  unlessFocusIn?: string;
}

/** Runs `handler` when `shortcut` is pressed anywhere in the window. */
export function useShortcut(
  shortcut: Shortcut,
  handler: () => void,
  { enabled = true, unlessFocusIn }: UseShortcutOptions = {},
): void {
  const handlerRef = useRef(handler);
  const layer = useOwnLayer();
  useEffect(() => {
    handlerRef.current = handler;
  }, [handler]);

  useEffect(() => {
    if (!enabled) return undefined;
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.repeat || !matchesShortcut(event, shortcut)) return;
      // A dialog above this screen owns the entry keys (see modalLayer.ts).
      if (heldBackByDialog(shortcut, layer)) return;
      if (unlessFocusIn !== undefined) {
        const focused = document.activeElement;
        if (focused instanceof Element && focused.closest(unlessFocusIn) !== null) return;
      }
      event.preventDefault();
      handlerRef.current();
    };
    window.addEventListener('keydown', onKeyDown);
    return () => {
      window.removeEventListener('keydown', onKeyDown);
    };
  }, [shortcut, enabled, layer, unlessFocusIn]);
}
