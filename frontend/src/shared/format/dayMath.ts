// Plain-day helpers for screens that move a window of days (پچھلی بکری: 7 days, a party's statement:
// 30 days; PRD 12.7, FR-12.47). Days are `YYYY-MM-DD` as the backend sends them. Pure dates, no money:
// a day is never worked out from a time zone, so a month end moves right in every zone.

import { readDateText } from './dates';

const DAY_MS = 86_400_000;
const ISO = /^(\d{4})-(\d{2})-(\d{2})$/;

function pad(value: number, width = 2): string {
  return String(value).padStart(width, '0');
}

/** True when the text is a real calendar day, `YYYY-MM-DD`. */
export function isIsoDay(text: unknown): text is string {
  if (typeof text !== 'string') return false;
  const match = ISO.exec(text);
  if (match === null) return false;
  const [year, month, day] = [Number(match[1]), Number(match[2]), Number(match[3])];
  const moment = new Date(Date.UTC(year, month - 1, day));
  return (
    moment.getUTCFullYear() === year &&
    moment.getUTCMonth() === month - 1 &&
    moment.getUTCDate() === day
  );
}

/** The day `days` after `iso` (before, when negative). Example: 2026-11-12, -7 gives 2026-11-05. */
export function addDays(iso: string, days: number): string {
  const match = ISO.exec(iso);
  if (match === null) return iso;
  const start = Date.UTC(Number(match[1]), Number(match[2]) - 1, Number(match[3]));
  const moved = new Date(start + days * DAY_MS);
  return `${pad(moved.getUTCFullYear(), 4)}-${pad(moved.getUTCMonth() + 1)}-${pad(moved.getUTCDate())}`;
}

/**
 * The day a typed date means, as `YYYY-MM-DD`, or null when the text is empty, partial or not a real
 * day. The shortcuts of PRD FR-7.19 are read ("10", "10-9"). `today` is the backend's today.
 */
export function dayOfText(text: string, today: string): string | null {
  const match = ISO.exec(today);
  const base =
    match === null
      ? new Date()
      : new Date(Number(match[1]), Number(match[2]) - 1, Number(match[3]));
  const parts = readDateText(text, base);
  if (parts === null) return null;
  return `${pad(parts.year, 4)}-${pad(parts.month)}-${pad(parts.day)}`;
}
