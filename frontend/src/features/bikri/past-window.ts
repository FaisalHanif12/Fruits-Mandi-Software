// The window of earlier days of پچھلی بکری (PRD 12.7, plan 72 P4 to P6). Pure date helpers, no money:
// the window is 7 days that end on a chosen day (yesterday at first), it moves by 7 days, and it never
// passes yesterday because today has its own section. Days are `YYYY-MM-DD` as the backend sends them.
// The address of the screen keeps the window and the row, so the way back from an edit lands on the
// same place (P10).

import { addDays, dayOfText, isIsoDay } from '../../shared/format';

// The plain-day helpers live in `shared/format` (a party's statement moves a window of days too, FR-12.47);
// they are passed on from here so this screen's tests and imports keep one place.
export { addDays, dayOfText, isIsoDay };

/** The days in the window (plan 72 P5). */
export const WINDOW_DAYS = 7;

/** Yesterday, the last day the window may end on. */
export function lastWindowEnd(today: string): string {
  return addDays(today, -1);
}

/**
 * The day the window ends on: the day asked for, never later than yesterday. Nothing asked for
 * gives yesterday. Example: today 12-11-2026; asked 05-11-2026 gives 05-11-2026; asked 12-11-2026
 * gives 11-11-2026.
 */
export function windowEnd(asked: string | null, today: string): string {
  const last = lastWindowEnd(today);
  return asked === null || asked > last ? last : asked;
}

/** The first and the last day of the window that ends on `end`. */
export function windowOf(end: string): { from: string; to: string } {
  return { from: addDays(end, -(WINDOW_DAYS - 1)), to: end };
}

/** The end of the window after a step of 7 days back (-1) or nearer (+1), never past yesterday. */
export function stepWindowEnd(end: string, direction: -1 | 1, today: string): string {
  return windowEnd(addDays(end, direction * WINDOW_DAYS), today);
}

/** True when the window can come nearer: it does not end on yesterday yet. */
export function canComeNearer(end: string, today: string): boolean {
  return end < lastWindowEnd(today);
}

/**
 * The days that get a heading on one page of the window. `days` are all the days of the window,
 * newest first; `rowDays` are the days of the rows on the page, in order. A page shows the headings
 * from its first row's day to its last row's day, so a day that goes on from the page before shows
 * its heading again, and a day with no بکری shows between the days around it. The first page also
 * shows the days before its rows, and the last page the days after them.
 */
export function headingDaysOnPage(
  days: readonly string[],
  rowDays: readonly string[],
  firstPage: boolean,
  lastPage: boolean,
): string[] {
  if (rowDays.length === 0) return firstPage ? [...days] : [];
  const first = days.indexOf(rowDays[0] ?? '');
  const last = days.indexOf(rowDays[rowDays.length - 1] ?? '');
  if (first < 0 || last < 0) return [...days];
  const from = firstPage ? 0 : first;
  const to = lastPage ? days.length - 1 : last;
  return days.slice(from, to + 1);
}

/** What the address of the screen keeps: the window, the row to come back to, and the tick. */
export interface PastSearch {
  /** The last day of the window. Nothing: it ends yesterday. */
  end?: string;
  /** The بکری to focus when the list opens (the one the Owner came back from). */
  row?: number;
  /** "Show every بکری" was on. */
  all?: true;
}

function positiveId(value: unknown): number | undefined {
  const id = Number(value);
  return Number.isSafeInteger(id) && id > 0 ? id : undefined;
}

/** The address of the screen, checked: a day that is not real or a row that is not a number is dropped. */
export function parsePastSearch(search: Record<string, unknown>): PastSearch {
  const end = search['end'];
  const row = positiveId(search['row']);
  return {
    ...(isIsoDay(end) ? { end } : {}),
    ...(row === undefined ? {} : { row }),
    ...(search['all'] === true || search['all'] === 'true' ? { all: true as const } : {}),
  };
}

/** Where the sale book sends the Owner back to: the same window and row (plan 72 P10). */
export interface PastReturn {
  /** The last day of the window he came from. */
  past: string;
  /** "Show every بکری" was on. */
  pastAll?: true;
}

/** What the address of the sale book keeps of the way back, checked. */
export function parsePastReturn(search: Record<string, unknown>): PastReturn | undefined {
  const past = search['past'];
  if (!isIsoDay(past)) return undefined;
  return {
    past,
    ...(search['pastAll'] === true || search['pastAll'] === 'true'
      ? { pastAll: true as const }
      : {}),
  };
}

/** The address of the list for a way back, with the row to focus. */
export function pastAddress(back: PastReturn, row: number): PastSearch {
  return { end: back.past, row, ...(back.pastAll === true ? { all: true as const } : {}) };
}
