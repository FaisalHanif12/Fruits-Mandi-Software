// Development page /dev/settings (plan 22): the real Settings screen inside the app shell, answered by
// a stand-in for the backend with the backend's own fixtures (tests/helpers/dev-bridge-settings.ts), so
// the new tabs can be opened in a plain browser tab and photographed in both languages and at every
// window size. The flags in the hash choose the tab and the state (see the bridge). Never shipped: the
// route exists only when import.meta.env.DEV is true (app/router.tsx).

import { useState } from 'react';
import { installSettingsDevBridge } from '../../../tests/helpers/dev-bridge-settings';
import { SettingsPage } from '../../features/settings/pages/SettingsPage';
import { useEscapeDispatcher } from '../../shared/keyboard';
import { AppShell } from '../layout/AppShell';
import { MainNav } from '../layout/MainNav';
import { SAMPLE_USER } from './sample';

const SAMPLE_DATE = '10-10-2026';

export function SettingsDevPage() {
  // The real app mounts this in AppFrame, which the development routes sit outside of.
  useEscapeDispatcher();
  // The stand-in is installed before the screen asks its first question.
  useState(() => {
    installSettingsDevBridge();
    return true;
  });
  return (
    <AppShell
      nav={<MainNav current="settings" onSelect={() => undefined} />}
      moduleMenu={null}
      userName={SAMPLE_USER}
      onLock={() => undefined}
      onGuide={() => undefined}
      onSignOut={() => undefined}
      onOpenAccount={() => undefined}
      onOpenBikri={() => undefined}
      onOpenVoucher={() => undefined}
      date={SAMPLE_DATE}
      day="open"
    >
      <SettingsPage />
    </AppShell>
  );
}
