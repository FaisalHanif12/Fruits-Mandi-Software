import { useEffect, useRef } from 'react';

/**
 * The fields that can take focus. A drop-down (Radix Select) draws a hidden native select next to
 * its button for the first moment it is on screen (`aria-hidden`); focus cannot go to it, so it is
 * skipped, and the first field the user can see takes the focus (a report that starts with the
 * period drop-down puts the focus on the first date).
 */
const FIRST_FIELD = 'input:not([type="hidden"]), select:not([aria-hidden="true"]), textarea';

/**
 * A ref for the element that holds a screen's fields: the first field has focus when the screen
 * opens, so the user can start typing at once (PRD FR-7.1, NFR-USE-7). When `refocusKey` changes
 * (a wizard moves to its next step), the first field takes focus again.
 */
export function useFocusFirstField<T extends HTMLElement>(refocusKey?: unknown) {
  const ref = useRef<T>(null);
  useEffect(() => {
    ref.current?.querySelector<HTMLElement>(FIRST_FIELD)?.focus();
  }, [refocusKey]);
  return ref;
}

/** A text box the user can type in: not hidden, not read only, not switched off. */
const TEXT_BOX =
  'input:not([type="hidden"]):not([readonly]):not(:disabled), textarea:not(:disabled)';

/**
 * Puts the focus in the first box of `root` that is still empty (plan 32, R8, decision D1): the
 * Index form a picker opens starts with the text already typed there, so the cursor goes to the
 * first thing left to write. When every box has something in it, the first box takes the focus.
 */
export function focusFirstEmptyField(root: HTMLElement | null): void {
  if (root === null) return;
  const boxes = Array.from(root.querySelectorAll<HTMLInputElement | HTMLTextAreaElement>(TEXT_BOX));
  (boxes.find((box) => box.value === '') ?? boxes[0])?.focus();
}
