// روکڑ بک, the cash book (route /rokar; PRD 16.1, FR-16.1, FR-19.10, FR-19.11, NFR-SPD-24): the
// shop's one cash account. For a range of days it shows the opening cash, every cash movement of
// every module (date, what it was, the account, in, out and the running cash) and the closing
// cash. It loads page by page; the opening and closing cash and the totals under the columns are
// the backend's over the whole range, never a sum of the rows on screen. There is no box to type
// the counted cash (FR-16.1). The day starts as today; clearing a date widens the range.
//
// Four states: loading, an error with a retry, nothing in the range (a helpful message with a
// button to the خرچہ screen), and the rows. A language switch loses nothing: the dates are text in
// this screen's state (F39).

import { Localized } from '@fluent/react';
import { useNavigate } from '@tanstack/react-router';
import { Landmark } from 'lucide-react';
import { useCallback, useMemo } from 'react';
import { useDayStatus } from '../../../app/use-day-status';
import type { CashbookRow } from '../../../bindings/CashbookRow';
import { uiErrorOf } from '../../../shared/api';
import { PagedList, type ListContext } from '../../../shared/grid';
import { usePeriod } from '../../../shared/hooks';
import { SHORTCUTS, formNavigationProps } from '../../../shared/keyboard';
import {
  Button,
  EmptyState,
  ErrorState,
  Fact,
  FilterBar,
  ListPage,
  LoadingState,
  PeriodBox,
} from '../../../shared/ui';
import { SignedNum } from '../components/SignedNum';
import { useCashbook } from '../hooks';
import { CASHBOOK_COLUMNS } from '../list-columns';

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
  // The book opens on today and the Owner may widen it (the period is remembered, plan 38 R4).
  const period = usePeriod('cashbook.book', 'today', today);
  const range = useMemo(
    () => ({ from: textOrNull(period.range.from), to: textOrNull(period.range.to) }),
    [period.range.from, period.range.to],
  );
  const book = useCashbook(range, true);
  const first = book.data?.pages[0];
  const rows = book.isError ? undefined : book.data?.pages.flatMap((page) => page.rows);
  // The totals under the columns are the backend's, over the whole range. The callback depends on the
  // loaded page, not on an amount: a Paisa value must not go into a dependency list (F3).
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

  return (
    <ListPage
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
        <div {...formNavigationProps}>
          <FilterBar screen="cashbook.book">
            <PeriodBox period={period} />
            {/* The cash at the start and at the end of the range sit in the same row as the period
                (the backend's figures over the whole range). */}
            {first === undefined ? null : (
              <>
                <dl>
                  <Fact label={<Localized id="cashbook-opening" />}>
                    <SignedNum paisa={first.opening_paisa} />
                  </Fact>
                </dl>
                <dl>
                  <Fact label={<Localized id="cashbook-closing" />}>
                    <SignedNum paisa={first.closing_paisa} />
                  </Fact>
                </dl>
              </>
            )}
          </FilterBar>
        </div>
      }
    >
      {/* A list sorted by anything but the day has no running cash: the backend sends none. */}
      {book.sort !== null && book.sort.column !== 'entry_date' ? (
        <Localized id="cashbook-running-date-only">
          <p className="shrink-0 type-body text-ink-muted" />
        </Localized>
      ) : null}
      <PagedList<CashbookRow>
        columns={CASHBOOK_COLUMNS}
        rows={rows}
        getRowId={(row) => `${String(row.entry_id)}-${row.posting_type}`}
        labelId="cashbook-list-label"
        failedId="cashbook-load-failed"
        error={book.isError ? uiErrorOf(book.error) : null}
        onRetry={() => {
          void book.refetch();
        }}
        pager={book.pager}
        sort={{ value: book.sort, onChange: book.setSort }}
        months={
          first === undefined
            ? undefined
            : { months: first.months, monthOf: CASH_MONTH_OF, columnOf: CASH_MONTH_COLUMNS }
        }
        empty={
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
        }
        totals={totals}
      />
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
