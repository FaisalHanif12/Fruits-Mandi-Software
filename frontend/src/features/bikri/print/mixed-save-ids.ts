// The بکری a mixed save wrote (plan 76 point 3, PRD FR-12.48): the new بکری and the old بکری of each carried lot.
// The backend lists them in the answer of the save (`print_bikri_ids`, no duplicates). The sale print and the buyer
// parchi take them as `bikri_ids`, so "this save only" covers everything that save sold, not just the new بکری.
// The saved screen does not know the old بکری (a carried lot is a later sale of an older one), so the screen that
// made the save remembers them here, by the new بکری, for the rest of the session. After a reload there are no
// ids, and the print box offers the whole day and this بکری only, as for any بکری. Ids and numbers are the
// backend's; nothing is worked out (CLAUDE.md rule 1).

import type { MixedSaleSaved } from '../../../bindings/MixedSaleSaved';

/** The بکری of one mixed save: the ids to send and their numbers to show, new بکری first. */
export interface MixedSaveBikri {
  ids: readonly number[];
  numbers: readonly number[];
}

/** How many saves are kept: the screen only looks at the one just made, or a few before it. */
const KEPT = 20;

const saves = new Map<number, MixedSaveBikri>();

/**
 * Remembers a mixed save that made a new بکری. A save with only carried lots has none: its lots print with
 * their own بکری, and the whole-day print is the print box's choice, so nothing is kept for it.
 */
export function rememberMixedSave(answer: MixedSaleSaved): void {
  if (answer.new_bikri_id === null || answer.new_bikri_number === null) return;
  // The number of each بکری to show: the new one's, and the old one of each carried lot.
  const numberOf = new Map<number, number>([[answer.new_bikri_id, answer.new_bikri_number]]);
  for (const lot of answer.carried) numberOf.set(lot.old_bikri_id, lot.old_number);
  const ids = [...answer.print_bikri_ids];
  const numbers = ids.flatMap((id) => {
    const number = numberOf.get(id);
    return number === undefined ? [] : [number];
  });
  saves.delete(answer.new_bikri_id);
  saves.set(answer.new_bikri_id, { ids, numbers });
  const oldest = saves.keys().next();
  if (saves.size > KEPT && oldest.done !== true) saves.delete(oldest.value);
}

/** The بکری of the mixed save that made this new بکری, or null when it was not made by one (or has only itself). */
export function mixedSaveOf(newBikriId: number): MixedSaveBikri | null {
  const found = saves.get(newBikriId);
  return found === undefined || found.ids.length < 2 ? null : found;
}

/** Forgets everything (a sign-out, and the tests). */
export function forgetMixedSaves(): void {
  saves.clear();
}
