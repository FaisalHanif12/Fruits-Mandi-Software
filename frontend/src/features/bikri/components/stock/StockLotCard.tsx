// The lot that has focus in the list of goods left, with every figure of PRD FR-12.14 and what can
// be done with it: sell what is left (a later sale), write off, undo a write-off. The list shows
// the columns that fit the window; this card holds the rest, so nothing needs a sideways scroll.
// All figures are the backend's (CLAUDE.md rule 1).

import { Localized } from '@fluent/react';
import { Ban, RotateCcw, ShoppingCart } from 'lucide-react';
import type { StockRow } from '../../../../bindings/StockRow';
import { isoToDisplayDate } from '../../../../shared/format';
import { displayName, useLanguage } from '../../../../shared/i18n';
import { Button, Card, Code, Fact, Num } from '../../../../shared/ui';
import { stockSellerName } from '../../stock-columns';

interface StockLotCardProps {
  row: StockRow;
  onSell: () => void;
  onWriteOff: () => void;
  onUndoWriteOff: () => void;
}

/** Every figure of a lot (PRD FR-12.14), shared by the stock list and the later sale. */
export function LotFacts({ row }: { row: StockRow }) {
  const language = useLanguage();
  return (
    <dl className="grid grid-cols-3 gap-x-4 gap-y-3 normal:grid-cols-5">
      <Fact label={<Localized id="glossary-beopari" />}>{stockSellerName(row, language)}</Fact>
      <Fact label={<Localized id="glossary-qism-maal" />}>{displayName(row.item, language)}</Fact>
      <Fact label={<Localized id="glossary-packing" />}>{displayName(row.packing, language)}</Fact>
      <Fact label={<Localized id="bikri-stock-col-received-date" />}>
        <Code>{isoToDisplayDate(row.received_date)}</Code>
      </Fact>
      <Fact label={<Localized id="bikri-stock-col-days" />}>
        <Num>{row.days_waiting}</Num>
      </Fact>
      <Fact label={<Localized id="bikri-col-qty-received" />}>
        <Num>{row.qty_received}</Num>
      </Fact>
      <Fact label={<Localized id="bikri-saved-qty-sold" />}>
        <Num>{row.qty_sold}</Num>
      </Fact>
      <Fact label={<Localized id="bikri-stock-col-written-off" />}>
        <Num>{row.qty_written_off}</Num>
      </Fact>
      <Fact label={<Localized id="bikri-col-balance" />}>
        <Num>{row.balance}</Num>
      </Fact>
    </dl>
  );
}

export function StockLotCard({ row, onSell, onWriteOff, onUndoWriteOff }: StockLotCardProps) {
  return (
    <Card
      compact
      title={
        <Localized
          id="bikri-stock-lot-title"
          vars={{ number: String(row.bikri_number), lot: String(row.lot_no) }}
        >
          <span />
        </Localized>
      }
    >
      <LotFacts row={row} />
      <div className="flex flex-wrap items-center gap-2">
        <Button variant="primary" disabled={row.balance <= 0} onClick={onSell}>
          <ShoppingCart aria-hidden="true" className="size-icon-md" />
          <Localized id="bikri-stock-sell" />
        </Button>
        <Button disabled={row.balance <= 0} onClick={onWriteOff}>
          <Ban aria-hidden="true" className="size-icon-md" />
          <Localized id="bikri-stock-write-off" />
        </Button>
        <Button disabled={row.write_offs.length === 0} onClick={onUndoWriteOff}>
          <RotateCcw aria-hidden="true" className="size-icon-md" />
          <Localized id="bikri-stock-undo-write-off" />
        </Button>
      </div>
    </Card>
  );
}
