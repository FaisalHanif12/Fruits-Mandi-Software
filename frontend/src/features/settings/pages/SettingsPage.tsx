// Settings (PRD Section 21): firm, charges, checks, printing, language and text, auto lock, the day and
// the books check, the audit log, and the app (updates, support file). The Owner is the only user, so
// he sees and changes everything (PRD Section 2). The four states: it loads, it fails with a message and a retry, and it has
// data. Unsaved text in a tab is guarded by LeaveGuard when the user leaves.

import { Localized, useLocalization } from '@fluent/react';
import { useSearch } from '@tanstack/react-router';
import { useState, type ReactNode } from 'react';
import { uiErrorOf } from '../../../shared/api';
import { ErrorState, LoadingState, Tabs } from '../../../shared/ui';
import { AuditLog } from '../../audit';
import { AppTab } from '../components/AppTab';
import { ChargesTab } from '../components/ChargesTab';
import { DayTab } from '../components/DayTab';
import { FirmTab } from '../components/FirmTab';
import { LanguageTab } from '../components/LanguageTab';
import { LockTab } from '../components/LockTab';
import { OverridesTab } from '../components/OverridesTab';
import { PrintingTab } from '../components/PrintingTab';
import { useSettings } from '../hooks';

/** The tabs a link may open (`/settings?tab=charges`). */
const TAB_IDS = [
  'firm',
  'charges',
  'overrides',
  'printing',
  'language',
  'lock',
  'day',
  'audit',
  'app',
];

export function SettingsPage() {
  const { l10n } = useLocalization();
  const settings = useSettings();
  const requested = useSearch({ strict: false }).tab;
  const [tab, setTab] = useState(
    requested !== undefined && TAB_IDS.includes(requested) ? requested : 'firm',
  );

  if (settings.isPending) return <LoadingState />;
  if (settings.isError) {
    return (
      <ErrorState
        title={<Localized id="settings-load-failed" />}
        error={uiErrorOf(settings.error)}
        onRetry={() => {
          void settings.refetch();
        }}
      />
    );
  }
  const data = settings.data;
  // A panel of fields scrolls inside its own area; the audit log fills the room itself (plan 38).
  const scrolls = (content: ReactNode) => (
    <div className="min-h-0 flex-1 overflow-auto">{content}</div>
  );
  return (
    <div className="flex h-full min-h-0 flex-col gap-3">
      <Localized id="settings-title">
        <h1 className="type-title" />
      </Localized>
      {/* ONE row of tabs: the ones that do not fit are behind its More menu. */}
      <Tabs
        overflow
        label={l10n.getString('settings-tabs')}
        value={tab}
        onValueChange={setTab}
        items={[
          {
            id: 'firm',
            label: <Localized id="settings-tab-firm" />,
            content: scrolls(<FirmTab firm={data.firm} />),
          },
          {
            id: 'charges',
            label: <Localized id="settings-tab-charges" />,
            content: scrolls(<ChargesTab />),
          },
          {
            id: 'overrides',
            label: <Localized id="settings-tab-overrides" />,
            content: scrolls(<OverridesTab />),
          },
          {
            id: 'printing',
            label: <Localized id="settings-tab-printing" />,
            content: scrolls(<PrintingTab />),
          },
          {
            id: 'language',
            label: <Localized id="settings-tab-language" />,
            content: scrolls(
              <LanguageTab
                defaultLanguage={data.default_language}
                numberGrouping={data.number_grouping}
              />,
            ),
          },
          {
            id: 'lock',
            label: <Localized id="settings-tab-lock" />,
            content: scrolls(<LockTab minutes={data.idle_lock_minutes} />),
          },
          {
            id: 'day',
            label: <Localized id="settings-tab-day" />,
            content: scrolls(<DayTab />),
          },
          {
            id: 'audit',
            label: <Localized id="settings-tab-audit" />,
            content: <AuditLog />,
          },
          {
            id: 'app',
            label: <Localized id="settings-tab-app" />,
            content: scrolls(<AppTab />),
          },
        ]}
      />
    </div>
  );
}
