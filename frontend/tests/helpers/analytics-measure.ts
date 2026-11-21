// Test helper: the answers of the analytics commands for the measure of plan 84 (PRD FR-18.10, the box "ناپ"),
// with the numbers of the plan (October 2026, after Events 1 to 5, both sides). They are typed fakes of the
// backend's answers: the shape is that of the recorded answers in analytics-real.ts, the figures are the plan's.
// The fake is a backend, so it may write each figure down; the screen under test adds up nothing.
//
//   خام بکری      کیلا 29,400   سیب 12,000   میزان 41,400
//   رقم گاہک       کیلا 31,564   سیب 12,800   میزان 44,364   (value + buyer commission + لاگا, C5)
//   نگ            کیلا 50       سیب 10        میزان 60
//   کل کمیشن      کیلا 3,928    سیب 1,520     میزان 5,448
//   بیوپاری کمیشن  کیلا 1,764    سیب 720       میزان 2,484
//   گاہک کمیشن     کیلا 1,764    سیب 720       میزان 2,484
//   لاگا          کیلا 400      سیب 80        میزان 480

import { vi } from 'vitest';
import type { AnalyticsCatalogEntry } from '../../src/bindings/AnalyticsCatalogEntry';
import type { AnalyticsMeasure } from '../../src/bindings/AnalyticsMeasure';
import type { AnalyticsPage } from '../../src/bindings/AnalyticsPage';
import type { AnalyticsPrintData } from '../../src/bindings/AnalyticsPrintData';
import type { AnalyticsView } from '../../src/bindings/AnalyticsView';
import type { UiError } from '../../src/bindings/UiError';
import * as analyticsApi from '../../src/features/analytics/api';
import { refusedWith } from './analytics';
import {
  ANALYTICS_CATALOG,
  ANALYTICS_PRINT_DATA_D_UR,
  ANALYTICS_RUN_A,
  ANALYTICS_RUN_D_MONTH,
  ANALYTICS_RUN_H,
} from './analytics-real';
import { ANALYTICS_RUN_G_MONTH } from './analytics-trips-real';

/** The views that take a measure (PRD FR-18.10); the others (F, I, J, K, M) are about partners, gardens, trips, fruit and expenses. */
export const MEASURE_VIEWS: readonly AnalyticsView[] = ['A', 'B', 'C', 'D', 'E', 'G', 'H', 'L'];
export const NO_MEASURE_VIEWS: readonly AnalyticsView[] = ['F', 'I', 'J', 'K', 'M'];

export const MEASURES: readonly AnalyticsMeasure[] = [
  'gross',
  'buyer_total',
  'qty',
  'total_comm',
  'seller_comm',
  'buyer_comm',
  'laga',
];

/** The views whose column of the measure has its key (the matrices D, E and L, and the top 10, H). */
const MEASURE_COLUMN_VIEWS: readonly AnalyticsView[] = ['D', 'E', 'H', 'L'];

/** The refusal of a sort column the view does not have. */
const SORT_REFUSED: UiError = {
  ok: false,
  errors: [{ code: 'ERR_OUT_OF_RANGE', field: 'sort.column', values: {} }],
  ref: 'E-20261120-0001',
};

/** The catalogue as the backend lists it from plan 84: the measure on the views that take it. */
export const MEASURE_CATALOG: AnalyticsCatalogEntry[] = ANALYTICS_CATALOG.map((entry) =>
  MEASURE_VIEWS.includes(entry.id) ? { ...entry, filters: [...entry.filters, 'measure'] } : entry,
);

/** Whole rupees (a count for the quantity) of each item, for each measure. */
const BANANA: Record<AnalyticsMeasure, number> = {
  gross: 29_400,
  buyer_total: 31_564,
  qty: 50,
  total_comm: 3_928,
  seller_comm: 1_764,
  buyer_comm: 1_764,
  laga: 400,
};
const APPLE: Record<AnalyticsMeasure, number> = {
  gross: 12_000,
  buyer_total: 12_800,
  qty: 10,
  total_comm: 1_520,
  seller_comm: 720,
  buyer_comm: 720,
  laga: 80,
};
/** The totals, written down on their own: they are what the backend sums over the whole filter. */
export const TOTAL: Record<AnalyticsMeasure, number> = {
  gross: 41_400,
  buyer_total: 44_364,
  qty: 60,
  total_comm: 5_448,
  seller_comm: 2_484,
  buyer_comm: 2_484,
  laga: 480,
};

