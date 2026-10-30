// Fixed widths and looks per column type (Code Standards F17, F18): the same type has the same
// width on every screen. Number-like cells sit on the physical right edge in both languages
// (NFR-USE-19); the CSS class `cell-num` does that in grid.css.

import type { EntryColumnType } from './types';

/** Column width in pixels, one per type (F18). */
export const COLUMN_WIDTHS: Record<EntryColumnType, number> = {
  account: 280,
  money: 140,
  qty: 100,
  weight: 120,
  rate: 160,
  date: 140,
  text: 240,
  calc: 140,
  label: 280,
  picker: 200,
  choice: 120,
};

/**
 * How a grid that must fit its room (`fit`) shares the free width between its columns: names get
 * the most, numbers the least, so a name is not cut while a figure has room to spare.
 */
export const FIT_WEIGHTS: Record<EntryColumnType, number> = {
  account: 300,
  money: 100,
  qty: 80,
  weight: 90,
  rate: 100,
  date: 100,
  text: 240,
  calc: 110,
  label: 200,
  picker: 300,
  choice: 100,
};

/**
 * Named least widths a column of a `fit` grid can ask for instead of its type's (`minWidth` in the
 * column). Words, not pixels, so a screen never writes a raw width (Code Standards F9).
 */
export const FIT_MIN = {
  /** A name that is read in full to tell two parties apart: a buyer. */
  longName: 150,
  /** A name that is short in practice: a seller. */
  name: 100,
  /** A heading that is one long word, such as "Munshiana". */
  longHeading: 96,
  /** A heading of a number column that is one word, such as "Received". */
  heading: 84,
  /** A plain figure with no icon. */
  figure: 72,
  /** A choice column whose heading is two short words ("ادھاری / نقدی"). */
  choiceHeading: 108,
} as const;

/**
 * The least width of a column in a grid that must fit its room (`fit`): the screen has to work at
 * 1024 px with the main menu and the side menu open, and never scroll sideways (NFR-USE-15). The
 * widths above are then the shares of the free width; these are the least a column keeps.
 */
export const FIT_MIN_WIDTHS: Record<EntryColumnType, number> = {
  account: 150,
  money: 72,
  qty: 64,
  weight: 72,
  rate: 80,
  date: 104,
  text: 120,
  // A calculated cell holds an amount of up to nine digits and the ⓘ (a 32 px target).
  calc: 108,
  label: 120,
  picker: 84,
  // One short word (ادھاری, Credit) with its padding; the heading may wrap to a second line.
  choice: 84,
};

/** The types whose cells are numbers, codes or dates: Latin digits, left to right, tabular. */
const NUMBER_LIKE: ReadonlySet<EntryColumnType> = new Set([
  'money',
  'qty',
  'weight',
  'rate',
  'date',
  'calc',
]);

export function isNumberLike(type: EntryColumnType): boolean {
  return NUMBER_LIKE.has(type);
}

/**
 * The columns of a read-only list (ListGrid). Same rule as above: one type, one width (F18). A
 * `name` column is wide enough for a Nastaliq name and the other script's name beside it.
 */
export type ListColumnType =
  | 'code'
  | 'name'
  | 'text'
  | 'choice'
  | 'number'
  | 'money'
  | 'date'
  // A day and a time, DD-MM-YYYY HH:MM (the list of بکری, the audit log).
  | 'datetime'
  | 'status'
  // A whole number such as a quantity, a count of days or of trucks (reports, dense tables).
  | 'count'
  // An amount in a dense table, such as a report with many money columns (up to 9 digits).
  | 'figure'
  // A short text such as a phone number (11 digits), an item or a city in a dense table.
  | 'short'
  // A balance: a figure and the side word in a slot after it (reports, `unit` of ListColumn).
  | 'balance';

export const LIST_COLUMN_WIDTHS: Record<ListColumnType, number> = {
  code: 120,
  name: 260,
  text: 200,
  // A phrase from a short fixed list, such as "sold by weight" (Urdu needs about 135 px for it).
  choice: 160,
  number: 140,
  money: 140,
  date: 140,
  datetime: 170,
  status: 120,
  count: 100,
  figure: 120,
  short: 140,
  // The figure keeps the width of a money column and the side word has its own slot after it.
  balance: 180,
};

/**
 * The least width of a column that takes the free width of the grid (`grow`). Its own type width is
 * where it starts; when the grid is narrower, a growing column gives up space down to this, so a
 * list never scrolls sideways (NFR-USE-15). Enough for a name in either script.
 */
export const LIST_GROW_MIN_WIDTH = 160;

/** Codes and numbers are Latin digits, written left to right and set against the right edge. */
export function isListNumberLike(type: ListColumnType): boolean {
  return (
    type === 'code' ||
    type === 'number' ||
    type === 'count' ||
    type === 'figure' ||
    type === 'money' ||
    type === 'date' ||
    type === 'datetime' ||
    type === 'balance'
  );
}
