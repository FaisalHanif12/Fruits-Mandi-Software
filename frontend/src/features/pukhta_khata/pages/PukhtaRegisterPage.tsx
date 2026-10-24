// پختہ کھاتہ register (route /bikri/pukhta; PRD FR-13.6, R9): every line of a one-time buyer and
// every lot of a one-time seller, with date, بکری number, side, name, phone, item, نگ, amount,
// commission, and paid or due (the open amount comes from the matching rows). Filters: the two
// days (empty means no limit), side, name, phone, paid or due. The name and the phone are searched
// as typed, in either script, and never converted (PRD FR-7.35, F41). ↑ ↓ and Enter open the row's
// بکری read-only. A cancelled line is marked in words. Four states: loading, error with a retry,
// empty with a way out, and the rows.

import { Localized } from '@fluent/react';
import { useNavigate } from '@tanstack/react-router';
import { SearchX, UsersRound } from 'lucide-react';
import { useMemo, useState } from 'react';
import type { PaidState } from '../../../bindings/PaidState';
import type { PukhtaRegisterInput } from '../../../bindings/PukhtaRegisterInput';
import type { PukhtaRegisterRow } from '../../../bindings/PukhtaRegisterRow';
import type { PukhtaSide } from '../../../bindings/PukhtaSide';
import { uiErrorOf } from '../../../shared/api';
import { PagedList } from '../../../shared/grid';
import { useFocusFirstField, useLayoutStep, useRememberedFlag } from '../../../shared/hooks';
import { formNavigationProps, useEscape } from '../../../shared/keyboard';
import {
  Button,
  Checkbox,
  DateInput,
  EmptyState,
  Field,
  Select,
  TextInput,
} from '../../../shared/ui';
import { pukhtaColumns } from '../columns';
import { usePukhtaRegister } from '../hooks';

const ALL = 'all';

/** A register row has no id of its own (two equal lines are possible), so its place is its key. */
type KeyedRow = PukhtaRegisterRow & { key: string };

