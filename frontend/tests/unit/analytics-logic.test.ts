// The logic of the analytics screens without a screen (plan 21, PRD 18.1, 18.2, FR-18.3): what the
// filter boxes become on the wire (exactly the requests the backend wrote), how a saved view comes
// back into the boxes, which columns fit each window, the window of the columns of views D and E, and
// what each chart draws. The figures come from the backend's own answers; nothing is added up here.

import { describe, expect, it } from 'vitest';
import type { AnalyticsRunInput } from '../../src/bindings/AnalyticsRunInput';
import type { AnalyticsView } from '../../src/bindings/AnalyticsView';
import { chartFor, MAX_CHART_ROWS } from '../../src/features/analytics/chart';
import {
  analyticsColumns,
  headingText,
  planColumns,
  signedKeysOf,
} from '../../src/features/analytics/columns';
import {
  EMPTY_ANALYTICS_FILTERS,
  filtersToWire,
  isUnfiltered,
  periodMessageId,
  stateFromWire,
  type AnalyticsFilterState,
} from '../../src/features/analytics/filters';
import { columnWidth, fitColumns, STEP_ROOM, windowOf } from '../../src/features/analytics/fit';
import { formatPaisa } from '../../src/shared/format';
import type { ListContext } from '../../src/shared/grid';
import { buildBarOption } from '../../src/shared/charts';
import {
  ANALYTICS_CATALOG,
  ANALYTICS_RUN_A,
  ANALYTICS_RUN_A_COMPARE,
  ANALYTICS_RUN_A_COMPARE_REQUEST,
  ANALYTICS_RUN_A_GROUP_BY_REFUSED_REQUEST,
  ANALYTICS_RUN_A_REQUEST,
  ANALYTICS_RUN_A_SELLER_SIDE_REQUEST,
  ANALYTICS_RUN_A_UNKNOWN_ITEM_REQUEST,
  ANALYTICS_RUN_B,
  ANALYTICS_RUN_C,
  ANALYTICS_RUN_D_DAY,
  ANALYTICS_RUN_D_DAY_REQUEST,
  ANALYTICS_RUN_D_MONTH,
  ANALYTICS_RUN_D_MONTH_REQUEST,
  ANALYTICS_RUN_E,
  ANALYTICS_RUN_H,
  VIEWS_GET_TWO,
} from '../helpers/analytics-real';
import {
  ANALYTICS_RUN_A_GOODS_CONSIGNED_REQUEST,
  ANALYTICS_RUN_A_GOODS_TRIP_REQUEST,
  ANALYTICS_RUN_F,
  ANALYTICS_RUN_F_COMPARE_REFUSED_REQUEST,
  ANALYTICS_RUN_F_REQUEST,
  ANALYTICS_RUN_F_TRIP,
  ANALYTICS_RUN_G_DAY,
  ANALYTICS_RUN_G_DAY_REQUEST,
  ANALYTICS_RUN_G_MONTH_REQUEST,
  ANALYTICS_RUN_G_TRIP_DAY,
  ANALYTICS_RUN_G_TRIP_WRITTEN_OFF,
  ANALYTICS_RUN_J_TRIP,
  ANALYTICS_RUN_J_TRIP_BY_PURCHASE,
  ANALYTICS_RUN_J_TRIP_BY_PURCHASE_REQUEST,
  ANALYTICS_RUN_J_TRIP_PARTNER_REQUEST,
  ANALYTICS_RUN_J_TRIP_REQUEST,
  ANALYTICS_RUN_J_TRIP_WRITTEN_OFF,
} from '../helpers/analytics-trips-real';

const takesOf = (view: AnalyticsView) => {
  const entry = ANALYTICS_CATALOG.find((candidate) => candidate.id === view);
  if (entry === undefined) throw new Error(`no ${view}`);
  return entry.filters;
};

/** The request of the backend's own test, as a plain filters object. */
const filtersOf = (request: AnalyticsRunInput) => request.filters;

