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
import { ListGrid, ListRegion, Pager, listHeightMode, useListPaging } from '../../../shared/grid';
import { Button, EmptyState, ErrorState, LoadingState } from '../../../shared/ui';
import { AUDIT_COLUMNS } from '../columns';
import type { useAuditPages } from '../hooks';

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
  return (
    <div className="flex min-h-0 flex-1 flex-col gap-2" aria-busy={query.isFetching}>
      <ListRegion rowCount={rows.length}>
        <ListGrid<AuditRow>
          columns={AUDIT_COLUMNS}
          rows={rows}
          getRowId={(row) => String(row.id)}
          labelId="audit-grid-label"
          onOpen={onOpen}
          onFocusRow={onFocusRow}
          focusRowId={focusRowId}
          heightMode={listHeightMode(rows.length)}
          paging={paging}
          sort={{ value: query.sort, onChange: query.setSort }}
          // A heading above each month, with how many entries it has: the log is in date order.
          months={{ months: last.months, monthOf: (row) => row.date.slice(0, 7) }}
        />
      </ListRegion>
      <div className="shrink-0">
        <Pager pager={query.pager} />
      </div>
    </div>
  );
}
