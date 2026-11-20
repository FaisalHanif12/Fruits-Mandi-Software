// The charts above a view's table (PRD FR-18.3): drawn from the `charts` the backend sends with
// every page (plan 38 R4). A chart covers the WHOLE filter, its numbers are the sums of the table,
// and it never changes with a sort or a page. This file only turns a chart of the backend into what
// the bar chart draws: every figure written on a bar is the text of the cell the backend sent
// (`cellText`) and the heights are numbers for the drawing only (`chartValueOfPaisa`), so the chart
// adds up nothing and can never disagree with the table under it (CLAUDE.md rule 1).
//
// A ranked chart is bars of the biggest rows with an `others` bar for the rest; a trend is one group
// of bars for each period (many thin bars when there are many). View H has one chart for each
// section; views F and K have none. With a compare on, a second, lighter series draws the compared period.
//
// The measure the result names (PRD FR-18.10, plan 84) is what the chart draws: its title and the name of its
// series say it (sales, buyer total, quantity or a commission). A quantity is a count: its points are `number`
// cells, written without a rupee sign, so the picture can never show a count as money.

import type { AnalyticsChart } from '../../bindings/AnalyticsChart';
import type { AnalyticsChartPoint } from '../../bindings/AnalyticsChartPoint';
import type { AnalyticsKey } from '../../bindings/AnalyticsKey';
import type { AnalyticsPage } from '../../bindings/AnalyticsPage';
import type { AnalyticsView } from '../../bindings/AnalyticsView';
import type { ReportCell } from '../../bindings/ReportCell';
import { chartValueOfPaisa } from '../../shared/format';
import type { ListContext } from '../../shared/grid';
import type { BarChartData, BarSeries } from '../../shared/charts';
import { cellText, nameWithTag } from '../reports';
import { MEASURE_CHOICES, measureOfFilters, measureTitleId, type MeasureChoice } from './filters';
import { WHOLE_FILTER_KEY } from './keys';

/** A trend with more periods than this is drawn as many thin bars, without figures on them. */
const DENSE_AFTER = 12;

/** What a click on a bar or point opens: the lines behind it (PRD FR-18.2, FR-18.3). */
export interface ChartDrill {
  row: AnalyticsKey;
  column: AnalyticsKey | null;
  /** What the figure is, in words, for the title of the lines behind it. */
  about: string;
}

export interface ChartPlan {
  data: BarChartData;
  /** For each category, whether a click on it opens lines (the "others" bar and a month sum do not). */
  clickable: readonly boolean[];
  /** The lines behind the bar or point of this category and series, or null when it has none. */
  drillOf: (categoryIndex: number, seriesIndex: number | null) => ChartDrill | null;
  /** The message of what the chart shows, for a screen reader and the line above it. */
  labelId: string;
  /** The measure the chart draws: the variable of the message above (`$measure`). */
  measure: MeasureChoice;
  /** For a view with sections (H): the heading of the section the chart belongs to. */
  caption: string | null;
  /** How many rows the chart shows, and how many there are: it says so when it cut them. */
  shown: number;
  total: number;
}

/**
 * The compared figure of a point, or null. A page read before the compare was added has no such field,
 * and reads as no compare.
 */
