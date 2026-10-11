// The dashboard after sign-in (PRD FR-5.1, FR-7.46, board ShellUr). For D1 it is the empty state
// the board draws: a greeting, today's date, and what to do first. The tiles and lists come with
// the modules that feed them (PRD Section 9).

import { Localized } from '@fluent/react';
import { Truck } from 'lucide-react';
import { useNavigate } from '@tanstack/react-router';
import { useDayStatus } from '../../../app/use-day-status';
import { useAuthStatus } from '../../auth/hooks';
import { isoToDisplayDate } from '../../../shared/format';
import { SHORTCUTS } from '../../../shared/keyboard';
import { Button, EmptyState, Num } from '../../../shared/ui';

export function DashboardPage() {
  const navigate = useNavigate();
  const status = useAuthStatus();
  const day = useDayStatus();
  const name = status.data?.session?.display_name ?? '';
  const today = day.data?.today ?? '';
  const dayOpen = day.data?.today_open ?? true;
  return (
    <div className="flex h-full flex-col gap-5">
      <div className="flex flex-col gap-1">
        <Localized id="dashboard-greeting" vars={{ name }}>
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
      <div className="flex flex-1 items-center justify-center">
        <EmptyState
          icon={Truck}
          title={<Localized id="dashboard-empty-title" />}
          help={<Localized id="dashboard-empty-help" />}
          action={
            <div className="flex flex-wrap justify-center gap-2">
              <Button
                variant="primary"
                onClick={() => {
                  void navigate({ to: '/index' });
                }}
              >
                <Localized id="dashboard-open-index" />
              </Button>
              <Button
                shortcut={SHORTCUTS.newBikri}
                onClick={() => {
                  void navigate({ to: '/bikri' });
                }}
              >
                <Localized id="dashboard-new-bikri" />
              </Button>
            </div>
          }
        />
      </div>
    </div>
  );
}
