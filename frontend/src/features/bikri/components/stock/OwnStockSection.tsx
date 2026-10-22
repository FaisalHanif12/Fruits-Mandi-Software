// The trips' goods that are not sold yet, under the lots in باقی مال (PRD FR-12.25, FR-12.28).
// They are listed apart because they are not a seller's goods: no seller is owed for them, they
// belong to the partners of a trip, and a write-off posts nothing (the cost only shows as a loss
// in the trip's result). Each row is marked with the purchase number in its own series (خ-5,
// PUR-5), its trip, and the heading "Own goods". The card of the row that has focus holds every
// figure and the actions: write off, undo a write-off, open the trip. Selling them is done in a
// بکری: choose اپنا مال as the seller.
//
// The list and the total are the backend's (`bikri_stock_list`); nothing is worked out here.

import { Localized, useLocalization } from '@fluent/react';
import { useNavigate } from '@tanstack/react-router';
import { Ban, RotateCcw, ShoppingBasket } from 'lucide-react';
import { useCallback, useMemo, useState } from 'react';
import type { OwnStockRow } from '../../../../bindings/OwnStockRow';
import type { StockList } from '../../../../bindings/StockList';
import { isoToDisplayDate } from '../../../../shared/format';
import { ListGrid, type ListContext } from '../../../../shared/grid';
import { displayName, useLanguage } from '../../../../shared/i18n';
import { Button, Card, Code, Fact, Num } from '../../../../shared/ui';
import { tripNumberText } from '../../../trips';
import {
  OwnWriteOffDialog,
  UndoOwnWriteOffDialog,
  purchaseNumberText,
  supplierText,
  type OwnStockTarget,
} from '../../../purchases';
import { OWN_STOCK_COLUMNS } from '../../own-stock-columns';

/** A list with more rows than this fills the room and scrolls inside; shorter ones are as tall as their rows. */
const SHORT_LIST_ROWS = 30;

function targetOf(row: OwnStockRow): OwnStockTarget {
  return {
    purchaseId: row.purchase_id,
    number: row.number,
    item: row.item,
    packing: row.packing,
    qtyLeft: row.balance,
    writeOffs: row.write_offs,
  };
}

interface OwnStockSectionProps {
  rows: readonly OwnStockRow[];
  /** The backend's totals under the list. */
  list: Pick<StockList, 'own_balance_total' | 'own_cost_left_paisa'>;
}

