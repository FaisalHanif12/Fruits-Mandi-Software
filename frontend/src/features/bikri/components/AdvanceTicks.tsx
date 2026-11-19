// The tick "پیشگی اس بکری سے کاٹیں" of each seller shown in the summary (PRD FR-12.23, plan 78 C3, Umar
// 17-11-2026): "Take the advance off this sale". It shows for a seller who has an advance open, with the open
// amount, and it is OFF when a بکری opens. One tick covers every lot of that seller in the بکری, so the
// summary of one lot and the whole بکری show the same tick. The amount is the party card's (the backend's),
// written with formatPaisa; nothing is worked out here (CLAUDE.md rule 1).

import { Localized } from '@fluent/react';
import type { EntryRow, PickedRecord } from '../../../shared/grid';
import { usePartyCard } from '../../../shared/accounts';
import { formatPaisa, useGrouping } from '../../../shared/format';
import { displayName, useLanguage } from '../../../shared/i18n';
import { Checkbox, Num } from '../../../shared/ui';
import { isSellerTicked, setSellerTick } from '../advance-tick';
import { useBikriDraft } from '../draft-store';
import { tickLotsOf } from '../draft';
import { LOT } from '../types';

interface SellerTickProps {
  seller: PickedRecord;
  rows: readonly EntryRow[];
}

function SellerTick({ seller, rows }: SellerTickProps) {
  const stores = useBikriDraft();
  const grouping = useGrouping();
  const language = useLanguage();
  const card = usePartyCard(seller.id);
  const advance = card.data?.advance_open ?? null;
  // The tick shows only for a seller who has an advance open (PRD 12.3): the test is on the text the backend's
  // number is written as, so nothing is compared as a number here.
  if (advance === null || formatPaisa(advance, { grouping }) === '0') return null;
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
      <Localized
        id="bikri-more-advance-open"
        vars={{ amount: formatPaisa(advance, { grouping }) }}
        elems={{ num: <Num /> }}
      >
        <span className="type-body text-ink-muted" />
      </Localized>
    </div>
  );
}

interface AdvanceTicksProps {
  /** The lot rows whose sellers are listed: the selected lot, or every lot of the بکری. */
  shown: readonly EntryRow[];
}

export function AdvanceTicks({ shown }: AdvanceTicksProps) {
  const stores = useBikriDraft();
  const rows = stores.lots((state) => state.rows);
  // Each seller once, in the order of the lots; a lot with no tick to take (own goods, a garden, carried) adds none.
  const sellers: PickedRecord[] = [];
  for (const row of shown) {
    const seller = row.picked?.[LOT.seller];
    if (seller === undefined || sellers.some((known) => known.id === seller.id)) continue;
    if (tickLotsOf(rows, seller.id).length > 0) sellers.push(seller);
  }
  if (sellers.length === 0) return null;
  return (
    <div className="flex flex-col gap-2">
      {sellers.map((seller) => (
        <SellerTick key={seller.id} seller={seller} rows={rows} />
      ))}
    </div>
  );
}
