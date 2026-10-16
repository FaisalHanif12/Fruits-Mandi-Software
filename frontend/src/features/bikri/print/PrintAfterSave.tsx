// Puts the Ctrl+Enter print in place while the sale book is on screen (PRD FR-7.57). It draws
// nothing. It is a component of its own, and not a hook of the page, so the words and settings it
// reads re-render only this empty component: a re-render of the whole sale book makes the grids
// close the cell that is being typed in.

import { usePrintAfterSave } from './usePrintAfterSave';

export function PrintAfterSave() {
  usePrintAfterSave();
  return null;
}
