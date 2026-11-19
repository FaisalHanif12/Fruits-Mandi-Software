// Date text helpers (PRD FR-7.19, Tech Stack 4.5). Business dates are typed and shown as
// DD-MM-YYYY. The backend reads the typed text and is the final judge (FR-6.6). This file only
// moves a date by one day for the + and - keys, and expands the typing shortcuts first so the
// key moves from the date the user means. No money is touched here.

const DAY_MS = 86_400_000;
const FULL = /^(\d{1,2})-(\d{1,2})-(\d{4})$/;
const DAY_MONTH = /^(\d{1,2})-(\d{1,2})$/;
const DAY_ONLY = /^(\d{1,2})$/;

interface Parts {
  day: number;
  month: number;
  year: number;
}

function pad(value: number, width: number): string {
  return String(value).padStart(width, '0');
}

/** A real calendar date as parts, or null (31-02-2026 is null). */
function realDate(day: number, month: number, year: number): Parts | null {
  const moment = new Date(Date.UTC(year, month - 1, day));
  const real =
    moment.getUTCFullYear() === year &&
    moment.getUTCMonth() === month - 1 &&
    moment.getUTCDate() === day;
  return real ? { day, month, year } : null;
}

/**
 * The date the text means: DD-MM-YYYY, "10-9" (this year) or "10" (this month and year).
 * Null when the text is empty, partial or not a real date.
 */
export function readDateText(text: string, today: Date): Parts | null {
  const trimmed = text.trim();
  const full = FULL.exec(trimmed);
  if (full) return realDate(Number(full[1]), Number(full[2]), Number(full[3]));
  const dayMonth = DAY_MONTH.exec(trimmed);
  if (dayMonth) return realDate(Number(dayMonth[1]), Number(dayMonth[2]), today.getFullYear());
  const dayOnly = DAY_ONLY.exec(trimmed);
  if (dayOnly) {
    return realDate(Number(dayOnly[1]), today.getMonth() + 1, today.getFullYear());
  }
  return null;
}

/** Parts as DD-MM-YYYY. */
export function formatDateParts({ day, month, year }: Parts): string {
  return `${pad(day, 2)}-${pad(month, 2)}-${pad(year, 4)}`;
}

/**
 * The text one day later (+1) or earlier (-1) than the date in `text`. Empty or unreadable text
 * starts from `today`, the computer's local date.
 */
export function shiftDateText(text: string, days: 1 | -1, today: Date): string {
  const start =
    readDateText(text, today) ??
    ({
      day: today.getDate(),
      month: today.getMonth() + 1,
      year: today.getFullYear(),
    } satisfies Parts);
  const moved = new Date(Date.UTC(start.year, start.month - 1, start.day) + days * DAY_MS);
  return formatDateParts({
    day: moved.getUTCDate(),
    month: moved.getUTCMonth() + 1,
    year: moved.getUTCFullYear(),
  });
}

const ISO = /^(\d{4})-(\d{2})-(\d{2})$/;

/** A date from the backend, YYYY-MM-DD, shown as DD-MM-YYYY (Tech Stack 4.5). Anything else is shown as it is. */
export function isoToDisplayDate(iso: string): string {
  const match = ISO.exec(iso);
  return match ? `${match[3] ?? ''}-${match[2] ?? ''}-${match[1] ?? ''}` : iso;
}

/**
 * Text that is exactly a real calendar day, YYYY-MM-DD, shown as DD-MM-YYYY. Anything else (other
 * text, 2026-02-30, 2026-10, a reference such as E-20261010-0001) comes back unchanged. For the
 * values of a backend error, which can be any text.
 */
export function isoDayToDisplay(text: string): string {
  const match = ISO.exec(text);
  if (!match) return text;
  const day = realDate(Number(match[3]), Number(match[2]), Number(match[1]));
  return day === null ? text : formatDateParts(day);
}

function two(value: number): string {
  return String(value).padStart(2, '0');
}

/**
 * A time from the backend (UTC, ISO text) shown as DD-MM-YYYY HH:MM in the computer's own time,
 * which is Pakistan time on the client's machine (Code Standards G9). Anything unreadable is
 * shown as it is.
 */
export function utcToLocalDisplay(iso: string): string {
  const moment = new Date(iso);
  if (Number.isNaN(moment.getTime())) return iso;
  return `${two(moment.getDate())}-${two(moment.getMonth() + 1)}-${String(moment.getFullYear())} ${two(moment.getHours())}:${two(moment.getMinutes())}`;
}

/**
 * The time of day (HH:MM) of a time from the backend (UTC, ISO text), in the computer's own time,
 * which is Pakistan time on the client's machine (Code Standards G9). Unreadable text is shown as
 * it is.
 */
export function utcToLocalTime(iso: string): string {
  const moment = new Date(iso);
  if (Number.isNaN(moment.getTime())) return iso;
  return `${two(moment.getHours())}:${two(moment.getMinutes())}`;
}

/** The computer's local date as DD-MM-YYYY, for a new record's date (PRD FR-6.6). */
export function todayText(today: Date): string {
  return formatDateParts({
    day: today.getDate(),
    month: today.getMonth() + 1,
    year: today.getFullYear(),
  });
}