/** The column title of each measure in the tables (the report words). */
const COLUMN_TITLE: Record<AnalyticsMeasure, string> = {
  gross: 'report-col-gross',
  buyer_total: 'report-col-buyer-total',
  qty: 'report-col-qty',
  total_comm: 'report-col-total-comm',
  seller_comm: 'report-col-seller-comm',
  buyer_comm: 'report-col-buyer-comm',
  laga: 'report-col-laga',
};

/** A figure of a measure as a cell: a count for the quantity, money (paisa) for the rest. */
function figure(measure: AnalyticsMeasure, value: number): Record<string, unknown> {
  return measure === 'qty' ? { type: 'number', value } : { type: 'money', paisa: value * 100 };
}

const kindOf = (measure: AnalyticsMeasure) => (measure === 'qty' ? 'number' : 'money');

/** What a result of this view and measure says about its filters: the measure it was read for. */
function echo<T extends { filters: object }>(base: T, measure: AnalyticsMeasure): T['filters'] {
  return { ...base.filters, measure };
}

/** View D (item × month) for a measure: the cells, the row total and the column total are that measure. */
export function matrixDPage(measure: AnalyticsMeasure): AnalyticsPage {
  const base = ANALYTICS_RUN_D_MONTH;
  const rows = [BANANA[measure], APPLE[measure]];
  const page = {
    ...base,
    filters: echo(base, measure),
    columns: base.columns.map((column) => {
      if (column.key === 'p_2026_10') return { ...column, kind: kindOf(measure) };
      // The row total has the key of the measure (the backend's rule) and keeps its heading message.
      if (column.heading === null && column.title_id === 'analytics-col-row-total') {
        return { ...column, key: measure, kind: kindOf(measure) };
      }
      return column;
    }),
    rows: base.rows.map((row, index) => ({
      ...row,
      cells: [row.cells[0], figure(measure, rows[index] ?? 0), figure(measure, rows[index] ?? 0)],
    })),
    totals: [{ type: 'empty' }, figure(measure, TOTAL[measure]), figure(measure, TOTAL[measure])],
    charts: base.charts.map((chart) => ({
      ...chart,
      series: [{ key: measure, kind: kindOf(measure) }],
      points: chart.points.map((point) => ({
        ...point,
        values: [figure(measure, TOTAL[measure])],
      })),
    })),
  };
  return page as unknown as AnalyticsPage;
}

/** View A (by item) for a measure: the table keeps every column, the chart draws the measure. */
export function rankedAPage(measure: AnalyticsMeasure): AnalyticsPage {
  const base = ANALYTICS_RUN_A;
  const rows = [BANANA[measure], APPLE[measure]];
  const page = {
    ...base,
    filters: echo(base, measure),
    charts: base.charts.map((chart) => ({
      ...chart,
      series: [{ key: measure, kind: kindOf(measure) }],
      points: chart.points.map((point, index) => ({
        ...point,
        values: [figure(measure, rows[index] ?? 0)],
      })),
    })),
  };
  return page as unknown as AnalyticsPage;
}

/**
 * View G (trend) for a measure: the table keeps every column, and the chart is that measure alone, one point for each
 * period (the backend's rule from plan 84), with one series that has the key of the measure.
 */
export function trendGPage(measure: AnalyticsMeasure): AnalyticsPage {
  const base = ANALYTICS_RUN_G_MONTH;
  const page = {
    ...base,
    filters: echo(base, measure),
    charts: base.charts.map((chart) => ({
      ...chart,
      series: [{ key: measure, kind: kindOf(measure) }],
      points: chart.points.map((point) => ({
        ...point,
        values: [figure(measure, TOTAL[measure])],
      })),
    })),
  };
  return page as unknown as AnalyticsPage;
}

/** The three lists of view H: items, sellers and buyers, with their gross sales and their quantity. */
const TOP_FIGURES: Record<'gross' | 'qty', { section: string; rows: [number, number][] }[]> = {
  // The plan: items کیلا then سیب; sellers رحیم then کریم; buyers ارشد 17,400, اکرم 12,000, شاہد 12,000.
  gross: [
    {
      section: 'analytics-top-items',
      rows: [
        [0, 29_400],
        [1, 12_000],
      ],
    },
    {
      section: 'analytics-top-sellers',
      rows: [
        [0, 29_400],
        [1, 12_000],
      ],
    },
    {
      section: 'analytics-top-buyers',
      rows: [
        [0, 17_400],
        [1, 12_000],
        [2, 12_000],
      ],
    },
  ],
  // By quantity the buyers are ارشد 30, اکرم 20, شاہد 10 and the others are as they were.
  qty: [
    {
      section: 'analytics-top-items',
      rows: [
        [0, 50],
        [1, 10],
      ],
    },
    {
      section: 'analytics-top-sellers',
      rows: [
        [0, 50],
        [1, 10],
      ],
    },
    {
      section: 'analytics-top-buyers',
      rows: [
        [0, 30],
        [1, 20],
        [2, 10],
      ],
    },
  ],
};

