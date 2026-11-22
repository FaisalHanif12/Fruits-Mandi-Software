// What happens when the munshi presses F10 on the voucher screen (PRD 15.1 to 15.4, FR-6.7, FR-15.3).
//
// 1. If the live answer says the entry takes cash in hand below zero (`cash.needs_confirm`), the
//    box "is this entry correct?" opens first. Yes saves with the confirmation, No goes back to the
//    amount with nothing saved. If the answer is older than what is on the screen, the save is
//    asked for without the confirmation, and the backend's refusal (`ERR_CASH_BELOW_ZERO_NOT_
//    CONFIRMED`) opens the box with the cash worked out for the entry as it is now.
// 2. A saved voucher that is being changed asks the reason and the Owner's sign in a box of its own
//    before anything is sent, on any day, today included (PRD FR-15.3, plan 92). Both are required.
//    The backend checks the sign; a wrong one is told under it in the box and nothing is saved. The
//    box comes before the cash question, which is asked after it. When the day was reopened to fix
//    this entry (PRD FR-21.4) the reopen already asked both: its reason goes with the edit and the
//    backend takes the sign from the reopen, so nothing is asked twice. If the backend still wants the
//    sign, the box opens.
// 3. The save is one call (a new voucher, or an edit of a saved one). One save runs at a time (a
//    second F10 while one is on its way does nothing, NFR-REL-1). The entry goes as typed; the
//    backend works every number out again.
// 4. If the backend refuses, nothing is silent: a problem of the box (the sign, the reason of the
//    change) is told in the box, every other one at its field or as words near the buttons. Nothing is
//    lost. A refusal goes away when the user changes something.
// 5. If it saved: a message "وصولی و-5 محفوظ ہو گئی", and the screen hands the result on
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
import { accountRequired, build, entryOf, isCashLent, savedKindOf } from './draft';
import { voucherNumberText } from './list-columns';
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

/** What the box of a change asks: why, and the Owner's sign. Both as typed. */
export interface ChangeSigned {
  reason: string;
  sign: string;
}

/** The box of a change is open when this is not null. */
export interface SignAsk {
  /** The save was started with Ctrl+Enter. */
  andNext: boolean;
}

/**
 * The problems that belong to the box of a change (plan 92): the sign, and the reason of the change. A cash lent has a
 * reason of its own that the backend also names `reason`; while that one is empty the problem is the form's.
 */
function belongsToBox(item: UiError['errors'][number], draft: VoucherDraft): boolean {
  if (item.field === 'sign') return true;
  return item.field === 'reason' && !(isCashLent(draft) && draft.reason.trim() === '');
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
  /**
   * The reason typed when the day of this voucher was reopened to fix it (PRD FR-21.4), or null. The reopen asked the
   * reason and the sign, so the edit asks nothing again (PRD FR-15.3, plan 92): this goes as the reason, and the
   * backend takes the sign from the reopen.
   */
  reopenedReason: string | null;
  /** The latest answer of the preview, and whether it is for what is on the screen now. */
  preview: VoucherPreview | undefined;
  current: boolean;
  /** The voucher was saved: the screen starts the next one. */
  onSaved: (saved: VoucherSaved, andNext: boolean) => void;
}