describe('the boxes become the request the backend wrote', () => {
  const cases: [string, AnalyticsView, Partial<AnalyticsFilterState>, AnalyticsRunInput][] = [
    ['nothing filled', 'A', {}, ANALYTICS_RUN_A_REQUEST],
    [
      'a comparison with both dates',
      'A',
      { from: '10-10-2026', to: '10-10-2026', compare: 'previous' },
      ANALYTICS_RUN_A_COMPARE_REQUEST,
    ],
    ['the seller side only', 'A', { side: 'seller' }, ANALYTICS_RUN_A_SELLER_SIDE_REQUEST],
    [
      'an item that does not exist',
      'A',
      { item: { id: 9999, name_ur: 'x', name_en: 'x', code: undefined } },
      ANALYTICS_RUN_A_UNKNOWN_ITEM_REQUEST,
    ],
    ['view D by day', 'D', { groupBy: 'day' }, ANALYTICS_RUN_D_DAY_REQUEST],
    ['view D by month (the default sends no grouping)', 'D', {}, ANALYTICS_RUN_D_MONTH_REQUEST],
  ];
  it.each(cases)('%s', (_name, view, part, request) => {
    const state: AnalyticsFilterState = { ...EMPTY_ANALYTICS_FILTERS, ...part };
    expect(filtersToWire(state, takesOf(view))).toEqual(filtersOf(request));
  });

  it('never sends a grouping to a view that has none, which the backend would refuse', () => {
    const state: AnalyticsFilterState = { ...EMPTY_ANALYTICS_FILTERS, groupBy: 'day' };
    expect(filtersToWire(state, takesOf('A')).group_by).toBeNull();
    // The backend’s refusal of exactly that request is in the fixtures; the screen cannot make it.
    expect(filtersOf(ANALYTICS_RUN_A_GROUP_BY_REFUSED_REQUEST).group_by).toBe('day');
  });

  it('a shortcut wins over the dates, and the dates are not sent with it', () => {
    const state: AnalyticsFilterState = {
      ...EMPTY_ANALYTICS_FILTERS,
      period: 'this_year',
      from: '01-01-2026',
      to: '31-12-2026',
    };
    const wire = filtersToWire(state, takesOf('A'));
    expect(wire.period).toBe('this_year');
    expect(wire.from).toBe('');
    expect(wire.to).toBe('');
  });

  it('sends a buyer, and a person inside a one-time buyer account with his key', () => {
    const buyer = { id: 9, name_ur: 'پختہ', name_en: 'Pukhta', code: '03049999' };
    const plain = filtersToWire({ ...EMPTY_ANALYTICS_FILTERS, buyer }, takesOf('C'));
    expect(plain.buyer).toEqual({ account_id: 9, person_key: null });
    const person = {
      accountId: 9,
      personKey: 'شاہد|03000000000',
      name_ur: 'شاہد',
      name_en: 'Shahid',
      phone: '03000000000',
      last_date: '2026-10-10',
    };
    const narrowed = filtersToWire({ ...EMPTY_ANALYTICS_FILTERS, buyer, person }, takesOf('C'));
    expect(narrowed.buyer).toEqual({ account_id: 9, person_key: 'شاہد|03000000000' });
  });

  it('knows when nothing is filled, so “clear the filters” is offered only when it helps', () => {
    expect(isUnfiltered(EMPTY_ANALYTICS_FILTERS, takesOf('A'))).toBe(true);
    expect(isUnfiltered({ ...EMPTY_ANALYTICS_FILTERS, side: 'buyer' }, takesOf('A'))).toBe(false);
    expect(isUnfiltered({ ...EMPTY_ANALYTICS_FILTERS, compare: 'last_year' }, takesOf('A'))).toBe(
      false,
    );
    expect(isUnfiltered({ ...EMPTY_ANALYTICS_FILTERS, groupBy: 'day' }, takesOf('D'))).toBe(true);
  });

  it('has a message for every period choice', () => {
    expect(periodMessageId('this_year')).toBe('analytics-period-this-year');
    expect(periodMessageId('this_month')).toBe('report-period-this-month');
    expect(periodMessageId('custom')).toBe('report-period-custom');
  });
});

describe('a saved view comes back into the boxes', () => {
  const first = VIEWS_GET_TWO[0];
  if (first === undefined) throw new Error('no saved view in the fixture');

  it('keeps the dates as typed and the item by its id, and sends the same filters again', () => {
    const state = stateFromWire(first.filters);
    expect(state.from).toBe('01-01-2026');
    expect(state.to).toBe('31-12-2026');
    expect(state.item?.id).toBe(1);
    // The names of a record come with the first answer; until then they are empty.
    expect(state.item?.name_ur).toBe('');
    expect(filtersToWire(state, takesOf(first.view))).toEqual(first.filters);
  });

  it('opens the second saved view with nothing filled', () => {
    const second = VIEWS_GET_TWO[1];
    expect(second?.view).toBe('H');
    expect(stateFromWire(second?.filters ?? first.filters)).toEqual(EMPTY_ANALYTICS_FILTERS);
  });
});

