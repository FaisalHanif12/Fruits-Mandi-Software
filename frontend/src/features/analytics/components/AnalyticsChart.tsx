// The charts above the table of a view (PRD FR-18.3): a simple picture of the whole filter, from the
// `charts` the backend sends with the page. What they draw is `chartsOf` (chart.ts); the table under
// them holds every figure. A chart says how many of the rows it shows when it cut them. A view with
// sections (H) has one chart for each section, named by the section.

import { Localized } from '@fluent/react';
import { useMemo } from 'react';
import { BarChart } from '../../../shared/charts';
import { useListContext } from '../list-context';
import { chartsOf } from '../chart';
import type { AnalyticsPage, AnalyticsView } from '../types';

interface AnalyticsChartProps {
  view: AnalyticsView;
  page: AnalyticsPage;
}

export function AnalyticsChart({ view, page }: AnalyticsChartProps) {
  const context = useListContext();
  const plans = useMemo(() => chartsOf(view, page, context), [view, page, context]);
  if (plans.length === 0) return null;
  return (
    <div className="flex flex-col gap-2">
      {plans.map((plan, index) => {
        const label = context.text(plan.labelId);
        return (
          <figure
            key={index}
            className="flex flex-col gap-2 rounded-3 border border-border bg-surface p-3"
          >
            <figcaption className="type-section text-ink">
              {plan.caption === null ? label : `${label} · ${plan.caption}`}
            </figcaption>
            <BarChart
              data={plan.data}
              label={plan.caption === null ? label : `${label} · ${plan.caption}`}
            />
            {plan.shown < plan.total ? (
              <Localized
                id="analytics-chart-limit"
                vars={{ shown: String(plan.shown), total: String(plan.total) }}
              >
                <p className="type-body text-ink-muted" />
              </Localized>
            ) : null}
          </figure>
        );
      })}
    </div>
  );
}
