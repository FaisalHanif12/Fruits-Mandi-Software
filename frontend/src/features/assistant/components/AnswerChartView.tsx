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

/** The name of a line in the legend. */
function seriesMessageId(kind: ChartSeriesKind): string {
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

/** The height of a bar, for the drawing only; a month with no value has no bar. */
function heightOf(cell: ReportCell): number {
  switch (cell.type) {
    case 'money':
    case 'balance':
      return chartValueOfPaisa(cell.paisa);
    case 'number':
      return cell.value;
    default:
      return 0;
  }
}

function dataOf(chart: AnswerChart, context: ListContext): BarChartData {
  return {
    categories: chart.months.map((month) => monthText(context.text, month.year, month.month)),
    series: chart.series.map((series) => ({
      name: context.text(seriesMessageId(series.kind)),
      values: series.values.map(heightOf),
      labels: series.values.map((cell) => cellText(cell, context)),
    })),
    stacked: chart.is_estimate,
    dense: chart.months.length > DENSE_FROM_MONTHS,
  };
}

export function AnswerChartView({ chart }: { chart: AnswerChart }) {
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
      <BarChart data={dataOf(chart, context)} label={caption} />
    </figure>
  );
}
