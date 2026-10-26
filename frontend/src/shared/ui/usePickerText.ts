// The text a user is typing in a picker, and the text as the last key left it.
//
// React state is one render behind inside an event: when Enter picks a row and moves focus on,
// the field blurs in the same event, and a blur handler that reads the state still sees the text
// from before the pick (it would resolve a short code twice and show "more than one account"
// under a filled field). So the latest text is also kept where an event handler can read it at
// once. This sits in a hook of its own so the screens that use it only call `readTyped()`.

import { useCallback, useRef, useState } from 'react';

/**
 * A yes/no that a handler sets and another reads later, without drawing again. The picker uses it
 * to know the Index form of its "add" button is open above the screen (plan 32, R8). It is a hook
 * of its own for the same reason as `usePickerText`: a screen only calls `get()` and `set()`.
 */
export function usePickerFlag() {
  const flag = useRef(false);
  const get = useCallback(() => flag.current, []);
  const set = useCallback((next: boolean) => {
    flag.current = next;
  }, []);
  return { get, set };
}

export function usePickerText(initial: string | null = null) {
  // What the user is typing, or null while the field shows the chosen name. A picker in a grid
  // cell starts with the key that opened it already typed.
  const [typed, setTypedState] = useState<string | null>(initial);
  const latest = useRef<string | null>(initial);
  const setTyped = useCallback((next: string | null) => {
    latest.current = next;
    setTypedState(next);
  }, []);
  /** The text as the last key left it. Call it from an event handler, never while drawing. */
  const readTyped = useCallback(() => latest.current, []);
  return { typed, setTyped, readTyped };
}
