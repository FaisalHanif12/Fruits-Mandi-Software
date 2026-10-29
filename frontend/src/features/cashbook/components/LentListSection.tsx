// The list under the ادھار رقم form (PRD FR-16.5, FR-19.11): the chosen person's loans, oldest
// first, with what was returned and what is still open, his balance, and the backend's totals over
// the whole list. Four states: loading, an error with a retry, nothing yet, and the rows. Without
// a person it says to choose one.

import { Localized } from '@fluent/react';
import { HandCoins } from 'lucide-react';
import { uiErrorOf } from '../../../shared/api';
import { PagedList } from '../../../shared/grid';
import { Balance, Card, EmptyState } from '../../../shared/ui';
import { useLentList } from '../hooks';
import { LENT_COLUMNS } from '../list-columns';

export function LentListSection({ personId }: { personId: number | null }) {
  const lent = useLentList(personId);
  const first = lent.data?.pages[0];
  const rows = lent.data?.pages.flatMap((page) => page.rows);
  return (
    <Card compact title={<Localized id="cashbook-lent-title" />}>
      {personId === null ? (
        <EmptyState
          icon={HandCoins}
          title={<Localized id="cashbook-lent-pick" />}
          help={<Localized id="cashbook-lent-pick-help" />}
        />
      ) : (
        <>
          {first === undefined ? null : (
            <p className="flex items-baseline gap-2 type-body">
              <Localized id="cashbook-lent-balance" />
              <Balance balance={first.balance} />
            </p>
          )}
          <PagedList
            columns={LENT_COLUMNS}
            rows={rows}
            getRowId={(row) => String(row.voucher_id)}
            labelId="cashbook-lent-label"
            failedId="cashbook-lent-failed"
            error={lent.isError ? uiErrorOf(lent.error) : null}
            onRetry={() => {
              void lent.refetch();
            }}
            pager={lent.pager}
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
        </>
      )}
    </Card>
  );
}
