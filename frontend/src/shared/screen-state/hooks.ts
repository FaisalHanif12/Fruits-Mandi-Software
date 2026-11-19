// What a screen remembers, as hooks (PRD FR-7.45, plan 19). `useScreenMemory` reads the saved text of
// one screen once, and keeps it up to date as the screen saves. A save waits for the user to
// pause, and is sent when the screen closes, so leaving at once loses nothing. A failed read or
// save is never shown: the screen only loses a convenience.

import { useQuery, useQueryClient } from '@tanstack/react-query';
import { useCallback, useEffect, useMemo, useRef } from 'react';
import type { ScreenState } from '../../bindings/ScreenState';
import { prefsScreenGet, prefsScreenSet } from './api';
import { canonicalText, readRemembered, rememberedText } from './remember';

/** How long the choices rest before they are saved, in milliseconds. */
export const REMEMBER_SETTLE_MS = 500;

export const SCREEN_STATE_KEY = 'screen-state';

export interface ScreenMemory {
  /** True when the saved text has arrived, or could not be read: the screen can start. */
  ready: boolean;
  /** True when the saved text could not be read. The screen starts as new, but knows it does not know. */
  failed: boolean;
  /** What was saved, as the screen wrote it, or null when nothing is kept. */
  saved: unknown;
  /** Keeps a value after a short pause. Null forgets it. */
  remember: (value: unknown) => void;
}

export function useScreenMemory(screen: string): ScreenMemory {
  const queryClient = useQueryClient();
  const query = useQuery({
    queryKey: [SCREEN_STATE_KEY, screen],
    queryFn: () => prefsScreenGet({ screen }),
    // What this screen kept is read once while the app runs; later it is what this screen saved.
    staleTime: Infinity,
    gcTime: Infinity,
    retry: false,
  });
  const timer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const pending = useRef<string | null>(null);

  const send = useCallback(
    (text: string) => {
      queryClient.setQueryData<ScreenState>([SCREEN_STATE_KEY, screen], {
        screen,
        value: text === '' ? null : text,
      });
      void prefsScreenSet({ screen, value: text }).catch(() => undefined);
    },
    [queryClient, screen],
  );

  const flush = useCallback(() => {
    if (timer.current !== null) clearTimeout(timer.current);
    timer.current = null;
    const text = pending.current;
    pending.current = null;
    if (text === null) return;
    // What is kept now: what was read, or what this screen saved last. A value that has not changed
    // (the screen only wrote back what it read) is not sent again.
    const kept = queryClient.getQueryData<ScreenState>([SCREEN_STATE_KEY, screen])?.value ?? null;
    if (canonicalText(text) !== canonicalText(kept)) send(text);
  }, [queryClient, screen, send]);

  const remember = useCallback(
    (value: unknown) => {
      pending.current = rememberedText(value);
      if (timer.current !== null) clearTimeout(timer.current);
      timer.current = setTimeout(flush, REMEMBER_SETTLE_MS);
    },
    [flush],
  );

  // The screen closes: what is waiting is saved now.
  useEffect(() => flush, [flush]);

  // The same object while the saved text is the same, so a screen can depend on it.
  const text = query.data?.value ?? null;
  const saved = useMemo(() => readRemembered(text), [text]);

  return { ready: !query.isPending, failed: query.isError, saved, remember };
}
