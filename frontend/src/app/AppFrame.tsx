// Everything the user reaches after the window opens: first the start-up data check (NFR-REL-7),
// then who is signed in, then the app and its screens.

import { Suspense, lazy } from 'react';
import { hasAppBridge } from '../shared/api';
import { useEscapeDispatcher } from '../shared/keyboard';
import { AuthGate } from './AuthGate';
import { DataCheckGate } from './data-check/DataCheckGate';

// Development only. In a plain browser tab there is no app window, so no command can answer: the
// page says so. In a build this constant is null and the branch below is removed.
const DevHome = import.meta.env.DEV
  ? lazy(() => import('./dev/DevHome').then((module) => ({ default: module.DevHome })))
  : null;

export function AppFrame() {
  // The one listener that sends Esc to the part that should answer it (PRD FR-7.18). It serves
  // every screen, signed in or not.
  useEscapeDispatcher();
  if (DevHome !== null && !hasAppBridge()) {
    return (
      <Suspense fallback={null}>
        <DevHome />
      </Suspense>
    );
  }
  return (
    <DataCheckGate>
      <AuthGate />
    </DataCheckGate>
  );
}