describe('which columns fit', () => {
  const room = (step: keyof typeof STEP_ROOM) => STEP_ROOM[step];
  const widths = [
    { key: 'a', width: 160 },
    { key: 'b', width: 120 },
    { key: 'c', width: 120 },
    { key: 'd', width: 100 },
  ];

  it('takes the columns in order of use and leaves out one that does not fit', () => {
    expect([...fitColumns(widths, ['a', 'b', 'c', 'd'], 380)]).toEqual(['a', 'b', 'd']);
    expect([...fitColumns(widths, ['a', 'b', 'c', 'd'], 400)]).toEqual(['a', 'b', 'c']);
    // d is narrower than c, so it fits where c did not.
    expect([...fitColumns(widths, ['a', 'c', 'b', 'd'], 380)].sort()).toEqual(['a', 'c', 'd']);
  });

  it('keeps at least one column, and names no column twice', () => {
    expect([...fitColumns(widths, ['b'], 10)]).toEqual(['b']);
    expect(fitColumns(widths, [], 10_000).size).toBe(4);
  });

  it('shows the figures of E2E-11 view A at the compact step: item, total, share, quantity, gross', () => {
    const plan = planColumns('A', ANALYTICS_RUN_A.columns, 'compact', 0);
    const keys = ANALYTICS_RUN_A.columns.filter((_, index) => plan.visible.has(index));
    expect(keys.map((column) => column.key)).toEqual([
      'item',
      'qty',
      'gross',
      'seller_comm',
      'buyer_comm',
      'total_comm',
      'share',
    ]);
    expect(plan.window).toBeNull();
    // The widest step has all nine.
    expect(planColumns('A', ANALYTICS_RUN_A.columns, 'wide', 0).visible.size).toBe(9);
  });

  const shownKeys = (
    page: typeof ANALYTICS_RUN_A,
    step: keyof typeof STEP_ROOM,
    other: boolean,
  ): string[] => {
    const plan = planColumns(page.view, page.columns, step, 0, other);
    return page.columns.filter((_, index) => plan.visible.has(index)).map((column) => column.key);
  };

  it('says how many columns fit, so a column that waits is never a surprise (PRD 18.2, E2E-11)', () => {
    // View A has nine columns: the average rate (588 and 1,200 of E2E-11) waits at the normal step.
    expect(planColumns('A', ANALYTICS_RUN_A.columns, 'normal', 0).trimmed).toEqual({
      shown: 8,
      total: 9,
    });
    expect(planColumns('A', ANALYTICS_RUN_A.columns, 'compact', 0).trimmed).toEqual({
      shown: 7,
      total: 9,
    });
    // Nothing waits at the wide step, so there is nothing to offer.
    expect(planColumns('A', ANALYTICS_RUN_A.columns, 'wide', 0).trimmed).toBeNull();
  });

  it('brings the waiting columns in when asked, and the least needed ones step aside', () => {
    // Normal step: the average rate comes in, the laga column gives up its place.
    const normal = shownKeys(ANALYTICS_RUN_A, 'normal', true);
    expect(normal).toContain('avg_rate');
    expect(normal).not.toContain('laga');
    expect(normal).toContain('item');
    expect(normal).toContain('total_comm');
    // The same number of columns as before: nothing needs a sideways scroll.
    expect(normal).toHaveLength(shownKeys(ANALYTICS_RUN_A, 'normal', false).length);
    // Compact step: both waiting columns come in; the commissions of each side step aside.
    const compact = shownKeys(ANALYTICS_RUN_A, 'compact', true);
    expect(compact).toEqual(expect.arrayContaining(['item', 'laga', 'avg_rate', 'total_comm']));
    expect(compact).not.toContain('seller_comm');
    expect(planColumns('A', ANALYTICS_RUN_A.columns, 'compact', 0, true).trimmed).toEqual({
      shown: 7,
      total: 9,
    });
  });

  it('never fits more than the window holds, and never drops the name of the row', () => {
    for (const page of [ANALYTICS_RUN_A, ANALYTICS_RUN_B, ANALYTICS_RUN_C]) {
      for (const step of ['compact', 'normal', 'wide'] as const) {
        for (const other of [false, true]) {
          const plan = planColumns(page.view, page.columns, step, 0, other);
          const shown = page.columns.filter((_, index) => plan.visible.has(index));
          const used = shown.reduce((sum, column) => sum + columnWidth(column.kind, column.key), 0);
          // One column is always kept, so a window too small for even that may be over.
          expect(used <= STEP_ROOM[step] || shown.length === 1).toBe(true);
          expect(shown[0]?.key).toBe(page.columns[0]?.key);
        }
      }
    }
  });

  it('offers nothing in views D and E: their window buttons reach every column', () => {
    for (const page of [ANALYTICS_RUN_D_MONTH, ANALYTICS_RUN_D_DAY, ANALYTICS_RUN_E]) {
      expect(planColumns(page.view, page.columns, 'compact', 0, true).trimmed).toBeNull();
    }
  });

  it('with a comparison, the compared period and the change are kept ahead of the parts', () => {
    const plan = planColumns('A', ANALYTICS_RUN_A_COMPARE.columns, 'compact', 0);
    const keys = ANALYTICS_RUN_A_COMPARE.columns
      .filter((_, index) => plan.visible.has(index))
      .map((column) => column.key);
    expect(keys).toContain('compare');
    expect(keys).toContain('change');
    expect(keys).toContain('total_comm');
    expect(keys).not.toContain('avg_rate');
  });

  it('never needs the window for the small views of the worked day', () => {
    for (const page of [ANALYTICS_RUN_D_MONTH, ANALYTICS_RUN_D_DAY, ANALYTICS_RUN_E]) {
      expect(planColumns(page.view, page.columns, 'compact', 0).window).toBeNull();
    }
  });

  it('windows the columns of view D when they do not all fit, and keeps the window inside the list', () => {
    expect(windowOf(12, 280, 120, room('compact'), 0)).toEqual({ start: 0, count: 4, total: 12 });
    expect(windowOf(12, 280, 120, room('normal'), 4)).toEqual({ start: 4, count: 6, total: 12 });
    // Past the end is pulled back so the window is full; before the start is the start.
    expect(windowOf(12, 280, 120, room('normal'), 99).start).toBe(6);
    expect(windowOf(12, 280, 120, room('normal'), -3).start).toBe(0);
    // Everything fits: no window.
    expect(windowOf(3, 280, 120, room('wide'), 0)).toEqual({ start: 0, count: 3, total: 3 });
    // At least one column is always shown.
    expect(windowOf(5, 5000, 120, room('compact'), 0).count).toBe(1);
  });
});

