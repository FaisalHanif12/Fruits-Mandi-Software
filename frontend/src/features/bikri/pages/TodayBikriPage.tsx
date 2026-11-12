// پچھلی بکری / Past Sales (route /bikri/today; PRD 12.7, plan 72). Two parts under one set of
// filters: today's بکری on top, and under a divider bar the window of 7 earlier days, grouped by day
// with a heading for each (the date, the count and the خام بکری of the day, all the backend's). The
// window starts on yesterday and moves 7 days at a time (the two buttons, Page Up and Page Down in
// the window), or to the day typed in "تاریخ پر جائیں". "پوری بکری دکھائیں" sets the filters aside
// (they keep their values). A click or Enter on a row opens the whole بکری on the sale book; the way
// back from an edit lands on the same window and row, and every figure is read again (P10).
//
// The window, the row and the tick are in the address, so a reload or a way back finds them. Four
// states in each list: loading, an error with a retry, empty with words, and the rows. No figure is
// worked out here (Code Standards F3).

import { Localized } from '@fluent/react';
import { useNavigate, useSearch } from '@tanstack/react-router';
import { Plus, SearchX } from 'lucide-react';
import { useEffect, useMemo, useState } from 'react';
import { useDayStatus } from '../../../app/use-day-status';
import type { BikriListRow } from '../../../bindings/BikriListRow';
import type { BikriStatus } from '../../../bindings/BikriStatus';
import { uiErrorOf } from '../../../shared/api';
import { isoToDisplayDate } from '../../../shared/format';
import { isLongList } from '../../../shared/grid';
import { useFocusFirstField, useLayoutStep, useRememberedFlag } from '../../../shared/hooks';
import { SHORTCUTS, formNavigationProps, useEscape } from '../../../shared/keyboard';
import { Remembered, type MemoryProps } from '../../../shared/screen-state';
import {
  AccountPicker,
  Button,
  Checkbox,
  EmptyState,
  ErrorState,
  Field,
  FilterBar,
  ListPage,
  LoadingState,
  Picker,
  Select,
  ShowClosedTick,
  type PickerValue,
} from '../../../shared/ui';
import { PastDivider } from '../components/PastDivider';
import { PastRowActions } from '../components/PastRowActions';
import { PastSectionHeader } from '../components/PastSectionHeader';
import { PastWeekNav } from '../components/PastWeekNav';
import { PastGoTo } from '../components/PastGoTo';
import { PastList, rowsOf } from '../components/PastList';
import { useBikriList, useBikriWindow } from '../hooks/useBikriReads';
import { usePastGroups } from '../hooks/usePastGroups';
import { usePastPrintScreen } from '../hooks/usePastPrintScreen';
import { bikriListColumns } from '../list-columns';
import type { BikriList } from '../../../bindings/BikriList';
import type { BikriListInput } from '../../../bindings/BikriListInput';
import {
  lastWindowEnd,
  parsePastSearch,
  stepWindowEnd,
  windowEnd,
  windowOf,
  type PastReturn,
  type PastSearch,
} from '../past-window';
import { TODAY_SCREEN, todayChoicesFrom, todayChoicesText } from '../remember';

const ALL = 'all';
type StatusChoice = BikriStatus | typeof ALL;

/** The filters of the two lists: the Owner's own, or none while "show every بکری" is ticked. */
type PastFilter = Omit<BikriListInput, 'offset' | 'limit' | 'sort' | 'date' | 'from' | 'to'>;

/** The row of a page that has focus, or null. */
function rowOf(page: BikriList | undefined, rowId: string | null): BikriListRow | null {
  if (page === undefined || rowId === null) return null;
  return page.rows.find((row) => String(row.id) === rowId) ?? null;
}

/** The filters start as the Owner left them (PRD FR-7.45); the window always starts on yesterday. */
export function TodayBikriPage() {
  return <Remembered screen={TODAY_SCREEN}>{(memory) => <PastBikriBody {...memory} />}</Remembered>;
}

