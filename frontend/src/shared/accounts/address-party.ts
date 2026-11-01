// The party an entry screen's address names (plan 49, PRD FR-7.79): `/bikri/voucher?type=receipt&party=
// {...}` opens a new voucher with that account already chosen, the way a button under an answer of the
// question panel does. The party is a whole picker value (id, names, code, tag), so the field can show
// who he is at once; a bare id or anything else is left out and the screen starts as it always did.
// Nothing is saved and no amount is filled by it (PRD Section 7.6: the app never fills money).

import type { PickerValue } from '../ui';
import { isRecord, pickedOrNull } from '../screen-state';

/** The party of an address: a whole picker value (with its tag, if it has one), or nothing. */
export function partyOfAddress(value: unknown): PickerValue | undefined {
  const found = pickedOrNull(value);
  if (found === null || !isRecord(value)) return undefined;
  const tag = value['tag'];
  return typeof tag === 'string' && tag !== '' ? { ...found, tag } : found;
}
