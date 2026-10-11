// Settings (PRD Section 21): firm, language and text, auto lock. The Owner is the only user, so
// he sees and changes everything (PRD Section 2). The four states: it loads, it fails with a message and a retry, and it has
// data. Unsaved text in a tab is guarded by LeaveGuard when the user leaves.

import { Localized, useLocalization } from '@fluent/react';
import { useState } from 'react';
import { uiErrorOf } from '../../../shared/api';
import { ErrorState, LoadingState, Tabs } from '../../../shared/ui';
import { FirmTab } from '../components/FirmTab';
import { LanguageTab } from '../components/LanguageTab';
import { LockTab } from '../components/LockTab';
import { useSettings } from '../hooks';

export function SettingsPage() {
  const { l10n } = useLocalization();
  const settings = useSettings();
  const [tab, setTab] = useState('firm');

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
  return (
    <div className="flex flex-col gap-5">
      <Localized id="settings-title">
        <h1 className="type-title" />
      </Localized>
      <Tabs
        label={l10n.getString('settings-tabs')}
        value={tab}
        onValueChange={setTab}
        items={[
          {
            id: 'firm',
            label: <Localized id="settings-tab-firm" />,
            content: <FirmTab firm={data.firm} />,
          },
          {
            id: 'language',
            label: <Localized id="settings-tab-language" />,
            content: <LanguageTab defaultLanguage={data.default_language} />,
          },
          {
            id: 'lock',
            label: <Localized id="settings-tab-lock" />,
            content: <LockTab minutes={data.idle_lock_minutes} />,
          },
        ]}
      />
    </div>
  );
}
