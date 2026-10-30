// Typed wrappers around the masters commands (plan 06 contract; module `masters`, Tech Stack 5.2).
// Components never call invoke() (Code Standards F4). The commands speak one row type per kind
// (bindings/); `wire.ts` turns them into the `MasterRecord` and `MasterDraft` the screens use.

import type { DeadMarked } from '../../bindings/DeadMarked';
import type { DeadMarkInput } from '../../bindings/DeadMarkInput';
import type { DeadPreview } from '../../bindings/DeadPreview';
import type { DeadPreviewInput } from '../../bindings/DeadPreviewInput';
import type { DeadUndone } from '../../bindings/DeadUndone';
import type { DeadUndoInput } from '../../bindings/DeadUndoInput';
import type { MasterRow } from '../../bindings/MasterRow';
import type { MastersList } from '../../bindings/MastersList';
import { callCommand } from '../../shared/api';
import type { MasterDraft, MasterKind, MasterPage, MasterRecord } from './types';
import { draftToInput, rowToRecord } from './wire';

/**
 * One page of the masters of a kind, in the backend's order, with the count and the totals of the
 * whole filter (PRD FR-19.11). Inactive ones only when asked for.
 */
export async function mastersList(
  kind: MasterKind,
  includeInactive: boolean,
  offset: number,
  limit: number,
): Promise<MasterPage> {
  const page = await callCommand<MastersList>('masters_list', {
    input: { kind, include_inactive: includeInactive, id: null, offset, limit },
  });
  return { rows: page.rows.map(rowToRecord), total_count: page.total_count, totals: page.totals };
}

/**
 * One master of a kind by its id, for its form (PRD 10.3): the same list command with the id filter,
 * so one record comes back and not the pages before it. Null when there is no such master. The
 * inactive ones are asked for, because a form opens a master that was switched off too.
 */
export async function mastersGetOne(kind: MasterKind, id: number): Promise<MasterRecord | null> {
  const page = await callCommand<MastersList>('masters_list', {
    input: { kind, include_inactive: true, id, offset: 0, limit: 0 },
  });
  const row = page.rows[0];
  return row === undefined ? null : rowToRecord(row);
}

/**
 * Creates (no id) or edits (with id and the version that was read) one master. Both names are
 * checked by the backend again, and so is the spelling (`confirmed_rules_words`, PRD FR-7.66).
 */
export async function mastersSave(
  kind: MasterKind,
  draft: MasterDraft,
  confirmedRulesWords: readonly string[],
): Promise<MasterRecord> {
  const row = await callCommand<MasterRow>('masters_save', {
    input: draftToInput(kind, draft, confirmedRulesWords),
  });
  return rowToRecord(row);
}

/** Makes a master inactive or active again. There is no delete (PRD E21). */
export async function mastersSetActive(
  kind: MasterKind,
  id: number,
  isActive: boolean,
): Promise<MasterRecord> {
  const row = await callCommand<MasterRow>('masters_set_active', {
    input: { kind, id, is_active: isActive },
  });
  return rowToRecord(row);
}

/** The next free code of a kind, to put in the code field of a new master. Empty if it has none. */
export function mastersNextCode(kind: MasterKind): Promise<string> {
  return callCommand<string>('masters_next_code', { input: { kind } });
}

/**
 * What moving a person to the dead amount would do (PRD FR-10.10): what he owes now, whether he can
 * be moved and why not (an error code), his last entry and his linked account. Read only.
 */
export function deadPreview(input: DeadPreviewInput): Promise<DeadPreview> {
  return callCommand<DeadPreview>('masters_dead_preview', { input });
}

/** Moves a person who owes the shop to the dead amount: nothing posts, the account turns inactive. */
export function deadMark(input: DeadMarkInput): Promise<DeadMarked> {
  return callCommand<DeadMarked>('masters_dead_mark', { input });
}

/** Makes a person active again ("دوبارہ فعال کریں"): the move ends. A reason is required. */
export function deadUndo(input: DeadUndoInput): Promise<DeadUndone> {
  return callCommand<DeadUndone>('masters_dead_undo', { input });
}
