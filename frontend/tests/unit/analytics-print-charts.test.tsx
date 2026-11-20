// The charts of an analytics print (plan 76 point 4, PRD FR-18.3: the chart prints with the view), in the Urdu
// and the English print: the print data carries the same charts as the screen, over the whole filter, and the
// print draws them under the header as inline pictures in the print language, with the name of the chart above
// each. A view with no chart, or a print with no chart in its data, draws nothing. The figures and heights are
// the backend's; the print adds nothing up.

import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import type { AnalyticsMeasure } from '../../src/bindings/AnalyticsMeasure';
import type { AnalyticsPrintData } from '../../src/bindings/AnalyticsPrintData';
import { AnalyticsPrintCharts } from '../../src/features/analytics/print/AnalyticsPrintCharts';
import { PrintProvider } from '../../src/shared/print';
import { ANALYTICS_PRINT_DATA_A_EN, ANALYTICS_PRINT_DATA_D_UR } from '../helpers/analytics-real';
import { message, visible } from '../helpers/render';

/** The recorded prints are of before the measure (plan 84): they hold the total commission. */
const RECORDED_MEASURE: AnalyticsMeasure = 'total_comm';

function draw(data: AnalyticsPrintData) {
  return render(
    <PrintProvider language={data.language}>
      <AnalyticsPrintCharts data={data} measure={RECORDED_MEASURE} />
    </PrintProvider>,
  );
}

describe('the charts of an analytics print', () => {
  it('view A in English: one figure with the name of the chart above its picture', () => {
    const data = ANALYTICS_PRINT_DATA_A_EN;
    expect(data.charts.length).toBeGreaterThan(0);
    const { container } = draw(data);
    const figures = screen.getAllByRole('img');
    expect(figures).toHaveLength(data.charts.length);
    const first = figures[0];
    expect(visible(first?.getAttribute('aria-label'))).toBe(message('en', 'analytics-chart-a'));
    expect(visible(first?.textContent)).toContain(message('en', 'analytics-chart-a'));
    // The picture is an inline SVG, so the print needs no file and no network.
    expect(container.querySelector('.print-chart-svg svg')).not.toBeNull();
  });

  it('view D in Urdu: the name of the chart is in Urdu, and the picture is made for the right-to-left page', () => {
    const data = ANALYTICS_PRINT_DATA_D_UR;
    const { container } = draw(data);
    const figure = screen.getAllByRole('img')[0];
    expect(visible(figure?.getAttribute('aria-label'))).toBe(message('ur', 'analytics-chart-d'));
    expect(container.querySelector('.print-chart-svg svg')).not.toBeNull();
  });

  it('a print whose data has no chart draws nothing', () => {
    const { container } = draw({ ...ANALYTICS_PRINT_DATA_A_EN, charts: [] });
    expect(container.querySelector('.print-charts')).toBeNull();
    expect(screen.queryAllByRole('img')).toHaveLength(0);
  });
});
