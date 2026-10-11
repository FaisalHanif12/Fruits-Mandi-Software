// Fixed widths and looks per column type (Code Standards F17, F18): the same type has the same
// width on every screen. Number-like cells sit on the physical right edge in both languages
// (NFR-USE-19); the CSS class `cell-num` does that in grid.css.

import type { EntryColumnType } from './types';

/** Column width in pixels, one per type (F18). */
export const COLUMN_WIDTHS: Record<EntryColumnType, number> = {
  account: 280,
  money: 140,
  qty: 100,
  rate: 160,
  date: 140,
  text: 240,
  calc: 140,
};

/** The types whose cells are numbers, codes or dates: Latin digits, left to right, tabular. */
const NUMBER_LIKE: ReadonlySet<EntryColumnType> = new Set(['money', 'qty', 'rate', 'date', 'calc']);

export function isNumberLike(type: EntryColumnType): boolean {
  return NUMBER_LIKE.has(type);
}

/**
 * The columns of a read-only list (ListGrid). Same rule as above: one type, one width (F18). A
 * `name` column is wide enough for a Nastaliq name and the other script's name beside it.
 */
export type ListColumnType = 'code' | 'name' | 'text' | 'number' | 'money' | 'date' | 'status';

export const LIST_COLUMN_WIDTHS: Record<ListColumnType, number> = {
  code: 120,
  name: 260,
  text: 200,
  number: 140,
  money: 140,
  date: 140,
  status: 120,
};

/** Codes and numbers are Latin digits, written left to right and set against the right edge. */
export function isListNumberLike(type: ListColumnType): boolean {
  return type === 'code' || type === 'number' || type === 'money' || type === 'date';
}
