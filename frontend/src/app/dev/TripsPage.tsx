// Development page /dev/trips (plan 16b): the real trip screens inside the app shell, answered by a
// stand-in for the backend with the worked numbers of E2E-34 (tests/helpers/trips-data.ts), so they
// can be opened in a plain browser tab and photographed in both languages and at every window size.
// The flags in the hash choose the screen: `?list` the list of trips (`&empty` for none), `?new` the
// new trip form, `?trip` one trip (`&closed` closed, `&cash` with little cash in hand, so the
// amber note shows). Never shipped: the route exists only when import.meta.env.DEV is true
// (app/router.tsx).

import { useMemo, useState } from 'react';
import { devFlag, installDevBridge } from '../../../tests/helpers/dev-bridge';
import { GoodsLinesGrid, LINE } from '../../features/purchases';
import { createEntryStore } from '../../shared/grid';
import { NewTripPage } from '../../features/trips/pages/NewTripPage';
import { TripScreen } from '../../features/trips/pages/TripPage';
import { TripsPage as TripsListPage } from '../../features/trips/pages/TripsPage';
import { AppShell } from '../layout/AppShell';
import { MainNav } from '../layout/MainNav';
import { SAMPLE_USER } from './sample';

const SAMPLE_DATE = '10-10-2026';

/** The item lines of a purchase entry, three lines typed (`?lines`, plan 35), for the screenshots. */
function LinesSample({ kind }: { kind: 'purchase' | 'arrival' }) {
  const store = useMemo(() => {
    const created = createEntryStore();
    const orange = { id: 61, code: '0044', name_ur: 'مالٹا', name_en: 'Orange' };
    const banana = { id: 62, code: '0045', name_ur: 'کیلا', name_en: 'Banana' };
    const box = { id: 71, code: '0050', name_ur: 'پیٹی', name_en: 'Box' };
    const first = created.getState().rows[0]?.rowId ?? '';
    created.getState().patchRow(first, {
      typed: { qty: '80', rate: '1,200', freight: '400', otherQty: '30', shop: 'Haji Shop' },
      picked: { item: orange, packing: box },
    });
    created.getState().addRow({ qty: '40', rate: '900' }, { item: banana, packing: box });
    return created;
  }, []);
  const rows = store((state) => state.rows);
  const calc = Object.fromEntries(
    rows.map((row, index) => [
      row.rowId,
      {
        [LINE.ourQty]: index === 0 ? '50' : index === 1 ? '40' : '',
        [LINE.value]: index === 0 ? '96,000' : index === 1 ? '36,000' : '',
      },
    ]),
  );
  return (
    <div className="flex max-w-page flex-col gap-4 p-4">
      <GoodsLinesGrid kind={kind} store={store} calc={calc} />
    </div>
  );
}

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
      onGuide={() => undefined}
      onSignOut={() => undefined}
      onOpenAccount={() => undefined}
      onOpenBikri={() => undefined}
      onOpenVoucher={() => undefined}
      date={SAMPLE_DATE}
      day="open"
    >
      {devFlag('lines') ? (
        <LinesSample kind={devFlag('arrival') ? 'arrival' : 'purchase'} />
      ) : devFlag('new') ? (
        <NewTripPage />
      ) : devFlag('list') ? (
        <TripsListPage />
      ) : (
        <TripScreen id={1} purchase={null} />
      )}
    </AppShell>
  );
}
