// What the two mini charts of the dashboard draw (round 9, PRD 9.4): the last 30 days with two bars
// each (خام بکری and commission), and the top items of the month. The heights are numbers for the
// drawing and every written figure is the text of `formatPaisa`, so a chart can never disagree with
// the tiles (CLAUDE.md rule 1).

import { describe, expect, it } from 'vitest';
import {
  daysAreEmpty,
  daysChart,
  itemsChart,
  type ChartWords,
} from '../../src/features/dashboard/charts';
import { formatPaisa } from '../../src/shared/format';
import { buildBarOption } from '../../src/shared/charts';
import { DASHBOARD_WORKED_DAY } from '../helpers/dashboard-real';
import { paisaForTest } from '../helpers/paisa';

const WORDS = (language: 'ur' | 'en'): ChartWords => ({
  language,
  text: (id) => id,
  money: (amount) => formatPaisa(amount, { grouping: 'international' }),
});

describe('the last 30 days', () => {
  const chart = daysChart(DASHBOARD_WORKED_DAY.chart_days, WORDS('en'));

  it('has a bar pair for each of the 30 days, oldest first, named by day and month', () => {
    expect(chart.categories).toHaveLength(30);
    expect(chart.categories[0]).toBe('11-09');
    expect(chart.categories.at(-1)).toBe('10-10');
    expect(chart.series.map((series) => series.name)).toEqual([
      'dashboard-chart-gross',
      'dashboard-chart-commission',
    ]);
  });

  it('writes the figures of the backend: today 41,400 sold and 5,448 commission', () => {
    expect(chart.series[0]?.labels.at(-1)).toBe('41,400');
    expect(chart.series[1]?.labels.at(-1)).toBe('5,448');
    expect(chart.series[0]?.values.at(-1)).toBe(4_140_000);
    expect(chart.series[0]?.values[0]).toBe(0);
  });

  it('is a dense chart: no figure on each of the 60 thin bars', () => {
    expect(chart.dense).toBe(true);
    const option = buildBarOption(chart, {
      direction: 'ltr',
      fontFamily: 'Inter',
      colors: ['#111', '#222'],
      ink: '#000',
      inkMuted: '#555',
      border: '#ccc',
      surface: '#fff',
    }) as { series: { label: { show: boolean } }[]; xAxis: { axisLabel: { interval: unknown } } };
    expect(option.series.every((series) => !series.label.show)).toBe(true);
    expect(option.xAxis.axisLabel.interval).toBe('auto');
  });

  it('knows a month with nothing sold', () => {
    expect(daysAreEmpty(DASHBOARD_WORKED_DAY.chart_days)).toBe(false);
    expect(
      daysAreEmpty(
        DASHBOARD_WORKED_DAY.chart_days.map((day) => ({
          ...day,
          gross_paisa: paisaForTest(0),
          commission_paisa: paisaForTest(0),
        })),
      ),
    ).toBe(true);
  });
});

describe('the top items of the month', () => {
  it('names each item in the language of the app and writes its commission', () => {
    const english = itemsChart(DASHBOARD_WORKED_DAY.chart_items, WORDS('en'));
    expect(english.categories).toEqual(['Banana', 'Apple']);
    expect(english.series[0]?.labels).toEqual(['3,928', '1,520']);
    const urdu = itemsChart(DASHBOARD_WORKED_DAY.chart_items, WORDS('ur'));
    expect(urdu.categories).toEqual(['کیلا', 'سیب']);
    // Same figures in both languages; only the names change (PRD FR-7.32).
    expect(urdu.series[0]?.labels).toEqual(english.series[0]?.labels);
    expect(english.dense).not.toBe(true);
  });
});
