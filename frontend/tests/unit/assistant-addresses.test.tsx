// The addresses a button under an answer opens (plan 49, PRD FR-7.74): the report address learns the
// rest of the report filters and the analytics address learns its filters, both additive (with no
// extra parameter they read exactly as before), the filters of an answer's source become the address of
// its screen and the filters of its print, and an analytics view opened with filters in its address
// asks the backend for exactly those filters.

import { act, screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { SourceAnalyticsFilters } from '../../src/bindings/SourceAnalyticsFilters';
import type { SourceReportFilters } from '../../src/bindings/SourceReportFilters';
import {
  analyticsFiltersOf,
  analyticsSearchFromFilters,
  reportFiltersOf,
  reportSearchFromFilters,
} from '../../src/features/assistant/addresses';
import * as analyticsApi from '../../src/features/analytics/api';
import {
  filtersFromSearch,
  hasAnalyticsSearch,
  parseAnalyticsSearch,
} from '../../src/features/analytics/search';
import { useAnalyticsStore } from '../../src/features/analytics/store';
import { parseReportSearch, stateFromSearch } from '../../src/features/reports/search';
import { EMPTY_FILTERS } from '../../src/features/reports/filters';
import * as accountApi from '../../src/shared/accounts/api';
import * as searchApi from '../../src/shared/search/api';
import {
  healthyBackend,
  leaveAppWindow,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { fakeAnalytics } from '../helpers/analytics';
import { REPLY_ANSWERED } from '../helpers/assistant-real';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/analytics/api');
vi.mock('../../src/features/reports/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/search/api');

function reportSource(filters: Partial<SourceReportFilters> = {}): SourceReportFilters {
  return {
    shortcut: null,
    from: '',
    to: '',
    as_of: '',
    party: null,
    seller: null,
    zamindar: null,
    item: null,
    city: null,
    adda: null,
    detail: false,
    group_by: null,
    status: null,
    partner: null,
    voucher_type: null,
    trip_view: null,
    garden: null,
    garden_view: null,
    bikri: null,
    guests_only: false,
    ...filters,
  };
}

function analyticsSource(filters: Partial<SourceAnalyticsFilters> = {}): SourceAnalyticsFilters {
  return {
    period: null,
    from: '',
    to: '',
    group_by: null,
    item: null,
    seller: null,
    buyer: null,
    city: null,
    adda: null,
    side: 'both',
    compare: 'off',
    partner: null,
    trip: null,
    garden: null,
    goods: 'all',
    trip_view: null,
    ...filters,
  };
}

const BANANA = { id: 5, code: null, name_ur: 'کیلا', name_en: 'Banana', tag: null };
const RAHIM = { id: 3, code: '0001', name_ur: 'رحیم', name_en: 'Rahim', tag: null };

describe('the report address learns the rest of the filters, and only adds', () => {
  it('with no extra parameter it reads exactly as before', () => {
    expect(parseReportSearch({})).toEqual({});
    expect(parseReportSearch({ party: 20, from: '01-10-2026' })).toEqual({
      party: 20,
      from: '01-10-2026',
    });
    expect(stateFromSearch({})).toEqual({ ...EMPTY_FILTERS });
  });

  it('reads a record whole or by its id, a layout, a kind, a garden, a بکری with its number', () => {
    const found = parseReportSearch({
      seller: 3,
      zamindar: { id: 4, name_ur: 'ز', name_en: 'Z' },
      item: '5',
      city: 7,
      adda: 8,
      voucher_type: 'receipt',
      trip_view: 'item',
      garden: 2,
      garden_view: 'partner',
      bikri: 11,
      bikri_number: 4,
      group_by: 'day',
      status: 'due',
      detail: true,
    });
    expect(found.seller).toEqual({ id: 3, name_ur: '', name_en: '', code: undefined });
    expect(found.zamindar).toEqual({ id: 4, name_ur: 'ز', name_en: 'Z' });
    expect(found.item?.id).toBe(5);
    expect(found).toMatchObject({
      voucher_type: 'receipt',
      trip_view: 'item',
      garden: 2,
      garden_view: 'partner',
      bikri: { id: 11, number: 4 },
      group_by: 'day',
      status: 'due',
      detail: true,
    });
    const state = stateFromSearch(found);
    expect(state).toMatchObject({
      voucherType: 'receipt',
      tripView: 'item',
      garden: 2,
      gardenView: 'partner',
      bikri: { id: 11, number: 4 },
      groupBy: 'day',
      paid: 'due',
      detail: true,
    });
    expect(state.seller?.id).toBe(3);
  });

  it('leaves out what it cannot read, and a بکری without its number', () => {
    expect(
      parseReportSearch({
        seller: -1,
        city: 'x',
        voucher_type: 'cheque',
        trip_view: 'sideways',
        garden: 0,
        bikri: 11,
        group_by: 'year',
        status: 'maybe',
      }),
    ).toEqual({});
  });

  it('`all` is the same as leaving the kind out', () => {
    expect(parseReportSearch({ voucher_type: 'all', status: 'all' })).toEqual({});
  });
});

describe('the analytics address', () => {
  it('reads its filters', () => {
    const found = parseAnalyticsSearch({
      period: 'this_month',
      from: '01-10-2026',
      to: '31-10-2026',
      group_by: 'day',
      item: 5,
      seller: { id: 3, name_ur: 'رحیم', name_en: 'Rahim' },
      buyer: '20',
      compare: 'previous',
      side: 'seller',
      goods: 'trip',
      partner: 1,
      trip: 2,
      garden: 3,
      city: 4,
      adda: 6,
      trip_view: 'purchase',
    });
    expect(found).toMatchObject({
      period: 'this_month',
      from: '01-10-2026',
      to: '31-10-2026',
      group_by: 'day',
      compare: 'previous',
      side: 'seller',
      goods: 'trip',
      partner: 1,
      trip: 2,
      garden: 3,
      trip_view: 'purchase',
    });
    expect(found.item?.id).toBe(5);
    expect(found.seller?.name_en).toBe('Rahim');
    expect(found.buyer?.id).toBe(20);
    expect(hasAnalyticsSearch(found)).toBe(true);
    const state = filtersFromSearch(found);
    expect(state).toMatchObject({
      period: 'this_month',
      groupBy: 'day',
      compare: 'previous',
      side: 'seller',
      goods: 'trip',
      partner: 1,
      trip: 2,
      garden: 3,
    });
  });

  it('with no parameter, or only words it does not know, it holds nothing', () => {
    expect(hasAnalyticsSearch(parseAnalyticsSearch({}))).toBe(false);
    expect(
      hasAnalyticsSearch(
        parseAnalyticsSearch({ period: 'custom', compare: 'always', item: -2, foo: 1 }),
      ),
    ).toBe(false);
  });
});

describe('the filters of an answer’s source', () => {
  it('a report: the address carries the shortcut or the dates, the party and the records with their names', () => {
    const search = reportSearchFromFilters(
      reportSource({
        shortcut: 'this_month',
        party: { record: RAHIM, person_key: null },
        item: BANANA,
        bikri: { id: 11, number: 4 },
        group_by: 'day',
      }),
    );
    expect(search).toEqual({
      period: 'this_month',
      party: 3,
      item: { id: 5, name_ur: 'کیلا', name_en: 'Banana', tag: null, code: undefined },
      bikri: { id: 11, number: 4 },
      group_by: 'day',
    });
    // The two dates are carried when there is no shortcut, as the boxes show them: DD-MM-YYYY.
    expect(reportSearchFromFilters(reportSource({ from: '2026-10-01', to: '2026-10-10' }))).toEqual(
      {
        from: '01-10-2026',
        to: '10-10-2026',
      },
    );
    // The as-of day too; a date that is not an ISO day is carried as it came.
    expect(reportSearchFromFilters(reportSource({ as_of: '2026-10-10' }))).toEqual({
      as_of: '10-10-2026',
    });
    expect(reportSearchFromFilters(reportSource({ from: '10-9' }))).toEqual({ from: '10-9' });
  });

  it('a print of a source asks for its dates the way the boxes show them', () => {
    const asked = reportFiltersOf(
      reportSource({ from: '2026-10-01', to: '2026-10-10', as_of: '2026-10-10' }),
    );
    expect([asked.from, asked.to, asked.as_of]).toEqual(['01-10-2026', '10-10-2026', '10-10-2026']);
    expect(reportFiltersOf(reportSource({})).from).toBe('');
  });

  it('a one-time person is one person of his account', () => {
    const search = reportSearchFromFilters(
      reportSource({ party: { record: RAHIM, person_key: 'shahid|0300' } }),
    );
    expect(search.person).toMatchObject({ accountId: 3, personKey: 'shahid|0300' });
    expect(search.party).toBeUndefined();
  });

  it('every address the answer fixtures carry reads back as the same filters it came from', () => {
    for (const source of REPLY_ANSWERED.answer?.sources ?? []) {
      if (source.kind !== 'report') continue;
      const search = reportSearchFromFilters(source.filters);
      const parsed = parseReportSearch(
        JSON.parse(JSON.stringify(search)) as Record<string, unknown>,
      );
      expect(parsed.party).toBe(source.filters.party?.record.id);
    }
  });

  it('a report print asks for the ids of the same records, and leaves out what the source did not set', () => {
    const wire = reportFiltersOf(
      reportSource({ party: { record: RAHIM, person_key: null }, item: BANANA, detail: true }),
    );
    expect(wire).toMatchObject({
      party: { account_id: 3, person_key: null },
      item_id: 5,
      seller_id: null,
      detail: true,
    });
    expect('group_by' in wire).toBe(false);
    expect('status' in wire).toBe(false);
    expect('guests_only' in wire).toBe(false);
  });

  it('an analytics view: the address and the print carry the same filters', () => {
    const source = analyticsSource({
      period: 'this_month',
      item: BANANA,
      seller: RAHIM,
      compare: 'previous',
      goods: 'trip',
    });
    const search = analyticsSearchFromFilters(source);
    expect(search).toMatchObject({ period: 'this_month', compare: 'previous', goods: 'trip' });
    expect(search.item?.name_en).toBe('Banana');
    expect(search.seller?.id).toBe(3);
    // Nothing is carried for a filter that is at its starting value.
    expect(analyticsSearchFromFilters(analyticsSource())).toEqual({});
    expect(analyticsFiltersOf(source)).toMatchObject({
      period: 'this_month',
      item_id: 5,
      seller_id: 3,
      buyer: null,
      compare: 'previous',
      goods: 'trip',
    });
  });
});

describe('an analytics view opened with filters in its address', () => {
  beforeEach(() => {
    vi.resetAllMocks();
    healthyBackend();
    resetClientState('en');
    signedIn(sessionUser({ language: 'en' }));
    vi.mocked(accountApi.searchAccounts).mockResolvedValue([]);
    vi.mocked(accountApi.searchAccountsRecent).mockResolvedValue([]);
    vi.mocked(searchApi.searchMasters).mockResolvedValue([]);
    vi.mocked(searchApi.searchRecent).mockResolvedValue([]);
    act(() => {
      useAnalyticsStore.getState().reset();
    });
    window.localStorage.clear();
  });

  afterEach(() => {
    leaveAppWindow();
  });

  it('asks the backend for exactly those filters', async () => {
    fakeAnalytics();
    await renderApp('/analytics/A?item=5&seller=3&period=this_month');
    await waitFor(() => {
      expect(analyticsApi.analyticsRun).toHaveBeenCalled();
    });
    const asked = vi.mocked(analyticsApi.analyticsRun).mock.calls.at(-1)?.[0];
    expect(asked?.view).toBe('A');
    expect(asked?.filters).toMatchObject({ period: 'this_month', item_id: 5, seller_id: 3 });
    // The boxes hold them too, so the Owner can change them.
    expect(useAnalyticsStore.getState().filters).toMatchObject({ period: 'this_month' });
    expect(useAnalyticsStore.getState().filters.item?.id).toBe(5);
    expect(screen.getByRole('tablist')).toBeInTheDocument();
  });

  it('with no filter in the address the view opens as it always did', async () => {
    fakeAnalytics();
    await renderApp('/analytics/A');
    await waitFor(() => {
      expect(analyticsApi.analyticsRun).toHaveBeenCalled();
    });
    const asked = vi.mocked(analyticsApi.analyticsRun).mock.calls.at(-1)?.[0];
    expect(asked?.filters).toMatchObject({ period: null, item_id: null, seller_id: null });
  });
});
