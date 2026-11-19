// The party of the row that has focus in a report, for the button "ڈوبی رقم میں ڈالیں" on R20, R24
// and R32 (PRD FR-10.10). A row key is the row's place in the whole report (`page.offset + index`),
// the way ReportResults makes it. A row with no party cell, a month heading, or nothing focused
// gives null.

import type { ReportId } from '../../bindings/ReportId';
import type { ReportPage } from '../../bindings/ReportPage';
import type { ReportParty } from '../../bindings/ReportParty';

/** The reports whose rows can be moved to the dead amount: the dues, the sellers and the money lent. */
export const MOVABLE_REPORTS: ReadonlySet<ReportId> = new Set(['R20', 'R24', 'R32']);

export function partyOfFocusedRow(
  pages: readonly ReportPage[] | undefined,
  focusRowId: string | null,
): ReportParty | null {
  if (pages === undefined || focusRowId === null) return null;
  const place = Number(focusRowId);
  if (!Number.isSafeInteger(place)) return null;
  for (const page of pages) {
    const row = page.rows[place - page.offset];
    if (row === undefined || place < page.offset) continue;
    if (row.kind !== 'normal') return null;
    for (const cell of row.cells) {
      if (cell.type === 'party') return cell.party;
    }
    return null;
  }
  return null;
}
