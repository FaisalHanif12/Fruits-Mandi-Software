// Which dialog owns the entry keys now (Code Standards F27).
//
// A form dialog opened over an entry screen (the Index form that a picker's "add" button opens,
// the quick add of a city) has its own save key. The screen under it has one too, and both are
// listeners on the window. Without a rule, F10 inside the dialog would save the dialog and also
// the sale under it. So every open form dialog takes a layer, and a shortcut of an entry or a grid
// (save, save and next, new entry, copy from above) answers only if it belongs to the top layer.
// Keys that work everywhere (language, search, text size) are not held back. Esc follows the same
// rule (escape.ts): while a form dialog is open, only the parts inside it answer Esc.

import { createContext, useContext, useEffect, useId } from 'react';
import type { Shortcut } from './shortcuts';

/** The open dialogs, the last one on top. */
const layers: string[] = [];

/** The layer a hook or a component sits in, or null for the screen itself. */
export const ModalLayerContext = createContext<string | null>(null);

/** Takes a layer while `open` is true. Returns the id to give to the dialog's content. */
export function useModalLayer(open: boolean): string {
  const id = useId();
  useEffect(() => {
    if (!open) return undefined;
    layers.push(id);
    return () => {
      const at = layers.lastIndexOf(id);
      if (at >= 0) layers.splice(at, 1);
    };
  }, [open, id]);
  return id;
}

/** The layer of the dialog that is open on top now, or undefined when no form dialog is open. */
export function topLayer(): string | undefined {
  return layers.at(-1);
}

/** True when a dialog is open above `own`, so an entry or grid key must not answer from there. */
export function heldBackByDialog(shortcut: Shortcut, own: string | null): boolean {
  if (shortcut.scope !== 'entry' && shortcut.scope !== 'grid') return false;
  const top = layers.at(-1);
  return top !== undefined && top !== own;
}

/** The layer of the component calling this. */
export function useOwnLayer(): string | null {
  return useContext(ModalLayerContext);
}
