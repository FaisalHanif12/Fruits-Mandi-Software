// The purchases of one trip with their stock (PRD FR-12.25, FR-12.27, FR-12.28): a list with the
// columns that fit the window, and under it the card of the purchase that has focus, with its
// figures, each partner's part, its result, and the actions: write off spoiled goods, undo a
// write-off, cancel the purchase. ↑ ↓ and Page Up and Page Down move between rows; Enter moves to
// the actions of the card.
//
// Four states: no purchase yet (a helpful message and a button to start one), the list, and, for
// the card, loading and an error with a retry (the card draws them). The list is the trip's own
// purchases, which the page already loaded, so it has no loading state of its own.

import { Localized } from '@fluent/react';
import { ShoppingBasket } from 'lucide-react';
import { useRef, useState } from 'react';
import { ListGrid } from '../../../shared/grid';
import { SHORTCUTS } from '../../../shared/keyboard';
import { Card, EmptyState } from '../../../shared/ui';
import {
  CancelPurchaseDialog,
  OwnWriteOffDialog,
  PurchaseDetail,
  TRIP_PURCHASE_COLUMNS,
  UndoOwnWriteOffDialog,
  usePurchase,
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
  purchases: readonly PurchaseRow[];
  /** The purchase that has focus in the list, or null. */
  selectedId: number | null;
  onSelect: (id: number | null) => void;
}

export function TripPurchasesSection({
  purchases,
  selectedId,
  onSelect,
}: TripPurchasesSectionProps) {
  const [writeOff, setWriteOff] = useState<OwnStockTarget | null>(null);
  const [undoWriteOff, setUndoWriteOff] = useState<OwnStockTarget | null>(null);
  const [cancelRow, setCancelRow] = useState<PurchaseRow | null>(null);
  const detailRef = useRef<HTMLDivElement>(null);
  const selected = usePurchase(selectedId);

  return (
    <Card compact title={<Localized id="trips-purchases-title" />}>
      {purchases.length === 0 ? (
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
          />
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
