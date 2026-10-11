// The one way a screen reacts to a shortcut key (Code Standards F27). Screens never add their
// own key listeners.

import { useEffect, useRef } from 'react';
import { matchesShortcut, type Shortcut } from './shortcuts';

interface UseShortcutOptions {
  /** False while the screen is not the one that should answer the key. Default true. */
  enabled?: boolean;
}

/** Runs `handler` when `shortcut` is pressed anywhere in the window. */
export function useShortcut(
  shortcut: Shortcut,
  handler: () => void,
  { enabled = true }: UseShortcutOptions = {},
): void {
  const handlerRef = useRef(handler);
  useEffect(() => {
    handlerRef.current = handler;
  }, [handler]);

  useEffect(() => {
    if (!enabled) return undefined;
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.repeat || !matchesShortcut(event, shortcut)) return;
      event.preventDefault();
      handlerRef.current();
    };
    window.addEventListener('keydown', onKeyDown);
    return () => {
      window.removeEventListener('keydown', onKeyDown);
    };
  }, [shortcut, enabled]);
}
