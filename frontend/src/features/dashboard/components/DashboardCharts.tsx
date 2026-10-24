// The two mini charts of the dashboard (PRD 9.4) through the shared chart wrapper: the last 30 days
// of خام بکری and commission, and this month's commission by the top items. Each is a picture of
// figures the backend sent; the tiles and lists hold the same numbers. With nothing sold in the
// period a chart says so instead of drawing bars at zero (PRD FR-7.46).

import { Localized, useLocalization } from '@fluent/react';
import { ChartColumn } from 'lucide-react';
import { useMemo, type ReactNode } from 'react';
import type { DashboardChartDay } from '../../../bindings/DashboardChartDay';
import type { DashboardChartItem } from '../../../bindings/DashboardChartItem';
import { BarChart } from '../../../shared/charts';
import { formatPaisa, useGrouping } from '../../../shared/format';
import { useLanguage } from '../../../shared/i18n';
import { EmptyState } from '../../../shared/ui';
import { daysAreEmpty, daysChart, itemsChart, type ChartWords } from '../charts';

interface DashboardChartsProps {
  days: readonly DashboardChartDay[];
  items: readonly DashboardChartItem[];
}

function ChartCard({
  titleId,
  empty,
  chart,
}: {
  titleId: string;
  empty: boolean;
  chart: ReactNode;
}) {
  return (
    <figure className="flex min-w-0 flex-col gap-2 rounded-3 border border-border bg-surface p-4">
      <Localized id={titleId}>
        <figcaption className="type-section text-ink" />
      </Localized>
      {empty ? (
        <EmptyState
          icon={ChartColumn}
          title={<Localized id="dashboard-chart-empty-title" />}
          help={<Localized id="dashboard-chart-empty-help" />}
        />
      ) : (
        chart
      )}
    </figure>
  );
}

export function DashboardCharts({ days, items }: DashboardChartsProps) {
  const { l10n } = useLocalization();
  const language = useLanguage();
  const grouping = useGrouping();
  const words = useMemo<ChartWords>(
    () => ({
      language,
      text: (id) => l10n.getString(id),
      money: (amount) => formatPaisa(amount, { grouping }),
    }),
    [language, l10n, grouping],
  );
  const dayData = useMemo(() => daysChart(days, words), [days, words]);
  const itemData = useMemo(() => itemsChart(items, words), [items, words]);
  return (
    <div className="grid grid-cols-1 gap-4 normal:grid-cols-2">
      <ChartCard
        titleId="dashboard-chart-days-title"
        empty={daysAreEmpty(days)}
        chart={<BarChart data={dayData} label={l10n.getString('dashboard-chart-days-label')} />}
      />
      <ChartCard
        titleId="dashboard-chart-items-title"
        empty={items.length === 0}
        chart={<BarChart data={itemData} label={l10n.getString('dashboard-chart-items-label')} />}
      />
    </div>
  );
}
