// Development page /dev/dashboard (round 9): the real dashboard inside the app shell, with the real
// attention bell, answered by a stand-in for the backend with the backend's own fixtures
// (tests/helpers/dev-bridge-dashboard.ts), so it can be opened in a plain browser tab, used, and
// photographed in both languages and at every window size. The flags in the hash choose the state
// (`?empty`, `?many`, `?down`, `?update`, `?fresh`; see the bridge). Never shipped: the route exists
// only when import.meta.env.DEV is true (app/router.tsx).

import { useEffect, useState } from 'react';
import { installDashboardDevBridge } from '../../../tests/helpers/dev-bridge-dashboard';
import { devFlag } from '../../../tests/helpers/dev-bridge';
import { DashboardPage } from '../../features/dashboard/pages/DashboardPage';
import { AppShell } from '../layout/AppShell';
import { MainNav } from '../layout/MainNav';
import { GuideCard } from '../guide/GuideCard';
import { useGuideStore } from '../guide/guide-store';
import { SAMPLE_USER } from './sample';

const SAMPLE_DATE = '10-10-2026';

export function DashboardDevPage() {
  // The stand-in is installed before the screen asks its first question.
  useState(() => {
    installDashboardDevBridge();
    return true;
  });
  // `?guide` opens the first-run guide at its first step, to photograph it.
  useEffect(() => {
    if (devFlag('guide')) useGuideStore.getState().open(0);
    return () => {
      useGuideStore.getState().close();
    };
  }, []);
  return (
    <>
      <AppShell
        nav={<MainNav current="dashboard" onSelect={() => undefined} />}
        moduleMenu={null}
        userName={SAMPLE_USER}
        onLock={() => undefined}
        onGuide={() => {
          useGuideStore.getState().open(0);
        }}
        onSignOut={() => undefined}
        onOpenAccount={() => undefined}
        onOpenBikri={() => undefined}
        onOpenVoucher={() => undefined}
        date={SAMPLE_DATE}
        day="open"
      >
        <DashboardPage />
      </AppShell>
      <GuideCard />
    </>
  );
}