/** The grid's context in English, with the message ID standing for its own words. */
const context: ListContext = {
  language: 'en',
  text: (id) => `<${id}>`,
  money: (amount) => formatPaisa(amount, { grouping: 'international' }),
};

describe('the headings of the columns of D and E', () => {
  it('names the month, the day, the year and the item', () => {
    expect(headingText({ kind: 'month', year: 2026, month: 10 }, context)).toBe(
      '<report-month-10> 2026',
    );
    expect(headingText({ kind: 'day', iso: '2026-10-10' }, context)).toBe('10-10-2026');
    expect(headingText({ kind: 'year', year: 2026 }, context)).toBe('2026');
    expect(headingText({ kind: 'item', names: { ur: 'کیلا', en: 'Banana' } }, context)).toBe(
      'Banana',
    );
    expect(headingText({ kind: 'other_items' }, context)).toBe('<analytics-col-other-items>');
  });

  it('makes a grid column for each shown column, with its heading written for the headed ones', () => {
    const plan = planColumns('D', ANALYTICS_RUN_D_MONTH.columns, 'normal', 0);
    const columns = analyticsColumns('D', ANALYTICS_RUN_D_MONTH.columns, plan);
    expect(columns.map((column) => column.field)).toEqual(['item', 'p_2026_10', 'total_comm']);
    expect(columns[1]?.headerText?.(context)).toBe('<report-month-10> 2026');
    expect(columns[0]?.headerText).toBeUndefined();
  });
});

