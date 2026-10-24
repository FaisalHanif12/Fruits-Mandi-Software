// باقی مال (route /bikri/stock; PRD 12.8, FR-12.14 to FR-12.16): the lots that still have goods
// left. Filters: the day received (from and to), the seller and the item (the party filter of
// FR-19.10). The list shows the columns that fit the window; the card of the lot under it shows
// every figure and holds the three actions: sell what is left (a later sale, FR-12.15), write off
// with a reason, and undo a write-off (FR-12.16). Enter on a row starts the later sale.
//
// - Four states: loading, an error with a retry, nothing left (with a button), and the rows.
// - The tick "also lots that are written off" lists a lot with nothing left that has a write-off
//   to undo, so the undo can always be found (contract 5).
// - The firm's own goods that are not sold yet are listed under the lots, apart and marked with the
//   number of their purchase (plan 16b, PRD FR-12.25). Their write-off and its undo are there too.
// - Every figure is the backend's. A save elsewhere reads the list again (PRD 6.2).

import { Localized } from '@fluent/react';
import { useNavigate } from '@tanstack/react-router';
import { PackageCheck, Plus, SearchX } from 'lucide-react';
import { useCallback, useEffect, useMemo, useState } from 'react';
import type { StockListInput } from '../../../bindings/StockListInput';
import type { StockRow } from '../../../bindings/StockRow';
import { uiErrorOf } from '../../../shared/api';
import { ListGrid, type ListContext } from '../../../shared/grid';
import { useFocusFirstField } from '../../../shared/hooks';
import { SHORTCUTS, formNavigationProps, useEscape } from '../../../shared/keyboard';
import { Remembered, type MemoryProps } from '../../../shared/screen-state';
import {
  AccountPicker,
  Button,
  Checkbox,
  DateInput,
  EmptyState,
  ErrorState,
  Field,
  LoadingState,
  Picker,
  type PickerValue,
} from '../../../shared/ui';
import { OwnStockSection } from '../components/stock/OwnStockSection';
import { StockLotCard } from '../components/stock/StockLotCard';
import { UndoWriteOffDialog, WriteOffDialog } from '../components/stock/WriteOffDialogs';
import { useStockList } from '../hooks/useBikriReads';
import { STOCK_SCREEN, stockChoicesFrom, stockChoicesText } from '../remember';
import { STOCK_COLUMNS } from '../stock-columns';

/** A list with more rows than this fills the room and scrolls inside; shorter ones are as tall as their rows. */
const SHORT_LIST_ROWS = 30;

/** Text or nothing: an empty filter is sent as no limit. */
function textOrNull(text: string): string | null {
  const trimmed = text.trim();
  return trimmed === '' ? null : trimmed;
}

/** باقی مال starts as the Owner left the filters (PRD FR-7.45). */
export function StockPage() {
  return <Remembered screen={STOCK_SCREEN}>{(memory) => <StockBody {...memory} />}</Remembered>;
}

