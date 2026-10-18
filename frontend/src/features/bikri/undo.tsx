// Undo straight after a save (PRD FR-7.47, plan 16). The message after a save of a new بکری has a
// button "واپس لیں" (Alt+Z) for 10 seconds. Pressing it asks for the undo (`requestUndo`); the
// `UndoHost`, mounted once in the app shell, does it, so it works from any screen, also after the
// user has left the sale book:
//
// 1. It asks the backend what taking the بکری back would do to cash (`bikri_cancel_preview`). If
//    it would take cash in hand below zero, the box "is this entry correct?" opens first (PRD
//    FR-6.7), on whatever screen he is on.
// 2. It sends `bikri_undo`: a cancel with the reason "bikri-undo-reason", by reversal, which gives
//    the values back as a draft.
// 3. It opens the sale book on a NEW unsaved entry with those values, to correct and save again.
//    The cancelled number is not used again. The draft goes through the same door as a draft
//    restored at start-up (`usePendingRestore`), so it works from any screen and replaces the
//    empty entry that was opened after the save.
//
// A refusal (the day was closed meanwhile, the بکری changed) is told in a message with the
// backend's words. Nothing ever ends in silence.

import { Localized } from '@fluent/react';
import { useNavigate } from '@tanstack/react-router';
import { useEffect, useState } from 'react';
import { create } from 'zustand';
import { uiErrorOf } from '../../shared/api';
import type { Paisa } from '../../shared/format';
import { useLatest } from '../../shared/hooks';
import { CashConfirmDialog, ErrorMessage, showToast } from '../../shared/ui';
import { bikriCancelPreview } from './api';
import { applyBackendDraft, resolveDraftRecords } from './draft-load';
import { usePendingRestore } from './draft-offer';
import { createBikriDraft } from './draft-store';
import { parseSnapshot, serialiseDraft } from './draft-snapshot';
import { useUndoBikri } from './hooks/useStockActions';
import type { BikriDraft } from './types';

/** How long the undo button stays on the message (PRD FR-7.47). */
export const UNDO_WINDOW_MS = 10_000;

/** The backend's refusal when the cash question was not answered yes (PRD FR-6.7). */
const CASH_REFUSAL = 'ERR_CASH_BELOW_ZERO_NOT_CONFIRMED';

/** The بکری whose save the user wants to take back. */
export interface UndoTarget {
  id: number;
  number: number;
}

interface UndoRequests {
  request: { target: UndoTarget; serial: number } | null;
  ask: (target: UndoTarget) => void;
  take: () => void;
}

const useUndoRequests = create<UndoRequests>()((set) => ({
  request: null,
  ask: (target) => {
    set((state) => ({ request: { target, serial: (state.request?.serial ?? 0) + 1 } }));
  },
  take: () => {
    set({ request: null });
  },
}));

/** The undo button or Alt+Z of the message after a save. Works from any screen. */
export function requestUndo(target: UndoTarget): void {
  useUndoRequests.getState().ask(target);
}

/** The values of the undone بکری as a snapshot of the sale book, or null if it cannot be made. */
async function snapshotOf(draft: BikriDraft) {
  const records = await resolveDraftRecords(draft, { accounts: [], masters: new Map() });
  const stores = createBikriDraft(draft.header.entry_date);
  applyBackendDraft(stores, draft, records);
  return parseSnapshot(serialiseDraft(stores));
}

/** A refusal as a message in words (the first problem of it), or a general one. */
function failureToast(failure: unknown) {
  const first = uiErrorOf(failure).errors[0];
  showToast(
    'error',
    first === undefined ? (
      <Localized id="bikri-undo-failed">
        <span />
      </Localized>
    ) : (
      <ErrorMessage item={first}>
        <span />
      </ErrorMessage>
    ),
  );
}

export function UndoHost() {
  const navigate = useNavigate();
  const undo = useUndoBikri();
  // The box that asks about cash below zero: which بکری, and the cash after the undo.
  const [ask, setAsk] = useState<{ target: UndoTarget; cashAfter: Paisa } | null>(null);

  const run = async (target: UndoTarget, confirm: boolean) => {
    try {
      const result = await undo.mutateAsync({ id: target.id, confirm });
      const snapshot = await snapshotOf(result.draft);
      if (snapshot === null) throw new Error('the draft cannot be shown');
      usePendingRestore.getState().offer(snapshot);
      showToast(
        'success',
        <Localized id="bikri-undo-done" vars={{ number: String(result.number) }}>
          <span />
        </Localized>,
      );
      void navigate({ to: '/bikri/book', search: {} });
    } catch (failure: unknown) {
      // The cash changed since the question was last asked (a voucher saved meanwhile). Ask it
      // again with the numbers as they are now, like the save does. If it cannot be asked, he is
      // told the refusal in words, with the cash after: never nothing (plan 16, PRD FR-6.7).
      if (uiErrorOf(failure).errors.some((item) => item.code === CASH_REFUSAL)) {
        const fresh = await bikriCancelPreview(target.id).catch(() => null);
        if (fresh?.cash.needs_confirm === true) {
          setAsk({ target, cashAfter: fresh.cash.cash_after_paisa });
          return;
        }
      }
      failureToast(failure);
    }
  };

  const start = async (target: UndoTarget) => {
    try {
      const preview = await bikriCancelPreview(target.id);
      if (preview.cash.needs_confirm) {
        setAsk({ target, cashAfter: preview.cash.cash_after_paisa });
        return;
      }
    } catch (failure: unknown) {
      failureToast(failure);
      return;
    }
    await run(target, false);
  };

  // A request from the message after a save, from any screen.
  const startNow = useLatest(start);
  useEffect(
    () =>
      useUndoRequests.subscribe((state) => {
        if (state.request === null) return;
        const { target } = state.request;
        state.take();
        void startNow()(target);
      }),
    [startNow],
  );

  return ask === null ? null : (
    <CashConfirmDialog
      open
      onOpenChange={(open) => {
        if (!open) setAsk(null);
      }}
      cashAfter={ask.cashAfter}
      onConfirm={() => {
        const target = ask.target;
        setAsk(null);
        void run(target, true);
      }}
    />
  );
}
