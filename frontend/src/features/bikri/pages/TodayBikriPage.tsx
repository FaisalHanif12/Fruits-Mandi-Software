// آج کی بکری (route /bikri/today; PRD FR-12.7): the list of بکری with the columns number, time,
// truck, sellers, items, نگ, خام, صافی, buyer total and status. The day starts as today; the
// filters are day, seller, item and status. ↑ ↓ Page Up and Page Down move between rows, Enter (or a
// click) opens the بکری read-only, Alt+B starts a new one. Four states: loading, error with a
// retry, empty with a button (PRD FR-7.46), and the rows. All amounts are the stored sums.

import { Localized } from '@fluent/react';
import { useNavigate } from '@tanstack/react-router';
import { FileText, Plus, SearchX } from 'lucide-react';
import { useEffect, useMemo, useState } from 'react';
import { useDayStatus } from '../../../app/use-day-status';
import type { BikriListInput } from '../../../bindings/BikriListInput';
import type { BikriListRow } from '../../../bindings/BikriListRow';
import type { BikriStatus } from '../../../bindings/BikriStatus';
import { uiErrorOf } from '../../../shared/api';
import { isoToDisplayDate } from '../../../shared/format';
import { ListGrid } from '../../../shared/grid';
import { useFocusFirstField, useLayoutStep, useRememberedFlag } from '../../../shared/hooks';
import { SHORTCUTS, formNavigationProps, useEscape } from '../../../shared/keyboard';
import { Remembered, type MemoryProps } from '../../../shared/screen-state';
import {
  AccountPicker,
  Button,
  Checkbox,
  DateInput,
  EmptyState,
  ErrorState,
  Field,
  LoadingState,
  Picker,
  Select,
  type PickerValue,
} from '../../../shared/ui';
import { useBikriList } from '../hooks/useBikriReads';
import { bikriListColumns } from '../list-columns';
import { TODAY_SCREEN, todayChoicesFrom, todayChoicesText } from '../remember';

const ALL = 'all';
type StatusChoice = BikriStatus | typeof ALL;

/** A list with more rows than this fills the room and scrolls inside; shorter ones are as tall as their rows. */
const SHORT_LIST_ROWS = 30;

/** Today's بکری start as the Owner left the filters (PRD FR-7.45); the day always starts as today. */
export function TodayBikriPage() {
  return (
    <Remembered screen={TODAY_SCREEN}>{(memory) => <TodayBikriBody {...memory} />}</Remembered>
  );
}

function TodayBikriBody({ saved, remember }: MemoryProps) {
  const navigate = useNavigate();
  const start = useMemo(() => todayChoicesFrom(saved), [saved]);
  const day = useDayStatus();
  // The day field is empty until today is known, then starts as today; the user may change it.
  const [dayText, setDayText] = useState<string | null>(null);
  const today = day.data === undefined ? '' : isoToDisplayDate(day.data.today);
  const date = dayText ?? today;
  const [seller, setSeller] = useState<PickerValue | null>(start.seller);
  const [item, setItem] = useState<PickerValue | null>(start.item);
  const [status, setStatus] = useState<StatusChoice>(start.status);
  useEffect(() => {
    remember(todayChoicesText({ seller, item, status }));
  }, [seller, item, status, remember]);
  const filter: BikriListInput = {
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
  const rows = list.data;

  let body;
  if (rows === undefined && list.isPending) {
    body = <LoadingState />;
  } else if (list.isError) {
    body = (
      <ErrorState
        title={<Localized id="bikri-list-failed" />}
        error={uiErrorOf(list.error)}
        onRetry={() => {
          void list.refetch();
        }}
      />
    );
  } else if (rows?.length === 0) {
    body = onlyToday ? (
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
  } else if (rows !== undefined) {
    body = (
      <ListGrid<BikriListRow>
        columns={columns}
        rows={rows}
        getRowId={(row) => String(row.id)}
        labelId="bikri-list-label"
        onOpen={(row) => {
          void navigate({ to: '/bikri/book', search: { id: row.id } });
        }}
        heightMode={rows.length <= SHORT_LIST_ROWS ? 'auto' : 'fill'}
        isInactive={(row) => row.status === 'cancelled'}
      />
    );
  }

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
