import { useEffect, useRef } from 'react';

/**
 * A ref for the element that holds a screen's fields: the first field has focus when the screen
 * opens, so the user can start typing at once (PRD FR-7.1, NFR-USE-7). When `refocusKey` changes
 * (a wizard moves to its next step), the first field takes focus again.
 */
export function useFocusFirstField<T extends HTMLElement>(refocusKey?: unknown) {
  const ref = useRef<T>(null);
  useEffect(() => {
    ref.current
      ?.querySelector<HTMLElement>('input:not([type="hidden"]), select, textarea')
      ?.focus();
  }, [refocusKey]);
  return ref;
}