describe('what each chart draws (from the rows of the backend, never worked out)', () => {
  it('A: the total commission of each item, with the written figure of each bar', () => {
    const plan = chartFor('A', ANALYTICS_RUN_A, context);
    expect(plan?.labelId).toBe('analytics-chart-a');
    expect(plan?.data.categories).toEqual(['Banana', 'Apple']);
    expect(plan?.data.series).toHaveLength(1);
    expect(plan?.data.series[0]?.labels).toEqual(['3,928', '1,520']);
    // The heights are paisa, for the drawing only.
    expect(plan?.data.series[0]?.values).toEqual([392_800, 152_000]);
    expect(plan?.data.stacked).toBe(false);
    expect([plan?.shown, plan?.total]).toEqual([2, 2]);
  });

  it('B: the total commission of each seller, by his name', () => {
    const plan = chartFor('B', ANALYTICS_RUN_B, context);
    expect(plan?.data.categories).toEqual(['Rahim', 'Karim']);
    expect(plan?.data.series[0]?.labels).toEqual(['3,928', '1,520']);
  });

  it('C: the purchase value of each buyer, one-time buyers included', () => {
    const plan = chartFor('C', ANALYTICS_RUN_C, context);
    expect(plan?.data.categories).toEqual(['Arshad Multani', 'Akram Toll Tax', 'Shahid']);
    expect(plan?.data.series[0]?.labels).toEqual(['17,400', '12,000', '12,000']);
  });

  it('D: one stack for each period with a part for each item', () => {
    const plan = chartFor('D', ANALYTICS_RUN_D_MONTH, context);
    expect(plan?.data.categories).toEqual(['<report-month-10> 2026']);
    expect(plan?.data.series.map((series) => series.name)).toEqual(['Banana', 'Apple']);
    expect(plan?.data.series.map((series) => series.labels[0])).toEqual(['3,928', '1,520']);
    expect(plan?.data.stacked).toBe(true);
  });

  it('E: one stack for each seller with a part for each item; an item he did not sell is a bar of nothing', () => {
    const plan = chartFor('E', ANALYTICS_RUN_E, context);
    expect(plan?.data.categories).toEqual(['Rahim', 'Karim']);
    expect(plan?.data.series.map((series) => series.name)).toEqual(['Banana', 'Apple']);
    expect(plan?.data.series[0]?.values).toEqual([392_800, 0]);
    expect(plan?.data.series[0]?.labels).toEqual(['3,928', '']);
  });

  it('H: the top items, from the first list only', () => {
    const plan = chartFor('H', ANALYTICS_RUN_H, context);
    expect(plan?.labelId).toBe('analytics-chart-h');
    expect(plan?.data.categories).toEqual(['Banana', 'Apple']);
    expect(plan?.data.series[0]?.labels).toEqual(['3,928', '1,520']);
  });

  it('shows the first rows only and says how many there are', () => {
    const row = ANALYTICS_RUN_A.rows[0];
    if (row === undefined) throw new Error('no row');
    const many = { ...ANALYTICS_RUN_A, rows: Array.from({ length: 25 }, () => row) };
    const plan = chartFor('A', many, context);
    expect(plan?.shown).toBe(MAX_CHART_ROWS);
    expect(plan?.total).toBe(25);
  });

  it('draws nothing when there are no rows', () => {
    expect(chartFor('A', { ...ANALYTICS_RUN_A, rows: [] }, context)).toBeNull();
    expect(chartFor('D', { ...ANALYTICS_RUN_D_MONTH, rows: [] }, context)).toBeNull();
  });
});

describe('the chart is placed for the direction of the app', () => {
  const data = chartFor('D', ANALYTICS_RUN_D_MONTH, context)?.data;
  const style = (direction: 'rtl' | 'ltr') => ({
    direction,
    fontFamily: 'Inter',
    colors: ['c1', 'c2'],
    ink: 'ink',
    inkMuted: 'muted',
    border: 'border',
    surface: 'surface',
  });
  const optionOf = (direction: 'rtl' | 'ltr') => {
    if (data === undefined) throw new Error('no data');
    return buildBarOption(data, style(direction)) as {
      animation: boolean;
      xAxis: { inverse: boolean };
      yAxis: { position: string };
      legend: { show: boolean };
    };
  };

  it('Urdu: the first category starts at the right', () => {
    const option = optionOf('rtl');
    expect(option.xAxis.inverse).toBe(true);
    expect(option.yAxis.position).toBe('right');
  });

  it('English: the first category starts at the left', () => {
    const option = optionOf('ltr');
    expect(option.xAxis.inverse).toBe(false);
    expect(option.yAxis.position).toBe('left');
  });

  it('has no animation, and no chart legend of its own (the component draws it in HTML)', () => {
    expect(optionOf('ltr').animation).toBe(false);
    expect(optionOf('ltr').legend.show).toBe(false);
  });
});

