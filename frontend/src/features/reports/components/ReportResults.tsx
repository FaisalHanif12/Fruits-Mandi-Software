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
import { ListGrid, Pager, useListPaging } from '../../../shared/grid';
import { useLayoutStep } from '../../../shared/hooks/useLayoutStep';
import { Button, EmptyState, ErrorState, LoadingState, Notice } from '../../../shared/ui';
import { reportColumns, signedColumnsOf, totalsFor, type ReportRowView } from '../columns';
import type { useReportPages } from '../hooks';
import { ReportRange } from './ReportRange';
import { ReportSummary } from './ReportSummary';

/** A list with more rows than this scrolls inside its region and draws only the rows in view (F34). */
const SHORT_LIST_ROWS = 30;

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
      pages?.flatMap((page) =>
        page.rows.map((row, index) => ({ key: String(page.offset + index), row })),
      ),
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

  const long = rows.length > SHORT_LIST_ROWS;
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
        <div className={long ? 'h-list-region' : undefined}>
          <ListGrid<ReportRowView>
            columns={columns}
            rows={rows}
            getRowId={(view) => view.key}
            labelId="report-grid-label"
            onOpen={onOpen}
            onFocusRow={onFocusRow}
            focusRowId={focusRowId}
            heightMode={long ? 'fill' : 'auto'}
            paging={paging}
            isGroup={(view) => view.row.kind === 'group'}
            totals={(context) =>
              totalsFor(
                last.columns,
                { labelId: last.totals_label_id, cells: last.totals },
                context,
                signedColumnsOf(last.id),
              )
            }
          />
        </div>
        {/* The slip's footer sits right under its lines, like the client's own (PRD FR-19.12). */}
        {sheet ? <ReportSummary lines={last.summary} /> : null}
        <Pager pager={query.pager} />
        <Localized id="report-open-hint">
          <p className="type-body text-ink-muted" />
        </Localized>
      </>
    );
  }

  return (
    <div className="flex flex-col gap-4" aria-busy={query.isFetching}>
      <ReportRange filters={last.filters} takes={takes} day={sheet} />
      {sheet ? null : <ReportSummary lines={last.summary} />}
      {body}
    </div>
  );
}
