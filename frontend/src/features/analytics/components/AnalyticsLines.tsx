// The lines behind a number (PRD FR-18.2): opened with Enter on a figure of a view or on its total.
// Each line is one sale line of the posting that makes up the figure, with the بکری it belongs to; a
// reversal line shows negative amounts, so the lines add up to the number (the totals row says so).
// Enter on a line opens its بکری. Esc (or the back button) returns to the view with its filters, its
// window of columns and its focused row as they were. Four states: loading, an error with a retry,
// nothing found, and the lines. Lists load 100 lines at a time with the backend's totals (F34).

import { Localized } from '@fluent/react';
import { useNavigate } from '@tanstack/react-router';
import { ArrowLeft, SearchX } from 'lucide-react';
import { useMemo, useState } from 'react';
import { uiErrorOf } from '../../../shared/api';
import { ListGrid } from '../../../shared/grid';
import { useLayoutStep } from '../../../shared/hooks/useLayoutStep';
import { Button, EmptyState, ErrorState, LoadingState } from '../../../shared/ui';
import { totalsFor, type ReportRowView } from '../../reports';
import { linesColumns } from '../columns';
import { useAnalyticsLines, type LinesRequest } from '../hooks';

const SHORT_LIST_ROWS = 30;

interface AnalyticsLinesProps {
  request: LinesRequest;
  /** What the number is, in words (for example the item and the period), under the title. */
  about: string;
  onBack: () => void;
}

export function AnalyticsLines({ request, about, onBack }: AnalyticsLinesProps) {
  const navigate = useNavigate();
  const step = useLayoutStep();
  const query = useAnalyticsLines(request);
  const [focusRowId, setFocusRowId] = useState<string | null>(null);
  const pages = query.data?.pages;
  const last = pages?.at(-1);
  const rows = useMemo<ReportRowView[] | undefined>(
    () =>
      pages?.flatMap((page) =>
        page.rows.map((row, index) => ({ key: String(page.offset + index), row })),
      ),
    [pages],
  );
  const columns = useMemo(
    () => (last === undefined ? [] : linesColumns(last.columns, step)),
    [last, step],
  );

  let body;
  if (last === undefined || rows === undefined) {
    body = query.isError ? (
      <ErrorState
        title={<Localized id="analytics-lines-failed" />}
        error={uiErrorOf(query.error)}
        onRetry={() => {
          void query.refetch();
        }}
      />
    ) : (
      <LoadingState />
    );
  } else if (rows.length === 0) {
    body = (
      <EmptyState
        icon={SearchX}
        title={<Localized id="analytics-lines-empty" />}
        help={<Localized id="analytics-lines-empty-help" />}
      />
    );
  } else {
    const long = rows.length > SHORT_LIST_ROWS;
    body = (
      <>
        <div className={long ? 'h-list-region' : undefined}>
          <ListGrid<ReportRowView>
            columns={columns}
            rows={rows}
            getRowId={(view) => view.key}
            labelId="analytics-lines-grid-label"
            onOpen={(view) => {
              const source = view.row.source;
              if (source?.kind === 'bikri') {
                void navigate({ to: '/bikri/book', search: { id: source.id } });
              }
            }}
            onFocusRow={setFocusRowId}
            focusRowId={focusRowId}
            heightMode={long ? 'fill' : 'auto'}
            totals={(ctx) =>
              totalsFor(last.columns, { labelId: last.totals_label_id, cells: last.totals }, ctx)
            }
          />
        </div>
        <div className="flex flex-wrap items-center justify-between gap-2">
          <Localized
            id="report-count"
            vars={{ shown: String(rows.length), total: String(last.total_count) }}
          >
            <p className="type-body text-ink-muted" />
          </Localized>
          {query.hasNextPage ? (
            <Button
              disabled={query.isFetchingNextPage}
              busy={query.isFetchingNextPage}
              onClick={() => {
                void query.fetchNextPage();
              }}
            >
              <Localized id="report-more" />
            </Button>
          ) : null}
        </div>
        <Localized id="analytics-lines-hint">
          <p className="type-body text-ink-muted" />
        </Localized>
      </>
    );
  }

  return (
    <div className="flex flex-col gap-4" data-analytics-lines>
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div className="flex flex-col gap-1">
          <Localized id="analytics-lines-title">
            <h2 className="type-section" />
          </Localized>
          {about === '' ? null : <p className="type-body text-ink-muted">{about}</p>}
        </div>
        <Button onClick={onBack}>
          <ArrowLeft aria-hidden="true" className="size-icon-md rtl:rotate-180" />
          <Localized id="analytics-lines-back" />
        </Button>
      </div>
      {body}
    </div>
  );
}
