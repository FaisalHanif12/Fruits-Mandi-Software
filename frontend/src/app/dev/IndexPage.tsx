// Development page /dev/index (plan 17e): the account form with its tag (نشان) field and the firm
// settings with the contact persons and the trade mark, inside the app shell, answered by a stand-in
// for the backend (tests/helpers/dev-bridge-index.ts), so they can be opened in a plain browser tab
// and photographed in both languages at every window size. Flags in the hash: `?screen=account|firm`,
// `&similar`, `&contacts`. Never shipped: the route exists only when import.meta.env.DEV is true
// (app/router.tsx).

import { useState } from 'react';
import { devFlag, devValue, installDevBridge } from '../../../tests/helpers/dev-bridge';
import { devFirm } from '../../../tests/helpers/dev-bridge-index';
import { AccountForm } from '../../features/masters/components/AccountForm';
import { FirmTab } from '../../features/settings/components/FirmTab';
import { AppShell } from '../layout/AppShell';
import { MainNav } from '../layout/MainNav';
import { SAMPLE_USER } from './sample';

const SAMPLE_DATE = '10-10-2026';

export function IndexPage() {
  // The stand-in is installed before the screen asks its first question.
  useState(() => {
    installDevBridge();
    return true;
  });
  const firm = devValue('screen') === 'firm';
  return (
    <AppShell
      nav={<MainNav current={firm ? 'settings' : 'index'} onSelect={() => undefined} />}
      moduleMenu={null}
      userName={SAMPLE_USER}
      onLock={() => undefined}
      onSignOut={() => undefined}
      onOpenAccount={() => undefined}
      onOpenBikri={() => undefined}
      onOpenVoucher={() => undefined}
      date={SAMPLE_DATE}
      day="open"
    >
      {firm ? (
        <FirmTab firm={devFirm(devFlag('contacts'))} />
      ) : (
        <AccountForm account={null} today="2026-10-10" />
      )}
    </AppShell>
  );
}
