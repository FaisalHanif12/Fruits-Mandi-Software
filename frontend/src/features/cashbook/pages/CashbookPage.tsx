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
import { useMemo, useState } from 'react';
import { useDayStatus } from '../../../app/use-day-status';
import { uiErrorOf } from '../../../shared/api';
import { isoToDisplayDate } from '../../../shared/format';
import { PagedList } from '../../../shared/grid';
import { SHORTCUTS, formNavigationProps } from '../../../shared/keyboard';
import {
  Button,
  Card,
  DateInput,
  EmptyState,
  ErrorState,
  Fact,
  Field,
  LoadingState,
} from '../../../shared/ui';
import { SignedNum } from '../components/SignedNum';
import { useCashbook } from '../hooks';
import { CASHBOOK_COLUMNS } from '../list-columns';

/** Text or nothing: an empty date is sent as no limit. */
function textOrNull(text: string): string | null {
  const trimmed = text.trim();
  return trimmed === '' ? null : trimmed;
}

function CashbookScreen({ today }: { today: string }) {
  const navigate = useNavigate();
  const [from, setFrom] = useState(() => isoToDisplayDate(today));
  const [to, setTo] = useState(() => isoToDisplayDate(today));
  const range = useMemo(() => ({ from: textOrNull(from), to: textOrNull(to) }), [from, to]);
  const book = useCashbook(range);
  const first = book.data?.pages[0];
  const rows = book.data?.pages.flatMap((page) => page.rows);

  return (
    <div className="flex max-w-page flex-col gap-4">
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div className="flex flex-col gap-1">
          <Localized id="cashbook-title">
            <h1 className="sr-only normal:not-sr-only normal:type-title" />
          </Localized>
          <Localized id="cashbook-help">
            <p className="hidden type-body text-ink-muted normal:block" />
          </Localized>
        </div>
        <Button
          variant="primary"
          shortcut={SHORTCUTS.newExpense}
          onClick={() => {
            void navigate({ to: '/rokar/expense', search: { type: 'shop_expense' } });
          }}
        >
          <Localized id="cashbook-new-expense" />
        </Button>
      </div>
      <Card compact>
        <div
          {...formNavigationProps}
          className="grid grid-cols-2 items-end gap-4 normal:grid-cols-4"
        >
          <Field label={<Localized id="cashbook-filter-from" />}>
            {(control) => <DateInput {...control} value={from} onChange={setFrom} />}
          </Field>
          <Field label={<Localized id="cashbook-filter-to" />}>
            {(control) => <DateInput {...control} value={to} onChange={setTo} />}
          </Field>
          {first === undefined ? null : (
            <dl className="col-span-2 grid grid-cols-2 gap-4">
              <Fact label={<Localized id="cashbook-opening" />}>
                <SignedNum paisa={first.opening_paisa} />
              </Fact>
              <Fact label={<Localized id="cashbook-closing" />}>
                <SignedNum paisa={first.closing_paisa} />
              </Fact>
            </dl>
          )}
        </div>
        <PagedList
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
          empty={
            <EmptyState
              icon={Landmark}
              title={<Localized id="cashbook-empty" />}
              help={<Localized id="cashbook-empty-help" />}
              action={
                <Button
                  variant="primary"
                  onClick={() => {
                    void navigate({ to: '/rokar/expense', search: { type: 'shop_expense' } });
                  }}
                >
                  <Localized id="cashbook-new-expense" />
                </Button>
              }
            />
          }
          totals={
            first === undefined
              ? undefined
              : (context) => ({
                  labelField: 'kind',
                  values: {
                    kind: context.text('glossary-meezan'),
                    in: context.money(first.totals.in_paisa),
                    out: context.money(first.totals.out_paisa),
                  },
                })
          }
        />
      </Card>
    </div>
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
