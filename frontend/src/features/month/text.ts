// The words around a month of the month close screen (PRD FR-17.6). A month arrives as `YYYY-MM`;
// its name comes from Fluent with the year ("October 2026", "اکتوبر 2026"), the way the reports
// write a month. Nothing here works out money.

import { monthText } from '../reports';

/** The year and the month number (1 to 12) of a `YYYY-MM` text, or null when it is not one. */
export function parseMonth(month: string): { year: number; month: number } | null {
  const match = /^(\d{4})-(0[1-9]|1[0-2])$/.exec(month);
  if (match === null) return null;
  return { year: Number(match[1]), month: Number(match[2]) };
}

/** The name of a month with its year; the text itself when it is not a `YYYY-MM`. */
export function monthName(text: (id: string) => string, month: string): string {
  const parsed = parseMonth(month);
  return parsed === null ? month : monthText(text, parsed.year, parsed.month);
}

/** The Fluent message of a month's state, `month-state-days-open` for `days_open`. */
export function stateMessageId(state: string): string {
  return `month-state-${state.replaceAll('_', '-')}`;
}

/** The Fluent message of what a line of the close is, `month-kind-partner-capital`. */
export function kindMessageId(kind: string): string {
  return `month-kind-${kind.replaceAll('_', '-')}`;
}

/**
 * The month screen's own words for the refusals whose general words belong to the day screen
 * (`err-day-not-closed` talks about reopening a day). The others use the general words.
 */
const MONTH_ERROR_IDS: Readonly<Record<string, string>> = {
  ERR_DATE_IN_FUTURE: 'month-error-not-finished',
  ERR_DAY_NOT_CLOSED: 'month-error-days-open',
  ERR_MONTH_CLOSED: 'month-error-already-closed',
  ERR_NOTHING_TO_CLOSE: 'month-error-nothing-to-close',
  ERR_NOT_FOUND: 'month-error-not-in-force',
};

/** The message that says a refusal in the month screen's words, or null for the general words. */
export function monthErrorId(code: string): string | null {
  return MONTH_ERROR_IDS[code] ?? null;
}
