// The goods arrived entries of one garden (مال آیا, PRD FR-12.40): the garden's stock. Each entry
// says what came (item, packing, quantity), the day to sell and what is left. A new بکری holds a
// lot for every entry whose day to sell has come and that still has goods left; goods not sold
// stay here and come again the next day. The entry that has focus is shown in full under the list:
// every figure, the lots that sold it and its write-offs, and what can be done with it: change the
// day to sell, write goods off, undo a write-off, cancel (only while nothing of it is sold).
// None of these posts any money.
//
// Four states: loading, an error with a retry, nothing yet (a helpful message), and the rows.

import { Localized, useLocalization } from '@fluent/react';
import { ArrowLeftRight, Ban, CalendarClock, RotateCcw, Sprout } from 'lucide-react';
import { useCallback, useMemo, useState } from 'react';
import type { UiError } from '../../../bindings/UiError';
import { uiErrorOf } from '../../../shared/api';
import { isoToDisplayDate } from '../../../shared/format';
import { ListGrid, Pager, useListPaging, type ListContext } from '../../../shared/grid';
import { displayName, useLanguage } from '../../../shared/i18n';
import {
  Button,
  CalcInfo,
  Card,
  Code,
  EmptyState,
  ErrorState,
  Fact,
  LoadingState,
  Num,
  SellDateDialog,
  SplitDialog,
  Table,
  showToast,
  type TableColumn,
} from '../../../shared/ui';
import { ARRIVAL_COLUMNS } from '../arrival-columns';
import { useArrivalsList, useChangeArrivalSplit, useSetArrivalSellDate } from '../hooks';
import { arrivalNumberText } from '../list-columns';
import type { ArrivalRow, GardenRow } from '../types';
import {
  CancelArrivalDialog,
  UndoWriteOffArrivalDialog,
  WriteOffArrivalDialog,
} from './ArrivalDialogs';

/** A list with more rows than this fills the room and scrolls inside; shorter ones are as tall as their rows. */
const SHORT_LIST_ROWS = 30;

interface GardenArrivalsSectionProps {
  garden: GardenRow;
}

