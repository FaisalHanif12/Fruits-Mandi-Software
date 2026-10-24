// What the master screens work on, whatever the kind. The commands speak one generated row type
// per kind (bindings/); `wire.ts` turns them into these and back (Code Standards F2).

import type { MasterKind } from '../../bindings/MasterKind';
import type { MastersTotals } from '../../bindings/MastersTotals';
import type { Named } from '../../shared/i18n';
import type { PickerValue } from '../../shared/ui';

export type { MasterKind };

/** One master, the way the screens use it, whatever its kind. */
export interface MasterRecord {
  kind: MasterKind;
  id: number;
  /** Checked when an edit is saved: a stale row is refused (ERR_VERSION_CHANGED). */
  version: number;
  /** Empty for a master with no code (narration, staff). */
  code: string;
  is_active: boolean;
  /** The Urdu and English name. Null for a master with no name (marka). */
  names: Named | null;
  /** One-script fields as typed, and the value of each select (for example `item_group`). */
  text: Readonly<Record<string, string>>;
  /** Records this one points at (for example the default packing of an item). */
  refs: Readonly<Record<string, PickerValue | null>>;
  /** Read-only facts the backend works out (for example the code of the account it made). */
  shown: Readonly<Record<string, string>>;
  /** Lists the form does not edit but must send back unchanged (the "used in" of a narration). */
  lists: Readonly<Record<string, readonly string[]>>;
}

/** One page of a master list: the records, the count and the totals of the whole filter (FR-19.11). */
export interface MasterPage {
  rows: MasterRecord[];
  total_count: number;
  totals: MastersTotals;
}

/** What the form holds for one master being added or edited. Everything is text, as typed. */
export interface MasterDraft {
  id: number | null;
  version: number | null;
  code: string;
  names: { urdu: string; english: string };
  text: Readonly<Record<string, string>>;
  refs: Readonly<Record<string, PickerValue | null>>;
  lists: Readonly<Record<string, readonly string[]>>;
}
