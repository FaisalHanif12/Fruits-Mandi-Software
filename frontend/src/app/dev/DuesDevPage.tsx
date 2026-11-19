// Development pages /dev/dues/<page> (plan 79): the real pages behind the dashboard tiles inside the app shell, answered
// by a stand-in for the backend with the numbers of the plan (tests/helpers/dev-bridge-dues.ts), so they can be
// opened in a plain browser tab, used, and photographed in both languages and at every window size. The page is
// in the path: `sellers`, `buyers`, `total`, `lent`, `advances` or `cash`; the flags in the hash choose the state (`?empty`,
// `?down`, `?full`; see the bridge). Never shipped: the route exists only when import.meta.env.DEV is true
// (app/router.tsx).

import { useParams } from '@tanstack/react-router';
import { useState, type ReactNode } from 'react';
import { installDuesDevBridge } from '../../../tests/helpers/dev-bridge-dues';
import { BuyerDuesPage } from '../../features/dashboard/pages/BuyerDuesPage';
import { AdvancesOpenPage } from '../../features/dashboard/pages/AdvancesOpenPage';
import { CashSalesPage } from '../../features/dashboard/pages/CashSalesPage';
import { LentOpenPage } from '../../features/dashboard/pages/LentOpenPage';
import { SellerDuesPage } from '../../features/dashboard/pages/SellerDuesPage';
import { TotalDuesPage } from '../../features/dashboard/pages/TotalDuesPage';
import { useEscapeDispatcher } from '../../shared/keyboard';
import { AppShell } from '../layout/AppShell';
import { MainNav } from '../layout/MainNav';
import { SAMPLE_USER } from './sample';

const SAMPLE_DATE = '10-10-2026';

function pageOf(name: string): ReactNode {
  switch (name) {
    case 'buyers':
      return <BuyerDuesPage />;
    case 'total':
      return <TotalDuesPage />;
    case 'lent':
      return <LentOpenPage />;
    case 'advances':
      return <AdvancesOpenPage />;
    case 'cash':
      return <CashSalesPage />;
    default:
      return <SellerDuesPage />;
  }
}

export function DuesDevPage() {
  const { page } = useParams({ strict: false });
  // The real app mounts this in AppFrame, which the development routes sit outside of.
  useEscapeDispatcher();
  // The stand-in is installed before the screen asks its first question.
  useState(() => {
    installDuesDevBridge();
    return true;
  });
  return (
    <AppShell
      nav={<MainNav current="dashboard" onSelect={() => undefined} onSignOut={() => undefined} />}
      moduleMenu={null}
      userName={SAMPLE_USER}
      onLock={() => undefined}
      onGuide={() => undefined}
      onOpenAccount={() => undefined}
      onOpenBikri={() => undefined}
      onOpenVoucher={() => undefined}
      date={SAMPLE_DATE}
      day="open"
    >
      {pageOf(page ?? 'sellers')}
    </AppShell>
  );
}
