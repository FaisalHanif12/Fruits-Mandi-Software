// The "Edited" word of a list row (plan 44, PRD FR-7.71): the status cell of an edited entry reads
// "Saved · Edited", with the time and the name of the last edit in its tip. The word is in the cell,
// so colour is never the only signal (NFR-USE-12). Display only.

import type { EditStamp } from '../../bindings/EditStamp';
import type { ListContext } from '../grid/ListGrid';
import { isoToDisplayDate } from '../format';
import { clockPartsOf } from './time';

/** The words of a status, with "Edited" after them when the entry was edited. */
export function statusWithEdited(
  status: string,
  edited: EditStamp | null,
  context: ListContext,
): string {
  return edited === null ? status : `${status} · ${context.text('edit-mark')}`;
}

/**
 * The tip of an edited row: "Edited: 31-10-2026 4:35 pm, Owner", with the reason after it when one was given
 * (PRD FR-7.71, v1.58). Never the Owner's sign. Nothing for an entry never edited.
 */
export function editedTip(edited: EditStamp | null, context: ListContext): string | undefined {
  if (edited === null) return undefined;
  const clock = clockPartsOf(edited.local_time);
  const time =
    clock === null
      ? edited.local_time
      : context.text('edit-time', {
          hour: clock.hour,
          minute: clock.minute,
          part: clock.part,
          meridian: clock.meridian,
        });
  const vars = { date: isoToDisplayDate(edited.local_date), time, by: edited.by };
  return edited.reason === null || edited.reason.trim() === ''
    ? context.text('edit-mark-tip', vars)
    : context.text('edit-mark-tip-reason', { ...vars, reason: edited.reason });
}
