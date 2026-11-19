// The shared bar chart (plan 21, PRD FR-18.3, Tech Stack 3 and 4.3): it draws with ECharts as SVG from
// the app's own files, reads its colours and its font from the design tokens, places the axis and the
// legend for the direction of the app, and is made again when the language (and so the direction)
// changes. The drawing library is replaced by a stand-in here, so the test sees what the wrapper asks
// of it: the option, the size and the clean-up.

import { act, render } from '@testing-library/react';
import { LocalizationProvider, ReactLocalization } from '@fluent/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { bundles, LANGUAGES, type Language } from '../../src/i18n';
import { useLanguageStore } from '../../src/shared/i18n';
import type { BarChartData } from '../../src/shared/charts';
import { message } from '../helpers/render';

const chart = vi.hoisted(() => ({
  setOption: vi.fn(),
  on: vi.fn(),
  resize: vi.fn(),
  dispose: vi.fn(),
}));
const init = vi.hoisted(() => vi.fn());

vi.mock('echarts/core', () => ({ use: vi.fn(), init }));
vi.mock('echarts/charts', () => ({ BarChart: {}, LineChart: {} }));
vi.mock('echarts/components', () => ({
  AriaComponent: {},
  GridComponent: {},
  LegendComponent: {},
  TooltipComponent: {},
}));
vi.mock('echarts/renderers', () => ({ SVGRenderer: {} }));

// The component is imported after the mocks.
const { BarChart, buildBarOption, readChartStyle } = await import('../../src/shared/charts');

const DATA: BarChartData = {
  categories: ['Banana', 'Apple'],
  series: [{ name: 'Total commission', values: [392_800, 152_000], labels: ['3,928', '1,520'] }],
  stacked: false,
};

beforeEach(() => {
  vi.clearAllMocks();
  init.mockReturnValue(chart);
});

function show(language: Language, data: BarChartData = DATA) {
  act(() => {
    useLanguageStore.setState({ language });
  });
  const l10n = new ReactLocalization([bundles[language]]);
  return render(
    <LocalizationProvider l10n={l10n}>
      <BarChart data={data} label={message(language, 'analytics-chart-a')} />
    </LocalizationProvider>,
  );
}

