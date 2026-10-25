// The dashboard, the gardens tile and the attention bell are read again after any save (PRD FR-7.44, NFR-SPD-22): a
// saved بکری, voucher or entry changes the tiles and the counts, and the Owner should see that at
// once, on any screen (the bell is always on screen). Every successful command that writes goes
// through a mutation, so one place after the mutation cache is enough, and no screen has to remember
// to do it. Several saves close together are one read, 300 ms after the last one; the read is in the
// background and never slows typing.

import type { QueryClient } from '@tanstack/react-query';
import { ATTENTION_KEY, DASHBOARD_KEY } from '../features/dashboard';
import { GARDENS_SUMMARY_KEY } from '../features/gardens';

/** How long after the last save the lists are read again, in milliseconds. */
export const REFRESH_AFTER_SAVE_MS = 300;

/** The function to call after a successful mutation. One timer for the whole app. */
export function makeRefreshAfterSave(client: QueryClient): () => void {
  let timer: ReturnType<typeof setTimeout> | null = null;
  return () => {
    if (timer !== null) clearTimeout(timer);
    timer = setTimeout(() => {
      timer = null;
      void client.invalidateQueries({ queryKey: DASHBOARD_KEY });
      void client.invalidateQueries({ queryKey: ATTENTION_KEY });
      // The gardens tile asks for itself: a بکری of a garden's lot or a cancel changes its balance.
      void client.invalidateQueries({ queryKey: GARDENS_SUMMARY_KEY });
    }, REFRESH_AFTER_SAVE_MS);
  };
}
