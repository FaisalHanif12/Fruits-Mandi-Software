// The filters of the R5 print that the sale book's second print button asks for (PRD FR-12.12,
// plan 68, D8 Feature 8): "the whole day" of the بکری on screen, or "this بکری only". R5 is the
// report ادھاری تفصیل: one slip for each buyer, with what he paid on the spot and what is left.
// The backend works out every figure; this only says which lines to read.

import type { ReportFilters } from '../../../bindings/ReportFilters';

/** What the second print button prints: the day of the بکری (the default), or that بکری alone. */
export type AdhariScope = 'day' | 'bikri';

/**
 * The filters for the scope. The day is the posting day of the بکری (`YYYY-MM-DD`, as the backend
 * stored it), from and to the same day, so every بکری of that day is in it. This بکری only is its
 * number's own lines whatever their day (a later sale of it is part of it), with no dates.
 */
export function adhariFilters(
  scope: AdhariScope,
  entryDate: string,
  bikriId: number,
): ReportFilters {
  return {
    shortcut: null,
    from: scope === 'day' ? entryDate : '',
    to: scope === 'day' ? entryDate : '',
    as_of: '',
    party: null,
    seller_id: null,
    zamindar_id: null,
    item_id: null,
    city_id: null,
    adda_id: null,
    detail: false,
    partner_id: null,
    garden_id: null,
    bikri_id: scope === 'bikri' ? bikriId : null,
  };
}
