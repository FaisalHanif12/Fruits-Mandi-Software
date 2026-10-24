// What the address of the month close screen can carry: `#/month?month=2026-10` opens on that month.
// Anything that is not a `YYYY-MM` is left out.

import { parseMonth } from './text';

export interface MonthSearch {
  month?: string;
}

export function parseMonthSearch(search: Record<string, unknown>): MonthSearch {
  const month = search['month'];
  return typeof month === 'string' && parseMonth(month) !== null ? { month } : {};
}
