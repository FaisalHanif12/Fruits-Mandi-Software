// The flags every saved-entry row carries (plan 44, contract T1): what the two buttons may do, why
// an entry is locked, whether the day must be reopened first, and its last edit. A test helper
// builds a row with `OPEN_ENTRY` and changes the one flag a test is about.

import type { EntryFlags } from '../../src/bindings/EntryFlags';

/** An entry of an open day that nothing stops from being changed or cancelled. */
export const OPEN_ENTRY: EntryFlags = {
  can_edit: true,
  can_cancel: true,
  locked_reason: null,
  needs_reopen: false,
  edited: null,
};

/** An entry of a closed day: Edit first reopens the day (PRD FR-21.4). */
export const CLOSED_DAY_ENTRY: EntryFlags = { ...OPEN_ENTRY, needs_reopen: true };

/** An entry that can no longer be changed or cancelled, for the reason given. */
export function lockedEntry(reason: NonNullable<EntryFlags['locked_reason']>): EntryFlags {
  return { ...OPEN_ENTRY, can_edit: false, can_cancel: false, locked_reason: reason };
}
