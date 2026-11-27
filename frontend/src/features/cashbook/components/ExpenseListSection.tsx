// The list under the خرچہ form (PRD 16.2, FR-16.2, FR-19.11): for دکان خرچہ every shop expense, 30 days at
// a time, the latest first (plan 100 F, `EntryListSection`); for ذاتی خرچہ and its واپسی the chosen
// partner's money taken, oldest first, with what was returned and what is still open, and with no partner
// chosen every partner's takes, latest day first, with his name on each row (plan 98). Both load from the
// backend page by page, with the backend's totals. Four states each: loading, an error with a retry,
// nothing yet, and the rows.

import { Localized } from '@fluent/react';
import { HandCoins, ReceiptText } from 'lucide-react';
import { useState } from 'react';
import { uiErrorOf } from '../../../shared/api';
import { PagedList } from '../../../shared/grid';
import { useListWindow } from '../../../shared/hooks';
import { Card, Checkbox, EmptyState, ListWindowBar, PartnerLockedBox } from '../../../shared/ui';
import { useDayStatus } from '../../../app/use-day-status';
import { useOpenVoucher } from '../../../shared/edit';
import { SHOP_ENTRY_COLUMNS, SHOP_ENTRY_TYPES } from '../entry-list-columns';
import { takeColumns } from '../list-columns';
import { usePartnerTakes } from '../hooks';
import type { ExpenseType } from '../screen-type';
import type { TakeRow } from '../types';
import { EntryListSection } from './EntryListSection';

interface ExpenseListSectionProps {
  type: ExpenseType;
  /** Today, `YYYY-MM-DD`. */
  today: string;
  /** The partner chosen on the form, or null. */
  partnerId: number | null;
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
  // A partner's own spending needs no partner code since v1.61 (plan 100 E, PRD FR-17.11). If the backend ever
  // refuses it with ERR_PARTNER_LOCKED, the code box stands in the list's place, never an error with a retry.
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

export function ExpenseListSection({ type, today, partnerId }: ExpenseListSectionProps) {
  if (type === 'shop_expense') {
    return (
      <EntryListSection
        screen="cashbook.shop.list"
        titleId="cashbook-entries-shop-title"
        types={SHOP_ENTRY_TYPES}
        accountId={null}
        source={{ kind: 'vouchers', voucher_type: 'shop_expense' }}
        columns={SHOP_ENTRY_COLUMNS}
        today={today}
      />
    );
  }
  return (
    <Card
      compact
      icon={ReceiptText}
      title={
        <Localized id={partnerId === null ? 'cashbook-all-takes-title' : 'cashbook-takes-title'} />
      }
    >
      {/* No partner chosen: every partner's money with his name on each row (plan 98), not a request to choose. */}
      <PartnerTakesList partnerId={partnerId} />
    </Card>
  );
}