export function OwnStockSection({ rows, list }: OwnStockSectionProps) {
  const { l10n } = useLocalization();
  const language = useLanguage();
  const navigate = useNavigate();
  const [selectedId, setSelectedId] = useState<number | null>(null);
  const [writeOff, setWriteOff] = useState<OwnStockTarget | null>(null);
  const [undo, setUndo] = useState<OwnStockTarget | null>(null);
  // The row that has focus; if it left the list (sold out), the first row is used.
  const selected = rows.find((row) => row.purchase_id === selectedId) ?? rows[0] ?? null;
  const columns = useMemo(() => OWN_STOCK_COLUMNS, []);
  const say = (id: string, vars?: Record<string, string>) => l10n.getString(id, vars);

  // The total row is the backend's: a count of نگ (as text) and the cost left (money). The callback
  // depends on the loaded list, not on an amount: a Paisa value must not go into a dependency list
  // (Code Standards F3).
  const totals = useCallback(
    (context: ListContext) => ({
      labelField: 'trip',
      values: {
        trip: context.text('glossary-meezan'),
        balance: String(list.own_balance_total),
        cost_left: context.money(list.own_cost_left_paisa),
      },
    }),
    [list],
  );

  return (
    <section aria-labelledby="bikri-stock-own-heading" className="flex flex-col gap-3">
      <div className="flex flex-col gap-1">
        <Localized id="bikri-stock-own-title">
          <h2 id="bikri-stock-own-heading" className="type-section text-ink" />
        </Localized>
        <Localized id="bikri-stock-own-help">
          <p className="type-body text-ink-muted" />
        </Localized>
      </div>
      <ListGrid<OwnStockRow>
        columns={columns}
        rows={rows}
        getRowId={(row) => String(row.purchase_id)}
        labelId="bikri-stock-own-label"
        onOpen={(row) => {
          void navigate({
            to: '/trips/$id',
            params: { id: String(row.trip.id) },
            search: { purchase: row.purchase_id },
          });
        }}
        onFocusRow={(rowId) => {
          setSelectedId(Number(rowId));
        }}
        focusRowId={selectedId === null ? null : String(selectedId)}
        heightMode={rows.length <= SHORT_LIST_ROWS ? 'auto' : 'fill'}
        totals={totals}
      />
      {selected === null ? null : (
        <Card
          compact
          title={
            <Localized
              id="bikri-stock-own-card-title"
              vars={{
                number: purchaseNumberText(selected.number, say),
                item: displayName(selected.item, language),
              }}
            >
              <span />
            </Localized>
          }
        >
          <dl className="grid grid-cols-3 gap-x-4 gap-y-3 normal:grid-cols-5">
            <Fact label={<Localized id="bikri-stock-own-trip" />}>
              {tripNumberText(selected.trip.number, say)}
            </Fact>
            <Fact label={<Localized id="purchases-field-supplier" />}>
              {supplierText(selected.supplier, say)}
            </Fact>
            <Fact label={<Localized id="glossary-qism-maal" />}>
              {displayName(selected.item, language)}
            </Fact>
            <Fact label={<Localized id="glossary-packing" />}>
              {displayName(selected.packing, language)}
            </Fact>
            <Fact label={<Localized id="bikri-stock-col-received-date" />}>
              <Code>{isoToDisplayDate(selected.received_date)}</Code>
            </Fact>
            <Fact label={<Localized id="bikri-stock-col-days" />}>
              <Num>{selected.days_waiting}</Num>
            </Fact>
            <Fact label={<Localized id="purchases-col-qty" />}>
              <Num>{selected.qty_bought}</Num>
            </Fact>
            <Fact label={<Localized id="bikri-saved-qty-sold" />}>
              <Num>{selected.qty_sold}</Num>
            </Fact>
            <Fact label={<Localized id="bikri-stock-col-written-off" />}>
              <Num>{selected.qty_written_off}</Num>
            </Fact>
            <Fact label={<Localized id="bikri-col-balance" />}>
              <Num>{selected.balance}</Num>
            </Fact>
            <Fact label={<Localized id="bikri-stock-own-total-cost" />}>
              <Num paisa={selected.total_cost_paisa} />
            </Fact>
            <Fact label={<Localized id="purchases-col-cost-left" />}>
              <Num paisa={selected.cost_left_paisa} />
            </Fact>
          </dl>
          <div className="flex flex-wrap items-center gap-2">
            <Button
              disabled={selected.balance <= 0}
              onClick={() => {
                setWriteOff(targetOf(selected));
              }}
            >
              <Ban aria-hidden="true" className="size-icon-md" />
              <Localized id="purchases-write-off" />
            </Button>
            <Button
              disabled={selected.write_offs.length === 0}
              onClick={() => {
                setUndo(targetOf(selected));
              }}
            >
              <RotateCcw aria-hidden="true" className="size-icon-md" />
              <Localized id="purchases-undo-write-off" />
            </Button>
            <Button
              onClick={() => {
                void navigate({
                  to: '/trips/$id',
                  params: { id: String(selected.trip.id) },
                  search: { purchase: selected.purchase_id },
                });
              }}
            >
              <ShoppingBasket aria-hidden="true" className="size-icon-md" />
              <Localized id="bikri-stock-own-open" />
            </Button>
          </div>
        </Card>
      )}
      <OwnWriteOffDialog
        target={writeOff}
        onClose={() => {
          setWriteOff(null);
        }}
      />
      <UndoOwnWriteOffDialog
        target={undo}
        onClose={() => {
          setUndo(null);
        }}
      />
    </section>
  );
}