function PastBikriBody({ saved, remember }: MemoryProps) {
  const navigate = useNavigate();
  const rawSearch: Record<string, unknown> = useSearch({ strict: false });
  const address = parsePastSearch(rawSearch);
  const start = useMemo(() => todayChoicesFrom(saved), [saved]);
  const day = useDayStatus();
  const today = day.data?.today ?? null;
  const end = today === null ? null : windowEnd(address.end ?? null, today);
  const showAll = address.all === true;

  const [seller, setSeller] = useState<PickerValue | null>(start.seller);
  const [item, setItem] = useState<PickerValue | null>(start.item);
  const [status, setStatus] = useState<StatusChoice>(start.status);
  const [search, setSearch] = useState('');
  // Cancelled بکری step aside until the tick is on (plan 38 R5). A chosen status still decides.
  const [showFinished, setShowFinished] = useState(false);
  const moreSet = [seller !== null, item !== null, status !== ALL].filter(Boolean).length;
  const filtersSet = moreSet > 0 || search.trim() !== '' || showFinished;
  useEffect(() => {
    remember(todayChoicesText({ seller, item, status }));
  }, [seller, item, status, remember]);

  // "Show every بکری" sets the filters aside: the backend is told so, and they keep their values.
  const filter: PastFilter = {
    seller_id: showAll ? null : (seller?.id ?? null),
    item_id: showAll ? null : (item?.id ?? null),
    status: showAll || status === ALL ? null : status,
    show_finished: showAll ? false : showFinished,
    search: showAll || search.trim() === '' ? null : search.trim(),
    group_by_month: false,
    // Only the window asks for the days (below): today's section is one day.
    group_by_day: false,
    all: showAll,
  };
  const range = end === null ? null : windowOf(end);
  const todayList = useBikriList({
    ...filter,
    date: today === null ? null : isoToDisplayDate(today),
    from: null,
    to: null,
  });
  const windowList = useBikriWindow({
    ...filter,
    group_by_day: true,
    date: null,
    from: range === null ? null : isoToDisplayDate(range.from),
    to: range === null ? null : isoToDisplayDate(range.to),
  });

  const step = useLayoutStep();
  // Print, PDF and Excel follow the screen exactly: the same window, filters and switch (PRD FR-12.46).
  // Every row goes, not only the page on screen. The buttons are dimmed until the days are known.
  const print = usePastPrintScreen(
    range === null
      ? null
      : {
          from: isoToDisplayDate(range.from),
          to: isoToDisplayDate(range.to),
          seller_id: filter.seller_id,
          item_id: filter.item_id,
          status: filter.status,
          show_finished: filter.show_finished,
          search: filter.search,
          all: showAll,
        },
    // At 1024 px wide Print is its icon and its key and Excel its icon, so the title row is not crowded.
    { compact: step === 'compact' },
  );

  // The tick that adds the نگ column, which does not fit beside the others (FR-12.7). It is kept.
  const [moreColumns, setMoreColumns] = useRememberedFlag('bikri.list_more_columns', false);
  const columns = useMemo(() => bikriListColumns(step, moreColumns), [step, moreColumns]);
  // The row the Owner came back to (plan 72 P10) has focus when the list opens; otherwise the date box
  // does. The row that has focus is kept here, so a language switch keeps it too.
  // Each part has its own: the two buttons above a part work on the row that has focus in it.
  const [focusToday, setFocusToday] = useState<string | null>(
    address.row === undefined ? null : String(address.row),
  );
  const [focusWindow, setFocusWindow] = useState<string | null>(
    address.row === undefined ? null : String(address.row),
  );
  const searchRef = useFocusFirstField<HTMLDivElement>();

  /** Goes to a window or a tick: the address says where the list is, so a way back finds it. */
  const go = (next: PastSearch) => {
    void navigate({ to: '/bikri/today', search: next, replace: true });
  };
  const goEnd = (wanted: string) => {
    if (today === null) return;
    const kept = windowEnd(wanted, today);
    // The window that ends yesterday is the one a new day moves on with: it is not written down.
    go({
      ...(kept === lastWindowEnd(today) ? {} : { end: kept }),
      ...(showAll ? { all: true as const } : {}),
    });
  };
  const move = (direction: -1 | 1) => {
    if (end === null || today === null) return;
    const next = stepWindowEnd(end, direction, today);
    if (next !== end) goEnd(next);
  };
  const openRow = (row: BikriListRow) => {
    void navigate({
      to: '/bikri/book',
      search: {
        id: row.id,
        ...(end === null ? {} : { past: end }),
        ...(showAll ? { pastAll: true as const } : {}),
      },
    });
  };
  const openNew = () => {
    void navigate({ to: '/bikri/book', search: {} });
  };
  // Esc leaves the screen, unless the preview of the print is open: it closes that first.
  useEscape(
    'leave',
    () => {
      void navigate({ to: '/' });
    },
    print.preview === null,
  );

  const newButton = (
    <Button variant="primary" shortcut={SHORTCUTS.newBikri} onClick={openNew}>
      <Plus aria-hidden="true" className="size-icon-md" />
      <Localized id="bikri-new" />
    </Button>
  );
  const clearFilters = () => {
    setSeller(null);
    setItem(null);
    setStatus(ALL);
    setSearch('');
    setShowFinished(false);
  };

  const todayRows = rowsOf(todayList.data, todayList.isError);
  const todayFirst = todayList.data?.pages[0];
  const windowFirst = windowList.data?.pages[0];
  const groups = usePastGroups(
    windowFirst,
    windowList.pager.page,
    windowList.pager.pageCount,
    windowList.sort !== null,
  );

  // Today has no بکری: one line, because the window of earlier days is under it.
  const todayEmpty = (
    <p className="type-body text-ink-muted">
      {filtersSet && !showAll ? (
        <Localized id="bikri-past-today-no-match" />
      ) : (
        <Localized id="bikri-past-today-empty" vars={{ key: SHORTCUTS.newBikri.label }} />
      )}
    </p>
  );
  // The whole window has no بکری (P12): why, and what to do.
  const windowEmpty =
    filtersSet && !showAll ? (
      <EmptyState
        icon={SearchX}
        title={<Localized id="bikri-list-no-match" />}
        help={<Localized id="bikri-list-no-match-help" />}
        action={
          <Button onClick={clearFilters}>
            <Localized id="bikri-list-clear" />
          </Button>
        }
      />
    ) : (
      <EmptyState
        icon={SearchX}
        title={<Localized id="bikri-past-window-empty" />}
        help={<Localized id="bikri-past-window-empty-help" />}
      />
    );

  const filters = (
    <div {...formNavigationProps} className="flex flex-col gap-3">
      {/* The date box is above the filters (PRD 12.7). It is not the first thing in focus: the search
          box is, since a number or a truck is what is looked for most (PRD FR-7.1). */}
      <PastGoTo today={today} onGo={goEnd} />
      <div ref={address.row === undefined ? searchRef : undefined}>
        <FilterBar
          screen="bikri.today"
          search={{
            value: search,
            onChange: setSearch,
            placeholderId: 'bikri-list-search-placeholder',
          }}
          moreSet={moreSet}
          more={
            <>
              <Field label={<Localized id="glossary-beopari" />}>
                {(control) => (
                  <AccountPicker
                    {...control}
                    canAdd={false}
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
                    canAdd={false}
                    scope="item"
                    value={item}
                    onChange={(hit) => {
                      setItem(hit);
                    }}
                  />
                )}
              </Field>
              <Field label={<Localized id="bikri-list-col-status" />}>
                {(control) => (
                  <Select
                    {...control}
                    value={status}
                    onChange={setStatus}
                    options={[
                      { value: ALL, label: <Localized id="bikri-list-status-all" /> },
                      { value: 'saved', label: <Localized id="bikri-status-saved" /> },
                      { value: 'cancelled', label: <Localized id="bikri-status-cancelled" /> },
                    ]}
                  />
                )}
              </Field>
            </>
          }
          extra={
            <>
              <Checkbox
                checked={showAll}
                onChange={(checked) => {
                  go({
                    ...(address.end === undefined ? {} : { end: address.end }),
                    ...(checked ? { all: true as const } : {}),
                  });
                }}
                label={<Localized id="bikri-past-show-all" />}
              />
              <ShowClosedTick checked={showFinished} onChange={setShowFinished} />
              <Checkbox
                checked={moreColumns}
                onChange={setMoreColumns}
                label={<Localized id="common-more-columns" />}
              />
            </>
          }
        />
      </div>
      {showAll ? (
        <p className="type-body text-ink-muted">
          <Localized id="bikri-past-show-all-note" />
        </p>
      ) : null}
      {print.notices}
    </div>
  );

  /** The window and the switch the lists are on: where the Owner comes back to after an edit. */
  const wayBack: PastReturn = {
    past: end ?? '',
    ...(showAll ? { pastAll: true as const } : {}),
  };

  let body;
  if (day.isError) {
    body = (
      <ErrorState
        title={<Localized id="bikri-load-failed" />}
        error={uiErrorOf(day.error)}
        onRetry={() => {
          void day.refetch();
        }}
      />
    );
  } else if (today === null || range === null) {
    body = <LoadingState />;
  } else {
    body = (
      <>
        <section
          className={`flex flex-col gap-2 ${isLongList(todayRows?.length ?? 0) ? 'min-h-0 flex-1' : 'shrink-0'}`}
        >
          <div className="flex flex-wrap items-center justify-between gap-x-4 gap-y-2">
            <PastSectionHeader
              part="today"
              date={today}
              count={todayFirst?.total_count ?? null}
              gross={todayFirst?.totals.gross ?? null}
            />
            <PastRowActions row={rowOf(todayFirst, focusToday)} back={wayBack} />
          </div>
          <PastList
            query={todayList}
            columns={columns}
            labelId="bikri-list-label"
            failedId="bikri-list-failed"
            empty={todayEmpty}
            onOpen={openRow}
            focusRowId={focusToday}
            onFocusRow={setFocusToday}
          />
        </section>
        <PastDivider from={range.from} to={range.to} />
        <section className="flex min-h-0 flex-1 flex-col gap-2">
          <div className="flex flex-wrap items-center justify-between gap-x-4 gap-y-2">
            <PastSectionHeader
              part="window"
              from={range.from}
              to={range.to}
              count={windowFirst?.total_count ?? null}
              gross={windowFirst?.totals.gross ?? null}
            />
            <PastRowActions row={rowOf(windowFirst, focusWindow)} back={wayBack} />
          </div>
          <PastList
            query={windowList}
            columns={columns}
            labelId="bikri-past-label"
            failedId="bikri-past-failed"
            empty={windowEmpty}
            onOpen={openRow}
            focusRowId={focusWindow}
            onFocusRow={setFocusWindow}
            groups={groups}
            onPageKey={(direction) => {
              move(direction === 'previous' ? -1 : 1);
            }}
          />
        </section>
        {/* The buttons that move the window are at the end, after the table and its page buttons. */}
        <PastWeekNav
          end={range.to}
          today={today}
          onEarlier={() => {
            move(-1);
          }}
          onNearer={() => {
            move(1);
          }}
          onLatest={() => {
            goEnd(lastWindowEnd(today));
          }}
        />
      </>
    );
  }
  // The preview of the print takes the place of the page while it is open; the filters stay as they were.
  if (print.preview !== null) return print.preview;

  return (
    <ListPage
      titleId="layout-module-today"
      help={
        <Localized id="bikri-list-help">
          <p />
        </Localized>
      }
      actions={
        <>
          {print.buttons}
          {newButton}
        </>
      }
      filters={filters}
    >
      {body}
    </ListPage>
  );
}
