// "Market trip goods" and "Garden goods": the card of goods to load into the بکری by hand (PRD FR-12.50, plan 78
// C1). A dialog over the screen with every row of one kind that has goods left for our shop, oldest day to sell
// first (the backend's `bikri_goods_to_load`), minus what is on the screen already. Enter, or a double click, on a
// row puts it on the screen as a lot and closes the card; Esc closes it and leaves everything as it was. Nothing is
// worked out here: the goods left are the backend's.

import { Localized } from '@fluent/react';
import { PackageX } from 'lucide-react';
import { useEffect, useMemo, useRef } from 'react';
import type { VentureKind } from '../../../bindings/VentureKind';
import { uiErrorOf } from '../../../shared/api';
import { ListGrid, ListRegion, listHeightMode, type ListColumn } from '../../../shared/grid';
import { displayName } from '../../../shared/i18n';
import { SHORTCUTS } from '../../../shared/keyboard';
import { Button, EmptyState, ErrorState, FormDialog, LoadingState } from '../../../shared/ui';
import { isoToDisplayDate } from '../../../shared/format';
import { arrivalNumberText } from '../../gardens';
import { purchaseNumberText } from '../../purchases';
import { useGoodsToLoad } from '../hooks/useGoodsSources';
import type { ReadyStock } from '../ready-stock';

/** How many frames the first row is waited for: the grid draws its rows a moment after it opens. */
const FOCUS_TRIES = 30;

const NUMBER_ID: Record<VentureKind, string> = {
  trip: 'bikri-goods-col-number-trip',
  garden: 'bikri-goods-col-number-garden',
};
const WHERE_ID: Record<VentureKind, string> = {
  trip: 'bikri-goods-col-trip',
  garden: 'bikri-goods-col-garden',
};

/** The columns of the card: where the goods are from, what they are and what is left; the day to sell is information. */
function columnsOf(kind: VentureKind): readonly ListColumn<ReadyStock>[] {
  return [
    {
      field: 'number',
      type: 'code',
      headerId: NUMBER_ID[kind],
      value: (row, { text }) =>
        kind === 'trip'
          ? purchaseNumberText(row.number, text)
          : arrivalNumberText(row.number, text),
    },
    {
      field: 'where',
      type: 'text',
      headerId: WHERE_ID[kind],
      value: (row, { language }) =>
        displayName(row.trip ?? row.garden ?? { name_ur: '', name_en: '' }, language),
    },
    {
      field: 'item',
      type: 'text',
      headerId: 'bikri-stock-col-item',
      grow: true,
      value: (row, { language, text }) =>
        text('bikri-stock-item-packing', {
          item: displayName(row.item, language),
          packing: displayName(row.packing, language),
        }),
    },
    {
      field: 'qty_left',
      type: 'number',
      headerId: 'bikri-goods-col-left',
      value: (row) => String(row.qtyLeft),
    },
    {
      field: 'sell_date',
      type: 'date',
      headerId: 'bikri-goods-col-sell-date',
      from: 'normal',
      value: (row) => isoToDisplayDate(row.sellDate),
    },
  ];
}

interface GoodsLoadPickerProps {
  kind: VentureKind;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  /** The ids of the purchases or arrivals on the screen already: they are not offered again. */
  taken: ReadonlySet<number>;
  /** The goods to put on the screen as a lot. */
  onPick: (stock: ReadyStock) => void;
}

export function GoodsLoadPicker({ kind, open, onOpenChange, taken, onPick }: GoodsLoadPickerProps) {
  const { query, rows: found } = useGoodsToLoad(kind, open);
  const columns = useMemo(() => columnsOf(kind), [kind]);
  const rows = useMemo(
    () => (found ?? []).filter((row) => row.kind === kind && !taken.has(row.id)),
    [found, kind, taken],
  );

  // The cursor goes to the first row when the rows are drawn, so a keyboard user can go down with the arrows and
  // take a row with Enter at once, with no Tab (the Back button holds the focus until then).
  const listRef = useRef<HTMLDivElement>(null);
  const hasRows = rows.length > 0;
  useEffect(() => {
    if (!hasRows) return;
    let frame = 0;
    let tries = 0;
    const focusFirst = () => {
      const cell = listRef.current?.querySelector<HTMLElement>('.ag-row .ag-cell');
      if (cell !== undefined && cell !== null) {
        cell.focus();
      } else if (tries < FOCUS_TRIES) {
        tries += 1;
        frame = requestAnimationFrame(focusFirst);
      }
    };
    frame = requestAnimationFrame(focusFirst);
    return () => {
      cancelAnimationFrame(frame);
    };
  }, [hasRows]);

  let body;
  if (query.isError) {
    body = (
      <ErrorState
        title={<Localized id="bikri-goods-failed" />}
        error={uiErrorOf(query.error)}
        onRetry={() => {
          void query.refetch();
        }}
      />
    );
  } else if (found === undefined) {
    body = <LoadingState rows={4} />;
  } else if (rows.length === 0) {
    body = (
      <EmptyState
        icon={PackageX}
        title={<Localized id="bikri-goods-empty" />}
        help={<Localized id="bikri-goods-empty-help" />}
      />
    );
  } else {
    body = (
      <div ref={listRef}>
        <ListRegion rowCount={rows.length}>
          <ListGrid<ReadyStock>
            columns={columns}
            rows={rows}
            getRowId={(row) => `${row.kind}-${String(row.id)}`}
            labelId={kind === 'trip' ? 'bikri-goods-trip-label' : 'bikri-goods-garden-label'}
            onOpen={(row) => {
              onPick(row);
              onOpenChange(false);
            }}
            heightMode={listHeightMode(rows.length)}
          />
        </ListRegion>
      </div>
    );
  }

  return (
    <FormDialog
      open={open}
      onOpenChange={onOpenChange}
      title={
        <Localized id={kind === 'trip' ? 'bikri-goods-trip-title' : 'bikri-goods-garden-title'} />
      }
      description={
        <Localized id={kind === 'trip' ? 'bikri-goods-trip-help' : 'bikri-goods-garden-help'} />
      }
    >
      <div className="flex w-full flex-col gap-3">
        {body}
        {/* A way out with the mouse too (NFR-USE-8): Esc and a click on the scrim close it as well. */}
        <div className="flex justify-end">
          <Button
            shortcut={SHORTCUTS.back}
            onClick={() => {
              onOpenChange(false);
            }}
          >
            <Localized id="common-back" />
          </Button>
        </div>
      </div>
    </FormDialog>
  );
}
