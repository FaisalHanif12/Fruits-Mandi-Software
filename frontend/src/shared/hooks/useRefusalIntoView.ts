import { useEffect, type RefObject } from 'react';

/**
 * When the backend refuses a save, the words that say why can be above the part of the screen the
 * user is looking at (he is in a grid, the problem list is above it). This brings the first of
 * them into view, by the shortest scroll, each time a NEW refusal arrives (PRD FR-7.6: a refusal
 * is never left unseen). It only scrolls: the focus stays where the user is typing.
 *
 * `root` holds the screen. A problem is a `role="alert"` message or a control marked
 * `aria-invalid` (the field shows its words just under it).
 */
export function useRefusalIntoView(root: RefObject<HTMLElement | null>, refusal: unknown): void {
  useEffect(() => {
    if (refusal === null || refusal === undefined) return;
    root.current
      ?.querySelector<HTMLElement>('[role="alert"], [aria-invalid="true"]')
      ?.scrollIntoView({ block: 'nearest' });
  }, [root, refusal]);
}
