// The reason of an edit, asked before the save when the backend says the entry is not of today
// (`reason_required`, PRD FR-7.70). One box for every entry, built on ReasonDialog, so the words and
// the keys are the same everywhere. The screen sends the save and closes the box when the backend
// has said yes; a refusal is shown inside it and what was typed stays.

import { Localized } from '@fluent/react';
import type { UiError } from '../../bindings/UiError';
import { isoToDisplayDate } from '../format';
import { ReasonDialog } from './ReasonDialog';

interface EditReasonDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  /** The day of the entry, `YYYY-MM-DD`. */
  date: string;
  error: UiError | null;
  busy: boolean;
  /** What the box starts with: the reason typed when the days were reopened. */
  initialReason?: string;
  onConfirm: (reason: string) => void;
}

export function EditReasonDialog({
  open,
  onOpenChange,
  date,
  error,
  busy,
  initialReason,
  onConfirm,
}: EditReasonDialogProps) {
  return (
    <ReasonDialog
      open={open}
      onOpenChange={onOpenChange}
      title={<Localized id="edit-reason-title" />}
      description={
        <Localized id="edit-reason-text" vars={{ date: isoToDisplayDate(date) }}>
          <span />
        </Localized>
      }
      reasonLabel={<Localized id="edit-reason-label" />}
      confirmLabel={<Localized id="edit-reason-confirm" />}
      confirmVariant="primary"
      error={error}
      busy={busy}
      {...(initialReason === undefined ? {} : { initialReason })}
      onConfirm={onConfirm}
    />
  );
}
