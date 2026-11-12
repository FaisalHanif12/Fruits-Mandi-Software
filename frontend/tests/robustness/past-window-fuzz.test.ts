// Plan 72 end gate, attack on the date logic of پچھلی بکری (PRD 12.7, FR-12.46, FR-7.19; Code Standards
// 4.2). Owner: `robustness`.
//
// The window of 7 earlier days, the step buttons, "تاریخ پر جائیں" and the address are pure helpers in
// `features/bikri/past-window.ts`. This file attacks them with seeded random days, month and year ends,
// leap days, and hostile text, and checks each answer against a reference built another way (plain
// calendar counting in days since 1970, no `Date` arithmetic on the day itself). Rules:
//
//   1. The window never ends after yesterday, and a day asked for that is not later than yesterday is
//      taken as it is. Nothing asked for gives yesterday.
//   2. The window is exactly 7 real days, oldest first to newest, that end on the end.
//   3. Seven steps back are 49 days back exactly; a step nearer never passes yesterday; `canComeNearer`
//      is true exactly when the end is before yesterday.
//   4. The text of the date box is a real day or null: never an impossible day, never a throw, whatever
//      the text is (Urdu digits, zero-width marks, control characters, SQL, 10,000 digits).
//   5. The address keeps only a real day, a positive whole row number, and the true flag.
//   6. The day headings of a page are a run of the window's days in order, they include every day a row
//      sits under, and they never invent a day.

import { describe, expect, it } from 'vitest';
import {
  WINDOW_DAYS,
  addDays,
  canComeNearer,
  dayOfText,
  headingDaysOnPage,
  isIsoDay,
  lastWindowEnd,
  parsePastReturn,
  parsePastSearch,
  stepWindowEnd,
  windowEnd,
  windowOf,
} from '../../src/features/bikri/past-window';

/** A small seeded generator (the same one the other fuzz files use). */
function rng(seed: number) {
  let state = seed * 2_654_435_761 + 12_345;
  const next = () => {
    state = (state * 1_664_525 + 1_013_904_223) % 4_294_967_296;
    return state / 4_294_967_296;
  };
  return {
    below: (n: number) => Math.floor(next() * n),
    pick: <T>(items: readonly T[]): T => items[Math.floor(next() * items.length)] as T,
  };
}

/** The item at an index, or a failure that names the index (no `!`, Code Standards F1). */
function at<T>(list: readonly T[], index: number): T {
  const found = list[index];
  if (found === undefined) throw new Error(`no item at ${String(index)}`);
  return found;
}

// ---- the reference: days counted from 1970-01-01 by the civil-calendar rule, no Date -------------

function isLeap(year: number): boolean {
  return (year % 4 === 0 && year % 100 !== 0) || year % 400 === 0;
}
function monthDays(year: number, month: number): number {
  return at([31, isLeap(year) ? 29 : 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31], month - 1);
}
/** Days since 1970-01-01 (negative before). Works for years 1 to 9999. */
function serial(year: number, month: number, day: number): number {
  let total = 0;
  if (year >= 1970) {
    for (let y = 1970; y < year; y += 1) total += isLeap(y) ? 366 : 365;
  } else {
    for (let y = year; y < 1970; y += 1) total -= isLeap(y) ? 366 : 365;
  }
  for (let m = 1; m < month; m += 1) total += monthDays(year, m);
  return total + day - 1;
}
function fromSerial(value: number): { year: number; month: number; day: number } {
  let year = 1970;
  let left = value;
  while (left < 0) {
    year -= 1;
    left += isLeap(year) ? 366 : 365;
  }
  while (left >= (isLeap(year) ? 366 : 365)) {
    left -= isLeap(year) ? 366 : 365;
    year += 1;
  }
  let month = 1;
  while (left >= monthDays(year, month)) {
    left -= monthDays(year, month);
    month += 1;
  }
  return { year, month, day: left + 1 };
}
function iso(year: number, month: number, day: number): string {
  return `${String(year).padStart(4, '0')}-${String(month).padStart(2, '0')}-${String(day).padStart(2, '0')}`;
}
function isoOfSerial(value: number): string {
  const { year, month, day } = fromSerial(value);
  return iso(year, month, day);
}
function serialOfIso(text: string): number {
  const [y, m, d] = text.split('-').map(Number) as [number, number, number];
  return serial(y, m, d);
}
/** A random real day between 2018 and 2040, with month and year ends and leap days often. */
function randomDay(random: ReturnType<typeof rng>): string {
  const year = 2018 + random.below(23);
  if (random.below(4) === 0) {
    const month = random.pick([1, 2, 3, 12]);
    const last = monthDays(year, month);
    const day = random.pick([1, 2, last - 1, last, 29]);
    const clamped = Math.min(Math.max(day, 1), last);
    return iso(year, month, clamped);
  }
  const month = 1 + random.below(12);
  return iso(year, month, 1 + random.below(monthDays(year, month)));
}

