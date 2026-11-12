// A fake `bikri_list` for پچھلی بکری (plan 72): one command, two kinds of call. A call with `date` is
// today's section; a call with `from` and `to` is the window of earlier days, which answers its rows
// and one entry for each day of the range, empty days included, with the count and the خام بکری of
// each day. A test double only: the real counts and totals are the backend's (the `*-real-*` tests
// and the fixtures). A double adds no money (the frontend never does, F3): the خام بکری of a day is
// given by the test, never summed from the rows.

import type { BikriListRow } from '../../src/bindings/BikriListRow';
import type { BikriDay } from '../../src/bindings/BikriDay';
import type { BikriList } from '../../src/bindings/BikriList';
import type { BikriListInput } from '../../src/bindings/BikriListInput';
import { addDays, isIsoDay } from '../../src/features/bikri/past-window';
import { bikriListOf } from './paged';
import { paisaForTest } from './paisa';
import { listRow } from './bikri-saved';

/** `DD-MM-YYYY` as `YYYY-MM-DD`. */
export function isoOf(display: string): string {
  const [day = '', month = '', year = ''] = display.split('-');
  return `${year}-${month}-${day}`;
}

/** The most days a window may hold, as the backend allows (366 days apart). */
const MOST_DAYS = 366;

/**
 * Every day from `from` to `to` (both `YYYY-MM-DD`), newest first, with the given figures. Two real days
 * with `from` not after `to` are needed, as the backend needs them; the loop stops after 366 days, so a
 * wrong input can never hang a test or the development page (plan 72).
 */
export function daysOf(
  from: string,
  to: string,
  rows: readonly BikriListRow[],
  gross: Readonly<Record<string, number>> = {},
): BikriDay[] {
  if (!isIsoDay(from) || !isIsoDay(to) || from > to) {
    throw new Error(`bikri_list: the days need two real days, from not after to (${from}, ${to})`);
  }
  const days: BikriDay[] = [];
  let date = to;
  for (let step = 0; step <= MOST_DAYS && date >= from; step += 1) {
    days.push({
      date,
      count: rows.filter((row) => row.entry_date === date).length,
      // Whole rupees given by the test, never worked out from the rows.
      gross_paisa: paisaForTest((gross[date] ?? 0) * 100),
    });
    date = addDays(date, -1);
  }
  return days;
}

export interface PastFake {
  /** The rows of today's section. */
  today?: readonly BikriListRow[];
  /** The rows of every earlier day; a call gets those inside its range. */
  window?: readonly BikriListRow[];
  /** The خام بکری of a day in whole rupees, by `YYYY-MM-DD`. */
  gross?: Readonly<Record<string, number>>;
  /** The sums over the whole filter of each section, when a test or a photograph wants to see them. */
  todayTotals?: Partial<BikriList['totals']>;
  windowTotals?: Partial<BikriList['totals']>;
}

/** What `bikri_list` answers for a call. */
export function pastAnswer(
  input: BikriListInput,
  {
    today = [listRow()],
    window = [],
    gross = {},
    todayTotals = {},
    windowTotals = {},
  }: PastFake = {},
): BikriList {
  if (input.date !== null) return bikriListOf(today, input.offset, input.limit, todayTotals);
  const from = isoOf(input.from ?? '');
  const to = isoOf(input.to ?? '');
  const inside = window.filter((row) => row.entry_date >= from && row.entry_date <= to);
  // The backend sends the days only when it is asked for them (`group_by_day`).
  return bikriListOf(
    inside,
    input.offset,
    input.limit,
    windowTotals,
    input.group_by_day ? daysOf(from, to, inside, gross) : [],
  );
}
