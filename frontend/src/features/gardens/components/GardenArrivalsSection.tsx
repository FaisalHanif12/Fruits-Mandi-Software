// The goods arrived entries of one garden (مال آیا, PRD FR-12.40, FR-12.53), in two sections: the goods
// not sold yet ("بکنا باقی": arrived, sold so far, left for our shop and for another shop, the days
// waiting and the day to sell) and the goods sold ("بک چکا": arrived, sold in our shop and in another
// shop, written off, the sale value, the commission and the last sale). Each section has its own list
// at 30 rows a page with the newest first, its own search (a number, an item or another shop) and the
// backend's totals over its whole filter. None of these posts any money.
//
// The details of the entry that has focus open in place, under its list: its facts, what can be done
// with it (change the day to sell, change the split, write off, undo a write-off, cancel, edit) and
// its sale lines with where their money is. The goods not sold yet show the first entry's details from
// the start; the goods sold show them once an entry is chosen. The keys: ↑ ↓ move between entries,
// Enter goes to the buttons of the details, Esc goes back to the list.
//
// Four states: loading, an error with a retry, nothing yet (a helpful message), and the rows; a search
// with no match says so and offers to clear it.

import { Localized, useLocalization } from '@fluent/react';
import { useRouter } from '@tanstack/react-router';
import { CircleCheck, Package, ReceiptText, SearchX, Sprout } from 'lucide-react';
import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import type { UiError } from '../../../bindings/UiError';
import { uiErrorOf } from '../../../shared/api';
import { useFixDayFlow } from '../../../shared/edit';
import { isoToDisplayDate } from '../../../shared/format';
import { ListGrid, Pager, useListPaging, type ListContext } from '../../../shared/grid';
import { displayName, useLanguage } from '../../../shared/i18n';
import { useEscape } from '../../../shared/keyboard';
import {
  Button,
  Card,
  EmptyState,
  ErrorState,
  FilterBar,
  LoadingState,
  SellDateDialog,
  SplitDialog,
  showToast,
} from '../../../shared/ui';
import { ARRIVAL_IN_STOCK_COLUMNS, ARRIVAL_SOLD_COLUMNS } from '../arrival-columns';
import { useArrivalsList, useChangeArrivalSplit, useSetArrivalSellDate } from '../hooks';
import { arrivalNumberText } from '../list-columns';
import type { ArrivalRow, GardenRow } from '../types';
import {
  CancelArrivalDialog,
  UndoWriteOffArrivalDialog,
  WriteOffArrivalDialog,
} from './ArrivalDialogs';
import { ArrivalDetail } from './ArrivalDetail';
import { SaleMoneyList } from './SaleMoneyList';

/** A list with more rows than this has the height of its own and scrolls inside; a shorter one is as tall as its rows. */
const SHORT_LIST_ROWS = 10;
/** How long the rows an entry has just added stay tinted. */
const MARK_MS = 6000;
const NO_ROWS: readonly string[] = [];

/** Which of the two sections of the goods arrived: not sold yet, or sold (PRD FR-12.53). */
export type ArrivalStock = 'in_stock' | 'sold_out';

/** The words of each section: its list, heading, search and empty states. */
const SECTION = {
  in_stock: {
    columns: ARRIVAL_IN_STOCK_COLUMNS,
    icon: Package,
    titleId: 'gardens-arrival-title',
    helpId: 'gardens-arrival-help',
    labelId: 'gardens-arrival-label',
    searchLabelId: 'gardens-arrival-search-label',
    failedId: 'gardens-arrival-failed',
    emptyIcon: Sprout,
    emptyId: 'gardens-arrival-empty',
    emptyHelpId: 'gardens-arrival-empty-help',
  },
  sold_out: {
    columns: ARRIVAL_SOLD_COLUMNS,
    icon: CircleCheck,
    titleId: 'gardens-sold-title',
    helpId: 'gardens-sold-help',
    labelId: 'gardens-sold-label',
    searchLabelId: 'gardens-sold-search-label',
    failedId: 'gardens-sold-failed',
    emptyIcon: ReceiptText,
    emptyId: 'gardens-sold-empty',
    emptyHelpId: 'gardens-sold-empty-help',
  },
} as const;

/** The goods arrived entries a save has just added (plan 96): the list shows them first, tinted. */
export interface AddedArrivals {
  /** The ids of the new entries, in the order they were saved. */
  ids: readonly number[];
  /** A new number for each save, so the same ids twice are still two saves. */
  token: number;
}

