// The two buttons every saved entry has, in the same place on every screen and in every list
// (plan 44, PRD FR-7.70): تبدیل کریں (Edit) and منسوخ کریں (Cancel). What they do comes from the flags
// on the entry's row: Edit reads "دن کھولیں اور درست کریں" when the day is closed (PRD FR-21.4), and
// an entry that cannot be changed says why in words, next to the buttons, not only in a tooltip.

import { Localized } from '@fluent/react';
import { Ban, Pencil } from 'lucide-react';
import { editLabelId, editStateOf, lockedMessageId, type EntryFlags } from '../edit/flags';
import { Button } from './Button';

interface EntryActionsProps {
  flags: Pick<EntryFlags, 'can_edit' | 'can_cancel' | 'locked_reason' | 'needs_reopen'>;
  onEdit: () => void;
  onCancel: () => void;
  /** The entry has no cancel (a master record). The Edit button is then alone. */
  noCancel?: boolean;
  /** The buttons are working on something now (a day is being reopened). */
  busy?: boolean;
}

export function EntryActions({
  flags,
  onEdit,
  onCancel,
  noCancel = false,
  busy,
}: EntryActionsProps) {
  const state = editStateOf(flags);
  const lockedId = lockedMessageId(flags.locked_reason);
  return (
    <div className="flex flex-col gap-2" data-edit-actions="">
      <div className="flex flex-wrap items-center gap-2">
        <Button
          disabled={state === 'locked'}
          {...(busy === undefined ? {} : { busy })}
          onClick={onEdit}
        >
          <Pencil aria-hidden="true" className="size-icon-md" />
          <Localized id={editLabelId(state)} />
        </Button>
        {noCancel ? null : (
          <Button variant="danger" disabled={!flags.can_cancel} onClick={onCancel}>
            <Ban aria-hidden="true" className="size-icon-md" />
            <Localized id="edit-action-cancel" />
          </Button>
        )}
      </div>
      {lockedId === null ? null : (
        <Localized id={lockedId}>
          <p className="type-body text-ink-muted" />
        </Localized>
      )}
    </div>
  );
}
