// The list under the خرچہ form (PRD 16.2, FR-16.2, FR-19.11): for دکان خرچہ the shop expenses of the
// day in the date field, with their heads and the backend's total; for ذاتی خرچہ and its واپسی the
// chosen partner's money taken, oldest first, with what was returned and what is still open. Both
// load from the backend (the day's list in one call, the takes page by page with totals over the
// whole list). Four states each: loading, an error with a retry, nothing yet, and the rows.

import { Localized } from '@fluent/react';
import { HandCoins, ReceiptText } from 'lucide-react';
import { uiErrorOf } from '../../../shared/api';
import { ListGrid, type ListContext } from '../../../shared/grid';
import { Card, EmptyState, ErrorState, LoadingState } from '../../../shared/ui';
import { useVouchersList } from '../../vouchers';
import { EXPENSE_DAY_COLUMNS, TAKE_COLUMNS } from '../list-columns';
import { usePartnerTakes } from '../hooks';
import type { ExpenseType } from '../screen-type';
import { PagedList } from './PagedList';

interface ExpenseListSectionProps {
  type: ExpenseType;
  /** The day in the date field, `DD-MM-YYYY`, when it is a whole date; else today. */
  day: string;
  /** The partner chosen on the form, or null. */
  partnerId: number | null;
}

function ShopDay({ day }: { day: string }) {
  const list = useVouchersList({
    voucher_type: 'shop_expense',
    from: day,
    to: day,
    account_id: null,
    person_key: null,
  });
  if (list.isPending) return <LoadingState rows={2} />;
  if (list.isError) {
    return (
      <ErrorState
        title={<Localized id="cashbook-day-failed" />}
        error={uiErrorOf(list.error)}
        onRetry={() => {
          void list.refetch();
        }}
      />
    );
  }
  if (list.data.rows.length === 0) {
    return (
      <EmptyState
        icon={ReceiptText}
        title={<Localized id="cashbook-day-empty" />}
        help={<Localized id="cashbook-day-empty-help" />}
      />
    );
  }
  const total = list.data.total_paisa;
  return (
    <ListGrid
      columns={EXPENSE_DAY_COLUMNS}
      rows={list.data.rows}
      getRowId={(row) => String(row.id)}
      labelId="cashbook-day-label"
      onOpen={() => undefined}
      heightMode="auto"
      isInactive={(row) => row.status === 'cancelled'}
      totals={(context: ListContext) => ({
        labelField: 'head',
        values: {
          head: context.text('glossary-meezan'),
          ...(total === null ? {} : { amount: context.money(total) }),
        },
      })}
    />
  );
}

function PartnerTakesList({ partnerId }: { partnerId: number }) {
  const takes = usePartnerTakes(partnerId);
  const first = takes.data?.pages[0];
  const rows = takes.data?.pages.flatMap((page) => page.rows);
  return (
    <PagedList
      columns={TAKE_COLUMNS}
      rows={rows}
      totalCount={first?.total_count}
      getRowId={(row) => `${row.source}-${String(row.voucher_id ?? row.entry_date)}`}
      labelId="cashbook-takes-label"
      failedId="cashbook-takes-failed"
      error={takes.isError ? uiErrorOf(takes.error) : null}
      onRetry={() => {
        void takes.refetch();
      }}
      hasMore={takes.hasNextPage}
      loadingMore={takes.isFetchingNextPage}
      onLoadMore={() => {
        void takes.fetchNextPage();
      }}
      empty={
        <EmptyState
          icon={HandCoins}
          title={<Localized id="cashbook-takes-empty" />}
          help={<Localized id="cashbook-takes-empty-help" />}
        />
      }
      totals={
        first === undefined
          ? undefined
          : (context) => ({
              labelField: 'reason',
              values: {
                reason: context.text('glossary-meezan'),
                amount: context.money(first.totals.taken_paisa),
                returned: context.money(first.totals.returned_paisa),
                open: context.money(first.totals.open_paisa),
              },
            })
      }
    />
  );
}

export function ExpenseListSection({ type, day, partnerId }: ExpenseListSectionProps) {
  const shop = type === 'shop_expense';
  return (
    <Card compact title={<Localized id={shop ? 'cashbook-day-title' : 'cashbook-takes-title'} />}>
      {shop ? (
        <ShopDay day={day} />
      ) : partnerId === null ? (
        <EmptyState
          icon={HandCoins}
          title={<Localized id="cashbook-takes-pick" />}
          help={<Localized id="cashbook-takes-pick-help" />}
        />
      ) : (
        <PartnerTakesList partnerId={partnerId} />
      )}
    </Card>
  );
}
