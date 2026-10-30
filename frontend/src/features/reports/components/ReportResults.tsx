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
import { ReportRange } from './ReportRange';
import { ReportSummary } from './ReportSummary';

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
}

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
  // The party day sheet (R35) is laid out like the client's own slip: its lines first, and what the
  // day comes to underneath, right under them (PRD FR-19.12). Every other report has its figures
  // above the table.
  const sheet = last.id === 'R35';
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
    body = (
      <>
        <ListRegion rowCount={rows.length}>
          <ListGrid<ReportRowView>
            columns={columns}
            rows={rows}
            getRowId={(view) => view.key}
            labelId="report-grid-label"
            onOpen={onOpen}
            onFocusRow={onFocusRow}
            focusRowId={focusRowId}
            heightMode={listHeightMode(rows.length)}
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
        {/* The slip's footer sits right under its lines, like the client's own (PRD FR-19.12). */}
        {sheet ? <ReportSummary lines={last.summary} card /> : null}
        <div className="shrink-0">
          <Pager pager={query.pager} />
        </div>
      </>
    );
  }

  // The days of the report and its summary lines are one strip under the filters (plan 38).
  const showsRange = takes.includes('as_of') || takes.includes('date_range');
  const showsStrip = showsRange || (!sheet && last.summary.length > 0);
  return (
    <div className="flex min-h-0 flex-1 flex-col gap-2" aria-busy={query.isFetching}>
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
