// The sign-in wait (PRD FR-8.2): after the fifth wrong try the backend answers ERR_LOCKED_WAIT with
// the seconds left, on every screen that checks a password (sign-in, recovery, unlock). This
// counts those seconds down by itself, shows the message with the live number, keeps the button
// off while the wait lasts, and drops the message when it is over.

import type { UiError } from '../../bindings/UiError';
import { useCountdown } from '../../shared/hooks';
import { lockSeconds } from './format';

interface LockCountdown {
  /** The error to show: the same, with the seconds counted down, or null once the wait is over. */
  shown: UiError | null;
  /** True while the user must still wait. */
  waiting: boolean;
}

/** `startKey` changes with every new try (the mutation's `submittedAt`), so a new lock restarts. */
export function useLockCountdown(error: UiError | null, startKey: unknown): LockCountdown {
  const wait = error === null ? null : lockSeconds(error.errors);
  const remaining = useCountdown(wait, startKey);
  if (error === null) return { shown: null, waiting: false };
  if (wait === null) return { shown: error, waiting: false };
  if (remaining === 0) return { shown: null, waiting: false };
  return {
    shown: {
      ...error,
      errors: error.errors.map((item) =>
        item.code === 'ERR_LOCKED_WAIT'
          ? { ...item, values: { ...item.values, seconds: remaining } }
          : item,
      ),
    },
    waiting: true,
  };
}
