// The filters of one party's day slip, R35 for one account and one date (PRD FR-12.43 level 3,
// FR-19.12): the party ledgers screen opens the slip of a date of a party's statement. It is the
// same request the report screen makes when a party is picked in its box, so the slip, its print and
// its Excel sheet are the report's own and no figure can differ from it (plan 66).

import type { ReportFilterKind } from '../../bindings/ReportFilterKind';
import type { ReportFilters } from '../../bindings/ReportFilters';
import { isoToDisplayDate } from '../../shared/format';
import { EMPTY_FILTERS, filtersToWire } from './filters';

/** The boxes R35 takes that a day slip fills. */
export const PARTY_DAY_TAKES: readonly ReportFilterKind[] = ['as_of', 'party'];

/**
 * The filters of the day slip of `accountId` on `date` (`YYYY-MM-DD`, as the statement sends it).
 * The date goes as `DD-MM-YYYY`, the way the backend reads a typed date.
 */
export function partyDayFilters(accountId: number, date: string): ReportFilters {
  return filtersToWire(
    {
      ...EMPTY_FILTERS,
      asOf: isoToDisplayDate(date),
      party: { id: accountId, name_ur: '', name_en: '', code: '' },
    },
    PARTY_DAY_TAKES,
  );
}
