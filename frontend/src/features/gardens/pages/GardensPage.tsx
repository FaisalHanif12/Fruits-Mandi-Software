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
import { useMemo, useState } from 'react';
import { uiErrorOf } from '../../../shared/api';
import { PagedList } from '../../../shared/grid';
import { SHORTCUTS, formNavigationProps, useShortcut } from '../../../shared/keyboard';
import { Button, Card, DateInput, EmptyState, Field, Select, TextInput } from '../../../shared/ui';
import { useGardensList } from '../hooks';
import { GARDEN_COLUMNS } from '../list-columns';
import type { GardenRow, VentureStatus } from '../types';

type StatusChoice = VentureStatus | 'all';
const STATUSES: readonly StatusChoice[] = ['all', 'open', 'closed'];

/** Text or nothing: an empty filter is sent as no limit. */
function textOrNull(text: string): string | null {
  const trimmed = text.trim();
  return trimmed === '' ? null : trimmed;
}

export function GardensPage() {
  const navigate = useNavigate();
  const [from, setFrom] = useState('');
  const [to, setTo] = useState('');
  const [status, setStatus] = useState<StatusChoice>('all');
  const [search, setSearch] = useState('');
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
  const list = useGardensList(filter);
  const first = list.data?.pages[0];
  // A refusal shows as the state, not as the rows of the filter before it.
  const rows = list.isError ? undefined : list.data?.pages.flatMap((page) => page.rows);
  const newGarden = () => {
    void navigate({ to: '/gardens/new' });
  };
  useShortcut(SHORTCUTS.newEntry, newGarden);
  const noFilter =
    filter.from === null && filter.to === null && filter.status === null && filter.search === null;

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
            setFrom('');
            setTo('');
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
    <div className="flex max-w-page flex-col gap-4">
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div className="flex flex-col gap-1">
          <Localized id="gardens-title">
            <h1 className="sr-only normal:not-sr-only normal:type-title" />
          </Localized>
          <Localized id="gardens-help">
            <p className="hidden type-body text-ink-muted normal:block" />
          </Localized>
        </div>
        <Button variant="primary" shortcut={SHORTCUTS.newEntry} onClick={newGarden}>
          <Plus aria-hidden="true" className="size-icon-md" />
          <Localized id="gardens-new" />
        </Button>
      </div>
      <Card compact>
        <div
          {...formNavigationProps}
          className="grid grid-cols-2 items-end gap-4 normal:grid-cols-4"
        >
          <Field label={<Localized id="gardens-list-from" />}>
            {(control) => <DateInput {...control} value={from} onChange={setFrom} />}
          </Field>
          <Field label={<Localized id="gardens-list-to" />}>
            {(control) => <DateInput {...control} value={to} onChange={setTo} />}
          </Field>
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
          <Field label={<Localized id="gardens-list-search" />}>
            {(control) => <TextInput {...control} value={search} onChange={setSearch} />}
          </Field>
        </div>
        <PagedList<GardenRow>
          columns={GARDEN_COLUMNS}
          rows={rows}
          totalCount={first?.total_count}
          getRowId={(row) => String(row.id)}
          labelId="gardens-list-label"
          failedId="gardens-list-failed"
          error={list.isError ? uiErrorOf(list.error) : null}
          onRetry={() => {
            void list.refetch();
          }}
          hasMore={list.hasNextPage}
          loadingMore={list.isFetchingNextPage}
          onLoadMore={() => {
            void list.fetchNextPage();
          }}
          empty={empty}
          onOpen={(row) => {
            void navigate({ to: '/gardens/$id', params: { id: String(row.id) } });
          }}
          isInactive={(row) => row.status === 'closed'}
          totals={
            first === undefined
              ? undefined
              : (context) => ({
                  labelField: 'name',
                  values: {
                    name: context.text('glossary-meezan'),
                    balance: context.money(first.totals.balance_paisa),
                    commission: context.money(first.totals.commission_paisa),
                    rent_due: context.money(first.totals.rent_due_paisa),
                    rent: context.money(first.totals.rent_paisa),
                    paid: context.money(first.totals.paid_paisa),
                    money_in: context.money(first.totals.money_in_paisa),
                  },
                })
          }
        />
      </Card>
    </div>
  );
}
