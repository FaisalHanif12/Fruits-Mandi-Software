// A seller's amount typed in his summary becomes the نقد of his lots (plan 91, PRD FR-12.23 v1.53). The screen asks the
// backend to spread the amount (`bikri_seller_paid_spread`: lot order, each up to its credit, the rest on his last lot)
// and writes each lot's answer into its نقد field, a value the app fills, visible like a typed one and still editable
// (NFR-USE-27, Code Standards F43). The lots' نقد are then the draft's truth, so a language switch, the autosaved draft
// and the save lose nothing (F39). The screen splits no money.
//
// The call waits for a pause in typing, and an answer for an older amount never replaces the one for a newer amount.

import { useCallback, useEffect, useRef, useState } from 'react';
import { uiErrorOf } from '../../../shared/api';
import type { UiError } from '../../../bindings/UiError';
import { formatPaisa, type Paisa } from '../../../shared/format';
import { bikriSellerPaidSpread } from '../api';
import { buildDraft, plainRupees } from '../draft';
import type { BikriDraftStores } from '../draft-store';
import { LOT } from '../types';

/** The pause after the last key before the amount is spread (the live preview waits as long, NFR-SPD-7). */
export const SPREAD_DELAY_MS = 50;

/** The نقد as the text a user would type: nothing for 0, so a lot with no cash shows an empty cell. */
export function cashText(paisa: Paisa): string {
  return formatPaisa(paisa, { grouping: 'international' }) === '0' ? '' : plainRupees(paisa);
}

export interface SellerPaidSpreading {
  /** Spreads what was typed in a seller's summary. Empty text means 0: the lots' نقد are cleared. */
  spread: (sellerId: number, text: string) => void;
  /** What the backend refused in an amount, by seller (not a number, decimals, a sum below 0). */
  errors: Readonly<Record<number, UiError>>;
}

/**
 * @param entryId the posting being edited (plan 16), or null for a new بکری: the backend leaves that posting out of
 *   each seller's previous balance.
 */
export function useSellerPaidSpread(
  stores: BikriDraftStores,
  entryId: number | null,
): SellerPaidSpreading {
  const [errors, setErrors] = useState<Readonly<Record<number, UiError>>>({});
  // The newest request of each seller: an older answer is dropped.
  const latest = useRef(new Map<number, number>());
  const timers = useRef(new Map<number, number>());
  useEffect(
    () => () => {
      for (const timer of timers.current.values()) window.clearTimeout(timer);
    },
    [],
  );

  const run = useCallback(
    async (sellerId: number, text: string, request: number) => {
      // The new lots only: a mixed screen's carried lots keep their own نقد (plan 91 point 5).
      const { header, lots } = buildDraft(stores);
      try {
        const answer = await bikriSellerPaidSpread({
          draft: { header, lots },
          entry_id: entryId,
          seller_paid: [{ seller_account_id: sellerId, amount: text.trim() === '' ? '0' : text }],
        });
        if (latest.current.get(sellerId) !== request) return;
        setErrors((known) => {
          const { [sellerId]: _dropped, ...rest } = known;
          return rest;
        });
        const lotsStore = stores.lots.getState();
        for (const lot of answer.lots) {
          lotsStore.patchRow(lot.key, { typed: { [LOT.cash]: cashText(lot.cash_paid_paisa) } });
        }
      } catch (failure: unknown) {
        if (latest.current.get(sellerId) !== request) return;
        setErrors((known) => ({ ...known, [sellerId]: uiErrorOf(failure) }));
      }
    },
    [entryId, stores],
  );

  const spread = useCallback(
    (sellerId: number, text: string) => {
      const request = (latest.current.get(sellerId) ?? 0) + 1;
      latest.current.set(sellerId, request);
      const waiting = timers.current.get(sellerId);
      if (waiting !== undefined) window.clearTimeout(waiting);
      timers.current.set(
        sellerId,
        window.setTimeout(() => {
          timers.current.delete(sellerId);
          void run(sellerId, text, request);
        }, SPREAD_DELAY_MS),
      );
    },
    [run],
  );

  return { spread, errors };
}
