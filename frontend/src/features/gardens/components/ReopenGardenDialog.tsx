// Undoing the closing of a garden (PRD FR-12.38): a box that says exactly what will happen, asks for
// a reason, and undoes the closing by reversal while its day is open. Nothing is deleted: the
// closing entry is reversed and stays in the garden's history. A refusal (the day of the closing
// is not open any more) shows inside the box, and what was typed stays.

import { Localized, useLocalization } from '@fluent/react';
import { useState } from 'react';
import type { UiError } from '../../../bindings/UiError';
import { uiErrorOf } from '../../../shared/api';
import { ReasonDialog, showToast } from '../../../shared/ui';
import { useReopenGarden } from '../hooks';
import { gardenNumberText } from '../list-columns';
import type { GardenRow } from '../types';

interface ReopenGardenDialogProps {
  garden: GardenRow;
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export function ReopenGardenDialog({ garden, open, onOpenChange }: ReopenGardenDialogProps) {
  const { l10n } = useLocalization();
  const reopen = useReopenGarden();
  const [error, setError] = useState<UiError | null>(null);
  const say = (id: string, vars?: Record<string, string>) => l10n.getString(id, vars);

  const confirm = (reason: string) => {
    setError(null);
    reopen
      .mutateAsync({ id: garden.id, version: garden.version, reason })
      .then(() => {
        showToast(
          'success',
          l10n.getString('gardens-reopened-toast', {
            number: gardenNumberText(garden.number, say),
          }),
        );
        onOpenChange(false);
      })
      .catch((failure: unknown) => {
        setError(uiErrorOf(failure));
      });
  };

  return (
    <ReasonDialog
      open={open}
      onOpenChange={(next) => {
        if (!next) setError(null);
        onOpenChange(next);
      }}
      title={<Localized id="gardens-reopen-title" />}
      description={<Localized id="gardens-reopen-text" />}
      reasonLabel={<Localized id="gardens-reopen-reason" />}
      confirmLabel={<Localized id="gardens-reopen-confirm" />}
      confirmVariant="primary"
      error={error}
      busy={reopen.isPending}
      onConfirm={confirm}
    />
  );
}
