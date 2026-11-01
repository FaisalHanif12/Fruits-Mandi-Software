// What happens when the Save button of an edit is pressed (plan 44, PRD FR-7.70, FR-6.7): one flow
// for the edits of the trip and the garden, so each of them asks the same questions in the same
// order and saves the same way.
//
// 1. An entry that is not of today asks why it is changed. The backend says so in the review
//    (`reason_required`); until the review has come the box is asked, never guessed away.
// 2. If the review says the edit takes cash in hand below zero and lowers it (`cash.needs_confirm`),
//    the box "is this entry correct?" opens. Yes saves with the answer, No goes back to the form.
//    If the review was old and the backend still refuses (`ERR_CASH_BELOW_ZERO_NOT_CONFIRMED`),
//    the box opens then, with the cash the review shows.
// 3. The save is one call, one at a time (NFR-REL-1). A refusal shows at its fields and goes away
//    when the user changes something. Nothing typed is lost.
//
// The frontend works out no money here (CLAUDE.md rule 1): the cash after the edit, the need for a
// reason and every number are the backend's, from the review.

import { useMemo, useRef, useState } from 'react';
import type { EditReview } from '../../bindings/EditReview';
import type { UiError } from '../../bindings/UiError';
import { uiErrorOf } from '../api';
import { useSingleFlight } from '../hooks/useSingleFlight';
import type { ReasonText } from './useReasonText';

interface Refusal {
  error: UiError;
  /** The draft it was for, as text: typing something new makes the refusal old. */
  draftText: string;
}

interface EditSaveOptions<TSaved> {
  /** The review of what is typed now, or undefined while it is not known yet. */
  review: EditReview | undefined;
  reason: ReasonText;
  /** The entry as text, to tell whether a refusal still belongs to what is on the screen. */
  draftText: string;
  /** Sends the save with the reason and the answer to the cash question. */
  send: (reason: string, confirmCash: boolean) => Promise<TSaved>;
  /** The edit was saved: the screen shows a message and goes back to a new entry. */
  onSaved: (saved: TSaved) => void;
}

export function useEditSave<TSaved>({
  review,
  reason,
  draftText,
  send,
  onSaved,
}: EditSaveOptions<TSaved>) {
  const flight = useSingleFlight();
  const [refusal, setRefusal] = useState<Refusal | null>(null);
  const [askReason, setAskReason] = useState(false);
  const [askCash, setAskCash] = useState(false);
  // The box calls onConfirm and then onOpenChange(false); only a "no" is a decline.
  const confirmed = useRef(false);

  /** Sends the save. True when the entry was saved. */
  const run = (confirmCash: boolean): Promise<boolean> =>
    flight.run(async () => {
      try {
        const saved = await send(reason.get().trim(), confirmCash);
        setRefusal(null);
        onSaved(saved);
        return true;
      } catch (error: unknown) {
        const refused = uiErrorOf(error);
        if (
          review?.effect.cash != null &&
          refused.errors.some((item) => item.code === 'ERR_CASH_BELOW_ZERO_NOT_CONFIRMED')
        ) {
          setAskCash(true);
          return false;
        }
        setRefusal({ error: refused, draftText });
        return false;
      }
    }, false);

  /** The Save button, F10 and Ctrl+S. True when it was saved. */
  const save = async (): Promise<boolean> => {
    // An entry of an earlier day asks why it is changed (the backend says so; unknown asks).
    if ((review?.reason_required ?? true) && reason.get().trim() === '') {
      setAskReason(true);
      return false;
    }
    if (review?.effect.cash?.needs_confirm === true) {
      setAskCash(true);
      return false;
    }
    return run(false);
  };

  return {
    save,
    busy: flight.busy,
    /** The backend's refusal of the last save, until the draft changes. */
    refusal:
      refusal !== null && refusal.draftText === draftText
        ? refusal.error
        : (null as UiError | null),
    askReason,
    setAskReason,
    /** The reason was typed in the box: keep it and save. */
    confirmReason: (text: string) => {
      reason.set(text);
      setAskReason(false);
      void save();
    },
    /** The cash box is open when this is true. */
    askCash,
    /** Cash in hand after the edit, from the review, for the words of the box. */
    cashAfter: review?.effect.cash?.after_paisa ?? null,
    confirmCash: () => {
      confirmed.current = true;
      setAskCash(false);
      void run(true);
    },
    /** The box closed. Anything but a yes is a no: nothing is saved. True after a no. */
    onCashOpenChange: (open: boolean): boolean => {
      if (open) return false;
      if (confirmed.current) {
        confirmed.current = false;
        return false;
      }
      setAskCash(false);
      return true;
    },
  };
}

export type EditSave = ReturnType<typeof useEditSave>;

/** Memo helper for a screen: the draft as text for `useEditSave`. */
export function useDraftText(draft: unknown): string {
  return useMemo(() => JSON.stringify(draft), [draft]);
}