function compareOf(point: AnalyticsChartPoint): ReportCell | null {
  return (point as { compare?: ReportCell | null }).compare ?? null;
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

/** What a chart needs of a column of its page: the name of a series and, on screen, the lines behind it. */
interface ChartColumn {
  key: string;
  title_id: string;
  drill?: AnalyticsKey | null;
}

/**
 * The name of a series: the heading of its column (the row total of a matrix names the measure),
 * else the words of the measure when the series is one (a table has no column for the buyer total of view A),
 * else its key.
 */
function seriesName(
  key: string,
  columns: readonly ChartColumn[],
  measure: MeasureChoice,
  context: ListContext,
): string {
  const column = columns.find((candidate) => candidate.key === key);
  if (column !== undefined) return context.text(measureTitleId(column.title_id, measure));
  const own = MEASURE_CHOICES.find((choice) => choice === key);
  return own === undefined ? key : context.text('analytics-measure-name', { measure: own });
}

/**
 * The message of what a chart shows. View M has two charts; the trend G draws five figures, or, with a measure, that
 * measure alone (its one series has the key of the measure), and then it is named for it (PRD FR-18.10).
 */
function labelIdOf(view: AnalyticsView, chart: AnalyticsChart, measure: MeasureChoice): string {
  if (view === 'M' && chart.series.some((one) => one.key === 'own')) {
    return 'analytics-chart-m-partner';
  }
  if (view === 'G' && chart.series.length === 1 && chart.series[0]?.key === measure) {
    return 'analytics-chart-g-measure';
  }
  return `analytics-chart-${view.toLowerCase()}`;
}

function planOf(
  view: AnalyticsView,
  columns: readonly ChartColumn[],
  chart: AnalyticsChart,
  context: ListContext,
  measure: MeasureChoice,
): ChartPlan | null {
  if (chart.points.length === 0 || chart.series.length === 0) return null;
  const series: BarSeries[] = chart.series.map((one, position) => ({
    name: seriesName(one.key, columns, measure, context),
    values: chart.points.map((point) => heightOf(point.values[position])),
    labels: chart.points.map((point) => {
      const cell = point.values[position];
      return cell === undefined ? '' : cellText(cell, context);
    }),
  }));
  // With a compare on (PRD FR-18.1, plan 76 point 4) each point also has the same figure in the compared
  // period: the lighter, dashed series next to the first measure, so it is never taken for the period.
  if (chart.points.some((point) => compareOf(point) !== null)) {
    series.push({
      name: context.text('analytics-col-compare'),
      values: chart.points.map((point) => {
        const compared = compareOf(point);
        return compared === null ? null : heightOf(compared);
      }),
      labels: chart.points.map((point) => {
        const compared = compareOf(point);
        return compared === null ? '' : cellText(compared, context);
      }),
      previous: true,
    });
  }
  const shown = chart.points.filter((point) => !point.is_others).length;
  // Views D and M (expenses by head, plan 77) draw one point for each period column, and the key of a point
  // is that column's key; every other chart has the key of the row of a point and the column of the series
  // that was clicked.
  const byColumn = view === 'D' || view === 'M';
  const categoryTexts = chart.points.map((point) =>
    point.is_others ? context.text('analytics-chart-others') : categoryText(point.label, context),
  );
  const drillOf = (categoryIndex: number, seriesIndex: number | null): ChartDrill | null => {
    const point = chart.points[categoryIndex];
    if (point === undefined || point.is_others || point.key === null) return null;
    // The compared figure is of another period: the lines behind it are not this view's lines.
    if (seriesIndex !== null && seriesIndex >= chart.series.length) return null;
    const seriesKey = chart.series[seriesIndex ?? 0]?.key;
    const column = columns.find((candidate) => candidate.key === seriesKey);
    const columnText =
      column === undefined ? '' : context.text(measureTitleId(column.title_id, measure));
    const about = [categoryTexts[categoryIndex] ?? '', byColumn ? '' : columnText]
      .filter((part) => part !== '')
      .join(' · ');
    return byColumn
      ? { row: WHOLE_FILTER_KEY, column: point.key, about }
      : { row: point.key, column: column?.drill ?? null, about };
  };
  return {
    clickable: chart.points.map((_, index) => drillOf(index, 0) !== null),
    drillOf,
    data: {
      kind: chart.kind === 'trend' ? 'line' : 'bar',
      categories: chart.points.map((point) =>
        point.is_others
          ? context.text('analytics-chart-others')
          : categoryText(point.label, context),
      ),
      series,
      stacked: false,
      dense: chart.kind === 'trend' && chart.points.length > DENSE_AFTER,
    },
    // View M has two charts: the shop's by period, and the partners' own kharch (series `own`), each with its words.
    labelId: labelIdOf(view, chart, measure),
    measure,
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
  // The page echoes the measure it was read for (PRD FR-18.10).
  const measure = measureOfFilters(page.filters);
  return page.charts.flatMap((chart) => {
    const plan = planOf(view, page.columns, chart, context, measure);
    return plan === null ? [] : [plan];
  });
}

/**
 * The charts of a print, as the bar chart draws them (PRD FR-18.3: the chart prints with the view). The print
 * data carries the same charts as the screen, over the whole filter, and the names of its columns; `context`
 * is in the print language, so the names and the figures are in the language of the paper. Nothing is worked
 * out here: the heights and the written figures are the backend's.
 */
export function printChartsOf(
  view: AnalyticsView,
  charts: readonly AnalyticsChart[],
  columns: readonly ChartColumn[],
  context: ListContext,
  measure: MeasureChoice,
): ChartPlan[] {
  return charts.flatMap((chart) => {
    const plan = planOf(view, columns, chart, context, measure);
    return plan === null ? [] : [plan];
  });
}
