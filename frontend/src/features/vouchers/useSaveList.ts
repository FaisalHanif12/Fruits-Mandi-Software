// Saving the waiting list (PRD FR-15.1): several vouchers entered one after another, saved together.
// Each voucher is its own posting event (its own save command and transaction), in the order of the
// list. The run goes on past a voucher the backend refuses: that one stays in the list with its
// reason and the others are saved.
//
// Cash below zero (PRD FR-6.7): the confirmation is never sent unless the user saw the box for that
// very voucher. The first try of every voucher is sent without it. If the backend refuses with
// `ERR_CASH_BELOW_ZERO_NOT_CONFIRMED`, the run stops at that voucher and asks, with the cash after
// THAT entry as the backend says it now (earlier vouchers of the run are already in the books).
// Yes saves it with the confirmation and the run goes on; No keeps it in the list and the run goes
// on with the rest.
//
// The frontend works out no money (CLAUDE.md rule 1): amounts go as typed, and the cash in the box
// is the backend's.

import { useRef, useState } from 'react';
import type { CashCheck } from '../../bindings/CashCheck';
import { uiErrorOf } from '../../shared/api';
import { useSingleFlight } from '../../shared/hooks';
import { vouchersPreview } from './api';
import { accountRequired, build, entryOf, type ListedOutcome, type ListedVoucher } from './draft';
import { useSaveVoucher } from './hooks';

const CASH_NOT_CONFIRMED = 'ERR_CASH_BELOW_ZERO_NOT_CONFIRMED';

export function useSaveList() {
  const create = useSaveVoucher();
  const flight = useSingleFlight();
  const [ask, setAsk] = useState<CashCheck | null>(null);
  // The last cash asked about, kept so the box can close with its words still in place.
  const [lastCash, setLastCash] = useState<CashCheck | null>(null);
  // The run is waiting for the answer of the box.
  const answer = useRef<((yes: boolean) => void) | null>(null);
  // The box calls onConfirm and then onOpenChange(false); only a "no" is a decline.
  const confirmed = useRef(false);

  const askCash = (cash: CashCheck): Promise<boolean> =>
    new Promise((resolve) => {
      answer.current = resolve;
      setLastCash(cash);
      setAsk(cash);
    });

  /** Saves one voucher of the list. */
  const saveOne = async (voucher: ListedVoucher): Promise<ListedOutcome> => {
    const first = build(voucher.draft, false);
    if (first === null) {
      return { id: voucher.id, kind: 'refused', error: accountRequired(voucher.draft) };
    }
    try {
      const saved = await create.mutateAsync(first);
      return { id: voucher.id, kind: 'saved', draft: voucher.draft, saved };
    } catch (failure: unknown) {
      const refused = uiErrorOf(failure);
      if (!refused.errors.some((item) => item.code === CASH_NOT_CONFIRMED)) {
        return { id: voucher.id, kind: 'refused', error: refused };
      }
      // The cash after this entry, as the books are now.
      const fresh = await vouchersPreview(entryOf(first), null).catch(() => null);
      if (fresh?.cash.needs_confirm !== true) {
        return { id: voucher.id, kind: 'refused', error: refused };
      }
      if (!(await askCash(fresh.cash))) return { id: voucher.id, kind: 'declined' };
      const confirmedBuilt = build(voucher.draft, true);
      if (confirmedBuilt === null) return { id: voucher.id, kind: 'declined' };
      try {
        const saved = await create.mutateAsync(confirmedBuilt);
        return { id: voucher.id, kind: 'saved', draft: voucher.draft, saved };
      } catch (second: unknown) {
        return { id: voucher.id, kind: 'refused', error: uiErrorOf(second) };
      }
    }
  };

  /**
   * Saves the vouchers in order and tells how each one came out, in the same order. The outcome of
   * a voucher is handed on as soon as it is known, so the screen can take it off the list at once.
   */
  const run = (
    vouchers: readonly ListedVoucher[],
    onOutcome: (outcome: ListedOutcome) => void,
  ): Promise<readonly ListedOutcome[]> =>
    flight.run(async () => {
      const outcomes: ListedOutcome[] = [];
      for (const voucher of vouchers) {
        const outcome = await saveOne(voucher);
        outcomes.push(outcome);
        onOutcome(outcome);
      }
      return outcomes;
    }, []);

  /** The user said yes to "is this entry correct?". */
  const confirmCash = () => {
    confirmed.current = true;
    setAsk(null);
    answer.current?.(true);
    answer.current = null;
  };

  /** The box closed. Anything but a yes is a no: that voucher stays in the list. */
  const onCashOpenChange = (open: boolean): boolean => {
    if (open) return false;
    if (confirmed.current) {
      confirmed.current = false;
      return false;
    }
    setAsk(null);
    answer.current?.(false);
    answer.current = null;
    return true;
  };

  return {
    run,
    busy: flight.busy,
    /** The cash box is open when this is not null. */
    ask,
    /** The cash after the voucher in the box, kept while it closes. */
    cashAfter: lastCash?.cash_after_paisa ?? null,
    confirmCash,
    onCashOpenChange,
  };
}
