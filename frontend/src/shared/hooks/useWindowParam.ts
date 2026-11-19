// The window a list or statement shows (a month, or the last day of a window of days) kept in the ADDRESS of the
// screen, so a language switch, a reload and the way back from an entry land on the same window (plan 72
// P10, plan 74, plan 76). The screen gives the name of the search key and a check; a value that is not
// valid is read as "nothing asked", and the screen then opens on its latest window. The new window
// replaces the entry in the history, so Back leaves the screen and does not step through windows. The
// value is text as the backend sends it (`YYYY-MM` or `YYYY-MM-DD`); there is no money here.

import { useNavigate, useSearch } from '@tanstack/react-router';
import { useCallback } from 'react';

export function useWindowParam(
  key: string,
  isValid: (value: unknown) => value is string,
): readonly [string | null, (next: string | null) => void] {
  const navigate = useNavigate();
  const search: Record<string, unknown> = useSearch({ strict: false });
  const raw = search[key];
  const asked = isValid(raw) ? raw : null;
  const set = useCallback(
    (next: string | null) => {
      void navigate({
        to: '.',
        search: (previous: Record<string, unknown>) => {
          const rest = Object.fromEntries(
            Object.entries(previous).filter(([name]) => name !== key),
          );
          return next === null ? rest : { ...rest, [key]: next };
        },
        replace: true,
      });
    },
    [navigate, key],
  );
  return [asked, set];
}