export function PukhtaRegisterPage() {
  const navigate = useNavigate();
  const [from, setFrom] = useState('');
  const [to, setTo] = useState('');
  const [side, setSide] = useState<PukhtaSide | typeof ALL>(ALL);
  const [name, setName] = useState('');
  const [phone, setPhone] = useState('');
  const [state, setState] = useState<PaidState | typeof ALL>(ALL);
  const filter: Omit<PukhtaRegisterInput, 'offset' | 'limit'> = {
    from: from.trim(),
    to: to.trim(),
    side: side === ALL ? null : side,
    name: name.trim() === '' ? null : name.trim(),
    phone: phone.trim() === '' ? null : phone.trim(),
    state: state === ALL ? null : state,
  };
  const filtering =
    filter.from !== '' ||
    filter.to !== '' ||
    filter.side !== null ||
    filter.name !== null ||
    filter.phone !== null ||
    filter.state !== null;
  const list = usePukhtaRegister(filter);
  // The tick that adds the نگ column, which does not fit beside the others (FR-13.6). It is kept.
  const [moreColumns, setMoreColumns] = useRememberedFlag('pukhta.more_columns', false);
  const step = useLayoutStep();
  const columns = useMemo(() => pukhtaColumns(step, moreColumns), [step, moreColumns]);
  const containerRef = useFocusFirstField<HTMLDivElement>();
  useEscape('leave', () => {
    void navigate({ to: '/' });
  });
  const clear = () => {
    setFrom('');
    setTo('');
    setSide(ALL);
    setName('');
    setPhone('');
    setState(ALL);
  };

  const first = list.data?.pages[0];
  // A refusal shows as the state, not as the rows of the filter before it. A register row has no id
  // of its own, so its place in the whole list is its key; the next page continues the count.
  const rows: KeyedRow[] | undefined = list.isError
    ? undefined
    : list.data?.pages
        .flatMap((page) => page.rows)
        .map((row, index) => ({ ...row, key: String(index) }));
  // The sums are over the whole filter and only mean something for one side: with no side chosen the
  // backend sends them all null (buyers and sellers would be added together), and then there is no
  // totals row at all.
  const sums = first?.totals;
  const noSums =
    sums !== undefined &&
    [sums.qty, sums.amount, sums.commission, sums.open].every((sum) => sum === null);
  const empty = filtering ? (
    <EmptyState
      icon={SearchX}
      title={<Localized id="pukhta-no-match" />}
      help={<Localized id="pukhta-no-match-help" />}
      action={
        <Button onClick={clear}>
          <Localized id="pukhta-clear" />
        </Button>
      }
    />
  ) : (
    <EmptyState
      icon={UsersRound}
      title={<Localized id="pukhta-empty" />}
      help={<Localized id="pukhta-empty-help" />}
    />
  );
  // The totals row is the backend's, over every line of the filter and not over the rows loaded.
  const body = (
    <PagedList<KeyedRow>
      columns={columns}
      rows={rows}
      totalCount={first?.total_count}
      getRowId={(row) => row.key}
      labelId="pukhta-register-label"
      failedId="pukhta-load-failed"
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
        void navigate({ to: '/bikri/book', search: { id: row.bikri_id } });
      }}
      isInactive={(row) => row.cancelled}
      totals={
        first === undefined || noSums
          ? undefined
          : (context) => {
              const { qty, amount, commission, open } = first.totals;
              return {
                labelField: 'name',
                values: {
                  name: context.text('glossary-meezan'),
                  ...(qty === null ? {} : { qty: String(qty) }),
                  ...(amount === null ? {} : { amount: context.money(amount) }),
                  ...(commission === null ? {} : { commission: context.money(commission) }),
                  ...(open === null
                    ? {}
                    : { state: context.text('pukhta-state-due', { amount: context.money(open) }) }),
                },
              };
            }
      }
    />
  );

  return (
    <div className="flex h-full flex-col gap-4">
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div className="flex flex-col gap-1">
          <Localized id="glossary-pukhta-khata">
            <h1 className="type-title" />
          </Localized>
          <Localized id="pukhta-help">
            <p className="type-body text-ink-muted" />
          </Localized>
        </div>
        <Checkbox
          checked={moreColumns}
          onChange={setMoreColumns}
          label={<Localized id="common-more-columns" />}
        />
      </div>
      <div
        ref={containerRef}
        {...formNavigationProps}
        className="grid grid-cols-3 items-end gap-4 wide:grid-cols-6"
      >
        <Field label={<Localized id="pukhta-filter-from" />}>
          {(control) => <DateInput {...control} value={from} onChange={setFrom} />}
        </Field>
        <Field label={<Localized id="pukhta-filter-to" />}>
          {(control) => <DateInput {...control} value={to} onChange={setTo} />}
        </Field>
        <Field label={<Localized id="pukhta-col-side" />}>
          {(control) => (
            <Select
              {...control}
              value={side}
              onChange={setSide}
              options={[
                { value: ALL, label: <Localized id="pukhta-all" /> },
                { value: 'seller', label: <Localized id="pukhta-side-seller" /> },
                { value: 'buyer', label: <Localized id="pukhta-side-buyer" /> },
              ]}
            />
          )}
        </Field>
        <Field label={<Localized id="pukhta-col-name" />}>
          {(control) => <TextInput {...control} value={name} onChange={setName} />}
        </Field>
        <Field label={<Localized id="pukhta-col-phone" />}>
          {(control) => <TextInput {...control} value={phone} onChange={setPhone} lang="en" />}
        </Field>
        <Field label={<Localized id="pukhta-col-state" />}>
          {(control) => (
            <Select
              {...control}
              value={state}
              onChange={setState}
              options={[
                { value: ALL, label: <Localized id="pukhta-all" /> },
                { value: 'paid', label: <Localized id="pukhta-state-paid" /> },
                { value: 'due', label: <Localized id="pukhta-filter-due" /> },
              ]}
            />
          )}
        </Field>
      </div>
      {body}
    </div>
  );
}
