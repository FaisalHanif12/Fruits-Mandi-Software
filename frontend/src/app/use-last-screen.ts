// The screen the Owner left the app on (PRD FR-7.45): the next time he signs in he starts there, not
// on the dashboard. Only the path is kept (never the typed data of a screen); the report and the
// list screens remember their own filters. A screen that needs a record (an account, a trip) or is
// a new, empty form is not offered again, because the record may be gone. Esc leaves a screen for
// the dashboard, and the dashboard is the start again next time.
//
// The path is kept the same way as the choices of a screen (`prefs_screen_*`), under the screen
// name `app.last-screen`. A failed read or save is never shown: the app simply opens on the
// dashboard.

import { useNavigate, useRouterState } from '@tanstack/react-router';
import { useEffect, useRef, useState } from 'react';
import { useScreenMemory } from '../shared/screen-state';

/** The name under which the last screen is kept. */
export const LAST_SCREEN = 'app.last-screen';

/** The screens that are never offered again: the hidden print window and anything made for one record. */
export function isRestorablePath(path: string): boolean {
  if (!path.startsWith('/') || path === '/' || path.startsWith('/print')) return false;
  // A record id in the path (/index/accounts/12) or a new, empty form (/trips/new).
  return !path.split('/').some((part) => /^\d+$/.test(part) || part === 'new');
}

/** Opens the screen the app was left on once, then keeps the path of the screen that is open. */
export function useLastScreen(active: boolean): void {
  const navigate = useNavigate();
  const pathname = useRouterState({ select: (state) => state.location.pathname });
  const memory = useScreenMemory(LAST_SCREEN);
  const decided = useRef(false);
  const [settled, setSettled] = useState(false);

  // At the start: if the Owner is still on the dashboard, go where he left off.
  useEffect(() => {
    if (!active || !memory.ready || decided.current) return;
    decided.current = true;
    const saved = typeof memory.saved === 'string' ? memory.saved : null;
    if (pathname === '/' && saved !== null && isRestorablePath(saved)) {
      void navigate({ to: saved });
    }
    setSettled(true);
  }, [active, memory.ready, memory.saved, pathname, navigate]);

  // After that: every screen he opens is the one to come back to.
  const { remember } = memory;
  useEffect(() => {
    if (!active || !settled) return;
    remember(isRestorablePath(pathname) ? pathname : null);
  }, [active, settled, pathname, remember]);
}
