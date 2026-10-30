// The charts above a view's table (PRD FR-18.3): drawn from the `charts` the backend sends with
// every page (plan 38 R4). A chart covers the WHOLE filter, its numbers are the sums of the table,
// and it never changes with a sort or a page. This file only turns a chart of the backend into what
// the bar chart draws: every figure written on a bar is the text of the cell the backend sent
// (`cellText`) and the heights are numbers for the drawing only (`chartValueOfPaisa`), so the chart
// adds up nothing and can never disagree with the table under it (CLAUDE.md rule 1).
//
// A ranked chart is bars of the biggest rows with an `others` bar for the rest; a trend is one group
// of bars for each period (many thin bars when there are many). View H has one chart for each
// section; views F and K have none.

import type { AnalyticsChart } from '../../bindings/AnalyticsChart';
import type { AnalyticsPage } from '../../bindings/AnalyticsPage';
import type { AnalyticsView } from '../../bindings/AnalyticsView';
import type { ReportCell } from '../../bindings/ReportCell';
import { chartValueOfPaisa } from '../../shared/format';
import type { ListContext } from '../../shared/grid';
import type { BarChartData, BarSeries } from '../../shared/charts';
import { cellText, nameWithTag } from '../reports';

/** A trend with more periods than this is drawn as many thin bars, without figures on them. */
const DENSE_AFTER = 12;

export interface ChartPlan {
  data: BarChartData;
  /** The message of what the chart shows, for a screen reader and the line above it. */
  labelId: string;
  /** For a view with sections (H): the heading of the section the chart belongs to. */
  caption: string | null;
  /** How many rows the chart shows, and how many there are: it says so when it cut them. */
  shown: number;
  total: number;
}

/** The height of a bar for a cell: money and balances as paisa, a count or a share as it is. */
function heightOf(cell: ReportCell | undefined): number {
  if (cell === undefined) return 0;
  switch (cell.type) {
    case 'money':
    case 'balance':
      return chartValueOfPaisa(cell.paisa);
    case 'number':
      return cell.value;
    case 'percent':
      return cell.hundredths;
    default:
      return 0;
  }
}

/**
 * The name under a bar. A party is written by his name with his tag, not with his code: the code is
 * long and a name is what is read in a chart (the table under it has both).
 */
function categoryText(cell: ReportCell, context: ListContext): string {
  return cell.type === 'party'
    ? nameWithTag(cell.party, cell.party.tag, context.language)
    : cellText(cell, context);
}

function planOf(
  view: AnalyticsView,
  page: AnalyticsPage,
  chart: AnalyticsChart,
  context: ListContext,
): ChartPlan | null {
  if (chart.points.length === 0 || chart.series.length === 0) return null;
  const series: BarSeries[] = chart.series.map((one, position) => {
    const column = page.columns.find((candidate) => candidate.key === one.key);
    return {
      name: column === undefined ? one.key : context.text(column.title_id),
      values: chart.points.map((point) => heightOf(point.values[position])),
      labels: chart.points.map((point) => {
        const cell = point.values[position];
        return cell === undefined ? '' : cellText(cell, context);
      }),
    };
  });
  const shown = chart.points.filter((point) => !point.is_others).length;
  return {
    data: {
      categories: chart.points.map((point) =>
        point.is_others
          ? context.text('analytics-chart-others')
          : categoryText(point.label, context),
      ),
      series,
      stacked: false,
      dense: chart.kind === 'trend' && chart.points.length > DENSE_AFTER,
    },
    labelId: `analytics-chart-${view.toLowerCase()}`,
    caption: chart.title === null ? null : categoryText(chart.title, context),
    shown,
    total: shown + chart.others_count,
  };
}

/** The charts of a page of a view, as the bar chart draws them. None when the backend sent none. */
export function chartsOf(
  view: AnalyticsView,
  page: AnalyticsPage,
  context: ListContext,
): ChartPlan[] {
  return page.charts.flatMap((chart) => {
    const plan = planOf(view, page, chart, context);
    return plan === null ? [] : [plan];
  });
}
