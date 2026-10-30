// What a view shows under its filters (PRD 18.2, FR-18.2, FR-18.8, FR-19.11): the days it covers, the
// chart, the table page by page with its totals row from the backend, and the four states: loading, an
// error with a retry, nothing found (with a way out), and the data. Enter (or a double click) on a
// figure opens the lines behind it; Enter on the totals row opens the lines behind that total. The
// totals are the backend's, over the whole filter; the rows on screen are never added up (CLAUDE.md
// rule 1, Code Standards F34).
//
// A table never scrolls sideways (NFR-USE-15). The columns shown are the ones that fit the window
// (columns.ts); views D and E, with one column for each period or item, show a window of them, with
// two buttons to move it.

import { Localized } from '@fluent/react';
import { ChevronLeft, ChevronRight, SearchX } from 'lucide-react';
import { useMemo } from 'react';
import type { AnalyticsKey } from '../../../bindings/AnalyticsKey';
import type { AnalyticsPage } from '../../../bindings/AnalyticsPage';
import { uiErrorOf } from '../../../shared/api';
import { ListGrid, ListRegion, Pager, listHeightMode, useListPaging } from '../../../shared/grid';
import { useRememberedFlag } from '../../../shared/hooks';
import { useLayoutStep } from '../../../shared/hooks/useLayoutStep';
import { Button, EmptyState, ErrorState, LoadingState, RUNNING_NUMBER } from '../../../shared/ui';
import { cellText, totalsFor } from '../../reports';
import {
  analyticsColumns,
  headingText,
  planColumns,
  signedKeysOf,
  type AnalyticsRowView,
} from '../columns';
import type { useAnalyticsPages } from '../hooks';
import { useListContext } from '../list-context';
import type { AnalyticsView } from '../types';
import { AnalyticsChart } from './AnalyticsChart';
import { AnalyticsRange } from './AnalyticsRange';

/** The key of the whole filter: the lines behind a total (the backend reads an empty key so). */
/**
 * A window this high or lower starts with the chart folded: the table, its totals and its page
 * buttons are what the user came for, and the chart is one click away (plan 38). 1366 x 768 is
 * included; 1920 x 1080 shows the chart.
 */
const CHART_FOLD_HEIGHT_PX = 800;

/** True in a window where the chart starts folded (read where no local name hides `window`). */
function isShortWindow(): boolean {
  // The same rule as a style: the height of the window, not of the page.
  return (
    typeof window.matchMedia === 'function' &&
    window.matchMedia(`(max-height: ${String(CHART_FOLD_HEIGHT_PX)}px)`).matches
  );
}

export const WHOLE_FILTER_KEY: AnalyticsKey = {
  item_id: null,
  seller_id: null,
  buyer_id: null,
  person_key: null,
  from: null,
  to: null,
  not_item_ids: [],
  partner_id: null,
  purchase_ids: [],
};

interface AnalyticsResultsProps {
  view: AnalyticsView;
  query: ReturnType<typeof useAnalyticsPages>;
  /** The page last shown, kept while a half-typed filter is refused (see AnalyticsBody). */
  kept: { pages: AnalyticsPage[] } | null;
  /** A filter that the view takes is filled: the empty state offers to clear it. */
  filtering: boolean;
  onClear: () => void;
  /** The first of the headed columns shown (views D and E), kept above the screen. */
  windowStart: number;
  onWindow: (start: number) => void;
  /** The user asked for the columns that do not fit this window by default (columns.ts). */
  otherColumns: boolean;
  onOtherColumns: (other: boolean) => void;
  /**
   * Enter on a figure: the row, the column that has the figure (null for a plain column) and what
   * the figure is, in words, for the title of the lines behind it.
   */
  onDrill: (row: AnalyticsKey, column: AnalyticsKey | null, about: string) => void;
  focusRowId: string | null;
  onFocusRow: (rowId: string) => void;
}

