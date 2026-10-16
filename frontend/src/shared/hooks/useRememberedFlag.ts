import { useCallback, useState } from 'react';

/** Every remembered choice of a screen starts with this, so they are easy to find and clear. */
const STORAGE_PREFIX = 'mandi.filters.';

function read(key: string): boolean | null {
  try {
    const stored = window.localStorage.getItem(STORAGE_PREFIX + key);
    return stored === null ? null : stored === '1';
  } catch {
    // Storage can be blocked or empty; the screen then starts from its default (never an error).
    return null;
  }
}

function write(key: string, value: boolean): void {
  try {
    window.localStorage.setItem(STORAGE_PREFIX + key, value ? '1' : '0');
  } catch {
    // Not saved this time. The choice still holds while the screen is open.
  }
}

/**
 * A tick of a screen that the user wants kept for next time, for example "more columns" on the
 * account list (PRD FR-7.45: filters are remembered). `key` is the name of the choice, written like
 * the `user_prefs` key it will move to: `accounts.more_columns`.
 *
 * It is kept on this computer. The preference table of the backend (`user_prefs`, Architecture
 * 15.3) has no `filters.<screen>` key yet, so this is the one place to change when it has. The
 * state is held here, above the grid, so a language switch that re-creates the grid loses nothing.
 */
export function useRememberedFlag(
  key: string,
  fallback: boolean,
): readonly [boolean, (value: boolean) => void] {
  const [value, setValue] = useState(() => read(key) ?? fallback);
  const change = useCallback(
    (next: boolean) => {
      setValue(next);
      write(key, next);
    },
    [key],
  );
  return [value, change];
}
