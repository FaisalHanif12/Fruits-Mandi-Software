// The ECharts option of a bar chart, placed for the direction of the app (PRD FR-7.31, Tech Stack
// 4.3: ECharts has no automatic right-to-left mode, so the wrapper places everything itself).
//
// In Urdu (right to left) the first category is at the right edge; in English the first is at the left. The value axis shows no numbers of its own: ticks would be
// worked out by the chart from the heights, and a figure on a screen of money must come from the
// backend's amounts (formatPaisa), so each bar carries its written figure instead. No animation
// (NFR-USE-24: nothing longer than 200 ms).

import type { EChartsCoreOption } from 'echarts/core';
import type { BarChartData, BarSeries, ChartStyle } from './types';

/** The room kept above the bars for their written figures, in CSS pixels. */
const GRID_TOP = 40;
const GRID_SIDE = 8;
const GRID_BOTTOM = 8;
const BAR_MAX_WIDTH = 48;
/** The dashed outline of a scenario bar, in CSS pixels. */
const SCENARIO_OUTLINE = 2;
/** How strong the bars of the period before are drawn. */
const PREVIOUS_OPACITY = 0.45;
/** The thickness of a line of a trend chart, in CSS pixels. */
const LINE_WIDTH = 2;
const LABEL_SIZE = 14;
const AXIS_LABEL_SIZE = 14;
/** Names longer than this are cut with an ellipsis on the axis (the tooltip has the whole name). */
const AXIS_LABEL_WIDTH = 96;

/** How a chart is made for paper: no tooltip, patterns so a series reads in black and white. */
export interface BarOptionMode {
  print?: boolean | undefined;
  /** The categories a click does something on; the pointer shows only there. */
  clickable?: readonly boolean[] | undefined;
}

export function buildBarOption(
  data: BarChartData,
  style: ChartStyle,
  mode: BarOptionMode = {},
): EChartsCoreOption {
  const print = mode.print === true;
  const line = data.kind === 'line';
  const rtl = style.direction === 'rtl';
  // ECharts names the edges of a box `left` and `right`. What is wanted is the edge the reading starts
  // at and the one it ends at, so the two keys are chosen from the direction (CLAUDE.md rule 8).
  const near = rtl ? 'right' : 'left';
  const far = rtl ? 'left' : 'right';
  return {
    animation: false,
    textStyle: { fontFamily: style.fontFamily, color: style.ink },
    color: [...style.colors],
    // On paper the series are told apart by a pattern as well as a colour, so a black and white
    // printer keeps them apart (a chart is never read by colour alone, NFR-USE-12).
    ...(print ? { aria: { enabled: true, decal: { show: true } } } : {}),
    grid: {
      [near]: GRID_SIDE,
      [far]: GRID_SIDE,
      top: GRID_TOP,
      bottom: GRID_BOTTOM,
      containLabel: true,
    },
    // The legend is drawn by the component in HTML (BarChart.tsx), where text follows the direction
    // of the page by itself; ECharts' own legend measured Nastaliq names wrongly and overlapped them.
    legend: { show: false },
    tooltip: {
      show: !print,
      trigger: 'axis',
      axisPointer: { type: line ? 'line' : 'shadow' },
      backgroundColor: style.surface,
      borderColor: style.border,
      textStyle: { color: style.ink, fontFamily: style.fontFamily },
      // The text of a tooltip follows the direction, and each figure is isolated left to right.
      extraCssText: `direction: ${style.direction}; text-align: start;`,
      formatter: (raw: unknown) => {
        const items = Array.isArray(raw) ? (raw as TooltipItem[]) : [];
        const head = items[0]?.axisValueLabel ?? '';
        const lines = items.map((item) => {
          const label = data.series[item.seriesIndex]?.labels[item.dataIndex] ?? '';
          return `${item.marker}${escapeHtml(item.seriesName)}: <bdi dir="ltr">${escapeHtml(label)}</bdi>`;
        });
        return [`<bdi>${escapeHtml(head)}</bdi>`, ...lines].join('<br/>');
      },
    },
    xAxis: {
      type: 'category',
      data: [...data.categories],
      // Urdu reads from the right: the first category is at the right edge.
      inverse: rtl,
      axisTick: { show: false },
      axisLine: { lineStyle: { color: style.border } },
      // A click on the name under a bar is a click on the bar (it is easier to hit than a thin bar).
      triggerEvent: !print,
      axisLabel: {
        color: style.ink,
        fontFamily: style.fontFamily,
        fontSize: AXIS_LABEL_SIZE,
        width: AXIS_LABEL_WIDTH,
        overflow: 'truncate',
        // Every name is shown, unless the bars are many and thin: then only the names that fit.
        interval: data.dense === true ? 'auto' : 0,
      },
    },
    yAxis: {
      type: 'value',
      position: rtl ? 'right' : 'left',
      axisLabel: { show: false },
      axisTick: { show: false },
      axisLine: { show: false },
      splitLine: { lineStyle: { color: style.border } },
    },
    series: data.series.map((series, index) => {
      const color = style.colors[index % Math.max(style.colors.length, 1)] ?? style.ink;
      // A stack of parts, or many thin bars or points, have no figure on each one (they would not
      // fit): the tooltip and the table under the chart have them. A single bar shows its figure above it.
      const label =
        data.stacked || data.dense === true || (line && data.series.length > 1)
          ? { show: false }
          : {
              show: true,
              position: 'top' as const,
              color: style.ink,
              fontFamily: style.fontFamily,
              fontSize: LABEL_SIZE,
              formatter: (params: { data?: { label?: string } }) => params.data?.label ?? '',
            };
      const points = series.values.map((value, position) => ({
        value,
        // The written figure of this bar, carried with it so the label never has to be worked out.
        label: series.labels[position] ?? '',
        // The pointer shows only where a click opens something.
        ...(mode.clickable === undefined
          ? {}
          : { cursor: mode.clickable[position] === true ? 'pointer' : 'default' }),
      }));
      if (line) {
        return {
          type: 'line',
          name: series.name,
          emphasis: { focus: 'series' },
          symbol: 'circle',
          symbolSize: data.dense === true ? 4 : 8,
          showSymbol: true,
          lineStyle: {
            width: LINE_WIDTH,
            color,
            ...(series.previous === true ? { type: 'dashed' as const } : {}),
          },
          itemStyle: { color },
          data: points,
          label,
        };
      }
      return {
        type: 'bar',
        name: series.name,
        stack: data.stacked ? (series.stack ?? 'total') : undefined,
        barMaxWidth: BAR_MAX_WIDTH,
        emphasis: { focus: 'series' },
        data: points,
        label,
        itemStyle: itemStyleOf(series, color),
      };
    }),
  };
}

/** A bar in its colour; a scenario is an empty bar with a dashed outline in that colour. */
function itemStyleOf(series: BarSeries, color: string) {
  // The period before is the same colour, lighter.
  if (series.previous === true) return { color, opacity: PREVIOUS_OPACITY };
  if (series.scenario !== true) return { color };
  return {
    color: 'transparent',
    borderColor: color,
    borderWidth: SCENARIO_OUTLINE,
    borderType: 'dashed' as const,
  };
}

interface TooltipItem {
  axisValueLabel: string;
  seriesName: string;
  seriesIndex: number;
  dataIndex: number;
  marker: string;
}

function escapeHtml(text: string): string {
  return text
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;')
    .replaceAll('"', '&quot;');
}
