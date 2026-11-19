// The periods a list can be opened on (plan 38 R4, PRD FR-19.2): today, this week, this month, last
// month, this year, last year, from the first date, or two dates the user types. Dates are worked out
// here as calendar days only (no money, no time zone); the backend checks them again. A "this" period
// runs from its first day to today, because a day after today has no entries. Pure.

import { formatDateParts } from './dates';

export type PeriodKind =
  'today' | 'week' | 'month' | 'last_month' | 'year' | 'last_year' | 'all' | 'custom';

/** In the order the period box offers them. */
export const PERIOD_KINDS: readonly PeriodKind[] = [
  'today',
  'week',
  'month',
  'last_month',
  'year',
  'last_year',
  'all',
  'custom',
];

/** A period as DD-MM-YYYY. An empty end is open: `all` has no first and no last day. */
export interface PeriodRange {
  from: string;
  to: string;
}

const ISO = /^(\d{4})-(\d{2})-(\d{2})$/;

/** The day of an ISO date (YYYY-MM-DD) as a UTC date, or null when it is not one. */
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

/** The first of a month, `back` months before the month of `date`. */
function monthStart(date: Date, back: number): Date {
  return new Date(Date.UTC(date.getUTCFullYear(), date.getUTCMonth() - back, 1));
}

/**
 * The days a period covers, for `todayIso` (YYYY-MM-DD, the day the app is on). `custom` has no days
 * of its own: the caller keeps the two dates. Example for 10-10-2026: `month` is 01-10-2026 to
 * 10-10-2026; `last_month` is 01-09-2026 to 30-09-2026; `week` starts on the Monday, 05-10-2026.
 */
export function periodRange(kind: PeriodKind, todayIso: string): PeriodRange {
  const today = parse(todayIso);
  if (today === null || kind === 'all' || kind === 'custom') return { from: '', to: '' };
  switch (kind) {
    case 'today':
      return { from: text(today), to: text(today) };
    case 'week': {
      // Monday is the first day of the week (the backend counts weekdays from Monday).
      const sinceMonday = (today.getUTCDay() + 6) % 7;
      const monday = new Date(today.getTime() - sinceMonday * 86_400_000);
      return { from: text(monday), to: text(today) };
    }
    case 'month':
      return { from: text(monthStart(today, 0)), to: text(today) };
    case 'last_month': {
      const first = monthStart(today, 1);
      const last = new Date(monthStart(today, 0).getTime() - 86_400_000);
      return { from: text(first), to: text(last) };
    }
    case 'year':
      return {
        from: text(new Date(Date.UTC(today.getUTCFullYear(), 0, 1))),
        to: text(today),
      };
    case 'last_year':
      return {
        from: text(new Date(Date.UTC(today.getUTCFullYear() - 1, 0, 1))),
        to: text(new Date(Date.UTC(today.getUTCFullYear() - 1, 11, 31))),
      };
  }
}

/** The Fluent message of a period kind: `common-period-last-month` for `last_month`. */
export function periodMessageId(kind: PeriodKind): string {
  return `common-period-${kind.replaceAll('_', '-')}`;
}
