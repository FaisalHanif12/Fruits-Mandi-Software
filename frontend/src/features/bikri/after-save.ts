// What follows a save (PRD FR-7.57, plan 13). `Ctrl+Enter` saves, prints the documents ticked in
// "print after save", and opens a new بکری. Printing is plan 14: it registers its function here,
// and the save calls it only for `Ctrl+Enter`. Until then `Ctrl+Enter` saves and opens a new
// بکری, the same as F10, and prints nothing.

import type { BikriSaved } from '../../bindings/BikriSaved';

export type PrintAfterSave = (saved: BikriSaved) => void;

const printers: PrintAfterSave[] = [];

/** Plan 14 calls this once, with the function that prints the ticked documents of a saved بکری. */
export function registerPrintAfterSave(print: PrintAfterSave): () => void {
  printers.push(print);
  return () => {
    const at = printers.indexOf(print);
    if (at >= 0) printers.splice(at, 1);
  };
}

/** Runs after a successful save. `andPrint` is true when the save was started with Ctrl+Enter. */
export function afterSave(saved: BikriSaved, andPrint: boolean): void {
  if (!andPrint) return;
  for (const print of printers) print(saved);
}
