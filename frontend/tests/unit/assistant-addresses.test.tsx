// The addresses a button under an answer opens (plan 49, PRD FR-7.74): the report address learns the
// rest of the report filters and the analytics address learns its filters, both additive (with no
// extra parameter they read exactly as before), the filters of an answer's source become the address of
// its screen and the filters of its print, and an analytics view opened with filters in its address
// asks the backend for exactly those filters.

import { act, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
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
import {
  analyticsFromRemembered,
  rememberedAnalytics,
} from '../../src/features/analytics/remember';
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
import { message, visible } from '../helpers/render';

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

  it('an item group on an analytics view (A, B, D, G): the address carries it, the print sends it, and none sends nothing', () => {
    const source = analyticsSource({ item_group: 'vegetable', period: 'this_month' });
    expect(analyticsSearchFromFilters(source)).toEqual({
      period: 'this_month',
      item_group: 'vegetable',
    });
    expect(analyticsFiltersOf(source)).toMatchObject({ item_group: 'vegetable' });
    expect(analyticsSearchFromFilters(analyticsSource())).not.toHaveProperty('item_group');
    expect(analyticsFiltersOf(analyticsSource())).not.toHaveProperty('item_group');
    expect(parseAnalyticsSearch({ item_group: 'fruit' }).item_group).toBe('fruit');
    expect(parseAnalyticsSearch({ item_group: 'meat' })).not.toHaveProperty('item_group');
  });

  it('an item group, fruit or vegetables (R2, R3, R4, R7, R10, R27): the address carries it, the print sends it, and none sends nothing', () => {
    const source = reportSource({ item_group: 'fruit', from: '2026-10-01', to: '2026-10-31' });
    expect(reportSearchFromFilters(source)).toEqual({
      from: '01-10-2026',
      to: '31-10-2026',
      item_group: 'fruit',
    });
    expect(reportFiltersOf(source)).toMatchObject({ item_group: 'fruit' });
    expect(reportSearchFromFilters(reportSource())).not.toHaveProperty('item_group');
    expect(reportFiltersOf(reportSource())).not.toHaveProperty('item_group');
    // The address, read again, chooses the group.
    expect(parseReportSearch({ item_group: 'vegetable' }).item_group).toBe('vegetable');
    expect(parseReportSearch({ item_group: 'meat' })).not.toHaveProperty('item_group');
  });

  it('the stock by item (R10): the address carries the layout, the print sends it, and by lot or none sends nothing', () => {
    const source = reportSource({ stock_view: 'item', from: '2026-10-01', to: '2026-10-31' });
    expect(reportSearchFromFilters(source)).toEqual({
      from: '01-10-2026',
      to: '31-10-2026',
      stock_view: 'item',
    });
    expect(reportFiltersOf(source)).toMatchObject({ stock_view: 'item' });
    expect(reportSearchFromFilters(reportSource())).not.toHaveProperty('stock_view');
    expect(reportFiltersOf(reportSource())).not.toHaveProperty('stock_view');
    // The address, read again, chooses the layout; an unknown word chooses nothing.
    expect(parseReportSearch({ stock_view: 'item' }).stock_view).toBe('item');
    expect(parseReportSearch({ stock_view: 'pile' })).not.toHaveProperty('stock_view');
  });

  it('a quality on a report (R2, R3, R4): the address carries it, the print sends it, and none sends nothing', () => {
    const source = reportSource({ grade: 'second', from: '2026-10-01', to: '2026-10-31' });
    expect(reportSearchFromFilters(source)).toEqual({
      from: '01-10-2026',
      to: '31-10-2026',
      grade: 'second',
    });
    expect(reportFiltersOf(source)).toMatchObject({ grade: 'second' });
    expect(reportSearchFromFilters(reportSource())).not.toHaveProperty('grade');
    expect(reportFiltersOf(reportSource())).not.toHaveProperty('grade');
  });

  it('a quality on an analytics view (A): the address carries it, the print sends it, and none sends nothing', () => {
    const source = analyticsSource({ grade: 'first', period: 'this_month' });
    expect(analyticsSearchFromFilters(source)).toEqual({ period: 'this_month', grade: 'first' });
    expect(analyticsFiltersOf(source)).toMatchObject({ grade: 'first' });
    expect(analyticsSearchFromFilters(analyticsSource())).not.toHaveProperty('grade');
    expect(analyticsFiltersOf(analyticsSource())).not.toHaveProperty('grade');
    // The address, read again, starts the filters; "all", an unknown word or none is no filter.
    expect(parseAnalyticsSearch({ grade: 'third' }).grade).toBe('third');
    expect(parseAnalyticsSearch({ grade: 'all' })).not.toHaveProperty('grade');
    expect(parseAnalyticsSearch({ grade: 'fourth' })).not.toHaveProperty('grade');
    expect(filtersFromSearch(parseAnalyticsSearch({ grade: 'third' })).grade).toBe('third');
    expect(filtersFromSearch({}).grade).toBe('all');
  });

  it('a collector, a staff member (R22, R23, R35): the address carries him, the print sends his id, and none sends nothing', () => {
    const rasheed = { id: 12, code: null, name_ur: 'رشید', name_en: 'Rasheed', tag: null };
    const source = reportSource({ collector: rasheed, as_of: '2026-10-10' });
    expect(reportSearchFromFilters(source)).toEqual({
      as_of: '10-10-2026',
      collector: { id: 12, name_ur: 'رشید', name_en: 'Rasheed', tag: null, code: undefined },
    });
    expect(reportFiltersOf(source)).toMatchObject({ collector_id: 12, as_of: '10-10-2026' });
    // The backend leaves the key out when there is none, and so does the address and the print.
    expect(reportSearchFromFilters(reportSource())).not.toHaveProperty('collector');
    expect(reportFiltersOf(reportSource())).not.toHaveProperty('collector_id');
    // The address, read again, fills the box.
    expect(parseReportSearch({ collector: { id: 12, name_en: 'Rasheed' } }).collector?.id).toBe(12);
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

  it('a quality in the address is asked for on view A, and not on a view that does not take it', async () => {
    fakeAnalytics();
    await renderApp('/analytics/A?grade=first&period=this_month');
    await waitFor(() => {
      expect(analyticsApi.analyticsRun).toHaveBeenCalled();
    });
    const asked = vi.mocked(analyticsApi.analyticsRun).mock.calls.at(-1)?.[0];
    expect(asked?.view).toBe('A');
    expect(asked?.filters).toMatchObject({ grade: 'first', period: 'this_month' });
  });

  it('the quality box shows the quality the address names, and is remembered for the next opening', async () => {
    fakeAnalytics();
    await renderApp('/analytics/A?grade=second');
    await waitFor(() => {
      expect(analyticsApi.analyticsRun).toHaveBeenCalled();
    });
    const state = useAnalyticsStore.getState().filters;
    expect(state.grade).toBe('second');
    // The box is in view, with the quality in it (never behind "More filters").
    const box = await screen.findByRole('combobox', {
      name: (name) => visible(name).includes(message('en', 'common-grade')),
    });
    expect(visible(box.textContent)).toContain(message('en', 'glossary-darja-doyam'));
    // It is kept like the other boxes and read back the same.
    expect(rememberedAnalytics(state)).toMatchObject({ grade: 'second' });
    expect(analyticsFromRemembered({ grade: 'second' })?.grade).toBe('second');
    expect(analyticsFromRemembered({ grade: 'fourth' })?.grade).toBe('all');
    expect(rememberedAnalytics({ ...state, grade: 'all' }) ?? {}).not.toHaveProperty('grade');
  });

  it('the boxes sit in the shared filter grid, so no box is squeezed narrower than a filter (NFR-USE-16)', async () => {
    fakeAnalytics();
    await renderApp('/analytics/A');
    const box = await screen.findByRole('combobox', {
      name: (name) => visible(name).includes(message('en', 'common-grade')),
    });
    expect(box.closest('.filter-grid')).not.toBeNull();
    expect(box.closest('[role="group"]')).toBe(document.querySelector('.filter-grid'));
  });

  it('choosing a quality runs view A again with it, and "all" takes it away', async () => {
    fakeAnalytics();
    const user = userEvent.setup();
    await renderApp('/analytics/A?period=this_month');
    const box = await screen.findByRole('combobox', {
      name: (name) => visible(name).includes(message('en', 'common-grade')),
    });
    await user.click(box);
    await user.click(
      await screen.findByRole('option', { name: message('en', 'glossary-darja-awwal') }),
    );
    await waitFor(() => {
      expect(vi.mocked(analyticsApi.analyticsRun).mock.calls.at(-1)?.[0].filters).toMatchObject({
        grade: 'first',
      });
    });
    await user.click(box);
    await user.click(
      await screen.findByRole('option', { name: message('en', 'common-grade-all') }),
    );
    await waitFor(() => {
      expect(
        vi.mocked(analyticsApi.analyticsRun).mock.calls.at(-1)?.[0].filters,
      ).not.toHaveProperty('grade');
    });
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
