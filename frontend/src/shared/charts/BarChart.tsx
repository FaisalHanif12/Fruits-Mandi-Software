// A bar chart (Apache ECharts, Tech Stack 3), the one chart of the app for now (PRD FR-18.3, Code
// Standards F7). It is drawn as SVG from the app's own files: no remote script, no remote font (CSP,
// NFR-SEC-9). The wrapper reads the colours and the font from the design tokens when it draws, places
// the axis and the legend for the direction of the app (option.ts), and draws again when the
// language, and so the direction, changes. The table under a chart holds every figure, so the chart
// is a picture of it: it has a label for a screen reader and the figures are written on the bars. The
// legend of a chart with several series is a list in HTML above it.

import * as echarts from 'echarts/core';
import { BarChart as EBarChart } from 'echarts/charts';
import { GridComponent, LegendComponent, TooltipComponent } from 'echarts/components';
import { SVGRenderer } from 'echarts/renderers';
import { useEffect, useMemo, useRef } from 'react';
import { directionOf, useLanguage } from '../i18n';
import { buildBarOption } from './option';
import type { BarChartData, ChartStyle } from './types';

echarts.use([EBarChart, GridComponent, LegendComponent, TooltipComponent, SVGRenderer]);

/** The size used when the element has none yet (a test, a window being built). */
const FALLBACK_WIDTH = 640;
const FALLBACK_HEIGHT = 224;
const CHART_TOKEN_COUNT = 6;

function token(name: string): string {
  return getComputedStyle(document.documentElement).getPropertyValue(name).trim();
}

/** The look of a chart from the design tokens, read when it is drawn. */
export function readChartStyle(direction: 'rtl' | 'ltr'): ChartStyle {
  return {
    direction,
    // The font of the language the chart is drawn for, read from the two language tokens. The
    // per-language `--font-text` is set on the page after the chart is drawn, so it is not read.
    fontFamily: token(direction === 'rtl' ? '--font-urdu' : '--font-latin'),
    colors: Array.from({ length: CHART_TOKEN_COUNT }, (_, index) =>
      token(`--color-chart-${String(index + 1)}`),
    ),
    ink: token('--color-ink'),
    inkMuted: token('--color-ink-muted'),
    border: token('--color-border'),
    surface: token('--color-surface'),
  };
}

/** The swatch of each series in the legend: the same six colours as the bars, from the tokens. */
const SWATCH_CLASSES = [
  'bg-chart-1',
  'bg-chart-2',
  'bg-chart-3',
  'bg-chart-4',
  'bg-chart-5',
  'bg-chart-6',
] as const;

interface BarChartProps {
  data: BarChartData;
  /** What the chart shows, in the app language, for a screen reader. */
  label: string;
}

export function BarChart({ data, label }: BarChartProps) {
  const language = useLanguage();
  const direction = directionOf(language);
  const host = useRef<HTMLDivElement>(null);
  const option = useMemo(() => buildBarOption(data, readChartStyle(direction)), [data, direction]);

  // The chart is made again when the direction changes: the axis and the legend are placed for it.
  useEffect(() => {
    const element = host.current;
    if (element === null) return undefined;
    const chart = echarts.init(element, undefined, {
      renderer: 'svg',
      width: element.clientWidth > 0 ? element.clientWidth : FALLBACK_WIDTH,
      height: element.clientHeight > 0 ? element.clientHeight : FALLBACK_HEIGHT,
    });
    chart.setOption(option, true);
    const watcher =
      typeof ResizeObserver === 'function'
        ? new ResizeObserver(() => {
            if (element.clientWidth > 0) chart.resize();
          })
        : null;
    watcher?.observe(element);
    return () => {
      watcher?.disconnect();
      chart.dispose();
    };
  }, [option]);

  return (
    <div className="flex flex-col gap-2" dir={direction}>
      {/* The legend is a list of the series, in HTML, so names in either script read in order. A single
          series needs none: the title says what it is. */}
      {data.series.length > 1 ? (
        <ul className="flex flex-wrap gap-x-4 gap-y-1" data-chart-legend>
          {data.series.map((series, index) => (
            <li
              key={`${series.name}-${String(index)}`}
              className="flex items-center gap-2 type-body"
            >
              <span
                aria-hidden="true"
                className={`size-icon-sm shrink-0 rounded-1 ${SWATCH_CLASSES[index % SWATCH_CLASSES.length] ?? 'bg-chart-1'}`}
              />
              <bdi dir="auto">{series.name}</bdi>
            </li>
          ))}
        </ul>
      ) : null}
      <div
        ref={host}
        role="img"
        aria-label={label}
        data-chart="bar"
        dir={direction}
        className="h-chart w-full"
      />
    </div>
  );
}
