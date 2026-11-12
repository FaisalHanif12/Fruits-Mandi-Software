// A small stateful world for the fuzz of پچھلی بکری (plan 72, PRD 12.7, FR-12.46). Owner: `robustness`.
//
// The fake `bikri_list` of the other tests answers from a fixed list. This one answers from a world
// that a trial changes (a cancel), with the rules the real list has: a cancelled بکری is hidden until
// "show every بکری" is on, a typed text finds a number or the start of a truck, rows stand newest day
// first and highest number first, and a window has one entry for each day of its range, empty days
// included, with the count and the خام بکری of that day. It is a double: it adds money only to give the
// day figures the real backend gives, in whole rupees, because the screen never adds (Code Standards F3).
// Its day arithmetic is written here again, with plain calendar maths, so a wrong date helper in the
// screen cannot hide behind the same helper in the double.

import type { BikriDay } from '../../src/bindings/BikriDay';
import type { BikriList } from '../../src/bindings/BikriList';
import type { BikriListInput } from '../../src/bindings/BikriListInput';
import type { BikriListRow } from '../../src/bindings/BikriListRow';
import { money } from '../helpers/accounts';
import { listRow } from '../helpers/bikri-saved';
import { bikriListOf } from '../helpers/paged';
import { paisaForTest } from '../helpers/paisa';

export const TODAY = '2026-10-10';
export const YESTERDAY = '2026-10-09';
/** Most days a window may hold, as the backend allows. */
const MOST_DAYS = 366;

/** A small seeded generator (the same one the other fuzz files use). */
export function rng(seed: number) {
  let state = seed * 2_654_435_761 + 12_345;
  const next = () => {
    state = (state * 1_664_525 + 1_013_904_223) % 4_294_967_296;
    return state / 4_294_967_296;
  };
  return {
    below: (n: number) => Math.floor(next() * n),
    pick: <T>(items: readonly T[]): T => {
      const found = items[Math.floor(next() * items.length)];
      if (found === undefined) throw new Error('nothing to pick from');
      return found;
    },
  };
}
export type Random = ReturnType<typeof rng>;

/** The day `delta` days from `iso`, by calendar maths that also works for the years before 100. */
export function dayAfter(iso: string, delta: number): string {
  const [year = 0, month = 1, day = 1] = iso.split('-').map(Number);
  const moment = new Date(0);
  moment.setUTCFullYear(year, month - 1, day + delta);
  const text = (n: number, width: number) => String(n).padStart(width, '0');
  return `${text(moment.getUTCFullYear(), 4)}-${text(moment.getUTCMonth() + 1, 2)}-${text(moment.getUTCDate(), 2)}`;
}

/** `DD-MM-YYYY` as `YYYY-MM-DD`. */
export function isoOf(display: string | null): string {
  const [day = '', month = '', year = ''] = (display ?? '').split('-');
  return `${year}-${month}-${day}`;
}

/** The first day of the window that ends on `end`. */
export const windowFrom = (end: string) => dayAfter(end, -6);

/** The days of a window, newest first. */
export function daysOfWindow(end: string): string[] {
  return Array.from({ length: 7 }, (_, index) => dayAfter(end, -index));
}

export interface World {
  /** Every بکری of every day, today's included. */
  rows: BikriListRow[];
  /** The whole rupees of each بکری's خام بکری, so a day's figure is the sum the backend would give. */
  rupees: Map<number, number>;
}

/** What a list call shows of a row: cancelled ones step aside until the tick is on. */
export function shownBy(input: Pick<BikriListInput, 'all' | 'show_finished' | 'search'>) {
  const typed = input.all ? '' : (input.search ?? '').trim();
  return (row: BikriListRow): boolean => {
    if (!(input.all || input.show_finished || row.status !== 'cancelled')) return false;
    if (typed === '') return true;
    return (
      String(row.number) === typed ||
      (row.truck_no ?? '').toLowerCase().startsWith(typed.toLowerCase())
    );
  };
}

