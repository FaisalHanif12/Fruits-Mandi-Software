// Development page /dev/kharch (plan 77): the real kharch page inside the app shell, answered by a
// stand-in for the backend with the backend's own answers of the analytics view M (tests/helpers/dev-bridge-kharch.ts), so it can be
// opened in a plain browser tab, used with a period and a partner, and photographed in both languages and at every
// window size. The flags in the hash choose the state (`?empty`, `?down`, `?locked`: the partner part asks for the code; see the bridge). Never
// shipped: the route exists only when import.meta.env.DEV is true (app/router.tsx).

import { useState } from 'react';
import { installKharchDevBridge } from '../../../tests/helpers/dev-bridge-kharch';
import { KharchPage } from '../../features/dashboard/pages/KharchPage';
import { useEscapeDispatcher } from '../../shared/keyboard';
import { AppShell } from '../layout/AppShell';
import { MainNav } from '../layout/MainNav';
import { SAMPLE_USER } from './sample';

const SAMPLE_DATE = '10-10-2026';

export function KharchDevPage() {
  // The real app mounts this in AppFrame, which the development routes sit outside of.
  useEscapeDispatcher();
  // The stand-in is installed before the screen asks its first question.
  useState(() => {
    installKharchDevBridge();
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
      <KharchPage />
    </AppShell>
  );
}
