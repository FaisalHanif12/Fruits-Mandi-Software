// The edit stamp, the edit history and the reopen-and-fix commands (plan 44, PRD FR-7.71, FR-21.4).
// The only file of the edit kit that knows the wire shape (Code Standards F4). Nothing here works
// out money: every figure is the backend's.

import type { DayFixClosed } from '../../bindings/DayFixClosed';
import type { DayFixOpened } from '../../bindings/DayFixOpened';
import type { EditEntity } from '../../bindings/EditEntity';
import type { EditHistory } from '../../bindings/EditHistory';
import type { EditStampRow } from '../../bindings/EditStampRow';
import { callCommand } from '../api';

/** The last edit of each entry in `ids`: the stamp, or null when the entry was never edited. */
export function auditEditStamp(
  entity: EditEntity,
  ids: readonly number[],
): Promise<EditStampRow[]> {
  return callCommand<EditStampRow[]>('audit_edit_stamp', { input: { entity, ids: [...ids] } });
}

/** Every version of one entry, with the old and the new value of each field that changed. */
export function auditEditHistory(entity: EditEntity, id: number): Promise<EditHistory> {
  return callCommand<EditHistory>('audit_edit_history', { input: { entity, id } });
}

/** Reopens a closed date and every later closed date, with the reason (PRD FR-21.4). */
export function dayFixOpen(date: string, reason: string): Promise<DayFixOpened> {
  return callCommand<DayFixOpened>('day_fix_open', { input: { date, reason } });
}

/** Closes the days of a fix again, in order. Answers which closed and which stayed open. */
export function dayFixClose(token: string): Promise<DayFixClosed> {
  return callCommand<DayFixClosed>('day_fix_close', { input: { token } });
}