const SEEDS = 40;
const PER_SEED = 100;

describe('the reference itself (so a wrong reference cannot hide a wrong answer)', () => {
  it('counts the known days', () => {
    expect(serial(1970, 1, 1)).toBe(0);
    expect(serial(2000, 3, 1)).toBe(11_017);
    expect(isoOfSerial(serial(2028, 2, 29))).toBe('2028-02-29');
    expect(isoOfSerial(serial(2100, 2, 28) + 1)).toBe('2100-03-01');
    expect(isoOfSerial(serial(2026, 11, 12) - 7)).toBe('2026-11-05');
  });
});

describe('the window of 7 days (rules 1 to 3)', () => {
  it.each(Array.from({ length: SEEDS }, (_, i) => i + 1))(
    'random today and asked days, seed %i',
    (seed) => {
      const random = rng(seed);
      for (let n = 0; n < PER_SEED; n += 1) {
        const today = randomDay(random);
        const yesterday = isoOfSerial(serialOfIso(today) - 1);
        expect(lastWindowEnd(today), `yesterday of ${today}`).toBe(yesterday);
        expect(addDays(today, -1)).toBe(yesterday);
        expect(addDays(today, 0)).toBe(today);

        // Rule 1: what the window ends on.
        expect(windowEnd(null, today)).toBe(yesterday);
        const asked = randomDay(random);
        const end = windowEnd(asked, today);
        expect(end <= yesterday, `end ${end} after yesterday ${yesterday}`).toBe(true);
        expect(end).toBe(asked <= yesterday ? asked : yesterday);
        expect(windowEnd(today, today), 'asking for today gives yesterday').toBe(yesterday);

        // Rule 2: exactly seven real days, ending on the end.
        const { from, to } = windowOf(end);
        expect(to).toBe(end);
        expect(isIsoDay(from) && isIsoDay(to)).toBe(true);
        expect(serialOfIso(to) - serialOfIso(from)).toBe(WINDOW_DAYS - 1);
        expect(from).toBe(isoOfSerial(serialOfIso(end) - (WINDOW_DAYS - 1)));

        // Rule 3: steps.
        const back = stepWindowEnd(end, -1, today);
        expect(back).toBe(isoOfSerial(serialOfIso(end) - 7));
        const near = stepWindowEnd(end, 1, today);
        expect(near).toBe(
          isoOfSerial(serialOfIso(end) + 7) > yesterday
            ? yesterday
            : isoOfSerial(serialOfIso(end) + 7),
        );
        expect(near <= yesterday).toBe(true);
        expect(canComeNearer(end, today)).toBe(end < yesterday);
        expect(canComeNearer(yesterday, today)).toBe(false);
      }
    },
  );

  it('walks back 52 times and forward again without ever passing yesterday or losing a day', () => {
    const random = rng(99);
    for (let n = 0; n < 200; n += 1) {
      const today = randomDay(random);
      const yesterday = lastWindowEnd(today);
      let end = windowEnd(null, today);
      const trail = [end];
      for (let step = 0; step < 52; step += 1) {
        end = stepWindowEnd(end, -1, today);
        trail.push(end);
      }
      expect(serialOfIso(at(trail, 0)) - serialOfIso(end), 'one year is 52 weeks').toBe(364);
      // Back to the start: the same days, in reverse, with no day lost or repeated.
      for (let step = 51; step >= 0; step -= 1) {
        end = stepWindowEnd(end, 1, today);
        expect(end).toBe(trail[step]);
      }
      expect(end).toBe(yesterday);
      // One more step nearer stays on yesterday.
      expect(stepWindowEnd(end, 1, today)).toBe(yesterday);
    }
  });

  it('month ends, year ends and leap days', () => {
    const cases: [string, string][] = [
      ['2026-03-01', '2026-02-28'],
      ['2028-03-01', '2028-02-29'],
      ['2100-03-01', '2100-02-28'],
      ['2000-03-01', '2000-02-29'],
      ['2027-01-01', '2026-12-31'],
      ['2026-11-01', '2026-10-31'],
      ['2026-12-01', '2026-11-30'],
      ['2024-02-29', '2024-02-28'],
    ];
    for (const [today, yesterday] of cases) {
      expect(lastWindowEnd(today)).toBe(yesterday);
      const { from, to } = windowOf(windowEnd(null, today));
      expect(to).toBe(yesterday);
      expect(from).toBe(isoOfSerial(serialOfIso(yesterday) - 6));
    }
    // Seven days across the end of February.
    expect(windowOf('2028-03-02')).toEqual({ from: '2028-02-25', to: '2028-03-02' });
    expect(windowOf('2026-03-02')).toEqual({ from: '2026-02-24', to: '2026-03-02' });
    expect(windowOf('2027-01-03')).toEqual({ from: '2026-12-28', to: '2027-01-03' });
  });
});

