// What the two buttons of a saved entry do, from the flags the backend sends on its row (plan 44,
// PRD FR-7.70, FR-21.4). One place, so every screen and every list row reads the flags the same
// way. Pure: nothing here looks at a date or works out whether a day is closed. The backend does.

import type { EntryFlags } from '../../bindings/EntryFlags';
import type { LockedReason } from '../../bindings/LockedReason';

/** The flags a row carries. Every saved-entry row has them as its own fields. */
export type { EntryFlags, LockedReason };

/** The flags shown before a row is chosen: both buttons off, nothing to say. */
export const NO_ENTRY_FLAGS = {
  can_edit: false,
  can_cancel: false,
  locked_reason: null,
  needs_reopen: false,
} as const;

/** How the Edit button stands: it opens the entry, it first reopens the day, or it is off. */
export type EditState = 'ready' | 'reopen' | 'locked';

export function editStateOf(flags: Pick<EntryFlags, 'can_edit' | 'needs_reopen'>): EditState {
  if (!flags.can_edit) return 'locked';
  return flags.needs_reopen ? 'reopen' : 'ready';
}

/** The Fluent message of the Edit button for a state. */
export function editLabelId(state: EditState): string {
  return state === 'reopen' ? 'edit-action-fix-day' : 'edit-action-edit';
}

/**
 * The Fluent message that tells why an entry cannot be changed, or null when it can. A code this
 * version of the app does not know still gets a plain sentence, never a raw code.
 */
export function lockedMessageId(reason: LockedReason | null): string | null {
  if (reason === null) return null;
  const known: readonly LockedReason[] = [
    'goods_moved',
    'lot_seller_locked',
    'garden_closed',
    'month_closed',
    'cancelled',
    'returns_stand',
  ];
  return known.includes(reason)
    ? `edit-locked-${reason.replaceAll('_', '-')}`
    : 'edit-locked-unknown';
}
