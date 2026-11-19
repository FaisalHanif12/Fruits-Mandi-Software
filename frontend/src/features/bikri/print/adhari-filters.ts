// What the print menu of the sale book asks for (PRD FR-12.12, FR-19.15, plan 68, plan 76 point 3, plan 78 A1): the
// whole day of the بکری on screen, or this بکری only, as the buyer slips, the seller bills or the whole sale print.
// The backend works out every figure; this only says which slips to read.

import type { DaySlipsScope } from '../../reports';

/** Which lines the print holds: the day of the بکری (the default), or that بکری alone. */
export type AdhariScope = 'day' | 'bikri';

/** What is printed: the buyer slips (the default), the seller bills or the whole sale print. */
export type SlipsKind = 'parchi' | 'seller-bills' | 'sale';

/**
 * The slips to read for the scope. The day is the posting day of the بکری (`YYYY-MM-DD`, as the backend stored
 * it): every بکری of that day is in it. This بکری only is its own sale lines of that day; no buyers are
 * picked, so every buyer of the lines is meant. `saveIds` are the بکری of the mixed save that made this one
 * (the new بکری and the old بکری of each carried lot): then "this بکری only" is that whole save.
 */
export function daySlipsScope(
  scope: AdhariScope,
  entryDate: string,
  bikriId: number,
  saveIds: readonly number[] | null = null,
): DaySlipsScope {
  const own = scope === 'bikri';
  return {
    date: entryDate,
    bikri_id: own ? bikriId : null,
    bikri_ids: own && saveIds !== null ? [...saveIds] : null,
    buyer_ids: null,
  };
}
