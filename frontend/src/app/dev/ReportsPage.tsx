// Development pages /dev/reports and /dev/reports/R18 (plan 20): the real catalogue and the real
// report screen inside the app shell, answered by a stand-in for the backend with the backend's own
// fixtures (tests/helpers/dev-bridge-reports.ts), so they can be opened in a plain browser tab and
// photographed in both languages and at every window size. The flags in the hash choose the state
// (`?empty`, `?many`, `?down`, `?catalog`; see the bridge), and `?party=20` starts a ledger. Never
// shipped: the route exists only when import.meta.env.DEV is true (app/router.tsx).

import { useParams } from '@tanstack/react-router';
import { useState } from 'react';
import { installReportsDevBridge } from '../../../tests/helpers/dev-bridge-reports';
import { ReportPage } from '../../features/reports/pages/ReportPage';
import { ReportsHomePage } from '../../features/reports/pages/ReportsHomePage';
import { AppShell } from '../layout/AppShell';
import { MainNav } from '../layout/MainNav';
import { SAMPLE_USER } from './sample';

const SAMPLE_DATE = '10-10-2026';

export function ReportsPage() {
  // The stand-in is installed before the screen asks its first question.
  useState(() => {
    installReportsDevBridge();
    return true;
  });
  const { id } = useParams({ strict: false });
  return (
    <AppShell
      nav={<MainNav current="reports" onSelect={() => undefined} />}
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
      {id === undefined ? <ReportsHomePage /> : <ReportPage />}
    </AppShell>
  );
}
