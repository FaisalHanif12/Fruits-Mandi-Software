// The measure of the analytics views (plan 84, PRD FR-18.10, the box "ناپ"), without a screen: which views take
// it and what is sent to each, what a saved view or the address or the Owner's memory brings back, what the
// charts and the row total say about it, and that a quantity is drawn as a count and never as rupees. The
// numbers are those of the plan (October 2026, after Events 1 to 5). The screen adds up nothing: every figure
// is the backend's.

import { describe, expect, it } from 'vitest';
import type { AnalyticsMeasure } from '../../src/bindings/AnalyticsMeasure';
import type { AnalyticsView } from '../../src/bindings/AnalyticsView';
import { chartsOf, printChartsOf } from '../../src/features/analytics/chart';
import { analyticsColumns, planColumns } from '../../src/features/analytics/columns';
import {
  CLASSIC_MEASURE,
  DEFAULT_MEASURE,
  EMPTY_ANALYTICS_FILTERS,
  MEASURE_CHOICES,
  filtersToWire,
  isUnfiltered,
  measureOfFilters,
  measureTitleId,
  stateFromWire,
  takesMeasure,
} from '../../src/features/analytics/filters';
import {
  analyticsFromRemembered,
  rememberedAnalytics,
} from '../../src/features/analytics/remember';
import {
  filtersFromSearch,
  hasAnalyticsSearch,
  parseAnalyticsSearch,
} from '../../src/features/analytics/search';
import { formatPaisa } from '../../src/shared/format';
import type { ListContext } from '../../src/shared/grid';
import { LANGUAGES, type Language } from '../../src/i18n';
import {
  MEASURES,
  MEASURE_CATALOG,
  MEASURE_VIEWS,
  NO_MEASURE_VIEWS,
  TOTAL,
  matrixDPage,
  rankedAPage,
  trendGPage,
} from '../helpers/analytics-measure';
import { ANALYTICS_RUN_G_MONTH } from '../helpers/analytics-trips-real';
import { message } from '../helpers/render';

const takes = (view: AnalyticsView) => {
  const found = MEASURE_CATALOG.find((entry) => entry.id === view);
  if (found === undefined) throw new Error(`no view ${view}`);
  return found.filters;
};

/** The words and figures of a list in a language, as the app's list context makes them. */
function contextOf(language: Language): ListContext {
  return {
    language,
    text: (id, vars) => message(language, id, vars ?? {}),
    money: (amount) => formatPaisa(amount, { grouping: 'international' }),
    signedMoney: (value) => String(value),
  };
}

describe('which views take the measure', () => {
  it('the catalogue lists it on A, B, C, D, E, G, H and L and on no other view', () => {
    for (const view of MEASURE_VIEWS) expect(takesMeasure(takes(view)), view).toBe(true);
    for (const view of NO_MEASURE_VIEWS) expect(takesMeasure(takes(view)), view).toBe(false);
  });

  it('sends the chosen measure to each of those views, sales included, and none to the others', () => {
    for (const measure of MEASURE_CHOICES) {
      const state = { ...EMPTY_ANALYTICS_FILTERS, measure };
      for (const view of MEASURE_VIEWS) {
        expect(filtersToWire(state, takes(view), view), `${view} ${measure}`).toMatchObject({
          measure,
        });
      }
      for (const view of NO_MEASURE_VIEWS) {
        // The backend refuses a measure on these views, so a choice left from view A never goes there.
        expect(filtersToWire(state, takes(view), view), `${view} ${measure}`).not.toHaveProperty(
          'measure',
        );
      }
    }
  });

  it('a new screen starts with sales, خام بکری', () => {
    expect(DEFAULT_MEASURE).toBe('gross');
    expect(EMPTY_ANALYTICS_FILTERS.measure).toBe('gross');
    expect(filtersToWire(EMPTY_ANALYTICS_FILTERS, takes('D'), 'D')).toMatchObject({
      measure: 'gross',
    });
  });

  it('the seven choices are the backend’s seven, in the order of the box', () => {
    expect([...MEASURE_CHOICES]).toEqual([...MEASURES]);
  });

  it('the measure is not a filter: choosing one leaves "nothing to clear"', () => {
    const state = { ...EMPTY_ANALYTICS_FILTERS, measure: 'qty' as const };
    expect(isUnfiltered(state, takes('D'), 'D')).toBe(true);
  });
});

