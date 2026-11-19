// What the backend says about the entry on the form, in words under it (PRD FR-6.7, FR-7.58,
// FR-7.59, FR-7.68, FR-15.5, FR-15.7): the balance before and after, the part of a receipt that is
// more than all the bills, what a seller's return can be, the warnings that still let the user save,
// the note that offers a پیشگی when a seller has nothing payable, and the cash going below zero.
// Every figure is the backend's; nothing is worked out here (CLAUDE.md rule 1).

import { Localized } from '@fluent/react';
import { isPositivePaisa } from '../../../shared/format';
import { Balance, Button, CashNote, ErrorMessage, Notice, Num } from '../../../shared/ui';
import type { VoucherDraft, VoucherPreview } from '../types';

interface VoucherNotesProps {
  draft: VoucherDraft;
  preview: VoucherPreview;
  /** The user pressed "make it an advance" (PRD FR-7.68). Never done by itself. */
  onMakeAdvance: () => void;
}

export function VoucherNotes({ draft, preview, onMakeAdvance }: VoucherNotesProps) {
  const sellerReceipt = draft.type === 'receipt' && draft.from === 'seller';
  return (
    <div className="flex flex-col gap-3">
      <p className="flex flex-wrap items-baseline gap-x-6 gap-y-1 type-body">
        <span className="flex items-baseline gap-2">
          <Localized id="vouchers-current-balance" />
          <Balance balance={preview.current_balance} />
        </span>
        <span className="flex items-baseline gap-2">
          <Localized id="vouchers-new-balance" />
          <Balance balance={preview.new_balance} />
        </span>
        {sellerReceipt && preview.return_limit_paisa !== null ? (
          <span className="flex items-baseline gap-2">
            <Localized id="vouchers-return-limit" />
            <Num paisa={preview.return_limit_paisa} />
          </span>
        ) : null}
      </p>
      {isPositivePaisa(preview.extra_credit_paisa) ? (
        <Notice tone="info">
          <Localized id="vouchers-extra-credit-note" /> <Num paisa={preview.extra_credit_paisa} />
        </Notice>
      ) : null}
      {sellerReceipt && isPositivePaisa(preview.owes_settled_paisa) ? (
        <Notice tone="info">
          <Localized id="vouchers-owes-settled-note" /> <Num paisa={preview.owes_settled_paisa} />
        </Notice>
      ) : null}
      {preview.warnings.map((warning, index) => (
        <Notice key={`${warning.code}-${String(index)}`} tone="warn">
          <ErrorMessage item={warning}>
            <span />
          </ErrorMessage>
        </Notice>
      ))}
      {preview.payable_is_zero_for_seller ? (
        <Notice
          tone="info"
          action={
            <Button onClick={onMakeAdvance}>
              <Localized id="vouchers-make-advance" />
            </Button>
          }
        >
          <Localized id="vouchers-advance-hint" />
        </Notice>
      ) : null}
      <CashNote cash={preview.cash} />
    </div>
  );
}