describe('round 11: the boxes of views F, G and J and the new filters become the backend’s requests', () => {
  const cases: [string, AnalyticsView, Partial<AnalyticsFilterState>, AnalyticsRunInput][] = [
    ['F with nothing filled', 'F', {}, ANALYTICS_RUN_F_REQUEST],
    ['G by month (no grouping is sent)', 'G', {}, ANALYTICS_RUN_G_MONTH_REQUEST],
    ['G by day', 'G', { groupBy: 'day' }, ANALYTICS_RUN_G_DAY_REQUEST],
    ['J by trip (no layout is sent)', 'J', {}, ANALYTICS_RUN_J_TRIP_REQUEST],
    ['J by purchase', 'J', { tripView: 'purchase' }, ANALYTICS_RUN_J_TRIP_BY_PURCHASE_REQUEST],
    ['J for one partner', 'J', { partner: 1 }, ANALYTICS_RUN_J_TRIP_PARTNER_REQUEST],
    [
      'A with consigned goods',
      'A',
      { goods: 'consigned' },
      ANALYTICS_RUN_A_GOODS_CONSIGNED_REQUEST,
    ],
    ['A with trip goods', 'A', { goods: 'trip' }, ANALYTICS_RUN_A_GOODS_TRIP_REQUEST],
  ];
  it.each(cases)('%s', (_name, view, part, request) => {
    const state: AnalyticsFilterState = { ...EMPTY_ANALYTICS_FILTERS, ...part };
    expect(filtersToWire(state, takesOf(view))).toEqual(filtersOf(request));
  });

  it('never sends a comparison to F, G or J: the backend refuses it, so the boxes do not offer it', () => {
    const state: AnalyticsFilterState = {
      ...EMPTY_ANALYTICS_FILTERS,
      from: '01-10-2026',
      to: '31-10-2026',
      compare: 'previous',
    };
    for (const view of ['F', 'G', 'J'] as const) {
      expect(takesOf(view)).not.toContain('compare');
      expect(filtersToWire(state, takesOf(view)).compare).toBe('off');
    }
    // The refused request of the backend is exactly the one the boxes cannot make.
    expect(filtersOf(ANALYTICS_RUN_F_COMPARE_REFUSED_REQUEST).compare).toBe('previous');
  });

  it('keeps a box of another view out of the request', () => {
    const state: AnalyticsFilterState = {
      ...EMPTY_ANALYTICS_FILTERS,
      partner: 1,
      trip: 1,
      tripView: 'item',
    };
    // View A takes no partner and no layout; a trip and goods only where the catalogue says.
    const a = filtersToWire(state, takesOf('A'));
    expect([a.partner_id, a.trip_view]).toEqual([null, null]);
    // View F takes the partner and nothing about trips.
    const f = filtersToWire(state, takesOf('F'));
    expect([f.partner_id, f.trip_id, f.trip_view, f.goods]).toEqual([1, null, null, 'all']);
  });

  it('sends the trip layout only when it is not the first, like the grouping', () => {
    const state = { ...EMPTY_ANALYTICS_FILTERS, tripView: 'trip' as const };
    expect(filtersToWire(state, takesOf('J')).trip_view).toBeNull();
    expect(filtersToWire({ ...state, tripView: 'day' }, takesOf('J')).trip_view).toBe('day');
  });

  it('knows a partner, a trip or other goods is a filter, so “clear the filters” is offered', () => {
    expect(isUnfiltered({ ...EMPTY_ANALYTICS_FILTERS, partner: 2 }, takesOf('F'))).toBe(false);
    expect(isUnfiltered({ ...EMPTY_ANALYTICS_FILTERS, trip: 1 }, takesOf('A'))).toBe(false);
    expect(isUnfiltered({ ...EMPTY_ANALYTICS_FILTERS, goods: 'garden' }, takesOf('A'))).toBe(false);
    // A layout is not a filter: nothing is left out of the rows by it.
    expect(isUnfiltered({ ...EMPTY_ANALYTICS_FILTERS, tripView: 'item' }, takesOf('J'))).toBe(true);
  });

  it('a saved view brings the partner, trip, goods and layout back and sends the same request', () => {
    const state: AnalyticsFilterState = {
      ...EMPTY_ANALYTICS_FILTERS,
      partner: 2,
      trip: 1,
      goods: 'trip',
      tripView: 'purchase',
    };
    const wire = filtersToWire(state, takesOf('J'));
    expect(wire).toMatchObject({ partner_id: 2, trip_id: 1, goods: 'trip', trip_view: 'purchase' });
    expect(stateFromWire(wire)).toMatchObject({
      partner: 2,
      trip: 1,
      goods: 'trip',
      tripView: 'purchase',
    });
    expect(filtersToWire(stateFromWire(wire), takesOf('J'))).toEqual(wire);
  });
});

