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
import { useCallback, useEffect, useMemo, useState } from 'react';
import { useDayStatus } from '../../../app/use-day-status';
import type { PaidState } from '../../../bindings/PaidState';
import type { PukhtaRegisterInput } from '../../../bindings/PukhtaRegisterInput';
import type { PukhtaRegisterRow } from '../../../bindings/PukhtaRegisterRow';
import type { PukhtaSide } from '../../../bindings/PukhtaSide';
import { uiErrorOf } from '../../../shared/api';
import { PagedList, type ListContext } from '../../../shared/grid';
import {
  useFocusFirstField,
  useLayoutStep,
  usePeriod,
  useRememberedFlag,
} from '../../../shared/hooks';
import { formNavigationProps, useEscape } from '../../../shared/keyboard';
import { Remembered, type MemoryProps } from '../../../shared/screen-state';
import {
  Button,
  Checkbox,
  EmptyState,
  Field,
  FilterBar,
  ListPage,
  PeriodBox,
  Select,
  ShowClosedTick,
} from '../../../shared/ui';
import { pukhtaColumns } from '../columns';
import { usePukhtaRegister } from '../hooks';
import { PUKHTA_REGISTER_SCREEN, registerChoicesFrom, registerChoicesText } from '../remember';

const ALL = 'all';

/** The month of a register line, `YYYY-MM`: the rows come in date order. */
const MONTH_OF = (row: PukhtaRegisterRow) => row.entry_date.slice(0, 7);
/** The backend's open amount of a month goes under the column of paid or due. */
const MONTH_COLUMNS = { open: 'state' } as const;

/** A register row has no id of its own (two equal lines are possible), so its place is its key. */
type KeyedRow = PukhtaRegisterRow & { key: string };

/** The register starts as the Owner left its filters (PRD FR-7.45). */
export function PukhtaRegisterPage() {
  return (
    <Remembered screen={PUKHTA_REGISTER_SCREEN}>
      {(memory) => <PukhtaRegisterBody {...memory} />}
    </Remembered>
  );
}

function PukhtaRegisterBody({ saved, remember }: MemoryProps) {
  const navigate = useNavigate();
  const start = useMemo(() => registerChoicesFrom(saved), [saved]);
  const day = useDayStatus();
  // The register opens on this month, as the Owner last chose (plan 38 R4); the name, the phone, the
  // item and the number are one search box.
  const period = usePeriod('pukhta.register', 'month', day.data?.today);
  const [search, setSearch] = useState('');
  const [side, setSide] = useState<PukhtaSide | typeof ALL>(start.side);
  const [state, setState] = useState<PaidState | typeof ALL>(start.state);
  // Lines of a cancelled بکری step aside until the tick is on.
  const [showFinished, setShowFinished] = useState(false);
  useEffect(() => {
    remember(registerChoicesText({ from: '', to: '', side, name: '', phone: '', state }));
  }, [side, state, remember]);
  const filter: Omit<PukhtaRegisterInput, 'offset' | 'limit' | 'sort'> = {
    from: period.range.from,
    to: period.range.to,
    side: side === ALL ? null : side,
    name: null,
    phone: null,
    state: state === ALL ? null : state,
    show_finished: showFinished,
    search: search.trim() === '' ? null : search.trim(),
    group_by_month: true,
  };
  const filtering =
    period.state.kind !== 'all' ||
    filter.side !== null ||
    filter.search !== null ||
    filter.state !== null;
  const list = usePukhtaRegister(filter, period.ready);
  // The tick that adds the نگ column, which does not fit beside the others (FR-13.6). It is kept.
  const [moreColumns, setMoreColumns] = useRememberedFlag('pukhta.more_columns', false);
  const step = useLayoutStep();
  const columns = useMemo(() => pukhtaColumns(step, moreColumns), [step, moreColumns]);
  const containerRef = useFocusFirstField<HTMLDivElement>();
  useEscape('leave', () => {
    void navigate({ to: '/' });
  });
  const clear = () => {
    period.change({ kind: 'all', from: '', to: '' });
    setSide(ALL);
    setSearch('');
    setState(ALL);
    setShowFinished(false);
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
  // The totals row is the backend's, over every line of the filter and not over the rows loaded. The
  // callback depends on the loaded page and is made once per page (F3).
  const totals = useCallback(
    (context: ListContext) => {
      const sumsOfPage = first?.totals;
      return {
        labelField: 'name',
        values:
          sumsOfPage === undefined
            ? {}
            : {
                name: context.text('glossary-meezan'),
                ...(sumsOfPage.qty === null ? {} : { qty: String(sumsOfPage.qty) }),
                ...(sumsOfPage.amount === null ? {} : { amount: context.money(sumsOfPage.amount) }),
                ...(sumsOfPage.commission === null
                  ? {}
                  : { commission: context.money(sumsOfPage.commission) }),
                ...(sumsOfPage.open === null
                  ? {}
                  : {
                      state: context.text('pukhta-state-due', {
                        amount: context.money(sumsOfPage.open),
                      }),
                    }),
              },
      };
    },
    [first],
  );
  const body = (
    <PagedList<KeyedRow>
      columns={columns}
      rows={rows}
      getRowId={(row) => row.key}
      labelId="pukhta-register-label"
      failedId="pukhta-load-failed"
      error={list.isError ? uiErrorOf(list.error) : null}
      onRetry={() => {
        void list.refetch();
      }}
      pager={list.pager}
      sort={{ value: list.sort, onChange: list.setSort }}
      months={
        first === undefined
          ? undefined
          : { months: first.months, monthOf: MONTH_OF, columnOf: MONTH_COLUMNS }
      }
      empty={empty}
      onOpen={(row) => {
        void navigate({ to: '/bikri/book', search: { id: row.bikri_id } });
      }}
      isInactive={(row) => row.cancelled}
      totals={noSums ? undefined : totals}
    />
  );

  return (
    <ListPage
      titleId="glossary-pukhta-khata"
      help={
        <Localized id="pukhta-help">
          <p />
        </Localized>
      }
      filters={
        <div ref={containerRef} {...formNavigationProps}>
          <FilterBar
            screen="pukhta.register"
            search={{
              value: search,
              onChange: setSearch,
              placeholderId: 'pukhta-search-placeholder',
            }}
            moreSet={state === ALL ? 0 : 1}
            more={
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
            }
            extra={
              <>
                <ShowClosedTick checked={showFinished} onChange={setShowFinished} />
                <Checkbox
                  checked={moreColumns}
                  onChange={setMoreColumns}
                  label={<Localized id="common-more-columns" />}
                />
              </>
            }
          >
            {period.ready ? <PeriodBox period={period} /> : null}
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
          </FilterBar>
        </div>
      }
    >
      {body}
    </ListPage>
  );
}
