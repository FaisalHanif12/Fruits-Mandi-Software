// The place of a list grid on a screen (plan 38 G2). A short list is as tall as its rows. A longer
// one fills the height the screen leaves: its heading and its totals row stay fixed and the rows
// scroll between them, and the page buttons under it stay in view. On a screen that is not given a
// fixed height (a list under a form) the region keeps a floor of a few rows and the page scrolls.
// Written once here, so no list works this out for itself.

import type { ReactNode } from 'react';

/** A list of this many rows or fewer is as tall as its rows; a longer one fills the screen. */
export const SHORT_LIST_ROWS = 10;

/** True for a list that fills the room the screen leaves, with the rows scrolling inside it. */
export function isLongList(rowCount: number): boolean {
  return rowCount > SHORT_LIST_ROWS;
}

/** The `heightMode` of a `ListGrid` that is drawn inside a `ListRegion` with the same row count. */
export function listHeightMode(rowCount: number): 'auto' | 'fill' {
  return isLongList(rowCount) ? 'fill' : 'auto';
}

interface ListRegionProps {
  /** The rows on the page on screen. */
  rowCount: number;
  children: ReactNode;
}

export function ListRegion({ rowCount, children }: ListRegionProps) {
  return (
    <div className={isLongList(rowCount) ? 'min-h-list-region flex-1' : undefined}>{children}</div>
  );
}
