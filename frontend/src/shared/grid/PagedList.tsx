// A list of any module with its four states (Code Standards F8, F34, PRD FR-19.11, FR-7.46):
// loading, an error with a retry, nothing yet (the screen's own helpful message), and the rows. The
// rows come one page at a time with the page buttons under them (`Pager`, plan 38 R1), and only the
// rows in view are drawn. The totals under the columns and the count are the backend's over the
// whole filter, never a sum of the rows on screen. Page Up and Page Down in the table turn the
// page. The rows are read only here: Enter on a row does nothing unless the screen gives `onOpen`.

import { Localized } from '@fluent/react';
import { useState, type ReactNode } from 'react';
import type { UiError } from '../../bindings/UiError';
import type { PagerState } from '../hooks';
import { ErrorState, LoadingState } from '../ui';
import {
  ListGrid,
  type ListColumn,
  type ListContext,
  type ListMonths,
  type ListSort,
} from './ListGrid';
import { ListRegion, listHeightMode } from './ListRegion';
import { Pager } from './Pager';
import { useListPaging } from './useListPaging';

interface PagedListProps<T> {
  columns: readonly ListColumn<T>[];
  /** The rows of the page on screen, or undefined while the first page is on its way. */
  rows: readonly T[] | undefined;
  getRowId: (row: T) => string;
  /** The Fluent message ID of the list's name, read by screen readers. */
  labelId: string;
  /** The Fluent message ID of "the list could not be opened". */
  failedId: string;
  /** Why the first page could not load, or null. */
  error: UiError | null;
  onRetry: () => void;
  /** Where the list is: the page on screen, the rows per page and the count of the whole filter. */
  pager: PagerState;
  /**
   * The order of the list, from `usePagedQuery` (`{ value: list.sort, onChange: list.setSort }`).
   * With it, the title of every column that has a `sortKey` is a button (plan 38 R4).
   */
  sort?: ListSort | undefined;
  /**
   * A heading row above each month, with the backend's count and subtotals of the month over the
   * whole filter (plan 38 R4). Only for a list in date order: ask the backend for `months` only
   * when no other sort is chosen.
   */
  months?: ListMonths<T> | undefined;
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
  getRowId,
  labelId,
  failedId,
  error,
  onRetry,
  pager,
  sort,
  months,
  empty,
  isInactive,
  onOpen,
  onFocusRow,
  focusRowId,
  totals,
}: PagedListProps<T>) {
  // The row that has focus, kept here so a language switch (which rebuilds the grid) keeps it.
  const [ownFocusId, setOwnFocusId] = useState<string | null>(null);
  const paging = useListPaging(pager);
  const focusId = focusRowId === undefined ? ownFocusId : focusRowId;
  if (rows === undefined) {
    return error === null ? (
      <LoadingState />
    ) : (
      <ErrorState title={<Localized id={failedId} />} error={error} onRetry={onRetry} />
    );
  }
  if (rows.length === 0) return <>{empty}</>;
  // The work area of a list (plan 38 G2): the grid takes the height the screen leaves, with its
  // header and its totals row fixed and the rows scrolling between them, and the page buttons stay
  // under it, always in view. On a screen that is not given a fixed height (a list under a form)
  // the grid keeps a floor of a few rows and the page scrolls as before.
  return (
    <div className="flex min-h-0 flex-1 flex-col gap-2">
      <ListRegion rowCount={rows.length}>
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
          heightMode={listHeightMode(rows.length)}
          isInactive={isInactive}
          totals={totals}
          paging={paging}
          sort={sort}
          months={months}
        />
      </ListRegion>
      <div className="shrink-0">
        <Pager pager={pager} />
      </div>
    </div>
  );
}
