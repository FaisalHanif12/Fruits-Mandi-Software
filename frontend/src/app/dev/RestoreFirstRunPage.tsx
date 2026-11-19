// Development page /dev/restore (plan 56): the first screen of a new install and the restore pages,
// answered by a stand-in for the backend (tests/helpers/dev-bridge-restore.ts), so they can be
// opened in a plain browser tab and photographed in both languages and at every window size. The
// flags in the hash choose the state (see the bridge; `?after` shows the reminders of a restored
// computer after sign-in). Never shipped: the route exists only when
// import.meta.env.DEV is true (app/router.tsx).

import { useState } from 'react';
import {
  exposeProgress,
  installRestoreDevBridge,
  openRestoreAt,
  restoreAtFlag,
} from '../../../tests/helpers/dev-bridge-restore';
import { devFlag } from '../../../tests/helpers/dev-bridge';
import { useAfterRestoreStore } from '../../features/restore/after-restore-store';
import { AfterRestoreCard } from '../../features/restore';
import { LanguageSwitch } from '../layout/LanguageSwitch';
import { FirstRunGate } from '../FirstRunGate';

export function RestoreFirstRunPage() {
  // The stand-in is installed, and the flow opened at the page asked for, before the first draw.
  useState(() => {
    installRestoreDevBridge();
    exposeProgress();
    openRestoreAt(restoreAtFlag());
    // `?after`: the reminders shown once after the first sign-in on a restored computer.
    if (devFlag('after')) {
      useAfterRestoreStore.getState().remember({
        assistantWasOn: !devFlag('noai'),
        cloudNeedsKeys: devFlag('cloudkeys'),
      });
    }
    return true;
  });
  if (devFlag('after')) {
    return (
      <div className="min-h-screen bg-bg p-6">
        <LanguageSwitch />
        <AfterRestoreCard />
      </div>
    );
  }
  return <FirstRunGate />;
}
