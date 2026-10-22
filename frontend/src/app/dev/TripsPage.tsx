// Development page /dev/trips (plan 16b): the real trip screens inside the app shell, answered by a
// stand-in for the backend with the worked numbers of E2E-34 (tests/helpers/trips-data.ts), so they
// can be opened in a plain browser tab and photographed in both languages and at every window size.
// The flags in the hash choose the screen: `?list` the list of trips (`&empty` for none), `?new` the
// new trip form, `?trip` one trip (`&closed` closed, `&cash` with little cash in hand, so the
// amber note shows). Never shipped: the route exists only when import.meta.env.DEV is true
// (app/router.tsx).

import { useState } from 'react';
import { devFlag, installDevBridge } from '../../../tests/helpers/dev-bridge';
import { NewTripPage } from '../../features/trips/pages/NewTripPage';
import { TripScreen } from '../../features/trips/pages/TripPage';
import { TripsPage as TripsListPage } from '../../features/trips/pages/TripsPage';
import { AppShell } from '../layout/AppShell';
import { MainNav } from '../layout/MainNav';
import { SAMPLE_USER } from './sample';

const SAMPLE_DATE = '10-10-2026';

export function TripsPage() {
  // The stand-in is installed before the screen asks its first question.
  useState(() => {
    installDevBridge();
    return true;
  });
  return (
    <AppShell
      nav={<MainNav current="trips" onSelect={() => undefined} />}
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
      {devFlag('new') ? (
        <NewTripPage />
      ) : devFlag('list') ? (
        <TripsListPage />
      ) : (
        <TripScreen id={1} purchase={null} />
      )}
    </AppShell>
  );
}
