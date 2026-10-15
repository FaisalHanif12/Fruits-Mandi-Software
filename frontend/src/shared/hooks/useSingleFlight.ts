// One save at a time (NFR-REL-1). A save that is still on its way must not be started again by a
// second F10, a double Enter or a click on the button meanwhile: the backend has no request
// number, so two identical saves would make two records. The guard is a ref and not state, so it
// holds even when two key events arrive in the same instant, before the screen has redrawn.
//
//   const flight = useSingleFlight();
//   const save = () => flight.run(() => mutation.mutateAsync(draft), undefined);
//   <Button busy={flight.busy} onClick={save}>
//
// While a task runs, `busy` is true, and the Save button shows it is working and cannot be pressed.

import { useCallback, useRef, useState } from 'react';

export function useSingleFlight() {
  const running = useRef(false);
  const [busy, setBusy] = useState(false);

  /**
   * Runs `task` unless one is already running. When one is, nothing starts and `whenBusy` is the
   * answer, so a caller that wants a yes or no (did it save?) gets a plain "no".
   */
  const run = useCallback(<T>(task: () => Promise<T>, whenBusy: T): Promise<T> => {
    if (running.current) return Promise.resolve(whenBusy);
    running.current = true;
    setBusy(true);
    const finish = () => {
      running.current = false;
      setBusy(false);
    };
    let started: Promise<T>;
    try {
      started = task();
    } catch (error: unknown) {
      finish();
      return Promise.reject(error instanceof Error ? error : new Error('The task failed.'));
    }
    return started.finally(finish);
  }, []);

  return { run, busy };
}