// ---- the text of the date box (rule 4) -----------------------------------------------------------

const TODAY = '2026-11-12';

/** What the date box means by a text: the reference of PRD FR-7.19, written out by hand. */
function referenceOfText(text: string, today: string): string | null {
  const trimmed = text.trim();
  const [ty, tm] = today.split('-').map(Number) as [number, number];
  const real = (y: number, m: number, d: number) =>
    y >= 100 && m >= 1 && m <= 12 && d >= 1 && d <= monthDays(y, m) ? iso(y, m, d) : null;
  let match = /^(\d{1,2})-(\d{1,2})-(\d{4})$/.exec(trimmed);
  if (match) return real(Number(match[3]), Number(match[2]), Number(match[1]));
  match = /^(\d{1,2})-(\d{1,2})$/.exec(trimmed);
  if (match) return real(ty, Number(match[2]), Number(match[1]));
  match = /^(\d{1,2})$/.exec(trimmed);
  if (match) return real(ty, tm, Number(match[1]));
  return null;
}

const HOSTILE_TEXTS = [
  '',
  ' ',
  '   ',
  '\t',
  '\n',
  '0',
  '00',
  '1',
  '01',
  '31',
  '32',
  '99',
  '100',
  '-1',
  '+1',
  '1e1',
  '0x10',
  '10.5',
  '10,5',
  '10-9',
  '10-09',
  '10-9-2026',
  '10-09-2026',
  '10-9-26',
  '10-09-26',
  '10/9/2026',
  '10 09 2026',
  '2026-09-10',
  '2026-9-10',
  '31-02-2026',
  '29-02-2027',
  '29-02-2028',
  '30-02-2028',
  '31-04-2026',
  '00-00-0000',
  '00-01-2026',
  '01-00-2026',
  '01-13-2026',
  '99-99-9999',
  '01-01-0000',
  '01-01-0001',
  '01-01-0050',
  '01-01-0100',
  '31-12-9999',
  '01-01-10000',
  '١٠-٠٩-٢٠٢٦',
  '۱۰-۰۹-۲۰۲۶',
  '١٠',
  '۳۰',
  '１０-０９-２０２６',
  '10\u200b-09-2026',
  '10-09-2026\u200b',
  '\u200f10-09-2026',
  '10\u0000-09-2026',
  '10-09-2026\u0000',
  ' 10-09-2026 ',
  '10-09-2026 extra',
  "'; DROP TABLE bikri_entries; --",
  "' OR 1=1 --",
  '<script>alert(1)</script>',
  '..\\..\\windows',
  '10-09-2026'.repeat(50),
  '9'.repeat(10_000),
  '1-'.repeat(5_000),
  '-'.repeat(10_000),
  '99999999999999999999-1-2026',
];

