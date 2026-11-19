// What a report shows under its filters (PRD 19.1, FR-19.5, FR-19.11): the lines above the table,
// the table one page at a time with its totals row from the backend, and the four states: loading, an
// error with a retry, nothing found (with a way out), and the rows. Enter (or a double click) on a
// row opens the record behind it (FR-19.4). The totals are the backend's, over the whole filter;
// the rows on screen are never added up (CLAUDE.md rule 1, Code Standards F34).

import { Localized } from '@fluent/react';
import { SearchX } from 'lucide-react';
import { useMemo } from 'react';
import type { ReportFilterKind } from '../../../bindings/ReportFilterKind';
import type { ReportPage } from '../../../bindings/ReportPage';
import { uiErrorOf } from '../../../shared/api';
import { ListGrid, ListRegion, Pager, listHeightMode, useListPaging } from '../../../shared/grid';
import { useLayoutStep } from '../../../shared/hooks/useLayoutStep';
import { Button, EmptyState, ErrorState, LoadingState, Notice } from '../../../shared/ui';
import { reportColumns, signedColumnsOf, totalsFor, type ReportRowView } from '../columns';
import type { useReportPages } from '../hooks';
import { DayTables } from './DayTables';
import { DeadHeading } from './DeadHeading';
import { ReportRange } from './ReportRange';
import { ReportSummary } from './ReportSummary';
import { SlipSummary, isSlipPage } from './SlipSummary';

/** The reports with a running balance: it shows in date order only (plan 38 R4). */
const BALANCE_REPORTS: ReadonlySet<string> = new Set(['R12', 'R13', 'R14']);

interface ReportResultsProps {
  /** The filter boxes the report takes, so the line under them knows what to say. */
  takes: readonly ReportFilterKind[];
  query: ReturnType<typeof useReportPages>;
  /** The page last shown, kept while a half-typed filter is refused (see ReportBody). */
  kept: { pages: ReportPage[] } | null;
  /** A filter that the report takes is filled: the empty state offers to clear it. */
  filtering: boolean;
  /** The detail box is off for a report that has one (R14): balances only. */
  balanceOnly: boolean;
  onClear: () => void;
  /** Enter on a row. */
  onOpen: (view: ReportRowView) => void;
  focusRowId: string | null;
  onFocusRow: (rowId: string) => void;
  /**
   * From which layout step a party's slip puts its goods and its lines side by side (the default:
   * the normal step). A screen with a side menu of its own (the party ledgers, plan 66) leaves less
   * room at the normal step than the report screen does, and asks for the wide step, so the amount
   * column of the goods is never cut off.
   */
  slipBesideFrom?: 'normal' | 'wide' | undefined;
  /**
   * The page buttons under the rows are left out while the whole answer is one page: "1 to 1 of 1" means
   * nothing inside a day opened under its row in a party's statement (plan 74).
   */
  hideSinglePagePager?: boolean | undefined;
}

/** The classes that lay a slip's goods and lines side by side from a step; below it they stack. */
const SLIP_BESIDE = {
  normal: { row: 'normal:flex-row normal:items-start', goods: 'normal:flex-1' },
  wide: { row: 'wide:flex-row wide:items-start', goods: 'wide:flex-1' },
} as const;

