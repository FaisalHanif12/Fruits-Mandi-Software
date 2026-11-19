// What the address of the seller dues page can carry (PRD FR-9.6, FR-9.1, plan 79 item 2a): the side the
// tile opened it on. The tile بیوپاریوں کو ادائیگی باقی opens `?side=shop_owes`, the tile بیوپاریوں کے
// ذمے `?side=seller_owes`, and the page opens on سب with no side. The party is not in the address: the
// page always starts on سب (PRD FR-19.1), and the filter is the user's. Anything that does not read is
// left out, so a bad address shows سب.

import type { SellerDuesSide } from '../../bindings/SellerDuesSide';

export interface SellerDuesSearch {
  side?: SellerDuesSide;
}

/** The sides, in the order of the side filter: سب is the first and the start. */
export const SELLER_DUES_SIDES: readonly SellerDuesSide[] = ['all', 'shop_owes', 'seller_owes'];

/** The side a value from an address means, or null for any other value. */
export function asSellerDuesSide(value: unknown): SellerDuesSide | null {
  return SELLER_DUES_SIDES.find((side) => side === value) ?? null;
}

export function parseSellerDuesSearch(raw: Record<string, unknown>): SellerDuesSearch {
  const side = asSellerDuesSide(raw['side']);
  return side === null || side === 'all' ? {} : { side };
}
