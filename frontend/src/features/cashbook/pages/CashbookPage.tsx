// روکڑ بک, the cash book (route /rokar; PRD 16.1, FR-16.1, FR-16.14 v1.60, FR-19.10, FR-19.11, NFR-SPD-24):
// the shop's one cash account. Three cards at the top (کل سرمایہ, شروع میں روکڑ, آخر میں روکڑ), then for a
// window of days every cash movement of every module, each named for what it is with its party, garden or
// trip. A type filter, a switch that shows old and cancelled postings too (hidden otherwise), and what came
// in and went out by type. It loads page by page; every card, total and summary figure is the backend's
// over the whole window, never a sum of the rows on screen. There is no box to type the counted cash
// (FR-16.1). The window starts as the last 30 days.
//
// Four states: loading, an error with a retry, nothing to show (a helpful message with a button to the
// خرچہ screen, or back to every type), and the rows. A language switch loses nothing: the window, the
// filter and the switch are in this screen's state (F39).

import { Localized } from '@fluent/react';
import { useNavigate } from '@tanstack/react-router';
import { ArrowRight, Landmark } from 'lucide-react';
import { useCallback, useId, useMemo, useState } from 'react';
import { useDayStatus } from '../../../app/use-day-status';
import type { CashbookRow } from '../../../bindings/CashbookRow';
import { uiErrorOf } from '../../../shared/api';
import { PagedList, type ListContext } from '../../../shared/grid';
import { useListWindow, useRememberedFlag } from '../../../shared/hooks';
import { SHORTCUTS, formNavigationProps } from '../../../shared/keyboard';
import {
  Button,
  Checkbox,
  EmptyState,
  ErrorState,
  Field,
  FilterBar,
  ListPage,
  LoadingState,
  ListWindowBar,
  FilterCard,
  Notice,
  Num,
  Select,
  type SelectOption,
} from '../../../shared/ui';
import { useOpenRowSource } from '../../reports';
import { CashCards } from '../components/CashCards';
import { CashSummary, CashSummaryToggle } from '../components/CashSummary';
import { OpeningsToCashView } from '../components/OpeningsToCashView';
import { useCashbook, useOpeningsToCashPreview } from '../hooks';
import { CASHBOOK_COLUMNS } from '../list-columns';
import { groupMessageId } from '../row-text';
import type { CashGroup, CashGroupSummary, CashKind } from '../types';

/** The choice of the type filter that shows every type. */
const EVERY = 'all';
type TypeChoice = CashGroup | typeof EVERY;

/** The chosen type with its kinds as the backend named them in the summary. */
interface TypeFilter {
  group: CashGroup;
  kinds: readonly CashKind[];
}

/** Text or nothing: an empty date is sent as no limit. */
function textOrNull(text: string): string | null {
  const trimmed = text.trim();
  return trimmed === '' ? null : trimmed;
}

/** The month of a cash book line, `YYYY-MM`: the book is in date order. */
const CASH_MONTH_OF = (row: CashbookRow) => row.entry_date.slice(0, 7);
/** The backend names the subtotal of a month like its totals; the columns have short names. */
const CASH_MONTH_COLUMNS = { in_paisa: 'in', out_paisa: 'out', running_paisa: 'running' } as const;

