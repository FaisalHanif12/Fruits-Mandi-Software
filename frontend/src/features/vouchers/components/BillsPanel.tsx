// A buyer's open bills, oldest first, under the form (PRD FR-7.58, FR-15.4): the date, where the bill
// came from, its amount, how much is still open and its age. While an amount is typed, the backend
// says which bills it settles (`bill_effects`), and the panel marks them in words and numbers. The
// panel shows what the backend sent and works out nothing (CLAUDE.md rule 1).

import { Localized, useLocalization } from '@fluent/react';
import { Check } from 'lucide-react';
import { isoToDisplayDate, isPositivePaisa } from '../../../shared/format';
import { Num, Table, type TableColumn } from '../../../shared/ui';
import { useOpenBills } from '../hooks';
import type { BillEffect, OpenBill } from '../types';
import { NoneMark } from './NoneMark';
import { PanelShell } from './PanelShell';
import { SourceLabel } from './SourceLabel';

interface BillsPanelProps {
  accountId: number;
  /** The one-time person, for a پختہ کھاتہ account. */
  personKey: string | null;
  /** What the amount typed settles, from the preview; null while there is none. */
  effects: readonly BillEffect[] | null;
}

export function BillsPanel({ accountId, personKey, effects }: BillsPanelProps) {
  const { l10n } = useLocalization();
  const query = useOpenBills(accountId, personKey);
  return (
    <PanelShell
      query={query}
      title={<Localized id="vouchers-bills-title" />}
      failed={<Localized id="vouchers-bills-failed" />}
    >
      {(data) => {
        if (data.bills.length === 0) {
          return (
            <Localized id="vouchers-bills-none">
              <p className="type-body text-ink-muted" />
            </Localized>
          );
        }
        const effectOf = (bill: OpenBill): BillEffect | undefined =>
          effects?.find((effect) => effect.bill.line_id === bill.line_id);
        const columns: TableColumn<OpenBill>[] = [
          {
            id: 'date',
            header: <Localized id="vouchers-col-date" />,
            kind: 'number',
            size: 'date',
            cell: (bill) => <Num>{isoToDisplayDate(bill.entry_date)}</Num>,
          },
          {
            id: 'source',
            header: <Localized id="vouchers-col-source" />,
            cell: (bill) => <SourceLabel source={bill.source} number={bill.number} />,
          },
          {
            id: 'amount',
            header: <Localized id="vouchers-col-bill" />,
            kind: 'number',
            size: 'money',
            cell: (bill) => <Num paisa={bill.amount_paisa} />,
          },
          {
            id: 'open',
            header: <Localized id="vouchers-col-open" />,
            kind: 'number',
            size: 'money',
            cell: (bill) => <Num paisa={bill.open_paisa} />,
          },
          {
            id: 'days',
            header: <Localized id="vouchers-col-days" />,
            kind: 'number',
            size: 'qty',
            cell: (bill) => <Num>{String(bill.days)}</Num>,
          },
          ...(effects === null
            ? []
            : [
                {
                  id: 'settles',
                  header: <Localized id="vouchers-col-settles" />,
                  cell: (bill: OpenBill) => {
                    const effect = effectOf(bill);
                    if (effect === undefined || !isPositivePaisa(effect.settled_paisa)) {
                      return <NoneMark />;
                    }
                    const full = !isPositivePaisa(effect.still_open_paisa);
                    return (
                      <span className="flex flex-wrap items-center gap-x-2 text-money-in">
                        <Check aria-hidden="true" className="size-icon-sm self-center" />
                        <Num paisa={effect.settled_paisa} />
                        <Localized id={full ? 'vouchers-effect-full' : 'vouchers-effect-part'} />
                      </span>
                    );
                  },
                } satisfies TableColumn<OpenBill>,
              ]),
        ];
        return (
          <>
            <p className="flex flex-wrap items-baseline gap-x-6 gap-y-1 type-body">
              <span className="flex items-baseline gap-2">
                <Localized id="vouchers-bills-open" />
                <Num paisa={data.open_paisa} />
              </span>
              {data.oldest_days === null ? null : (
                <span className="flex items-baseline gap-2">
                  <Localized id="vouchers-bills-oldest" />
                  <Localized id="vouchers-days" vars={{ days: String(data.oldest_days) }}>
                    <span />
                  </Localized>
                </span>
              )}
              {isPositivePaisa(data.unmatched_credit_paisa) ? (
                <span className="flex items-baseline gap-2">
                  <Localized id="vouchers-bills-credit" />
                  <Num paisa={data.unmatched_credit_paisa} />
                </span>
              ) : null}
            </p>
            {data.truncated ? (
              // The backend sends at most the oldest 200; the totals above are for all of them.
              <Localized
                id="vouchers-truncated-note"
                vars={{ shown: String(data.bills.length), total: String(data.total_count) }}
              >
                <p className="type-body text-ink-muted" />
              </Localized>
            ) : null}
            <Table
              label={l10n.getString('vouchers-bills-title')}
              columns={columns}
              rows={data.bills}
              rowKey={(bill) => String(bill.line_id)}
            />
          </>
        );
      }}
    </PanelShell>
  );
}
