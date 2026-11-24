// The goods arrived entries of one garden (مال آیا, PRD FR-12.40): the garden's stock. Each entry
// says what came (item, packing, quantity), the day to sell and what is left. A new بکری holds a
// lot for every entry whose day to sell has come and that still has goods left; goods not sold
// stay here and come again the next day. None of these posts any money.
//
// A busy garden has about 90 of them (plan 96), so this section is a list with a place of its own and
// nothing under it that moves: a search box (a number, an item or another shop), the list at 30 rows a
// page with the newest first and the backend's totals, and the details of the entry that has focus:
// beside the list from the normal window width, in a window of its own on a narrow one. The details
// hold what can be done with the entry: change the day to sell, change the split, write off, undo a
// write-off, cancel, edit. The keys: ↑ ↓ move between entries, Enter goes to the buttons of the
// details (or opens them), Esc goes back to the list.
//
// Four states: loading, an error with a retry, nothing yet (a helpful message), and the rows; a search
// with no match says so and offers to clear it.

import { Localized, useLocalization } from '@fluent/react';
import { useRouter } from '@tanstack/react-router';
import { Package, SearchX, Sprout } from 'lucide-react';
import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import type { UiError } from '../../../bindings/UiError';
import { uiErrorOf } from '../../../shared/api';
import { useFixDayFlow } from '../../../shared/edit';
import { isoToDisplayDate } from '../../../shared/format';
import { ListGrid, Pager, useListPaging, type ListContext } from '../../../shared/grid';
import { isFromStep, useLayoutStep } from '../../../shared/hooks';
import { displayName, useLanguage } from '../../../shared/i18n';
import { useEscape } from '../../../shared/keyboard';
import {
  Button,
  Card,
  EmptyState,
  ErrorState,
  FilterBar,
  FormDialog,
  LoadingState,
  SellDateDialog,
  SplitDialog,
  showToast,
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
import { ArrivalDetail } from './ArrivalDetail';

/** A list with more rows than this has the height of its own and scrolls inside; a shorter one is as tall as its rows. */
const SHORT_LIST_ROWS = 10;
/** How long the rows an entry has just added stay tinted. */
const MARK_MS = 6000;
const NO_ROWS: readonly string[] = [];

/** The goods arrived entries a save has just added (plan 96): the list shows them first, tinted. */
export interface AddedArrivals {
  /** The ids of the new entries, in the order they were saved. */
  ids: readonly number[];
  /** A new number for each save, so the same ids twice are still two saves. */
  token: number;
}

interface GardenArrivalsSectionProps {
  garden: GardenRow;
  /** Edit was pressed on an arrival whose day is open (or was reopened for it): the page edits it. */
  onEdit: (arrival: ArrivalRow) => void;
  /** The entries a save has just added: the list goes to its first page, clears its search and tints them. */
  added?: AddedArrivals | null;
}

export function GardenArrivalsSection({
  garden,
  onEdit,
  added = null,
}: GardenArrivalsSectionProps) {
  const { l10n } = useLocalization();
  const router = useRouter();
  const fix = useFixDayFlow();
  const language = useLanguage();
  const docked = isFromStep(useLayoutStep(), 'normal');
  const [search, setSearch] = useState('');
  const list = useArrivalsList(garden.id, { search });
  const paging = useListPaging(list.pager);
  const setSellDate = useSetArrivalSellDate();
  const [selectedId, setSelectedId] = useState<number | null>(null);
  const [detailOpen, setDetailOpen] = useState(false);
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
  // The row that has focus; if it left the list, the first row is used.
  const selected = rows.find((row) => row.id === selectedId) ?? rows[0] ?? null;
  const say = (id: string, vars?: Record<string, string>) => l10n.getString(id, vars);
  // On a narrow screen the details are a window of their own; it never stays open on a wide one.
  const detailShown = !docked && detailOpen && selected !== null;

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

  // Esc in the details beside the list goes back to the entry in the list (PRD FR-7.18).
  useEscape(
    'edit',
    () => {
      const cell =
        listRef.current?.querySelector<HTMLElement>('.ag-cell-focus') ??
        listRef.current?.querySelector<HTMLElement>('.ag-cell');
      cell?.focus();
    },
    () => docked && panelRef.current?.contains(document.activeElement) === true,
  );

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

  /** What a button of the details does: the window of the details gives way to the box of the button. */
  const act = (run: (arrival: ArrivalRow) => void) => () => {
    if (selected === null) return;
    setDetailOpen(false);
    run(selected);
  };
  const details =
    selected === null ? null : (
      <ArrivalDetail
        key={selected.id}
        arrival={selected}
        busy={fix.busy}
        titleless={detailShown}
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
        title={<Localized id="gardens-arrival-failed" />}
        error={uiErrorOf(list.error)}
        onRetry={() => {
          void list.refetch();
        }}
      />
    );
  } else if (rows.length === 0 && search.trim() === '') {
    body = (
      <EmptyState
        icon={Sprout}
        title={<Localized id="gardens-arrival-empty" />}
        help={<Localized id="gardens-arrival-empty-help" />}
      />
    );
  } else {
    body = (
      <div className="flex min-w-0 flex-col gap-3">
        <div className="max-w-form">
          <FilterBar
            screen="gardens.arrivals"
            search={{
              value: search,
              onChange: setSearch,
              labelId: 'gardens-arrival-search-label',
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
          <div className="flex min-w-0 items-start gap-3">
            <div className="flex min-w-0 flex-1 flex-col gap-3">
              <div ref={listRef} className={long ? 'h-arrivals-region' : undefined}>
                <ListGrid<ArrivalRow>
                  columns={ARRIVAL_COLUMNS}
                  rows={rows}
                  getRowId={(row) => String(row.id)}
                  labelId="gardens-arrival-label"
                  // Enter goes to the buttons of the details, or opens them in their window.
                  onOpen={(row) => {
                    setSelectedId(row.id);
                    if (!docked) {
                      setDetailOpen(true);
                      return;
                    }
                    window.requestAnimationFrame(() => {
                      panelRef.current
                        ?.querySelector<HTMLElement>('button:not([disabled])')
                        ?.focus();
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
              <Localized id={docked ? 'gardens-arrival-pick-help' : 'gardens-arrival-open-help'}>
                <p className="type-body text-ink-muted" />
              </Localized>
            </div>
            {docked && details !== null ? (
              <aside
                ref={panelRef}
                aria-label={l10n.getString('gardens-arrival-card-title', {
                  number: arrivalNumberText(selected?.number ?? 0, say),
                  item: selected === null ? '' : displayName(selected.item, language),
                })}
                className="sticky top-0 max-h-arrivals-region w-arrivals-panel shrink-0 self-start overflow-y-auto rounded-3 border border-border bg-surface p-3"
              >
                {details}
              </aside>
            ) : null}
          </div>
        )}
      </div>
    );
  }

  return (
    <Card compact icon={Package} title={<Localized id="gardens-arrival-title" />}>
      <Localized id="gardens-arrival-help">
        <p className="type-body text-ink-muted" />
      </Localized>
      {body}
      {detailShown ? (
        <FormDialog
          open
          onOpenChange={setDetailOpen}
          title={
            <Localized
              id="gardens-arrival-card-title"
              vars={{
                number: arrivalNumberText(selected.number, say),
                item: displayName(selected.item, language),
              }}
            >
              <span />
            </Localized>
          }
        >
          {details}
          <div>
            <Button
              onClick={() => {
                setDetailOpen(false);
              }}
            >
              <Localized id="gardens-arrival-detail-close" />
            </Button>
          </div>
        </FormDialog>
      ) : null}
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