describe('round 11: the columns of views F, G and J', () => {
  const columnsOf = (page: typeof ANALYTICS_RUN_J_TRIP) => page.columns.map((column) => column.key);

  it('names a partner column by his name in the app language', () => {
    const heading = { kind: 'partner' as const, names: { ur: 'الف', en: 'Partner A' } };
    expect(headingText(heading, context)).toBe('Partner A');
    expect(headingText(heading, { ...context, language: 'ur' })).toBe('الف');
  });

  it('J is never windowed: it shows the most useful columns and says how many it shows', () => {
    for (const step of ['compact', 'normal', 'wide'] as const) {
      const plan = planColumns('J', ANALYTICS_RUN_J_TRIP.columns, step, 0);
      expect(plan.window).toBeNull();
      expect(plan.trimmed?.total).toBe(18);
    }
  });

  it('J keeps the trip first, then its result and each partner’s part before the least used columns', () => {
    const plan = planColumns('J', ANALYTICS_RUN_J_TRIP.columns, 'wide', 0);
    const shown = ANALYTICS_RUN_J_TRIP.columns
      .filter((_column, index) => plan.visible.has(index))
      .map((column) => column.key);
    expect(shown[0]).toBe('group');
    expect(shown).toContain('result');
    for (const part of ['part_1', 'part_2', 'part_3']) expect(shown).toContain(part);
    // The columns of the PRD that do not fit wait behind the other-columns button, none is lost.
    expect(plan.trimmed?.shown).toBe(shown.length);
  });

  it('J brings the waiting columns in when asked, and the trip stays', () => {
    const plan = planColumns('J', ANALYTICS_RUN_J_TRIP.columns, 'compact', 0, true);
    const shown = ANALYTICS_RUN_J_TRIP.columns
      .filter((_column, index) => plan.visible.has(index))
      .map((column) => column.key);
    expect(shown).toContain('group');
  });

  it('F shows the partner and what he gained at every step, and G the period and the profit', () => {
    for (const step of ['compact', 'normal', 'wide'] as const) {
      const f = planColumns('F', ANALYTICS_RUN_F.columns, step, 0);
      const shownF = ANALYTICS_RUN_F.columns
        .filter((_column, index) => f.visible.has(index))
        .map((column) => column.key);
      expect(shownF).toEqual(expect.arrayContaining(['partner', 'profit_share', 'net_change']));
      const g = planColumns('G', ANALYTICS_RUN_G_DAY.columns, step, 0);
      const shownG = ANALYTICS_RUN_G_DAY.columns
        .filter((_column, index) => g.visible.has(index))
        .map((column) => column.key);
      expect(shownG).toEqual(expect.arrayContaining(['period', 'profit']));
    }
  });

  it('every column of F and G is shown at the wide step, none is left out', () => {
    expect(planColumns('F', ANALYTICS_RUN_F.columns, 'wide', 0).trimmed).toBeNull();
    expect(planColumns('G', ANALYTICS_RUN_G_DAY.columns, 'wide', 0).trimmed).toBeNull();
  });

  it('the partner columns of J are headed with whose they are, then what they hold', () => {
    const plan = planColumns('J', ANALYTICS_RUN_J_TRIP.columns, 'wide', 0, true);
    const grid = analyticsColumns('J', ANALYTICS_RUN_J_TRIP.columns, plan);
    // A grid context whose text shows the message and the values it was given.
    const showing: ListContext = {
      ...context,
      text: (id, vars) =>
        vars === undefined ? `<${id}>` : `<${id}|${Object.values(vars).join('|')}>`,
    };
    const header = (field: string) => {
      const found = grid.find((column) => column.field === field);
      if (found?.headerText === undefined) throw new Error(`no header for ${field}`);
      return found.headerText(showing);
    };
    expect(header('part_1')).toBe(
      '<analytics-col-partner-heading|Partner A|<analytics-col-partner-part>>',
    );
    expect(header('share_2')).toBe(
      '<analytics-col-partner-heading|Partner B|<analytics-col-partner-share>>',
    );
  });

  it('the figures that can be a loss are marked: the result of a trip and of each partner, G’s trips', () => {
    expect([...signedKeysOf('J', ANALYTICS_RUN_J_TRIP.columns)].sort()).toEqual([
      'part_1',
      'part_2',
      'part_3',
      'result',
    ]);
    expect([...signedKeysOf('G', ANALYTICS_RUN_G_DAY.columns)]).toEqual(['trips_result']);
    expect(signedKeysOf('F', ANALYTICS_RUN_F.columns).size).toBe(0);
    expect(signedKeysOf('A', ANALYTICS_RUN_A.columns).size).toBe(0);
    // A signed money column takes a balance's room, because its word has a slot after the figure.
    expect(columnWidth('money', 'result', true)).toBeGreaterThan(columnWidth('money', 'result'));
  });

  it('the columns keep the backend’s order', () => {
    expect(columnsOf(ANALYTICS_RUN_J_TRIP_BY_PURCHASE)[0]).toBe('group');
  });
});