/** View H (top 10) for a measure that has a worked list: the lists and their charts follow it. */
export function topHPage(measure: 'gross' | 'qty'): AnalyticsPage {
  const base = ANALYTICS_RUN_H;
  const figures = TOP_FIGURES[measure];
  // The base rows are, in order: heading, rows of the items, heading, sellers, heading, buyers.
  const bySection = [base.rows.slice(1, 3), base.rows.slice(4, 6), base.rows.slice(7, 10)];
  const rows = figures.flatMap((section, at) => [
    base.rows[[0, 3, 6][at] ?? 0],
    ...(bySection[at] ?? []).map((row, index) => ({
      ...row,
      cells: [row.cells[0], row.cells[1], figure(measure, section.rows[index]?.[1] ?? 0)],
    })),
  ]);
  const page = {
    ...base,
    filters: echo(base, measure),
    columns: [
      base.columns[0],
      base.columns[1],
      {
        key: measure,
        title_id: COLUMN_TITLE[measure],
        kind: kindOf(measure),
        heading: null,
        drill: null,
      },
    ],
    rows: rows.map((row) =>
      row?.kind === 'group'
        ? { ...row, cells: [row.cells[0], { type: 'empty' }, { type: 'empty' }] }
        : row,
    ),
    totals: [
      { type: 'empty' },
      { type: 'empty' },
      figure(measure, measure === 'qty' ? 60 : 41_400),
    ],
    charts: base.charts.map((chart, at) => ({
      ...chart,
      series: [{ key: measure, kind: kindOf(measure) }],
      points: chart.points.map((point, index) => ({
        ...point,
        values: [figure(measure, figures[at]?.rows[index]?.[1] ?? 0)],
      })),
    })),
  };
  return page as unknown as AnalyticsPage;
}

/** The print data of view D in the Urdu print, for a measure: the header says it, the row total names it. */
export function printDataD(measure: AnalyticsMeasure): AnalyticsPrintData {
  const base = ANALYTICS_PRINT_DATA_D_UR;
  const page = matrixDPage(measure);
  const print = {
    ...base,
    filters: [
      ...base.filters,
      { label_id: `analytics-filter-measure-${measure.replaceAll('_', '-')}`, value: '' },
    ],
    columns: base.columns.map((column) =>
      column.heading === null && column.title_id === 'analytics-col-row-total'
        ? { ...column, key: measure, kind: kindOf(measure) }
        : column,
    ),
    charts: page.charts,
  };
  return print as unknown as AnalyticsPrintData;
}

/**
 * Makes the fake backend answer the views A, D and H by the measure it is asked for (the others answer as
 * `fakeAnalytics` does). Call it after `fakeAnalytics`, which sets the other commands.
 */
export function answerByMeasure(): void {
  const before = vi.mocked(analyticsApi.analyticsRun).getMockImplementation();
  vi.mocked(analyticsApi.analyticsRun).mockImplementation((input) => {
    const measure = input.filters.measure;
    if (measure !== undefined) {
      // The matrices and the top 10 have a column with the key of the measure: an order on the key of another
      // measure is refused, as the backend does (`ERR_OUT_OF_RANGE` on `sort.column`).
      const sorted = input.sort?.column;
      if (
        MEASURE_COLUMN_VIEWS.includes(input.view) &&
        MEASURES.some((key) => key === sorted && key !== measure)
      ) {
        return Promise.reject(refusedWith(SORT_REFUSED));
      }
      if (input.view === 'A') return Promise.resolve(rankedAPage(measure));
      if (input.view === 'D') return Promise.resolve(matrixDPage(measure));
      if (input.view === 'H' && (measure === 'gross' || measure === 'qty')) {
        return Promise.resolve(topHPage(measure));
      }
    }
    if (before === undefined) return Promise.reject(new Error('the fake backend has no answer'));
    return before(input);
  });
}
