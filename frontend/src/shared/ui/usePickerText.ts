// The text a user is typing in a picker, and the text as the last key left it.
//
// React state is one render behind inside an event: when Enter picks a row and moves focus on,
// the field blurs in the same event, and a blur handler that reads the state still sees the text
// from before the pick (it would resolve a short code twice and show "more than one account"
// under a filled field). So the latest text is also kept where an event handler can read it at
// once. This sits in a hook of its own so the screens that use it only call `readTyped()`.

import { useCallback, useRef, useState } from 'react';

export function usePickerText() {
  // What the user is typing, or null while the field shows the chosen name.
  const [typed, setTypedState] = useState<string | null>(null);
  const latest = useRef<string | null>(null);
  const setTyped = useCallback((next: string | null) => {
    latest.current = next;
    setTypedState(next);
  }, []);
  /** The text as the last key left it. Call it from an event handler, never while drawing. */
  const readTyped = useCallback(() => latest.current, []);
  return { typed, setTyped, readTyped };
}