export function useSaveFlow({
  draft,
  editing,
  reopenedReason,
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
  // The box of a change (plan 92): open or not, what the backend refused in it, and what was typed in it, kept for
  // the cash question that may come after it and for the box when it opens again.
  const [askSign, setAskSign] = useState<SignAsk | null>(null);
  const [signError, setSignError] = useState<UiError | null>(null);
  const [signed, setSigned] = useState<ChangeSigned | null>(null);
  // The box calls onConfirm and then onOpenChange(false); only a "no" is a decline.
  const confirmed = useRef(false);
  const draftText = useMemo(() => JSON.stringify(draft), [draft]);

  /** Sends the save. True when the voucher was saved. `by` is the reason and sign of a change. */
  const run = (confirmCash: boolean, andNext: boolean, by: ChangeSigned | null): Promise<boolean> =>
    flight.run(async () => {
      const built = build(draft, confirmCash);
      if (built === null) {
        setAskSign(null);
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
                reason: by?.reason ?? '',
                sign: by?.sign ?? '',
                confirm: confirmCash,
              });
        setRefusal(null);
        setAskSign(null);
        setSignError(null);
        setSigned(null);
        // The number is in the series of the voucher the backend saved, which is a payment's for an
        // advance to a buyer (cash lent, PRD FR-15.11).
        const number = voucherNumberText(saved, (id, vars) => l10n.getString(id, vars));
        showToast(
          'success',
          l10n.getString(editing === null ? 'vouchers-saved' : 'vouchers-edited', {
            kind: editing === null ? savedKindOf(draft, saved) : draft.type,
            number,
          }),
        );
        onSaved(saved, andNext);
        return true;
      } catch (error: unknown) {
        const refused = uiErrorOf(error);
        // The sign or the reason of a change was refused: the box tells it under its field, with what was typed
        // still in it. What is not the box's is told at the form, as for a new voucher (plan 92).
        if (editing !== null && refused.errors.some((item) => belongsToBox(item, draft))) {
          const rest = refused.errors.filter((item) => !belongsToBox(item, draft));
          setSignError({
            ...refused,
            errors: refused.errors.filter((item) => belongsToBox(item, draft)),
          });
          setAskSign({ andNext });
          setRefusal(rest.length === 0 ? null : { error: { ...refused, errors: rest }, draftText });
          return false;
        }
        // Anything else is the form's: the box closes so the problem can be seen at its field.
        setAskSign(null);
        if (refused.errors.some((item) => item.code === 'ERR_CASH_BELOW_ZERO_NOT_CONFIRMED')) {
          // The cash changed since the last answer. Ask the question with the numbers as they are.
          const fresh = await vouchersPreview(
            entryOf(build(draft, false) ?? built),
            editing?.id ?? null,
          ).catch(() => null);
          if (fresh?.cash.needs_confirm === true) {
            setSigned(by);
            setLastCash(fresh.cash);
            setAsk({ cash: fresh.cash, andNext });
            return false;
          }
        }
        setRefusal({ error: refused, draftText });
        return false;
      }
    }, false);

  /** The cash question first if the entry needs it (PRD FR-6.7), then the save. */
  const proceed = (andNext: boolean, by: ChangeSigned | null): Promise<boolean> => {
    if (current && preview?.cash.needs_confirm === true) {
      setSigned(by);
      setLastCash(preview.cash);
      setAsk({ cash: preview.cash, andNext });
      // The box of the change closes while the cash question is asked.
      setAskSign(null);
      return Promise.resolve(false);
    }
    return run(false, andNext, by);
  };

  /** F10, Ctrl+S, Ctrl+Enter and the button. `andNext` is Ctrl+Enter. True when it was saved. */
  const save = async (andNext: boolean): Promise<boolean> => {
    if (editing === null) return proceed(andNext, null);
    // The day was reopened for this fix: the reopen asked the reason and the sign already.
    if (reopenedReason !== null && reopenedReason.trim() !== '') {
      return proceed(andNext, { reason: reopenedReason.trim(), sign: '' });
    }
    setSignError(null);
    setAskSign({ andNext });
    return false;
  };

  /** The reason and the sign were typed in the box. The backend checks the sign. */
  const confirmSign = (reason: string, sign: string) => {
    setSigned({ reason, sign });
    setSignError(null);
    void proceed(askSign?.andNext ?? false, { reason, sign });
  };

  /** The box of the change closed. Anything but a save is a no: nothing is saved. */
  const onSignOpenChange = (open: boolean) => {
    if (open || flight.busy) return;
    setAskSign(null);
    setSignError(null);
    setSigned(null);
  };

  /** The user said yes to "is this entry correct?". */
  const confirmCash = () => {
    const asked = ask;
    confirmed.current = true;
    setAsk(null);
    if (asked !== null) void run(true, asked.andNext, signed);
  };

  /** The box closed. Anything but a yes is a no: nothing is saved. */
  const onCashOpenChange = (open: boolean): boolean => {
    if (open) return false;
    if (confirmed.current) {
      confirmed.current = false;
      return false;
    }
    setAsk(null);
    setSigned(null);
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
    /** Forgets the refusal: the day it named was reopened, so it is old (PRD FR-21.4, plan 85). */
    clearRefusal: () => {
      setRefusal(null);
    },
    /** The box that asks the reason and the Owner's sign of a change is open when this is not null. */
    askSign,
    /** What the backend refused in that box (the sign, the reason), shown under its fields. */
    signError,
    /** What was typed in the box, so it opens again with it after the cash question. */
    signedReason: signed?.reason ?? '',
    confirmSign,
    onSignOpenChange,
    /** The cash box is open when this is not null. */
    ask,
    /** The cash after the entry in the box, kept while it closes. */
    cashAfter: lastCash?.cash_after_paisa ?? null,
    confirmCash,
    /** True when the box was closed with a "no": the screen puts focus on the amount. */
    onCashOpenChange,
  };
}
