// The chosen khata's account, under the entries of the الگ کھاتہ form (PRD FR-16.6, FR-19.11): its
// entries in date order with the reason, the signed amount and the running balance, page by page, with
// the opening and closing balance. Plus means the shop holds money for the khata, minus means it has
// taken more than it brought. Four states: loading, an error with a retry, nothing yet, and the rows. It
// shows only once a khata is chosen.

import { Localized } from '@fluent/react';
import { Notebook, Wallet } from 'lucide-react';
import { uiErrorOf } from '../../../shared/api';
import { PagedList } from '../../../shared/grid';
import { useListWindow } from '../../../shared/hooks';
import { Card, EmptyState, Fact, ListWindowBar } from '../../../shared/ui';
import { useDayStatus } from '../../../app/use-day-status';
import { useKhataStatement } from '../hooks';
import { KHATA_COLUMNS } from '../list-columns';
import { useOpenVoucher } from '../../../shared/edit';
import { KhataLinkCard } from './KhataLinkCard';
import { SignedNum } from './SignedNum';

export function KhataListSection({ khataId }: { khataId: number }) {
  const open = useOpenVoucher();
  const dayStatus = useDayStatus();
  const today = dayStatus.data?.today;
  // The khata's entries show one month at a time, the latest first (PRD FR-19.11, plan 76 point 8).
  const listWindow = useListWindow({
    kind: 'month',
    source: { kind: 'khata', account_id: khataId },
    today,
  });
  // Without the day (its read failed) the list asks for the whole history, as before the window.
  const windowed = !dayStatus.isError;
  const statement = useKhataStatement(
    khataId,
    true,
    windowed ? { from: listWindow.range.from, to: listWindow.range.to } : { from: null, to: null },
    listWindow.ready || !windowed,
  );
  const first = statement.data?.pages[0];
  const rows = statement.data?.pages.flatMap((page) => page.rows);
  return (
    <Card compact icon={Notebook} title={<Localized id="cashbook-khata-title" />}>
      {first === undefined ? null : <KhataLinkCard khata={first.khata} />}
      {first === undefined ? null : (
        <dl className="grid grid-cols-2 gap-3">
          <Fact stat label={<Localized id="cashbook-khata-opening" />}>
            <SignedNum inherit paisa={first.opening_paisa} />
          </Fact>
          <Fact stat label={<Localized id="cashbook-khata-closing" />}>
            <SignedNum inherit paisa={first.closing_paisa} />
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
        // A payment or a receipt opens its voucher; a sale of the khata's goods has none.
        onOpen={(row) => {
          if (row.voucher_id === null) return;
          if (row.kind === 'paid_out') open({ id: row.voucher_id, voucher_type: 'khata_out' });
          else if (row.kind === 'received') {
            open({ id: row.voucher_id, voucher_type: 'khata_in' });
          }
        }}
        labelId="cashbook-khata-label"
        failedId="cashbook-khata-failed"
        error={statement.isError ? uiErrorOf(statement.error) : null}
        onRetry={() => {
          void statement.refetch();
        }}
        pager={statement.pager}
        onPageKey={listWindow.onPageKey}
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
      {windowed ? <ListWindowBar window={listWindow} /> : null}
    </Card>
  );
}
