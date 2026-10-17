// A seller's advances under his party card (PRD FR-7.60): how much is open and the date and
// amount of the last one, then each open advance with how much of it is adjusted and how much is
// still open. An advance is a پیشگی voucher or an opening debit marked as an advance. On a
// وصولی from a seller (he returns advance money, FR-15.12) the backend also says what the amount
// typed adjusts (`effects`), and the panel shows it beside each advance. Everything is the
// backend's; nothing is added up here. Four states: loading, error with a retry, nothing yet, and
// the list.

import { Localized, useLocalization } from '@fluent/react';
import { Check } from 'lucide-react';
import { isoToDisplayDate, isPositivePaisa } from '../../../shared/format';
import { Code, Num, Table, type TableColumn } from '../../../shared/ui';
import { useAdvances } from '../hooks';
import type { AdvanceEffect, OpenAdvance } from '../types';
import { NoneMark } from './NoneMark';
import { PanelShell } from './PanelShell';

interface AdvanceHistoryProps {
  sellerId: number;
  /** What the amount typed adjusts, from the preview; null when the screen does not ask. */
  effects?: readonly AdvanceEffect[] | null;
}

/** The same advance in the list and in the effects: by source, number and day. */
function sameAdvance(a: OpenAdvance, b: OpenAdvance): boolean {
  return (
    a.source === b.source && a.voucher_number === b.voucher_number && a.entry_date === b.entry_date
  );
}

export function AdvanceHistory({ sellerId, effects = null }: AdvanceHistoryProps) {
  const { l10n } = useLocalization();
  const query = useAdvances(sellerId);
  return (
    <PanelShell
      query={query}
      title={<Localized id="vouchers-advances-title" />}
      failed={<Localized id="vouchers-advances-failed" />}
    >
      {({ open, open_paisa, last }) => {
        if (open.length === 0 && last === null) {
          return (
            <Localized id="vouchers-advances-none">
              <p className="type-body text-ink-muted" />
            </Localized>
          );
        }
        const columns: TableColumn<OpenAdvance>[] = [
          {
            id: 'date',
            header: <Localized id="vouchers-col-date" />,
            kind: 'number',
            cell: (item) => <Num>{isoToDisplayDate(item.entry_date)}</Num>,
          },
          {
            id: 'number',
            header: <Localized id="vouchers-col-number" />,
            cell: (item) =>
              item.source === 'opening' || item.voucher_number === null ? (
                <Localized id="vouchers-source-opening" />
              ) : (
                <Code>
                  <Localized
                    id="vouchers-number-advance"
                    vars={{ number: String(item.voucher_number) }}
                  />
                </Code>
              ),
          },
          {
            id: 'given',
            header: <Localized id="vouchers-col-amount" />,
            kind: 'number',
            cell: (item) => <Num paisa={item.given_paisa} />,
          },
          {
            id: 'adjusted',
            header: <Localized id="vouchers-col-adjusted" />,
            kind: 'number',
            cell: (item) => <Num paisa={item.adjusted_paisa} />,
          },
          {
            id: 'open',
            header: <Localized id="vouchers-col-open" />,
            kind: 'number',
            cell: (item) => <Num paisa={item.open_paisa} />,
          },
          ...(effects === null
            ? []
            : [
                {
                  id: 'returns',
                  header: <Localized id="vouchers-col-returns" />,
                  cell: (item: OpenAdvance) => {
                    const effect = effects.find((one) => sameAdvance(one.advance, item));
                    if (effect === undefined || !isPositivePaisa(effect.adjusted_paisa)) {
                      return <NoneMark />;
                    }
                    return (
                      <span className="flex flex-wrap items-baseline gap-x-2 text-money-in">
                        <Check aria-hidden="true" className="size-icon-sm self-center" />
                        <Num paisa={effect.adjusted_paisa} />
                      </span>
                    );
                  },
                } satisfies TableColumn<OpenAdvance>,
              ]),
        ];
        return (
          <>
            <p className="flex flex-wrap items-baseline gap-x-6 gap-y-1 type-body">
              <span className="flex items-baseline gap-2">
                <Localized id="vouchers-advances-open" />
                <Num paisa={open_paisa} />
              </span>
              {last === null ? null : (
                <span className="flex items-baseline gap-2">
                  <Localized id="vouchers-advances-last" />
                  <Num>{isoToDisplayDate(last.entry_date)}</Num>
                  <Num paisa={last.amount_paisa} />
                </span>
              )}
            </p>
            {open.length === 0 ? null : (
              <Table
                label={l10n.getString('vouchers-advances-title')}
                columns={columns}
                rows={open}
                rowKey={(item) =>
                  `${item.source}-${String(item.voucher_number)}-${item.entry_date}`
                }
              />
            )}
          </>
        );
      }}
    </PanelShell>
  );
}
