// One list of پچھلی بکری, today's or the window's (PRD 12.7, plan 72): the shared paged list with the
// columns of the بکری list, the backend's totals row over the whole filter (FR-19.11), a click or
// Enter that opens the whole بکری, and the four states. The window gives day headings through
// `groups`; today gives none. No figure is worked out here.

import { useCallback, useMemo, type ReactNode } from 'react';
import type { BikriListRow } from '../../../bindings/BikriListRow';
import { uiErrorOf } from '../../../shared/api';
import {
  PagedList,
  type ListColumn,
  type ListContext,
  type ListGroups,
} from '../../../shared/grid';
import type { PagerState } from '../../../shared/hooks';
import type { UiError } from '../../../bindings/UiError';
import type { BikriList as PastListAnswer } from '../../../bindings/BikriList';
import type { SortInput } from '../../../bindings/SortInput';

/** What a list hook of `useBikriReads` gives, as far as this component reads it. */
export interface PastListQuery {
  data: { pages: PastListAnswer[] } | undefined;
  pager: PagerState;
  sort: SortInput | null;
  setSort: (next: SortInput | null) => void;
  isError: boolean;
  error: unknown;
  refetch: () => Promise<unknown>;
}

interface PastListProps {
  query: PastListQuery;
  columns: readonly ListColumn<BikriListRow>[];
  labelId: string;
  failedId: string;
  /** Shown when the filter has no rows. */
  empty: ReactNode;
  onOpen: (row: BikriListRow) => void;
  /** The row that has focus, kept by the screen so a language switch or a way back keeps it. */
  focusRowId: string | null;
  onFocusRow: (rowId: string) => void;
  /** Day headings among the rows (the window only). */
  groups?: ListGroups<BikriListRow> | undefined;
  /** Page Up and Page Down do this instead of turning the page (the window only). */
  onPageKey?: ((direction: 'previous' | 'next') => void) | undefined;
}

/** The rows on screen, or undefined while the first page is on its way or could not be read. */
export function rowsOf(data: PastListQuery['data'], isError: boolean): BikriListRow[] | undefined {
  // A refusal shows as the state, not as the rows of the filter before it.
  return isError ? undefined : data?.pages.flatMap((page) => page.rows);
}

export function PastList({
  query,
  columns,
  labelId,
  failedId,
  empty,
  onOpen,
  focusRowId,
  onFocusRow,
  groups,
  onPageKey,
}: PastListProps) {
  const first = query.data?.pages[0];
  // The rows are made once for the page on screen, so the grid keeps them between renders.
  const { data, isError } = query;
  const rows = useMemo(() => rowsOf(data, isError), [data, isError]);
  // The callback depends on the loaded page, not on an amount, and is made once per page: a new one
  // on every render would rebuild the grid's columns under a click on a title (F3).
  const totals = useCallback(
    (context: ListContext) =>
      first === undefined
        ? { labelField: 'sellers', values: {} }
        : {
            labelField: 'sellers',
            values: {
              sellers: context.text('glossary-meezan'),
              qty: context.text('bikri-list-qty', {
                sold: String(first.totals.qty_sold),
                received: String(first.totals.qty_received),
              }),
              gross: context.money(first.totals.gross),
              net: context.money(first.totals.net),
              buyer_total: context.money(first.totals.buyer_total),
            },
          },
    [first],
  );
  const error: UiError | null = query.isError ? uiErrorOf(query.error) : null;
  return (
    <PagedList<BikriListRow>
      columns={columns}
      rows={rows}
      getRowId={(row) => String(row.id)}
      labelId={labelId}
      failedId={failedId}
      error={error}
      onRetry={() => {
        void query.refetch();
      }}
      pager={query.pager}
      sort={{ value: query.sort, onChange: query.setSort }}
      empty={empty}
      onOpen={onOpen}
      onFocusRow={onFocusRow}
      focusRowId={focusRowId}
      isInactive={(row) => row.status === 'cancelled'}
      totals={totals}
      groups={groups}
      onPageKey={onPageKey}
    />
  );
}
