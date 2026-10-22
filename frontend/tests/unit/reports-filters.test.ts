// The filters of a report (plan 20, PRD FR-19.1, FR-19.2, FR-19.10): what the boxes hold becomes the
// filters the backend reads, with every key sent and nothing for a box the report does not take;
// the address of a report starts the boxes; a ledger cannot run without its party. The screen sends
// text and ids and works out nothing.

import { describe, expect, it } from 'vitest';
import { groupCatalog, helpIdOf } from '../../src/features/reports/catalog';
import {
  EMPTY_FILTERS,
  canRun,
  filtersToWire,
  isUnfiltered,
  periodMessageId,
  type ReportFilterState,
} from '../../src/features/reports/filters';
import { parseReportSearch, stateFromSearch } from '../../src/features/reports/search';
import {
  CATALOG,
  R11_TODAY_REQUEST,
  R13_AKRAM_REQUEST,
  R18_AS_OF_REQUEST,
} from '../helpers/reports-real';

const PARTY = { id: 20, name_ur: 'اکرم ٹول ٹیکس', name_en: 'Akram Toll Tax', code: '03040001' };
const entryOf = (id: string) => {
  const found = CATALOG.find((entry) => entry.id === id);
  if (found === undefined) throw new Error(`no ${id}`);
  return found;
};

describe('filtersToWire', () => {
  it('sends every key, empty, for nothing filled', () => {
    expect(filtersToWire(EMPTY_FILTERS, entryOf('R11').filters)).toEqual({
      shortcut: null,
      from: '',
      to: '',
      as_of: '',
      party: null,
      seller_id: null,
      zamindar_id: null,
      item_id: null,
      city_id: null,
      adda_id: null,
      detail: false,
    });
  });

  it('is exactly the request the backend wrote for the day book of today', () => {
    const state: ReportFilterState = { ...EMPTY_FILTERS, period: 'today' };
    expect(filtersToWire(state, entryOf('R11').filters)).toEqual(R11_TODAY_REQUEST.filters);
  });

  it('is exactly the request the backend wrote for the as-of day of R18', () => {
    const state: ReportFilterState = { ...EMPTY_FILTERS, asOf: '09-10-2026' };
    expect(filtersToWire(state, entryOf('R18').filters)).toEqual(R18_AS_OF_REQUEST.filters);
  });

  it('is exactly the request the backend wrote for the ledger of اکرم', () => {
    const state: ReportFilterState = { ...EMPTY_FILTERS, party: PARTY };
    expect(filtersToWire(state, entryOf('R13').filters)).toEqual(R13_AKRAM_REQUEST.filters);
  });

  it('a shortcut wins over the dates, and the dates are not sent with it', () => {
    const state: ReportFilterState = {
      ...EMPTY_FILTERS,
      period: 'this_week',
      from: '01-10-2026',
      to: '10-10-2026',
    };
    const wire = filtersToWire(state, entryOf('R11').filters);
    expect(wire.shortcut).toBe('this_week');
    expect(wire.from).toBe('');
    expect(wire.to).toBe('');
  });

  it('trims what was typed, and sends dates as typed', () => {
    const wire = filtersToWire(
      { ...EMPTY_FILTERS, from: ' 10 ', to: ' 10-9 ' },
      entryOf('R11').filters,
    );
    expect(wire.from).toBe('10');
    expect(wire.to).toBe('10-9');
  });

  it('leaves out a box the report does not take, even when a left-over value is in it', () => {
    const left: ReportFilterState = {
      ...EMPTY_FILTERS,
      period: 'today',
      from: '01-10-2026',
      asOf: '09-10-2026',
      party: PARTY,
      seller: PARTY,
      item: PARTY,
      detail: true,
    };
    // R19 takes nothing: the backend would refuse every one of these (ERR_OUT_OF_RANGE).
    expect(filtersToWire(left, entryOf('R19').filters)).toEqual({
      shortcut: null,
      from: '',
      to: '',
      as_of: '',
      party: null,
      seller_id: null,
      zamindar_id: null,
      item_id: null,
      city_id: null,
      adda_id: null,
      detail: false,
    });
    // R18 takes only the day.
    const r18 = filtersToWire(left, entryOf('R18').filters);
    expect(r18.as_of).toBe('09-10-2026');
    expect(r18.party).toBeNull();
    expect(r18.shortcut).toBeNull();
    expect(r18.detail).toBe(false);
  });

  it('the detail tick is sent only for a report that has it (R14)', () => {
    const state: ReportFilterState = { ...EMPTY_FILTERS, party: PARTY, detail: true };
    expect(filtersToWire(state, entryOf('R14').filters).detail).toBe(true);
    expect(filtersToWire(state, entryOf('R13').filters).detail).toBe(false);
  });

  it('the city of R16 is sent as its id', () => {
    const state: ReportFilterState = {
      ...EMPTY_FILTERS,
      city: { id: 7, name_ur: 'کوٹ رادھا کشن', name_en: 'Kot Radha Kishan', code: undefined },
    };
    expect(filtersToWire(state, entryOf('R16').filters).city_id).toBe(7);
  });
});

