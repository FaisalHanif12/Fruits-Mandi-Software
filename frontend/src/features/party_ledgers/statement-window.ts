// The window of 30 days of one party's statement (PRD FR-12.47, plan 74 W1 to W5). Pure date helpers,
// no money: the window is 30 days that end on a chosen day (today at first), it moves by 30 days, and
// the later button stops at today. Days are `YYYY-MM-DD` as the backend sends them. The address of the
// screen keeps the day the window ends on, so a language switch or a reload lands on the same window.

import { addDays } from '../../shared/format';

/** The days in the window (plan 74 W1). */
export const STATEMENT_DAYS = 30;

/**
 * The day the window ends on: the day asked for, never after today. Nothing asked for gives today.
 * Example: today 12-11-2026; asked 14-10-2026 gives 14-10-2026; asked 20-11-2026 gives 12-11-2026.
 */
export function statementEnd(asked: string | null, today: string): string {
  return asked === null || asked > today ? today : asked;
}

/** The first and the last day of the window that ends on `end`. Example: 12-11-2026 gives 14-10-2026 to 12-11-2026. */
export function statementWindowOf(end: string): { from: string; to: string } {
  return { from: addDays(end, -(STATEMENT_DAYS - 1)), to: end };
}

/** The end of the window after a step of 30 days back (-1) or nearer (+1), never after today. */
export function stepStatementEnd(end: string, direction: -1 | 1, today: string): string {
  return statementEnd(addDays(end, direction * STATEMENT_DAYS), today);
}

/** True when the window can come nearer: it does not end today yet. */
export function canComeNearer(end: string, today: string): boolean {
  return end < today;
}
