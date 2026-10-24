// A list of any module with its four states (Code Standards F8, F34, PRD FR-19.11, FR-7.46):
// loading, an error with a retry, nothing yet (the screen's own helpful message), and the rows. The
// rows load page by page and only the rows in view are drawn. The totals under the columns and the
// count are the backend's over the whole filter, never a sum of the rows on screen. The rows are
// read only here: Enter on a row does nothing unless the screen gives `onOpen`.

import { Localized } from '@fluent/react';
import { useState, type ReactNode } from 'react';
import type { UiError } from '../../bindings/UiError';
import { ErrorState, LoadingState } from '../ui';
import { ListGrid, type ListColumn, type ListContext } from './ListGrid';
import { PagedFooter } from './PagedFooter';

/** A list with more rows than this scrolls inside its region and draws only the rows in view. */
const SHORT_LIST_ROWS = 30;

interface PagedListProps<T> {
  columns: readonly ListColumn<T>[];
  /** The rows loaded so far, or undefined while the first page is on its way. */
  rows: readonly T[] | undefined;
  /** How many rows the whole filter has (the backend's count). */
  totalCount: number | undefined;
  getRowId: (row: T) => string;
  /** The Fluent message ID of the list's name, read by screen readers. */
  labelId: string;
  /** The Fluent message ID of "the list could not be opened". */
  failedId: string;
  /** Why the first page could not load, or null. */
  error: UiError | null;
  onRetry: () => void;
  hasMore: boolean;
  loadingMore: boolean;
  onLoadMore: () => void;
  /** Shown when the filter has no rows: why, and what to do next. */
  empty: ReactNode;
  isInactive?: ((row: T) => boolean) | undefined;
  /** Enter or a click on a row, for a list whose rows open something. Default: nothing opens. */
  onOpen?: ((row: T) => void) | undefined;
  /**
   * For a screen that acts on the row that has focus (edit, cancel, print): it is told which row
   * has focus, and keeps it, so a language switch (which rebuilds the grid) keeps it too.
   */
  onFocusRow?: ((rowId: string) => void) | undefined;
  focusRowId?: string | null | undefined;
  /** The backend's total row under the columns. */
  totals?:
    ((context: ListContext) => { labelField: string; values: Record<string, string> }) | undefined;
}

export function PagedList<T>({
  columns,
  rows,
  totalCount,
  getRowId,
  labelId,
  failedId,
  error,
  onRetry,
  hasMore,
  loadingMore,
  onLoadMore,
  empty,
  isInactive,
  onOpen,
  onFocusRow,
  focusRowId,
  totals,
}: PagedListProps<T>) {
  // The row that has focus, kept here so a language switch (which rebuilds the grid) keeps it.
  const [ownFocusId, setOwnFocusId] = useState<string | null>(null);
  const focusId = focusRowId === undefined ? ownFocusId : focusRowId;
  if (rows === undefined) {
    return error === null ? (
      <LoadingState />
    ) : (
      <ErrorState title={<Localized id={failedId} />} error={error} onRetry={onRetry} />
    );
  }
  if (rows.length === 0) return <>{empty}</>;
  const long = rows.length > SHORT_LIST_ROWS;
  return (
    <>
      <div className={long ? 'h-list-region' : undefined}>
        <ListGrid<T>
          columns={columns}
          rows={rows}
          getRowId={getRowId}
          labelId={labelId}
          onOpen={onOpen ?? (() => undefined)}
          onFocusRow={(rowId) => {
            setOwnFocusId(rowId);
            onFocusRow?.(rowId);
          }}
          focusRowId={focusId}
          heightMode={long ? 'fill' : 'auto'}
          isInactive={isInactive}
          totals={totals}
        />
      </div>
      <PagedFooter
        shown={rows.length}
        total={totalCount ?? rows.length}
        hasMore={hasMore}
        loadingMore={loadingMore}
        onLoadMore={onLoadMore}
      />
    </>
  );
}
