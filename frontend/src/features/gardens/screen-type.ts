// What the address of one garden can ask its screen to open on (PRD FR-7.79, plan 55 Section 11.1): one of
// the types of its entry form, or the closing of the garden. Kept apart from the screen so the router can
// read the address without loading it (Code Standards F36). A value that is not one of these is left out, and
// the screen starts as it always did, on the first type.

/** The entry types an address can choose, and `close`: the screen scrolls to the closing and focuses its button. */
export const GARDEN_OPEN_TYPES = ['installment', 'cost', 'other_sale', 'close'] as const;
export type GardenOpenType = (typeof GARDEN_OPEN_TYPES)[number];

export interface GardenSearch {
  type?: GardenOpenType;
}

export function parseGardenSearch(search: Record<string, unknown>): GardenSearch {
  const type = GARDEN_OPEN_TYPES.find((candidate) => candidate === search['type']);
  return type === undefined ? {} : { type };
}
