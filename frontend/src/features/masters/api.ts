// Typed wrappers around the masters commands (plan 06 contract; module `masters`, Tech Stack 5.2).
// Components never call invoke() (Code Standards F4). The commands speak one row type per kind
// (bindings/); `wire.ts` turns them into the `MasterRecord` and `MasterDraft` the screens use.

import type { MasterRow } from '../../bindings/MasterRow';
import { callCommand } from '../../shared/api';
import type { MasterDraft, MasterKind, MasterRecord } from './types';
import { draftToInput, rowToRecord } from './wire';

/** Every master of a kind, in the backend's order. Inactive ones only when asked for. */
export async function mastersList(
  kind: MasterKind,
  includeInactive: boolean,
): Promise<MasterRecord[]> {
  const rows = await callCommand<MasterRow[]>('masters_list', {
    input: { kind, include_inactive: includeInactive },
  });
  return rows.map(rowToRecord);
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
