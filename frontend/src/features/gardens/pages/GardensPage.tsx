// باغات (route /gardens; PRD 12.11, FR-5.2, FR-12.37, FR-19.10, FR-19.11): the list of gardens,
// newest first, with a button for a new garden. The list loads page by page; the totals under it
// are the backend's over the whole filter, never a sum of the rows on screen. Enter (or a double
// click) on a row opens the garden. A closed garden stays in the list, marked with the word
// "closed" and the day. The filters are the days, open or closed, and a search over the name.
//
// Four states: loading, an error with a retry, nothing yet (a helpful message with a button), and
// the rows. A filter with no match says so and offers to clear it.

import { Localized } from '@fluent/react';
import { useNavigate } from '@tanstack/react-router';
import { Plus, SearchX, Trees } from 'lucide-react';
import { useCallback, useMemo, useState } from 'react';
import { useDayStatus } from '../../../app/use-day-status';
import { uiErrorOf } from '../../../shared/api';
import { PagedList, type ListContext } from '../../../shared/grid';
import { usePeriod } from '../../../shared/hooks';
import { SHORTCUTS, formNavigationProps, useShortcut } from '../../../shared/keyboard';
import {
  Button,
  EmptyState,
  Field,
  FilterBar,
  ListPage,
  PeriodBox,
  Select,
} from '../../../shared/ui';
import { useGardensList } from '../hooks';
import { GARDEN_COLUMNS } from '../list-columns';
import type { GardenRow, VentureStatus } from '../types';

/** The month of a garden's day, `YYYY-MM`: the list is in date order. */
const GARDEN_MONTH_OF = (row: GardenRow) => row.entry_date.slice(0, 7);
/** The backend names a month's subtotal like the field of the totals; the column has a short name. */
const GARDEN_MONTH_COLUMNS = {
  balance_paisa: 'balance',
  commission_paisa: 'commission',
  rent_due_paisa: 'rent_due',
  rent_paisa: 'rent',
  paid_paisa: 'paid',
  money_in_paisa: 'money_in',
} as const;

type StatusChoice = VentureStatus | 'all';
const STATUSES: readonly StatusChoice[] = ['all', 'open', 'closed'];

/** Text or nothing: an empty filter is sent as no limit. */
function textOrNull(text: string): string | null {
  const trimmed = text.trim();
  return trimmed === '' ? null : trimmed;
}

export function GardensPage() {
  const navigate = useNavigate();
  const day = useDayStatus();
  // Gardens last a season, so the list opens on every garden; the Owner may narrow it by days.
  const period = usePeriod('gardens.list', 'all', day.data?.today);
  const [status, setStatus] = useState<StatusChoice>('all');
  const [search, setSearch] = useState('');
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
  const list = useGardensList(filter, period.ready);
  const first = list.data?.pages[0];
  // A refusal shows as the state, not as the rows of the filter before it.
  const rows = list.isError ? undefined : list.data?.pages.flatMap((page) => page.rows);
  const newGarden = () => {
    void navigate({ to: '/gardens/new' });
  };
  useShortcut(SHORTCUTS.newEntry, newGarden);
  const noFilter =
    filter.from === null && filter.to === null && filter.status === null && filter.search === null;
  // The totals under the columns are the backend's, over the whole filter. The callback depends on
  // the loaded page, not on an amount: a Paisa value must not go into a dependency list (F3).
  const totals = useCallback(
    (context: ListContext) => ({
      labelField: 'name',
      values:
        first === undefined
          ? {}
          : {
              name: context.text('glossary-meezan'),
              balance: context.money(first.totals.balance_paisa),
              commission: context.money(first.totals.commission_paisa),
              rent_due: context.money(first.totals.rent_due_paisa),
              rent: context.money(first.totals.rent_paisa),
              paid: context.money(first.totals.paid_paisa),
              money_in: context.money(first.totals.money_in_paisa),
            },
    }),
    [first],
  );

  const empty = noFilter ? (
    <EmptyState
      icon={Trees}
      title={<Localized id="gardens-list-empty" />}
      help={<Localized id="gardens-list-empty-help" />}
      action={
        <Button variant="primary" shortcut={SHORTCUTS.newEntry} onClick={newGarden}>
          <Localized id="gardens-new" />
        </Button>
      }
    />
  ) : (
    <EmptyState
      icon={SearchX}
      title={<Localized id="gardens-list-no-match" />}
      help={<Localized id="gardens-list-no-match-help" />}
      action={
        <Button
          onClick={() => {
            period.change({ kind: 'all', from: '', to: '' });
            setStatus('all');
            setSearch('');
          }}
        >
          <Localized id="gardens-list-clear" />
        </Button>
      }
    />
  );

  return (
    <ListPage
      titleId="gardens-title"
      help={
        <Localized id="gardens-help">
          <p />
        </Localized>
      }
      actions={
        <Button variant="primary" shortcut={SHORTCUTS.newEntry} onClick={newGarden}>
          <Plus aria-hidden="true" className="size-icon-md" />
          <Localized id="gardens-new" />
        </Button>
      }
      filters={
        <div {...formNavigationProps}>
          <FilterBar
            screen="gardens.list"
            search={{ value: search, onChange: setSearch, labelId: 'gardens-list-search' }}
          >
            {period.ready ? <PeriodBox period={period} /> : null}
            <Field label={<Localized id="gardens-list-status" />}>
              {(control) => (
                <Select
                  {...control}
                  value={status}
                  onChange={setStatus}
                  options={STATUSES.map((choice) => ({
                    value: choice,
                    label: <Localized id={`gardens-state-${choice}`} />,
                  }))}
                />
              )}
            </Field>
          </FilterBar>
        </div>
      }
    >
      <PagedList<GardenRow>
        columns={GARDEN_COLUMNS}
        rows={rows}
        getRowId={(row) => String(row.id)}
        labelId="gardens-list-label"
        failedId="gardens-list-failed"
        error={list.isError ? uiErrorOf(list.error) : null}
        onRetry={() => {
          void list.refetch();
        }}
        pager={list.pager}
        sort={{ value: list.sort, onChange: list.setSort }}
        months={
          first === undefined
            ? undefined
            : { months: first.months, monthOf: GARDEN_MONTH_OF, columnOf: GARDEN_MONTH_COLUMNS }
        }
        empty={empty}
        onOpen={(row) => {
          void navigate({ to: '/gardens/$id', params: { id: String(row.id) } });
        }}
        isInactive={(row) => row.status === 'closed'}
        totals={totals}
      />
    </ListPage>
  );
}