describe('canRun and isUnfiltered', () => {
  it('a ledger and a statement need a party; the others run at once', () => {
    expect(canRun(EMPTY_FILTERS, entryOf('R13'))).toBe(false);
    expect(canRun(EMPTY_FILTERS, entryOf('R14'))).toBe(false);
    expect(canRun({ ...EMPTY_FILTERS, party: PARTY }, entryOf('R13'))).toBe(true);
    expect(canRun(EMPTY_FILTERS, entryOf('R18'))).toBe(true);
  });

  it('only a box the report takes counts as a filter to clear', () => {
    const r18 = entryOf('R18').filters;
    expect(isUnfiltered(EMPTY_FILTERS, r18)).toBe(true);
    expect(isUnfiltered({ ...EMPTY_FILTERS, asOf: '09-10-2026' }, r18)).toBe(false);
    expect(isUnfiltered({ ...EMPTY_FILTERS, party: PARTY }, r18)).toBe(true);
  });
});

describe('the address of a report', () => {
  it('reads the period, the dates, the day and the party; anything else is left out', () => {
    expect(
      parseReportSearch({
        period: 'last_month',
        from: ' 01-10-2026 ',
        to: '',
        as_of: '09-10-2026',
        party: '20',
        junk: 'x',
      }),
    ).toEqual({ period: 'last_month', from: '01-10-2026', as_of: '09-10-2026', party: 20 });
    expect(parseReportSearch({ period: 'someday', party: '-3' })).toEqual({});
    expect(parseReportSearch({ party: 'abc' })).toEqual({});
  });

  it('starts the boxes: a party comes without names, to be filled by the answer', () => {
    const state = stateFromSearch({ party: 20, from: '01-10-2026' });
    expect(state.party).toEqual({ id: 20, name_ur: '', name_en: '', code: undefined });
    expect(state.from).toBe('01-10-2026');
    expect(state.period).toBe('custom');
  });
});

describe('the catalogue', () => {
  it('groups in the order of PRD 19.2 and leaves out the groups with no report', () => {
    expect(groupCatalog(CATALOG).map((group) => group.group)).toEqual(['accounts']);
    const sales = {
      ...entryOf('R11'),
      group: 'sales' as const,
      group_title_id: 'report-group-sales',
    };
    const business = {
      ...entryOf('R13'),
      group: 'business' as const,
      group_title_id: 'report-group-business',
    };
    expect(groupCatalog([business, ...CATALOG, sales]).map((group) => group.group)).toEqual([
      'sales',
      'accounts',
      'business',
    ]);
  });

  it('puts R9 before R11 inside a group (a number, not a text, order)', () => {
    const nine = { ...entryOf('R11'), id: 'R2' as never };
    const ids = groupCatalog([entryOf('R18'), entryOf('R11'), nine])[0]?.entries.map(
      (entry) => entry.id,
    );
    expect(ids).toEqual(['R2', 'R11', 'R18']);
  });

  it('the help message of a report is the title message with -help', () => {
    expect(helpIdOf(entryOf('R18'))).toBe('report-r18-help');
  });

  it('every period choice has a message', () => {
    expect(periodMessageId('this_week')).toBe('report-period-this-week');
    expect(periodMessageId('custom')).toBe('report-period-custom');
  });
});
