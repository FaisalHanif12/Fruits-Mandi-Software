// The ECharts option of a bar chart, placed for the direction of the app (PRD FR-7.31, Tech Stack
// 4.3: ECharts has no automatic right-to-left mode, so the wrapper places everything itself).
//
// In Urdu (right to left) the first category is at the right edge; in English the first is at the left. The value axis shows no numbers of its own: ticks would be
// worked out by the chart from the heights, and a figure on a screen of money must come from the
// backend's amounts (formatPaisa), so each bar carries its written figure instead. No animation
// (NFR-USE-24: nothing longer than 200 ms).

import type { EChartsCoreOption } from 'echarts/core';
import type { BarChartData, ChartStyle } from './types';

/** The room kept above the bars for their written figures, in CSS pixels. */
const GRID_TOP = 40;
const GRID_SIDE = 8;
const GRID_BOTTOM = 8;
const BAR_MAX_WIDTH = 48;
const LABEL_SIZE = 14;
const AXIS_LABEL_SIZE = 14;
/** Names longer than this are cut with an ellipsis on the axis (the tooltip has the whole name). */
const AXIS_LABEL_WIDTH = 96;

export function buildBarOption(data: BarChartData, style: ChartStyle): EChartsCoreOption {
  const rtl = style.direction === 'rtl';
  // ECharts names the edges of a box `left` and `right`. What is wanted is the edge the reading starts
  // at and the one it ends at, so the two keys are chosen from the direction (CLAUDE.md rule 8).
  const near = rtl ? 'right' : 'left';
  const far = rtl ? 'left' : 'right';
  return {
    animation: false,
    textStyle: { fontFamily: style.fontFamily, color: style.ink },
    color: [...style.colors],
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
      trigger: 'axis',
      axisPointer: { type: 'shadow' },
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
    series: data.series.map((series, index) => ({
      type: 'bar',
      name: series.name,
      stack: data.stacked ? 'total' : undefined,
      barMaxWidth: BAR_MAX_WIDTH,
      emphasis: { focus: 'series' },
      data: series.values.map((value, position) => ({
        value,
        // The written figure of this bar, carried with it so the label never has to be worked out.
        label: series.labels[position] ?? '',
      })),
      // A stack of parts, or many thin bars, have no figure on each bar (they would not fit): the
      // tooltip and the table under the chart have them. A single bar shows its figure above it.
      label:
        data.stacked || data.dense === true
          ? { show: false }
          : {
              show: true,
              position: 'top',
              color: style.ink,
              fontFamily: style.fontFamily,
              fontSize: LABEL_SIZE,
              formatter: (params: { data?: { label?: string } }) => params.data?.label ?? '',
            },
      itemStyle: { color: style.colors[index % Math.max(style.colors.length, 1)] ?? style.ink },
    })),
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