describe('what brings a measure back', () => {
  it('a saved view keeps it, and a view saved before plan 84 opens as the total commission it was', () => {
    for (const measure of MEASURE_CHOICES) {
      const saved = filtersToWire({ ...EMPTY_ANALYTICS_FILTERS, measure }, takes('D'), 'D');
      expect(stateFromWire(saved).measure).toBe(measure);
    }
    const before = filtersToWire(EMPTY_ANALYTICS_FILTERS, takes('D'), 'D');
    delete before.measure;
    expect(stateFromWire(before).measure).toBe(CLASSIC_MEASURE);
    expect(CLASSIC_MEASURE).toBe('total_comm');
  });

  it('is kept for the Owner and read back; sales is not kept (it is the start); an unreadable one is sales', () => {
    expect(rememberedAnalytics(EMPTY_ANALYTICS_FILTERS)).toBeNull();
    const kept = rememberedAnalytics({ ...EMPTY_ANALYTICS_FILTERS, measure: 'qty' });
    expect(kept).toEqual({ measure: 'qty' });
    expect(analyticsFromRemembered(JSON.parse(JSON.stringify(kept)))?.measure).toBe('qty');
    expect(analyticsFromRemembered({ measure: 'profit' })?.measure).toBe('gross');
    expect(analyticsFromRemembered({ measure: 7 })?.measure).toBe('gross');
  });

  it('opens from an address, as the assistant’s buttons do, and an unknown word is left out', () => {
    const address = parseAnalyticsSearch({ measure: 'buyer_total', item: 5 });
    expect(address.measure).toBe('buyer_total');
    expect(filtersFromSearch(address).measure).toBe('buyer_total');
    expect(hasAnalyticsSearch(parseAnalyticsSearch({ measure: 'buyer_total' }))).toBe(true);
    expect(parseAnalyticsSearch({ measure: 'profit' })).toEqual({});
    expect(filtersFromSearch(parseAnalyticsSearch({})).measure).toBe('gross');
  });

  it('reads the measure a result echoes, and a result that names none as the total commission', () => {
    expect(measureOfFilters({ measure: 'laga' })).toBe('laga');
    expect(measureOfFilters({})).toBe('total_comm');
  });
});

describe('the heading of a row total names the measure its cells hold', () => {
  it('maps the one heading message to one message for each measure', () => {
    expect(measureTitleId('analytics-col-row-total', 'total_comm')).toBe('analytics-col-row-total');
    expect(measureTitleId('analytics-col-row-total', 'gross')).toBe(
      'analytics-col-row-total-gross',
    );
    expect(measureTitleId('analytics-col-row-total', 'buyer_total')).toBe(
      'analytics-col-row-total-buyer-total',
    );
    expect(measureTitleId('analytics-col-row-total', 'seller_comm')).toBe(
      'analytics-col-row-total-seller-comm',
    );
    // No other column is renamed.
    expect(measureTitleId('report-col-gross', 'qty')).toBe('report-col-gross');
  });

  it.each(LANGUAGES)('view D draws "%s" words for the measure it holds', (language) => {
    const context = contextOf(language);
    const expected: Record<AnalyticsMeasure, string> = {
      gross: message(language, 'analytics-col-row-total-gross'),
      buyer_total: message(language, 'analytics-col-row-total-buyer-total'),
      qty: message(language, 'analytics-col-row-total-qty'),
      total_comm: message(language, 'analytics-col-row-total'),
      seller_comm: message(language, 'analytics-col-row-total-seller-comm'),
      buyer_comm: message(language, 'analytics-col-row-total-buyer-comm'),
      laga: message(language, 'analytics-col-row-total-laga'),
    };
    for (const measure of MEASURES) {
      const page = matrixDPage(measure);
      const plan = planColumns('D', page.columns, 'normal', 0);
      const column = analyticsColumns('D', page.columns, plan, measure).find(
        (candidate) => candidate.field === measure,
      );
      // The total commission keeps the heading message it always had; the others are named by `headerText`.
      expect(column?.headerText?.(context) ?? context.text(column?.headerId ?? ''), measure).toBe(
        expected[measure],
      );
    }
    // Every heading is a different phrase, so the Owner can tell the measures apart.
    expect(new Set(Object.values(expected)).size).toBe(MEASURES.length);
  });
});