interface GardenArrivalsSectionProps {
  garden: GardenRow;
  stock: ArrivalStock;
  /** Edit was pressed on an arrival whose day is open (or was reopened for it): the page edits it. */
  onEdit: (arrival: ArrivalRow) => void;
  /** The entries a save has just added: the list goes to its first page, clears its search and tints them. */
  added?: AddedArrivals | null;
}

export function GardenArrivalsSection({
  garden,
  stock,
  onEdit,
  added = null,
}: GardenArrivalsSectionProps) {
  const { l10n } = useLocalization();
  const router = useRouter();
  const fix = useFixDayFlow();
  const language = useLanguage();
  const section = SECTION[stock];
  const [search, setSearch] = useState('');
  const list = useArrivalsList(garden.id, { state: stock, search });
  const paging = useListPaging(list.pager);
  const setSellDate = useSetArrivalSellDate();
  const [selectedId, setSelectedId] = useState<number | null>(null);
  const [markedIds, setMarkedIds] = useState<readonly string[]>(NO_ROWS);
  const [sellDateOpen, setSellDateOpen] = useState(false);
  const [sellDateError, setSellDateError] = useState<UiError | null>(null);
  const changeSplit = useChangeArrivalSplit();
  const [splitOpen, setSplitOpen] = useState(false);
  const [splitError, setSplitError] = useState<UiError | null>(null);
  const [writeOff, setWriteOff] = useState<ArrivalRow | null>(null);
  const [undo, setUndo] = useState<ArrivalRow | null>(null);
  const [cancel, setCancel] = useState<ArrivalRow | null>(null);
  const panelRef = useRef<HTMLElement>(null);
  const listRef = useRef<HTMLDivElement>(null);

  const first = list.data?.pages[0];
  const rows = useMemo(() => list.data?.pages.flatMap((page) => page.rows) ?? [], [list.data]);
  // The row that has focus. The goods not sold yet start on the first entry (and fall back to it
  // when the chosen one leaves the list); the goods sold show details once an entry is chosen.
  const chosen = rows.find((row) => row.id === selectedId) ?? null;
  const selected =
    chosen ?? (stock === 'in_stock' || selectedId !== null ? (rows[0] ?? null) : null);
  const say = (id: string, vars?: Record<string, string>) => l10n.getString(id, vars);

  // A save has just added entries: the first page of the whole list, the new rows first and tinted, the
  // newest of them in the details. The search is cleared so none of them is hidden (plan 96).
  const seen = useRef(0);
  const { goTo } = list.pager;
  useEffect(() => {
    if (added === null || added.token === seen.current) return;
    seen.current = added.token;
    setSearch('');
    goTo(1);
    setSelectedId(added.ids.at(-1) ?? null);
    setMarkedIds(added.ids.map(String));
  }, [added, goTo]);
  useEffect(() => {
    if (markedIds.length === 0) return;
    const timer = window.setTimeout(() => {
      setMarkedIds(NO_ROWS);
    }, MARK_MS);
    return () => {
      window.clearTimeout(timer);
    };
  }, [markedIds]);

  // Esc in the details under the list goes back to the entry in the list (PRD FR-7.18).
  useEscape(
    'edit',
    () => {
      const cell =
        listRef.current?.querySelector<HTMLElement>('.ag-cell-focus') ??
        listRef.current?.querySelector<HTMLElement>('.ag-cell');
      cell?.focus();
    },
    () => panelRef.current?.contains(document.activeElement) === true,
  );

  // The total row is the backend's, over the whole filter. The callback depends on the loaded list,
  // not on an amount (Code Standards F3).
  const totals = useCallback(
    (context: ListContext) => {
      const sum = first?.totals;
      return {
        labelField: 'number',
        values: {
          number: context.text('glossary-meezan'),
          ...(sum === undefined
            ? {}
            : {
                qty: String(sum.qty),
                qty_sold: String(sum.qty_sold),
                qty_our_left: String(sum.qty_our_left),
                qty_other_left: String(sum.qty_other_left),
                qty_our_sold: String(sum.qty_our_sold),
                qty_other_sold: String(sum.qty_other_sold),
                qty_written_off: String(sum.qty_written_off),
                sale_value: context.money(sum.sale_value_paisa),
                commission: context.money(sum.commission_paisa),
              }),
        },
      };
    },
    [first],
  );

  /** What a button of the details does, for the entry they show. */
  const act = (run: (arrival: ArrivalRow) => void) => () => {
    if (selected === null) return;
    run(selected);
  };
  const details =
    selected === null ? null : (
      <ArrivalDetail
        key={selected.id}
        arrival={selected}
        busy={fix.busy}
        onEdit={act((arrival) => {
          if (!arrival.needs_reopen) {
            onEdit(arrival);
            return;
          }
          // A closed day: the days are reopened first, then the arrival is edited (FR-21.4).
          fix.start({
            date: arrival.entry_date,
            ownerHref: router.state.location.href,
            then: () => {
              onEdit(arrival);
            },
          });
        })}
        onChangeSellDate={act(() => {
          setSellDateOpen(true);
        })}
        onChangeSplit={act(() => {
          setSplitOpen(true);
        })}
        onWriteOff={act(setWriteOff)}
        onUndoWriteOff={act(setUndo)}
        onCancel={act(setCancel)}
      />
    );

  const long = rows.length > SHORT_LIST_ROWS;
  let body;
  if (list.isPending) {
    body = <LoadingState rows={3} />;
  } else if (list.isError) {
    body = (
      <ErrorState
        title={<Localized id={section.failedId} />}
        error={uiErrorOf(list.error)}
        onRetry={() => {
          void list.refetch();
        }}
      />
    );
  } else if (rows.length === 0 && search.trim() === '') {
    body = (
      <EmptyState
        icon={section.emptyIcon}
        title={<Localized id={section.emptyId} />}
        help={<Localized id={section.emptyHelpId} />}
      />
    );
  } else {
    body = (
      <div className="flex min-w-0 flex-col gap-3">
        <div className="max-w-form">
          <FilterBar
            screen={stock === 'in_stock' ? 'gardens.arrivals' : 'gardens.arrivals_sold'}
            search={{
              value: search,
              onChange: setSearch,
              labelId: section.searchLabelId,
              placeholderId: 'gardens-arrival-search-hint',
            }}
          />
        </div>
        {rows.length === 0 ? (
          <EmptyState
            icon={SearchX}
            title={<Localized id="gardens-arrival-no-match" />}
            help={<Localized id="gardens-arrival-no-match-help" />}
            action={
              <Button
                onClick={() => {
                  setSearch('');
                }}
              >
                <Localized id="gardens-arrival-search-clear" />
              </Button>
            }
          />
        ) : (
          <div className="flex min-w-0 flex-col gap-3">
            <div ref={listRef} className={long ? 'h-arrivals-region' : undefined}>
              <ListGrid<ArrivalRow>
                columns={section.columns}
                rows={rows}
                getRowId={(row) => String(row.id)}
                labelId={section.labelId}
                // Enter goes to the buttons of the details under the list.
                onOpen={(row) => {
                  setSelectedId(row.id);
                  window.requestAnimationFrame(() => {
                    panelRef.current?.querySelector<HTMLElement>('button:not([disabled])')?.focus();
                  });
                }}
                onFocusRow={(rowId) => {
                  setSelectedId(Number(rowId));
                }}
                focusRowId={selectedId === null ? null : String(selectedId)}
                markedRowIds={markedIds}
                heightMode={long ? 'fill' : 'auto'}
                isInactive={(row) => row.status === 'cancelled'}
                sort={{ value: list.sort, onChange: list.setSort }}
                totals={totals}
                paging={paging}
              />
            </div>
            <Pager pager={list.pager} />
            <Localized id="gardens-arrival-pick-help">
              <p className="type-body text-ink-muted" />
            </Localized>
            {selected === null ? null : (
              <aside
                ref={panelRef}
                aria-label={l10n.getString('gardens-arrival-card-title', {
                  number: arrivalNumberText(selected.number, say),
                  item: displayName(selected.item, language),
                })}
                className="flex min-w-0 flex-col gap-3 rounded-3 border border-border bg-surface p-3"
              >
                {details}
                {/* Its sales, and where their money is, in place (FR-12.53). */}
                <div className="flex min-w-0 flex-col gap-2">
                  <Localized id="gardens-arrival-lots">
                    <h4 className="type-label text-ink" />
                  </Localized>
                  <SaleMoneyList
                    key={selected.id}
                    gardenId={garden.id}
                    filter={{ from: null, to: null, arrivalId: selected.id }}
                    screen="gardens.arrival_sales"
                    labelId="gardens-arrival-sales-label"
                    labelVars={{ number: arrivalNumberText(selected.number, say) }}
                    empty={
                      <Localized id="gardens-arrival-no-sales">
                        <p className="type-body text-ink-muted" />
                      </Localized>
                    }
                  />
                </div>
              </aside>
            )}
          </div>
        )}
      </div>
    );
  }

  return (
    <Card compact icon={section.icon} title={<Localized id={section.titleId} />}>
      <Localized id={section.helpId}>
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
      {fix.dialog}
      <CancelArrivalDialog
        row={cancel}
        onClose={() => {
          setCancel(null);
        }}
      />
    </Card>
  );
}
