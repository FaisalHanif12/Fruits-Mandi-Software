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
  resize: vi.fn(),
  dispose: vi.fn(),
}));
const init = vi.hoisted(() => vi.fn());

vi.mock('echarts/core', () => ({ use: vi.fn(), init }));
vi.mock('echarts/charts', () => ({ BarChart: {} }));
vi.mock('echarts/components', () => ({
  GridComponent: {},
  LegendComponent: {},
  TooltipComponent: {},
}));
vi.mock('echarts/renderers', () => ({ SVGRenderer: {} }));

// The component is imported after the mocks.
const { BarChart, readChartStyle } = await import('../../src/shared/charts');

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
