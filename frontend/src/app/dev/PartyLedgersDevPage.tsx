// Development page /dev/party-ledgers (plan 66): the party ledgers screen inside the app shell,
// answered by a stand-in for the backend with the backend's own fixtures of the worked day
// (tests/helpers/dev-bridge-party-ledgers.ts), so it can be opened in a plain browser tab, used, and
// photographed in both languages and at every window size. The address carries the level
// (`?side=seller`, `?party=20`, `?party=20&date=2026-10-10`); other flags choose the state (`empty`,
// `many`, `dead`, `down`). Never shipped: the route exists only when import.meta.env.DEV is true
// (app/router.tsx).

import { useState } from 'react';
import { installPartyLedgersDevBridge } from '../../../tests/helpers/dev-bridge-party-ledgers';
import { PartyLedgersPage } from '../../features/party_ledgers/pages/PartyLedgersPage';
import { AppShell } from '../layout/AppShell';
import { MainNav } from '../layout/MainNav';
import { ModuleMenu } from '../layout/ModuleMenu';
import { SAMPLE_USER } from './sample';

const SAMPLE_DATE = '10-10-2026';

export function PartyLedgersDevPage() {
  // The stand-in is installed before the screen asks its first question.
  useState(() => {
    installPartyLedgersDevBridge();
    return true;
  });
  return (
    <AppShell
      nav={<MainNav current="bikri" onSelect={() => undefined} onSignOut={() => undefined} />}
      moduleMenu={<ModuleMenu current="party-ledgers" onSelect={() => undefined} />}
      userName={SAMPLE_USER}
      onLock={() => undefined}
      onGuide={() => undefined}
      onOpenAccount={() => undefined}
      onOpenBikri={() => undefined}
      onOpenVoucher={() => undefined}
      date={SAMPLE_DATE}
      day="open"
    >
      <PartyLedgersPage />
    </AppShell>
  );
}
