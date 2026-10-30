// The partner overview (route /partners; PRD 17.3): ONE table, one row per active partner, with his
// کل رقم first and in bold, his share, profit and خالص, and "More columns" for the rest (plan 38).
// Every number comes from the backend (partners_overview). A button opens the setup page. Four
// states: loading, error with a retry, empty with a button, and the table. A row opens the
// partner's statement and capital.

import { Localized } from '@fluent/react';
import { useNavigate } from '@tanstack/react-router';
import { CalendarCheck, Landmark, Settings2, Users } from 'lucide-react';
import { uiErrorOf } from '../../../shared/api';
import { useRememberedFlag } from '../../../shared/hooks';
import { SHORTCUTS, useShortcut } from '../../../shared/keyboard';
import {
  Button,
  Checkbox,
  EmptyState,
  ErrorState,
  ListPage,
  LoadingState,
} from '../../../shared/ui';
import { OverviewTable } from '../components/OverviewTable';
import { usePartnerOverview } from '../hooks';

export function PartnersOverviewPage() {
  const navigate = useNavigate();
  const overview = usePartnerOverview();
  const [more, setMore] = useRememberedFlag('partners.overview_more', false);
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
    body = <OverviewTable rows={overview.data} more={more} />;
  }

  return (
    <ListPage
      titleId="partners-title"
      help={
        <Localized id="partners-overview-help">
          <p />
        </Localized>
      }
      actions={
        <>
          {/* The rest of the figures of every partner, under his row (plan 38). */}
          <Checkbox
            checked={more}
            onChange={setMore}
            label={<Localized id="common-more-columns" />}
          />
          <Button
            onClick={() => {
              void navigate({ to: '/partners/capital' });
            }}
          >
            <Landmark aria-hidden="true" className="size-icon-md" />
            <Localized id="partners-capital-open" />
          </Button>
          {/* The month close moves the month's income and expenses into the partners' capital (FR-17.6). */}
          <Button
            onClick={() => {
              void navigate({ to: '/month' });
            }}
          >
            <CalendarCheck aria-hidden="true" className="size-icon-md" />
            <Localized id="month-open" />
          </Button>
          <Button onClick={openSetup}>
            <Settings2 aria-hidden="true" className="size-icon-md" />
            <Localized id="partners-setup-open" />
          </Button>
        </>
      }
    >
      <div className="min-h-0 flex-1 overflow-auto">{body}</div>
    </ListPage>
  );
}
