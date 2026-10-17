// A seller's payable postings, oldest first, under the form (PRD FR-7.59): the date, where the
// credit came from (a بکری, his opening credit, a جنرل credit), the credit, the advance already
// adjusted against it, what was paid and what is still open. "Paid" and "still open" are worked
// out by the backend when shown, oldest first. Nothing is added up here (CLAUDE.md rule 1).

import { Localized, useLocalization } from '@fluent/react';
import { isoToDisplayDate, isPositivePaisa } from '../../../shared/format';
import { Num, Table, type TableColumn } from '../../../shared/ui';
import { useOpenPayables } from '../hooks';
import type { PayablePosting } from '../types';
import { PanelShell } from './PanelShell';
import { SourceLabel } from './SourceLabel';

interface PayablesPanelProps {
  sellerId: number;
  personKey: string | null;
}

export function PayablesPanel({ sellerId, personKey }: PayablesPanelProps) {
  const { l10n } = useLocalization();
  const query = useOpenPayables(sellerId, personKey);
  return (
    <PanelShell
      query={query}
      title={<Localized id="vouchers-payables-title" />}
      failed={<Localized id="vouchers-payables-failed" />}
    >
      {(data) => {
        const columns: TableColumn<PayablePosting>[] = [
          {
            id: 'date',
            header: <Localized id="vouchers-col-date" />,
            kind: 'number',
            cell: (posting) => <Num>{isoToDisplayDate(posting.entry_date)}</Num>,
          },
          {
            id: 'source',
            header: <Localized id="vouchers-col-source" />,
            cell: (posting) => <SourceLabel source={posting.source} number={posting.number} />,
          },
          {
            id: 'credit',
            header: <Localized id="vouchers-col-credit" />,
            kind: 'number',
            cell: (posting) => <Num paisa={posting.credit_paisa} />,
          },
          {
            id: 'adjusted',
            header: <Localized id="vouchers-col-adjusted" />,
            kind: 'number',
            cell: (posting) => <Num paisa={posting.advance_adjusted_paisa} />,
          },
          {
            id: 'paid',
            header: <Localized id="vouchers-col-paid" />,
            kind: 'number',
            cell: (posting) => <Num paisa={posting.paid_paisa} />,
          },
          {
            id: 'open',
            header: <Localized id="vouchers-col-open" />,
            kind: 'number',
            cell: (posting) => <Num paisa={posting.still_open_paisa} />,
          },
        ];
        return (
          <>
            <p className="flex flex-wrap items-baseline gap-x-6 gap-y-1 type-body">
              <span className="flex items-baseline gap-2">
                <Localized id="vouchers-payables-open" />
                <Num paisa={data.open_paisa} />
              </span>
              {isPositivePaisa(data.advances_open_paisa) ? (
                <span className="flex items-baseline gap-2">
                  <Localized id="vouchers-advances-open" />
                  <Num paisa={data.advances_open_paisa} />
                </span>
              ) : null}
              {isPositivePaisa(data.owes_firm_paisa) ? (
                <span className="flex items-baseline gap-2 text-due">
                  <Localized id="vouchers-payables-owes" />
                  <Num paisa={data.owes_firm_paisa} />
                </span>
              ) : null}
            </p>
            {data.postings.length === 0 ? (
              <Localized id="vouchers-payables-none">
                <p className="type-body text-ink-muted" />
              </Localized>
            ) : (
              <>
                {data.truncated ? (
                  // The backend sends at most the oldest 200; the totals are for all of them.
                  <Localized
                    id="vouchers-truncated-note"
                    vars={{ shown: String(data.postings.length), total: String(data.total_count) }}
                  >
                    <p className="type-body text-ink-muted" />
                  </Localized>
                ) : null}
                <Table
                  label={l10n.getString('vouchers-payables-title')}
                  columns={columns}
                  rows={data.postings}
                  rowKey={(posting) => String(data.postings.indexOf(posting))}
                />
              </>
            )}
          </>
        );
      }}
    </PanelShell>
  );
}
