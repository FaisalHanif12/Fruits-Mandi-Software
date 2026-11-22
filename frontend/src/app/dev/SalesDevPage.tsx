// Development page /dev/sales (plan 83): the real sales page inside the app shell, answered by a
// stand-in for the backend with the numbers of the plan (tests/helpers/dev-bridge-sales.ts), so it can be
// opened in a plain browser tab, used with a period and a party, and photographed in both languages and at every
// window size. The flags in the hash choose the state (`?empty`, `?down`; see the bridge). Never
// shipped: the route exists only when import.meta.env.DEV is true (app/router.tsx).

import { useState } from 'react';
import { installSalesDevBridge } from '../../../tests/helpers/dev-bridge-sales';
import { SalesPage } from '../../features/dashboard/pages/SalesPage';
import { useEscapeDispatcher } from '../../shared/keyboard';
import { AppShell } from '../layout/AppShell';
import { MainNav } from '../layout/MainNav';
import { SAMPLE_USER } from './sample';

const SAMPLE_DATE = '10-10-2026';

export function SalesDevPage() {
  // The real app mounts this in AppFrame, which the development routes sit outside of.
  useEscapeDispatcher();
  // The stand-in is installed before the screen asks its first question.
  useState(() => {
    installSalesDevBridge();
    return true;
  });
  return (
    <AppShell
      nav={<MainNav current="dashboard" onSelect={() => undefined} onSignOut={() => undefined} />}
      sectionTabs={null}
      userName={SAMPLE_USER}
      onLock={() => undefined}
      onGuide={() => undefined}
      onOpenAccount={() => undefined}
      onOpenBikri={() => undefined}
      onOpenVoucher={() => undefined}
      date={SAMPLE_DATE}
      day="open"
    >
      <SalesPage />
    </AppShell>
  );
}
