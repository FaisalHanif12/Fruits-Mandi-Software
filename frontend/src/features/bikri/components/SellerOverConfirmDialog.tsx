// The box that asks "is this entry correct?" when money paid to a seller on the spot is more than he may take without a
// question, so his balance will grow (plan 91, PRD FR-12.23 v1.53). It is the same question as the cash one (PRD FR-6.7)
// and one yes answers both: it names each seller and the balance he will have, and, when cash in hand would go below zero
// too, says so. Every figure is the backend's. Yes saves with the confirmation; No goes back to the seller's field. The
// words and the keys are the shared `ConfirmEntryDialog`'s.

import { Localized, useLocalization } from '@fluent/react';
import type { Balance as BalanceValue } from '../../../bindings/Balance';
import { formatPaisa, useGrouping, type Paisa } from '../../../shared/format';
import { ConfirmEntryDialog, Num, sideMessageId } from '../../../shared/ui';

/** A seller the question is about: his name as shown, and the balance the backend says he will have. */
export interface SellerOverEntry {
  sellerId: number;
  name: string;
  balance: BalanceValue;
}

interface SellerOverConfirmDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  entries: readonly SellerOverEntry[];
  /** Cash in hand after the entry, when it would go below zero as well (the backend's `cash_after_paisa`). */
  cashAfter: Paisa | null;
  onConfirm: () => void;
  onCloseFocus?: ((event: Event) => void) | undefined;
}

export function SellerOverConfirmDialog({
  open,
  onOpenChange,
  entries,
  cashAfter,
  onConfirm,
  onCloseFocus,
}: SellerOverConfirmDialogProps) {
  const { l10n } = useLocalization();
  const grouping = useGrouping();
  return (
    <ConfirmEntryDialog
      open={open}
      onOpenChange={onOpenChange}
      title={<Localized id="bikri-seller-over-title" />}
      body={
        <div className="flex flex-col gap-1">
          {entries.map((entry) => {
            const sideId = sideMessageId(entry.balance.side);
            return (
              <Localized
                key={entry.sellerId}
                id="bikri-seller-over-line"
                vars={{
                  name: entry.name,
                  amount: formatPaisa(entry.balance.amount_paisa, { grouping }),
                  side: sideId === null ? '' : l10n.getString(sideId),
                }}
                elems={{ num: <Num /> }}
              >
                <p />
              </Localized>
            );
          })}
          {cashAfter === null ? null : (
            <Localized
              id="common-cash-after"
              vars={{ amount: formatPaisa(cashAfter, { grouping }) }}
              elems={{ num: <Num /> }}
            >
              <p />
            </Localized>
          )}
          <Localized id="bikri-seller-over-question">
            <p />
          </Localized>
        </div>
      }
      onConfirm={onConfirm}
      onCloseFocus={onCloseFocus}
    />
  );
}