describe('the text of "تاریخ پر جائیں" (rule 4)', () => {
  it.each(HOSTILE_TEXTS.map((text, index) => [index, text.slice(0, 24)] as const))(
    'hostile text %i: "%s"',
    (index) => {
      const text = at(HOSTILE_TEXTS, index);
      const read: { day: string | null } = { day: 'not read' };
      expect(() => {
        read.day = dayOfText(text, TODAY);
      }).not.toThrow();
      expect(read.day, `text #${String(index)}`).toBe(referenceOfText(text, TODAY));
      if (read.day !== null) expect(isIsoDay(read.day)).toBe(true);
    },
  );

  it('the shortcuts mean what FR-7.19 says, in a short month, on a leap day and at a year end', () => {
    expect(dayOfText('10', '2026-11-12')).toBe('2026-11-10');
    expect(dayOfText('31', '2026-11-12')).toBeNull(); // November has 30 days
    expect(dayOfText('29', '2026-02-12')).toBeNull();
    expect(dayOfText('29', '2028-02-12')).toBe('2028-02-29');
    expect(dayOfText('10-9', '2026-11-12')).toBe('2026-09-10');
    expect(dayOfText('29-2', '2027-01-05')).toBeNull();
    expect(dayOfText('29-2', '2028-01-05')).toBe('2028-02-29');
    expect(dayOfText('31-12', '2026-01-01')).toBe('2026-12-31');
    expect(dayOfText('1', '2027-01-01')).toBe('2027-01-01');
  });

  it.each(Array.from({ length: SEEDS }, (_, i) => i + 1))(
    'random text from digits, dashes and strange marks, seed %i',
    (seed) => {
      const random = rng(seed + 500);
      const alphabet = [
        '0',
        '1',
        '2',
        '3',
        '4',
        '5',
        '6',
        '7',
        '8',
        '9',
        '-',
        '-',
        ' ',
        '/',
        '.',
        ',',
        '+',
        'e',
        '\u0663',
        '\u06f4',
        '\u200b',
        '\u200d',
        '\u0000',
        'a',
        '\u0627',
        '\n',
      ];
      for (let n = 0; n < PER_SEED; n += 1) {
        const length = random.below(14);
        let text = '';
        for (let i = 0; i < length; i += 1) text += random.pick(alphabet);
        const today = randomDay(random);
        const day = dayOfText(text, today);
        expect(day, `text ${JSON.stringify(text)} today ${today}`).toBe(
          referenceOfText(text, today),
        );
        if (day !== null) expect(isIsoDay(day)).toBe(true);
      }
    },
  );

  it('a real full date in the future is read as that day, so the box can refuse it in words', () => {
    // The refusal "after today" is the box's job (PastGoTo); the reader must not clamp it silently.
    expect(dayOfText('13-11-2026', TODAY)).toBe('2026-11-13');
    expect(dayOfText('12-11-2026', TODAY)).toBe('2026-11-12');
    expect(dayOfText('01-01-2100', TODAY)).toBe('2100-01-01');
  });
});

describe('isIsoDay', () => {
  it('accepts only a real day written YYYY-MM-DD', () => {
    for (const good of ['2026-11-12', '2028-02-29', '2000-02-29', '0100-01-01', '9999-12-31']) {
      expect(isIsoDay(good), good).toBe(true);
    }
    for (const bad of [
      '2026-02-29',
      '2100-02-29',
      '2026-13-01',
      '2026-00-10',
      '2026-01-00',
      '2026-1-1',
      '26-01-01',
      '0000-01-01',
      '0001-01-01',
      '0050-06-06',
      ' 2026-11-12',
      '2026-11-12 ',
      '2026-11-12\n',
      '٢٠٢٦-١١-١٢',
      '2026-11-12T00:00:00Z',
      '10-11-2026',
      '',
    ]) {
      expect(isIsoDay(bad), JSON.stringify(bad)).toBe(false);
    }
    for (const notText of [null, undefined, 20_261_112, {}, [], ['2026-11-12'], true]) {
      expect(isIsoDay(notText)).toBe(false);
    }
  });
});

// ---- the address (rule 5) ------------------------------------------------------------------------

describe('the address of the screen (rule 5)', () => {
  const HOSTILE: Record<string, unknown>[] = [
    {},
    { end: null, row: null, all: null },
    { end: '', row: '', all: '' },
    { end: '2026-02-30', row: -1, all: 'yes' },
    { end: '2026-11-12 ', row: 0, all: 1 },
    { end: '٢٠٢٦-١١-١٢', row: 1.5, all: 'TRUE' },
    { end: ['2026-11-12'], row: '1e3', all: [true] },
    { end: { toString: () => '2026-11-12' }, row: Number.NaN, all: {} },
    { end: 20_261_112, row: Number.POSITIVE_INFINITY, all: 'true' },
    { end: '2026-11-12', row: Number.MAX_SAFE_INTEGER + 2, all: true },
    { end: '0050-01-01', row: 1e21, all: false },
    { end: '2026-11-12', row: '12', all: 'true' },
    { end: '2026-11-12', row: 12, all: true, extra: '<script>' },
    { end: "'; DROP TABLE", row: '1; DROP TABLE', all: 'true' },
    { end: '9'.repeat(10_000), row: '9'.repeat(10_000), all: '9'.repeat(10_000) },
    { row: -0 },
    { row: ' 12 ' },
    { row: '0x1f' },
    { row: true },
    { row: [7] },
  ];

  it.each(HOSTILE.map((search, index) => [index, search] as const))(
    'hostile address %i keeps only what is real',
    (_index, search) => {
      let parsed: ReturnType<typeof parsePastSearch> = {};
      expect(() => {
        parsed = parsePastSearch(search);
      }).not.toThrow();
      if (parsed.end !== undefined) expect(isIsoDay(parsed.end)).toBe(true);
      if (parsed.row !== undefined) {
        expect(Number.isSafeInteger(parsed.row)).toBe(true);
        expect(parsed.row).toBeGreaterThan(0);
      }
      if (parsed.all !== undefined) expect(parsed.all).toBe(true);
      // Nothing else is kept.
      expect(Object.keys(parsed).every((key) => ['end', 'row', 'all'].includes(key))).toBe(true);
      // The way back keeps a real day and the true flag, nothing else.
      const back = parsePastReturn({ past: search['end'], pastAll: search['all'] });
      if (back !== undefined) {
        expect(isIsoDay(back.past)).toBe(true);
        if (back.pastAll !== undefined) expect(back.pastAll).toBe(true);
      }
    },
  );

  it('keeps what is real exactly', () => {
    expect(parsePastSearch({ end: '2026-11-05', row: 42, all: true })).toEqual({
      end: '2026-11-05',
      row: 42,
      all: true,
    });
    expect(parsePastSearch({ end: '2026-11-05', row: '42', all: 'true' })).toEqual({
      end: '2026-11-05',
      row: 42,
      all: true,
    });
    expect(parsePastSearch({})).toEqual({});
    expect(parsePastReturn({ past: '2026-11-05', pastAll: true })).toEqual({
      past: '2026-11-05',
      pastAll: true,
    });
    expect(parsePastReturn({ past: '2026-02-30' })).toBeUndefined();
  });
});