const newestFirst = (a: BikriListRow, b: BikriListRow) =>
  a.entry_date < b.entry_date ? 1 : a.entry_date > b.entry_date ? -1 : b.number - a.number;

/** The rows a call with these filters shows, for one day or a range, in the order the backend gives. */
export function rowsFor(
  world: World,
  filter: Pick<BikriListInput, 'all' | 'show_finished' | 'search'>,
  from: string,
  to: string,
): BikriListRow[] {
  const keep = shownBy(filter);
  return world.rows
    .filter((row) => row.entry_date >= from && row.entry_date <= to && keep(row))
    .sort(newestFirst);
}

const sumOf = (world: World, rows: readonly BikriListRow[]) =>
  rows.reduce((total, row) => total + (world.rupees.get(row.id) ?? 0), 0);

/** The days of a call: one for each day from `from` to `to`, newest first, with count and gross. */
export function daysFor(world: World, rows: readonly BikriListRow[], from: string, to: string) {
  const days: BikriDay[] = [];
  let date = to;
  for (let step = 0; step <= MOST_DAYS && date >= from; step += 1) {
    const ofDay = rows.filter((row) => row.entry_date === date);
    days.push({
      date,
      count: ofDay.length,
      gross_paisa: paisaForTest(sumOf(world, ofDay) * 100),
    });
    date = dayAfter(date, -1);
  }
  return days;
}

/** What `bikri_list` answers. A call with no day at all, or one after today, gets nothing (it is a fault). */
export function answerList(world: World, input: BikriListInput): BikriList {
  const single = input.date !== null;
  const from = single ? isoOf(input.date) : isoOf(input.from);
  const to = single ? isoOf(input.date) : isoOf(input.to);
  const rows = rowsFor(world, input, from, to);
  const totals = { gross: paisaForTest(sumOf(world, rows) * 100) };
  const days = input.group_by_day && !single ? daysFor(world, rows, from, to) : [];
  return bikriListOf(rows, input.offset, input.limit, totals, days);
}

/** The rupees of a day, formatted as the screen shows a whole-rupee amount. */
export const rupeesText = (rupees: number) => rupees.toLocaleString('en-US');

function addRow(world: World, id: number, number: number, day: string, extra = {}): void {
  const rupees = (1 + (id % 90)) * 1_000;
  world.rupees.set(id, rupees);
  world.rows.push(
    listRow({
      id,
      number,
      entry_date: day,
      created_at: `${day}T07:10:00Z`,
      truck_no: null,
      gross: money(rupees),
      ...extra,
    }),
  );
}

/**
 * A world for one trial. Row 41 is always there (بکری 1 of yesterday), because the edit and cancel
 * doubles of the other tests are built on it. A normal world has at most 23 بکری, so no window needs a
 * second page; a heavy one has 30 on one day, so the page buttons are there too.
 */
export function buildWorld(random: Random, heavy: boolean): World {
  const world: World = { rows: [], rupees: new Map() };
  addRow(world, 41, 1, YESTERDAY);
  const count = heavy ? 6 : 6 + random.below(17);
  for (let index = 0; index < count; index += 1) {
    const back = random.below(10) < 6 ? random.below(14) : random.below(40);
    const day = dayAfter(YESTERDAY, -back);
    addRow(world, 100 + index, 2 + index, day, {
      status: random.below(7) === 0 ? 'cancelled' : 'saved',
      truck_no: random.below(8) === 0 ? `LES-${String(1000 + index)}` : null,
    });
  }
  if (heavy) {
    const busy = dayAfter(YESTERDAY, -1);
    for (let index = 0; index < 30; index += 1) {
      addRow(world, 500 + index, 200 + index, busy);
    }
  }
  const todays = random.below(4);
  for (let index = 0; index < todays; index += 1) {
    addRow(world, 300 + index, 900 + index, TODAY);
  }
  return world;
}