export function GardenArrivalsSection({ garden }: GardenArrivalsSectionProps) {
  const { l10n } = useLocalization();
  const language = useLanguage();
  const list = useArrivalsList(garden.id);
  const paging = useListPaging(list.pager);
  const setSellDate = useSetArrivalSellDate();
  const [selectedId, setSelectedId] = useState<number | null>(null);
  const [sellDateOpen, setSellDateOpen] = useState(false);
  const [sellDateError, setSellDateError] = useState<UiError | null>(null);
  const changeSplit = useChangeArrivalSplit();
  const [splitOpen, setSplitOpen] = useState(false);
  const [splitError, setSplitError] = useState<UiError | null>(null);
  const [writeOff, setWriteOff] = useState<ArrivalRow | null>(null);
  const [undo, setUndo] = useState<ArrivalRow | null>(null);
  const [cancel, setCancel] = useState<ArrivalRow | null>(null);

  const first = list.data?.pages[0];
  const rows = useMemo(() => list.data?.pages.flatMap((page) => page.rows) ?? [], [list.data]);
  // The row that has focus; if it left the list, the first row is used.
  const selected = rows.find((row) => row.id === selectedId) ?? rows[0] ?? null;
  const say = (id: string, vars?: Record<string, string>) => l10n.getString(id, vars);

  // The total row is the backend's, over the whole filter. The callback depends on the loaded list,
  // not on an amount (Code Standards F3).
  const totals = useCallback(
    (context: ListContext) => ({
      labelField: 'number',
      values: {
        number: context.text('glossary-meezan'),
        qty_left: String(first?.totals.qty_left ?? 0),
        qty: String(first?.totals.qty ?? 0),
        qty_sold: String(first?.totals.qty_sold ?? 0),
        ...(first === undefined
          ? {}
          : { commission: context.money(first.totals.commission_paisa) }),
      },
    }),
    [first],
  );

  const lotColumns: readonly TableColumn<ArrivalRow['lots'][number]>[] = [
    {
      id: 'bikri',
      header: l10n.getString('gardens-arrival-lot-bikri'),
      kind: 'number',
      cell: (lot) => <Num>{String(lot.bikri_number)}</Num>,
    },
    {
      id: 'date',
      header: l10n.getString('purchases-col-date'),
      kind: 'number',
      cell: (lot) => <Code>{isoToDisplayDate(lot.entry_date)}</Code>,
    },
    {
      id: 'qty',
      header: l10n.getString('purchases-lot-qty-sold'),
      kind: 'number',
      cell: (lot) => <Num>{String(lot.qty_sold)}</Num>,
    },
  ];

  let body;
  if (list.isPending) {
    body = <LoadingState rows={3} />;
  } else if (list.isError) {
    body = (
      <ErrorState
        title={<Localized id="gardens-arrival-failed" />}
        error={uiErrorOf(list.error)}
        onRetry={() => {
          void list.refetch();
        }}
      />
    );
  } else if (rows.length === 0) {
    body = (
      <EmptyState
        icon={Sprout}
        title={<Localized id="gardens-arrival-empty" />}
        help={<Localized id="gardens-arrival-empty-help" />}
      />
    );
  } else {
    body = (
      <>
        <ListGrid<ArrivalRow>
          columns={ARRIVAL_COLUMNS}
          rows={rows}
          getRowId={(row) => String(row.id)}
          labelId="gardens-arrival-label"
          // Enter shows the entry in full under the list: it is already the row that has focus.
          onOpen={(row) => {
            setSelectedId(row.id);
          }}
          onFocusRow={(rowId) => {
            setSelectedId(Number(rowId));
          }}
          focusRowId={selectedId === null ? null : String(selectedId)}
          heightMode={rows.length <= SHORT_LIST_ROWS ? 'auto' : 'fill'}
          totals={totals}
          paging={paging}
        />
        <Pager pager={list.pager} />
        {selected === null ? null : (
          <div className="flex flex-col gap-3 rounded-3 border border-border bg-surface p-3">
            <Localized
              id="gardens-arrival-card-title"
              vars={{
                number: arrivalNumberText(selected.number, say),
                item: displayName(selected.item, language),
              }}
            >
              <h3 className="type-section text-ink" />
            </Localized>
            {selected.status === 'cancelled' ? (
              <Localized
                id="gardens-arrival-cancelled"
                vars={{ reason: selected.cancel_reason ?? '' }}
              >
                <p role="status" className="type-label text-due" />
              </Localized>
            ) : null}
            <div className="@container min-w-0">
              <dl className="grid grid-cols-3 gap-x-4 gap-y-3 @form3:grid-cols-5">
                <Fact label={<Localized id="gardens-arrival-col-date" />}>
                  <Code>{isoToDisplayDate(selected.entry_date)}</Code>
                </Fact>
                <Fact label={<Localized id="glossary-sell-date" />}>
                  <Code>{isoToDisplayDate(selected.sell_date)}</Code>
                </Fact>
                <Fact label={<Localized id="glossary-qism-maal" />}>
                  {displayName(selected.item, language)}
                </Fact>
                <Fact label={<Localized id="glossary-packing" />}>
                  {displayName(selected.packing, language)}
                </Fact>
                <Fact label={<Localized id="gardens-arrival-col-qty" />}>
                  <Num>{String(selected.qty)}</Num>
                </Fact>
                <Fact label={<Localized id="bikri-saved-qty-sold" />}>
                  <Num>{String(selected.qty_sold)}</Num>
                </Fact>
                <Fact label={<Localized id="bikri-stock-col-written-off" />}>
                  <Num>{String(selected.qty_written_off)}</Num>
                </Fact>
                <Fact label={<Localized id="gardens-arrival-col-left" />}>
                  <Num>{String(selected.qty_left)}</Num>
                </Fact>
                <Fact label={<Localized id="purchases-detail-qty-other" />}>
                  <Num>{String(selected.qty_other)}</Num>
                </Fact>
                <Fact label={<Localized id="goods-line-col-shop" />}>
                  {selected.shop_name ?? ''}
                </Fact>
                <Fact label={<Localized id="purchases-detail-our-sold" />}>
                  <Num>{String(selected.qty_our_sold)}</Num>
                </Fact>
                <Fact label={<Localized id="purchases-detail-other-sold" />}>
                  <Num>{String(selected.qty_other_sold)}</Num>
                </Fact>
                <Fact label={<Localized id="purchases-detail-our-left" />}>
                  <Num>{String(selected.qty_our_left)}</Num>
                </Fact>
                <Fact label={<Localized id="purchases-detail-other-left" />}>
                  <Num>{String(selected.qty_other_left)}</Num>
                </Fact>
                <Fact label={<Localized id="glossary-commission" />}>
                  <span className="inline-flex items-center gap-1">
                    <Num paisa={selected.commission_paisa} />
                    <CalcInfo working={selected.commission_working} inCell />
                  </span>
                </Fact>
                <Fact label={<Localized id="bikri-stock-col-days" />}>
                  <Num>{String(selected.days_waiting)}</Num>
                </Fact>
                {selected.note === null || selected.note === '' ? null : (
                  <Fact label={<Localized id="gardens-field-note" />}>{selected.note}</Fact>
                )}
              </dl>
            </div>
            {selected.lots.length === 0 ? null : (
              <Table
                columns={lotColumns}
                rows={selected.lots}
                rowKey={(lot) => String(lot.lot_id)}
                label={l10n.getString('gardens-arrival-lots')}
              />
            )}
            {selected.status === 'cancelled' ? null : (
              <div className="flex flex-wrap items-center gap-2">
                <Button
                  disabled={!selected.can_change_sell_date}
                  onClick={() => {
                    setSellDateOpen(true);
                  }}
                >
                  <CalendarClock aria-hidden="true" className="size-icon-md" />
                  <Localized id="purchases-change-sell-date" />
                </Button>
                <Button
                  disabled={!selected.can_change_split}
                  onClick={() => {
                    setSplitOpen(true);
                  }}
                >
                  <ArrowLeftRight aria-hidden="true" className="size-icon-md" />
                  <Localized id="split-change" />
                </Button>
                <Button
                  disabled={selected.qty_left <= 0}
                  onClick={() => {
                    setWriteOff(selected);
                  }}
                >
                  <Ban aria-hidden="true" className="size-icon-md" />
                  <Localized id="purchases-write-off" />
                </Button>
                <Button
                  disabled={selected.write_offs.length === 0}
                  onClick={() => {
                    setUndo(selected);
                  }}
                >
                  <RotateCcw aria-hidden="true" className="size-icon-md" />
                  <Localized id="purchases-undo-write-off" />
                </Button>
                <Button
                  variant="danger"
                  disabled={!selected.can_cancel}
                  onClick={() => {
                    setCancel(selected);
                  }}
                >
                  <Ban aria-hidden="true" className="size-icon-md" />
                  <Localized id="gardens-arrival-cancel" />
                </Button>
              </div>
            )}
          </div>
        )}
      </>
    );
  }

  return (
    <Card compact title={<Localized id="gardens-arrival-title" />}>
      <Localized id="gardens-arrival-help">
        <p className="type-body text-ink-muted" />
      </Localized>
      {body}
      <SellDateDialog
        open={sellDateOpen}
        onOpenChange={(open) => {
          setSellDateOpen(open);
          if (!open) setSellDateError(null);
        }}
        title={<Localized id="gardens-arrival-sell-date-title" />}
        description={
          selected === null ? null : (
            <Localized
              id="gardens-arrival-sell-date-text"
              vars={{ number: arrivalNumberText(selected.number, say) }}
            >
              <span />
            </Localized>
          )
        }
        current={selected === null ? '' : isoToDisplayDate(selected.sell_date)}
        busy={setSellDate.isPending}
        error={sellDateError}
        onSave={(sellDate) => {
          if (selected === null) return;
          setSellDateError(null);
          setSellDate
            .mutateAsync({ id: selected.id, version: selected.version, sell_date: sellDate })
            .then(() => {
              showToast('success', <Localized id="purchases-sell-date-done" />);
              setSellDateOpen(false);
            })
            .catch((failure: unknown) => {
              setSellDateError(uiErrorOf(failure));
            });
        }}
      />
      <SplitDialog
        open={splitOpen}
        onOpenChange={(open) => {
          setSplitOpen(open);
          if (!open) setSplitError(null);
        }}
        title={<Localized id="split-title" />}
        description={
          selected === null ? null : (
            <Localized
              id="split-text"
              vars={{
                number: arrivalNumberText(selected.number, say),
                item: displayName(selected.item, language),
                packing: displayName(selected.packing, language),
                qty_left: String(selected.qty_left),
                our_left: String(selected.qty_our_left),
                other_left: String(selected.qty_other_left),
              }}
            >
              <span />
            </Localized>
          )
        }
        qty={selected?.qty ?? 0}
        currentOtherQty={selected?.qty_other ?? 0}
        currentShop={selected?.shop_name ?? null}
        busy={changeSplit.isPending}
        error={splitError}
        onSave={(otherQty, shopName) => {
          if (selected === null) return;
          setSplitError(null);
          changeSplit
            .mutateAsync({
              id: selected.id,
              version: selected.version,
              other_qty: otherQty,
              shop_name: shopName === '' ? null : shopName,
            })
            .then(() => {
              showToast('success', <Localized id="split-done" />);
              setSplitOpen(false);
            })
            .catch((failure: unknown) => {
              setSplitError(uiErrorOf(failure));
            });
        }}
      />
      <WriteOffArrivalDialog
        row={writeOff}
        onClose={() => {
          setWriteOff(null);
        }}
      />
      <UndoWriteOffArrivalDialog
        row={undo}
        onClose={() => {
          setUndo(null);
        }}
      />
      <CancelArrivalDialog
        row={cancel}
        onClose={() => {
          setCancel(null);
        }}
      />
    </Card>
  );
}