describe.each(LANGUAGES)('the bar chart (%s)', (language: Language) => {
  it('is a picture with a label for a screen reader, in the direction of the app', () => {
    const view = show(language);
    const host = view.getByRole('img', { name: message(language, 'analytics-chart-a') });
    expect(host.getAttribute('data-chart')).toBe('bar');
    expect(host.getAttribute('dir')).toBe(language === 'ur' ? 'rtl' : 'ltr');
  });

  it('draws as SVG, at the size it has or a fallback, and hands the chart the option for its direction', () => {
    show(language);
    expect(init).toHaveBeenCalledTimes(1);
    const [, theme, options] = init.mock.calls[0] as [
      unknown,
      unknown,
      { renderer: string; width: number; height: number },
    ];
    expect(theme).toBeUndefined();
    expect(options).toMatchObject({ renderer: 'svg' });
    expect(options.width).toBeGreaterThan(0);
    expect(options.height).toBeGreaterThan(0);
    const option = chart.setOption.mock.calls[0]?.[0] as {
      xAxis: { inverse: boolean; data: string[] };
      series: { name: string; data: { value: number; label: string }[] }[];
    };
    expect(option.xAxis.inverse).toBe(language === 'ur');
    expect(option.xAxis.data).toEqual(['Banana', 'Apple']);
    // Each bar carries the figure the screen wrote, so the chart writes no amount of its own.
    expect(option.series[0]?.data.map((point) => point.label)).toEqual(['3,928', '1,520']);
  });

  it('writes the legend of several series as a list in HTML, and none for a single series', () => {
    const one = show(language);
    expect(one.container.querySelector('[data-chart-legend]')).toBeNull();
    one.unmount();
    const two = show(language, {
      ...DATA,
      series: [
        { name: 'Banana', values: [1, 2], labels: ['1', '2'] },
        { name: 'Apple', values: [3, 4], labels: ['3', '4'] },
      ],
      stacked: true,
    });
    const items = Array.from(two.container.querySelectorAll('[data-chart-legend] li'));
    expect(items.map((item) => item.textContent)).toEqual(['Banana', 'Apple']);
    expect(items[0]?.querySelector('span')?.className).toContain('bg-chart-1');
    expect(items[1]?.querySelector('span')?.className).toContain('bg-chart-2');
  });

  it('stacks by name: series with the same stack name are one stack, the others stand side by side', () => {
    show(language, {
      categories: ['Oct', 'Nov'],
      series: [
        { name: 'Past', values: [5, null], labels: ['5', ''], stack: 'past' },
        { name: 'Pace', values: [null, 6], labels: ['', '6'], stack: 'pace' },
        { name: 'Pace again', values: [null, 1], labels: ['', '1'], stack: 'pace' },
      ],
      stacked: true,
    });
    const option = chart.setOption.mock.calls[0]?.[0] as {
      series: { stack: string; data: { value: number | null }[] }[];
    };
    expect(option.series.map((series) => series.stack)).toEqual(['past', 'pace', 'pace']);
    // A month with no value has no bar at all, not a bar of height 0.
    expect(option.series[0]?.data.map((point) => point.value)).toEqual([5, null]);
  });

  it('draws a scenario as an empty bar with a dashed outline, and as a dashed box in the legend', () => {
    const view = show(language, {
      categories: ['Oct'],
      series: [
        { name: 'Pace', values: [4], labels: ['4'], stack: 'pace' },
        { name: 'If the pace is 200.00%', values: [8], labels: ['8'], stack: 'if', scenario: true },
      ],
      stacked: true,
    });
    const option = chart.setOption.mock.calls[0]?.[0] as {
      series: { itemStyle: { color: string; borderType?: string } }[];
    };
    expect(option.series[0]?.itemStyle.borderType).toBeUndefined();
    expect(option.series[1]?.itemStyle).toMatchObject({
      color: 'transparent',
      borderType: 'dashed',
    });
    const swatches = Array.from(view.container.querySelectorAll('[data-chart-legend] li > span'));
    expect(swatches[0]?.className).toContain('bg-chart-1');
    expect(swatches[1]?.className).toContain('border-dashed');
    expect(swatches[1]?.className).toContain('border-chart-2');
  });

  it('makes the drawing follow its box when the box changes, passing the size on purpose', () => {
    const resize = vi.fn<() => void>();
    const observe = vi.fn();
    class FakeObserver {
      constructor(callback: () => void) {
        resize.mockImplementation(callback);
      }
      observe = observe;
      disconnect = vi.fn();
    }
    vi.stubGlobal('ResizeObserver', FakeObserver);
    try {
      show(language);
      Object.defineProperty(HTMLElement.prototype, 'clientWidth', {
        configurable: true,
        value: 900,
      });
      Object.defineProperty(HTMLElement.prototype, 'clientHeight', {
        configurable: true,
        value: 300,
      });
      resize();
      expect(chart.resize).toHaveBeenCalledWith({ width: 900, height: 300 });
    } finally {
      vi.unstubAllGlobals();
      Reflect.deleteProperty(HTMLElement.prototype, 'clientWidth');
      Reflect.deleteProperty(HTMLElement.prototype, 'clientHeight');
    }
  });

  it('gives the drawing back when the screen goes away', () => {
    const view = show(language);
    view.unmount();
    expect(chart.dispose).toHaveBeenCalledTimes(1);
  });
});

describe('a language switch', () => {
  it('makes the chart again for the other direction, and disposes the old one', () => {
    const view = show('ur');
    expect(init).toHaveBeenCalledTimes(1);
    act(() => {
      useLanguageStore.setState({ language: 'en' });
    });
    view.rerender(
      <LocalizationProvider l10n={new ReactLocalization([bundles.en])}>
        <BarChart data={DATA} label={message('en', 'analytics-chart-a')} />
      </LocalizationProvider>,
    );
    expect(init).toHaveBeenCalledTimes(2);
    expect(chart.dispose).toHaveBeenCalledTimes(1);
    const option = chart.setOption.mock.calls[1]?.[0] as { xAxis: { inverse: boolean } };
    expect(option.xAxis.inverse).toBe(false);
  });
});

