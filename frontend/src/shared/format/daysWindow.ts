// The window of days of a list that shows many rows a day (PRD FR-19.11 v1.57, plan 95 items 14 and 18): the sales
// registers, the day book, the cash book, the audit log and the وصولی, ادائیگی and پیشگی lists show 30 days at a time.
// پچھلی بکری keeps its own 7-day window (`features/bikri/past-window.ts`, FR-12.46). The window is that many days
// ending on a chosen day (today at first), it moves by its own length, and it never passes today nor goes before the
// first day that has data. Pure dates, no money. Days are `YYYY-MM-DD` as the backend sends them.

import { addDays } from './dayMath';

/** The days in the window of every list of days (PRD FR-19.11, v1.57). */
export const DAYS_WINDOW = 30;

/**
 * The day the window ends on: the day asked for, never after today and never so early that the window
 * would end before the first day of data. Nothing asked for gives today. `first` is the first day with data
 * (null while it is not known). Example: today 14-11-2026; asked 20-11-2026 gives 14-11-2026; asked 01-01-2020
 * with first 10-10-2026 gives 16-10-2026 (the first window that holds the first day).
 */
export function daysWindowEnd(
  asked: string | null,
  today: string,
  first: string | null,
  size: number = DAYS_WINDOW,
): string {
  let end = asked === null || asked > today ? today : asked;
  if (first !== null) {
    const earliest = addDays(first, size - 1);
    if (end < earliest) end = earliest > today ? today : earliest;
  }
  return end;
}

/** The first and the last day of the window of `size` days that ends on `end`. */
export function daysWindowOf(
  end: string,
  size: number = DAYS_WINDOW,
): { from: string; to: string } {
  return { from: addDays(end, -(size - 1)), to: end };
}

/** The end of the window after a step of its own length back (-1) or nearer (+1), inside the limits. */
export function stepDaysWindow(
  end: string,
  direction: -1 | 1,
  today: string,
  first: string | null,
  size: number = DAYS_WINDOW,
): string {
  return daysWindowEnd(addDays(end, direction * size), today, first, size);
}

/** True when the window can come nearer: it does not end on today yet. */
export function canDaysWindowComeNearer(end: string, today: string): boolean {
  return end < today;
}

/** True when the window can go further back: it still holds days before the first day of data. */
export function canDaysWindowGoBack(
  end: string,
  first: string | null,
  size: number = DAYS_WINDOW,
): boolean {
  return first === null || daysWindowOf(end, size).from > first;
}
