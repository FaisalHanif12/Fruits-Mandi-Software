// Which garden and trip cards the summary shows (PRD FR-12.52, plan 97 item 5). The backend works out one card for each
// garden and each trip that has a complete lot in the بکری, over all its lots. In the whole-بکری view every card shows; in
// the view of one lot only the card of that lot's garden or trip does. Nothing is worked out here: the lot is matched to
// the card by the ids the backend sent (the garden's account is the seller of its lots; a trip lot carries its trip).

import type { GardenCardLines } from '../../bindings/GardenCardLines';
import type { LotPreview } from '../../bindings/LotPreview';
import type { TripCardLines } from '../../bindings/TripCardLines';
import type { EntryRow } from '../../shared/grid';
import { LOT } from './types';

/** The garden cards of the lots shown: those whose garden account is the seller of one of the lots. */
export function gardenCardsFor(
  shown: readonly EntryRow[],
  cards: readonly GardenCardLines[],
): GardenCardLines[] {
  const sellers = new Set(shown.map((row) => row.picked?.[LOT.seller]?.id));
  return cards.filter((card) => sellers.has(card.garden.account.id));
}

/** The trip cards of the lots shown: those whose trip is the trip of one of the lots (`own.trip` of its preview). */
export function tripCardsFor(
  shown: readonly EntryRow[],
  lots: readonly LotPreview[],
  cards: readonly TripCardLines[],
): TripCardLines[] {
  const keys = new Set(shown.map((row) => row.rowId));
  const trips = new Set(lots.filter((lot) => keys.has(lot.key)).map((lot) => lot.own?.trip.id));
  return cards.filter((card) => trips.has(card.trip.id));
}
