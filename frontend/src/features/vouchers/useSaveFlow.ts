// What happens when the munshi presses F10 on the voucher screen (PRD 15.1 to 15.4, FR-6.7, FR-15.3).
//
// 1. If the live answer says the entry takes cash in hand below zero (`cash.needs_confirm`), the
//    box "is this entry correct?" opens first. Yes saves with the confirmation, No goes back to the
//    amount with nothing saved. If the answer is older than what is on the screen, the save is
//    asked for without the confirmation, and the backend's refusal (`ERR_CASH_BELOW_ZERO_NOT_
//    CONFIRMED`) opens the box with the cash worked out for the entry as it is now.
// 2. The save is one call (a new voucher, or an edit of a saved one). One save runs at a time (a
//    second F10 while one is on its way does nothing, NFR-REL-1). The entry goes as typed; the
//    backend works every number out again.
// 3. If the backend refuses, each problem shows at its field and the rest as words near the
//    buttons. Nothing is lost. The refusal goes away when the user changes something.
// 4. If it saved: a message "وصولی و-5 محفوظ ہو گئی", and the screen hands the result on
//    (`onSaved`), which starts the next voucher.
//
// The frontend works out no money here (CLAUDE.md rule 1): the cash after the entry and every
// number come from the backend.

import { useLocalization } from '@fluent/react';
import { useMemo, useRef, useState } from 'react';
import type { CashCheck } from '../../bindings/CashCheck';
import type { UiError } from '../../bindings/UiError';
import { uiErrorOf } from '../../shared/api';
import { useSingleFlight } from '../../shared/hooks';
import { showToast } from '../../shared/ui';
import { vouchersPreview } from './api';
import { accountRequired, build, entryOf } from './draft';
import { useEditVoucher, useSaveVoucher } from './hooks';
import type { VoucherDraft, VoucherPreview, VoucherSaved } from './types';

/** The box that asks about cash below zero: the cash after the entry, from the backend. */
export interface CashAsk {
  cash: CashCheck;
  /** The save was started with Ctrl+Enter: the screen prints after it. */
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

/** A saved voucher that is being changed: its id, the version read with it and its number. */
interface EditingVoucher {
  id: number;
  version: number;
  number: number;
}

interface SaveFlowOptions {
  draft: VoucherDraft;
  editing: EditingVoucher | null;
  /** An optional note on why a saved voucher was changed (kept in the audit log). */
  editReason: string;
  /** The latest answer of the preview, and whether it is for what is on the screen now. */
  preview: VoucherPreview | undefined;
  current: boolean;
  /** The voucher was saved: the screen starts the next one. */
  onSaved: (saved: VoucherSaved, andNext: boolean) => void;
}

export function useSaveFlow({
  draft,
  editing,
  editReason,
  preview,
  current,
  onSaved,
}: SaveFlowOptions) {
  const { l10n } = useLocalization();
  const create = useSaveVoucher();
  const edit = useEditVoucher();
  const flight = useSingleFlight();
  const [refusal, setRefusal] = useState<Refusal | null>(null);
  const [ask, setAsk] = useState<CashAsk | null>(null);
  // The last cash asked about, kept so the box can close with its words still in place.
  const [lastCash, setLastCash] = useState<CashCheck | null>(null);
  // The box calls onConfirm and then onOpenChange(false); only a "no" is a decline.
  const confirmed = useRef(false);
  const draftText = useMemo(() => JSON.stringify(draft), [draft]);

  /** Sends the save. True when the voucher was saved. */
  const run = (confirmCash: boolean, andNext: boolean): Promise<boolean> =>
    flight.run(async () => {
      const built = build(draft, confirmCash);
      if (built === null) {
        setRefusal({ error: accountRequired(draft), draftText });
        return false;
      }
      try {
        const saved =
          editing === null
            ? await create.mutateAsync(built)
            : await edit.mutateAsync({
                id: editing.id,
                version: editing.version,
                entry: entryOf(built),
                reason: editReason,
                confirm: confirmCash,
              });
        setRefusal(null);
        const number = l10n.getString(`vouchers-number-${draft.type}`, {
          number: String(saved.number),
        });
        showToast(
          'success',
          l10n.getString(editing === null ? 'vouchers-saved' : 'vouchers-edited', {
            kind: draft.type,
            number,
          }),
        );
        onSaved(saved, andNext);
        return true;
      } catch (error: unknown) {
        const refused = uiErrorOf(error);
        if (refused.errors.some((item) => item.code === 'ERR_CASH_BELOW_ZERO_NOT_CONFIRMED')) {
          // The cash changed since the last answer. Ask the question with the numbers as they are.
          const fresh = await vouchersPreview(
            entryOf(build(draft, false) ?? built),
            editing?.id ?? null,
          ).catch(() => null);
          if (fresh?.cash.needs_confirm === true) {
            setLastCash(fresh.cash);
            setAsk({ cash: fresh.cash, andNext });
            return false;
          }
        }
        setRefusal({ error: refused, draftText });
        return false;
      }
    }, false);

  /** F10, Ctrl+S, Ctrl+Enter and the button. `andNext` is Ctrl+Enter. True when it was saved. */
  const save = async (andNext: boolean): Promise<boolean> => {
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

  /** The box closed. Anything but a yes is a no: nothing is saved. */
  const onCashOpenChange = (open: boolean): boolean => {
    if (open) return false;
    if (confirmed.current) {
      confirmed.current = false;
      return false;
    }
    setAsk(null);
    return true;
  };

  /** Tells a problem at its field, for a voucher that cannot be listed or saved yet. */
  const tell = (error: UiError) => {
    setRefusal({ error, draftText });
  };

  const shown = refusal !== null && refusal.draftText === draftText ? refusal.error : null;
  return {
    save,
    tell,
    busy: flight.busy,
    /** The backend's refusal of the last save, until the draft changes. */
    refusal: shown,
    /** The cash box is open when this is not null. */
    ask,
    /** The cash after the entry in the box, kept while it closes. */
    cashAfter: lastCash?.cash_after_paisa ?? null,
    confirmCash,
    /** True when the box was closed with a "no": the screen puts focus on the amount. */
    onCashOpenChange,
  };
}
