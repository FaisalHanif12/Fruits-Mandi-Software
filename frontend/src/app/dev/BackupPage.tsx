// Development page /dev/backup (plan 18): the real backup screen inside the app shell, answered by a
// stand-in for the backend with the backend's own fixtures (tests/helpers/dev-bridge-backup.ts), so
// it can be opened in a plain browser tab and photographed in both languages and at every window
// size. The flags in the hash choose the state (`?old`, `?new`, `?key`, `?cloud`, `?empty`, `?many`,
// `?bad`, `?down`, `?lost`; see the bridge). Never shipped: the route exists only when
// import.meta.env.DEV is true (app/router.tsx).

import { useState } from 'react';
import { installBackupDevBridge } from '../../../tests/helpers/dev-bridge-backup';
import { BackupStatusItem } from '../../features/backup';
import { BackupPage as BackupScreen } from '../../features/backup/pages/BackupPage';
import { AppShell } from '../layout/AppShell';
import { MainNav } from '../layout/MainNav';
import { SAMPLE_USER } from './sample';

const SAMPLE_DATE = '10-10-2026';

export function BackupPage() {
  // The stand-in is installed before the screen asks its first question.
  useState(() => {
    installBackupDevBridge();
    return true;
  });
  return (
    <AppShell
      nav={<MainNav current="backup" onSelect={() => undefined} />}
      moduleMenu={null}
      userName={SAMPLE_USER}
      onLock={() => undefined}
      onSignOut={() => undefined}
      onOpenAccount={() => undefined}
      onOpenBikri={() => undefined}
      onOpenVoucher={() => undefined}
      date={SAMPLE_DATE}
      day="open"
      backup={<BackupStatusItem onOpen={() => undefined} />}
    >
      <BackupScreen />
    </AppShell>
  );
}
