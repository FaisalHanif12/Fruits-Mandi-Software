// What the address of one trip can carry (PRD FR-7.79, plan 55 Section 11.1): the purchase to show in the
// card under the list (`?purchase=7`), and the type of the entry form to open on (`?type=cost`). Kept apart
// from the screen so the router can read the address without loading it (Code Standards F36). A value that
// is not valid is left out, and the screen starts as it always did.

/** The types of the entry form of a trip an address can choose. */
export const TRIP_OPEN_TYPES = ['purchase', 'cost', 'other_sale'] as const;
export type TripOpenType = (typeof TRIP_OPEN_TYPES)[number];

export interface TripSearch {
  purchase?: number;
  type?: TripOpenType;
}

/** A whole positive number of the address, or undefined. */
function positiveId(value: unknown): number | undefined {
  const id = Number(value);
  return Number.isSafeInteger(id) && id > 0 ? id : undefined;
}

export function parseTripSearch(search: Record<string, unknown>): TripSearch {
  const purchase = positiveId(search['purchase']);
  const type = TRIP_OPEN_TYPES.find((candidate) => candidate === search['type']);
  return {
    ...(purchase === undefined ? {} : { purchase }),
    ...(type === undefined ? {} : { type }),
  };
}
