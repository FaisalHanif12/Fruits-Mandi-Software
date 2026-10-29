// Development page /dev/lists/<screen> (plan 38, A1): the lists and screens of the انڈکس area, the
// rules, the audit log and the partner setup inside the app shell, answered by a stand-in for the
// backend (tests/helpers/dev-bridge-lists.ts), so they can be opened in a plain browser tab and
// photographed in both languages at every window size. `<screen>` is one of `accounts`, `openings`,
// `chart`, `rules`, `audit`, `partners`, `menu`, or the path of a master (`cities`, `items`,
// `packings`...). Flags in the hash: `?many` (340 rows, to see the page buttons), `&empty`, `&down`.
// Never shipped: the route exists only when import.meta.env.DEV is true (app/router.tsx).

import { useParams } from '@tanstack/react-router';
import { useState, type ReactElement } from 'react';
import { installDevBridge } from '../../../tests/helpers/dev-bridge';
import { AuditLog } from '../../features/audit/pages/AuditPage';
import { AccountListPage } from '../../features/masters/pages/AccountListPage';
import { ChartPage } from '../../features/masters/pages/ChartPage';
import { MasterListPage } from '../../features/masters/pages/MasterListPage';
import { MastersMenuPage } from '../../features/masters/pages/MastersMenuPage';
import { OpeningsPage } from '../../features/masters/pages/OpeningsPage';
import { PartnerSetupPage } from '../../features/partners/pages/PartnerSetupPage';
import { RulesListPage } from '../../features/rules/pages/RulesListPage';
import { AppShell } from '../layout/AppShell';
import { MainNav } from '../layout/MainNav';
import { SAMPLE_USER } from './sample';

const SAMPLE_DATE = '10-10-2026';

const SCREENS: Record<string, () => ReactElement> = {
  accounts: () => <AccountListPage />,
  openings: () => <OpeningsPage />,
  chart: () => <ChartPage />,
  rules: () => <RulesListPage />,
  audit: () => <AuditLog />,
  partners: () => <PartnerSetupPage />,
  menu: () => <MastersMenuPage />,
};

export function ListsDevPage() {
  // The stand-in is installed before the screen asks its first question.
  useState(() => {
    installDevBridge();
    return true;
  });
  const { kind } = useParams({ strict: false });
  const own = SCREENS[kind ?? ''];
  return (
    <AppShell
      nav={<MainNav current="index" onSelect={() => undefined} />}
      moduleMenu={null}
      userName={SAMPLE_USER}
      onLock={() => undefined}
      onSignOut={() => undefined}
      onGuide={() => undefined}
      onOpenAccount={() => undefined}
      onOpenBikri={() => undefined}
      onOpenVoucher={() => undefined}
      date={SAMPLE_DATE}
      day="open"
    >
      {own === undefined ? <MasterListPage /> : own()}
    </AppShell>
  );
}
