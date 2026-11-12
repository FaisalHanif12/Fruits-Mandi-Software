// The periods the statement print box offers (plan 74 T3, PRD FR-12.47): the last 10 days, the last 30 days
// (the default), the last 6 months, or two dates the user types. Dates are worked out here as calendar days
// only (no money, no time zone); the backend reads them as typed (DD-MM-YYYY) and checks them again. The
// last N days end today and include it: on 12-11-2026 "the last 10 days" is 03-11-2026 to 12-11-2026. The last
// N months start on the same day of the month N months back, and a month end moves to the last day of a
// shorter month: on 31-08-2026 "the last 6 months" is 28-02-2026 to 31-08-2026. Pure.

import { formatDateParts } from '../../../shared/format';

export type StatementPeriod = 'last_10' | 'last_30' | 'last_6_months' | 'custom';

/** In the order the box offers them. */
export const STATEMENT_PERIODS: readonly StatementPeriod[] = [
  'last_10',
  'last_30',
  'last_6_months',
  'custom',
];

export const DEFAULT_STATEMENT_PERIOD: StatementPeriod = 'last_30';

/** The Fluent message of a period. */
export const STATEMENT_PERIOD_MESSAGE: Record<StatementPeriod, string> = {
  last_10: 'statement-print-last-10',
  last_30: 'statement-print-last-30',
  last_6_months: 'statement-print-last-6-months',
  custom: 'statement-print-custom',
};

/** A period as DD-MM-YYYY text, the way the backend reads a typed date. */
export interface StatementRange {
  from: string;
  to: string;
}

const DAY_MS = 86_400_000;
const ISO = /^(\d{4})-(\d{2})-(\d{2})$/;

function parse(iso: string): Date | null {
  const found = ISO.exec(iso);
  if (found === null) return null;
  const date = new Date(Date.UTC(Number(found[1]), Number(found[2]) - 1, Number(found[3])));
  return Number.isNaN(date.getTime()) ? null : date;
}

function text(date: Date): string {
  return formatDateParts({
    day: date.getUTCDate(),
    month: date.getUTCMonth() + 1,
    year: date.getUTCFullYear(),
  });
}

/** The same day of the month `back` months earlier, or the last day of that month when it is shorter. */
function monthsBack(date: Date, back: number): Date {
  const year = date.getUTCFullYear();
  const month = date.getUTCMonth() - back;
  const lastDay = new Date(Date.UTC(year, month + 1, 0)).getUTCDate();
  return new Date(Date.UTC(year, month, Math.min(date.getUTCDate(), lastDay)));
}

/**
 * The days a period covers, for `todayIso` (YYYY-MM-DD, the day the app is on). `custom` has no days of its
 * own: the caller keeps the two dates the user typed.
 */
export function statementRange(period: StatementPeriod, todayIso: string): StatementRange | null {
  const today = parse(todayIso);
  if (today === null || period === 'custom') return null;
  switch (period) {
    case 'last_10':
      return { from: text(new Date(today.getTime() - 9 * DAY_MS)), to: text(today) };
    case 'last_30':
      return { from: text(new Date(today.getTime() - 29 * DAY_MS)), to: text(today) };
    case 'last_6_months':
      return { from: text(monthsBack(today, 6)), to: text(today) };
  }
}
