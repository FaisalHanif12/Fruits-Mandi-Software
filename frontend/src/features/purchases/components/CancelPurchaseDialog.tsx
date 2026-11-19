// Cancelling a purchase (PRD FR-12.24, FR-7.8): a box that says exactly what will happen, asks for
// a reason, and cancels by reversal. Nothing is deleted: the posting is reversed on its own day and
// the purchase stays in the trip, marked cancelled with its reason.
//
// A purchase can be cancelled only while nothing of it was sold or written off, and while it
// carries no part of a trip cost. When the backend says it cannot (`can_cancel` false), the box
// opens anyway and says why in words, with the button dimmed, so the user is never left
// wondering. If the backend refuses at the press all the same (`ERR_PURCHASE_HAS_SALES`,
// `ERR_PURCHASE_CARRIES_TRIP_COST`), the words show inside the box and what was typed stays.
//
// If taking the purchase back would take cash in hand below zero (a purchase paid from the shop
// gives cash back, so this is rare, PRD FR-6.7), the backend refuses the first try. The box then
// shows the cash after the cancel and the next press confirms it, so the user always sees it
// first.

import { Localized, useLocalization } from '@fluent/react';
import { useState } from 'react';
import type { UiError } from '../../../bindings/UiError';
import { uiErrorOf } from '../../../shared/api';
import { formatPaisa, useGrouping } from '../../../shared/format';
import { displayName, useLanguage } from '../../../shared/i18n';
import { ErrorMessage, Num, ReasonDialog, showToast } from '../../../shared/ui';
import { useCancelPurchase } from '../hooks';
import { purchaseNumberText, supplierText } from '../list-columns';
import type { PurchaseCancelled, PurchaseRow } from '../types';

interface CancelPurchaseDialogProps {
  /** The purchase to cancel, or null when the box is closed. */
  row: PurchaseRow | null;
  onClose: () => void;
  /** The purchase was cancelled. */
  onCancelled: (cancelled: PurchaseCancelled) => void;
}

const CASH_CODE = 'ERR_CASH_BELOW_ZERO_NOT_CONFIRMED';

export function CancelPurchaseDialog({ row, onClose, onCancelled }: CancelPurchaseDialogProps) {
  const { l10n } = useLocalization();
  const language = useLanguage();
  const grouping = useGrouping();
  const cancel = useCancelPurchase();
  const [error, setError] = useState<UiError | null>(null);
  // The backend said the cancel takes cash below zero: the next press confirms it.
  const [cashConfirmed, setCashConfirmed] = useState(false);

  const say = (id: string, vars?: Record<string, string>) => l10n.getString(id, vars);
  const cashItem = error?.errors.find((item) => item.code === CASH_CODE);
  const shownError =
    error === null
      ? null
      : { ...error, errors: error.errors.filter((item) => item.code !== CASH_CODE) };

  const confirm = (reason: string) => {
    if (row === null) return;
    setError(null);
    cancel
      .mutateAsync({
        id: row.id,
        version: row.version,
        reason,
        confirm_cash_below_zero: cashConfirmed,
      })
      .then((cancelled) => {
        showToast(
          'success',
          l10n.getString('purchases-cancelled', {
            number: purchaseNumberText(row.number, say),
          }),
        );
        setCashConfirmed(false);
        onCancelled(cancelled);
      })
      .catch((failure: unknown) => {
        const refused = uiErrorOf(failure);
        setCashConfirmed(refused.errors.some((item) => item.code === CASH_CODE));
        setError(refused);
      });
  };

  // The test is on the quantities, which are plain counts: sold or written off, or else it
  // carries a part of a trip cost.
  const blockedBySales = row !== null && (row.qty_sold > 0 || row.qty_written_off > 0);

  return (
    <ReasonDialog
      open={row !== null}
      onOpenChange={(open) => {
        if (open) return;
        setError(null);
        setCashConfirmed(false);
        onClose();
      }}
      title={<Localized id="purchases-cancel-title" />}
      description={
        row === null ? null : (
          <Localized
            id="purchases-cancel-text"
            vars={{
              number: purchaseNumberText(row.number, say),
              item: displayName(row.item, language),
              packing: displayName(row.packing, language),
              qty: String(row.qty),
              cost: formatPaisa(row.paid_now_paisa, { grouping }),
              supplier: supplierText(row.supplier, say),
            }}
            elems={{ num: <Num /> }}
          >
            <span />
          </Localized>
        )
      }
      reasonLabel={<Localized id="purchases-cancel-reason" />}
      confirmLabel={
        <Localized
          id={cashConfirmed ? 'purchases-cancel-confirm-cash' : 'purchases-cancel-confirm'}
        />
      }
      confirmDisabled={row !== null && !row.can_cancel}
      error={shownError}
      busy={cancel.isPending}
      note={
        row !== null && !row.can_cancel ? (
          blockedBySales ? (
            <Localized
              id="purchases-cancel-blocked"
              vars={{ sold: String(row.qty_sold), written_off: String(row.qty_written_off) }}
            >
              <span />
            </Localized>
          ) : (
            <Localized id="purchases-cancel-blocked-cost">
              <span />
            </Localized>
          )
        ) : cashItem === undefined ? undefined : (
          <ErrorMessage item={cashItem}>
            <span />
          </ErrorMessage>
        )
      }
      onConfirm={confirm}
    />
  );
}
