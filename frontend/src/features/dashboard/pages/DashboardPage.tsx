// The dashboard after sign-in (PRD Section 9, FR-5.1, FR-9.1, FR-7.44, FR-7.46): a greeting and the
// date, a note when a new version is ready, the quick action buttons, the tiles (each opens the
// screen or report behind it), the partner strip, today's بکری, the oldest buyer dues, today's tasks
// (the attention list) and the two mini charts. Everything but the greeting comes from one backend
// call (`dashboard_get`), read again after every save. The backup and the books check tiles ask for
// themselves, so they still show when that call fails.
//
// Four states: while the call is on its way, a loading state; when it fails (the backend sends one
// error for the whole dashboard), an error with a retry in place of the figures; with nothing yet,
// each list says why and offers the next step; and the data. The figures are the backend's and
// nothing is worked out here (CLAUDE.md rule 1).

import { Localized } from '@fluent/react';
import { CircleCheck } from 'lucide-react';
import type { ReactNode } from 'react';
import { useDayStatus } from '../../../app/use-day-status';
import { uiErrorOf } from '../../../shared/api';
import { isoToDisplayDate } from '../../../shared/format';
import { formNavigationProps } from '../../../shared/keyboard';
import { EmptyState, ErrorState, LoadingState, Num, ScriptText } from '../../../shared/ui';
import { useAuthStatus } from '../../auth/hooks';
import { AttentionLines } from '../components/AttentionLines';
import { DashboardCharts } from '../components/DashboardCharts';
import { DashboardTiles } from '../components/DashboardTiles';
import { OpenGardens } from '../components/OpenGardens';
import { OldestDues } from '../components/OldestDues';
import { PartnerStrip } from '../components/PartnerStrip';
import { QuickActions } from '../components/QuickActions';
import { BackupTile, HealthTile } from '../components/StatusTiles';
import { TodayBikriList } from '../components/TodayBikriList';
import { UpdateNote } from '../components/UpdateNote';
import { useOpenDestination } from '../destinations';
import { useDashboard } from '../hooks';

function Section({ id, title, children }: { id: string; title: ReactNode; children: ReactNode }) {
  return (
    <section aria-labelledby={id} className="flex min-w-0 flex-col gap-3">
      <h2 id={id} className="type-section text-ink">
        {title}
      </h2>
      {children}
    </section>
  );
}

export function DashboardPage() {
  const status = useAuthStatus();
  const day = useDayStatus();
  const dashboard = useDashboard();
  const open = useOpenDestination();
  const name = status.data?.session?.display_name ?? '';
  const today = day.data?.today ?? '';
  const dayOpen = day.data?.today_open ?? true;
  const statusTiles = (
    <>
      <BackupTile />
      <HealthTile />
    </>
  );

  let body: ReactNode;
  if (dashboard.isPending) {
    body = <LoadingState rows={6} />;
  } else if (dashboard.isError) {
    body = (
      <>
        <ErrorState
          title={<Localized id="dashboard-load-failed" />}
          error={uiErrorOf(dashboard.error)}
          onRetry={() => {
            void dashboard.refetch();
          }}
        />
        <div
          {...formNavigationProps}
          className="grid grid-cols-3 gap-4 normal:grid-cols-4 wide:grid-cols-5"
        >
          {statusTiles}
        </div>
      </>
    );
  } else {
    const data = dashboard.data;
    body = (
      <>
        <Section id="dashboard-tiles-title" title={<Localized id="dashboard-tiles-title" />}>
          <DashboardTiles tiles={data.tiles}>{statusTiles}</DashboardTiles>
        </Section>
        <Section id="dashboard-partners-title" title={<Localized id="dashboard-partners-title" />}>
          <PartnerStrip partners={data.partners} />
        </Section>
        {data.gardens.length === 0 ? null : (
          <Section id="dashboard-gardens-title" title={<Localized id="dashboard-gardens-title" />}>
            <OpenGardens rows={data.gardens} />
          </Section>
        )}
        <Section id="dashboard-bikri-title" title={<Localized id="dashboard-bikri-title" />}>
          <TodayBikriList rows={data.bikri_today} count={data.bikri_today_count} />
        </Section>
        <div className="grid grid-cols-1 gap-6 normal:grid-cols-2">
          <Section id="dashboard-dues-title" title={<Localized id="dashboard-dues-title" />}>
            <OldestDues rows={data.oldest_dues} />
          </Section>
          <Section id="dashboard-tasks-title" title={<Localized id="dashboard-tasks-title" />}>
            {data.attention.items.length === 0 ? (
              <EmptyState
                icon={CircleCheck}
                title={<Localized id="dashboard-tasks-empty-title" />}
                help={<Localized id="dashboard-tasks-empty-help" />}
              />
            ) : (
              <AttentionLines items={data.attention.items} onOpen={open} />
            )}
          </Section>
        </div>
        <Section id="dashboard-charts-title" title={<Localized id="dashboard-charts-title" />}>
          <DashboardCharts days={data.chart_days} items={data.chart_items} />
        </Section>
      </>
    );
  }

  return (
    <div className="flex flex-col gap-6">
      <div className="flex flex-col gap-1">
        <Localized id="dashboard-greeting" vars={{ name }} elems={{ name: <ScriptText /> }}>
          <h1 className="type-title" />
        </Localized>
        <p className="type-body flex items-center gap-2 text-ink-muted">
          {today === '' ? null : (
            <Localized
              id="dashboard-today"
              vars={{ date: isoToDisplayDate(today) }}
              elems={{ num: <Num /> }}
            >
              <span />
            </Localized>
          )}
          {today === '' ? null : (
            <Localized id={dayOpen ? 'layout-status-day-open' : 'layout-status-day-closed'}>
              <span />
            </Localized>
          )}
        </p>
      </div>
      <UpdateNote />
      <Section id="dashboard-quick-title" title={<Localized id="dashboard-quick-title" />}>
        <QuickActions />
      </Section>
      {body}
    </div>
  );
}
