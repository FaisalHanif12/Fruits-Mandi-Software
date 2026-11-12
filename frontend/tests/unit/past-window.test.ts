// The window of earlier days of پچھلی بکری (plan 72 P4 to P6, P10): the date arithmetic, which headings a
// page of rows gets, and the address that keeps the window and the row. Pure functions, no screen.

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
  pastAddress,
  stepWindowEnd,
  windowEnd,
  windowOf,
} from '../../src/features/bikri/past-window';

describe('the window of 7 days', () => {
  it('is 7 days, and the first window of 12-11-2026 is 05-11-2026 to 11-11-2026 (PRD 12.7)', () => {
    expect(WINDOW_DAYS).toBe(7);
    expect(lastWindowEnd('2026-11-12')).toBe('2026-11-11');
    expect(windowOf('2026-11-11')).toEqual({ from: '2026-11-05', to: '2026-11-11' });
  });

  it('one step back is 29-10-2026 to 04-11-2026', () => {
    const end = stepWindowEnd('2026-11-11', -1, '2026-11-12');
    expect(windowOf(end)).toEqual({ from: '2026-10-29', to: '2026-11-04' });
  });

  it('never goes past yesterday', () => {
    expect(stepWindowEnd('2026-11-11', 1, '2026-11-12')).toBe('2026-11-11');
    expect(stepWindowEnd('2026-11-08', 1, '2026-11-12')).toBe('2026-11-11');
    expect(windowEnd('2026-11-12', '2026-11-12')).toBe('2026-11-11');
    expect(windowEnd('2027-01-01', '2026-11-12')).toBe('2026-11-11');
    expect(windowEnd(null, '2026-11-12')).toBe('2026-11-11');
    expect(windowEnd('2026-11-05', '2026-11-12')).toBe('2026-11-05');
  });

  it('can come nearer only while it does not end on yesterday', () => {
    expect(canComeNearer('2026-11-11', '2026-11-12')).toBe(false);
    expect(canComeNearer('2026-11-04', '2026-11-12')).toBe(true);
  });

  it('moves over a month and a year end', () => {
    expect(addDays('2026-03-01', -1)).toBe('2026-02-28');
    expect(addDays('2024-03-01', -1)).toBe('2024-02-29');
    expect(addDays('2027-01-03', -7)).toBe('2026-12-27');
    expect(windowOf('2027-01-03')).toEqual({ from: '2026-12-28', to: '2027-01-03' });
  });

  it('knows a real day from a text that only looks like one', () => {
    expect(isIsoDay('2026-02-28')).toBe(true);
    expect(isIsoDay('2026-02-30')).toBe(false);
    expect(isIsoDay('10-10-2026')).toBe(false);
    expect(isIsoDay(20261010)).toBe(false);
  });
});

describe('a typed date', () => {
  it('reads the day, with the shortcuts of FR-7.19', () => {
    expect(dayOfText('05-10-2026', '2026-10-10')).toBe('2026-10-05');
    expect(dayOfText('5-9', '2026-10-10')).toBe('2026-09-05');
    expect(dayOfText('5', '2026-10-10')).toBe('2026-10-05');
  });

  it('is nothing for a text that is not a real day', () => {
    expect(dayOfText('31-02-2026', '2026-10-10')).toBeNull();
    expect(dayOfText('abc', '2026-10-10')).toBeNull();
    expect(dayOfText('', '2026-10-10')).toBeNull();
  });
});

describe('the headings of a page of the window', () => {
  // 7 days, newest first, as the backend sends them.
  const days = [
    '2026-10-09',
    '2026-10-08',
    '2026-10-07',
    '2026-10-06',
    '2026-10-05',
    '2026-10-04',
    '2026-10-03',
  ];

  it('the only page has every day, the empty ones too', () => {
    expect(headingDaysOnPage(days, ['2026-10-09', '2026-10-09', '2026-10-07'], true, true)).toEqual(
      days,
    );
  });

  it('a window with no rows has all its days on its first page and none on another', () => {
    expect(headingDaysOnPage(days, [], true, true)).toEqual(days);
    expect(headingDaysOnPage(days, [], false, true)).toEqual([]);
  });

  it('the first page starts with the days before its rows and stops at its last row', () => {
    // The rows of 09 and 07 are on page 1 of more; 06 goes on to page 2.
    expect(headingDaysOnPage(days, ['2026-10-07', '2026-10-07'], true, false)).toEqual([
      '2026-10-09',
      '2026-10-08',
      '2026-10-07',
    ]);
  });

  it('a middle page has the days from its first row to its last row, and a day that goes on shows again', () => {
    expect(headingDaysOnPage(days, ['2026-10-07', '2026-10-05'], false, false)).toEqual([
      '2026-10-07',
      '2026-10-06',
      '2026-10-05',
    ]);
  });

  it('the last page ends with the empty days after its rows', () => {
    expect(headingDaysOnPage(days, ['2026-10-05'], false, true)).toEqual([
      '2026-10-05',
      '2026-10-04',
      '2026-10-03',
    ]);
  });

  it('a row on a day the backend did not list leaves every heading, so nothing is lost', () => {
    expect(headingDaysOnPage(days, ['2026-09-01'], false, false)).toEqual(days);
  });
});

describe('the address', () => {
  it('keeps the window, the row and the tick, and drops what is not valid', () => {
    expect(parsePastSearch({ end: '2026-10-02', row: '41', all: 'true' })).toEqual({
      end: '2026-10-02',
      row: 41,
      all: true,
    });
    expect(parsePastSearch({ end: '2026-02-30', row: 'x', all: 'no' })).toEqual({});
    expect(parsePastSearch({})).toEqual({});
  });

  it('the sale book keeps the way back, and the way back is the window and the row', () => {
    const back = parsePastReturn({ past: '2026-10-02', pastAll: true });
    expect(back).toEqual({ past: '2026-10-02', pastAll: true });
    if (back === undefined) throw new Error('no way back');
    expect(pastAddress(back, 41)).toEqual({ end: '2026-10-02', row: 41, all: true });
    expect(parsePastReturn({ past: 'yesterday' })).toBeUndefined();
    expect(parsePastReturn({})).toBeUndefined();
    expect(pastAddress({ past: '2026-10-02' }, 7)).toEqual({ end: '2026-10-02', row: 7 });
  });
});
