// The box that asks for the reason and the Owner's sign before the partners' opening capital is put into روکڑ
// (PRD FR-16.14, FR-21.4, plan 87). It is the box of the reopen-and-fix flow (`ReasonDialog` with a sign field),
// with the words of this action. The backend checks the sign and the reason; a refusal is told inside the box under
// its field and nothing changes. The box does not close itself: the card closes it when the backend has said yes.

import { Localized } from '@fluent/react';
import type { UiError } from '../../../../bindings/UiError';
import { ReasonDialog } from '../../../../shared/ui';

interface PlaceCapitalDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  error: UiError | null;
  busy: boolean;
  onConfirm: (reason: string, sign: string) => void;
}

export function PlaceCapitalDialog({
  open,
  onOpenChange,
  error,
  busy,
  onConfirm,
}: PlaceCapitalDialogProps) {
  return (
    <ReasonDialog
      open={open}
      onOpenChange={onOpenChange}
      title={<Localized id="cashbook-capital-place-title" />}
      description={
        <Localized id="cashbook-capital-place-text">
          <span />
        </Localized>
      }
      reasonLabel={<Localized id="cashbook-capital-place-label" />}
      signLabel={<Localized id="edit-fix-sign-label" />}
      signHint={<Localized id="cashbook-capital-place-sign-hint" />}
      confirmLabel={<Localized id="cashbook-capital-place-confirm" />}
      confirmVariant="primary"
      error={error}
      busy={busy}
      onConfirm={(reason, sign) => {
        onConfirm(reason, sign ?? '');
      }}
    />
  );
}
