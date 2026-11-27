// The list under the خرچہ form (PRD 16.2, FR-16.2, FR-19.11): for دکان خرچہ the shop expenses of the
// day in the date field, with their heads and the backend's total; for ذاتی خرچہ and its واپسی the
// chosen partner's money taken, oldest first, with what was returned and what is still open, and with
// no partner chosen every partner's takes, latest day first, with his name on each row (plan 98). Both
// load from the backend (the day's list in one call, the takes page by page with totals over the whole
// list). Four states each: loading, an error with a retry, nothing yet, and the rows.

import { Localized } from '@fluent/react';
import { HandCoins, ReceiptText } from 'lucide-react';
import { useState } from 'react';
import { uiErrorOf } from '../../../shared/api';
import { PagedList } from '../../../shared/grid';
import { useDebouncedValue, useListWindow } from '../../../shared/hooks';
import { formNavigationProps } from '../../../shared/keyboard';
import {
  Card,
  Checkbox,
  EmptyState,
  FilterBar,
  ListWindowBar,
  PartnerLockedBox,
  ShowClosedTick,
} from '../../../shared/ui';
import { useDayStatus } from '../../../app/use-day-status';
import { useVouchersList } from '../../vouchers';
import { useOpenVoucher } from '../../../shared/edit';
import { VoucherRowActions } from './VoucherRowActions';
import { EXPENSE_DAY_COLUMNS, takeColumns } from '../list-columns';
import { usePartnerTakes } from '../hooks';
import type { ExpenseType } from '../screen-type';
import type { TakeRow } from '../types';

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
  const open = useOpenVoucher();
  const [selectedId, setSelectedId] = useState<number | null>(null);
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
    show_finished: showFinished,
    search: searchText === '' ? null : searchText,
    group_by_month: false,
  });
  const first = list.data?.pages[0];
  const rows = list.isError ? undefined : list.data?.pages.flatMap((page) => page.rows);
  const selected = rows?.find((row) => row.id === selectedId) ?? null;
  return (
    <>
      <div {...formNavigationProps}>
        <FilterBar
          screen="cashbook.day"
          search={{ value: search, onChange: setSearch }}
          extra={<ShowClosedTick checked={showFinished} onChange={setShowFinished} />}
        />
      </div>
      <VoucherRowActions row={selected} />
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
        onOpen={(row) => {
          if (row.can_edit && !row.needs_reopen) open(row);
          else setSelectedId(row.id);
        }}
        onFocusRow={(rowId) => {
          setSelectedId(rows?.find((row) => String(row.id) === rowId)?.id ?? null);
        }}
        focusRowId={selectedId === null ? null : String(selectedId)}
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

/**
 * The identity of a row of money taken: the partner, where it came from and which entry (two partners may have an
 * opening on one day). The grid also asks for the id of its totals row, an empty object, so a field may be missing.
 */
function takeRowId(row: Omit<TakeRow, 'partner'> & { partner?: TakeRow['partner'] }): string {
  return `${String(row.partner?.id)}-${row.source}-${String(row.voucher_id ?? row.entry_date)}`;
}

/** The money taken: one partner's (`partnerId`), or every partner's with his name on each row (null, plan 98). */
function PartnerTakesList({ partnerId }: { partnerId: number | null }) {
  const everyone = partnerId === null;
  const open = useOpenVoucher();
  // A take that is fully returned is left out until the tick is on (plan 38 G6).
  const [showReturned, setShowReturned] = useState(false);
  const dayStatus = useDayStatus();
  // The partner's money taken shows one month at a time (PRD FR-19.11, plan 76 point 8).
  const listWindow = useListWindow({
    kind: 'month',
    source: { kind: 'partner_takes', partner_id: partnerId },
    today: dayStatus.data?.today,
  });
  const windowed = !dayStatus.isError;
  const takes = usePartnerTakes(
    partnerId,
    true,
    showReturned,
    windowed ? { from: listWindow.range.from, to: listWindow.range.to } : { from: null, to: null },
    listWindow.ready || !windowed,
  );
  const first = takes.data?.pages[0];
  const rows = takes.data?.pages.flatMap((page) => page.rows);
  // A partner's money is behind his code: while it is locked the box stands in the list's place, never an error with a
  // retry (PRD FR-17.11). The month window is refused for the same reason, so there is no window bar either.
  const locked =
    takes.isError &&
    uiErrorOf(takes.error).errors.some((item) => item.code === 'ERR_PARTNER_LOCKED');
  if (locked) return <PartnerLockedBox />;
  return (
    <>
      <Checkbox
        checked={showReturned}
        onChange={setShowReturned}
        label={<Localized id="cashbook-show-returned" />}
      />
      <PagedList
        columns={takeColumns(everyone)}
        rows={rows}
        getRowId={takeRowId}
        // A row made by a voucher opens that voucher; an opening balance has no voucher.
        onOpen={(row) => {
          if (row.source === 'voucher' && row.voucher_id !== null) {
            open({ id: row.voucher_id, voucher_type: 'personal_expense' });
          }
        }}
        labelId={everyone ? 'cashbook-all-takes-title' : 'cashbook-takes-label'}
        failedId={everyone ? 'cashbook-all-failed' : 'cashbook-takes-failed'}
        error={takes.isError ? uiErrorOf(takes.error) : null}
        onRetry={() => {
          void takes.refetch();
        }}
        pager={takes.pager}
        onPageKey={listWindow.onPageKey}
        sort={{ value: takes.sort, onChange: takes.setSort }}
        months={
          first === undefined || first.months.length === 0
            ? undefined
            : { months: first.months, monthOf: (row) => row.entry_date.slice(0, 7) }
        }
        empty={
          <EmptyState
            icon={HandCoins}
            title={
              <Localized id={everyone ? 'cashbook-all-takes-empty' : 'cashbook-takes-empty'} />
            }
            help={
              <Localized id={everyone ? 'cashbook-all-empty-help' : 'cashbook-takes-empty-help'} />
            }
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
      {windowed ? <ListWindowBar window={listWindow} /> : null}
    </>
  );
}

/** The Fluent id of the card's title: the day's shop expenses, one partner's money, or every partner's (plan 98). */
function titleIdOf(type: ExpenseType, partnerId: number | null): string {
  if (type === 'shop_expense') return 'cashbook-day-title';
  return partnerId === null ? 'cashbook-all-takes-title' : 'cashbook-takes-title';
}

export function ExpenseListSection({ type, day, partnerId }: ExpenseListSectionProps) {
  return (
    <Card compact icon={ReceiptText} title={<Localized id={titleIdOf(type, partnerId)} />}>
      {type === 'shop_expense' ? (
        <ShopDay day={day} />
      ) : (
        // No partner chosen: every partner's money with his name on each row (plan 98), not a request to choose.
        <PartnerTakesList partnerId={partnerId} />
      )}
    </Card>
  );
}
