// What the two mini charts of the dashboard draw (PRD 9.4): the last 30 days of خام بکری and
// commission, and this month's commission by item. The heights are numbers for the drawing only
// (`chartValueOfPaisa`); every figure written on or in a chart is the text of `formatPaisa`, so a
// chart adds up nothing and can never disagree with the tiles (CLAUDE.md rule 1).

import type { DashboardChartDay } from '../../bindings/DashboardChartDay';
import type { DashboardChartItem } from '../../bindings/DashboardChartItem';
import type { Language } from '../../i18n';
import type { BarChartData } from '../../shared/charts';
import { chartValueOfPaisa, isoToDisplayDate, isZeroPaisa, type Paisa } from '../../shared/format';
import { displayName } from '../../shared/i18n';

/** What a chart needs to write its words: Fluent, the language and the money format. */
export interface ChartWords {
  language: Language;
  text: (id: string) => string;
  money: (amount: Paisa) => string;
}

/** The day and month of a date as the user reads them ("10-10"), for the axis. */
function dayAndMonth(iso: string): string {
  return isoToDisplayDate(iso).slice(0, 5);
}

/** True when nothing was sold and no commission was made on any of the days. */
export function daysAreEmpty(days: readonly DashboardChartDay[]): boolean {
  return days.every((day) => isZeroPaisa(day.gross_paisa) && isZeroPaisa(day.commission_paisa));
}

/** The last 30 days: two bars for each day, خام بکری and commission. Oldest first, today last. */
export function daysChart(days: readonly DashboardChartDay[], words: ChartWords): BarChartData {
  return {
    categories: days.map((day) => dayAndMonth(day.date)),
    series: [
      {
        name: words.text('dashboard-chart-gross'),
        values: days.map((day) => chartValueOfPaisa(day.gross_paisa)),
        labels: days.map((day) => words.money(day.gross_paisa)),
      },
      {
        name: words.text('dashboard-chart-commission'),
        values: days.map((day) => chartValueOfPaisa(day.commission_paisa)),
        labels: days.map((day) => words.money(day.commission_paisa)),
      },
    ],
    stacked: false,
    dense: true,
  };
}

/** This month's commission by item: one bar for each of the top items, the biggest first. */
export function itemsChart(items: readonly DashboardChartItem[], words: ChartWords): BarChartData {
  return {
    categories: items.map((item) => displayName(item, words.language)),
    series: [
      {
        name: words.text('dashboard-chart-commission'),
        values: items.map((item) => chartValueOfPaisa(item.commission_paisa)),
        labels: items.map((item) => words.money(item.commission_paisa)),
      },
    ],
    stacked: false,
  };
}
