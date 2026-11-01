// Reopen the days of a closed entry, fix it, close the days again (plan 44, PRD FR-21.4). The hook
// gives a screen the two steps: `open` before the entry is edited, `finish` after the save or when
// the user leaves. `finish` is safe to call twice: the first call takes the token, the second finds
// none. The backend decides which days close (the checks of FR-21.1); this only asks and tells.

import { Localized, useLocalization, type ReactLocalization } from '@fluent/react';
import { useCallback, useState } from 'react';
import type { DayFixClosed } from '../../bindings/DayFixClosed';
import type { DayFixOpened } from '../../bindings/DayFixOpened';
import { isoToDisplayDate } from '../format';
import { showToast } from '../ui/toast-store';
import { uiErrorOf } from '../api';
import { dayFixClose, dayFixOpen } from './api';
import { useFixStore } from './fix-store';

/** The dates as one line of text: `10-10-2026، 11-10-2026` (the separator is the language's own). */
export function daysText(days: readonly string[], l10n: ReactLocalization): string {
  return days.map(isoToDisplayDate).join(l10n.getString('edit-list-separator'));
}

/**
 * Closes the days of the session in the store. Safe to call from anywhere (the shell calls it
 * when the user leaves the screen). Answers null when there is nothing to close.
 */
export async function finishFixSession(l10n: ReactLocalization): Promise<DayFixClosed | null> {
  const { session, end, setStayedOpen } = useFixStore.getState();
  if (session === null) return null;
  // Take the token first, so a second call during the request finds nothing to close.
  end();
  try {
    const closed = await dayFixClose(session.token);
    setStayedOpen(closed.stayed_open);
    if (closed.closed.length > 0) {
      showToast(
        'success',
        <Localized id="edit-fix-closed-toast" vars={{ days: daysText(closed.closed, l10n) }}>
          <span />
        </Localized>,
      );
    }
    return closed;
  } catch (failure) {
    showToast('error', <Localized id="edit-fix-failed" />);
    // The days stay open; the attention list tells (PRD FR-21.4). The error itself is logged by
    // the backend with its reference.
    void uiErrorOf(failure);
    return null;
  }
}

export function useFixDay() {
  const session = useFixStore((state) => state.session);
  const begin = useFixStore((state) => state.begin);
  const { l10n } = useLocalization();
  const [busy, setBusy] = useState(false);

  /** Reopens the date and every later closed date. Throws what the backend refused. */
  const open = useCallback(
    async (date: string, reason: string, ownerHref: string): Promise<DayFixOpened> => {
      setBusy(true);
      try {
        const opened = await dayFixOpen(date, reason);
        begin({ token: opened.token, days: opened.days, reason, ownerHref });
        return opened;
      } finally {
        setBusy(false);
      }
    },
    [begin],
  );

  const finish = useCallback(() => finishFixSession(l10n), [l10n]);
  return { session, busy, open, finish };
}
