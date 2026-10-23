// The filters of a report (plan 20, PRD FR-19.1, FR-19.2, FR-19.10): what the boxes hold becomes the
// filters the backend reads, with every key sent and nothing for a box the report does not take;
// the address of a report starts the boxes; a ledger cannot run without its party. The screen sends
// text and ids and works out nothing.

import { describe, expect, it } from 'vitest';
import type { ReportFilters } from '../../src/bindings/ReportFilters';
import type { ReportRunInput } from '../../src/bindings/ReportRunInput';
import { groupCatalog, helpIdOf, partyBoxOf } from '../../src/features/reports/catalog';
import {
  EMPTY_FILTERS,
  canRun,
  filtersToWire,
  isUnfiltered,
  periodMessageId,
  personFromHit,
  type ReportFilterState,
} from '../../src/features/reports/filters';
import {
  emptyNameMessageId,
  monthMessageId,
  monthText,
  percentText,
} from '../../src/features/reports/cells';
import { parseReportSearch, stateFromSearch } from '../../src/features/reports/search';
import {
  CATALOG,
  PEOPLE_SHAHID,
  R11_TODAY_REQUEST,
  R13_AKRAM_REQUEST,
  R18_AS_OF_REQUEST,
  R2_ALL_REQUEST,
  R2_AKRAM_REQUEST,
  R2_UNKNOWN_SELLER_REQUEST,
  R20_AS_OF_10_10_REQUEST,
  R20_DUES_REQUEST,
  R24_PAYABLES_REQUEST,
  R28_BY_DAY_REQUEST,
  R28_BY_MONTH_REQUEST,
  R3_CASH_REQUEST,
  R4_CREDIT_REQUEST,
  R7_ITEMS_REQUEST,
  R9_ALL_REQUEST,
  R9_DUE_REQUEST,
  R9_NOT_A_ONE_TIME_ACCOUNT_REQUEST,
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
    expect(groupCatalog(CATALOG).map((group) => group.group)).toEqual([
      'sales',
      'accounts',
      'collections',
      'business',
    ]);
    const accounts = CATALOG.filter((entry) => entry.group === 'accounts');
    expect(groupCatalog(accounts).map((group) => group.group)).toEqual(['accounts']);
    const business = {
      ...entryOf('R13'),
      group: 'business' as const,
      group_title_id: 'report-group-business',
    };
    const sales = {
      ...entryOf('R11'),
      group: 'sales' as const,
      group_title_id: 'report-group-sales',
    };
    expect(groupCatalog([business, ...accounts, sales]).map((group) => group.group)).toEqual([
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

/**
 * The filters of a request the backend wrote, as the screen sends them. The backend's requests
 * carry the two newer keys (`group_by`, `status`) as null when unset; the screen leaves them out,
 * which the backend reads the same way (its default), so null and missing are one thing here.
 */
function wireOf(request: ReportRunInput): ReportFilters {
  const { group_by: groupBy, status, ...rest } = request.filters;
  return {
    ...rest,
    ...(groupBy === undefined || (groupBy as unknown) === null ? {} : { group_by: groupBy }),
    ...(status === undefined || (status as unknown) === null ? {} : { status }),
  };
}

const ITEM = { id: 1, name_ur: 'کیلا', name_en: 'Banana', code: undefined };
const SELLER_999 = { id: 999999, name_ur: 'x', name_en: 'x', code: undefined };

describe('the requests the screen builds are the ones the backend wrote (round 4 reports)', () => {
  const cases: [string, ReportRunInput, Partial<ReportFilterState>][] = [
    ['R2 for اکرم (the buyer)', R2_AKRAM_REQUEST, { party: PARTY }],
    ['R2 with nothing filled', R2_ALL_REQUEST, {}],
    ['R2 for a seller that does not exist', R2_UNKNOWN_SELLER_REQUEST, { seller: SELLER_999 }],
    ['R3 for today', R3_CASH_REQUEST, { period: 'today' }],
    ['R4 for one item', R4_CREDIT_REQUEST, { item: ITEM }],
    ['R7 for one day, typed', R7_ITEMS_REQUEST, { from: '10-10-2026', to: '10-10-2026' }],
    ['R9 with nothing filled', R9_ALL_REQUEST, {}],
    ['R9 with the due lines only', R9_DUE_REQUEST, { paid: 'due' }],
    ['R20 with nothing filled', R20_DUES_REQUEST, {}],
    ['R20 as of one day', R20_AS_OF_10_10_REQUEST, { asOf: '10-10-2026' }],
    ['R24 with nothing filled', R24_PAYABLES_REQUEST, {}],
    ['R28 by day', R28_BY_DAY_REQUEST, { groupBy: 'day' }],
    ['R28 by month (the default sends no group_by)', R28_BY_MONTH_REQUEST, {}],
  ];
  it.each(cases)('%s', (_name, request, part) => {
    const state: ReportFilterState = { ...EMPTY_FILTERS, ...part };
    expect(filtersToWire(state, entryOf(request.id).filters)).toEqual(wireOf(request));
  });

  it('sends the one-time account the backend refuses for R9 only if the address put it there', () => {
    // The R9 screen has no account box, only a person box, so a left-over account cannot be typed.
    // An address such as /reports/R9?party=20 does carry one; the backend then refuses it.
    const state = stateFromSearch({ party: 20 });
    expect(filtersToWire(state, entryOf('R9').filters)).toEqual(
      wireOf(R9_NOT_A_ONE_TIME_ACCOUNT_REQUEST),
    );
  });

  it('never sends the two newer keys to a report that does not take them', () => {
    const state: ReportFilterState = { ...EMPTY_FILTERS, paid: 'due', groupBy: 'day' };
    const wire = filtersToWire(state, entryOf('R11').filters);
    expect(wire).not.toHaveProperty('status');
    expect(wire).not.toHaveProperty('group_by');
  });
});

describe('a one-time person in the party filter (PRD FR-13.5, FR-13.9)', () => {
  const hit = PEOPLE_SHAHID[0];
  if (hit === undefined) throw new Error('no person in the fixture');

  it('is the person key with his account, as the contract says', () => {
    const person = personFromHit(hit, null);
    expect(person).toEqual({
      accountId: 9,
      personKey: 'شاہد|03000000000',
      name_ur: 'شاہد',
      name_en: 'Shahid',
      phone: '03000000000',
      last_date: '2026-10-10',
    });
    const state: ReportFilterState = { ...EMPTY_FILTERS, person };
    expect(filtersToWire(state, entryOf('R9').filters).party).toEqual({
      account_id: 9,
      person_key: 'شاہد|03000000000',
    });
  });

  it('is left out when he has no lines on the account the report is on', () => {
    expect(personFromHit(hit, 9)).not.toBeNull();
    expect(personFromHit(hit, 20)).toBeNull();
  });

  it('is not sent to a report that takes no party', () => {
    const state: ReportFilterState = { ...EMPTY_FILTERS, person: personFromHit(hit, null) };
    expect(filtersToWire(state, entryOf('R30').filters).party).toBeNull();
  });

  it('counts as a filled box, so “clear the filters” is offered', () => {
    const state: ReportFilterState = { ...EMPTY_FILTERS, person: personFromHit(hit, null) };
    expect(isUnfiltered(state, entryOf('R9').filters)).toBe(false);
    expect(isUnfiltered({ ...EMPTY_FILTERS, paid: 'paid' }, entryOf('R9').filters)).toBe(false);
    expect(isUnfiltered({ ...EMPTY_FILTERS, groupBy: 'day' }, entryOf('R28').filters)).toBe(true);
  });

  it('comes from the address whole or not at all', () => {
    const person = personFromHit(hit, null);
    expect(person).not.toBeNull();
    const search = parseReportSearch({ person });
    expect(search.person).toEqual(person);
    expect(stateFromSearch(search).person).toEqual(person);
    // A half-read person would ask for the wrong lines, so any missing piece drops him.
    expect(parseReportSearch({ person: { ...person, personKey: '' } }).person).toBeUndefined();
    expect(parseReportSearch({ person: { accountId: 9 } }).person).toBeUndefined();
    expect(parseReportSearch({ person: 'x' }).person).toBeUndefined();
  });
});

describe('which accounts the party box of a report takes', () => {
  it.each([
    ['R2', 'buyer'],
    ['R3', 'buyer'],
    ['R4', 'buyer'],
    ['R7', 'buyer'],
    ['R20', 'buyer'],
    ['R21', 'buyer'],
    ['R22', 'buyer'],
    ['R24', 'seller'],
    ['R9', 'one_time'],
    ['R11', 'any'],
    ['R13', 'any'],
    ['R14', 'any'],
    ['R28', 'any'],
  ] as const)('%s takes %s', (id, box) => {
    expect(partyBoxOf(id)).toBe(box);
  });
});

describe('the words and figures of a cell that is not money (round 4)', () => {
  it('writes a share in hundredths of a percent with two decimals and the sign', () => {
    expect(percentText(7210)).toBe('72.10%');
    expect(percentText(2790)).toBe('27.90%');
    expect(percentText(10000)).toBe('100.00%');
    expect(percentText(5)).toBe('0.05%');
    expect(percentText(0)).toBe('0.00%');
  });

  it('names a month from its number and puts the year after it', () => {
    expect(monthMessageId(1)).toBe('report-month-1');
    expect(monthMessageId(12)).toBe('report-month-12');
    expect(monthText((id) => `<${id}>`, 2026, 10)).toBe('<report-month-10> 2026');
  });

  it('has words for a name left empty on purpose, in the reports that leave one', () => {
    expect(emptyNameMessageId('R21', 'group')).toBe('report-group-no-city');
    expect(emptyNameMessageId('R22', 'group')).toBe('report-group-no-collector');
    expect(emptyNameMessageId('R8', 'normal')).toBe('report-adda-none');
    // A buyer under a heading has an empty first cell by design, and says nothing.
    expect(emptyNameMessageId('R21', 'normal')).toBeNull();
    expect(emptyNameMessageId('R20', 'normal')).toBeNull();
  });
});
