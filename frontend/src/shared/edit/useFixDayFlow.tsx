// "Reopen the day and fix" as one piece a screen can use (plan 44, PRD FR-21.4): `start` opens the
// box that asks for the reason; on confirm the days reopen and `then` runs (open the entry for
// editing). The box is `dialog`; the screen puts it anywhere in its tree. A refusal (a month that is
// month-closed, or a sign that is not the Owner's, for example) stays inside the box, with what was typed.
// The one answer that is not a refusal is ERR_DAY_NOT_CLOSED: the day is open already, so the box closes and
// the work goes on (plan 85).

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
      onConfirm={(reason, sign) => {
        if (pending === null) return;
        const request = pending;
        fix
          .open(request.date, reason, sign, request.ownerHref)
          .then(() => {
            setPending(null);
            request.then();
          })
          .catch((failure: unknown) => {
            const refused = uiErrorOf(failure);
            // The day is open already (someone reopened it): there is nothing to reopen, so the work goes on
            // with no session and no days to close again (plan 85, `day_fix_open` contract).
            if (refused.errors.some((item) => item.code === 'ERR_DAY_NOT_CLOSED')) {
              setPending(null);
              request.then();
              return;
            }
            setError(refused);
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
