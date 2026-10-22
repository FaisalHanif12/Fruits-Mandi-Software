// مارکیٹ سفر (route /trips; PRD 12.10, FR-5.2, FR-12.25, FR-19.10, FR-19.11): the list of trips,
// newest first, with a button for a new trip. The list loads page by page; the totals under it
// are the backend's over the whole filter, never a sum of the rows on screen. Enter (or a double
// click) on a row opens the trip. The filters are the days, open or closed, and a search over the
// name, the market and the person sent.
//
// Four states: loading, an error with a retry, nothing yet (a helpful message with a button), and
// the rows. A filter with no match says so and offers to clear it.

import { Localized } from '@fluent/react';
import { useNavigate } from '@tanstack/react-router';
import { Plus, Route, SearchX } from 'lucide-react';
import { useCallback, useMemo, useState } from 'react';
import { uiErrorOf } from '../../../shared/api';
import { ListGrid, type ListContext } from '../../../shared/grid';
import { SHORTCUTS, formNavigationProps, useShortcut } from '../../../shared/keyboard';
import {
  Button,
  Card,
  DateInput,
  EmptyState,
  ErrorState,
  Field,
  LoadingState,
  Select,
  TextInput,
} from '../../../shared/ui';
import { useTripsList } from '../hooks';
import { TRIP_COLUMNS } from '../list-columns';
import type { TripRow, VentureStatus } from '../types';

/** A list with more rows than this scrolls inside its region and draws only the rows in view (F34). */
const SHORT_LIST_ROWS = 30;

type StatusChoice = VentureStatus | 'all';
const STATUSES: readonly StatusChoice[] = ['all', 'open', 'closed'];

/** Text or nothing: an empty filter is sent as no limit. */
function textOrNull(text: string): string | null {
  const trimmed = text.trim();
  return trimmed === '' ? null : trimmed;
}

export function TripsPage() {
  const navigate = useNavigate();
  const [from, setFrom] = useState('');
  const [to, setTo] = useState('');
  const [status, setStatus] = useState<StatusChoice>('all');
  const [search, setSearch] = useState('');
  const [focusId, setFocusId] = useState<string | null>(null);
  const filter = useMemo(
    () => ({
      from: textOrNull(from),
      to: textOrNull(to),
      status: status === 'all' ? null : status,
      partner_id: null,
      search: textOrNull(search),
    }),
    [from, to, status, search],
  );
  const list = useTripsList(filter);
  const rows = useMemo(() => list.data?.pages.flatMap((page) => page.rows), [list.data]);
  const first = list.data?.pages[0];
  const newTrip = () => {
    void navigate({ to: '/trips/new' });
  };
  useShortcut(SHORTCUTS.newEntry, newTrip);

  // The totals under the columns are the backend's figures for the whole filter. The callback
  // depends on the loaded page, not on an amount: a Paisa value must not go into a dependency
  // list (F3).
  const totals = useCallback(
    (context: ListContext) => ({
      labelField: 'market',
      values: {
        market: context.text('glossary-meezan'),
        ...(first === undefined
          ? {}
          : {
              qty_left: String(first.totals.qty_left),
              cost_left: context.money(first.totals.cost_left_paisa),
              result: context.money(first.totals.result_paisa),
              cost: context.money(first.totals.cost_paisa),
              sales: context.money(first.totals.sales_paisa),
            }),
      },
    }),
    [first],
  );
  const noFilter =
    filter.from === null && filter.to === null && filter.status === null && filter.search === null;

  let body;
  if (rows === undefined && list.isPending) {
    body = <LoadingState />;
  } else if (list.isError && rows === undefined) {
    body = (
      <ErrorState
        title={<Localized id="trips-list-failed" />}
        error={uiErrorOf(list.error)}
        onRetry={() => {
          void list.refetch();
        }}
      />
    );
  } else if (rows?.length === 0) {
    body = noFilter ? (
      <EmptyState
        icon={Route}
        title={<Localized id="trips-list-empty" />}
        help={<Localized id="trips-list-empty-help" />}
        action={
          <Button variant="primary" shortcut={SHORTCUTS.newEntry} onClick={newTrip}>
            <Localized id="trips-new" />
          </Button>
        }
      />
    ) : (
      <EmptyState
        icon={SearchX}
        title={<Localized id="trips-list-no-match" />}
        help={<Localized id="trips-list-no-match-help" />}
        action={
          <Button
            onClick={() => {
              setFrom('');
              setTo('');
              setStatus('all');
              setSearch('');
            }}
          >
            <Localized id="trips-list-clear" />
          </Button>
        }
      />
    );
  } else if (rows !== undefined) {
    const long = rows.length > SHORT_LIST_ROWS;
    body = (
      <>
        <div className={long ? 'h-list-region' : undefined}>
          <ListGrid<TripRow>
            columns={TRIP_COLUMNS}
            rows={rows}
            getRowId={(row) => String(row.id)}
            labelId="trips-list-label"
            onOpen={(row) => {
              void navigate({ to: '/trips/$id', params: { id: String(row.id) } });
            }}
            onFocusRow={setFocusId}
            focusRowId={focusId}
            heightMode={long ? 'fill' : 'auto'}
            isInactive={(row) => row.status === 'closed'}
            totals={totals}
          />
        </div>
        <div className="flex flex-wrap items-center justify-between gap-2">
          <Localized
            id="trips-list-count"
            vars={{ shown: String(rows.length), total: String(first?.total_count ?? rows.length) }}
          >
            <p className="type-body text-ink-muted" />
          </Localized>
          {list.hasNextPage ? (
            <Button
              disabled={list.isFetchingNextPage}
              busy={list.isFetchingNextPage}
              onClick={() => {
                void list.fetchNextPage();
              }}
            >
              <Localized id="trips-list-more" />
            </Button>
          ) : null}
        </div>
      </>
    );
  }

  return (
    <div className="flex max-w-page flex-col gap-4">
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div className="flex flex-col gap-1">
          <Localized id="trips-title">
            <h1 className="sr-only normal:not-sr-only normal:type-title" />
          </Localized>
          <Localized id="trips-help">
            <p className="hidden type-body text-ink-muted normal:block" />
          </Localized>
        </div>
        <Button variant="primary" shortcut={SHORTCUTS.newEntry} onClick={newTrip}>
          <Plus aria-hidden="true" className="size-icon-md" />
          <Localized id="trips-new" />
        </Button>
      </div>
      <Card compact>
        <div
          {...formNavigationProps}
          className="grid grid-cols-2 items-end gap-4 normal:grid-cols-4"
        >
          <Field label={<Localized id="trips-list-from" />}>
            {(control) => <DateInput {...control} value={from} onChange={setFrom} />}
          </Field>
          <Field label={<Localized id="trips-list-to" />}>
            {(control) => <DateInput {...control} value={to} onChange={setTo} />}
          </Field>
          <Field label={<Localized id="trips-list-status" />}>
            {(control) => (
              <Select
                {...control}
                value={status}
                onChange={setStatus}
                options={STATUSES.map((choice) => ({
                  value: choice,
                  label: <Localized id={`trips-state-${choice}`} />,
                }))}
              />
            )}
          </Field>
          <Field label={<Localized id="trips-list-search" />}>
            {(control) => <TextInput {...control} value={search} onChange={setSearch} />}
          </Field>
        </div>
        {body}
      </Card>
    </div>
  );
}