function StockBody({ saved, remember }: MemoryProps) {
  const navigate = useNavigate();
  const start = useMemo(() => stockChoicesFrom(saved), [saved]);
  const [from, setFrom] = useState(start.from);
  const [to, setTo] = useState(start.to);
  const [seller, setSeller] = useState<PickerValue | null>(start.seller);
  const [item, setItem] = useState<PickerValue | null>(start.item);
  const [includeWrittenOff, setIncludeWrittenOff] = useState(start.includeWrittenOff);
  useEffect(() => {
    remember(stockChoicesText({ from, to, seller, item, includeWrittenOff }));
  }, [from, to, seller, item, includeWrittenOff, remember]);
  const filter: StockListInput = {
    date: null,
    from: textOrNull(from),
    to: textOrNull(to),
    seller_id: seller?.id ?? null,
    item_id: item?.id ?? null,
    include_written_off: includeWrittenOff,
  };
  const list = useStockList(filter);
  const rows = list.data?.rows;
  const ownRows = list.data?.own_rows ?? [];
  const containerRef = useFocusFirstField<HTMLDivElement>();
  const noFilter = filter.from === null && filter.to === null && seller === null && item === null;

  // The lot that has focus in the list. If it leaves the list (sold out), the first row is used.
  const [selectedId, setSelectedId] = useState<number | null>(null);
  const selected = rows?.find((row) => row.lot_id === selectedId) ?? rows?.[0] ?? null;
  const [writeOffRow, setWriteOffRow] = useState<StockRow | null>(null);
  const [undoRow, setUndoRow] = useState<StockRow | null>(null);

  const sell = (row: StockRow) => {
    if (row.balance <= 0) return;
    void navigate({ to: '/bikri/later', search: { bikri: row.bikri_id, lot: row.lot_id } });
  };
  useEscape('leave', () => {
    void navigate({ to: '/' });
  });

  // The total under the balance column is the backend's figure (Code Standards F19). It is a
  // count of نگ, not an amount, so it is passed as text.
  const total = list.data?.balance_total;
  const totals = useCallback(
    (context: ListContext) => ({
      labelField: 'seller',
      values: {
        seller: context.text('glossary-meezan'),
        ...(total === undefined ? {} : { balance: String(total) }),
      },
    }),
    [total],
  );
  const columns = useMemo(() => STOCK_COLUMNS, []);

  let body;
  if (rows === undefined && list.isPending) {
    body = <LoadingState />;
  } else if (list.isError && rows === undefined) {
    body = (
      <ErrorState
        title={<Localized id="bikri-stock-failed" />}
        error={uiErrorOf(list.error)}
        onRetry={() => {
          void list.refetch();
        }}
      />
    );
  } else if (rows?.length === 0 && ownRows.length === 0) {
    body = noFilter ? (
      <EmptyState
        icon={PackageCheck}
        title={<Localized id="bikri-stock-empty" />}
        help={<Localized id="bikri-stock-empty-help" vars={{ key: SHORTCUTS.newBikri.label }} />}
        action={
          <Button
            variant="primary"
            shortcut={SHORTCUTS.newBikri}
            onClick={() => {
              void navigate({ to: '/bikri/book', search: {} });
            }}
          >
            <Plus aria-hidden="true" className="size-icon-md" />
            <Localized id="bikri-new" />
          </Button>
        }
      />
    ) : (
      <EmptyState
        icon={SearchX}
        title={<Localized id="bikri-stock-no-match" />}
        help={<Localized id="bikri-stock-no-match-help" />}
        action={
          <Button
            onClick={() => {
              setFrom('');
              setTo('');
              setSeller(null);
              setItem(null);
            }}
          >
            <Localized id="bikri-list-clear" />
          </Button>
        }
      />
    );
  } else if (rows !== undefined) {
    body = (
      <>
        {rows.length === 0 ? null : (
          <>
            <ListGrid<StockRow>
              columns={columns}
              rows={rows}
              getRowId={(row) => String(row.lot_id)}
              labelId="bikri-stock-label"
              onOpen={sell}
              onFocusRow={(rowId) => {
                setSelectedId(Number(rowId));
              }}
              focusRowId={selectedId === null ? null : String(selectedId)}
              heightMode={rows.length <= SHORT_LIST_ROWS ? 'auto' : 'fill'}
              isInactive={(row) => row.balance <= 0}
              totals={totals}
            />
            {selected === null ? null : (
              <StockLotCard
                row={selected}
                onSell={() => {
                  sell(selected);
                }}
                onWriteOff={() => {
                  setWriteOffRow(selected);
                }}
                onUndoWriteOff={() => {
                  setUndoRow(selected);
                }}
              />
            )}
          </>
        )}
        {ownRows.length === 0 || list.data === undefined ? null : (
          <OwnStockSection rows={ownRows} list={list.data} />
        )}
      </>
    );
  }

  return (
    <div className="flex h-full flex-col gap-4">
      <div className="flex flex-col gap-1">
        <Localized id="layout-module-stock">
          <h1 className="type-title" />
        </Localized>
        <Localized id="bikri-stock-help">
          <p className="type-body text-ink-muted" />
        </Localized>
      </div>
      <div
        ref={containerRef}
        {...formNavigationProps}
        className="grid grid-cols-2 items-end gap-4 normal:grid-cols-4"
      >
        <Field label={<Localized id="bikri-stock-from" />}>
          {(control) => <DateInput {...control} value={from} onChange={setFrom} />}
        </Field>
        <Field label={<Localized id="bikri-stock-to" />}>
          {(control) => <DateInput {...control} value={to} onChange={setTo} />}
        </Field>
        <Field label={<Localized id="glossary-beopari" />}>
          {(control) => (
            <AccountPicker
              {...control}
              scope="seller"
              showCard={false}
              value={seller}
              onChange={setSeller}
            />
          )}
        </Field>
        <Field label={<Localized id="glossary-qism-maal" />}>
          {(control) => (
            <Picker
              {...control}
              scope="item"
              value={item}
              onChange={(hit) => {
                setItem(hit);
              }}
            />
          )}
        </Field>
      </div>
      <Checkbox
        checked={includeWrittenOff}
        onChange={setIncludeWrittenOff}
        label={<Localized id="bikri-stock-include-written-off" />}
      />
      {body}
      <WriteOffDialog
        row={writeOffRow}
        onClose={() => {
          setWriteOffRow(null);
        }}
      />
      <UndoWriteOffDialog
        row={undoRow}
        onClose={() => {
          setUndoRow(null);
        }}
      />
    </div>
  );
}