describe('the charts say and draw the measure', () => {
  it.each(LANGUAGES)('view D with sales or the total commission in %s', (language) => {
    const context = contextOf(language);
    const gross = chartsOf('D', matrixDPage('gross'), context)[0];
    const comm = chartsOf('D', matrixDPage('total_comm'), context)[0];
    expect(gross?.measure).toBe('gross');
    expect(comm?.measure).toBe('total_comm');
    // The title of the chart and the name of its series are the measure's words.
    expect(context.text(gross?.labelId ?? '', { measure: 'gross' })).toBe(
      message(language, 'analytics-chart-d', { measure: 'gross' }),
    );
    expect(message(language, 'analytics-chart-d', { measure: 'gross' })).not.toBe(
      message(language, 'analytics-chart-d', { measure: 'total_comm' }),
    );
    expect(gross?.data.series[0]?.name).toBe(message(language, 'analytics-col-row-total-gross'));
    expect(comm?.data.series[0]?.name).toBe(message(language, 'analytics-col-row-total'));
    // The plan's October figures: 41,400 of sales, 5,448 of commission, written by the backend's cells.
    expect(gross?.data.series[0]?.labels).toEqual(['41,400']);
    expect(comm?.data.series[0]?.labels).toEqual(['5,448']);
  });

  it('view A by the buyer total names the series with the measure’s own words (the table has no such column)', () => {
    const page = rankedAPage('buyer_total');
    expect(page.columns.some((column) => column.key === 'buyer_total')).toBe(false);
    for (const language of LANGUAGES) {
      const plan = chartsOf('A', page, contextOf(language))[0];
      expect(plan?.data.series[0]?.name).toBe(
        message(language, 'analytics-measure-name', { measure: 'buyer_total' }),
      );
      expect(plan?.data.series[0]?.labels).toEqual(['31,564', '12,800']);
    }
  });

  it('a quantity is a count: its bars are 50 and 10, never paisa or rupees', () => {
    const context = contextOf('en');
    const qty = chartsOf('A', rankedAPage('qty'), context)[0];
    expect(qty?.data.series[0]?.values).toEqual([50, 10]);
    expect(qty?.data.series[0]?.labels).toEqual(['50', '10']);
    // The count has no paisa in it: 50 is not drawn as 5,000, and the money measures keep their cells' own figures.
    expect(qty?.data.series[0]?.values.every((height) => height !== null && height < 100)).toBe(
      true,
    );
    const gross = chartsOf('A', rankedAPage('gross'), context)[0];
    expect(gross?.data.series[0]?.labels).toEqual(['29,400', '12,000']);
    expect(TOTAL.qty).toBe(60);
  });

  it.each(LANGUAGES)(
    'view G draws the measure alone and is titled for it; with no measure it keeps its five figures (%s)',
    (language) => {
      const context = contextOf(language);
      for (const measure of ['gross', 'total_comm', 'qty'] as const) {
        const plan = chartsOf('G', trendGPage(measure), context)[0];
        expect(plan?.labelId, measure).toBe('analytics-chart-g-measure');
        expect(plan?.data.series).toHaveLength(1);
        // The table of G has a column for sales but none with the key of the total commission or the
        // quantity: the series is named from the measure's own words in every case.
        expect(plan?.data.series[0]?.name, measure).toBe(
          message(language, 'analytics-measure-name', { measure }),
        );
        expect(message(language, 'analytics-chart-g-measure', { measure })).toContain(
          message(language, 'analytics-measure-name', { measure }),
        );
      }
      // A page read with no measure draws the five figures it always did, under its own title.
      const classic = chartsOf('G', ANALYTICS_RUN_G_MONTH, context)[0];
      expect(classic?.labelId).toBe('analytics-chart-g');
      expect(classic?.data.series.length).toBeGreaterThan(1);
    },
  );

  it('a print draws the same: the title and the series in the print language, from the measure sent', () => {
    for (const language of LANGUAGES) {
      const page = matrixDPage('qty');
      const plans = printChartsOf('D', page.charts, page.columns, contextOf(language), 'qty');
      expect(plans[0]?.measure).toBe('qty');
      expect(plans[0]?.data.series[0]?.name).toBe(message(language, 'analytics-col-row-total-qty'));
    }
  });
});
