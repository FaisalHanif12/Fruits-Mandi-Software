// The small chart of an answer (PRD FR-7.74, FR-7.86): a monthly series from the books, or an estimate
// with the months ahead. It goes through the shared chart wrapper, which places the axis and the
// legend for the direction of the app. The heights and the written figures come from the backend's
// cells (`chartValueOfPaisa` and `cellText`); the panel adds up nothing (CLAUDE.md rule 1). An
// estimate chart is stacked, so each month has one bar: the past in one colour and the months ahead
// in the colour of their method.

import { useLocalization } from '@fluent/react';
import type { AnswerChart } from '../../../bindings/AnswerChart';
import type { ChartSeriesKind } from '../../../bindings/ChartSeriesKind';
import type { ReportCell } from '../../../bindings/ReportCell';
import { BarChart, type BarChartData } from '../../../shared/charts';
import { chartValueOfPaisa } from '../../../shared/format';
import type { ListContext } from '../../../shared/grid';
import { useListContext } from '../../analytics';
import { cellText, monthText } from '../../reports';

/** More months than this and the figures are not written on the bars (the tooltip still has them). */
const DENSE_FROM_MONTHS = 8;

/**
 * The name of a line in the legend. A what-if says its factor ("if the pace is 200.00%"), which the
 * estimate band carries; the chart's own series has only the kind.
 */
function seriesName(kind: ChartSeriesKind, whatIfFactor: ReportCell | null, context: ListContext) {
  if (kind === 'what_if_pace') {
    return whatIfFactor === null
      ? context.text('assistant-estimate-what-if-any')
      : context.text('assistant-chart-what-if', { factor: cellText(whatIfFactor, context) });
  }
  return context.text(seriesMessageId(kind));
}

function seriesMessageId(kind: Exclude<ChartSeriesKind, 'what_if_pace'>): string {
  switch (kind) {
    case 'actual':
      return 'assistant-chart-past';
    case 'same_pace':
      return 'assistant-estimate-same-pace';
    case 'same_growth':
      return 'assistant-estimate-same-growth';
    case 'same_season':
      return 'assistant-estimate-same-season';
  }
}

/** The height of a bar, for the drawing only; a month with no value has no bar (null, not a flat bar). */
function heightOf(cell: ReportCell): number | null {
  switch (cell.type) {
    case 'money':
    case 'balance':
      return chartValueOfPaisa(cell.paisa);
    case 'number':
      return cell.value;
    default:
      return null;
  }
}

function dataOf(
  chart: AnswerChart,
  context: ListContext,
  whatIfFactor: ReportCell | null,
): BarChartData {
  return {
    categories: chart.months.map((month) => monthText(context.text, month.year, month.month)),
    series: chart.series.map((series) => ({
      name: seriesName(series.kind, whatIfFactor, context),
      values: series.values.map(heightOf),
      labels: series.values.map((cell) => cellText(cell, context)),
      // The methods of an estimate are alternatives for the same months, so each stands in its own stack
      // beside the others, never on top of them (their heights would add up to a figure nobody asked for).
      stack: series.kind === 'actual' ? 'past' : series.kind,
      scenario: series.kind === 'what_if_pace',
    })),
    stacked: chart.is_estimate,
    dense: chart.months.length > DENSE_FROM_MONTHS,
  };
}

interface AnswerChartViewProps {
  chart: AnswerChart;
  /** The factor of the what-if method of the answer's estimate, for the legend. Null when there is none. */
  whatIfFactor?: ReportCell | null;
}

export function AnswerChartView({ chart, whatIfFactor = null }: AnswerChartViewProps) {
  const { l10n } = useLocalization();
  const context = useListContext();
  // A measure this build has no words for shows the plain caption, never the raw id.
  const caption =
    l10n.getBundle(chart.measure_id) === null
      ? l10n.getString('assistant-chart-label')
      : l10n.getString(chart.measure_id);
  return (
    <figure className="flex flex-col gap-2 rounded-3 border border-border bg-surface p-3">
      <figcaption className="type-label text-ink">{caption}</figcaption>
      <BarChart data={dataOf(chart, context, whatIfFactor)} label={caption} />
    </figure>
  );
}
