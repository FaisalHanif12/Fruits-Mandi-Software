// What happens when the munshi presses F10 (PRD FR-12.8, FR-6.7, FR-7.57, plan 13).
//
// 1. If the live answer says the entry takes cash in hand below zero (`cash.needs_confirm`), the
//    box "is this entry correct?" opens first. Yes saves with the confirmation, No goes back to
//    the نقد cell with nothing saved. If the answer is older than what is on the screen, the save
//    is asked for without the confirmation, and the backend's refusal (`ERR_CASH_BELOW_ZERO_NOT_
//    CONFIRMED`) opens the box with the cash worked out for the draft as it is now.
// 2. The save is one call. One save runs at a time (a second F10 while one is on its way does
//    nothing, NFR-REL-1). The draft goes as typed; the backend works every number out again.
// 3. If the backend refuses, each problem shows at its cell (the problem list and the cell by
//    field path) and the rest as words near the buttons. Nothing is lost: the screen keeps the
//    draft. The refusal goes away when the user changes something.
// 4. If it saved: a message "بکری 5 محفوظ ہو گئی", and the screen hands the result on (`onSaved`),
//    which opens a new empty بکری (and, in plan 14, prints with Ctrl+Enter).
//
// The frontend works out no money here (CLAUDE.md rule 1): the cash after the entry and every
// number come from the backend.

import { Localized } from '@fluent/react';
import { useMemo, useRef, useState } from 'react';
import type { BikriSaved } from '../../../bindings/BikriSaved';
import type { CashCheck } from '../../../bindings/CashCheck';
import type { UiError } from '../../../bindings/UiError';
import { uiErrorOf } from '../../../shared/api';
import { useSingleFlight } from '../../../shared/hooks';
import { showToast } from '../../../shared/ui';
import { bikriPreview } from '../api';
import { buildDraft } from '../draft';
import type { BikriDraftStores } from '../draft-store';
import { LOT, type BikriDraft, type BikriPreview } from '../types';
import { useSaveBikri } from './useSaveBikri';

/** The box that asks about cash below zero: the cash after the entry, from the backend. */
export interface CashAsk {
  cash: CashCheck;
  /** The save was started with Ctrl+Enter: the screen prints after it (plan 14). */
  andNext: boolean;
}

interface Refusal {
  error: UiError;
  /**
   * The draft it was for, as text. Typing something new makes the refusal old; moving the cursor
   * or switching the language does not, because the text sent is the same.
   */
  draftText: string;
}

interface SaveFlowOptions {
  stores: BikriDraftStores;
  /** The draft as typed now (the same object the preview is asked for). */
  draft: BikriDraft;
  /** The latest answer of the preview, and whether it is for what is on the screen now. */
  preview: BikriPreview | undefined;
  current: boolean;
  /** The بکری was saved: the screen opens a new one. */
  onSaved: (saved: BikriSaved, andNext: boolean) => void;
}

export function useSaveFlow({ stores, draft: typed, preview, current, onSaved }: SaveFlowOptions) {
  const mutation = useSaveBikri();
  const flight = useSingleFlight();
  const [refusal, setRefusal] = useState<Refusal | null>(null);
  const [tried, setTried] = useState(false);
  const [ask, setAsk] = useState<CashAsk | null>(null);
  // The last cash asked about, kept so the box can close with its words still in place.
  const [lastCash, setLastCash] = useState<CashCheck | null>(null);
  // The box calls onConfirm and then onOpenChange(false); only a "no" is a decline.
  const confirmed = useRef(false);

  /** Sends the save. True when the بکری was saved. */
  const run = (confirmCashBelowZero: boolean, andNext: boolean): Promise<boolean> =>
    flight.run(async () => {
      const draft = buildDraft(stores);
      try {
        const saved = await mutation.mutateAsync({ draft, confirmCashBelowZero });
        setRefusal(null);
        setTried(false);
        showToast(
          'success',
          <Localized id="bikri-saved-toast" vars={{ number: String(saved.number) }}>
            <span />
          </Localized>,
        );
        onSaved(saved, andNext);
        return true;
      } catch (error: unknown) {
        const refused = uiErrorOf(error);
        if (refused.errors.some((item) => item.code === 'ERR_CASH_BELOW_ZERO_NOT_CONFIRMED')) {
          // The cash changed since the last answer. Ask the question with the numbers as they are.
          const fresh = await bikriPreview(draft).catch(() => null);
          if (fresh?.cash.needs_confirm === true) {
            setLastCash(fresh.cash);
            setAsk({ cash: fresh.cash, andNext });
            return false;
          }
        }
        setRefusal({ error: refused, draftText: JSON.stringify(draft) });
        return false;
      }
    }, false);

  /** F10, Ctrl+S and the button. `andNext` is Ctrl+Enter. True when it was saved. */
  const save = async (andNext: boolean): Promise<boolean> => {
    setTried(true);
    if (current && preview?.cash.needs_confirm === true) {
      setLastCash(preview.cash);
      setAsk({ cash: preview.cash, andNext });
      return false;
    }
    return run(false, andNext);
  };

  /** The user said yes to "is this entry correct?". */
  const confirmCash = () => {
    const asked = ask;
    confirmed.current = true;
    setAsk(null);
    if (asked !== null) void run(true, asked.andNext);
  };

  /** The box closed. Anything but a yes is a no: nothing is saved, the نقد cell is selected. */
  const onCashOpenChange = (open: boolean) => {
    if (open) return;
    if (confirmed.current) {
      confirmed.current = false;
      return;
    }
    setAsk(null);
    const rows = stores.lots.getState().rows;
    const first = rows.findIndex((row) => (row.typed[LOT.cash] ?? '').trim() !== '');
    stores.lots.getState().setPendingFocus({ rowIndex: Math.max(first, 0), field: LOT.cash });
  };

  /** After a "no" focus goes to the نقد cell, not back to the button (PRD FR-6.7). */
  const onCashCloseFocus = (event: Event) => {
    if (!confirmed.current) event.preventDefault();
  };

  const typedText = useMemo(() => JSON.stringify(typed), [typed]);
  const shown = refusal !== null && refusal.draftText === typedText ? refusal.error : null;

  return {
    save,
    busy: flight.busy,
    /** The backend's refusal of the last save, until the draft changes. */
    refusal: shown,
    /** The user has tried to save: problems that only say "empty" show now (plan 12 contract). */
    tried,
    /** The cash box is open when this is not null. */
    ask,
    /** The cash after the entry in the box, kept while it closes. */
    cashAfter: lastCash?.cash_after_paisa ?? null,
    confirmCash,
    onCashOpenChange,
    onCashCloseFocus,
  };
}
