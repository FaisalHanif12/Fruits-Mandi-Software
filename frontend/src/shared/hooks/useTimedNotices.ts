// Notes that show for a while and then hide, and show again when the user repeats the action that gives them
// (plan 95 items 1 and 2, PRD FR-12.52, Umar 23-11-2026). The same idea as the error bubble of a compact-row field
// (`FieldErrorBubble`): a note that stays for ever on a screen of hundreds of lines is noise, but it must never be lost.
//
// - A key that is new in `keys` shows at once, for `ms` milliseconds.
// - When `pulse` goes up (the user did the action again: a press, a try to save, a change in the lots) every key
//   that is in `keys` now shows again for `ms`.
// - A key that goes out of `keys` is forgotten, so it shows again when it comes back.
// - The words stay for a screen reader in the screen's own live region; this hook only says what is drawn.
//
// The timer belongs to an effect that depends on the stamps, so React's strict mode (an effect that runs twice in
// development) just starts it again: a note can never be left on screen with no timer behind it.

import { useEffect, useRef, useState } from 'react';
import { useLatest } from './useLatest';

/**
 * How long every short warning or error note of an entry screen stays on screen, in milliseconds (plan 97 item 4, PRD
 * FR-12.52, Umar 24-11-2026, Code Standards G8). The one value for the notes above the lots, the notes under the grids,
 * the "all sold" note and the error bubble of a field (`ERROR_BUBBLE_MS` is this value).
 */
export const NOTICE_MS = 5_000;

export function useTimedNotices(
  keys: readonly string[],
  pulse: number,
  ms: number,
): ReadonlySet<string> {
  // When each key was last shown, and "now" as the last timer saw it. A key is on screen while it is inside `ms`.
  const [stamps, setStamps] = useState<Readonly<Record<string, number>>>({});
  const [now, setNow] = useState(() => Date.now());
  const known = useRef<ReadonlySet<string>>(new Set());
  const lastPulse = useRef(pulse);
  const signature = keys.join('\u0000');
  const keysNow = useLatest(keys);

  // A new key, or the user repeated the action: show.
  useEffect(() => {
    // `keys` is read through its signature: a new array with the same keys is not a new key.
    const list = keysNow();
    const present = new Set(list);
    const fresh = list.filter((key) => !known.current.has(key));
    known.current = present;
    const again = pulse !== lastPulse.current;
    lastPulse.current = pulse;
    const reveal = again ? list : fresh;
    const at = Date.now();
    setStamps((before) => {
      // Keys that left are forgotten, so they show again when they come back.
      const next: Record<string, number> = {};
      for (const [key, stamp] of Object.entries(before)) {
        if (present.has(key)) next[key] = stamp;
      }
      for (const key of reveal) next[key] = at;
      return next;
    });
    if (reveal.length > 0) setNow(at);
  }, [signature, pulse, keysNow]);

  // One timer, for the key that is going to hide first. When it fires, "now" moves and the next one is set.
  useEffect(() => {
    const ends = Object.values(stamps)
      .map((stamp) => stamp + ms)
      .filter((end) => end > now);
    if (ends.length === 0) return undefined;
    const timer = window.setTimeout(
      () => {
        setNow(Date.now());
      },
      Math.max(Math.min(...ends) - Date.now(), 0) + 1,
    );
    return () => {
      window.clearTimeout(timer);
    };
  }, [stamps, now, ms]);

  const shown = new Set<string>();
  for (const key of keys) {
    const stamp = stamps[key];
    // A key that has no stamp yet is new: it shows at once, and the effect stamps it.
    if (stamp === undefined || stamp + ms > now) shown.add(key);
  }
  return shown;
}
