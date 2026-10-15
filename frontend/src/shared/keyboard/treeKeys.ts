// Arrow keys in a tree (PRD FR-7.13, FR-7.17): ↑ ↓ Home End Page Up Page Down move between rows
// (the same moves as every menu and list, `moveIndex`), the arrow that points at the children
// opens a row or goes in, the arrow that points at the parent closes it or goes out, and Enter
// opens. A screen with a tree gives this the things only it knows (how many rows, what opening
// means) and writes no key handler of its own (Code Standards F27).

import type { KeyboardEvent } from 'react';
import { arrowSide, elementDirection } from './direction';
import { DEFAULT_PAGE_SIZE, moveIndex } from './rovingFocus';

export interface TreeKeyOptions {
  /** How many rows the tree has now. */
  count: number;
  /** The row that has focus. */
  active: number;
  /** Rows per Page Up or Page Down. */
  pageSize?: number | undefined;
  /** A vertical key moved to this row. */
  onMove: (index: number) => void;
  /**
   * The arrow that points at the children (`opens`) or at the parent (`closes`) was pressed. True
   * when it did something, so the key is taken and the page does not scroll.
   */
  onHorizontal: (action: 'opens' | 'closes') => boolean;
  /** Enter on the row. */
  onEnter: () => void;
}

/** The key handler of a tree. Put it on the element that has focus. */
export function treeKeyHandler({
  count,
  active,
  pageSize = DEFAULT_PAGE_SIZE,
  onMove,
  onHorizontal,
  onEnter,
}: TreeKeyOptions) {
  return (event: KeyboardEvent<HTMLElement>): void => {
    if (event.altKey || event.ctrlKey || event.metaKey || event.shiftKey) return;
    const side = arrowSide(event.key);
    if (side !== null) {
      // The arrow that points away from the start edge leads to the children.
      const toChildren = elementDirection(event.currentTarget) === 'rtl' ? 'left' : 'right';
      if (onHorizontal(side === toChildren ? 'opens' : 'closes')) event.preventDefault();
      return;
    }
    if (event.key === 'Enter') {
      event.preventDefault();
      onEnter();
      return;
    }
    const to = moveIndex(active, count, event.key, pageSize);
    if (to === null) return;
    event.preventDefault();
    onMove(to);
  };
}
