// The chart above the table of a view (PRD FR-18.3): a simple picture of the rows the screen has.
// What it draws is `chartFor` (chart.ts); the table under it holds every figure. It says how many of
// the rows it shows when it cut them.

import { Localized } from '@fluent/react';
import { useMemo } from 'react';
import { BarChart } from '../../../shared/charts';
import { useListContext } from '../list-context';
import { chartFor } from '../chart';
import type { AnalyticsPage, AnalyticsView } from '../types';

interface AnalyticsChartProps {
  view: AnalyticsView;
  page: AnalyticsPage;
}

export function AnalyticsChart({ view, page }: AnalyticsChartProps) {
  const context = useListContext();
  const plan = useMemo(() => chartFor(view, page, context), [view, page, context]);
  if (plan === null) return null;
  return (
    <figure className="flex flex-col gap-2 rounded-3 border border-border bg-surface p-3">
      <Localized id={plan.labelId}>
        <figcaption className="type-section text-ink" />
      </Localized>
      <BarChart data={plan.data} label={context.text(plan.labelId)} />
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
}
