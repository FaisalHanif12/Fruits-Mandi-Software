// The charts of an analytics print (PRD FR-18.3: the chart prints with the view). They are the charts the screen
// shows, over the whole filter, from the print data (`charts`), drawn in the print language: the names, the
// figures and the direction are those of the paper, not of the app. Each chart is an inline SVG made by
// ECharts' server mode (`chartToSvg`), with patterns so a black and white printer keeps the series apart,
// so it needs no file and no network (CSP) and stays sharp at any size. A view with no chart (or no rows)
// draws nothing. The heights and the written figures are the backend's; nothing is worked out here
// (CLAUDE.md rule 1).

import { useLocalization } from '@fluent/react';
import { useMemo } from 'react';
import type { AnalyticsPrintData } from '../../../bindings/AnalyticsPrintData';
import { chartToSvg, readChartStyle } from '../../../shared/charts';
import { formatPaisa, formatPaisaValue, useGrouping } from '../../../shared/format';
import type { ListContext } from '../../../shared/grid';
import { directionOf } from '../../../shared/i18n';
import { printChartsOf } from '../chart';

/** The size a chart is drawn at, in CSS pixels; the sheet scales it to its width. */
const CHART_WIDTH = 720;
const CHART_HEIGHT = 240;

export function AnalyticsPrintCharts({ data }: { data: AnalyticsPrintData }) {
  const { l10n } = useLocalization();
  const grouping = useGrouping();
  const figures = useMemo(() => {
    // The words and the figures are in the print language: this sits inside the print provider.
    const context: ListContext = {
      language: data.language,
      text: (id, vars) => l10n.getString(id, vars),
      money: (amount, plain = false) => formatPaisa(amount, { grouping, plain }),
      signedMoney: (value) => formatPaisaValue(value, { grouping, plain: false }),
    };
    const style = readChartStyle(directionOf(data.language));
    // A print read before the charts were added has no such field, and reads as no chart.
    const charts = (data as { charts?: AnalyticsPrintData['charts'] }).charts ?? [];
    return printChartsOf(data.view, charts, data.columns, context).map((plan) => ({
      label: l10n.getString(plan.labelId),
      caption: plan.caption,
      svg: chartToSvg(plan.data, style, { width: CHART_WIDTH, height: CHART_HEIGHT }),
    }));
  }, [data, l10n, grouping]);
  if (figures.length === 0) return null;
  return (
    <div className="print-charts">
      {figures.map((figure, index) => (
        <figure key={index} className="print-chart" role="img" aria-label={figure.label}>
          <figcaption className="print-chart-caption">
            {figure.caption === null ? figure.label : `${figure.label} · ${figure.caption}`}
          </figcaption>
          <div className="print-chart-svg" dangerouslySetInnerHTML={{ __html: figure.svg }} />
        </figure>
      ))}
    </div>
  );
}
