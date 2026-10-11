import { useEffect, useState } from 'react';

const TICK_MS = 1_000;

/**
 * Counts whole seconds down to 0, once a second, from `seconds`. Starts again whenever `startKey`
 * changes (a new lock). Used for the sign-in wait (PRD FR-8.2): the backend says how many
 * seconds are left, and the screen counts them down by itself.
 */
export function useCountdown(seconds: number | null, startKey: unknown): number {
  const [remaining, setRemaining] = useState(seconds ?? 0);
  useEffect(() => {
    setRemaining(seconds ?? 0);
    if (seconds === null || seconds <= 0) return undefined;
    const timer = window.setInterval(() => {
      setRemaining((current) => Math.max(current - 1, 0));
    }, TICK_MS);
    return () => {
      window.clearInterval(timer);
    };
    // The countdown restarts for a new lock only.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [startKey]);
  return remaining;
}
