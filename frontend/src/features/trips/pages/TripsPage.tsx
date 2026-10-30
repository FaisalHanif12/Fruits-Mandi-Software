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
import { useCallback, useEffect, useMemo, useState } from 'react';
import { useDayStatus } from '../../../app/use-day-status';
import { uiErrorOf } from '../../../shared/api';
import { PagedList, type ListContext } from '../../../shared/grid';
import { usePeriod } from '../../../shared/hooks';
import { SHORTCUTS, formNavigationProps, useShortcut } from '../../../shared/keyboard';
import { Remembered, type MemoryProps } from '../../../shared/screen-state';
import {
  Button,
  EmptyState,
  Field,
  FilterBar,
  ListPage,
  PeriodBox,
  Select,
} from '../../../shared/ui';
import { useTripsList } from '../hooks';
import { TRIP_COLUMNS } from '../list-columns';
import { TRIPS_SCREEN, tripChoicesFrom, tripChoicesText } from '../remember';
import type { TripRow, VentureStatus } from '../types';

/** The month of a trip's day, `YYYY-MM`: the list is in date order. */
const TRIP_MONTH_OF = (row: TripRow) => row.entry_date.slice(0, 7);

type StatusChoice = VentureStatus | 'all';
const STATUSES: readonly StatusChoice[] = ['all', 'open', 'closed'];

/** Text or nothing: an empty filter is sent as no limit. */
function textOrNull(text: string): string | null {
  const trimmed = text.trim();
  return trimmed === '' ? null : trimmed;
}

/** The trips list starts as the Owner left its filters (PRD FR-7.45). */
export function TripsPage() {
  return <Remembered screen={TRIPS_SCREEN}>{(memory) => <TripsBody {...memory} />}</Remembered>;
}

function TripsBody({ saved, remember }: MemoryProps) {
  const navigate = useNavigate();
  const start = useMemo(() => tripChoicesFrom(saved), [saved]);
  const day = useDayStatus();
  // The trip list opens on every trip; the Owner may narrow it by days (remembered by the period box).
  const period = usePeriod('trips.list', 'all', day.data?.today);
  const [status, setStatus] = useState<StatusChoice>(start.status);
  const [search, setSearch] = useState(start.search);
  useEffect(() => {
    remember(tripChoicesText({ from: '', to: '', status, search }));
  }, [status, search, remember]);
  const filter = useMemo(
    () => ({
      from: textOrNull(period.range.from),
      to: textOrNull(period.range.to),
      status: status === 'all' ? null : status,
      partner_id: null,
      search: textOrNull(search),
      group_by_month: true,
    }),
    [period.range.from, period.range.to, status, search],
  );
  const list = useTripsList(filter, period.ready);
  // A refusal shows as the state, not as the rows of the filter before it.
  const rows = useMemo(
    () => (list.isError ? undefined : list.data?.pages.flatMap((page) => page.rows)),
    [list.data, list.isError],
  );
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
              commission: context.money(first.totals.commission_paisa),
              cost: context.money(first.totals.cost_paisa),
              sales: context.money(first.totals.sales_paisa),
            }),
      },
    }),
    [first],
  );
  const noFilter =
    filter.from === null && filter.to === null && filter.status === null && filter.search === null;

  const empty = noFilter ? (
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
            period.change({ kind: 'all', from: '', to: '' });
            setStatus('all');
            setSearch('');
          }}
        >
          <Localized id="trips-list-clear" />
        </Button>
      }
    />
  );

  return (
    <ListPage
      titleId="trips-title"
      help={
        <Localized id="trips-help">
          <p />
        </Localized>
      }
      actions={
        <Button variant="primary" shortcut={SHORTCUTS.newEntry} onClick={newTrip}>
          <Plus aria-hidden="true" className="size-icon-md" />
          <Localized id="trips-new" />
        </Button>
      }
      filters={
        <div {...formNavigationProps}>
          <FilterBar
            screen="trips.list"
            search={{ value: search, onChange: setSearch, labelId: 'trips-list-search' }}
          >
            {period.ready ? <PeriodBox period={period} /> : null}
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
          </FilterBar>
        </div>
      }
    >
      <PagedList<TripRow>
        columns={TRIP_COLUMNS}
        rows={rows}
        getRowId={(row) => String(row.id)}
        labelId="trips-list-label"
        failedId="trips-list-failed"
        error={list.isError ? uiErrorOf(list.error) : null}
        onRetry={() => {
          void list.refetch();
        }}
        pager={list.pager}
        sort={{ value: list.sort, onChange: list.setSort }}
        months={
          first === undefined
            ? undefined
            : { months: first.months, monthOf: TRIP_MONTH_OF, columnOf: undefined }
        }
        empty={empty}
        onOpen={(row) => {
          void navigate({ to: '/trips/$id', params: { id: String(row.id) } });
        }}
        isInactive={(row) => row.status === 'closed'}
        totals={totals}
      />
    </ListPage>
  );
}
