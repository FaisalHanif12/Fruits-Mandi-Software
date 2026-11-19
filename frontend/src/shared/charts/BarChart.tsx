// A bar or line chart (Apache ECharts, Tech Stack 3), the one chart of the app (PRD FR-18.3, Code
// Standards F7). A click on a bar or point, or on the name under it, tells the screen which one
// (`onPointClick`), and the pointer shows only on the ones that open something (`clickable`); the
// table under the chart stays the keyboard path to the same lines. It is drawn as SVG from the app's own files: no remote script, no remote font (CSP,
// NFR-SEC-9). The wrapper reads the colours and the font from the design tokens when it draws, places
// the axis and the legend for the direction of the app (option.ts), and draws again when the
// language, and so the direction, changes. The table under a chart holds every figure, so the chart
// is a picture of it: it has a label for a screen reader and the figures are written on the bars. The
// legend of a chart with several series is a list in HTML above it.

import * as echarts from 'echarts/core';
import { BarChart as EBarChart, LineChart as ELineChart } from 'echarts/charts';
import {
  AriaComponent,
  GridComponent,
  LegendComponent,
  TooltipComponent,
} from 'echarts/components';
import { SVGRenderer } from 'echarts/renderers';
import { useEffect, useMemo, useRef } from 'react';
import { directionOf, useLanguage } from '../i18n';
import { buildBarOption } from './option';
import type { BarChartData, ChartPointClick, ChartStyle } from './types';

echarts.use([
  EBarChart,
  ELineChart,
  AriaComponent,
  GridComponent,
  LegendComponent,
  TooltipComponent,
  SVGRenderer,
]);

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

/** The swatch of a scenario in the legend: a dashed box in the colour of its bars. */
const SCENARIO_SWATCH_CLASSES = [
  'border-chart-1',
  'border-chart-2',
  'border-chart-3',
  'border-chart-4',
  'border-chart-5',
  'border-chart-6',
] as const;

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
  /** A click on a bar or point, or on the name under it. Without it the chart only shows. */
  onPointClick?: ((click: ChartPointClick) => void) | undefined;
  /** For each category, whether a click on it opens something (the pointer shows only there). */
  clickable?: readonly boolean[] | undefined;
}

/** What ECharts tells a click handler, as far as this chart reads it. */
interface ClickParams {
  componentType?: string;
  seriesIndex?: number;
  dataIndex?: number;
  value?: unknown;
}

export function BarChart({ data, label, onPointClick, clickable }: BarChartProps) {
  const language = useLanguage();
  const direction = directionOf(language);
  const host = useRef<HTMLDivElement>(null);
  const interactive = onPointClick !== undefined;
  const option = useMemo(
    () => buildBarOption(data, readChartStyle(direction), { clickable }),
    [data, direction, clickable],
  );
  // The handler is read when a click happens, so a new handler does not draw the chart again.
  const clickRef = useRef(onPointClick);
  useEffect(() => {
    clickRef.current = onPointClick;
  }, [onPointClick]);

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
    chart.on('click', (raw: unknown) => {
      const handler = clickRef.current;
      if (handler === undefined) return;
      const params = raw as ClickParams;
      if (params.componentType === 'series' && typeof params.dataIndex === 'number') {
        handler({ categoryIndex: params.dataIndex, seriesIndex: params.seriesIndex ?? 0 });
      } else if (params.componentType === 'xAxis') {
        // The name under a bar: its position in the categories is the position of the bar.
        const index = data.categories.indexOf(String(params.value));
        if (index >= 0) handler({ categoryIndex: index, seriesIndex: null });
      }
    });
    const watcher =
      typeof ResizeObserver === 'function'
        ? new ResizeObserver(() => {
            // The size is passed on purpose: a chart made with an explicit size keeps it on a plain
            // `resize()`, so a window made larger (the chat window's large size, a resized window)
            // would leave the chart at its first width.
            if (element.clientWidth > 0) {
              chart.resize({ width: element.clientWidth, height: element.clientHeight });
            }
          })
        : null;
    watcher?.observe(element);
    return () => {
      watcher?.disconnect();
      chart.dispose();
    };
  }, [option, data.categories]);

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
                className={
                  series.scenario === true
                    ? `size-icon-sm shrink-0 rounded-1 border-2 border-dashed ${SCENARIO_SWATCH_CLASSES[index % SCENARIO_SWATCH_CLASSES.length] ?? 'border-chart-1'}`
                    : `size-icon-sm shrink-0 rounded-1 ${SWATCH_CLASSES[index % SWATCH_CLASSES.length] ?? 'bg-chart-1'}`
                }
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
        data-chart={data.kind === 'line' ? 'line' : 'bar'}
        data-chart-interactive={interactive ? '' : undefined}
        dir={direction}
        className="h-chart w-full"
      />
    </div>
  );
}

/**
 * A chart as an SVG picture in a string, for paper (PRD FR-18.3: the chart prints with the view). It
 * is drawn without a page: ECharts' server mode makes the picture from the option alone, with
 * patterns so a black and white printer keeps the series apart. The picture is inline, so it needs no
 * file and no network and stays sharp at any size.
 */
export function chartToSvg(
  data: BarChartData,
  style: ChartStyle,
  size: { width: number; height: number },
): string {
  const chart = echarts.init(null, undefined, {
    renderer: 'svg',
    ssr: true,
    width: size.width,
    height: size.height,
  });
  try {
    chart.setOption(buildBarOption(data, style, { print: true }), true);
    return chart.renderToSVGString();
  } finally {
    chart.dispose();
  }
}
