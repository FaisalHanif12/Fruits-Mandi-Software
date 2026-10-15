// The partner overview (route /partners; PRD 17.3): one card per active partner with his share
// and his income, expense, profit, money taken and capital. Every number comes from the backend
// (partners_overview). A button opens the setup page. Four states: loading, error with a retry,
// empty with a button, and the cards. The returned, خالص and کل رقم columns come in D4.

import { Localized } from '@fluent/react';
import { useNavigate } from '@tanstack/react-router';
import { Settings2, Users } from 'lucide-react';
import { uiErrorOf } from '../../../shared/api';
import { SHORTCUTS, useShortcut } from '../../../shared/keyboard';
import { Button, EmptyState, ErrorState, LoadingState } from '../../../shared/ui';
import { OverviewCard } from '../components/OverviewCard';
import { usePartnerOverview } from '../hooks';

export function PartnersOverviewPage() {
  const navigate = useNavigate();
  const overview = usePartnerOverview();
  const openSetup = () => {
    void navigate({ to: '/partners/setup' });
  };
  useShortcut(SHORTCUTS.newEntry, openSetup);

  let body;
  if (overview.isPending) {
    body = <LoadingState />;
  } else if (overview.isError) {
    body = (
      <ErrorState
        title={<Localized id="partners-load-failed" />}
        error={uiErrorOf(overview.error)}
        onRetry={() => {
          void overview.refetch();
        }}
      />
    );
  } else if (overview.data.length === 0) {
    body = (
      <EmptyState
        icon={Users}
        title={<Localized id="partners-overview-empty" />}
        help={<Localized id="partners-overview-empty-help" />}
        action={
          <Button variant="primary" shortcut={SHORTCUTS.newEntry} onClick={openSetup}>
            <Localized id="partners-overview-empty-action" />
          </Button>
        }
      />
    );
  } else {
    body = (
      <div className="flex flex-col gap-4">
        {overview.data.map((row) => (
          <OverviewCard key={row.partner_id} row={row} />
        ))}
      </div>
    );
  }

  return (
    <div className="flex flex-col gap-4">
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div className="flex flex-col gap-1">
          <Localized id="partners-title">
            <h1 className="type-title" />
          </Localized>
          <Localized id="partners-overview-help">
            <p className="type-body text-ink-muted" />
          </Localized>
        </div>
        <Button onClick={openSetup}>
          <Settings2 aria-hidden="true" className="size-icon-md" />
          <Localized id="partners-setup-open" />
        </Button>
      </div>
      {body}
    </div>
  );
}
