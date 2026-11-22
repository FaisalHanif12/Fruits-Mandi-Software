// The purchases of one trip with their stock (PRD FR-12.25, FR-12.27, FR-12.28): a list with the
// columns that fit the window, and under it the card of the purchase that has focus, with its
// figures, each partner's part, its result, and the actions: write off spoiled goods, undo a
// write-off, cancel the purchase. ↑ ↓ and Page Up and Page Down move between rows; Enter moves to
// the actions of the card.
//
// The list loads page by page from the backend, with the totals over all the trip's purchases in
// its last row (PRD FR-19.11, plan 90). Four states: loading, an error with a retry, no purchase yet
// (a helpful message and a button to start one), and the rows; the card has its own.

import { Localized } from '@fluent/react';
import { ShoppingBasket } from 'lucide-react';
import { useCallback, useMemo, useRef, useState } from 'react';
import { uiErrorOf } from '../../../shared/api';
import { ListGrid, Pager, useListPaging, type ListContext } from '../../../shared/grid';
import { SHORTCUTS } from '../../../shared/keyboard';
import { Card, EmptyState, ErrorState, LoadingState } from '../../../shared/ui';
import {
  CancelPurchaseDialog,
  OwnWriteOffDialog,
  PurchaseDetail,
  TRIP_PURCHASE_COLUMNS,
  UndoOwnWriteOffDialog,
  usePurchase,
  useTripPurchases,
  type OwnStockTarget,
  type PurchaseRow,
} from '../../purchases';

/** The write-off boxes take this of a purchase (the list of باقی مال gives the same). */
function targetOf(row: PurchaseRow): OwnStockTarget {
  return {
    purchaseId: row.id,
    number: row.number,
    item: row.item,
    packing: row.packing,
    qtyLeft: row.qty_left,
    writeOffs: row.write_offs,
  };
}

interface TripPurchasesSectionProps {
  /** The trip whose purchases are listed. */
  tripId: number;
  /** The purchase that has focus in the list, or null. */
  selectedId: number | null;
  onSelect: (id: number | null) => void;
}

export function TripPurchasesSection({ tripId, selectedId, onSelect }: TripPurchasesSectionProps) {
  const [writeOff, setWriteOff] = useState<OwnStockTarget | null>(null);
  const [undoWriteOff, setUndoWriteOff] = useState<OwnStockTarget | null>(null);
  const [cancelRow, setCancelRow] = useState<PurchaseRow | null>(null);
  const detailRef = useRef<HTMLDivElement>(null);
  const selected = usePurchase(selectedId);
  const list = useTripPurchases(tripId);
  const paging = useListPaging(list.pager);
  const first = list.data?.pages[0];
  const purchases = useMemo(() => list.data?.pages.flatMap((page) => page.rows) ?? [], [list.data]);
  // The total row is the backend's, over all the purchases of the trip. The callback depends on the
  // loaded list, not on an amount (Code Standards F3).
  const totals = useCallback(
    (context: ListContext) => ({
      labelField: 'number',
      values: {
        number: context.text('glossary-meezan'),
        qty_left: String(first?.totals.qty_left ?? 0),
        ...(first === undefined
          ? {}
          : {
              cost_left: context.money(first.totals.cost_left_paisa),
              result: context.money(first.totals.result_paisa),
              commission: context.money(first.totals.commission_paisa),
              total_cost: context.money(first.totals.total_cost_paisa),
            }),
      },
    }),
    [first],
  );

  return (
    <Card compact icon={ShoppingBasket} title={<Localized id="trips-purchases-title" />}>
      {list.isPending ? (
        <LoadingState rows={3} />
      ) : list.isError ? (
        <ErrorState
          title={<Localized id="trips-purchases-failed" />}
          error={uiErrorOf(list.error)}
          onRetry={() => {
            void list.refetch();
          }}
        />
      ) : purchases.length === 0 ? (
        <EmptyState
          icon={ShoppingBasket}
          title={<Localized id="trips-purchases-empty" />}
          help={<Localized id="trips-purchases-empty-help" vars={{ key: SHORTCUTS.save.label }} />}
        />
      ) : (
        <>
          <Localized id="trips-purchases-select-help">
            <p className="type-body text-ink-muted" />
          </Localized>
          <ListGrid<PurchaseRow>
            columns={TRIP_PURCHASE_COLUMNS}
            rows={purchases}
            getRowId={(row) => String(row.id)}
            labelId="trips-purchases-label"
            onOpen={(row) => {
              onSelect(row.id);
              // Enter on a row moves the cursor to the actions of its card.
              window.requestAnimationFrame(() => {
                detailRef.current?.querySelector<HTMLElement>('button')?.focus();
              });
            }}
            onFocusRow={(rowId) => {
              const id = Number(rowId);
              onSelect(Number.isSafeInteger(id) ? id : null);
            }}
            focusRowId={selectedId === null ? null : String(selectedId)}
            heightMode="auto"
            isInactive={(row) => row.status === 'cancelled'}
            totals={totals}
            paging={paging}
          />
          <Pager pager={list.pager} />
        </>
      )}
      {selectedId === null ? null : (
        <div ref={detailRef}>
          <PurchaseDetail
            query={selected}
            onWriteOff={(row) => {
              setWriteOff(targetOf(row));
            }}
            onUndoWriteOff={(row) => {
              setUndoWriteOff(targetOf(row));
            }}
            onCancel={setCancelRow}
          />
        </div>
      )}
      <OwnWriteOffDialog
        target={writeOff}
        onClose={() => {
          setWriteOff(null);
        }}
      />
      <UndoOwnWriteOffDialog
        target={undoWriteOff}
        onClose={() => {
          setUndoWriteOff(null);
        }}
      />
      <CancelPurchaseDialog
        row={cancelRow}
        onClose={() => {
          setCancelRow(null);
        }}
        onCancelled={() => {
          setCancelRow(null);
        }}
      />
    </Card>
  );
}
