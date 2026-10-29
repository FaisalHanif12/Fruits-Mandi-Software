// The list of audit entries under the filters (PRD FR-21.3, FR-19.11): one page at a time with the
// backend's count of the whole filter, never a sum of the rows on screen (Code Standards F34). Enter (or a double
// click) on an entry opens its changed fields. Four states: loading, an error with a retry, nothing
// found (with a way out), and the data. The log is read only: nothing here edits or deletes.

import { Localized } from '@fluent/react';
import { SearchX } from 'lucide-react';
import { useMemo } from 'react';
import type { AuditPage } from '../../../bindings/AuditPage';
import type { AuditRow } from '../../../bindings/AuditRow';
import { uiErrorOf } from '../../../shared/api';
import { ListGrid, Pager, useListPaging } from '../../../shared/grid';
import { Button, EmptyState, ErrorState, LoadingState } from '../../../shared/ui';
import { AUDIT_COLUMNS } from '../columns';
import type { useAuditPages } from '../hooks';

/** A list with more rows than this scrolls inside its region and draws only the rows in view (F34). */
const SHORT_LIST_ROWS = 30;

interface AuditResultsProps {
  query: ReturnType<typeof useAuditPages>;
  /** The page last shown, kept while a half-typed filter is refused (it stays under the box). */
  kept: { pages: AuditPage[] } | null;
  /** A box is filled: the empty state offers to clear it. */
  filtering: boolean;
  onClear: () => void;
  onOpen: (row: AuditRow) => void;
  focusRowId: string | null;
  onFocusRow: (rowId: string) => void;
}

export function AuditResults({
  query,
  kept,
  filtering,
  onClear,
  onOpen,
  focusRowId,
  onFocusRow,
}: AuditResultsProps) {
  const data = query.data ?? kept ?? undefined;
  const last = data?.pages.at(-1);
  const rows = useMemo<AuditRow[] | undefined>(
    () => data?.pages.flatMap((page) => page.rows),
    [data],
  );
  const paging = useListPaging(query.pager);

  if (last === undefined || rows === undefined) {
    if (query.isError) {
      return (
        <ErrorState
          title={<Localized id="audit-load-failed" />}
          error={uiErrorOf(query.error)}
          onRetry={() => {
            void query.refetch();
          }}
        />
      );
    }
    return <LoadingState />;
  }
  if (rows.length === 0) {
    return (
      <EmptyState
        icon={SearchX}
        title={<Localized id="audit-empty" />}
        help={<Localized id="audit-empty-help" />}
        action={
          filtering ? (
            <Button onClick={onClear}>
              <Localized id="report-clear-filters" />
            </Button>
          ) : undefined
        }
      />
    );
  }
  const long = rows.length > SHORT_LIST_ROWS;
  return (
    <div className="flex flex-col gap-3" aria-busy={query.isFetching}>
      <div className={long ? 'h-list-region' : undefined}>
        <ListGrid<AuditRow>
          columns={AUDIT_COLUMNS}
          rows={rows}
          getRowId={(row) => String(row.id)}
          labelId="audit-grid-label"
          onOpen={onOpen}
          onFocusRow={onFocusRow}
          focusRowId={focusRowId}
          heightMode={long ? 'fill' : 'auto'}
          paging={paging}
        />
      </div>
      <Pager pager={query.pager} />
      <Localized id="audit-open-hint">
        <p className="type-body text-ink-muted" />
      </Localized>
    </div>
  );
}
