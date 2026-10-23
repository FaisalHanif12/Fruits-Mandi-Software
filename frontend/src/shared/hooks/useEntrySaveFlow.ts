// What happens when the munshi presses F10 on the trip screen, for every type of entry (PRD
// FR-12.24, FR-12.31, FR-12.32, FR-6.7). One flow, so a purchase, a trip cost and a sale in
// another shop save the same way:
//
// 1. If the live answer says the shop parts take cash in hand below zero (`cash.needs_confirm`),
//    the box "is this entry correct?" opens first. Yes saves with the confirmation, No goes back
//    to the form with nothing saved. If the answer is older than what is on the screen, the save
//    is asked for without the confirmation, and the backend's refusal
//    (`ERR_CASH_BELOW_ZERO_NOT_CONFIRMED`) opens the box with the cash worked out for the entry as
//    it is now. A sale in another shop only brings money in, so it never asks.
// 2. The save is one call. One save runs at a time (a second F10 while one is on its way does
//    nothing, NFR-REL-1). The entry goes as typed; the backend works every number out again.
// 3. If the backend refuses, each problem shows at its field and the rest as words near the
//    buttons. Nothing is lost. The refusal goes away when the user changes something.
// 4. If it saved: a message with the number, and the screen starts the next entry.
//
// The frontend works out no money here (CLAUDE.md rule 1): the cash after the entry and every
// number come from the backend.

import { useMemo, useRef, useState } from 'react';
import type { CashCheck } from '../../bindings/CashCheck';
import type { UiError } from '../../bindings/UiError';
import { uiErrorOf } from '../api';
import { useSingleFlight } from './useSingleFlight';

interface Refusal {
  error: UiError;
  /**
   * The draft it was for, as text. Typing something new makes the refusal old; moving the cursor
   * or switching the language does not, because the text sent is the same.
   */
  draftText: string;
}

interface EntrySaveFlowOptions<TInput, TSaved> {
  /** The draft as text, to tell whether a refusal still belongs to what is on screen. */
  draftText: string;
  /** The input to send, with the answer to the cash question. */
  build: (confirmCash: boolean) => TInput;
  save: (input: TInput) => Promise<TSaved>;
  /**
   * The cash after the entry as it is now, asked after a refusal for cash. Null for an entry that
   * never takes cash out.
   */
  freshCash: ((input: TInput) => Promise<CashCheck | null>) | null;
  /** The live answer of the preview, and whether it is for what is on the screen now. */
  cash: CashCheck | undefined;
  current: boolean;
  /** The entry was saved: the screen shows a message and starts the next one. */
  onSaved: (saved: TSaved) => void;
}

export function useEntrySaveFlow<TInput, TSaved>({
  draftText,
  build,
  save,
  freshCash,
  cash,
  current,
  onSaved,
}: EntrySaveFlowOptions<TInput, TSaved>) {
  const flight = useSingleFlight();
  const [refusal, setRefusal] = useState<Refusal | null>(null);
  const [ask, setAsk] = useState(false);
  // The last cash asked about, kept so the box can close with its words still in place.
  const [lastCash, setLastCash] = useState<CashCheck | null>(null);
  // The box calls onConfirm and then onOpenChange(false); only a "no" is a decline.
  const confirmed = useRef(false);

  /** Sends the save. True when the entry was saved. */
  const run = (confirmCash: boolean): Promise<boolean> =>
    flight.run(async () => {
      try {
        const saved = await save(build(confirmCash));
        setRefusal(null);
        onSaved(saved);
        return true;
      } catch (error: unknown) {
        const refused = uiErrorOf(error);
        if (
          freshCash !== null &&
          refused.errors.some((item) => item.code === 'ERR_CASH_BELOW_ZERO_NOT_CONFIRMED')
        ) {
          // The cash changed since the last answer. Ask the question with the numbers as they are.
          const fresh = await freshCash(build(false)).catch(() => null);
          if (fresh?.needs_confirm === true) {
            setLastCash(fresh);
            setAsk(true);
            return false;
          }
        }
        setRefusal({ error: refused, draftText });
        return false;
      }
    }, false);

  /** F10, Ctrl+S and the button. True when it was saved. */
  const saveNow = async (): Promise<boolean> => {
    if (current && cash?.needs_confirm === true) {
      setLastCash(cash);
      setAsk(true);
      return false;
    }
    return run(false);
  };

  /** The user said yes to "is this entry correct?". */
  const confirmCash = () => {
    confirmed.current = true;
    setAsk(false);
    void run(true);
  };

  /** The box closed. Anything but a yes is a no: nothing is saved. True after a no. */
  const onCashOpenChange = (open: boolean): boolean => {
    if (open) return false;
    if (confirmed.current) {
      confirmed.current = false;
      return false;
    }
    setAsk(false);
    return true;
  };

  const shown = useMemo(
    () => (refusal !== null && refusal.draftText === draftText ? refusal.error : null),
    [refusal, draftText],
  );
  return {
    save: saveNow,
    busy: flight.busy,
    /** The backend's refusal of the last save, until the draft changes. */
    refusal: shown,
    /** The cash box is open when this is true. */
    ask,
    /** The cash after the entry in the box, kept while it closes. */
    cashAfter: lastCash?.cash_after_paisa ?? null,
    confirmCash,
    onCashOpenChange,
  };
}
