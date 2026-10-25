// Development page /dev/gardens (plan 17c): the real garden screens inside the app shell, answered by
// a stand-in for the backend with the worked numbers of E2E-40 (tests/helpers/gardens-data.ts), so
// they can be opened in a plain browser tab and photographed in both languages and at every window
// size. The flags in the hash choose the screen: `?list` the list of gardens (`&empty` for none),
// `?new` the new garden form, `?overview` the partners overview with the open gardens under each
// card, else one garden (`&closed` closed, `&fresh` with nothing paid yet, `&cash` with little cash
// in hand, so the amber note shows). Never shipped: the route exists only when import.meta.env.DEV
// is true (app/router.tsx).

import { useState } from 'react';
import { devFlag, installDevBridge } from '../../../tests/helpers/dev-bridge';
import { GardenScreen } from '../../features/gardens/pages/GardenPage';
import { GardensPage as GardensListPage } from '../../features/gardens/pages/GardensPage';
import { NewGardenPage } from '../../features/gardens/pages/NewGardenPage';
import { PartnersOverviewPage } from '../../features/partners/pages/PartnersOverviewPage';
import { AppShell } from '../layout/AppShell';
import { MainNav } from '../layout/MainNav';
import { SAMPLE_USER } from './sample';

const SAMPLE_DATE = '10-10-2026';

export function GardensPage() {
  // The stand-in is installed before the screen asks its first question.
  useState(() => {
    installDevBridge();
    return true;
  });
  const overview = devFlag('overview');
  return (
    <AppShell
      nav={<MainNav current={overview ? 'partners' : 'gardens'} onSelect={() => undefined} />}
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
      {overview ? (
        <PartnersOverviewPage />
      ) : devFlag('new') ? (
        <NewGardenPage />
      ) : devFlag('list') ? (
        <GardensListPage />
      ) : (
        <GardenScreen id={1} />
      )}
    </AppShell>
  );
}
