// Cancelling a trip cost or a sale in another shop (PRD FR-12.31, FR-12.32, FR-7.8): a box that
// says exactly what will happen, asks for a reason, and cancels by reversal. Nothing is deleted:
// the posting is reversed and the entry stays in the trip's history, marked cancelled.
//
// A trip cost cannot be cancelled once any purchase it was spread over was sold or written off
// (`ERR_TRIP_COST_HAS_SALES`); the words show inside the box and what was typed stays. A cancel
// of a sale in another shop takes its money out of روکڑ again, so the backend may ask the cash
// question: the box then shows the cash after the cancel and the next press confirms it.

import { Localized, useLocalization } from '@fluent/react';
import { useState } from 'react';
import type { UiError } from '../../../bindings/UiError';
import { uiErrorOf } from '../../../shared/api';
import { ErrorMessage, ReasonDialog, showToast } from '../../../shared/ui';
import { useCancelCost, useCancelOtherSale } from '../hooks';
import { entryNumberText } from '../list-columns';
import type { TripEntryRow } from '../types';

interface CancelEntryDialogProps {
  /** The entry to cancel (a trip cost or a sale in another shop), or null when the box is closed. */
  row: TripEntryRow | null;
  onClose: () => void;
}

const CASH_CODE = 'ERR_CASH_BELOW_ZERO_NOT_CONFIRMED';

export function CancelEntryDialog({ row, onClose }: CancelEntryDialogProps) {
  const { l10n } = useLocalization();
  const cancelCost = useCancelCost();
  const cancelSale = useCancelOtherSale();
  const [error, setError] = useState<UiError | null>(null);
  // The backend said the cancel takes cash below zero: the next press confirms it.
  const [cashConfirmed, setCashConfirmed] = useState(false);
  const say = (id: string, vars?: Record<string, string>) => l10n.getString(id, vars);
  const isCost = row?.kind === 'cost';
  const busy = cancelCost.isPending || cancelSale.isPending;
  const cashItem = error?.errors.find((item) => item.code === CASH_CODE);
  const shownError =
    error === null
      ? null
      : { ...error, errors: error.errors.filter((item) => item.code !== CASH_CODE) };

  const confirm = (reason: string) => {
    if (row === null) return;
    setError(null);
    const input = {
      id: row.id,
      version: row.version,
      reason,
      confirm_cash_below_zero: cashConfirmed,
    };
    const done = (cancelled: { trip_reopened: boolean }) => {
      showToast(
        'success',
        l10n.getString(cancelled.trip_reopened ? 'trips-cancelled-reopened' : 'trips-cancelled', {
          entry: entryNumberText(row.kind, row.number, say),
        }),
      );
      setCashConfirmed(false);
      onClose();
    };
    const failed = (failure: unknown) => {
      const refused = uiErrorOf(failure);
      setCashConfirmed(refused.errors.some((item) => item.code === CASH_CODE));
      setError(refused);
    };
    (isCost ? cancelCost.mutateAsync(input) : cancelSale.mutateAsync(input))
      .then(done)
      .catch(failed);
  };

  return (
    <ReasonDialog
      open={row !== null}
      onOpenChange={(open) => {
        if (open) return;
        setError(null);
        setCashConfirmed(false);
        onClose();
      }}
      title={<Localized id="trips-cancel-title" />}
      description={
        row === null ? null : (
          <Localized
            id={isCost ? 'trips-cancel-text-cost' : 'trips-cancel-text-sale'}
            vars={{ entry: entryNumberText(row.kind, row.number, say) }}
          >
            <span />
          </Localized>
        )
      }
      reasonLabel={<Localized id="trips-cancel-reason" />}
      confirmLabel={
        <Localized id={cashConfirmed ? 'trips-cancel-confirm-cash' : 'trips-cancel-confirm'} />
      }
      error={shownError}
      busy={busy}
      note={
        cashItem === undefined ? undefined : (
          <ErrorMessage item={cashItem}>
            <span />
          </ErrorMessage>
        )
      }
      onConfirm={confirm}
    />
  );
}
