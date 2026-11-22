// The box that asks before an entry takes cash in hand below zero (PRD FR-6.7, FR-7.8). One piece
// for every save that can do it: a بکری with نقد (plan 13), a پیشگی, an ادائیگی, a خرچہ and the
// rest (plans 15 to 17). It says what cash will be and asks "is this entry correct?". The shell, the focus on No and
// the keys are `ConfirmEntryDialog`'s. The amount is the cash after the entry from the backend, shown with formatPaisa;
// nothing is worked out here (CLAUDE.md rule 1).

import { Localized } from '@fluent/react';
import { formatPaisa, useGrouping, type Paisa } from '../format';
import { ConfirmEntryDialog } from './ConfirmEntryDialog';
import { Num } from './Num';

interface CashConfirmDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  /** Cash in hand after the entry, from the backend (`cash_after_paisa`). */
  cashAfter: Paisa;
  /** The user said yes: save with the confirmation. */
  onConfirm: () => void;
  /** Called when the box has closed and focus is about to return; the screen may take it. */
  onCloseFocus?: ((event: Event) => void) | undefined;
}

export function CashConfirmDialog({
  open,
  onOpenChange,
  cashAfter,
  onConfirm,
  onCloseFocus,
}: CashConfirmDialogProps) {
  const grouping = useGrouping();
  return (
    <ConfirmEntryDialog
      open={open}
      onOpenChange={onOpenChange}
      title={<Localized id="common-cash-confirm-title" />}
      body={
        // The child element is needed: without one Fluent does not read the <num> markup.
        <Localized
          id="common-cash-confirm-text"
          vars={{ amount: formatPaisa(cashAfter, { grouping }) }}
          elems={{ num: <Num /> }}
        >
          <span />
        </Localized>
      }
      onConfirm={onConfirm}
      onCloseFocus={onCloseFocus}
    />
  );
}
