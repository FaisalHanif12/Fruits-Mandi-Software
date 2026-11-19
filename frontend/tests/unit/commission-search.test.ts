// The address of the commission page (plan 77, PRD FR-9.3): what it reads, what it drops, and the address
// the page writes back. A month address of plan 76 (`?month=2026-10`) still opens that month; a party needs its
// side; only the values of the kind on screen are kept; the current month and the month kind are no keys.

import { describe, expect, it } from 'vitest';
import { carryToKind } from '../../src/features/dashboard/components/PeriodFilterBox';
import {
  addYears,
  commissionSearchOfSource,
  periodSearchOf,
  filtersOfSearch,
  isIsoYear,
  isPlainCommission,
  parseCommissionSearch,
  searchOfFilters,
  yearOf,
} from '../../src/features/dashboard/commission-search';

const RAHIM = { id: 7, name_ur: 'رحیم', name_en: 'Rahim', code: '0010' };

describe('reading the address', () => {
  it('opens the month of plan 76 as it always did', () => {
    const search = parseCommissionSearch({ month: '2026-10' });
    expect(search).toEqual({ month: '2026-10' });
    expect(filtersOfSearch(search)).toMatchObject({ kind: 'month', month: '2026-10', side: null });
  });

  it('reads a year, a day and a range, each with its kind', () => {
    expect(filtersOfSearch(parseCommissionSearch({ period: 'year', year: '2026' }))).toMatchObject({
      kind: 'year',
      year: '2026',
    });
    expect(
      filtersOfSearch(parseCommissionSearch({ period: 'day', date: '2026-10-10' })),
    ).toMatchObject({ kind: 'day', day: '2026-10-10' });
    expect(
      filtersOfSearch(
        parseCommissionSearch({ period: 'range', from: '2026-10-01', to: '2026-10-10' }),
      ),
    ).toMatchObject({ kind: 'range', from: '2026-10-01', to: '2026-10-10' });
  });

  it('reads a year that the router made a number', () => {
    expect(parseCommissionSearch({ period: 'year', year: 2026 })).toEqual({
      period: 'year',
      year: '2026',
    });
  });

  it('drops a word the page does not know and a date that is not real', () => {
    const search = parseCommissionSearch({
      period: 'decade',
      month: '2026-13',
      date: '2026-02-31',
      year: '26',
      from: 'yesterday',
      side: 'partner',
    });
    expect(search).toEqual({});
  });

  it('reads a party with its side, and drops a party that has no side', () => {
    expect(parseCommissionSearch({ side: 'seller', party: RAHIM })).toEqual({
      side: 'seller',
      party: RAHIM,
    });
    expect(parseCommissionSearch({ party: RAHIM })).toEqual({});
    // A side alone is kept: the box is on that side and nobody is picked yet.
    expect(parseCommissionSearch({ side: 'buyer' })).toEqual({ side: 'buyer' });
  });

  it('reads the person key of a چکتا buyer only with a party', () => {
    expect(parseCommissionSearch({ side: 'buyer', party: RAHIM, person: ' k-1 ' })).toMatchObject({
      person: 'k-1',
    });
    expect(parseCommissionSearch({ side: 'buyer', person: 'k-1' })).toEqual({ side: 'buyer' });
  });
});

describe('writing the address', () => {
  const base = filtersOfSearch({});

  it('writes nothing for the plain page, the current month of every party', () => {
    expect(searchOfFilters(base)).toEqual({});
    expect(isPlainCommission(base)).toBe(true);
  });

  it('writes the month, and the party with its side', () => {
    const filters = { ...base, month: '2026-09', side: 'seller' as const, party: RAHIM };
    expect(searchOfFilters(filters)).toEqual({ month: '2026-09', side: 'seller', party: RAHIM });
    expect(isPlainCommission(filters)).toBe(false);
  });

  it('keeps only the values of the kind on screen', () => {
    const filters = {
      ...base,
      kind: 'year' as const,
      year: '2026',
      month: '2026-09',
      day: '2026-10-10',
      from: '2026-10-01',
    };
    expect(searchOfFilters(filters)).toEqual({ period: 'year', year: '2026' });
    expect(
      searchOfFilters({ ...base, kind: 'range', from: '2026-10-01', to: '2026-10-10' }),
    ).toEqual({ period: 'range', from: '2026-10-01', to: '2026-10-10' });
  });

  it('drops the party when the side is cleared', () => {
    expect(searchOfFilters({ ...base, side: null, party: RAHIM })).toEqual({});
  });
});

