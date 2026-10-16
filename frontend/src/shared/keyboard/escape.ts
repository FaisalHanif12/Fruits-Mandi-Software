// The Esc order of PRD FR-7.18, kept in one place (Code Standards F27):
//   1. close an open list or dialog (`overlay`);
//   2. else cancel the typing in the current cell or field (`edit`);
//   3. else leave the screen, asking first if there is unsaved work (`leave`).
// Parts that can react to Esc register a handler with their level while they are active. One
// listener (useEscapeDispatcher, mounted once by the shell) runs the highest level, and within a
// level the one registered last, so a list opened inside a dialog closes before the dialog.

import { useEffect, useRef } from 'react';

export type EscapeLevel = 'overlay' | 'edit' | 'leave';

const PRIORITY: Record<EscapeLevel, number> = { overlay: 3, edit: 2, leave: 1 };

interface Entry {
  level: EscapeLevel;
  handler: () => void;
  /** Asked when Esc is pressed, so the answer is always current. */
  isActive: () => boolean;
  order: number;
}

const entries = new Set<Entry>();
let nextOrder = 0;

/**
 * Adds a handler. It answers only while `isActive()` is true, asked at the moment Esc is
 * pressed. Returns the function that removes it.
 */
export function registerEscape(
  level: EscapeLevel,
  handler: () => void,
  isActive: () => boolean = () => true,
): () => void {
  nextOrder += 1;
  const entry: Entry = { level, handler, isActive, order: nextOrder };
  entries.add(entry);
  return () => {
    entries.delete(entry);
  };
}

/**
 * Runs the handler that should answer Esc now. True when one ran. With `only`, handlers of lower
 * levels are left out: a dialog asks for `'overlay'` to learn whether an open list inside it must
 * close first (PRD FR-7.18).
 */
export function runEscape(only?: EscapeLevel): boolean {
  let best: Entry | undefined;
  for (const entry of entries) {
    if (only !== undefined && entry.level !== only) continue;
    if (!entry.isActive()) continue;
    if (
      best === undefined ||
      PRIORITY[entry.level] > PRIORITY[best.level] ||
      (entry.level === best.level && entry.order > best.order)
    ) {
      best = entry;
    }
  }
  if (best === undefined) return false;
  best.handler();
  return true;
}

/**
 * Answers Esc with `handler` at `level` while `active` is true. `active` may be a function, asked
 * when Esc is pressed, for things the screen cannot learn in time through state (a grid cell
 * that has just started to be typed in).
 */
export function useEscape(
  level: EscapeLevel,
  handler: () => void,
  active: boolean | (() => boolean) = true,
): void {
  const handlerRef = useRef(handler);
  const activeRef = useRef(active);
  useEffect(() => {
    handlerRef.current = handler;
    activeRef.current = active;
  }, [handler, active]);
  useEffect(
    () =>
      registerEscape(
        level,
        () => {
          handlerRef.current();
        },
        () => (typeof activeRef.current === 'boolean' ? activeRef.current : activeRef.current()),
      ),
    [level],
  );
}

/** Mount once in the shell: sends the Esc key to the handler chosen by the order above. */
export function useEscapeDispatcher(): void {
  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key !== 'Escape' || event.altKey || event.ctrlKey || event.shiftKey) return;
      if (event.defaultPrevented) return;
      if (runEscape()) event.preventDefault();
    };
    window.addEventListener('keydown', onKeyDown);
    return () => {
      window.removeEventListener('keydown', onKeyDown);
    };
  }, []);
}