function CashbookScreen({ today }: { today: string }) {
  const navigate = useNavigate();
  const openSource = useOpenRowSource();
  // The book shows 30 days at a time, the latest first (PRD FR-19.11, v1.57, plan 95 item 18).
  const listWindow = useListWindow({ kind: 'days', source: { kind: 'cashbook' }, today });
  const range = useMemo(
    () => ({ from: textOrNull(listWindow.range.from), to: textOrNull(listWindow.range.to) }),
    [listWindow.range.from, listWindow.range.to],
  );
  // The type filter and the switch of old and cancelled postings (PRD FR-16.14 v1.60).
  const [filter, setFilter] = useState<TypeFilter | null>(null);
  const [showReversed, setShowReversed] = useState(false);
  const [summaryOpen, setSummaryOpen] = useRememberedFlag('cashbook.summary_open', false);
  const summaryId = useId();
  const kinds = filter?.kinds ?? null;
  const view = useMemo(() => ({ showReversed, kinds }), [showReversed, kinds]);
  const book = useCashbook(range, true, view);
  // Old opening balances not yet in روکڑ are offered a one-time fix, shown in place of the book (FR-10.9).
  const oldOpenings = useOpeningsToCashPreview().data?.rows.length ?? 0;
  const [fixing, setFixing] = useState(false);
  const first = book.data?.pages[0];
  const rows = book.isError ? undefined : book.data?.pages.flatMap((page) => page.rows);
  // The summary lists every type with its kinds; it is the same whatever type is chosen.
  const summary: readonly CashGroupSummary[] = first?.summary ?? [];
  const chooseType = (group: TypeChoice) => {
    const line = summary.find((candidate) => candidate.group === group);
    setFilter(line === undefined ? null : { group: line.group, kinds: line.kinds });
  };
  const typeOptions: readonly SelectOption<TypeChoice>[] = [
    { value: EVERY, label: <Localized id="cashbook-filter-type-all" /> },
    ...summary.map((line) => ({
      value: line.group,
      label: <Localized id={groupMessageId(line.group)} />,
    })),
  ];
  // The totals under the columns are the backend's, over the whole range and filter. The callback depends
  // on the loaded page, not on an amount: a Paisa value must not go into a dependency list (F3).
  const totals = useCallback(
    (context: ListContext) => ({
      labelField: 'kind',
      values:
        first === undefined
          ? {}
          : {
              kind: context.text('glossary-meezan'),
              in: context.money(first.totals.in_paisa),
              out: context.money(first.totals.out_paisa),
            },
    }),
    [first],
  );
  const newExpense = () => {
    void navigate({ to: '/rokar/expense', search: { type: 'shop_expense' } });
  };
  if (fixing) {
    return (
      <OpeningsToCashView
        onBack={() => {
          setFixing(false);
        }}
      />
    );
  }
  // The running cash shows only in day order over every type (the backend sends none otherwise).
  const runningNote =
    filter !== null
      ? 'cashbook-running-all-types-only'
      : book.sort !== null && book.sort.column !== 'entry_date'
        ? 'cashbook-running-date-only'
        : null;

  return (
    <ListPage
      icon={Landmark}
      titleId="cashbook-title"
      help={
        <Localized id="cashbook-help">
          <p />
        </Localized>
      }
      actions={
        <Button variant="primary" shortcut={SHORTCUTS.newExpense} onClick={newExpense}>
          <Localized id="cashbook-new-expense" />
        </Button>
      }
      filters={
        <div className="flex flex-col gap-2 shortest:gap-1">
          {oldOpenings === 0 ? null : (
            <Notice
              tone="warn"
              action={
                <Button
                  onClick={() => {
                    setFixing(true);
                  }}
                >
                  <ArrowRight aria-hidden="true" className="size-icon-md rtl:rotate-180" />
                  <Localized id="cashbook-openings-button" />
                </Button>
              }
            >
              <Localized
                id="cashbook-openings-offer"
                vars={{ count: String(oldOpenings) }}
                elems={{ num: <Num /> }}
              >
                <span />
              </Localized>
            </Notice>
          )}
          {first === undefined ? null : <CashCards book={first} />}
          <FilterCard {...formNavigationProps} className="shortest:py-1">
            <FilterBar
              screen="cashbook.book"
              extra={
                <>
                  <Checkbox
                    checked={showReversed}
                    onChange={setShowReversed}
                    label={<Localized id="cashbook-show-reversed" />}
                  />
                  <CashSummaryToggle
                    summaryId={summaryId}
                    open={summaryOpen}
                    onToggle={() => {
                      setSummaryOpen(!summaryOpen);
                    }}
                  />
                </>
              }
            >
              <Field label={<Localized id="cashbook-filter-type" />}>
                {(control) => (
                  <Select<TypeChoice>
                    {...control}
                    value={filter?.group ?? EVERY}
                    options={typeOptions}
                    onChange={chooseType}
                  />
                )}
              </Field>
            </FilterBar>
          </FilterCard>
        </div>
      }
    >
      {summaryOpen ? <CashSummary id={summaryId} summary={summary} onChoose={chooseType} /> : null}
      {runningNote === null ? null : (
        <Localized id={runningNote}>
          <p className="shrink-0 type-body text-ink-muted" />
        </Localized>
      )}
      <PagedList<CashbookRow>
        columns={CASHBOOK_COLUMNS}
        rows={rows}
        // A sale's posting can give two rows (paid on the spot, cash paid) with one entry.
        getRowId={(row) => `${String(row.entry_id)}-${row.kind}`}
        // A row opens the entry behind it (PRD FR-19.4): a sale, a voucher, a trip purchase, a trip
        // or a garden, fruit taken or an account; an entry with no screen of its own opens nothing.
        onOpen={(row) => {
          openSource(row.source);
        }}
        labelId="cashbook-list-label"
        failedId="cashbook-load-failed"
        error={book.isError ? uiErrorOf(book.error) : null}
        onRetry={() => {
          void book.refetch();
        }}
        pager={book.pager}
        // The cards above leave little room, so a long book is as tall as its rows and the page scrolls with the
        // pager under it, in place of a few lines with a scroll inside (plan 96 T2).
        autoHeight
        onPageKey={listWindow.onPageKey}
        sort={{ value: book.sort, onChange: book.setSort }}
        months={
          first === undefined
            ? undefined
            : { months: first.months, monthOf: CASH_MONTH_OF, columnOf: CASH_MONTH_COLUMNS }
        }
        empty={
          filter === null ? (
            <EmptyState
              icon={Landmark}
              title={<Localized id="cashbook-empty" />}
              help={<Localized id="cashbook-empty-help" />}
              action={
                <Button variant="primary" onClick={newExpense}>
                  <Localized id="cashbook-new-expense" />
                </Button>
              }
            />
          ) : (
            <EmptyState
              icon={Landmark}
              title={<Localized id="cashbook-empty-type" />}
              help={<Localized id="cashbook-empty-type-help" />}
              action={
                <Button
                  variant="primary"
                  onClick={() => {
                    setFilter(null);
                  }}
                >
                  <Localized id="cashbook-empty-type-action" />
                </Button>
              }
            />
          )
        }
        totals={totals}
      />
      <ListWindowBar window={listWindow} />
    </ListPage>
  );
}

/** The route's page: it waits for today, then opens the book on that day. */
export function CashbookPage() {
  const day = useDayStatus();
  if (day.isPending) return <LoadingState />;
  if (day.isError) {
    return (
      <ErrorState
        title={<Localized id="cashbook-load-failed" />}
        error={uiErrorOf(day.error)}
        onRetry={() => {
          void day.refetch();
        }}
      />
    );
  }
  return <CashbookScreen today={day.data.today} />;
}
