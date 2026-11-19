// How many columns of a view fit the window (NFR-USE-15, Code Standards F22): a table never scrolls
// sideways, so a view that has more columns than the window holds shows the most useful ones, and
// views D and E, whose columns are one for each period or item, show a window of them with buttons
// to move it. The widths are those of the grid's column types (F18); the room is what the grid has
// at each layout step, measured at 1024, 1366 and 1920 px, less a margin for a scroll bar.

import { LIST_COLUMN_WIDTHS, LIST_GROW_MIN_WIDTH } from '../../shared/grid/columnTypes';
import type { LayoutStep } from '../../shared/hooks/useLayoutStep';
import type { ReportColumnKind } from '../../bindings/ReportColumnKind';

/** The width of the grid at each step, less 30 px for a scroll bar and the focus outline. */
export const STEP_ROOM: Readonly<Record<LayoutStep, number>> = {
  compact: 860,
  normal: 1040,
  wide: 1500,
};

/**
 * The width a column of this kind takes in the grid (a growing name column starts at its least). A
 * figure that can be a loss (`signed`) has a slot after it for its word, like a balance.
 */
export function columnWidth(kind: ReportColumnKind, key: string, signed = false): number {
  if (signed && kind === 'money') return LIST_COLUMN_WIDTHS.balance;
  switch (kind) {
    case 'party':
    case 'names':
      return LIST_GROW_MIN_WIDTH;
    case 'money':
    case 'percent':
      return LIST_COLUMN_WIDTHS.figure;
    case 'number':
      // A rank has the room of a short word: the headings of H are written in its column.
      if (key === 'avg_days') return LIST_COLUMN_WIDTHS.figure;
      return key === 'rank' ? LIST_COLUMN_WIDTHS.short : LIST_COLUMN_WIDTHS.count;
    case 'balance':
      return LIST_COLUMN_WIDTHS.balance;
    case 'date':
      return LIST_COLUMN_WIDTHS.date;
    case 'doc':
      return LIST_COLUMN_WIDTHS.status;
    case 'label':
      // A state is one short word (a garden is open or closed); any other label is a sentence.
      return key === 'status' ? LIST_COLUMN_WIDTHS.status : LIST_COLUMN_WIDTHS.text;
    default:
      return LIST_COLUMN_WIDTHS.text;
  }
}

export interface FitItem {
  key: string;
  width: number;
}

/**
 * The keys that fit `room`, taking the columns in the order of `priority` (most useful first) and
 * leaving a column out when it would not fit, so a narrower one behind it may still. A column the
 * list does not name comes last, in its own order. At least one column is always kept.
 */
export function fitColumns(
  items: readonly FitItem[],
  priority: readonly string[],
  room: number,
): Set<string> {
  const rank = (key: string) => {
    const found = priority.indexOf(key);
    return found === -1 ? priority.length : found;
  };
  const ordered = items
    .map((item, index) => ({ item, index }))
    .toSorted((a, b) => rank(a.item.key) - rank(b.item.key) || a.index - b.index);
  const chosen = new Set<string>();
  let used = 0;
  for (const { item } of ordered) {
    if (used + item.width <= room) {
      chosen.add(item.key);
      used += item.width;
    }
  }
  const first = ordered[0];
  if (chosen.size === 0 && first !== undefined) chosen.add(first.item.key);
  return chosen;
}

export interface ColumnWindow {
  /** The first of the headed columns shown (0 when they all fit). */
  start: number;
  /** How many headed columns are shown. */
  count: number;
  /** How many there are. */
  total: number;
}

/**
 * A window of the headed columns of views D and E (one for each period or item): as many as fit
 * beside the columns that always show, starting at `start` (kept inside the list).
 */
export function windowOf(
  total: number,
  fixedWidth: number,
  headedWidth: number,
  room: number,
  start: number,
): ColumnWindow {
  const capacity = Math.max(1, Math.floor((room - fixedWidth) / headedWidth));
  const count = Math.min(total, capacity);
  const last = Math.max(0, total - count);
  return { start: Math.min(Math.max(0, start), last), count, total };
}
