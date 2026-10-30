// The list under the خرچہ form (PRD 16.2, FR-16.2, FR-19.11): for دکان خرچہ the shop expenses of the
// day in the date field, with their heads and the backend's total; for ذاتی خرچہ and its واپسی the
// chosen partner's money taken, oldest first, with what was returned and what is still open. Both
// load from the backend (the day's list in one call, the takes page by page with totals over the
// whole list). Four states each: loading, an error with a retry, nothing yet, and the rows.

import { Localized } from '@fluent/react';
import { HandCoins, ReceiptText } from 'lucide-react';
import { useState } from 'react';
import { uiErrorOf } from '../../../shared/api';
import { PagedList } from '../../../shared/grid';
import { useDebouncedValue } from '../../../shared/hooks';
import { formNavigationProps } from '../../../shared/keyboard';
import { Card, Checkbox, EmptyState, FilterBar, ShowClosedTick } from '../../../shared/ui';
import { useVouchersList } from '../../vouchers';
import { EXPENSE_DAY_COLUMNS, TAKE_COLUMNS } from '../list-columns';
import { usePartnerTakes } from '../hooks';
import type { ExpenseType } from '../screen-type';

/** A pause so a burst of key presses in the search box asks once. */
const SEARCH_DELAY_MS = 150;

interface ExpenseListSectionProps {
  type: ExpenseType;
  /** The day in the date field, `DD-MM-YYYY`, when it is a whole date; else today. */
  day: string;
  /** The partner chosen on the form, or null. */
  partnerId: number | null;
}

function ShopDay({ day }: { day: string }) {
  // The cancelled expenses of the day step aside until the tick is on (plan 38 G6); the search box
  // finds a number, a head or the words of the narration.
  const [search, setSearch] = useState('');
  const [showFinished, setShowFinished] = useState(false);
  const searchText = useDebouncedValue(search.trim(), SEARCH_DELAY_MS);
  const list = useVouchersList({
    voucher_type: 'shop_expense',
    from: day,
    to: day,
    account_id: null,
    person_key: null,
    show_finished: showFinished,
    search: searchText === '' ? null : searchText,
    group_by_month: false,
  });
  const first = list.data?.pages[0];
  const rows = list.isError ? undefined : list.data?.pages.flatMap((page) => page.rows);
  return (
    <>
      <div {...formNavigationProps}>
        <FilterBar
          screen="cashbook.day"
          search={{ value: search, onChange: setSearch }}
          extra={<ShowClosedTick checked={showFinished} onChange={setShowFinished} />}
        />
      </div>
      <PagedList
        columns={EXPENSE_DAY_COLUMNS}
        rows={rows}
        getRowId={(row) => String(row.id)}
        labelId="cashbook-day-label"
        failedId="cashbook-day-failed"
        error={list.isError ? uiErrorOf(list.error) : null}
        onRetry={() => {
          void list.refetch();
        }}
        pager={list.pager}
        sort={{ value: list.sort, onChange: list.setSort }}
        empty={
          <EmptyState
            icon={ReceiptText}
            title={<Localized id="cashbook-day-empty" />}
            help={<Localized id="cashbook-day-empty-help" />}
          />
        }
        isInactive={(row) => row.status === 'cancelled'}
        totals={
          first === undefined
            ? undefined
            : (context) => ({
                labelField: 'head',
                values: {
                  head: context.text('glossary-meezan'),
                  ...(first.totals.total_paisa === null
                    ? {}
                    : { amount: context.money(first.totals.total_paisa) }),
                },
              })
        }
      />
    </>
  );
}

function PartnerTakesList({ partnerId }: { partnerId: number }) {
  // A take that is fully returned is left out until the tick is on (plan 38 G6).
  const [showReturned, setShowReturned] = useState(false);
  const takes = usePartnerTakes(partnerId, true, showReturned);
  const first = takes.data?.pages[0];
  const rows = takes.data?.pages.flatMap((page) => page.rows);
  return (
    <>
      <Checkbox
        checked={showReturned}
        onChange={setShowReturned}
        label={<Localized id="cashbook-show-returned" />}
      />
      <PagedList
        columns={TAKE_COLUMNS}
        rows={rows}
        getRowId={(row) => `${row.source}-${String(row.voucher_id ?? row.entry_date)}`}
        labelId="cashbook-takes-label"
        failedId="cashbook-takes-failed"
        error={takes.isError ? uiErrorOf(takes.error) : null}
        onRetry={() => {
          void takes.refetch();
        }}
        pager={takes.pager}
        sort={{ value: takes.sort, onChange: takes.setSort }}
        months={
          first === undefined || first.months.length === 0
            ? undefined
            : { months: first.months, monthOf: (row) => row.entry_date.slice(0, 7) }
        }
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
    </>
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
