// "Reopen the day and fix" (plan 44, PRD FR-21.4): the box that asks for the reason and says what
// will happen, before the days of a closed entry are reopened. The screen reopens them when the
// user confirms, and closes them again after the save or when the user leaves.

import { Localized } from '@fluent/react';
import type { UiError } from '../../bindings/UiError';
import { isoToDisplayDate } from '../format';
import { ReasonDialog } from './ReasonDialog';

interface FixDayDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  /** The day of the entry, `YYYY-MM-DD`. The closed days after it open too. */
  date: string;
  error: UiError | null;
  busy: boolean;
  onConfirm: (reason: string, sign: string) => void;
}

export function FixDayDialog({
  open,
  onOpenChange,
  date,
  error,
  busy,
  onConfirm,
}: FixDayDialogProps) {
  return (
    <ReasonDialog
      open={open}
      onOpenChange={onOpenChange}
      title={<Localized id="edit-fix-title" />}
      description={
        <Localized id="edit-fix-text" vars={{ date: isoToDisplayDate(date) }}>
          <span />
        </Localized>
      }
      reasonLabel={<Localized id="edit-fix-label" />}
      // Only the Owner's sign opens the day (PRD FR-21.4, plan 73): a second, required field below the reason.
      signLabel={<Localized id="edit-fix-sign-label" />}
      signHint={<Localized id="edit-fix-sign-hint" />}
      confirmLabel={<Localized id="edit-fix-confirm" />}
      confirmVariant="primary"
      error={error}
      busy={busy}
      onConfirm={(reason, sign) => {
        onConfirm(reason, sign ?? '');
      }}
    />
  );
}
