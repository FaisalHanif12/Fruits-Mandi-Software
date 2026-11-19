// Backspace and Delete only delete (PRD FR-7.91, FR-7.14, FR-7.15, D8 Bug 2). In a text field they
// delete text. Everywhere else they do nothing here: a grid cell clears itself in `shared/grid`
// (gridKeys.ts), and on anything else the key is simply not left to the web view.
//
// Why a rule is needed at all: with the focus on something that is not a text field (a grid cell, a
// button, the page), WebKit, the web view of the Mac app, takes Backspace as "go back" in its history.
// The screen's leave guard then stops the way out and asks "Save changes?" (app/LeaveGuard.tsx, FR-7.9),
// although the user only wanted to delete. Cancelling the key press stops that. The one listener is
// mounted once with the Esc dispatcher (escape.ts), so it serves every screen and no screen writes its
// own key handler (Code Standards F27).

import { useEffect } from 'react';

/** The kinds of `<input>` in which Backspace deletes text. */
const TEXT_INPUT_TYPES: ReadonlySet<string> = new Set([
  'text',
  'search',
  'tel',
  'url',
  'email',
  'password',
  'number',
  'date',
  'datetime-local',
  'month',
  'time',
  'week',
]);

/**
 * Is this the element where Backspace deletes text: a text field that can be typed in, or an area of
 * editable text? A read-only or disabled field has no text to delete, so it is not one.
 */
export function takesTyping(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) return false;
  if (target.isContentEditable) return true;
  if (target instanceof HTMLTextAreaElement) return !target.readOnly && !target.disabled;
  if (target instanceof HTMLInputElement) {
    return TEXT_INPUT_TYPES.has(target.type) && !target.readOnly && !target.disabled;
  }
  return false;
}

/**
 * Cancels a Backspace that is not meant for a text field, so the web view cannot take it as "go back".
 * It runs after the elements under it, so a grid cell that cleared itself has already answered.
 */
export function guardBackspace(event: KeyboardEvent): void {
  if (event.key !== 'Backspace' || takesTyping(event.target)) return;
  event.preventDefault();
}

/** Mount once in the shell (with the Esc dispatcher): no Backspace outside a text field goes back. */
export function useBackspaceGuard(): void {
  useEffect(() => {
    window.addEventListener('keydown', guardBackspace);
    return () => {
      window.removeEventListener('keydown', guardBackspace);
    };
  }, []);
}
