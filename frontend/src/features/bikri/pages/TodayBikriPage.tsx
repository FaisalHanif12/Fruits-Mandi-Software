// آج کی بکری (route /bikri/today; PRD FR-12.7): the list of بکری with the columns number, time,
// truck, sellers, items, نگ, خام, صافی, buyer total and status. The day starts as today; the
// filters are day, seller, item and status. ↑ ↓ Page Up and Page Down move between rows, Enter (or a
// click) opens the بکری read-only, Alt+B starts a new one. Four states: loading, error with a
// retry, empty with a button (PRD FR-7.46), and the rows. All amounts are the stored sums.

import { Localized } from '@fluent/react';
import { useNavigate } from '@tanstack/react-router';
import { FileText, Plus, SearchX } from 'lucide-react';
import { useMemo, useState } from 'react';
import { useDayStatus } from '../../../app/use-day-status';
import type { BikriListInput } from '../../../bindings/BikriListInput';
import type { BikriListRow } from '../../../bindings/BikriListRow';
import type { BikriStatus } from '../../../bindings/BikriStatus';
import { uiErrorOf } from '../../../shared/api';
import { isoToDisplayDate } from '../../../shared/format';
import { PagedList } from '../../../shared/grid';
import { useFocusFirstField, useLayoutStep, useRememberedFlag } from '../../../shared/hooks';
import { SHORTCUTS, formNavigationProps, useEscape } from '../../../shared/keyboard';
import {
  AccountPicker,
  Button,
  Checkbox,
  DateInput,
  EmptyState,
  Field,
  Picker,
  Select,
  type PickerValue,
} from '../../../shared/ui';
import { useBikriList } from '../hooks/useBikriReads';
import { bikriListColumns } from '../list-columns';

const ALL = 'all';
type StatusChoice = BikriStatus | typeof ALL;

export function TodayBikriPage() {
  const navigate = useNavigate();
  const day = useDayStatus();
  // The day field is empty until today is known, then starts as today; the user may change it.
  const [dayText, setDayText] = useState<string | null>(null);
  const today = day.data === undefined ? '' : isoToDisplayDate(day.data.today);
  const date = dayText ?? today;
  const [seller, setSeller] = useState<PickerValue | null>(null);
  const [item, setItem] = useState<PickerValue | null>(null);
  const [status, setStatus] = useState<StatusChoice>(ALL);
  const filter: Omit<BikriListInput, 'offset' | 'limit'> = {
    date: date.trim() === '' ? null : date.trim(),
    from: null,
    to: null,
    seller_id: seller?.id ?? null,
    item_id: item?.id ?? null,
    status: status === ALL ? null : status,
  };
  const list = useBikriList(filter);
  // The tick that adds the نگ column, which does not fit beside the others (FR-12.7). It is kept.
  const [moreColumns, setMoreColumns] = useRememberedFlag('bikri.list_more_columns', false);
  const step = useLayoutStep();
  const columns = useMemo(() => bikriListColumns(step, moreColumns), [step, moreColumns]);
  const containerRef = useFocusFirstField<HTMLDivElement>();
  const onlyToday = filter.date === today && seller === null && item === null && status === ALL;

  const openNew = () => {
    void navigate({ to: '/bikri/book', search: {} });
  };
  useEscape('leave', () => {
    void navigate({ to: '/' });
  });

  const newButton = (
    <Button variant="primary" shortcut={SHORTCUTS.newBikri} onClick={openNew}>
      <Plus aria-hidden="true" className="size-icon-md" />
      <Localized id="bikri-new" />
    </Button>
  );
  const first = list.data?.pages[0];
  // A refusal shows as the state, not as the rows of the filter before it.
  const rows = list.isError ? undefined : list.data?.pages.flatMap((page) => page.rows);
  const empty = onlyToday ? (
    <EmptyState
      icon={FileText}
      title={<Localized id="bikri-list-empty" />}
      help={<Localized id="bikri-list-empty-help" vars={{ key: SHORTCUTS.newBikri.label }} />}
      action={newButton}
    />
  ) : (
    <EmptyState
      icon={SearchX}
      title={<Localized id="bikri-list-no-match" />}
      help={<Localized id="bikri-list-no-match-help" />}
      action={
        <Button
          onClick={() => {
            setDayText(today);
            setSeller(null);
            setItem(null);
            setStatus(ALL);
          }}
        >
          <Localized id="bikri-list-clear" />
        </Button>
      }
    />
  );
  // The four states and the paging are the shared list's; the totals row is the backend's, over the
  // whole filter and not over the rows loaded (PRD FR-19.11).
  const body = (
    <PagedList<BikriListRow>
      columns={columns}
      rows={rows}
      totalCount={first?.total_count}
      getRowId={(row) => String(row.id)}
      labelId="bikri-list-label"
      failedId="bikri-list-failed"
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
        void navigate({ to: '/bikri/book', search: { id: row.id } });
      }}
      isInactive={(row) => row.status === 'cancelled'}
      totals={
        first === undefined
          ? undefined
          : (context) => ({
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
            })
      }
    />
  );

  return (
    <div className="flex h-full flex-col gap-4">
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div className="flex flex-col gap-1">
          <Localized id="layout-module-today">
            <h1 className="type-title" />
          </Localized>
          <Localized id="bikri-list-help">
            <p className="type-body text-ink-muted" />
          </Localized>
        </div>
        <div className="flex flex-wrap items-center gap-4">
          <Checkbox
            checked={moreColumns}
            onChange={setMoreColumns}
            label={<Localized id="common-more-columns" />}
          />
          {rows === undefined || rows.length === 0 ? null : newButton}
        </div>
      </div>
      <div
        ref={containerRef}
        {...formNavigationProps}
        className="grid grid-cols-2 items-end gap-4 normal:grid-cols-4"
      >
        <Field label={<Localized id="bikri-header-date" />}>
          {(control) => <DateInput {...control} value={date} onChange={setDayText} />}
        </Field>
        <Field label={<Localized id="glossary-beopari" />}>
          {(control) => (
            <AccountPicker
              {...control}
              scope="seller"
              showCard={false}
              value={seller}
              onChange={setSeller}
            />
          )}
        </Field>
        <Field label={<Localized id="glossary-qism-maal" />}>
          {(control) => (
            <Picker
              {...control}
              scope="item"
              value={item}
              onChange={(hit) => {
                setItem(hit);
              }}
            />
          )}
        </Field>
        <Field label={<Localized id="bikri-list-col-status" />}>
          {(control) => (
            <Select
              {...control}
              value={status}
              onChange={setStatus}
              options={[
                { value: ALL, label: <Localized id="bikri-list-status-all" /> },
                { value: 'saved', label: <Localized id="bikri-status-saved" /> },
                { value: 'cancelled', label: <Localized id="bikri-status-cancelled" /> },
              ]}
            />
          )}
        </Field>
      </div>
      {body}
    </div>
  );
}
