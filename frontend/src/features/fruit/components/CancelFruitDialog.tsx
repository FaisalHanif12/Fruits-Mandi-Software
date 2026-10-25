// Cancelling an entry (PRD FR-16.12, FR-7.8): a box that says exactly what will happen, asks for a
// reason, and cancels by reversal. Nothing is deleted: the posting is reversed, the lot's balance
// and the seller's account go back, and the entry stays in the list marked cancelled. No cash moves
// on this screen, so there is no cash question. A refusal (the day is closed) shows inside the box
// and what was typed stays.

import { Localized, useLocalization } from '@fluent/react';
import { useState } from 'react';
import type { UiError } from '../../../bindings/UiError';
import { uiErrorOf } from '../../../shared/api';
import { ReasonDialog, showToast } from '../../../shared/ui';
import { useCancelFruit } from '../hooks';
import { fruitNumberText } from '../list-columns';
import type { FruitRow } from '../types';

interface CancelFruitDialogProps {
  /** The entry to cancel, or null when the box is closed. */
  row: FruitRow | null;
  onClose: () => void;
}

export function CancelFruitDialog({ row, onClose }: CancelFruitDialogProps) {
  const { l10n } = useLocalization();
  const cancel = useCancelFruit();
  const [error, setError] = useState<UiError | null>(null);
  const say = (id: string, vars?: Record<string, string>) => l10n.getString(id, vars);

  const confirm = (reason: string) => {
    if (row === null) return;
    setError(null);
    cancel
      .mutateAsync({ id: row.id, version: row.version, reason })
      .then(() => {
        showToast(
          'success',
          l10n.getString('fruit-cancelled-toast', { number: fruitNumberText(row.number, say) }),
        );
        onClose();
      })
      .catch((failure: unknown) => {
        setError(uiErrorOf(failure));
      });
  };

  return (
    <ReasonDialog
      open={row !== null}
      onOpenChange={(open) => {
        if (open) return;
        setError(null);
        onClose();
      }}
      title={<Localized id="fruit-cancel-title" />}
      description={
        row === null ? null : (
          <Localized id="fruit-cancel-text" vars={{ number: fruitNumberText(row.number, say) }}>
            <span />
          </Localized>
        )
      }
      reasonLabel={<Localized id="fruit-cancel-reason" />}
      confirmLabel={<Localized id="fruit-cancel-confirm" />}
      error={error}
      busy={cancel.isPending}
      onConfirm={confirm}
    />
  );
}
