// The idle lock (PRD FR-8.4, NFR-SEC-3): after the set minutes with no key, click or mouse move,
// the app locks. Time is measured with the monotonic clock, so changing the computer's clock
// never locks early or late (NFR-REL-11). Nothing under the lock is lost: the screens stay where
// they are, only covered.

import { useEffect } from 'react';

const ACTIVITY_EVENTS = ['keydown', 'pointerdown', 'pointermove', 'wheel'] as const;
const CHECK_EVERY_MS = 1_000;
const MS_PER_MINUTE = 60_000;

/**
 * Calls `onIdle` once the user has done nothing for `minutes`. `enabled` is false while the app is
 * already locked, or when the setting is not known yet.
 */
export function useIdleLock(
  minutes: number | undefined,
  enabled: boolean,
  onIdle: () => void,
): void {
  useEffect(() => {
    if (!enabled || minutes === undefined || minutes <= 0) return undefined;
    let lastActivity = performance.now();
    const touch = () => {
      lastActivity = performance.now();
    };
    for (const name of ACTIVITY_EVENTS) {
      window.addEventListener(name, touch, { capture: true, passive: true });
    }
    const timer = window.setInterval(() => {
      if (performance.now() - lastActivity >= minutes * MS_PER_MINUTE) {
        lastActivity = performance.now();
        onIdle();
      }
    }, CHECK_EVERY_MS);
    return () => {
      for (const name of ACTIVITY_EVENTS) {
        window.removeEventListener(name, touch, { capture: true });
      }
      window.clearInterval(timer);
    };
  }, [minutes, enabled, onIdle]);
}