describe('years', () => {
  it('reads a year of the books and moves it', () => {
    expect(isIsoYear('2026')).toBe(true);
    expect(isIsoYear('26')).toBe(false);
    expect(isIsoYear('1800')).toBe(false);
    expect(addYears('2026', -1)).toBe('2025');
    expect(addYears('2026', 1)).toBe('2027');
    expect(yearOf('2026-10-10')).toBe('2026');
    expect(yearOf('2026-10')).toBe('2026');
  });
});

describe('changing the period kind', () => {
  const now = filtersOfSearch({});
  const TODAY = '2026-10-10';

  it('carries the place on screen: a month becomes its year, a date its month', () => {
    expect(carryToKind('year', { ...now, month: '2026-09' }, TODAY)).toMatchObject({
      kind: 'year',
      year: '2026',
    });
    expect(carryToKind('month', { ...now, kind: 'day', day: '2026-09-15' }, TODAY)).toMatchObject({
      kind: 'month',
      month: '2026-09',
    });
    expect(carryToKind('day', { ...now, kind: 'day', day: '2026-09-15' }, TODAY)).toMatchObject({
      kind: 'day',
      day: '2026-09-15',
    });
  });

  it('starts at the latest when nothing was chosen yet', () => {
    expect(carryToKind('year', now, TODAY)).toEqual({
      kind: 'year',
      month: null,
      day: null,
      year: null,
      from: null,
      to: null,
    });
  });

  it('opens a month as a range from its first day to its last, or to today when it is not over', () => {
    expect(carryToKind('range', { ...now, month: '2026-09' }, TODAY)).toMatchObject({
      kind: 'range',
      from: '2026-09-01',
      to: '2026-09-30',
    });
    // The current month is not over: its end is "today", which is the latest (null).
    expect(carryToKind('range', { ...now, month: '2026-10' }, TODAY)).toMatchObject({
      from: '2026-10-01',
      to: null,
    });
  });

  it('opens a year as a range from 1 January, to 31 December or to today when the year is not over', () => {
    expect(carryToKind('range', { ...now, kind: 'year', year: '2025' }, TODAY)).toMatchObject({
      from: '2025-01-01',
      to: '2025-12-31',
    });
    expect(carryToKind('range', { ...now, kind: 'year', year: '2026' }, TODAY)).toMatchObject({
      from: '2026-01-01',
      to: null,
    });
  });
});

describe('the page an answer of the question panel opens', () => {
  it('maps the days an answer read to a date, a month, a year or a range', () => {
    expect(periodSearchOf('2026-10-10', '2026-10-10', null)).toEqual({
      period: 'day',
      date: '2026-10-10',
    });
    expect(periodSearchOf('2026-10-01', '2026-10-31', null)).toEqual({ month: '2026-10' });
    expect(periodSearchOf('2026-02-01', '2026-02-28', null)).toEqual({ month: '2026-02' });
    expect(periodSearchOf('2026-01-01', '2026-12-31', null)).toEqual({
      period: 'year',
      year: '2026',
    });
    // The month so far is an exact range: the page never shows more than the answer read.
    expect(periodSearchOf('2026-10-01', '2026-10-10', null)).toEqual({
      period: 'range',
      from: '2026-10-01',
      to: '2026-10-10',
    });
  });

  it('a question with no dates opens every date from the first day with entries to today', () => {
    expect(periodSearchOf('', '', '2026-09-20')).toEqual({ period: 'range', from: '2026-09-20' });
    // An empty end alone is today; a given start is kept.
    expect(periodSearchOf('2026-09-01', '', '2026-09-20')).toEqual({
      period: 'range',
      from: '2026-09-01',
    });
  });

  it('with the first day not known yet, it opens the current month and never guesses a date', () => {
    expect(periodSearchOf('', '', null)).toEqual({});
    expect(periodSearchOf('', '2026-10-10', null)).toEqual({});
  });

  it('opens the party with his side, names and code, and a چکتا buyer with his key', () => {
    const source = {
      kind: 'commission',
      result_id: 'r2',
      side: 'seller',
      party: {
        record: { id: 19, code: '0010', name_ur: 'رحیم', name_en: 'Rahim', tag: null },
        person_key: null,
      },
      from: '2026-10-01',
      to: '2026-10-31',
    } as const;
    expect(commissionSearchOfSource(source, null)).toEqual({
      month: '2026-10',
      side: 'seller',
      party: { id: 19, name_ur: 'رحیم', name_en: 'Rahim', code: '0010' },
    });
    const walkIn = {
      ...source,
      side: 'buyer',
      party: { record: { ...source.party.record, id: 30, code: '0030' }, person_key: 'shahid' },
    } as const;
    expect(commissionSearchOfSource(walkIn, null)).toMatchObject({
      side: 'buyer',
      person: 'shahid',
    });
  });
});