export function ReportResults({
  takes,
  query,
  kept,
  filtering,
  balanceOnly,
  onClear,
  onOpen,
  focusRowId,
  onFocusRow,
  slipBesideFrom = 'normal',
  hideSinglePagePager = false,
}: ReportResultsProps) {
  const data = query.data ?? kept ?? undefined;
  const pages = data?.pages;
  const last = pages?.at(-1);
  const rows = useMemo<ReportRowView[] | undefined>(
    () =>
      pages?.flatMap((page) => [
        // A page that starts in the middle of a group has the group's heading above its first row,
        // with the totals of the WHOLE group (plan 38 R4).
        ...(page.group_above === null
          ? []
          : [{ key: `above-${String(page.offset)}`, row: page.group_above }]),
        ...page.rows.map((row, index) => ({ key: String(page.offset + index), row })),
      ]),
    [pages],
  );
  const step = useLayoutStep();
  const paging = useListPaging(query.pager);
  const columns = useMemo(
    () => (last === undefined ? [] : reportColumns(last.id, last.columns, step)),
    [last, step],
  );

  if (last === undefined || rows === undefined) {
    if (query.isError) {
      return (
        <ErrorState
          title={<Localized id="report-load-failed" />}
          error={uiErrorOf(query.error)}
          onRetry={() => {
            void query.refetch();
          }}
        />
      );
    }
    return <LoadingState />;
  }

  // The running balance of a ledger or a day book needs the date order: sorted by another column the
  // backend leaves the balance cells empty and the screen says why (plan 38 R4).
  const dateKey = last.columns.find((column) => column.kind === 'date')?.key;
  const balanceNote =
    BALANCE_REPORTS.has(last.id) &&
    query.sort !== null &&
    dateKey !== undefined &&
    query.sort.column !== dateKey;
  // The party day sheet (R35) is laid out as a slip: its lines first, and what the day comes to
  // underneath, right under them (PRD FR-19.12). Every other report has its figures above the table.
  // One party's slip (a buyer's or a seller's, plan 42) has its labelled lines as a card; the table of
  // every party has its totals as a strip of facts.
  const sheet = last.id === 'R35';
  const slip = isSlipPage(last);
  let body;
  if (rows.length === 0) {
    body = balanceOnly ? (
      <Notice tone="info">
        <Localized id="report-balance-only-help" />
      </Notice>
    ) : (
      <EmptyState
        icon={SearchX}
        title={<Localized id="report-empty" />}
        help={<Localized id="report-empty-help" />}
        action={
          filtering ? (
            <Button onClick={onClear}>
              <Localized id="report-clear-filters" />
            </Button>
          ) : undefined
        }
      />
    );
  } else {
    // A slip (one party's day) and the day total sheet (R34) have more under their rows than a table:
    // the labelled lines of the slip, the short buyers' and sellers' tables of each day. They are one
    // page of their own, so the rows are drawn in full and the page scrolls as a whole (plan 42).
    const extras = slip || last.tables.length > 0;
    const grid = (
      // The rows of a page with extras do not fill the height: the extras come under (or beside) them.
      <ListRegion rowCount={extras ? 0 : rows.length}>
        <ListGrid<ReportRowView>
          columns={columns}
          rows={rows}
          getRowId={(view) => view.key}
          labelId="report-grid-label"
          onOpen={onOpen}
          onFocusRow={onFocusRow}
          focusRowId={focusRowId}
          heightMode={extras ? 'auto' : listHeightMode(rows.length)}
          paging={paging}
          sort={{ value: query.sort, onChange: query.setSort }}
          // A group heading and a month heading read stronger and open nothing.
          isGroup={(view) => view.row.kind !== 'normal'}
          totals={(context) =>
            totalsFor(
              last.columns,
              { labelId: last.totals_label_id, cells: last.totals },
              context,
              signedColumnsOf(last.id),
            )
          }
        />
      </ListRegion>
    );
    body = (
      <>
        {extras ? (
          <div
            className={`flex min-h-0 flex-1 flex-col gap-3 overflow-auto ${slip ? SLIP_BESIDE[slipBesideFrom].row : ''}`}
          >
            <div className={`min-w-0 ${SLIP_BESIDE[slipBesideFrom].goods}`}>{grid}</div>
            {/* The short buyers' and sellers' tables of each day of the day total sheet (R34). */}
            {last.tables.length > 0 ? <DayTables tables={last.tables} /> : null}
            {/* The slip's lines sit with its goods, like the paper slip (PRD FR-19.12). */}
            {slip ? <SlipSummary lines={last.summary} /> : null}
          </div>
        ) : (
          <>
            {grid}
            {sheet ? <ReportSummary lines={last.summary} card /> : null}
          </>
        )}
        {hideSinglePagePager && query.pager.pageCount <= 1 ? null : (
          <div className="shrink-0">
            <Pager pager={query.pager} />
          </div>
        )}
      </>
    );
  }

  // The days of the report and its summary lines are one strip under the filters (plan 38).
  const showsRange = takes.includes('as_of') || takes.includes('date_range');
  const showsStrip = showsRange || (!sheet && last.summary.length > 0);
  return (
    <div className="flex min-h-0 flex-1 flex-col gap-2" aria-busy={query.isFetching}>
      <DeadHeading party={last.filters.party} />
      {showsStrip ? (
        <div className="flex shrink-0 flex-wrap items-baseline gap-x-6 gap-y-1 rounded-3 border border-border bg-surface px-3 py-2">
          <ReportRange filters={last.filters} takes={takes} day={sheet} />
          {sheet ? null : <ReportSummary lines={last.summary} />}
        </div>
      ) : null}
      {balanceNote ? (
        <Notice tone="info">
          <Localized id="report-balance-date-order-only" />
        </Notice>
      ) : null}
      {body}
    </div>
  );
}
