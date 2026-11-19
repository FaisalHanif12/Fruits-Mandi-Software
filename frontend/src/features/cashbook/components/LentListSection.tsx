// The list under the ادھار رقم form (PRD FR-16.5, FR-19.11): the chosen person's loans, oldest
// first, with what was returned and what is still open, his balance, and the backend's totals over
// the whole list. Four states: loading, an error with a retry, nothing yet, and the rows. Without
// a person it says to choose one.

import { Localized } from '@fluent/react';
import { HandCoins } from 'lucide-react';
import { useState } from 'react';
import { uiErrorOf } from '../../../shared/api';
import { PagedList } from '../../../shared/grid';
import { useListWindow } from '../../../shared/hooks';
import { isPositivePaisa } from '../../../shared/format';
import { Balance, Card, Checkbox, EmptyState, ListWindowBar } from '../../../shared/ui';
import { useDayStatus } from '../../../app/use-day-status';
import { useLentList } from '../hooks';
import { useOpenVoucher } from '../../../shared/edit';
import { LENT_COLUMNS } from '../list-columns';
import { PersonDetailsCard } from './PersonDetailsCard';

export function LentListSection({ personId }: { personId: number | null }) {
  const open = useOpenVoucher();
  // A loan that is fully returned is left out until the tick is on (plan 38 G6).
  const [showReturned, setShowReturned] = useState(false);
  const dayStatus = useDayStatus();
  // The loans show one month at a time, the latest first (PRD FR-19.11, plan 76 point 8). Without the day (its
  // read failed) the list asks for every loan, as before the window.
  const listWindow = useListWindow({
    kind: 'month',
    source: { kind: 'lent' },
    today: dayStatus.data?.today,
  });
  const windowed = !dayStatus.isError;
  const lent = useLentList(
    personId,
    true,
    showReturned,
    windowed ? { from: listWindow.range.from, to: listWindow.range.to } : { from: null, to: null },
    listWindow.ready || !windowed,
  );
  const first = lent.data?.pages[0];
  const rows = lent.data?.pages.flatMap((page) => page.rows);
  return (
    <Card compact icon={HandCoins} title={<Localized id="cashbook-lent-title" />}>
      {personId === null ? (
        <EmptyState
          icon={HandCoins}
          title={<Localized id="cashbook-lent-pick" />}
          help={<Localized id="cashbook-lent-pick-help" />}
        />
      ) : (
        <>
          {first === undefined ? null : (
            <PersonDetailsCard
              person={first.person}
              owes={isPositivePaisa(first.balance.amount_paisa)}
            />
          )}
          {first === undefined ? null : (
            <p className="flex items-baseline gap-2 type-body">
              <Localized id="cashbook-lent-balance" />
              <Balance balance={first.balance} />
            </p>
          )}
          <Checkbox
            checked={showReturned}
            onChange={setShowReturned}
            label={<Localized id="cashbook-show-returned" />}
          />
          <PagedList
            columns={LENT_COLUMNS}
            rows={rows}
            getRowId={(row) => String(row.voucher_id)}
            // A loan made by a voucher opens that voucher; an opening balance (id 0) has none.
            onOpen={(row) => {
              if (row.voucher_id > 0) open({ id: row.voucher_id, voucher_type: 'money_lent' });
            }}
            labelId="cashbook-lent-label"
            failedId="cashbook-lent-failed"
            error={lent.isError ? uiErrorOf(lent.error) : null}
            onRetry={() => {
              void lent.refetch();
            }}
            pager={lent.pager}
            onPageKey={listWindow.onPageKey}
            sort={{ value: lent.sort, onChange: lent.setSort }}
            months={
              first === undefined || first.months.length === 0
                ? undefined
                : { months: first.months, monthOf: (row) => row.entry_date.slice(0, 7) }
            }
            empty={
              <EmptyState
                icon={HandCoins}
                title={<Localized id="cashbook-lent-empty" />}
                help={<Localized id="cashbook-lent-empty-help" />}
              />
            }
            totals={
              first === undefined
                ? undefined
                : (context) => ({
                    labelField: 'reason',
                    values: {
                      reason: context.text('glossary-meezan'),
                      amount: context.money(first.totals.lent_paisa),
                      returned: context.money(first.totals.returned_paisa),
                      open: context.money(first.totals.open_paisa),
                    },
                  })
            }
          />
          {windowed ? <ListWindowBar window={listWindow} /> : null}
        </>
      )}
    </Card>
  );
}
