// The 7-day window of a list that shows many rows a day (PRD FR-19.11, plan 76 point 8): the sales registers,
// the cash book, the voucher lists, the audit log. The window is 7 days that end on a chosen day (today at
// first), it moves by 7 days, and it never passes today nor goes before the first day that has data. Pure
// dates, no money. Days are `YYYY-MM-DD` as the backend sends them.

import { addDays } from './dayMath';

/** The days in the window. */
export const DAYS_WINDOW = 7;

/**
 * The day the window ends on: the day asked for, never after today and never so early that the window
 * would end before the first day of data. Nothing asked for gives today. `first` is the first day with data
 * (null while it is not known). Example: today 14-11-2026; asked 20-11-2026 gives 14-11-2026; asked 01-01-2020
 * with first 10-10-2026 gives 16-10-2026 (the first window that holds the first day).
 */
export function daysWindowEnd(asked: string | null, today: string, first: string | null): string {
  let end = asked === null || asked > today ? today : asked;
  if (first !== null) {
    const earliest = addDays(first, DAYS_WINDOW - 1);
    if (end < earliest) end = earliest > today ? today : earliest;
  }
  return end;
}

/** The first and the last day of the window that ends on `end`. */
export function daysWindowOf(end: string): { from: string; to: string } {
  return { from: addDays(end, -(DAYS_WINDOW - 1)), to: end };
}

/** The end of the window after a step of 7 days back (-1) or nearer (+1), inside the limits. */
export function stepDaysWindow(
  end: string,
  direction: -1 | 1,
  today: string,
  first: string | null,
): string {
  return daysWindowEnd(addDays(end, direction * DAYS_WINDOW), today, first);
}

/** True when the window can come nearer: it does not end on today yet. */
export function canDaysWindowComeNearer(end: string, today: string): boolean {
  return end < today;
}

/** True when the window can go further back: it still holds days before the first day of data. */
export function canDaysWindowGoBack(end: string, first: string | null): boolean {
  return first === null || daysWindowOf(end).from > first;
}