describe('the look comes from the design tokens', () => {
  it('reads a colour for each of the six series and the font of the language from the root element', () => {
    document.documentElement.style.setProperty('--font-urdu', 'Gulzar, serif');
    document.documentElement.style.setProperty('--font-latin', 'Inter, sans-serif');
    for (let index = 1; index <= 6; index += 1) {
      document.documentElement.style.setProperty(
        `--color-chart-${String(index)}`,
        `#00000${String(index)}`,
      );
    }
    const style = readChartStyle('rtl');
    expect(style.direction).toBe('rtl');
    expect(style.fontFamily).toBe('Gulzar, serif');
    // English reads the Latin font, whatever the page says about its own language yet.
    expect(readChartStyle('ltr').fontFamily).toBe('Inter, sans-serif');
    expect(style.colors).toHaveLength(6);
    expect(style.colors[0]).toBe('#000001');
    document.documentElement.removeAttribute('style');
  });
});

describe('a click on the chart (plan 76, PRD FR-18.3)', () => {
  const clickHandler = () => {
    const call = chart.on.mock.calls.find(([name]) => name === 'click');
    return call?.[1] as ((params: unknown) => void) | undefined;
  };

  it('tells the screen which bar was clicked, and which series, but only when it asks for it', () => {
    const onPointClick = vi.fn();
    act(() => {
      useLanguageStore.setState({ language: 'en' });
    });
    render(
      <LocalizationProvider l10n={new ReactLocalization([bundles.en])}>
        <BarChart
          data={DATA}
          label={message('en', 'analytics-chart-a')}
          onPointClick={onPointClick}
          clickable={[true, false]}
        />
      </LocalizationProvider>,
    );
    const handler = clickHandler();
    expect(handler).toBeDefined();
    handler?.({ componentType: 'series', seriesIndex: 0, dataIndex: 1 });
    expect(onPointClick).toHaveBeenLastCalledWith({ categoryIndex: 1, seriesIndex: 0 });
    // The name under a bar is a click on that bar, with no series.
    handler?.({ componentType: 'xAxis', value: 'Banana' });
    expect(onPointClick).toHaveBeenLastCalledWith({ categoryIndex: 0, seriesIndex: null });
    // Anything else on the chart (the grid, a name that is not there) does nothing.
    handler?.({ componentType: 'xAxis', value: 'Nothing' });
    handler?.({ componentType: 'grid' });
    expect(onPointClick).toHaveBeenCalledTimes(2);
    // The pointer shows only on the bar that opens something.
    const option = chart.setOption.mock.calls[0]?.[0] as {
      series: { data: { cursor?: string }[] }[];
    };
    expect(option.series[0]?.data.map((point) => point.cursor)).toEqual(['pointer', 'default']);
  });

  it('draws a trend as a line, a previous period lighter, and a print with patterns and no tooltip', () => {
    const style = readChartStyle('ltr');
    const trend: BarChartData = {
      kind: 'line',
      categories: ['Sep', 'Oct'],
      series: [
        { name: 'Now', values: [5, 8], labels: ['5', '8'] },
        { name: 'Before', values: [4, 6], labels: ['4', '6'], previous: true },
      ],
      stacked: false,
    };
    const screenOption = buildBarOption(trend, style) as {
      series: { type: string; lineStyle: { type?: string } }[];
      tooltip: { show: boolean; axisPointer: { type: string } };
      aria?: unknown;
    };
    expect(screenOption.series.map((one) => one.type)).toEqual(['line', 'line']);
    expect(screenOption.series[1]?.lineStyle.type).toBe('dashed');
    expect(screenOption.tooltip).toMatchObject({ show: true, axisPointer: { type: 'line' } });
    expect(screenOption.aria).toBeUndefined();
    const printOption = buildBarOption(DATA, style, { print: true }) as {
      tooltip: { show: boolean };
      aria: { enabled: boolean; decal: { show: boolean } };
    };
    expect(printOption.tooltip.show).toBe(false);
    expect(printOption.aria).toEqual({ enabled: true, decal: { show: true } });
    const bars = buildBarOption(
      { ...DATA, series: [{ name: 'Before', values: [1, 2], labels: ['1', '2'], previous: true }] },
      style,
    ) as { series: { itemStyle: { opacity?: number } }[] };
    expect(bars.series[0]?.itemStyle.opacity).toBeLessThan(1);
  });
});