export function AnalyticsResults({
  view,
  query,
  kept,
  filtering,
  onClear,
  windowStart,
  onWindow,
  otherColumns,
  onOtherColumns,
  onDrill,
  focusRowId,
  onFocusRow,
}: AnalyticsResultsProps) {
  const step = useLayoutStep();
  // The chart starts folded in a short window, where it would push the table out of view; the
  // choice is kept for the screen.
  const [chartOpen, setChartOpen] = useRememberedFlag('analytics.chart_open', !isShortWindow());
  const context = useListContext();
  const paging = useListPaging(query.pager);
  const data = query.data ?? kept ?? undefined;
  const pages = data?.pages;
  const last = pages?.at(-1);
  const rows = useMemo<AnalyticsRowView[] | undefined>(
    () =>
      pages?.flatMap((page) =>
        page.rows.map((row, index) => ({ key: String(page.offset + index), row })),
      ),
    [pages],
  );
  const plan = useMemo(
    () =>
      last === undefined ? null : planColumns(view, last.columns, step, windowStart, otherColumns),
    [view, last, step, windowStart, otherColumns],
  );
  const columns = useMemo(
    () => (last === undefined || plan === null ? [] : analyticsColumns(view, last.columns, plan)),
    [view, last, plan],
  );

  if (last === undefined || rows === undefined || plan === null) {
    if (query.isError) {
      return (
        <ErrorState
          title={<Localized id="analytics-load-failed" />}
          error={uiErrorOf(query.error)}
          onRetry={() => {
            void query.refetch();
          }}
        />
      );
    }
    return <LoadingState />;
  }

  const drillOf = (row: AnalyticsRowView, columnId: string | undefined) => {
    const key = row.row.key;
    if (key === null) return;
    const column = last.columns.find((candidate) => candidate.key === columnId);
    const first = row.row.cells[0];
    const rowText = first === undefined ? '' : cellText(first, context);
    const columnText = column?.heading == null ? '' : headingText(column.heading, context);
    onDrill(
      key,
      column?.drill ?? null,
      [rowText, columnText].filter((part) => part !== '').join(' · '),
    );
  };

  const window = plan.window;
  let body;
  if (rows.length === 0) {
    body = (
      <EmptyState
        icon={SearchX}
        title={<Localized id="analytics-empty" />}
        help={<Localized id="analytics-empty-help" />}
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
        {window === null ? null : (
          <div className="flex flex-wrap items-center justify-between gap-2">
            <Localized
              id={
                last.columns.some((column) => column.heading?.kind === 'item')
                  ? 'analytics-columns-shown-items'
                  : 'analytics-columns-shown'
              }
              vars={{
                from: String(window.start + 1),
                to: String(window.start + window.count),
                total: String(window.total),
              }}
              elems={RUNNING_NUMBER}
            >
              <p className="type-body text-ink-muted" />
            </Localized>
            <div className="flex gap-2">
              <Button
                disabled={window.start === 0}
                onClick={() => {
                  onWindow(Math.max(0, window.start - window.count));
                }}
              >
                <ChevronLeft aria-hidden="true" className="size-icon-md rtl:rotate-180" />
                <Localized id="analytics-columns-earlier" />
              </Button>
              <Button
                disabled={window.start + window.count >= window.total}
                onClick={() => {
                  onWindow(window.start + window.count);
                }}
              >
                <Localized id="analytics-columns-later" />
                <ChevronRight aria-hidden="true" className="size-icon-md rtl:rotate-180" />
              </Button>
            </div>
          </div>
        )}
        {plan.trimmed === null ? null : (
          <div className="flex flex-wrap items-center justify-between gap-2">
            <Localized
              id="analytics-columns-fit"
              vars={{ shown: String(plan.trimmed.shown), total: String(plan.trimmed.total) }}
              elems={RUNNING_NUMBER}
            >
              <p className="type-body text-ink-muted" />
            </Localized>
            <Button
              onClick={() => {
                onOtherColumns(!otherColumns);
              }}
            >
              <Localized id={otherColumns ? 'analytics-columns-main' : 'analytics-columns-other'} />
            </Button>
          </div>
        )}
        <ListRegion rowCount={rows.length}>
          <ListGrid<AnalyticsRowView>
            columns={columns}
            rows={rows}
            getRowId={(item) => item.key}
            labelId="analytics-grid-label"
            onOpen={drillOf}
            onOpenTotals={(columnId) => {
              const column = last.columns.find((candidate) => candidate.key === columnId);
              const columnText =
                column?.heading == null ? '' : headingText(column.heading, context);
              onDrill(
                WHOLE_FILTER_KEY,
                column?.drill ?? null,
                [context.text('analytics-drill-all'), columnText]
                  .filter((part) => part !== '')
                  .join(' · '),
              );
            }}
            onFocusRow={onFocusRow}
            focusRowId={focusRowId}
            heightMode={listHeightMode(rows.length)}
            paging={paging}
            sort={{ value: query.sort, onChange: query.setSort }}
            isGroup={(item) => item.row.kind === 'group'}
            totals={(ctx) =>
              totalsFor(
                last.columns,
                { labelId: last.totals_label_id, cells: last.totals },
                ctx,
                signedKeysOf(view, last.columns),
              )
            }
          />
        </ListRegion>
        <div className="shrink-0">
          <Pager pager={query.pager} />
        </div>
      </>
    );
  }

  return (
    <div className="flex min-h-0 flex-1 flex-col gap-2" aria-busy={query.isFetching}>
      {/* The days of the view, and the choice to show or fold the chart (kept for the screen). */}
      <div className="flex shrink-0 flex-wrap items-center justify-between gap-2">
        <AnalyticsRange filters={last.filters} />
        {rows.length === 0 || last.charts.length === 0 ? null : (
          <Button
            onClick={() => {
              setChartOpen(!chartOpen);
            }}
          >
            <Localized id={chartOpen ? 'analytics-chart-hide' : 'analytics-chart-show'} />
          </Button>
        )}
      </div>
      {/* The charts cover the whole filter, whatever page or order the table has (plan 38 R4). */}
      {rows.length === 0 || !chartOpen ? null : <AnalyticsChart view={view} page={last} />}
      {body}
    </div>
  );
}
