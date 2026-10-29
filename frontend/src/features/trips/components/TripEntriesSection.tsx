// The history of one trip (PRD FR-7.67, FR-19.11): its purchases, trip costs and sales in other
// shops, newest first, loaded page by page with a button for the next page. The row that has
// focus is shown in full under the list: each partner's part of a payment and where it came from
// (C23), how a cost was spread (C24), or each partner's part of the money of a sale (C25). The
// row can be cancelled with a reason: a purchase with its own box, a cost or a sale in another
// shop with the box of this module.
//
// Four states: loading, an error with a retry, nothing yet (a helpful message), and the rows. A
// cancelled entry stays in the list, marked in words.

import { Localized, useLocalization } from '@fluent/react';
import { Ban, History } from 'lucide-react';
import { useState } from 'react';
import { ListGrid, Pager, useListPaging } from '../../../shared/grid';
import type { PagerState } from '../../../shared/hooks';
import { SHORTCUTS } from '../../../shared/keyboard';
import {
  Button,
  Card,
  EmptyState,
  EntryPartsList,
  Fact,
  Num,
  ShareAmounts,
} from '../../../shared/ui';
import { CancelPurchaseDialog, type PurchaseRow } from '../../purchases';
import { ENTRY_COLUMNS } from '../list-columns';
import type { TripEntryRow } from '../types';
import { CancelEntryDialog } from './CancelEntryDialog';

interface TripEntriesSectionProps {
  entries: readonly TripEntryRow[];
  /** How many entries the trip has in all. */
  count: number;
  /** Where the entries are: the page on screen and the page buttons. */
  pager: PagerState;
  /** The trip's purchases, to cancel a purchase from its row in the history. */
  purchases: readonly PurchaseRow[];
}

export function TripEntriesSection({ entries, count, pager, purchases }: TripEntriesSectionProps) {
  const { l10n } = useLocalization();
  const paging = useListPaging(pager);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [cancelEntry, setCancelEntry] = useState<TripEntryRow | null>(null);
  const [cancelPurchase, setCancelPurchase] = useState<PurchaseRow | null>(null);
  // A purchase and a cost can have the same id: the key says which kind it is.
  const keyOf = (row: TripEntryRow) => `${row.kind}-${String(row.id)}`;
  const selected = entries.find((row) => keyOf(row) === selectedId) ?? null;
  const purchaseOfSelected =
    selected?.kind === 'purchase' && selected.purchase_id !== null
      ? (purchases.find((purchase) => purchase.id === selected.purchase_id) ?? null)
      : null;
  const canCancel =
    selected !== null &&
    !selected.cancelled &&
    selected.can_cancel &&
    (selected.kind !== 'purchase' || purchaseOfSelected !== null);

  return (
    <Card compact title={<Localized id="trips-entries-title" />}>
      {entries.length === 0 ? (
        <EmptyState
          icon={History}
          title={<Localized id="trips-entries-empty" />}
          help={<Localized id="trips-entries-empty-help" vars={{ key: SHORTCUTS.save.label }} />}
        />
      ) : (
        <>
          <div className="flex flex-wrap items-center justify-between gap-2">
            <Localized
              id="trips-entries-count"
              vars={{ shown: String(entries.length), total: String(count) }}
            >
              <p className="type-body text-ink-muted" />
            </Localized>
            <Button
              variant="danger"
              disabled={!canCancel}
              onClick={() => {
                if (selected === null) return;
                if (selected.kind === 'purchase') setCancelPurchase(purchaseOfSelected);
                else setCancelEntry(selected);
              }}
            >
              <Ban aria-hidden="true" className="size-icon-md" />
              <Localized id="trips-entries-cancel" />
            </Button>
          </div>
          <ListGrid<TripEntryRow>
            columns={ENTRY_COLUMNS}
            rows={entries}
            getRowId={keyOf}
            labelId="trips-entries-label"
            onOpen={(row) => {
              setSelectedId(keyOf(row));
            }}
            onFocusRow={setSelectedId}
            focusRowId={selectedId}
            heightMode="auto"
            isInactive={(row) => row.cancelled}
            paging={paging}
          />
          <Pager pager={pager} />
        </>
      )}
      {selected === null ? null : (
        <div className="flex flex-col gap-2 rounded-2 border border-border p-3">
          {selected.cancelled ? (
            <Localized
              id="trips-entry-cancelled-reason"
              vars={{ reason: selected.cancel_reason ?? '' }}
            >
              <p className="type-label text-due" role="status" />
            </Localized>
          ) : null}
          {selected.parts.length === 0 ? null : (
            <>
              <Localized id="trips-parts-title">
                <h3 className="type-label text-ink-muted" />
              </Localized>
              <EntryPartsList parts={selected.parts} label={l10n.getString('trips-parts-title')} />
            </>
          )}
          {selected.credits.length === 0 ? null : (
            <>
              <Localized id="trips-sale-credits-title">
                <h3 className="type-label text-ink-muted" />
              </Localized>
              <ShareAmounts
                credits={selected.credits}
                label={l10n.getString('trips-sale-credits-title')}
              />
            </>
          )}
          {selected.spread.length === 0 ? null : (
            <>
              <Localized id="trips-cost-spread-title">
                <h3 className="type-label text-ink-muted" />
              </Localized>
              <dl className="flex flex-col">
                {selected.spread.map((part) => (
                  <Fact
                    key={part.purchase_id}
                    label={
                      <Localized
                        id="trips-entry-spread-line"
                        vars={{ number: String(part.purchase_number), qty: String(part.qty) }}
                      >
                        <span />
                      </Localized>
                    }
                  >
                    <Num paisa={part.amount_paisa} />
                  </Fact>
                ))}
              </dl>
            </>
          )}
        </div>
      )}
      <CancelEntryDialog
        row={cancelEntry}
        onClose={() => {
          setCancelEntry(null);
        }}
      />
      <CancelPurchaseDialog
        row={cancelPurchase}
        onClose={() => {
          setCancelPurchase(null);
        }}
        onCancelled={() => {
          setCancelPurchase(null);
        }}
      />
    </Card>
  );
}
