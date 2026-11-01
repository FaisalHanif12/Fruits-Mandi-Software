// "Reopen the day and fix" as one piece a screen can use (plan 44, PRD FR-21.4): `start` opens the
// box that asks for the reason; on confirm the days reopen and `then` runs (open the entry for
// editing). The box is `dialog`; the screen puts it anywhere in its tree. A refusal (a month that is
// month-closed, for example) stays inside the box, with what was typed.

import { useState, type ReactNode } from 'react';
import type { UiError } from '../../bindings/UiError';
import { uiErrorOf } from '../api';
import { FixDayDialog } from '../ui/FixDayDialog';
import { useFixDay } from './useFixDay';

interface FixStart {
  /** The day of the entry, `YYYY-MM-DD`. */
  date: string;
  /** The address of the screen where the entry is fixed. Leaving it closes the days again. */
  ownerHref: string;
  /** Runs when the days are open: opens the entry for editing. */
  then: () => void;
}

export function useFixDayFlow(): {
  start: (request: FixStart) => void;
  dialog: ReactNode;
  busy: boolean;
} {
  const fix = useFixDay();
  const [pending, setPending] = useState<FixStart | null>(null);
  const [error, setError] = useState<UiError | null>(null);

  const dialog = (
    <FixDayDialog
      open={pending !== null}
      onOpenChange={(open) => {
        if (!open) setPending(null);
      }}
      date={pending?.date ?? ''}
      error={error}
      busy={fix.busy}
      onConfirm={(reason) => {
        if (pending === null) return;
        const request = pending;
        fix
          .open(request.date, reason, request.ownerHref)
          .then(() => {
            setPending(null);
            request.then();
          })
          .catch((failure: unknown) => {
            setError(uiErrorOf(failure));
          });
      }}
    />
  );

  return {
    start: (request) => {
      setError(null);
      setPending(request);
    },
    dialog,
    busy: fix.busy,
  };
}
