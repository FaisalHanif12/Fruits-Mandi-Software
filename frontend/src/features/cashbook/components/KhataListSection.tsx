// The list under the الگ کھاتہ form (PRD FR-16.6, FR-19.11): the chosen khata's entries in date
// order with the reason, the signed amount and the running balance, page by page, with the opening
// and closing balance and the backend's totals over the whole list. Plus means the shop holds money
// for the khata, minus means it has taken more than it brought. Four states: loading, an error with
// a retry, nothing yet, and the rows. Without a khata it says to choose one.

import { Localized } from '@fluent/react';
import { Wallet } from 'lucide-react';
import { uiErrorOf } from '../../../shared/api';
import { PagedList } from '../../../shared/grid';
import { Card, EmptyState, Fact } from '../../../shared/ui';
import { useKhataStatement } from '../hooks';
import { KHATA_COLUMNS } from '../list-columns';
import { SignedNum } from './SignedNum';

export function KhataListSection({ khataId }: { khataId: number | null }) {
  const statement = useKhataStatement(khataId, true);
  const first = statement.data?.pages[0];
  const rows = statement.data?.pages.flatMap((page) => page.rows);
  return (
    <Card compact title={<Localized id="cashbook-khata-title" />}>
      {khataId === null ? (
        <EmptyState
          icon={Wallet}
          title={<Localized id="cashbook-khata-pick" />}
          help={<Localized id="cashbook-khata-pick-help" />}
        />
      ) : (
        <>
          {first === undefined ? null : (
            <dl className="grid grid-cols-2 gap-3">
              <Fact label={<Localized id="cashbook-khata-opening" />}>
                <SignedNum paisa={first.opening_paisa} />
              </Fact>
              <Fact label={<Localized id="cashbook-khata-closing" />}>
                <SignedNum paisa={first.closing_paisa} />
              </Fact>
            </dl>
          )}
          <Localized id="cashbook-khata-sign-hint">
            <p className="type-body text-ink-muted" />
          </Localized>
          <PagedList
            columns={KHATA_COLUMNS}
            rows={rows}
            getRowId={(row) => `${String(row.entry_id)}-${String(row.voucher_id ?? 0)}`}
            labelId="cashbook-khata-label"
            failedId="cashbook-khata-failed"
            error={statement.isError ? uiErrorOf(statement.error) : null}
            onRetry={() => {
              void statement.refetch();
            }}
            pager={statement.pager}
            sort={{ value: statement.sort, onChange: statement.setSort }}
            months={
              first === undefined || first.months.length === 0
                ? undefined
                : { months: first.months, monthOf: (row) => row.entry_date.slice(0, 7) }
            }
            empty={
              <EmptyState
                icon={Wallet}
                title={<Localized id="cashbook-khata-empty" />}
                help={<Localized id="cashbook-khata-empty-help" />}
              />
            }
          />
        </>
      )}
    </Card>
  );
}
