// The cards of the summary: the block of each seller shown: the tick "پیشگی اس بکری سے کاٹیں" (PRD FR-12.23, plan 78 C3, Umar
// 17-11-2026: "Take the advance off this sale"), and beside it the lines of his
// account (plan 91, `SellerPaid`). The tick shows for a seller who has an advance open, with the open amount (and,
// since v1.61, the line for the part his credit already covers), and it is OFF when a بکری opens. One tick covers every lot of that seller in the بکری, so the summary of one lot and the
// whole بکری show the same tick. The amount is the party card's (the backend's), written with formatPaisa; nothing
// is worked out here (CLAUDE.md rule 1).

import { Localized } from '@fluent/react';
import type { EntryRow, PickedRecord } from '../../../shared/grid';
import { usePartyCard } from '../../../shared/accounts';
import { formatPaisa, isPositivePaisa, useGrouping } from '../../../shared/format';
import { displayName, useLanguage } from '../../../shared/i18n';
import { AdvanceCoveredLine, Checkbox, Num } from '../../../shared/ui';
import { isSellerTicked, setSellerTick } from '../advance-tick';
import { useBikriDraft } from '../draft-store';
import { tickLotsOf } from '../draft';
import type { GardenCardLines } from '../../../bindings/GardenCardLines';
import type { LotPreview } from '../../../bindings/LotPreview';
import type { SellerPaidLines } from '../../../bindings/SellerPaidLines';
import type { TripCardLines } from '../../../bindings/TripCardLines';
import { LOT } from '../types';
import { gardenCardsFor, tripCardsFor } from '../venture-cards';
import { SellerPaid } from './SellerPaid';
import { GardenCard, TripCard } from './VentureCards';

interface SellerTickProps {
  seller: PickedRecord;
  rows: readonly EntryRow[];
}

function SellerTick({ seller, rows }: SellerTickProps) {
  const stores = useBikriDraft();
  const grouping = useGrouping();
  const language = useLanguage();
  const card = usePartyCard(seller.id);
  const uncovered = card.data?.advance_open ?? null;
  const covered = card.data?.advance_covered ?? null;
  const hasUncovered = uncovered !== null && isPositivePaisa(uncovered);
  // The tick shows for a seller who has an advance open (PRD 12.3), covered by his credit or not: the sale can
  // take it off either way (matching does not change, PRD FR-9.6 v1.61).
  if (!hasUncovered && covered === null) return null;
  return (
    <div className="flex flex-col gap-1" data-advance-tick={seller.id}>
      <Checkbox
        checked={isSellerTicked(rows, seller.id)}
        onChange={(on) => {
          setSellerTick(rows, seller.id, on, (rowId, field, text) => {
            stores.lots.getState().updateTyped(rowId, field, text);
          });
        }}
        label={
          <Localized
            id="bikri-summary-take-advance"
            vars={{ name: displayName(seller, language) }}
          />
        }
      />
      {hasUncovered ? (
        <Localized
          id="bikri-more-advance-open"
          vars={{ amount: formatPaisa(uncovered, { grouping }) }}
          elems={{ num: <Num /> }}
        >
          <span className="type-body text-ink-muted" />
        </Localized>
      ) : null}
      {covered === null ? null : <AdvanceCoveredLine amount={covered} />}
    </div>
  );
}

interface AdvanceTicksProps {
  /** The lot rows whose sellers are listed: the selected lot, or every lot of the بکری. */
  shown: readonly EntryRow[];
  /** What the preview says about each seller's account in this بکری (plan 91); empty while nothing is worked out. */
  sellers?: readonly SellerPaidLines[] | undefined;
  /**
   * The garden and trip cards of the backend's preview (plan 97 item 5): one card each, in the same grid. The whole-sale view
   * shows every card; the view of one lot shows the card of that lot's garden or trip (`previewLots` says a trip lot's trip).
   */
  gardens?: readonly GardenCardLines[] | undefined;
  trips?: readonly TripCardLines[] | undefined;
  previewLots?: readonly LotPreview[] | undefined;
  whole?: boolean | undefined;
}

export function AdvanceTicks({
  shown,
  sellers: summaries = [],
  gardens: allGardens = [],
  trips: allTrips = [],
  previewLots = [],
  whole = false,
}: AdvanceTicksProps) {
  const stores = useBikriDraft();
  const rows = stores.lots((state) => state.rows);
  // Each seller once, in the order of the lots; a lot with no tick to take (own goods, a garden, carried) adds none.
  const sellers: PickedRecord[] = [];
  for (const row of shown) {
    const seller = row.picked?.[LOT.seller];
    if (seller === undefined || sellers.some((known) => known.id === seller.id)) continue;
    if (tickLotsOf(rows, seller.id).length > 0) sellers.push(seller);
  }
  const gardens = whole ? allGardens : gardenCardsFor(shown, allGardens);
  const trips = whole ? allTrips : tripCardsFor(shown, previewLots, allTrips);
  if (sellers.length === 0 && gardens.length === 0 && trips.length === 0) return null;
  // The cards stand 2 or 3 to a row when the summary itself is wide enough (it follows the summary's own width, not the
  // window's: a container query), and one under the other in a narrow one, so 20 or 50 sellers stay short (PRD FR-12.52).
  return (
    <div className="@container min-w-0">
      <div className="grid grid-cols-1 gap-2 @form3:grid-cols-2 @form4:grid-cols-3">
        {sellers.map((seller) => (
          <SellerPaid
            key={seller.id}
            seller={seller}
            lines={summaries.find((one) => one.seller_account_id === seller.id)}
            tick={<SellerTick seller={seller} rows={rows} />}
          />
        ))}
        {gardens.map((card) => (
          <GardenCard key={`garden-${String(card.garden.id)}`} card={card} />
        ))}
        {trips.map((card) => (
          <TripCard key={`trip-${String(card.trip.id)}`} card={card} />
        ))}
      </div>
    </div>
  );
}
