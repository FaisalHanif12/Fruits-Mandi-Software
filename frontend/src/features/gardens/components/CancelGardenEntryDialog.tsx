// Cancelling an installment, a running cost or a sale in another shop (PRD FR-12.35, FR-12.36,
// FR-7.8): a box that says exactly what will happen, asks for a reason, and cancels by reversal.
// Nothing is deleted: the posting is reversed and the entry stays in the garden's history, marked
// cancelled. While the garden is closed nothing can be cancelled (`ERR_GARDEN_CLOSED`): the words
// show inside the box and what was typed stays; the closing is undone first.
//
// A cancel of a sale in another shop takes its money out of روکڑ again, so the backend may ask the
// cash question: the box then shows the cash after the cancel and the next press confirms it.

import { Localized, useLocalization } from '@fluent/react';
import { useState } from 'react';
import type { UiError } from '../../../bindings/UiError';
import { uiErrorOf } from '../../../shared/api';
import { ErrorMessage, ReasonDialog, showToast } from '../../../shared/ui';
import { useCancelGardenEntry } from '../hooks';
import { historyKindText } from '../list-columns';
import type { GardenHistoryRow } from '../types';

interface CancelGardenEntryDialogProps {
  /** The entry to cancel, or null when the box is closed. */
  row: GardenHistoryRow | null;
  onClose: () => void;
}

const CASH_CODE = 'ERR_CASH_BELOW_ZERO_NOT_CONFIRMED';

export function CancelGardenEntryDialog({ row, onClose }: CancelGardenEntryDialogProps) {
  const { l10n } = useLocalization();
  const cancel = useCancelGardenEntry();
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
    const entryId = row?.entry_id ?? null;
    const version = row?.version ?? null;
    if (row === null || entryId === null || version === null) return;
    setError(null);
    cancel
      .mutateAsync({
        id: entryId,
        version,
        reason,
        confirm_cash_below_zero: cashConfirmed,
      })
      .then(() => {
        showToast(
          'success',
          l10n.getString('gardens-cancelled', { entry: historyKindText(row.kind, say) }),
        );
        setCashConfirmed(false);
        onClose();
      })
      .catch((failure: unknown) => {
        const refused = uiErrorOf(failure);
        setCashConfirmed(refused.errors.some((item) => item.code === CASH_CODE));
        setError(refused);
      });
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
      title={<Localized id="gardens-cancel-title" />}
      description={
        row === null ? null : (
          <Localized id="gardens-cancel-text" vars={{ entry: historyKindText(row.kind, say) }}>
            <span />
          </Localized>
        )
      }
      reasonLabel={<Localized id="gardens-cancel-reason" />}
      confirmLabel={
        <Localized id={cashConfirmed ? 'gardens-cancel-confirm-cash' : 'gardens-cancel-confirm'} />
      }
      error={shownError}
      busy={cancel.isPending}
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
