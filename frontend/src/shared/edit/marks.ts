// The marks inside an edited entry (plan 44, PRD FR-7.71): each field, line or lot that was changed
// by an edit carries a small "Before 600" with the time and name of that edit in its tip. They are
// made from the entry's history (the audit log), never typed and never changed. The history names a
// field by its stored path, for example `lots[0].sales[0].seller_rate_paisa`; the last edit that
// changed a path gives its mark. Pure: nothing here asks the backend or works out money.

import type { AuditValue } from '../../bindings/AuditValue';
import type { EditHistory } from '../../bindings/EditHistory';
import type { EditVersion } from '../../bindings/EditVersion';
import type { SearchScope } from '../../bindings/SearchScope';

/** What a changed field held before and holds now, and the edit that changed it. */
export interface EntryMark {
  before: AuditValue;
  after: AuditValue;
  version: EditVersion;
}

/** The marks of one entry by the path of the field. */
export type MarkMap = ReadonlyMap<string, EntryMark>;

export const NO_MARKS: MarkMap = new Map();

/**
 * The last change of every path: the versions come oldest first, so a later one replaces an earlier
 * one. A field that was changed back to what it was still carries its mark (it was changed).
 */
export function markMapOf(history: EditHistory | undefined): MarkMap {
  if (history === undefined || history.versions.length === 0) return NO_MARKS;
  const marks = new Map<string, EntryMark>();
  for (const version of history.versions) {
    for (const change of version.changes) {
      marks.set(change.path, { before: change.old, after: change.new, version });
    }
  }
  return marks;
}

/** What a stored value stands for, so its words can be found. */
export type MarkKind =
  | { kind: 'plain' }
  /** Whole grams, shown as kilograms. */
  | { kind: 'grams' }
  /** The id of an account (a party, a khata, a person): its name is read. */
  | { kind: 'account' }
  /** The id of a master record: its name is read. */
  | { kind: 'master'; scope: SearchScope }
  /** The id of a record this layer cannot name (a partner): the words say it was another one. */
  | { kind: 'other' };

/** The key of a path: the part after the last dot. */
export function keyOfPath(path: string): string {
  return path.slice(path.lastIndexOf('.') + 1);
}

const MASTER_OF_KEY: Readonly<Record<string, SearchScope>> = {
  item_id: 'item',
  packing_id: 'packing',
  adda_id: 'adda',
  store_id: 'store',
  expense_head_id: 'expense_head',
  collector_staff_id: 'staff',
};

/** What the value at a path is, from the name of its last key (the audit uses the column names). */
export function markKindOf(path: string): MarkKind {
  const key = keyOfPath(path);
  if (key.endsWith('account_id')) return { kind: 'account' };
  const scope = MASTER_OF_KEY[key];
  if (scope !== undefined) return { kind: 'master', scope };
  if (key === 'partner_id' || key === 'lot_id' || key === 'bank_account_id')
    return { kind: 'other' };
  if (key.endsWith('_grams')) return { kind: 'grams' };
  return { kind: 'plain' };
}

/** The ids a set of marks needs names for, once each: `account:12`, `item:3`. */
export function nameRequestsOf(marks: MarkMap): { key: string; kind: MarkKind; id: number }[] {
  const wanted = new Map<string, { key: string; kind: MarkKind; id: number }>();
  for (const [path, mark] of marks) {
    const kind = markKindOf(path);
    if (kind.kind !== 'account' && kind.kind !== 'master') continue;
    for (const value of [mark.before, mark.after]) {
      if (value.kind !== 'number') continue;
      const key =
        kind.kind === 'account'
          ? `account:${String(value.value)}`
          : `${kind.scope}:${String(value.value)}`;
      wanted.set(key, { key, kind, id: value.value });
    }
  }
  return [...wanted.values()];
}

/** The key under which the name of a stored id is kept, or null for a value that is not an id. */
export function nameKeyOf(path: string, value: AuditValue): string | null {
  const kind = markKindOf(path);
  if (value.kind !== 'number') return null;
  if (kind.kind === 'account') return `account:${String(value.value)}`;
  if (kind.kind === 'master') return `${kind.scope}:${String(value.value)}`;
  return null;
}
