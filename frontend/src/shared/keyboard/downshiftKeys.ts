// Downshift (the combobox under every picker and the search box) answers some keys by itself. A
// key that the app's own order decides, such as Esc in PRD FR-7.18, must reach that order, so the
// screen tells Downshift to leave it alone. Without this, Downshift calls preventDefault on Esc
// and the shared Esc dispatcher (escape.ts) sees a key that is already taken.

import type { KeyboardEvent } from 'react';

type GuardedKeyEvent = KeyboardEvent<HTMLInputElement> & { preventDownshiftDefault?: boolean };

/** Tells Downshift that this key is handled by the app, so its own handler does not run. */
export function keepKeyForApp(event: KeyboardEvent<HTMLInputElement>): void {
  (event as GuardedKeyEvent).preventDownshiftDefault = true;
}
