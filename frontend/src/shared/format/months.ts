// Plain-month helpers for screens that show one month at a time (the commission page, a partner's
// statement, the month lists of expenses, capital, fruit, ادھار رقم and الگ کھاتے; PRD FR-9.3, FR-17.12,
// FR-19.11, plan 76). A month is `YYYY-MM` as the backend sends it. Pure dates, no money: a month is
// never worked out from a time zone, so a year end moves right in every zone.

import { addDays, isIsoDay } from './dayMath';

const ISO_MONTH = /^(\d{4})-(\d{2})$/;
const ISO_DAY_START = /^(\d{4})-(\d{2})-\d{2}$/;
const TYPED_MONTH = /^(\d{1,2})(?:[-/.](\d{2}|\d{4}))?$/;
const TYPED_DAY = /^(\d{1,2})[-/.](\d{1,2})[-/.](\d{2}|\d{4})$/;

function pad(value: number, width = 2): string {
  return String(value).padStart(width, '0');
}

/** True when the text is a real month, `YYYY-MM`. */
export function isIsoMonth(text: unknown): text is string {
  if (typeof text !== 'string') return false;
  const match = ISO_MONTH.exec(text);
  if (match === null) return false;
  const month = Number(match[2]);
  return month >= 1 && month <= 12;
}

/** The month of a day, `YYYY-MM-DD` to `YYYY-MM`. Example: 2026-11-12 gives 2026-11. */
export function monthOf(iso: string): string {
  const match = ISO_DAY_START.exec(iso);
  return match === null ? iso : `${match[1] ?? ''}-${match[2] ?? ''}`;
}

/** The year and month number of a month, or null when the text is not a month. */
export function monthParts(month: string): { year: number; month: number } | null {
  const match = ISO_MONTH.exec(month);
  if (match === null || !isIsoMonth(month)) return null;
  return { year: Number(match[1]), month: Number(match[2]) };
}

/**
 * The month `months` after `month` (before, when negative). Example: 2026-11, +2 gives 2027-01;
 * 2026-01, -1 gives 2025-12.
 */
export function addMonths(month: string, months: number): string {
  const parts = monthParts(month);
  if (parts === null) return month;
  const index = parts.year * 12 + (parts.month - 1) + months;
  return `${pad(Math.floor(index / 12), 4)}-${pad((index % 12) + 1)}`;
}

/**
 * The first and the last day of a month, `YYYY-MM-DD`. Example: 2026-02 gives 2026-02-01 and
 * 2026-02-28; a leap year gives the 29th.
 */
export function monthBounds(month: string): { from: string; to: string } {
  const next = addMonths(month, 1);
  const from = `${month}-01`;
  return { from, to: addDays(`${next}-01`, -1) };
}

/** The month as the Owner reads it, `MM-YYYY`. Example: 2026-10 gives 10-2026. */
export function displayMonth(month: string): string {
  const parts = monthParts(month);
  return parts === null ? month : `${pad(parts.month)}-${pad(parts.year, 4)}`;
}

/**
 * The month a screen opens on or is moved to: the one asked for, never after the month of today and
 * never before the first month of data. Nothing asked for gives the month of today. `first` is the
 * month of the first entry (null while it is not known, or when there is no data yet).
 * Example: today 2026-11-12; asked 2027-03 gives 2026-11; asked 2020-01 with first 2026-02 gives 2026-02.
 */
export function windowMonth(asked: string | null, today: string, first: string | null): string {
  const last = monthOf(today);
  let month = asked !== null && isIsoMonth(asked) ? asked : last;
  if (month > last) month = last;
  if (first !== null && isIsoMonth(first) && month < first) month = first;
  return month;
}

/** The month after a step back (-1) or nearer (+1), kept inside the limits of `windowMonth`. */
export function stepMonth(
  month: string,
  direction: -1 | 1,
  today: string,
  first: string | null,
): string {
  return windowMonth(addMonths(month, direction), today, first);
}

/** True when the window can come nearer: it is not the month of today yet. */
export function canMonthComeNearer(month: string, today: string): boolean {
  return month < monthOf(today);
}

/** True when the window can go further back: it is not the first month of data yet. */
export function canMonthGoBack(month: string, first: string | null): boolean {
  return first === null || !isIsoMonth(first) || month > first;
}

/**
 * The month a typed text means, `YYYY-MM`, or null when it is empty or not a month. Read: `10` (that
 * month of the year of `today`), `10-2026`, `10-26`, and a whole date `12-10-2026` (its month). A
 * year of two digits is 20xx. Example: today 2026-11-12; "9" gives 2026-09; "9-25" gives 2025-09.
 */
export function monthOfText(text: string, today: string): string | null {
  const typed = text.trim();
  if (typed === '') return null;
  const todayParts = monthParts(monthOf(today));
  const fullDay = TYPED_DAY.exec(typed);
  if (fullDay !== null) {
    const raw = fullDay[3] ?? '';
    const year = raw.length === 2 ? 2000 + Number(raw) : Number(raw);
    const iso = `${pad(year, 4)}-${pad(Number(fullDay[2]))}-${pad(Number(fullDay[1]))}`;
    return isIsoDay(iso) ? monthOf(iso) : null;
  }
  const match = TYPED_MONTH.exec(typed);
  if (match === null || todayParts === null) return null;
  const month = Number(match[1]);
  if (month < 1 || month > 12) return null;
  const rawYear = match[2];
  const year =
    rawYear === undefined
      ? todayParts.year
      : rawYear.length === 2
        ? 2000 + Number(rawYear)
        : Number(rawYear);
  return `${pad(year, 4)}-${pad(month)}`;
}
