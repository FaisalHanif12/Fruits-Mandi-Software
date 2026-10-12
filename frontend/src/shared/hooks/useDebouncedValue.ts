import { useEffect, useState } from 'react';

/**
 * A value that follows `value` only after it has stayed the same for `delayMs`. A live preview uses
 * it so a burst of key presses asks the backend once, 50 ms after the last key (Code Standards F35,
 * NFR-SPD-7).
 */
export function useDebouncedValue<T>(value: T, delayMs: number): T {
  const [settled, setSettled] = useState(value);
  useEffect(() => {
    const timer = window.setTimeout(() => {
      setSettled(value);
    }, delayMs);
    return () => {
      window.clearTimeout(timer);
    };
  }, [value, delayMs]);
  return settled;
}
