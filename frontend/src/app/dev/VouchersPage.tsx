// Development page /dev/vouchers (plan 15): the real voucher screens inside the app shell, answered by
// a stand-in for the backend with the books of the PRD worked day (tests/helpers/voucher-books.ts),
// so they can be opened in a plain browser tab and photographed in both languages and at every window
// size. The flags in the hash choose the screen: `?type=receipt|payment|advance` a new voucher of
// that type; `&id=501` a saved one; `?collection` the اگراہی فہرست. Never shipped: the route exists
// only when import.meta.env.DEV is true (app/router.tsx).

import { useState } from 'react';
import { devFlag, devValue, installDevBridge } from '../../../tests/helpers/dev-bridge';
import { CollectionPage } from '../../features/vouchers/pages/CollectionPage';
import { VoucherPage } from '../../features/vouchers/pages/VoucherPage';
import { asScreenType } from '../../features/vouchers/screen-type';
import { AppShell } from '../layout/AppShell';
import { MainNav } from '../layout/MainNav';
import { ModuleMenu } from '../layout/ModuleMenu';
import { SAMPLE_USER } from './sample';

const SAMPLE_DATE = '10-10-2026';

export function VouchersPage() {
  // The stand-in is installed before the screen asks its first question.
  useState(() => {
    installDevBridge();
    return true;
  });
  const type = asScreenType(devValue('type')) ?? 'receipt';
  return (
    <AppShell
      nav={<MainNav current="bikri" onSelect={() => undefined} />}
      moduleMenu={<ModuleMenu current={type} onSelect={() => undefined} />}
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
      {devFlag('collection') ? <CollectionPage /> : <VoucherPage />}
    </AppShell>
  );
}