// ---- the headings (rule 6) -----------------------------------------------------------------------

describe('the day headings of a page (rule 6)', () => {
  it.each(Array.from({ length: SEEDS }, (_, i) => i + 1))(
    'random windows and pages, seed %i',
    (seed) => {
      const random = rng(seed + 900);
      for (let n = 0; n < PER_SEED; n += 1) {
        const newest = randomDay(random);
        const days = Array.from({ length: 7 }, (_, i) => isoOfSerial(serialOfIso(newest) - i));
        // Rows on a page: a run of consecutive rows of a newest-first list, so their days never go up.
        const start = random.below(7);
        const length = random.below(8);
        const rowDays: string[] = [];
        let position = start;
        for (let i = 0; i < length; i += 1) {
          if (random.below(3) === 0 && position < 6) position += 1;
          rowDays.push(at(days, position));
        }
        const firstPage = random.below(2) === 0;
        const lastPage = random.below(2) === 0;
        const shown = headingDaysOnPage(days, rowDays, firstPage, lastPage);
        const label = `days ${days.join(',')} rows ${rowDays.join(',')} first ${String(firstPage)} last ${String(lastPage)}`;

        // Never a day outside the window, never twice, always in the window's order.
        expect(new Set(shown).size, label).toBe(shown.length);
        for (const day of shown) expect(days.includes(day), label).toBe(true);
        const indexes = shown.map((day) => days.indexOf(day));
        expect(indexes, label).toEqual([...indexes].sort((a, b) => a - b));
        // A run with no gap in it.
        for (let i = 1; i < indexes.length; i += 1) {
          expect(indexes[i], label).toBe(at(indexes, i - 1) + 1);
        }
        if (rowDays.length === 0) {
          expect(shown, label).toEqual(firstPage ? days : []);
        } else {
          // Every row sits under a heading.
          for (const day of rowDays) expect(shown.includes(day), label).toBe(true);
          if (firstPage) expect(shown[0], label).toBe(days[0]);
          if (lastPage) expect(shown[shown.length - 1], label).toBe(days[days.length - 1]);
        }
      }
    },
  );

  it('rows of a day the window does not know show every heading and never lose a row', () => {
    const days = ['2026-11-11', '2026-11-10', '2026-11-09'];
    expect(headingDaysOnPage(days, ['2026-10-01'], false, false)).toEqual(days);
    expect(headingDaysOnPage(days, ['2026-11-10', '1999-01-01'], false, false)).toEqual(days);
    expect(headingDaysOnPage([], ['2026-11-10'], true, true)).toEqual([]);
    expect(headingDaysOnPage([], [], false, false)).toEqual([]);
  });

  it('rows out of date order, which a sort on a column title gives, are why the screen draws no heading then', () => {
    // usePastGroups returns nothing while a sort is on, so this order never reaches the function. Shown
    // here so the day it does, the slice is empty and not wrong: the rows are not lost, they have no heading.
    const days = ['2026-11-11', '2026-11-10', '2026-11-09'];
    expect(headingDaysOnPage(days, ['2026-11-09', '2026-11-11'], false, false)).toEqual([]);
  });
});
