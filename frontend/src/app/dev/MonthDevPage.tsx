// Development page /dev/month (round 7): the real month close screen inside the app shell, answered
// by a stand-in for the backend with the backend's own fixtures (tests/helpers/dev-bridge-month.ts),
// so it can be opened in a plain browser tab, used, and photographed in both languages and at every
// window size. The flags in the hash choose the state (`?closed`, `?daysopen`, `?empty`, `?down`,
// `?twice`; see the bridge). Never shipped: the route exists only when import.meta.env.DEV is true
// (app/router.tsx).

import { useState } from 'react';
import { installMonthDevBridge } from '../../../tests/helpers/dev-bridge-month';
import { MonthClosePage } from '../../features/month/pages/MonthClosePage';
import { AppShell } from '../layout/AppShell';
import { MainNav } from '../layout/MainNav';
import { SAMPLE_USER } from './sample';

const SAMPLE_DATE = '02-11-2026';

export function MonthDevPage() {
  // The stand-in is installed before the screen asks its first question.
  useState(() => {
    installMonthDevBridge();
    return true;
  });
  return (
    <AppShell
      nav={<MainNav current="partners" onSelect={() => undefined} />}
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
      <MonthClosePage />
    </AppShell>
  );
}
