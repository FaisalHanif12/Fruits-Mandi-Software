import { useEffect, useState } from 'react';

const TICK_MS = 1_000;

/**
 * Counts whole seconds down to 0, once a second, from `seconds`. Starts again whenever `seconds`
 * or `startKey` changes (a new lock). Used for the sign-in wait (PRD FR-8.2): the backend says
 * how many seconds are left, and the screen counts them down by itself.
 */
export function useCountdown(seconds: number | null, startKey: unknown): number {
  const [run, setRun] = useState({ startKey, seconds, elapsed: 0 });
  let current = run;
  if (run.startKey !== startKey || run.seconds !== seconds) {
    // A new lock: start counting again from the new number.
    current = { startKey, seconds, elapsed: 0 };
    setRun(current);
  }
  useEffect(() => {
    if (seconds === null || seconds <= 0) return undefined;
    const timer = window.setInterval(() => {
      setRun((previous) => ({ ...previous, elapsed: previous.elapsed + 1 }));
    }, TICK_MS);
    return () => {
      window.clearInterval(timer);
    };
  }, [seconds, startKey]);
  return seconds === null ? 0 : Math.max(seconds - current.elapsed, 0);
}