describe('round 11: what the charts of F, G and J draw', () => {
  it('F: each partner’s profit share and net change side by side; the empty columns have no bar', () => {
    const plan = chartFor('F', ANALYTICS_RUN_F, context);
    expect(plan?.labelId).toBe('analytics-chart-f');
    expect(plan?.data.categories).toEqual(['Partner A', 'Partner B', 'Partner C']);
    expect(plan?.data.series.map((series) => series.name)).toEqual([
      '<report-col-profit-share>',
      '<analytics-col-net-change>',
    ]);
    expect(plan?.data.series[0]?.labels).toEqual(['996', '746', '746']);
    expect(plan?.data.stacked).toBe(false);
  });

  it('F: a loss is a bar below the line, written with its sign', () => {
    const plan = chartFor('F', ANALYTICS_RUN_F_TRIP, context);
    const net = plan?.data.series[1];
    expect(net?.labels).toEqual(['-296', '-222', '-222']);
    expect(net?.values.every((value) => value < 0)).toBe(true);
  });

  it('G: the money of each period side by side; a column with no figure in any row has no bar', () => {
    const day = chartFor('G', ANALYTICS_RUN_G_DAY, context);
    expect(day?.labelId).toBe('analytics-chart-g');
    expect(day?.data.categories).toEqual(['10-10-2026']);
    // The trips' result is empty on the worked day: four bars, not five.
    expect(day?.data.series.map((series) => series.name)).toEqual([
      '<report-col-total-comm>',
      '<analytics-col-shared-income>',
      '<analytics-col-shop-expenses>',
      '<analytics-col-profit>',
    ]);
    const trip = chartFor('G', ANALYTICS_RUN_G_TRIP_DAY, context);
    expect(trip?.data.series).toHaveLength(5);
    expect(trip?.data.series[4]?.labels).toEqual(['660']);
  });

  it('G: a day with no profit but a trip loss still shows its bar', () => {
    const plan = chartFor('G', ANALYTICS_RUN_G_TRIP_WRITTEN_OFF, context);
    expect(plan?.data.categories).toEqual(['10-10-2026', '11-10-2026']);
    expect(plan?.data.series[4]?.labels).toEqual(['660', '-2,600']);
  });

  it('G: many days are many thin bars with no figure written on them', () => {
    const row = ANALYTICS_RUN_G_DAY.rows[0];
    if (row === undefined) throw new Error('no row');
    const month = { ...ANALYTICS_RUN_G_DAY, rows: Array.from({ length: 30 }, () => row) };
    const plan = chartFor('G', month, context);
    expect(plan?.data.dense).toBe(true);
    // Every day is drawn: the bars are thin, not cut, so the picture is of the whole period.
    expect(plan?.shown).toBe(30);
    expect(chartFor('G', ANALYTICS_RUN_G_DAY, context)?.data.dense).toBe(false);
  });

  it('J: the sales money and the result of each trip, a loss below the line', () => {
    const plan = chartFor('J', ANALYTICS_RUN_J_TRIP, context);
    expect(plan?.labelId).toBe('analytics-chart-j');
    expect(plan?.data.categories).toEqual(['TRP-1 10-10-2026']);
    expect(plan?.data.series[0]?.labels).toEqual(['8,460']);
    expect(plan?.data.series[1]?.labels).toEqual(['660']);
    const lost = chartFor('J', ANALYTICS_RUN_J_TRIP_WRITTEN_OFF, context);
    expect(lost?.data.series[1]?.labels).toEqual(['-1,940']);
    expect(lost?.data.series[1]?.values[0]).toBeLessThan(0);
  });

  it('draws nothing when F, G or J has no rows', () => {
    expect(chartFor('F', { ...ANALYTICS_RUN_F, rows: [] }, context)).toBeNull();
    expect(chartFor('G', { ...ANALYTICS_RUN_G_DAY, rows: [] }, context)).toBeNull();
    expect(chartFor('J', { ...ANALYTICS_RUN_J_TRIP, rows: [] }, context)).toBeNull();
  });
});
